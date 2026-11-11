pub mod load;
pub mod metrics;

use metrics::{Metrics, Report, Traffic};
use midnight_network::{
    NetResult,
    identity::ServerIdentity,
    server::{Peer, ServerEvent, ServerTransport},
    tokio,
};
use shared::{
    authority::Authority, level::MapId, movement::FIXED_HZ, protocol::*, replication::SpatialIndex,
    survival::ActionOutcome,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct ServerOptions {
    pub listen: SocketAddr,
    pub data_dir: PathBuf,
    pub capacity: usize,
    pub map: MapId,
    pub spread: bool,
    pub seconds: Option<f64>,
    pub report: Option<PathBuf>,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:5000".parse().unwrap(),
            data_dir: "server-data".into(),
            capacity: MAX_PLAYERS,
            map: MapId::Polygon,
            spread: false,
            seconds: None,
            report: None,
        }
    }
}

struct Session {
    peer: Peer,
    player: PlayerId,
    visible: BTreeSet<PlayerId>,
    last_resync: Instant,
}

pub struct GameServer {
    pub authority: Authority,
    sessions: BTreeMap<u64, Session>,
    departed_traffic: Traffic,
    capacity: usize,
    spread: bool,
}

impl GameServer {
    pub fn new(map: MapId, capacity: usize, spread: bool) -> Self {
        Self {
            authority: Authority::new(map, capacity),
            sessions: BTreeMap::new(),
            departed_traffic: Traffic::default(),
            capacity,
            spread,
        }
    }

    pub fn try_new(map: MapId, capacity: usize, spread: bool) -> NetResult<Self> {
        let (loaded, collision) =
            shared::world::LoadedWorld::load(&shared::world::WorldSelection::for_map(map))
                .map_err(|error| error.to_string())?;
        let mut game = Self::new(map, capacity, spread);
        game.authority = Authority::from_loaded_world(&loaded, collision, capacity);
        Ok(game)
    }

    pub fn population(&self) -> usize {
        self.sessions.len()
    }

    fn traffic_for(peer: &Peer) -> Traffic {
        let s = peer.connection.stats();
        Traffic {
            sent_bytes: s.udp_tx.bytes,
            received_bytes: s.udp_rx.bytes,
            lost_packets: s.path.lost_packets,
        }
    }

    pub fn traffic(&self) -> Traffic {
        let mut result = self.departed_traffic;
        for session in self.sessions.values() {
            result.add(Self::traffic_for(&session.peer));
        }
        result
    }

    fn remove(&mut self, id: u64) {
        if let Some(session) = self.sessions.remove(&id) {
            self.departed_traffic.add(Self::traffic_for(&session.peer));
            self.authority.leave(session.player);
        }
    }

