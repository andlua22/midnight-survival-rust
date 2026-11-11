//! Source inventory state shared by the authority and Bevy client.
//!
//! Players are registered explicitly. The default world has no catalog, and
//! this plugin supplies no starter items, capacities, source frame, or session
//! authentication. Callers provide an explicit body and monotonic receive time.
use crate::protocol::PlayerId;
use bevy::prelude::*;
use inventory_wire::{InventoryBody, read_inventory_body};
use midnight_game_data::{
    DataError,
    source_bag::{SourceBagApply, SourceBagState, SourceInventoryClock},
};
pub use midnight_game_data::{GameplayCatalog, inventory_wire};
use std::{
    collections::{BTreeMap, btree_map::Entry},
    sync::Arc,
};

#[derive(Resource, Clone, Debug, Default)]
pub struct SourceInventoryWorld {
    catalog: Option<Arc<GameplayCatalog>>,
    players: BTreeMap<PlayerId, SourceBagState>,
    revision: u64,
}

impl SourceInventoryWorld {
    pub fn from_catalog(catalog: Arc<GameplayCatalog>) -> Self {
        Self {
            catalog: Some(catalog),
            ..Self::default()
        }
    }

    /// Register an uninitialized bag. Rejoining preserves its existing state.
    pub fn join(&mut self, player: PlayerId) -> bool {
        if self.catalog.is_none() {
            return false;
        }
        let Entry::Vacant(entry) = self.players.entry(player) else {
            return false;
        };
        entry.insert(SourceBagState::default());
        self.revision = self.revision.wrapping_add(1);
        true
    }

    pub fn leave(&mut self, player: PlayerId) -> bool {
        if self.players.remove(&player).is_none() {
            return false;
        }
        self.revision = self.revision.wrapping_add(1);
        true
    }

    pub fn player(&self, player: PlayerId) -> Option<&SourceBagState> {
        self.players.get(&player)
    }

    /// Changes on membership changes or changed bag commits, never on errors
    /// or count alerts. Like a change counter, it wraps after u64::MAX.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn apply_body(
        &mut self,
        player: PlayerId,
        body: &InventoryBody,
        realtime_seconds: f32,
    ) -> Result<SourceBagApply, DataError> {
        let catalog = self
            .catalog
            .as_deref()
            .ok_or_else(|| DataError("source inventory catalog is not configured".into()))?;
        let state = self.players.get_mut(&player).ok_or_else(|| {
            DataError(format!(
                "source inventory player {} is not registered",
                player.0
            ))
        })?;
        let clock = SourceInventoryClock::from_realtime_seconds(realtime_seconds)?;
        // SourceBagState stages and validates the entire change before commit.
        let outcome = state.apply(catalog, body, clock)?;
        if outcome.changed {
            self.revision = self.revision.wrapping_add(1);
        }
        Ok(outcome)
    }

    pub fn apply_bytes(
        &mut self,
        player: PlayerId,
        type_id: i32,
        bytes: &[u8],
        realtime_seconds: f32,
    ) -> Result<SourceBagApply, DataError> {
        // Do not decode data for a disabled world or an unregistered player.
        self.check_player(player)?;
        let body = read_inventory_body(type_id, bytes)?;
        self.apply_body(player, &body, realtime_seconds)
    }

    fn check_player(&self, player: PlayerId) -> Result<(), DataError> {
        if self.catalog.is_none() {
            return Err(DataError(
                "source inventory catalog is not configured".into(),
            ));
        }
        if !self.players.contains_key(&player) {
            return Err(DataError(format!(
                "source inventory player {} is not registered",
                player.0
            )));
        }
        Ok(())
    }
}

