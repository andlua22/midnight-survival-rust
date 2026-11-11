//! Source inventory projection shared by the client and authority.
//!
//! Rules follow the retained unpatched BagMgr methods. Original frame/session
//! decoding, IFix overrides, expiry callbacks and weapon effects are separate.
//! Positive stored counts, unique IDs within each collection and allocation
//! bounds are reconstruction admission policy, not source capacity rules.
use crate::{
    DataError, GameplayCatalog,
    inventory_wire::{
        BagItem, InventoryBody, ItemChanged, ItemNumberChange, ItemsSnapshot,
        MAX_INVENTORY_BODY_BYTES, MAX_INVENTORY_COLLECTION_ENTRIES, MAX_ITEM_EXTRA_BYTES,
        QuickUseItemChanged,
    },
};
use std::collections::BTreeSet;

/// Unity uses ceil(Time.realtimeSinceStartup), not Unix time or simulation ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceInventoryClock(i32);

impl SourceInventoryClock {
    pub fn from_realtime_seconds(seconds: f32) -> Result<Self, DataError> {
        if !seconds.is_finite() || seconds < 0.0 || seconds.ceil() >= i32::MAX as f32 {
            return Err(DataError("invalid source inventory monotonic clock".into()));
        }
        Ok(Self(seconds.ceil() as i32))
    }