    pub fn event(&mut self, event: ServerEvent) {
        match event {
            ServerEvent::Joined(peer) => {
                let admission = self.authority.join();
                match admission {
                    Ok((player, mut state)) => {
                        if self.spread {
                            // Test arena layout is selected by the server; clients
                            // never send a spawn position or teleport request.
                            for offset in 0..256 {
                                let slot = (self.sessions.len() + offset) % 256;
                                let group = slot % 4;
                                let p = [
                                    if group.is_multiple_of(2) {
                                        -160.0
                                    } else {
                                        160.0
                                    } + ((slot / 4) % 8) as f32 * 1.5
                                        - 5.25,
                                    2.0,
                                    if group < 2 { -160.0 } else { 160.0 }
                                        + (slot / 32) as f32 * 1.5
                                        - 5.25,
                                ];
                                if self.authority.world.free_at(p, Some(player)) {
                                    state = PlayerState::at(p);
                                    self.authority.world.set_player(player, state);
                                    break;
                                }
                            }
                        }
                        let message = ServerControl::Welcome {
                            player,
                            tick: self.authority.tick,
                            epoch: 1,
                            map: self.authority.world.map,
                            world: self.authority.descriptor.clone(),
                            state,
                            capacity: self.capacity as u16,
                        };
                        if peer.send(message).is_err() {
                            self.authority.leave(player);
                            peer.close("Клієнт не готовий");
                            return;
                        }
                        if let Some(survival) = self.authority.survival.player(player).cloned() {
                            let _ = peer.send(ServerControl::SurvivalState(survival));
                        }
                        if !self.authority.survival.nodes.is_empty() {
                            let _ = peer.send(ServerControl::ResourceNodes(
                                self.authority.survival.nodes.clone(),
                            ));
                        }
                        if !self.authority.survival.structures.is_empty() {
                            let _ = peer.send(ServerControl::Structures(
                                self.authority.survival.structures.clone(),
                            ));
                        }
                        if self.authority.survival.monument.fuse_ticks_remaining > 0
                            || self.authority.survival.monument.door_open_ticks > 0
                        {
                            let _ = peer
                                .send(ServerControl::Monument(self.authority.survival.monument));
                            if !self.authority.survival.npcs.is_empty()
                                || !self.authority.survival.vehicles.is_empty()
                            {
                                let _ = peer.send(ServerControl::WorldDynamicState {
                                    npcs: self.authority.survival.npcs.clone(),
                                    vehicles: self.authority.survival.vehicles.clone(),
                                    nuclear: self.authority.survival.nuclear,
                                });
                            }
                        }
                        self.sessions.insert(
                            peer.id,
                            Session {
                                peer,
                                player,
                                visible: BTreeSet::new(),
                                last_resync: Instant::now() - Duration::from_secs(2),
                            },
                        );
                    }
                    Err(reason) => {
                        let _ = peer.send(ServerControl::Rejected(reason.into()));
                        peer.close(reason);
                    }
                }
            }
            ServerEvent::Inputs { peer, batch } => {
                if let Some(session) = self.sessions.get(&peer)
                    && let Some(queue) = self.authority.inputs.get_mut(&session.player)
                    && let Err(reason) = queue.insert(&batch.frames)
                {
                    session.peer.close(reason);
                }
            }
            ServerEvent::Control { peer, message } => {
                let Some(session_player) = self.sessions.get(&peer).map(|s| s.player) else {
                    return;
                };
                let mut broadcast_nodes = false;
                let mut broadcast_structures = false;
                let mut broadcast_monument = None;
                let mut broadcast_dynamic = false;
                let mut damaged_target = None;
                let response = match message {
                    ClientControl::Ping(nonce) => Some(ServerControl::Pong {
                        nonce,
                        tick: self.authority.tick,
                    }),
                    ClientControl::Resync => {
                        let Some(session) = self.sessions.get_mut(&peer) else {
                            return;
                        };
                        if session.last_resync.elapsed() < Duration::from_secs(1) {
                            return;
                        };
                        session.last_resync = Instant::now();
                        let state = self.authority.world.state(session.player).unwrap();
                        let input = self.authority.inputs.get_mut(&session.player).unwrap();
                        input.reset(self.authority.tick, state);
                        Some(ServerControl::Reset {
                            tick: self.authority.tick,
                            epoch: input.epoch,
                            state,
                        })
                    }
                    ClientControl::SelectSlot(slot) => {
                        if let Some(survival) = self.authority.survival.player_mut(session_player) {
                            survival.inventory.select_slot(slot);
                            survival.notice = None;
                            Some(ServerControl::SurvivalState(survival.clone()))
                        } else {
                            None
                        }
                    }
                    ClientControl::MoveSlot { from, to } => {
                        if let Some(survival) = self.authority.survival.player_mut(session_player) {
                            survival.inventory.move_slot(from, to);
                            survival.notice = None;
                            Some(ServerControl::SurvivalState(survival.clone()))
                        } else {
                            None
                        }
                    }
                    ClientControl::Craft(recipe) => {
                        if let Some(survival) = self.authority.survival.player_mut(session_player) {
                            match survival.inventory.craft(recipe) {
                                Ok(item) => {
                                    survival.notice = Some(format!("Скрафчено: {}", item.label()));
                                }
                                Err(err) => {
                                    survival.notice = Some(err.into());
                                }
                            }
                            Some(ServerControl::SurvivalState(survival.clone()))
                        } else {
                            None
                        }
                    }
                    ClientControl::CycleStructure => {
                        if let Some(survival) = self.authority.survival.player_mut(session_player) {
                            survival.cycle_structure();
                            Some(ServerControl::SurvivalState(survival.clone()))
                        } else {
                            None
                        }
                    }
                    ClientControl::RotateStructure => {
                        if let Some(survival) = self.authority.survival.player_mut(session_player) {
                            survival.rotate_structure();
                            Some(ServerControl::SurvivalState(survival.clone()))
                        } else {
                            None
                        }
                    }
                    ClientControl::MountOrDismount { yaw, pitch } => {
                        let Some(actor_state) = self.authority.world.state(session_player) else {
                            return;
                        };
                        match self.authority.survival.toggle_mount_vehicle(
                            session_player,
                            actor_state.position,
                            yaw,
                            pitch,
                        ) {
                            Ok(_) => {
                                broadcast_dynamic = true;
                            }
                            Err(err) => {
                                if let Some(s) = self.authority.survival.player_mut(session_player)
                                {
                                    s.notice = Some(err.into());
                                }
                            }
                        }
                        self.authority
                            .survival
                            .player(session_player)
                            .cloned()
                            .map(ServerControl::SurvivalState)
                    }
                    ClientControl::PrimaryAction { yaw, pitch } => {
                        let Some(actor_state) = self.authority.world.state(session_player) else {
                            return;
                        };
                        let others: Vec<_> = self
                            .authority
                            .world
                            .states()
                            .map(|(id, s)| (id, s.position))
                            .collect();
                        match self.authority.survival.primary_action(
                            session_player,
                            self.authority.tick,
                            actor_state.position,
                            yaw,
                            pitch,
                            &others,
                        ) {
                            Ok(ActionOutcome::Harvested { .. }) => {
                                broadcast_nodes = true;
                            }
                            Ok(ActionOutcome::BuiltStructure(placed)) => {
                                self.authority.world.insert_structure(placed);
                                broadcast_structures = true;
                            }
                            Ok(ActionOutcome::UpgradedStructure(_)) => {
                                broadcast_structures = true;
                            }
                            Ok(ActionOutcome::DamagedStructure { .. }) => {
                                self.authority
                                    .world
                                    .sync_structures(&self.authority.survival.structures);
                                broadcast_structures = true;
                            }
                            Ok(ActionOutcome::DamagedNpc { .. }) => {
                                broadcast_dynamic = true;
                            }
                            Ok(ActionOutcome::DamagedPlayer {
                                target, defeated, ..
                            }) => {
                                damaged_target = Some((target, defeated));
                            }
                            Ok(ActionOutcome::FiredWeapon {
                                hit_player: Some(target),
                                ..
                            }) => {
                                let defeated =
                                    self.authority.survival.player(target).is_some_and(|p| {
                                        p.vitals.health >= 99.9
                                            && p.notice
                                                .as_deref()
                                                .is_some_and(|n| n.contains("ліквідовано"))
                                    });
                                damaged_target = Some((target, defeated));
                            }
                            Ok(ActionOutcome::FiredWeapon { .. }) => {
                                broadcast_dynamic = true;
                            }
                            Ok(ActionOutcome::MonumentInteracted(monument)) => {
                                broadcast_monument = Some(monument);
                                broadcast_dynamic = true;
                            }
                            Ok(_) => {}
                            Err(err) => {
                                if let Some(s) = self.authority.survival.player_mut(session_player)
                                {
                                    s.notice = Some(err.into());
                                }
                            }
                        }
                        self.authority
                            .survival
                            .player(session_player)
                            .cloned()
                            .map(ServerControl::SurvivalState)
                    }
                };
                if let Some(response) = response
                    && let Some(session) = self.sessions.get(&peer)
                    && session.peer.send(response).is_err()
                {
                    session.peer.close("Клієнт не приймає повідомлення");
                }
                if broadcast_nodes {
                    let nodes = self.authority.survival.nodes.clone();
                    for session in self.sessions.values() {
                        let _ = session
                            .peer
                            .send(ServerControl::ResourceNodes(nodes.clone()));
                    }
                }
                if broadcast_structures {
                    let structures = self.authority.survival.structures.clone();
                    for session in self.sessions.values() {
                        let _ = session
                            .peer
                            .send(ServerControl::Structures(structures.clone()));
                    }
                }
                if let Some(monument) = broadcast_monument {
                    for session in self.sessions.values() {
                        let _ = session.peer.send(ServerControl::Monument(monument));
                    }
                }
                if broadcast_dynamic {
                    let msg = ServerControl::WorldDynamicState {
                        npcs: self.authority.survival.npcs.clone(),
                        vehicles: self.authority.survival.vehicles.clone(),
                        nuclear: self.authority.survival.nuclear,
                    };
                    for session in self.sessions.values() {
                        let _ = session.peer.send(msg.clone());
                    }
                }
                if let Some((target, defeated)) = damaged_target {
                    if defeated
                        && let Some(spawn) = self.authority.world.spawn_position(Some(target))
                    {
                        let state = PlayerState::at(spawn);
                        self.authority.world.set_player(target, state);
                        if let Some(queue) = self.authority.inputs.get_mut(&target) {
                            queue.reset(self.authority.tick, state);
                            for session in self.sessions.values() {
                                if session.player == target {
                                    let _ = session.peer.send(ServerControl::Reset {
                                        tick: self.authority.tick,
                                        epoch: queue.epoch,
                                        state,
                                    });
                                }
                            }
                        }
                    }
                    if let Some(target_survival) = self.authority.survival.player(target).cloned() {
                        for session in self.sessions.values() {
                            if session.player == target {
                                let _ = session
                                    .peer
                                    .send(ServerControl::SurvivalState(target_survival.clone()));
                            }
                        }
                    }
                }
            }
            ServerEvent::Left { peer } => self.remove(peer),
        }
    }

