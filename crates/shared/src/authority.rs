use std::collections::BTreeMap;

use crate::{level::MapId, physics::CollisionWorld, protocol::*, survival::SurvivalWorld};

pub struct InputQueue {
    pub epoch: u32,
    pub finalized: ServerTick,
    pending: BTreeMap<ServerTick, InputFrame>,
    held: InputFrame,
    last_actual: ServerTick,
}

impl InputQueue {
    pub fn new(tick: ServerTick, state: PlayerState) -> Self {
        Self {
            epoch: 1,
            finalized: tick,
            pending: BTreeMap::new(),
            held: InputFrame::neutral(1, tick, state.motion),
            last_actual: tick,
        }
    }

    pub fn insert(&mut self, frames: &[InputFrame]) -> Result<(), &'static str> {
        if frames.len() > INPUT_REDUNDANCY {
            return Err("too many input frames");
        }
        for frame in frames {
            let frame = frame.validate()?;
            if frame.epoch != self.epoch
                || frame.tick <= self.finalized
                || frame.tick.0 > self.finalized.0.saturating_add(INPUT_WINDOW)
            {
                continue;
            }
            self.pending.entry(frame.tick).or_insert(frame);
        }
        Ok(())
    }

    pub fn finish(&mut self, tick: ServerTick) -> InputFrame {
        assert!(
            tick > self.finalized,
            "one input per player per server tick"
        );
        self.finalized = tick;
        self.pending.retain(|&t, _| t >= tick);
        if let Some(frame) = self.pending.remove(&tick) {
            self.held = frame;
            self.last_actual = tick;
            return frame;
        }
        let mut substitute = self.held;
        substitute.tick = tick;
        substitute.jump = false;
        if tick.0.saturating_sub(self.last_actual.0) > INPUT_HOLD_TICKS {
            substitute.movement = [0.0; 2];
            substitute.sprint = false;
        }
        substitute
    }

    pub fn reset(&mut self, tick: ServerTick, state: PlayerState) {
        let epoch = self.epoch.wrapping_add(1);
        *self = Self::new(tick, state);
        self.epoch = epoch;
        self.held.epoch = epoch;
    }
}

pub struct Authority {
    pub descriptor: crate::world::WorldDescriptor,
    pub gameplay: Option<std::sync::Arc<midnight_game_data::GameplayCatalog>>,
    pub source_inventory: crate::source_inventory::SourceInventoryWorld,
    pub world: CollisionWorld,
    pub survival: SurvivalWorld,
    pub tick: ServerTick,
    pub inputs: BTreeMap<PlayerId, InputQueue>,
    capacity: usize,
    next_id: u64,
}

impl Authority {
    pub fn new(map: MapId, capacity: usize) -> Self {
        Self {
            descriptor: crate::world::WorldDescriptor::fixture(map),
            gameplay: None,
            source_inventory: crate::source_inventory::SourceInventoryWorld::default(),
            world: CollisionWorld::new(map),
            survival: SurvivalWorld::new(map),
            tick: ServerTick(0),
            inputs: BTreeMap::new(),
            capacity: capacity.min(MAX_PLAYERS),
            next_id: 1,
        }
    }

    pub fn from_loaded_world(
        loaded: &crate::world::LoadedWorld,
        world: CollisionWorld,
        capacity: usize,
    ) -> Self {
        let mut authority = Self::new(loaded.descriptor.map, capacity);
        authority.descriptor = loaded.descriptor.clone();
        authority.gameplay = loaded.gameplay.clone();
        authority.source_inventory = loaded
            .gameplay
            .as_ref()
            .map(|catalog| {
                crate::source_inventory::SourceInventoryWorld::from_catalog(catalog.clone())
            })
            .unwrap_or_default();
        authority.world = world;
        authority
    }

