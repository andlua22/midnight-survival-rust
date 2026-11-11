use crate::{
    interface::{Interface, UiMode},
    launch::ClientOptions,
};
use bevy::prelude::*;
use midnight_network::{
    client::{ClientEvent, ClientLink},
    identity::Invitation,
};
use shared::{
    CharacterMotion, LevelGeometry, LocalPlayer, Player, ResourceNodeMarker, SimulationClock,
    SimulationInput, SimulationMode, SimulationPose, SimulationSet, SimulationSurvival,
    SimulationWorld, StructureMarker,
    movement::{FIXED_DT, PLAYER_HALF_SEGMENT, PLAYER_RADIUS, preview_look},
    prediction::Prediction,
    protocol::*,
    simulation::{spawn_geometry, spawn_local},
    survival::SurvivalWorld,
    world::LoadedWorld,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, Instant},
};

enum Request {
    Connect,
    Disconnect,
}

#[derive(Resource)]
pub(crate) struct Online {
    pub address: String,
    pub code: String,
    pub status: String,
    pub connected: bool,
    pub connecting: bool,
    pub population: u16,
    pub capacity: u16,
    pub rtt: Duration,
    pub lost_packets: u64,
    pub sent_bytes: u64,
    pub received_bytes: u64,
    pub camera_offset: Vec3,
    request: Option<Request>,
    link: Option<ClientLink>,
    prediction: Option<Prediction>,
    observed_tick: ServerTick,
    observed_at: Instant,
    resync_requested: bool,
}

impl Online {
    pub fn new(options: &ClientOptions) -> Self {
        let mut session = Self {
            address: options
                .connect
                .clone()
                .unwrap_or_else(|| "127.0.0.1:5000".into()),
            code: String::new(),
            status: "Офлайн".into(),
            connected: false,
            connecting: false,
            population: 0,
            capacity: MAX_PLAYERS as u16,
            rtt: Duration::from_millis(50),
            lost_packets: 0,
            sent_bytes: 0,
            received_bytes: 0,
            camera_offset: Vec3::ZERO,
            request: None,
            link: None,
            prediction: None,
            observed_tick: ServerTick(0),
            observed_at: Instant::now(),
            resync_requested: false,
        };
        if let Some(path) = &options.join_code_file {
            match std::fs::read_to_string(path) {
                Ok(code) => {
                    session.code = code.trim().into();
                    session.request = Some(Request::Connect);
                }
                Err(error) => session.status = format!("Не вдалося прочитати код доступу: {error}"),
            }
        }
        session
    }
    pub fn connect(&mut self) {
        self.request = Some(Request::Connect);
    }
    pub fn disconnect(&mut self) {
        self.request = Some(Request::Disconnect);
    }
    pub fn send_control(&self, control: ClientControl) {
        if let Some(link) = &self.link {
            let _ = link.send_control(control);
        }
    }
    pub fn prediction_statistics(&self) -> Option<(PlayerId, u64, f32)> {
        self.prediction
            .as_ref()
            .map(|p| (p.player, p.corrections, p.max_correction))
    }
    pub fn local_player_id(&self) -> PlayerId {
        self.prediction
            .as_ref()
            .map(|p| p.player)
            .unwrap_or(PlayerId(0))
    }
    fn observe(&mut self, tick: ServerTick, received: Instant) {
        if tick > self.observed_tick {
            self.observed_tick = tick;
            self.observed_at = received;
        }
    }
    fn server_time(&self) -> f64 {
        self.observed_tick.0 as f64
            + (self.observed_at.elapsed().as_secs_f64() + self.rtt.as_secs_f64() * 0.5)
                / FIXED_DT as f64
    }
}

pub(crate) struct OnlinePlugin;

impl Plugin for OnlinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, avatar_assets)
            .add_systems(PreUpdate, receive.after(bevy::input::InputSystems))
            .add_systems(FixedUpdate, predict.before(SimulationSet::Step))
            .add_systems(Update, present);
    }
}

#[derive(Component)]
struct RemoteAvatar(PlayerId);

#[derive(Resource)]
struct AvatarAssets {
    capsule: Handle<Mesh>,
    marker: Handle<Mesh>,
    colors: Vec<Handle<StandardMaterial>>,
    marker_material: Handle<StandardMaterial>,
}