    pub fn step(&mut self) -> metrics::StepWork {
        let started = Instant::now();
        let closed: Vec<_> = self
            .sessions
            .iter()
            .filter(|(_, session)| session.peer.connection.close_reason().is_some())
            .map(|(&id, _)| id)
            .collect();
        for id in closed {
            self.remove(id);
        }
        self.authority.step();
        let physics_seconds = started.elapsed().as_secs_f64();
        let started = Instant::now();
        self.replicate();
        metrics::StepWork {
            physics_seconds,
            replication_seconds: started.elapsed().as_secs_f64(),
        }
    }

    fn replicate(&mut self) {
        let tick = self.authority.tick;
        let states: BTreeMap<_, _> = self.authority.world.states().collect();
        let index = SpatialIndex::build(states.iter().map(|(&id, &s)| (id, s)));
        let population = self.population() as u16;
        for session in self.sessions.values_mut() {
            // Every peer still receives 20 Hz. Spread their phases across the
            // three simulation ticks instead of making one large 150-peer burst.
            if session.player.0 % SNAPSHOT_INTERVAL != tick.0 % SNAPSHOT_INTERVAL {
                continue;
            }
            let own = states[&session.player];
            let visible = index.visible(session.player, own.position, &session.visible);
            for &player in visible.difference(&session.visible) {
                if session
                    .peer
                    .send(ServerControl::Enter {
                        player,
                        tick,
                        state: states[&player],
                    })
                    .is_err()
                {
                    session.peer.close("Черга повідомлень переповнена");
                }
            }
            for &player in session.visible.difference(&visible) {
                if session
                    .peer
                    .send(ServerControl::Leave { player, tick })
                    .is_err()
                {
                    session.peer.close("Черга повідомлень переповнена");
                }
            }
            session.visible = visible;
            let input = &self.authority.inputs[&session.player];
            let snapshot = Snapshot {
                tick,
                population,
                owner: Some(OwnerState {
                    epoch: input.epoch,
                    acknowledged: input.finalized,
                    state: own,
                }),
                players: session
                    .visible
                    .iter()
                    .map(|&player| ReplicatedPlayer {
                        player,
                        state: states[&player],
                    })
                    .collect(),
            };
            if let Err(reason) = session.peer.send_snapshot(snapshot) {
                session.peer.close(&reason);
            }
            if tick.0 > 0
                && session.player.0 % 120 == tick.0 % 120
                && let Some(survival) = self.authority.survival.player(session.player).cloned()
            {
                let _ = session.peer.send(ServerControl::SurvivalState(survival));
                if !self.authority.survival.npcs.is_empty()
                    || !self.authority.survival.vehicles.is_empty()
                {
                    let _ = session.peer.send(ServerControl::WorldDynamicState {
                        npcs: self.authority.survival.npcs.clone(),
                        vehicles: self.authority.survival.vehicles.clone(),
                        nuclear: self.authority.survival.nuclear,
                    });
                }
            }
        }
    }
}

