//! Local replay uses timestamped collision proxies, never the delayed render
//! transforms. The authoritative world and its clock are not rolled back.
use crate::{level::MapId, movement::FIXED_DT, physics::CollisionWorld, protocol::*};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

struct RemoteTrack {
    entered: ServerTick,
    left: Option<ServerTick>,
    samples: VecDeque<(ServerTick, PlayerState)>,
}

impl RemoteTrack {
    fn sample(&self, tick: f64) -> Option<PlayerState> {
        if tick < self.entered.0 as f64 || self.left.is_some_and(|left| tick >= left.0 as f64) {
            return None;
        }
        let &(first_tick, first) = self.samples.front()?;
        if tick <= first_tick.0 as f64 {
            return Some(first);
        }
        for index in 1..self.samples.len() {
            let (b_tick, b) = self.samples[index];
            if tick <= b_tick.0 as f64 {
                let (a_tick, a) = self.samples[index - 1];
                let alpha = ((tick - a_tick.0 as f64) / (b_tick.0 - a_tick.0) as f64) as f32;
                let mut result = b;
                for axis in 0..3 {
                    result.position[axis] =
                        a.position[axis] + (b.position[axis] - a.position[axis]) * alpha;
                }
                let angle = (b.motion.yaw - a.motion.yaw + std::f32::consts::PI)
                    .rem_euclid(std::f32::consts::TAU)
                    - std::f32::consts::PI;
                result.motion.yaw = a.motion.yaw + angle * alpha;
                result.motion.pitch = a.motion.pitch + (b.motion.pitch - a.motion.pitch) * alpha;
                return Some(result);
            }
        }
        let &(last_tick, mut state) = self.samples.back()?;
        let dt = ((tick - last_tick.0 as f64).clamp(0.0, 12.0) as f32) * FIXED_DT;
        for axis in 0..3 {
            if axis != 1 || !state.motion.grounded {
                state.position[axis] += state.velocity[axis] * dt;
            }
        }
        Some(state)
    }

    fn insert(&mut self, tick: ServerTick, state: PlayerState) {
        if tick < self.entered || self.left.is_some_and(|t| tick >= t) {
            return;
        }
        if self
            .samples
            .back()
            .is_some_and(|s| s.0.0.saturating_sub(tick.0) > HISTORY_TICKS)
        {
            return;
        }
        match self.samples.binary_search_by_key(&tick, |s| s.0) {
            Ok(_) => return,
            Err(index) => self.samples.insert(index, (tick, state)),
        }
        let last = self.samples.back().unwrap().0.0;
        while self.samples.len() > 1 && self.samples[1].0.0 + HISTORY_TICKS < last {
            self.samples.pop_front();
        }
        while self.samples.len() > HISTORY_TICKS as usize + 2 {
            self.samples.pop_front();
        }
    }
}

pub struct Prediction {
    pub player: PlayerId,
    pub tick: ServerTick,
    pub epoch: u32,
    pub last_ack: ServerTick,
    pub needs_reset: bool,
    pub corrections: u64,
    pub max_correction: f32,
    world: CollisionWorld,
    history: VecDeque<InputFrame>,
    remotes: BTreeMap<PlayerId, RemoteTrack>,
}

impl Prediction {
    pub fn new(
        map: MapId,
        player: PlayerId,
        tick: ServerTick,
        epoch: u32,
        state: PlayerState,
    ) -> Self {
        let mut world = CollisionWorld::new(map);
        world.set_player(player, state);
        Self {
            player,
            tick,
            epoch,
            last_ack: tick,
            needs_reset: false,
            corrections: 0,
            max_correction: 0.0,
            world,
            history: VecDeque::new(),
            remotes: BTreeMap::new(),
        }
    }

    pub fn state(&self) -> PlayerState {
        self.world.state(self.player).expect("local player exists")
    }

    pub fn sync_structures(&mut self, structures: &[crate::survival::PlacedStructure]) {
        self.world.sync_structures(structures);
    }

    pub fn enter(&mut self, player: PlayerId, tick: ServerTick, state: PlayerState) {
        if player == self.player {
            return;
        }
        if self
            .remotes
            .get(&player)
            .is_some_and(|r| r.entered > tick || r.left.is_some_and(|t| t > tick))
        {
            return;
        }
        self.remotes.insert(
            player,
            RemoteTrack {
                entered: tick,
                left: None,
                samples: VecDeque::from([(tick, state)]),
            },
        );
    }

    pub fn leave(&mut self, player: PlayerId, tick: ServerTick) {
        if let Some(track) = self.remotes.get_mut(&player)
            && tick >= track.entered
        {
            track.left = Some(tick);
        }
        self.world.remove_player(player);
    }