/// An explicitly selected source body, after the caller's admission boundary.
#[derive(Message, Clone, Debug)]
pub struct SourceInventoryBodyMessage {
    pub player: PlayerId,
    pub type_id: i32,
    pub body: Vec<u8>,
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct SourceInventoryBodyResult {
    pub player: PlayerId,
    pub result: Result<SourceBagApply, String>,
}

/// Hosts provide TimePlugin and explicitly configure the catalog and players.
#[derive(Clone, Copy, Debug, Default)]
pub struct SourceInventoryPlugin;

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SourceInventorySet {
    Apply,
}

impl Plugin for SourceInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SourceInventoryWorld>()
            .add_message::<SourceInventoryBodyMessage>()
            .add_message::<SourceInventoryBodyResult>()
            .add_systems(
                Update,
                receive_inventory_bodies
                    .in_set(SourceInventorySet::Apply)
                    .run_if(on_message::<SourceInventoryBodyMessage>),
            );
    }
}

fn receive_inventory_bodies(
    mut messages: MessageReader<SourceInventoryBodyMessage>,
    mut results: MessageWriter<SourceInventoryBodyResult>,
    mut inventories: ResMut<SourceInventoryWorld>,
    time: Res<Time<Real>>,
) {
    let realtime_seconds = time.elapsed_secs();
    for message in messages.read() {
        let result = inventories
            .apply_bytes(
                message.player,
                message.type_id,
                &message.body,
                realtime_seconds,
            )
            .map_err(|error| error.to_string());
        results.write(SourceInventoryBodyResult {
            player: message.player,
            result,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        ecs::message::MessageCursor,
        time::{TimePlugin, TimeUpdateStrategy},
    };
    use midnight_game_data::inventory_wire::{
        BagSizeChange, ItemNumberChange, ItemsSnapshot, S_BAG_SIZE_CHANGE_TYPE_ID, S_ITEMS_TYPE_ID,
    };
    use std::time::Duration;

    fn empty_snapshot(capacity: i32) -> InventoryBody {
        InventoryBody::Items(ItemsSnapshot {
            bag_items: Vec::new(),
            skin_items: Vec::new(),
            quick_use_items: BTreeMap::new(),
            equip_items: Vec::new(),
            show_equip: false,
            capacity,
        })
    }

    // Empty SItems body with source masks 123/14/7/63/81/106, capacity 28.
    const EMPTY_SNAPSHOT_BODY: &[u8] = &[
        0x7b, 0x7b, 0x7b, 0x7b, 0x0e, 0x0e, 0x0e, 0x0e, 0x07, 0x07, 0x07, 0x07, 0x3f, 0x3f, 0x3f,
        0x3f, 0x51, 0x76, 0x6a, 0x6a, 0x6a,
    ];

    #[test]
    fn independent_worlds_apply_the_same_source_state_and_ignore_noop_changes() {
        let catalog = Arc::new(GameplayCatalog::default());
        let mut first = SourceInventoryWorld::from_catalog(catalog.clone());
        let mut second = SourceInventoryWorld::from_catalog(catalog);
        let player = PlayerId(7);
        for world in [&mut first, &mut second] {
            assert!(world.join(player));
            assert!(!world.join(player));
            assert!(world.player(player).unwrap().snapshot().is_none());
            assert!(
                world
                    .apply_bytes(player, S_ITEMS_TYPE_ID, EMPTY_SNAPSHOT_BODY, 10.25)
                    .unwrap()
                    .changed
            );
            let size = InventoryBody::BagSizeChange(BagSizeChange { size: 32 });
            assert!(world.apply_body(player, &size, 10.25).unwrap().changed);
            let revision = world.revision();
            assert!(!world.apply_body(player, &size, 10.25).unwrap().changed);
            assert_eq!(world.revision(), revision);
            let alert = ItemNumberChange {
                is_bind: false,
                change: -3,
                item_id: 321,
            };
            let outcome = world
                .apply_body(player, &InventoryBody::ItemNumberChange(alert), 10.25)
                .unwrap();
            assert_eq!(outcome.count_alert, Some(alert));
            assert!(!outcome.changed);
            assert_eq!(world.revision(), revision);
            assert_eq!(world.player(player).unwrap().visible_capacity(), Some(32));
        }
        assert_eq!(first.player(player), second.player(player));
        assert_eq!(first.revision(), second.revision());
        let second_revision = second.revision();
        assert!(first.leave(player));
        assert!(!first.leave(player));
        assert!(first.player(player).is_none());
        assert!(second.player(player).is_some());
        assert_eq!(second.revision(), second_revision);
    }

    #[test]
    fn wrong_players_malformed_bodies_and_invalid_clocks_preserve_state_and_revision() {
        let player = PlayerId(7);
        let mut world = SourceInventoryWorld::from_catalog(Arc::new(GameplayCatalog::default()));
        world.join(player);
        world.apply_body(player, &empty_snapshot(28), 1.0).unwrap();
        let before = world.player(player).unwrap().clone();
        let revision = world.revision();
        assert!(
            world
                .apply_body(PlayerId(8), &empty_snapshot(99), 1.0)
                .is_err()
        );
        assert!(
            world
                .apply_bytes(PlayerId(8), S_ITEMS_TYPE_ID, EMPTY_SNAPSHOT_BODY, 1.0)
                .is_err()
        );
        assert!(
            world
                .apply_bytes(player, S_ITEMS_TYPE_ID, &[0], 1.0)
                .is_err()
        );
        assert!(
            world
                .apply_bytes(player, 0, EMPTY_SNAPSHOT_BODY, 1.0)
                .is_err()
        );
        assert!(
            world
                .apply_body(player, &empty_snapshot(99), f32::NAN)
                .is_err()
        );
        assert_eq!(world.player(player), Some(&before));
        assert_eq!(world.revision(), revision);

        let mut disabled = SourceInventoryWorld::default();
        assert!(!disabled.join(player));
        let revision = disabled.revision();
        assert!(
            disabled
                .apply_body(player, &empty_snapshot(28), 0.0)
                .is_err()
        );
        assert!(
            disabled
                .apply_bytes(player, S_ITEMS_TYPE_ID, EMPTY_SNAPSHOT_BODY, 0.0)
                .is_err()
        );
        assert!(disabled.player(player).is_none());
        assert_eq!(disabled.revision(), revision);
    }

    #[test]
    fn bevy_message_path_applies_bodies_and_reports_errors_without_default_players() {
        let mut app = App::new();
        app.add_plugins((TimePlugin, SourceInventoryPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                50,
            )));
        app.update();
        assert!(
            app.world()
                .resource::<SourceInventoryWorld>()
                .player(PlayerId(7))
                .is_none()
        );
        let mut world = SourceInventoryWorld::from_catalog(Arc::new(GameplayCatalog::default()));
        world.join(PlayerId(7));
        app.insert_resource(world);
        let mut cursor = MessageCursor::<SourceInventoryBodyResult>::default();
        for (type_id, body) in [
            (S_ITEMS_TYPE_ID, EMPTY_SNAPSHOT_BODY.to_vec()),
            (S_BAG_SIZE_CHANGE_TYPE_ID, vec![0x53, 0x4d, 0x4d, 0x4d]), // Size 30.
            (S_ITEMS_TYPE_ID, vec![0]),
        ] {
            app.world_mut().write_message(SourceInventoryBodyMessage {
                player: PlayerId(7),
                type_id,
                body,
            });
        }
        app.update();
        let outcomes: Vec<_> = cursor
            .read(
                app.world()
                    .resource::<Messages<SourceInventoryBodyResult>>(),
            )
            .cloned()
            .collect();
        assert_eq!(outcomes.len(), 3);
        assert!(outcomes[0].result.as_ref().unwrap().changed);
        assert!(outcomes[1].result.as_ref().unwrap().changed);
        assert!(outcomes[2].result.is_err());
        let world = app.world().resource::<SourceInventoryWorld>();
        assert_eq!(
            world.player(PlayerId(7)).unwrap().visible_capacity(),
            Some(30)
        );
        assert_eq!(world.revision(), 3); // Join, snapshot, size; error is unchanged.
    }
}