pub async fn serve(options: ServerOptions) -> NetResult<Report> {
    if options.capacity == 0 || options.capacity > MAX_PLAYERS {
        return Err("Місткість має бути від 1 до 150".into());
    }
    if options.spread && options.map != MapId::LoadArena {
        return Err("--layout spread потребує --map load-arena".into());
    }
    let mut game = GameServer::try_new(options.map, options.capacity, options.spread)?;
    let identity = ServerIdentity::load_or_create(&options.data_dir)?;
    let mut transport = ServerTransport::bind(&identity, options.listen)?;
    println!(
        "Midnight server: {} | {:?} | capacity={} | 60 Hz",
        transport.endpoint.local_addr().map_err(|e| e.to_string())?,
        options.map,
        options.capacity
    );
    println!(
        "Код доступу: server invite --data-dir {}",
        options.data_dir.display()
    );
    let mut metrics = Metrics::default();
    let dt = Duration::from_secs_f64(1.0 / FIXED_HZ);
    let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + dt, dt);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Burst);
    let stop = tokio::signal::ctrl_c();
    tokio::pin!(stop);
    let layout = if options.spread { "spread" } else { "crowd" };
    let result: NetResult<()> = loop {
        let deadline = tokio::select! {
            deadline = interval.tick() => deadline,
            _ = &mut stop => break Ok(()),
        };
        let lag_seconds = deadline.elapsed().as_secs_f64();
        let start = Instant::now();
        let queue = transport.events.len();
        for _ in 0..2048 {
            match transport.events.try_recv() {
                Ok(event) => game.event(event),
                Err(_) => break,
            }
        }
        let work = game.step();
        metrics.observe(
            start.elapsed().as_secs_f64(),
            lag_seconds,
            game.population(),
            options.capacity,
            queue,
            work,
        );
        if game.authority.tick.0.is_multiple_of(300) {
            let report = metrics.report(options.map, layout, game.population(), game.traffic());
            println!(
                "tick={} players={} p99={:.2}ms full_rate={:.1}Hz physics_p99={:.2}ms replication_p99={:.2}ms rss={:.1}MB late={} lag_max={:.1}ms",
                game.authority.tick.0,
                game.population(),
                report.tick_p99_ms,
                report.ticks_per_second_at_capacity,
                report.physics_p99_ms_at_capacity,
                report.replication_p99_ms_at_capacity,
                report.current_rss_mb,
                report.late_ticks,
                report.max_tick_lag_ms,
            );
            if let Some(path) = &options.report
                && let Err(error) = metrics::write_json(path, &report)
            {
                break Err(error);
            }
        }
        if options
            .seconds
            .is_some_and(|seconds| metrics.elapsed() >= seconds)
        {
            break Ok(());
        }
    };
    let report = metrics.report(options.map, layout, game.population(), game.traffic());
    transport.shutdown().await;
    result?;
    if let Some(path) = &options.report {
        metrics::write_json(path, &report)?;
    }
    Ok(report)
}