    pub fn remote_snapshot(&mut self, tick: ServerTick, player: ReplicatedPlayer) {
        if let Some(track) = self.remotes.get_mut(&player.player) {
            track.insert(tick, player.state);
        }
    }

    pub fn render_remotes(&self, server_tick: f64) -> Vec<(PlayerId, PlayerState)> {
        self.remotes
            .iter()
            .filter_map(|(&id, track)| track.sample(server_tick - 6.0).map(|state| (id, state)))
            .collect()
    }

    fn proxies(&mut self, tick: ServerTick) {
        let states: Vec<_> = self
            .remotes
            .iter()
            .filter_map(|(&id, track)| track.sample(tick.0 as f64).map(|state| (id, state)))
            .collect();
        let present: BTreeSet<_> = states.iter().map(|p| p.0).collect();
        let removed: Vec<_> = self
            .world
            .states()
            .map(|p| p.0)
            .filter(|&id| id != self.player && !present.contains(&id))
            .collect();
        for id in removed {
            self.world.remove_player(id);
        }
        self.world.set_players(states);
        self.remotes.retain(|_, track| {
            !track
                .left
                .is_some_and(|left| left.0 + HISTORY_TICKS < self.last_ack.0)
        });
    }

    pub fn predict(&mut self, frame: InputFrame) -> Result<(), &'static str> {
        if self.needs_reset {
            return Err("resynchronization required");
        }
        if frame.epoch != self.epoch || frame.tick.0 != self.tick.0 + 1 {
            return Err("prediction must advance one tick");
        }
        let frame = frame.validate()?;
        self.proxies(frame.tick);
        self.world
            .step(self.player, frame.command(self.state().motion));
        self.tick = frame.tick;
        self.history.push_back(frame);
        if self.history.len() > HISTORY_TICKS as usize {
            self.history.pop_front();
            self.needs_reset = true;
        }
        Ok(())
    }

    pub fn batch(&self) -> InputBatch {
        let skip = self.history.len().saturating_sub(INPUT_REDUNDANCY);
        InputBatch {
            frames: self.history.iter().skip(skip).copied().collect(),
        }
    }

    pub fn reset(&mut self, tick: ServerTick, epoch: u32, state: PlayerState) {
        self.world.set_player(self.player, state);
        self.tick = tick;
        self.last_ack = tick;
        self.epoch = epoch;
        self.history.clear();
        self.needs_reset = false;
    }

    pub fn reconcile(&mut self, owner: OwnerState) {
        if owner.epoch != self.epoch || owner.acknowledged <= self.last_ack {
            return;
        }
        let before = self.state().position;
        if self
            .history
            .front()
            .is_some_and(|f| f.tick.0 > owner.acknowledged.0 + 1)
            && self.tick > owner.acknowledged
        {
            self.needs_reset = true;
            return;
        }
        self.last_ack = owner.acknowledged;
        self.history.retain(|f| f.tick > owner.acknowledged);
        self.world.set_player(self.player, owner.state);
        let frames: Vec<_> = self.history.iter().copied().collect();
        self.tick = owner.acknowledged;
        for frame in frames {
            self.proxies(frame.tick);
            self.world
                .step(self.player, frame.command(self.state().motion));
            self.tick = frame.tick;
        }
        let after = self.state().position;
        let error = before
            .iter()
            .zip(after)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt();
        if error > 0.001 {
            self.corrections += 1;
        }
        self.max_correction = self.max_correction.max(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::Authority;

    #[test]
    fn acknowledging_substituted_ticks_does_not_double_apply_movement() {
        let mut server = Authority::new(MapId::Polygon, 1);
        let (id, state) = server.join().unwrap();
        let mut client = Prediction::new(MapId::Polygon, id, server.tick, 1, state);
        for tick in 1..=80 {
            let mut frame = InputFrame::neutral(1, ServerTick(tick), client.state().motion);
            frame.movement = [0.0, if tick < 50 { 1.0 } else { 0.0 }];
            frame.jump = tick == 25;
            client.predict(frame).unwrap();
            // A contiguous loss burst, followed by redundant out-of-order data.
            if !(20..=30).contains(&tick) {
                let mut frames = client.batch().frames;
                frames.reverse();
                server.inputs.get_mut(&id).unwrap().insert(&frames).unwrap();
            }
            server.step();
            if tick % 3 == 0 {
                client.reconcile(OwnerState {
                    epoch: 1,
                    acknowledged: server.tick,
                    state: server.world.state(id).unwrap(),
                });
            }
        }
        client.reconcile(OwnerState {
            epoch: 1,
            acknowledged: server.tick,
            state: server.world.state(id).unwrap(),
        });
        assert_eq!(client.state(), server.world.state(id).unwrap());
        assert_eq!(client.tick, server.tick);
    }

    #[test]
    fn remote_history_reorders_samples_and_bounds_extrapolation() {
        let a = PlayerState::at([0.0, 1.0, 0.0]);
        let mut b = PlayerState::at([1.0, 1.0, 0.0]);
        b.velocity[0] = 6.0;
        let mut track = RemoteTrack {
            entered: ServerTick(0),
            left: None,
            samples: VecDeque::from([(ServerTick(0), a)]),
        };
        track.insert(ServerTick(10), b);
        track.insert(ServerTick(5), PlayerState::at([0.5, 1.0, 0.0]));
        assert!((track.sample(7.5).unwrap().position[0] - 0.75).abs() < 0.001);
        assert!((track.sample(1000.0).unwrap().position[0] - 2.2).abs() < 0.001);
        track.left = Some(ServerTick(11));
        assert!(track.sample(11.0).is_none());
    }

    #[test]
    fn replay_keeps_remote_blocking_and_departure_removes_it() {
        let mut state = PlayerState::at([0.0, 0.92, 12.0]);
        state.motion.grounded = true;
        let mut client = Prediction::new(MapId::Polygon, PlayerId(1), ServerTick(0), 1, state);
        let remote = PlayerState {
            position: [2.0, 0.92, 12.0],
            ..state
        };
        client.enter(PlayerId(2), ServerTick(0), remote);
        let mut checkpoint = state;
        for tick in 1..=60 {
            let mut frame = InputFrame::neutral(1, ServerTick(tick), client.state().motion);
            frame.movement = [1.0, 0.0];
            client.predict(frame).unwrap();
            if tick == 20 {
                checkpoint = client.state();
            }
        }
        assert!(
            client.state().position[0] <= 1.305,
            "prediction must respect player blocking"
        );
        checkpoint.position[0] -= 0.1;
        client.reconcile(OwnerState {
            epoch: 1,
            acknowledged: ServerTick(20),
            state: checkpoint,
        });
        assert!(
            client.state().position[0] <= 1.305,
            "replay must respect the same collision proxy"
        );
        assert_eq!(
            client.world.state(PlayerId(2)),
            Some(remote),
            "replay must not simulate other players"
        );

        client.leave(PlayerId(2), ServerTick(60));
        assert!(client.render_remotes(67.0).is_empty());
        for tick in 61..=90 {
            let mut frame = InputFrame::neutral(1, ServerTick(tick), client.state().motion);
            frame.movement = [1.0, 0.0];
            client.predict(frame).unwrap();
        }
        assert!(
            client.state().position[0] > 3.0,
            "departed players must stop blocking prediction"
        );
    }

    #[test]
    fn missing_history_requires_reset_and_old_epochs_cannot_overwrite_it() {
        let state = PlayerState::at([0.0, 2.0, 12.0]);
        let mut client = Prediction::new(MapId::Polygon, PlayerId(1), ServerTick(0), 1, state);
        for tick in 1..=HISTORY_TICKS + 1 {
            client
                .predict(InputFrame::neutral(
                    1,
                    ServerTick(tick),
                    client.state().motion,
                ))
                .unwrap();
        }
        assert!(client.needs_reset);
        assert_eq!(client.history.len(), HISTORY_TICKS as usize);
        assert!(
            client
                .predict(InputFrame::neutral(
                    1,
                    ServerTick(HISTORY_TICKS + 2),
                    state.motion
                ))
                .is_err()
        );

        let mut restored = PlayerState::at([3.0, 2.0, 12.0]);
        restored.motion.yaw = 0.5;
        restored.motion.vertical_velocity = 2.0;
        client.reset(ServerTick(200), 2, restored);
        client.reconcile(OwnerState {
            epoch: 1,
            acknowledged: ServerTick(500),
            state,
        });
        assert_eq!(client.state(), restored);
        assert_eq!(client.tick, ServerTick(200));
        assert!(client.batch().frames.is_empty());
        assert!(!client.needs_reset);
        client
            .predict(InputFrame::neutral(2, ServerTick(201), restored.motion))
            .unwrap();
        client.reconcile(OwnerState {
            epoch: 2,
            acknowledged: ServerTick(201),
            state: restored,
        });
        client.reconcile(OwnerState {
            epoch: 2,
            acknowledged: ServerTick(200),
            state,
        });
        assert_eq!(
            client.state(),
            restored,
            "full motor state must survive resets and stale ACKs"
        );
        assert!(client.batch().frames.is_empty());
    }
}