fn avatar_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AvatarAssets {
        capsule: meshes.add(Capsule3d::new(PLAYER_RADIUS, PLAYER_HALF_SEGMENT * 2.0)),
        marker: meshes.add(Cuboid::new(0.18, 0.12, 0.14)),
        colors: [
            [0.25, 0.67, 0.58],
            [0.72, 0.48, 0.28],
            [0.32, 0.5, 0.78],
            [0.7, 0.35, 0.45],
        ]
        .into_iter()
        .map(|[r, g, b]| materials.add(Color::srgb(r, g, b)))
        .collect(),
        marker_material: materials.add(Color::srgb(0.93, 0.94, 0.88)),
    });
}

#[allow(clippy::too_many_arguments)] // Bridge between session lifecycle and Bevy world resources.
fn receive(
    mut commands: Commands,
    mut online: ResMut<Online>,
    mut mode: ResMut<SimulationMode>,
    mut input: ResMut<SimulationInput>,
    mut physics: ResMut<SimulationWorld>,
    mut survival: ResMut<SimulationSurvival>,
    mut source_inventory: ResMut<shared::source_inventory::SourceInventoryWorld>,
    loaded: Res<LoadedWorld>,
    players: Query<Entity, With<Player>>,
    geometry: Query<Entity, With<LevelGeometry>>,
    mut node_entities: Query<(Entity, &mut ResourceNodeMarker)>,
    structure_entities: Query<Entity, With<StructureMarker>>,
    mut interface: ResMut<Interface>,
) {
    let mut return_offline = false;
    match online.request.take() {
        Some(Request::Connect) => {
            online.link = None;
            match Invitation::parse(&online.code)
                .and_then(|invite| ClientLink::start(online.address.trim().into(), invite))
            {
                Ok(link) => {
                    online.link = Some(link);
                    online.connecting = true;
                    online.status = "Підключення…".into();
                }
                Err(error) => {
                    online.connecting = false;
                    online.status = error;
                }
            }
            input.0.clear();
            interface.mode = UiMode::Menu;
        }
        Some(Request::Disconnect) => {
            online.status = "Від'єднано".into();
            return_offline = true;
        }
        None => {}
    }
    let mut events = if return_offline {
        Vec::new()
    } else {
        online
            .link
            .as_ref()
            .map(ClientLink::poll)
            .unwrap_or_default()
    };
    // A welcome and a close can arrive before the same Bevy frame. Handle the
    // terminal state before queuing any world replacement/spawn commands.
    if let Some(reason) = events.iter().find_map(|event| match event {
        ClientEvent::Disconnected(reason) => Some(reason.clone()),
        ClientEvent::Control {
            message: ServerControl::Rejected(reason),
            ..
        } => Some(reason.clone()),
        _ => None,
    }) {
        online.status = reason;
        return_offline = true;
        events.clear();
    }
    for event in events {
        match event {
            ClientEvent::Control { message, received } => match message {
                ServerControl::Welcome {
                    player,
                    tick,
                    epoch,
                    map,
                    world,
                    state,
                    capacity,
                } => {
                    if map != world.map || !loaded.descriptor.compatible_with(&world) {
                        online.status =
                            "Server world content does not match the loaded world.".into();
                        return_offline = true;
                        break;
                    }
                    let collision = match loaded.collision() {
                        Ok(collision) => collision,
                        Err(error) => {
                            online.status = format!("Cannot initialize server world: {error}");
                            return_offline = true;
                            break;
                        }
                    };
                    for entity in &players {
                        commands.entity(entity).despawn();
                    }
                    for entity in &geometry {
                        commands.entity(entity).despawn();
                    }
                    for (entity, _) in &node_entities {
                        commands.entity(entity).despawn();
                    }
                    for entity in &structure_entities {
                        commands.entity(entity).despawn();
                    }
                    survival.0 = SurvivalWorld::new(map);
                    survival.0.join(PlayerId(0));
                    *source_inventory = loaded
                        .gameplay
                        .as_ref()
                        .map(|catalog| {
                            shared::source_inventory::SourceInventoryWorld::from_catalog(
                                catalog.clone(),
                            )
                        })
                        .unwrap_or_default();
                    source_inventory.join(player);
                    spawn_geometry(&mut commands, map);
                    spawn_local(&mut commands, player, state);
                    *mode = SimulationMode::Network;
                    online.prediction = Some(Prediction::from_world(
                        collision, player, tick, epoch, state,
                    ));
                    online.observed_tick = tick;
                    online.observed_at = received;
                    online.connected = true;
                    online.connecting = false;
                    online.resync_requested = false;
                    online.capacity = capacity;
                    online.population = 1;
                    online.camera_offset = Vec3::ZERO;
                    online.status = "Підключено".into();
                    input.0.clear();
                    interface.mode = UiMode::Playing;
                }
                ServerControl::Enter {
                    player,
                    tick,
                    state,
                } => {
                    source_inventory.join(player);
                    if let Some(prediction) = &mut online.prediction {
                        prediction.enter(player, tick, state);
                    }
                }
                ServerControl::Leave { player, tick } => {
                    source_inventory.leave(player);
                    if let Some(prediction) = &mut online.prediction {
                        prediction.leave(player, tick);
                    }
                }
                ServerControl::Reset { tick, epoch, state } => {
                    if let Some(prediction) = &mut online.prediction {
                        prediction.reset(tick, epoch, state);
                    }
                    online.observe(tick, received);
                    online.resync_requested = false;
                    online.camera_offset = Vec3::ZERO;
                    input.0.clear();
                }
                ServerControl::Pong { tick, .. } => online.observe(tick, received),
                ServerControl::SurvivalState(state) => {
                    survival.0.players.insert(PlayerId(0), state);
                }
                ServerControl::ResourceNodes(nodes) => {
                    for (_, mut marker) in &mut node_entities {
                        if let Some(updated) = nodes.iter().find(|n| n.id == marker.0.id) {
                            marker.0 = *updated;
                        }
                    }
                    survival.0.nodes = nodes;
                }
                ServerControl::Structures(structures) => {
                    if let Some(prediction) = &mut online.prediction {
                        prediction.sync_structures(&structures);
                    }
                    survival.0.structures = structures;
                }
                ServerControl::Monument(monument) => {
                    survival.0.monument = monument;
                }
                ServerControl::WorldDynamicState {
                    npcs,
                    vehicles,
                    nuclear,
                } => {
                    survival.0.npcs = npcs;
                    survival.0.vehicles = vehicles;
                    survival.0.nuclear = nuclear;
                }
                ServerControl::Rejected(reason) => {
                    online.status = reason;
                    return_offline = true;
                }
            },
            ClientEvent::Snapshot { message, received } => {
                online.observe(message.tick, received);
                online.population = message.population;
                if let Some(prediction) = &mut online.prediction {
                    for remote in message.players {
                        prediction.remote_snapshot(message.tick, remote);
                    }
                    if let Some(owner) = message.owner {
                        let before = Vec3::from_array(prediction.state().position);
                        prediction.reconcile(owner);
                        let correction = before - Vec3::from_array(prediction.state().position);
                        if correction.length() < 1.0 {
                            online.camera_offset += correction;
                        } else {
                            online.camera_offset = Vec3::ZERO;
                        }
                    }
                }
            }
            ClientEvent::Statistics {
                rtt,
                sent_bytes,
                received_bytes,
                lost_packets,
            } => {
                online.rtt = rtt;
                online.sent_bytes = sent_bytes;
                online.received_bytes = received_bytes;
                online.lost_packets = lost_packets;
            }
            ClientEvent::Disconnected(reason) => {
                online.status = reason;
                return_offline = true;
            }
        }
    }
    if return_offline {
        online.link = None;
        online.prediction = None;
        *source_inventory = loaded
            .gameplay
            .as_ref()
            .map(|catalog| {
                shared::source_inventory::SourceInventoryWorld::from_catalog(catalog.clone())
            })
            .unwrap_or_default();
        source_inventory.join(PlayerId(0));
        online.connected = false;
        online.connecting = false;
        online.camera_offset = Vec3::ZERO;
        if *mode == SimulationMode::Network {
            for entity in &players {
                commands.entity(entity).despawn();
            }
            for entity in &geometry {
                commands.entity(entity).despawn();
            }
            for (entity, _) in &node_entities {
                commands.entity(entity).despawn();
            }
            for entity in &structure_entities {
                commands.entity(entity).despawn();
            }
            let collision = match loaded.collision() {
                Ok(collision) => collision,
                Err(error) => {
                    online.status = format!("Cannot restore local world: {error}");
                    commands.write_message(AppExit::error());
                    return;
                }
            };
            physics.0 = collision;
            survival.0 = SurvivalWorld::new(loaded.descriptor.map);
            survival.0.join(PlayerId(0));
            spawn_geometry(&mut commands, loaded.descriptor.map);
            spawn_local(&mut commands, PlayerId(0), PlayerState::at(loaded.spawn));
            *mode = SimulationMode::Offline;
        }
        input.0.clear();
        interface.mode = UiMode::Menu;
    }
}