    fn timeout(self, serialized: i32) -> Result<i32, DataError> {
        serialized
            .checked_add(self.0)
            .ok_or_else(|| DataError("source inventory timeout overflow".into()))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceBagApply {
    pub changed: bool,
    /// SItemNumberChange is a notification, not an inventory delta.
    pub count_alert: Option<ItemNumberChange>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBagState {
    snapshot: Option<ItemsSnapshot>,
    hand_instance_id: i32,
}

impl Default for SourceBagState {
    fn default() -> Self {
        Self {
            snapshot: None,
            hand_instance_id: -1,
        }
    }
}

#[derive(Clone, Copy)]
enum Location {
    Bag(usize),
    Quick(i32),
}

impl SourceBagState {
    pub fn snapshot(&self) -> Option<&ItemsSnapshot> {
        self.snapshot.as_ref()
    }

    pub fn hand_instance_id(&self) -> i32 {
        self.hand_instance_id
    }

    /// BagMgr displays max(stored capacity, bag count), without discarding items.
    /// This integer never allocates a capacity-sized array; UI must page it.
    pub fn visible_capacity(&self) -> Option<usize> {
        self.snapshot
            .as_ref()
            .map(|snapshot| snapshot.capacity.max(snapshot.bag_items.len() as i32) as usize)
    }

    pub fn hand_item(&self) -> Option<&BagItem> {
        let snapshot = self.snapshot.as_ref()?;
        match active_location(snapshot, self.hand_instance_id)? {
            Location::Bag(index) => snapshot.bag_items.get(index),
            Location::Quick(index) => snapshot.quick_use_items.get(&index),
        }
    }

    /// Applies a trusted decoded body atomically. DTO values remain unmodified;
    /// this projection converts runtime deadlines using an explicit receive clock.
    pub fn apply(
        &mut self,
        catalog: &GameplayCatalog,
        body: &InventoryBody,
        clock: SourceInventoryClock,
    ) -> Result<SourceBagApply, DataError> {
        let extra_size = match body {
            InventoryBody::ItemChanged(update) => update.extra_info.len(),
            InventoryBody::QuickUseItemChanged(update) => update.extra_info.len(),
            _ => 0,
        };
        if extra_size > MAX_ITEM_EXTRA_BYTES {
            return Err(DataError(
                "source update extra data exceeds admission limit".into(),
            ));
        }
        let mut next = self.clone();
        let mut result = SourceBagApply::default();
        if let InventoryBody::Items(incoming) = body {
            validate_snapshot(incoming, catalog)?;
            let mut snapshot = incoming.clone();
            for item in snapshot
                .bag_items
                .iter_mut()
                .chain(snapshot.quick_use_items.values_mut())
            {
                if catalog.items[&item.item_id].exist_time > 0 {
                    item.timeout = clock.timeout(item.timeout)?;
                }
            }
            sort_bag(&mut snapshot, catalog);
            next.snapshot = Some(snapshot);
            next.hand_instance_id = -1;
        } else {
            let snapshot = next.snapshot.as_mut().ok_or_else(|| {
                DataError("source inventory snapshot required before updates".into())
            })?;
            match body {
                InventoryBody::BagSizeChange(update) => snapshot.capacity = update.size,
                InventoryBody::ItemChanged(update) => apply_item_changed(snapshot, update, clock)?,
                InventoryBody::QuickUseItemChanged(update) => {
                    apply_quick_changed(snapshot, update, clock)?
                }
                InventoryBody::ItemDurationChanged(update) => {
                    if let Some(item) = first_item_mut(snapshot, update.instance_id) {
                        item.duration = update.duration;
                    }
                }
                InventoryBody::ItemNumberChange(update) => result.count_alert = Some(*update),
                InventoryBody::PutToHand(update) => next.hand_instance_id = update.instance_id,
                InventoryBody::SetQuickUseItem(_) => {
                    return Err(DataError(
                        "source quick-slot swap/coffer rules are not implemented".into(),
                    ));
                }
                InventoryBody::SyncBagItem(_) => {
                    return Err(DataError(
                        "source SyncBagItem consumer is not authenticated".into(),
                    ));
                }
                InventoryBody::ClearAllBag => {
                    return Err(DataError(
                        "source clear requires unported weapon/extra-data mutations".into(),
                    ));
                }
                InventoryBody::Items(_) => unreachable!("snapshot handled above"),
            }
            validate_snapshot(snapshot, catalog)?;
            if matches!(
                body,
                InventoryBody::ItemChanged(_) | InventoryBody::QuickUseItemChanged(_)
            ) {
                sort_bag(snapshot, catalog);
            }
        }
        result.changed = next != *self;
        *self = next;
        Ok(result)
    }
}

fn validate_snapshot(snapshot: &ItemsSnapshot, catalog: &GameplayCatalog) -> Result<(), DataError> {
    let mut bytes = 21usize + snapshot.quick_use_items.len() * 4;
    for item in snapshot
        .bag_items
        .iter()
        .chain(&snapshot.skin_items)
        .chain(&snapshot.equip_items)
        .chain(snapshot.quick_use_items.values())
    {
        bytes = bytes
            .checked_add(29)
            .and_then(|n| n.checked_add(item.extra_info.len()))
            .ok_or_else(|| DataError("source inventory size overflow".into()))?;
        if bytes > MAX_INVENTORY_BODY_BYTES {
            return Err(DataError(
                "source inventory state exceeds body-size admission limit".into(),
            ));
        }
    }
    for items in [
        &snapshot.bag_items,
        &snapshot.skin_items,
        &snapshot.equip_items,
    ] {
        validate_collection(items.iter(), catalog)?;
    }
    validate_collection(snapshot.quick_use_items.values(), catalog)
}

fn validate_collection<'a>(
    items: impl Iterator<Item = &'a BagItem>,
    catalog: &GameplayCatalog,
) -> Result<(), DataError> {
    let mut identities = BTreeSet::new();
    for item in items {
        if identities.len() == MAX_INVENTORY_COLLECTION_ENTRIES
            || !identities.insert(item.instance_id)
        {
            return Err(DataError(
                "duplicate instance ID or collection exceeds admission limit".into(),
            ));
        }
        if item.number <= 0 {
            return Err(DataError(
                "nonpositive stored item count rejected by admission policy".into(),
            ));
        }
        if !catalog.items.contains_key(&item.item_id) {
            return Err(DataError(format!(
                "unknown source item definition {}",
                item.item_id
            )));
        }
        if item.extra_info.len() > MAX_ITEM_EXTRA_BYTES {
            return Err(DataError(
                "source item extra data exceeds admission limit".into(),
            ));
        }
        // Community server overrides can raise pile limits. Do not reject an
        // authoritative snapshot using only ItemCfg.max_pile or pileable.
    }
    Ok(())
}

fn sort_bag(snapshot: &mut ItemsSnapshot, catalog: &GameplayCatalog) {
    snapshot.bag_items.sort_by_key(|item| {
        (
            catalog.items[&item.item_id].order,
            item.item_id,
            item.instance_id,
        )
    });
}

fn active_location(snapshot: &ItemsSnapshot, instance_id: i32) -> Option<Location> {
    snapshot
        .bag_items
        .iter()
        .position(|item| item.instance_id == instance_id)
        .map(Location::Bag)
        .or_else(|| {
            snapshot
                .quick_use_items
                .iter()
                .find(|(_, item)| item.instance_id == instance_id)
                .map(|(&index, _)| Location::Quick(index))
        })
}

fn first_item_mut(snapshot: &mut ItemsSnapshot, instance_id: i32) -> Option<&mut BagItem> {
    if let Some(location) = active_location(snapshot, instance_id) {
        return Some(match location {
            Location::Bag(index) => &mut snapshot.bag_items[index],
            Location::Quick(index) => snapshot.quick_use_items.get_mut(&index)?,
        });
    }
    if let Some(index) = snapshot
        .skin_items
        .iter()
        .position(|item| item.instance_id == instance_id)
    {
        return snapshot.skin_items.get_mut(index);
    }
    snapshot
        .equip_items
        .iter_mut()
        .find(|item| item.instance_id == instance_id)
}

fn remove_active(snapshot: &mut ItemsSnapshot, location: Location) {
    match location {
        Location::Bag(index) => {
            snapshot.bag_items.remove(index);
        }
        Location::Quick(index) => {
            snapshot.quick_use_items.remove(&index);
        }
    }
}

fn remove_equipment_fallback(snapshot: &mut ItemsSnapshot, instance_id: i32, item_id: i32) {
    if snapshot
        .equip_items
        .iter()
        .any(|item| item.item_id == item_id)
        && let Some(index) = snapshot
            .equip_items
            .iter()
            .position(|item| item.instance_id == instance_id)
    {
        snapshot.equip_items.remove(index);
    }
}

fn update_existing(item: &mut BagItem, update: &ItemChanged) {
    item.number = update.num;
    item.extra_info.clone_from(&update.extra_info);
    item.is_bind = update.is_bind;
    item.bind_timeout = update.bind_timeout;
    item.duration = update.duration;
    // Existing item/config identity and normalized deadline stay unchanged.
}

fn new_item(update: &ItemChanged, clock: SourceInventoryClock) -> Result<BagItem, DataError> {
    Ok(BagItem {
        instance_id: update.instance_id,
        item_id: update.item_id,
        number: update.num,
        duration: update.duration,
        timeout: clock.timeout(update.timeout)?,
        is_bind: update.is_bind,
        bind_timeout: update.bind_timeout,
        extra_info: update.extra_info.clone(),
    })
}

fn apply_item_changed(
    snapshot: &mut ItemsSnapshot,
    update: &ItemChanged,
    clock: SourceInventoryClock,
) -> Result<(), DataError> {
    if let Some(location) = active_location(snapshot, update.instance_id) {
        if update.num == 0 {
            remove_active(snapshot, location);
        } else {
            let item = match location {
                Location::Bag(index) => &mut snapshot.bag_items[index],
                Location::Quick(index) => snapshot
                    .quick_use_items
                    .get_mut(&index)
                    .ok_or_else(|| DataError("quick item disappeared".into()))?,
            };
            update_existing(item, update);
        }
    } else if update.num != 0 {
        snapshot.bag_items.push(new_item(update, clock)?);
    } else {
        remove_equipment_fallback(snapshot, update.instance_id, update.item_id);
    }
    Ok(())
}

fn apply_quick_changed(
    snapshot: &mut ItemsSnapshot,
    update: &QuickUseItemChanged,
    clock: SourceInventoryClock,
) -> Result<(), DataError> {
    let fields = ItemChanged {
        bind_timeout: update.bind_timeout,
        is_bind: update.is_bind,
        timeout: update.timeout,
        item_id: update.item_id,
        duration: update.duration,
        instance_id: update.instance_id,
        extra_info: update.extra_info.clone(),
        num: update.num,
    };
    match active_location(snapshot, update.instance_id) {
        Some(location) if update.num == 0 => remove_active(snapshot, location),
        Some(Location::Bag(index)) => {
            let mut item = snapshot.bag_items.remove(index);
            update_existing(&mut item, &fields);
            snapshot.quick_use_items.insert(update.index, item);
        }
        Some(Location::Quick(index)) => {
            if let Some(item) = snapshot.quick_use_items.get_mut(&index) {
                update_existing(item, &fields);
            }
            // Native known-quick branch preserves its key, ignoring msg.index.
        }
        None if update.num != 0 => {
            snapshot
                .quick_use_items
                .insert(update.index, new_item(&fields, clock)?);
        }
        None => remove_equipment_fallback(snapshot, update.instance_id, update.item_id),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory_wire::{BagSizeChange, ItemDurationChanged, PutToHand};
    use std::collections::BTreeMap;

    fn fixture_item(instance_id: i32, item_id: i32) -> BagItem {
        BagItem {
            instance_id,
            item_id,
            number: 17,
            duration: -4,
            timeout: 20,
            is_bind: false,
            bind_timeout: -7,
            extra_info: vec![0, 255, 42],
        }
    }

    fn fixture_snapshot() -> ItemsSnapshot {
        ItemsSnapshot {
            bag_items: vec![fixture_item(12, 50002), fixture_item(11, 50001)],
            skin_items: vec![fixture_item(21, 60001)],
            quick_use_items: BTreeMap::from([(8, fixture_item(31, 50001))]),
            equip_items: vec![fixture_item(41, 60001)],
            show_equip: true,
            capacity: 1,
        }
    }

    fn change(instance_id: i32, num: i32) -> ItemChanged {
        ItemChanged {
            bind_timeout: 33,
            is_bind: true,
            timeout: 999,
            item_id: 50002,
            duration: 72,
            instance_id,
            extra_info: vec![9, 0, 8],
            num,
        }
    }

    fn clock() -> SourceInventoryClock {
        SourceInventoryClock::from_realtime_seconds(10.25).unwrap()
    }

    fn initialized(catalog: &GameplayCatalog) -> SourceBagState {
        let mut state = SourceBagState::default();
        state
            .apply(catalog, &InventoryBody::Items(fixture_snapshot()), clock())
            .unwrap();
        state
    }

    #[test]
    fn snapshot_uses_config_identity_order_and_runtime_clock_without_stack_or_capacity_loss() {
        let mut catalog = crate::tests::catalog();
        catalog.items.get_mut(&50002).unwrap().order = -1;
        catalog.items.get_mut(&50001).unwrap().exist_time = 1;
        let original = fixture_snapshot();
        let mut state = SourceBagState::default();
        state
            .apply(&catalog, &InventoryBody::Items(original.clone()), clock())
            .unwrap();
        let snapshot = state.snapshot().unwrap();
        assert_eq!(snapshot.bag_items[0].instance_id, 12);
        assert_eq!(snapshot.bag_items[1].timeout, 31);
        assert_eq!(snapshot.quick_use_items[&8].timeout, 31);
        assert_eq!(snapshot.skin_items[0].timeout, 20);
        assert_eq!(snapshot.equip_items[0].timeout, 20);
        assert_eq!(state.visible_capacity(), Some(2));
        assert_eq!(snapshot.bag_items[1].number, 17); // Source table max is only 5.
        assert_eq!(original.bag_items[1].timeout, 20); // Raw DTO remains unchanged.
        state
            .apply(
                &catalog,
                &InventoryBody::BagSizeChange(BagSizeChange { size: 0 }),
                clock(),
            )
            .unwrap();
        assert_eq!(state.visible_capacity(), Some(2));
        assert_eq!(state.snapshot().unwrap().bag_items.len(), 2);
    }

    #[test]
    fn count_updates_are_absolute_preserve_existing_identity_and_deadline_and_create_with_receive_clock()
     {
        let catalog = crate::tests::catalog();
        let mut state = initialized(&catalog);
        state
            .apply(
                &catalog,
                &InventoryBody::ItemChanged(change(11, 3)),
                clock(),
            )
            .unwrap();
        let item = state
            .snapshot()
            .unwrap()
            .bag_items
            .iter()
            .find(|i| i.instance_id == 11)
            .unwrap();
        assert_eq!(
            (item.item_id, item.number, item.timeout, item.duration),
            (50001, 3, 20, 72)
        );
        assert_eq!(item.extra_info, [9, 0, 8]);
        state
            .apply(
                &catalog,
                &InventoryBody::ItemChanged(change(55, 8)),
                clock(),
            )
            .unwrap();
        let item = state
            .snapshot()
            .unwrap()
            .bag_items
            .iter()
            .find(|i| i.instance_id == 55)
            .unwrap();
        assert_eq!((item.item_id, item.number, item.timeout), (50002, 8, 1010));
        state
            .apply(
                &catalog,
                &InventoryBody::ItemChanged(change(11, 0)),
                clock(),
            )
            .unwrap();
        assert!(
            state
                .snapshot()
                .unwrap()
                .bag_items
                .iter()
                .all(|i| i.instance_id != 11)
        );
    }

    #[test]
    fn quick_updates_move_bag_entries_but_preserve_existing_quick_key() {
        let catalog = crate::tests::catalog();
        let mut state = initialized(&catalog);
        let quick = |instance_id, index, num| {
            InventoryBody::QuickUseItemChanged(QuickUseItemChanged {
                timeout: 200,
                index,
                is_bind: true,
                instance_id,
                bind_timeout: 4,
                duration: 8,
                num,
                item_id: 50002,
                extra_info: vec![1, 2],
            })
        };
        state.apply(&catalog, &quick(11, 2, 5), clock()).unwrap();
        let snapshot = state.snapshot().unwrap();
        assert!(snapshot.bag_items.iter().all(|i| i.instance_id != 11));
        assert_eq!(snapshot.quick_use_items[&2].item_id, 50001);
        state.apply(&catalog, &quick(11, 99, 2), clock()).unwrap();
        assert!(!state.snapshot().unwrap().quick_use_items.contains_key(&99));
        assert_eq!(state.snapshot().unwrap().quick_use_items[&2].number, 2);
        state.apply(&catalog, &quick(11, 77, 0), clock()).unwrap();
        assert!(!state.snapshot().unwrap().quick_use_items.contains_key(&2));
    }

    #[test]
    fn duration_searches_each_compartment_and_hand_and_alert_do_not_move_items() {
        let catalog = crate::tests::catalog();
        let mut state = initialized(&catalog);
        for id in [11, 31, 21, 41] {
            state
                .apply(
                    &catalog,
                    &InventoryBody::ItemDurationChanged(ItemDurationChanged {
                        duration: 99,
                        instance_id: id,
                    }),
                    clock(),
                )
                .unwrap();
        }
        let snapshot = state.snapshot().unwrap();
        assert!(
            snapshot
                .bag_items
                .iter()
                .filter(|i| i.instance_id == 11)
                .all(|i| i.duration == 99)
        );
        assert_eq!(snapshot.quick_use_items[&8].duration, 99);
        assert_eq!(snapshot.skin_items[0].duration, 99);
        assert_eq!(snapshot.equip_items[0].duration, 99);
        state
            .apply(
                &catalog,
                &InventoryBody::PutToHand(PutToHand { instance_id: 31 }),
                clock(),
            )
            .unwrap();
        assert_eq!(state.hand_item().unwrap().instance_id, 31);
        let old = state.clone();
        let alert = ItemNumberChange {
            is_bind: true,
            change: -3,
            item_id: 50001,
        };
        let outcome = state
            .apply(&catalog, &InventoryBody::ItemNumberChange(alert), clock())
            .unwrap();
        assert!(!outcome.changed);
        assert_eq!(outcome.count_alert, Some(alert));
        assert_eq!(state, old);
    }

    #[test]
    fn failures_and_unsupported_rules_leave_the_previous_inventory_unchanged() {
        let catalog = crate::tests::catalog();
        let mut state = initialized(&catalog);
        let old = state.clone();
        let mut duplicate = fixture_snapshot();
        duplicate.bag_items.push(duplicate.bag_items[0].clone());
        for body in [
            InventoryBody::Items(duplicate),
            InventoryBody::ItemChanged(change(88, -1)),
            InventoryBody::ClearAllBag,
            InventoryBody::SetQuickUseItem(crate::inventory_wire::SetQuickUseItem {
                index: 0,
                instance_id: 11,
            }),
        ] {
            assert!(state.apply(&catalog, &body, clock()).is_err());
            assert_eq!(state, old);
        }
        let mut unknown = change(99, 1);
        unknown.item_id = i32::MAX;
        assert!(
            state
                .apply(&catalog, &InventoryBody::ItemChanged(unknown), clock())
                .is_err()
        );
        assert_eq!(state, old);
        let mut overflow = change(99, 1);
        overflow.timeout = i32::MAX;
        assert!(
            state
                .apply(&catalog, &InventoryBody::ItemChanged(overflow), clock())
                .is_err()
        );
        assert_eq!(state, old);
        assert!(
            SourceBagState::default()
                .apply(
                    &catalog,
                    &InventoryBody::ItemChanged(change(11, 3)),
                    clock()
                )
                .is_err()
        );
        for value in [f32::NAN, f32::INFINITY, -1.0, i32::MAX as f32] {
            assert!(SourceInventoryClock::from_realtime_seconds(value).is_err());
        }
    }

    #[test]
    fn absent_active_zero_updates_remove_equipment_only_after_the_source_config_id_gate() {
        let catalog = crate::tests::catalog();
        let mut state = initialized(&catalog);
        state
            .apply(
                &catalog,
                &InventoryBody::ItemChanged(change(41, 0)),
                clock(),
            )
            .unwrap();
        assert_eq!(state.snapshot().unwrap().equip_items.len(), 1); // Wrong itemId.
        let mut remove = change(41, 0);
        remove.item_id = 60001;
        state
            .apply(
                &catalog,
                &InventoryBody::ItemChanged(remove.clone()),
                clock(),
            )
            .unwrap();
        assert!(state.snapshot().unwrap().equip_items.is_empty());
        assert_eq!(state.snapshot().unwrap().skin_items.len(), 1);
        let mut state = initialized(&catalog);
        state
            .apply(
                &catalog,
                &InventoryBody::QuickUseItemChanged(QuickUseItemChanged {
                    timeout: remove.timeout,
                    index: 88,
                    is_bind: remove.is_bind,
                    instance_id: remove.instance_id,
                    bind_timeout: remove.bind_timeout,
                    duration: remove.duration,
                    num: 0,
                    item_id: remove.item_id,
                    extra_info: remove.extra_info,
                }),
                clock(),
            )
            .unwrap();
        assert!(state.snapshot().unwrap().equip_items.is_empty());
        assert_eq!(state.snapshot().unwrap().quick_use_items.len(), 1);
    }

    #[test]
    fn alert_and_size_handlers_do_not_reorder_items_after_a_catalog_order_change() {
        let mut catalog = crate::tests::catalog();
        let mut state = initialized(&catalog);
        let old = state.clone();
        catalog.items.get_mut(&50002).unwrap().order = -100;
        let alert = InventoryBody::ItemNumberChange(ItemNumberChange {
            is_bind: false,
            change: 2,
            item_id: 50002,
        });
        assert!(!state.apply(&catalog, &alert, clock()).unwrap().changed);
        assert_eq!(state, old);
        state
            .apply(
                &catalog,
                &InventoryBody::BagSizeChange(BagSizeChange { size: 42 }),
                clock(),
            )
            .unwrap();
        assert_eq!(
            state.snapshot().unwrap().bag_items,
            old.snapshot().unwrap().bag_items
        );
    }
}