    pub fn join(&mut self) -> Result<(PlayerId, PlayerState), &'static str> {
        if self.inputs.len() >= self.capacity {
            return Err("Сервер заповнений");
        }
        let position = self
            .world
            .spawn_position(None)
            .ok_or("Немає вільного місця для появи")?;
        let id = PlayerId(self.next_id);
        self.next_id += 1;
        let state = PlayerState::at(position);
        self.world.set_player(id, state);
        self.inputs.insert(id, InputQueue::new(self.tick, state));
        self.survival.join(id);
        self.source_inventory.join(id);
        Ok((id, state))
    }

    pub fn leave(&mut self, id: PlayerId) {
        self.inputs.remove(&id);
        self.world.remove_player(id);
        self.survival.leave(id);
        self.source_inventory.leave(id);
    }

    /// Trusted source-body adapter boundary. Original packet authentication and
    /// frame/session decoding belong to the caller, not the fixture QUIC codec.
    pub fn apply_source_inventory_body(
        &mut self,
        player: PlayerId,
        type_id: i32,
        body: &[u8],
        realtime_seconds: f32,
    ) -> Result<midnight_game_data::source_bag::SourceBagApply, midnight_game_data::DataError> {
        if !self.inputs.contains_key(&player) {
            return Err(midnight_game_data::DataError(
                "inventory player is not admitted".into(),
            ));
        }
        self.source_inventory
            .apply_bytes(player, type_id, body, realtime_seconds)
    }

    pub fn step(&mut self) {
        self.tick.0 += 1;
        let mut sprinting = Vec::with_capacity(self.inputs.len());
        for (&id, queue) in &mut self.inputs {
            let mut frame = queue.finish(self.tick);
            if !self.survival.can_sprint(id) {
                frame.sprint = false;
            }
            let state = self.world.state(id).expect("admitted player exists");
            let updated = self
                .world
                .step(id, frame.command(state.motion))
                .unwrap_or(state);
            sprinting.push((
                id,
                frame.sprint && frame.movement != [0.0; 2],
                updated.position,
            ));
        }
        let prev_struct_count = self.survival.structures.len();
        let (_respawned_nodes, defeated) = self.survival.step_with_positions(self.tick, sprinting);
        if self.survival.structures.len() != prev_struct_count {
            self.world.sync_structures(&self.survival.structures);
        }
        for id in defeated {
            if let Some(spawn) = self.world.spawn_position(Some(id)) {
                let state = PlayerState::at(spawn);
                self.world.set_player(id, state);
                if let Some(queue) = self.inputs.get_mut(&id) {
                    queue.reset(self.tick, state);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_inventory_requires_admission_and_is_removed_on_player_departure() {
        use crate::source_inventory::SourceInventoryWorld;
        use midnight_game_data::{GameplayCatalog, inventory_wire::S_ITEMS_TYPE_ID};
        let mut authority = Authority::new(MapId::Polygon, 2);
        authority.source_inventory =
            SourceInventoryWorld::from_catalog(std::sync::Arc::new(GameplayCatalog::default()));
        let (player, _) = authority.join().unwrap();
        assert!(
            authority
                .source_inventory
                .player(player)
                .unwrap()
                .snapshot()
                .is_none()
        );
        let body = [
            123, 123, 123, 123, 14, 14, 14, 14, 7, 7, 7, 7, 63, 63, 63, 63, 81, 118, 106, 106, 106,
        ];
        authority
            .apply_source_inventory_body(player, S_ITEMS_TYPE_ID, &body, 1.0)
            .unwrap();
        assert_eq!(
            authority
                .source_inventory
                .player(player)
                .unwrap()
                .visible_capacity(),
            Some(28)
        );
        let outsider = PlayerId(99);
        authority.source_inventory.join(outsider);
        assert!(
            authority
                .apply_source_inventory_body(outsider, S_ITEMS_TYPE_ID, &body, 1.0)
                .is_err()
        );
        assert!(
            authority
                .source_inventory
                .player(outsider)
                .unwrap()
                .snapshot()
                .is_none()
        );
        authority.leave(player);
        assert!(authority.source_inventory.player(player).is_none());
        assert!(
            authority
                .apply_source_inventory_body(player, S_ITEMS_TYPE_ID, &body, 1.0)
                .is_err()
        );
    }

    #[test]
    fn duplicates_late_input_and_gaps_do_not_repeat_jump_or_keep_walking() {
        let mut q = InputQueue::new(ServerTick(0), PlayerState::at([0.0; 3]));
        let mut frame = q.held;
        frame.tick = ServerTick(1);
        frame.jump = true;
        frame.movement = [0.0, 1.0];
        q.insert(&[frame, frame]).unwrap();
        assert!(q.finish(ServerTick(1)).jump);
        for tick in 2..=20 {
            q.insert(&[frame]).unwrap();
            let substitute = q.finish(ServerTick(tick));
            assert!(!substitute.jump);
            if tick > 16 {
                assert_eq!(substitute.movement, [0.0; 2]);
            }
        }
        assert_eq!(q.finalized, ServerTick(20));
        assert!(q.pending.is_empty());
    }

    #[test]
    fn capacity_cleanup_and_clock_do_not_depend_on_clients() {
        let mut server = Authority::new(MapId::Polygon, 150);
        for _ in 0..150 {
            server.join().unwrap();
        }
        assert!(server.join().is_err());
        for _ in 0..3 {
            server.step();
        }
        assert_eq!(server.tick, ServerTick(3));
        server.leave(PlayerId(1));
        assert_eq!(server.join().unwrap().0, PlayerId(151));
    }

    #[test]
    fn reset_rejects_old_epoch_and_nonfinite_input() {
        let state = PlayerState::at([0.0; 3]);
        let mut q = InputQueue::new(ServerTick(0), state);
        let mut old = InputFrame::neutral(1, ServerTick(5), state.motion);
        q.reset(ServerTick(1), state);
        q.insert(&[old]).unwrap();
        assert!(q.pending.is_empty());
        old.yaw = f32::NAN;
        assert!(q.insert(&[old]).is_err());
    }

    #[test]
    fn dense_crowd_of_150_keeps_finite_nonintersecting_capsules() {
        crowd_stress(360);
    }

    #[test]
    #[ignore = "CPU-only 60s simulation stress and timing; run in release with --ignored --nocapture"]
    fn sustained_crowd_of_150() {
        crowd_stress(3600);
    }

    fn crowd_stress(ticks: u64) {
        use crate::movement::{PLAYER_HALF_SEGMENT, PLAYER_RADIUS};
        let mut server = Authority::new(MapId::Polygon, 150);
        for _ in 0..150 {
            server.join().unwrap();
        }
        let mut durations = Vec::new();
        for tick in 1..=ticks {
            let states: Vec<_> = server.world.states().collect();
            for (id, state) in states {
                let dx = -state.position[0];
                let dz = 15.0 - state.position[2];
                let frame = InputFrame {
                    epoch: 1,
                    tick: ServerTick(tick),
                    movement: [0.0, 1.0],
                    yaw: (-dx).atan2(-dz),
                    pitch: 0.0,
                    jump: tick % 180 == id.0 % 180,
                    sprint: true,
                };
                server
                    .inputs
                    .get_mut(&id)
                    .unwrap()
                    .insert(&[frame])
                    .unwrap();
            }
            let started = std::time::Instant::now();
            server.step();
            durations.push(started.elapsed().as_secs_f64() * 1000.0);
            {
                let states: Vec<_> = server.world.states().collect();
                for (i, (id, a)) in states.iter().enumerate() {
                    assert!(a.position.iter().all(|v| v.is_finite()) && a.position[1] > 0.87);
                    for (other, b) in &states[i + 1..] {
                        let dy = ((a.position[1] - b.position[1]).abs()
                            - 2.0 * PLAYER_HALF_SEGMENT)
                            .max(0.0);
                        let distance = ((a.position[0] - b.position[0]).powi(2)
                            + dy * dy
                            + (a.position[2] - b.position[2]).powi(2))
                        .sqrt();
                        assert!(
                            distance >= 2.0 * PLAYER_RADIUS - 0.005,
                            "tick={tick} {id:?}={a:?} {other:?}={b:?} distance={distance}"
                        );
                    }
                }
            }
        }
        durations.sort_by(f64::total_cmp);
        let percentile = |fraction: f64| {
            durations
                [((durations.len() as f64 * fraction).ceil() as usize - 1).min(durations.len() - 1)]
        };
        eprintln!(
            "150-player CPU-only crowd over {ticks} ticks: mean={:.2}ms p50={:.2}ms p95={:.2}ms p99={:.2}ms max={:.2}ms; character queries={:.2}ms collision index={:.2}ms",
            durations.iter().sum::<f64>() / durations.len() as f64,
            percentile(0.5),
            percentile(0.95),
            percentile(0.99),
            durations.last().unwrap(),
            server.world.query_nanos as f64 / 1e6,
            server.world.index_nanos as f64 / 1e6
        );
    }
}