fn predict(
    mut online: ResMut<Online>,
    mut input: ResMut<SimulationInput>,
    mut clock: ResMut<SimulationClock>,
    mut survival: ResMut<SimulationSurvival>,
    mut local: Query<
        (&mut CharacterMotion, &mut SimulationPose, &mut Transform),
        With<LocalPlayer>,
    >,
) {
    if !online.connected {
        return;
    }
    let lead = (online.rtt.as_secs_f64() * 0.5 / FIXED_DT as f64).ceil() as u64 + 3;
    let target = ServerTick(online.server_time().floor() as u64 + lead.min(30));
    let Some(prediction) = &mut online.prediction else {
        return;
    };
    let target = ServerTick(target.0.min(prediction.last_ack.0 + INPUT_WINDOW));
    if prediction.tick < target && !prediction.needs_reset {
        let mut command = input.0.next_command();
        if !survival.0.can_sprint(PlayerId(0)) {
            command.sprint = false;
        }
        let [yaw, pitch] = preview_look(
            prediction.state().motion.yaw,
            prediction.state().motion.pitch,
            command.look_delta,
        );
        let steps = (target.0 - prediction.tick.0).min(8);
        for step in 0..steps {
            let frame = InputFrame {
                epoch: prediction.epoch,
                tick: ServerTick(prediction.tick.0 + 1),
                movement: command.move_vec,
                yaw,
                pitch,
                jump: command.jump && step == 0,
                sprint: command.sprint,
            };
            if prediction.predict(frame).is_err() {
                break;
            }
            let pos = prediction.state().position;
            let sprinting = command.sprint && command.move_vec != [0.0; 2];
            survival
                .0
                .step_with_positions(prediction.tick, [(PlayerId(0), sprinting, pos)]);
        }
    }
    if let Ok((mut motion, mut pose, mut transform)) = local.single_mut() {
        let state = prediction.state();
        pose.previous = pose.current;
        pose.current = Vec3::from_array(state.position);
        transform.translation = pose.current;
        motion.0 = state.motion;
    }
    clock.tick = prediction.tick.0;
    let batch = prediction.batch();
    let needs_reset = prediction.needs_reset;
    if let Some(link) = &online.link {
        if !batch.frames.is_empty() {
            link.send_inputs(batch);
        }
        if needs_reset
            && !online.resync_requested
            && link.send_control(ClientControl::Resync).is_ok()
        {
            online.resync_requested = true;
        }
    }
}

fn present(
    mut commands: Commands,
    mut online: ResMut<Online>,
    time: Res<Time<Real>>,
    assets: Res<AvatarAssets>,
    mut avatars: Query<(Entity, &RemoteAvatar, &mut Transform)>,
) {
    online.camera_offset *= (-time.delta_secs() / 0.1).exp();
    let Some(prediction) = &online.prediction else {
        return;
    };
    let states = prediction.render_remotes(online.server_time());
    let visible: BTreeSet<_> = states.iter().map(|p| p.0).collect();
    let entities: BTreeMap<_, _> = avatars
        .iter()
        .map(|(entity, avatar, _)| (avatar.0, entity))
        .collect();
    for (entity, avatar, _) in &avatars {
        if !visible.contains(&avatar.0) {
            commands.entity(entity).despawn();
        }
    }
    for (id, state) in states {
        let transform = Transform::from_translation(Vec3::from_array(state.position))
            .with_rotation(Quat::from_rotation_y(state.motion.yaw));
        if let Some(&entity) = entities.get(&id) {
            if let Ok((_, _, mut current)) = avatars.get_mut(entity) {
                *current = transform;
            }
        } else {
            commands
                .spawn((
                    Player,
                    RemoteAvatar(id),
                    Mesh3d(assets.capsule.clone()),
                    MeshMaterial3d(assets.colors[id.0 as usize % assets.colors.len()].clone()),
                    transform,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(assets.marker.clone()),
                        MeshMaterial3d(assets.marker_material.clone()),
                        Transform::from_xyz(0.0, 0.45, -PLAYER_RADIUS),
                    ));
                });
        }
    }
}
