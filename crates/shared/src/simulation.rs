use std::time::Duration;

use bevy::{prelude::*, time::TimeUpdateStrategy, transform::TransformPlugin};

use crate::{
    input::InputAccumulator,
    level::{LevelBox, MapId, map_geometry},
    movement::{FIXED_HZ, MotionState},
    physics::CollisionWorld,
    protocol::{PlayerId, PlayerState, ServerTick},
    survival::{PlacedStructure, ResourceNode, SurvivalWorld},
    world::{LoadedWorld, WorldLoadFailure, WorldSelection},
};

#[derive(Resource, Default)]
pub struct SimulationInput(pub InputAccumulator);

#[derive(Resource, Default)]
pub struct SimulationClock {
    pub tick: u64,
}

#[derive(Resource, Default, PartialEq, Eq)]
pub enum SimulationMode {
    #[default]
    Offline,
    Network,
}

#[derive(Resource)]
pub struct SimulationWorld(pub CollisionWorld);

#[derive(Resource)]
pub struct SimulationSurvival(pub SurvivalWorld);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component)]
pub struct PlayerIdentity(pub PlayerId);

#[derive(Component, Default)]
pub struct PlayerControls(pub InputAccumulator);

#[derive(Component, Default)]
pub struct CharacterMotion(pub MotionState);

#[derive(Component, Clone, Copy)]
pub struct SimulationPose {
    pub previous: Vec3,
    pub current: Vec3,
}

#[derive(Component)]
pub struct LevelGeometry(pub LevelBox);

#[derive(Component, Clone, Copy)]
pub struct ResourceNodeMarker(pub ResourceNode);

#[derive(Component, Clone, Copy)]
pub struct StructureMarker(pub PlacedStructure);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    Step,
}

pub struct SharedSimulationPlugin;

impl Plugin for SharedSimulationPlugin {
    fn build(&self, app: &mut App) {
        let selection = app
            .world()
            .get_resource::<WorldSelection>()
            .cloned()
            .unwrap_or_default();
        app.init_resource::<SimulationInput>()
            .init_resource::<SimulationClock>()
            .init_resource::<SimulationMode>()
            .insert_resource(selection.clone())
            .insert_resource(SimulationWorld(CollisionWorld::new(selection.map)))
            .insert_resource(SimulationSurvival(SurvivalWorld::new(selection.map)))
            .insert_resource(Time::<Fixed>::from_hz(FIXED_HZ))
            .add_systems(PreStartup, initialize_world)
            .add_systems(Startup, spawn_world)
            .add_systems(
                FixedUpdate,
                drive_players
                    .in_set(SimulationSet::Step)
                    .run_if(resource_exists::<LoadedWorld>),
            );
    }
}

pub fn spawn_geometry(commands: &mut Commands, map: MapId) {
    for geometry in map_geometry(map) {
        commands.spawn((
            Name::new(geometry.name),
            LevelGeometry(geometry),
            Transform::from_translation(Vec3::from_array(geometry.center))
                .with_rotation(Quat::from_rotation_z(geometry.rotation_z)),
        ));
    }
    for node in crate::survival::map_resource_nodes(map) {
        commands.spawn((
            Name::new(node.kind.label()),
            ResourceNodeMarker(node),
            Transform::from_translation(Vec3::from_array(node.position)),
        ));
    }
}

pub fn spawn_local(commands: &mut Commands, id: PlayerId, state: PlayerState) -> Entity {
    let position = Vec3::from_array(state.position);
    commands
        .spawn((
            Name::new("Local player"),
            Player,
            LocalPlayer,
            PlayerIdentity(id),
            PlayerControls::default(),
            CharacterMotion(state.motion),
            SimulationPose {
                previous: position,
                current: position,
            },
            Transform::from_translation(position),
        ))
        .id()
}

fn initialize_world(world: &mut World) {
    let selection = world.resource::<WorldSelection>().clone();
    match LoadedWorld::load(&selection) {
        Ok((loaded, collision)) => {
            let mut survival = SurvivalWorld::new(selection.map);
            survival.join(PlayerId(0));
            world.insert_resource(SimulationWorld(collision));
            world.insert_resource(SimulationSurvival(survival));
            world.insert_resource(loaded);
        }
        Err(error) => {
            error!("Cannot initialize selected world: {error}");
            world.insert_resource(WorldLoadFailure(error.to_string()));
            if let Some(mut exits) = world.get_resource_mut::<Messages<AppExit>>() {
                exits.write(AppExit::error());
            }
        }
    }
}

fn spawn_world(mut commands: Commands, loaded: Option<Res<LoadedWorld>>) {
    let Some(loaded) = loaded else { return };
    spawn_geometry(&mut commands, loaded.descriptor.map);
    spawn_local(&mut commands, PlayerId(0), PlayerState::at(loaded.spawn));
}

#[allow(clippy::type_complexity)] // Explicit Bevy component access.
fn drive_players(
    mode: Res<SimulationMode>,
    mut input: ResMut<SimulationInput>,
    mut clock: ResMut<SimulationClock>,
    mut physics: ResMut<SimulationWorld>,
    mut survival: ResMut<SimulationSurvival>,
    mut nodes: Query<&mut ResourceNodeMarker>,
    mut players: Query<
        (
            Entity,
            &PlayerIdentity,
            &mut CharacterMotion,
            &mut SimulationPose,
            &mut Transform,
            &mut PlayerControls,
            Option<&LocalPlayer>,
        ),
        With<Player>,
    >,
) {
    if *mode != SimulationMode::Offline {
        return;
    }
    clock.tick += 1;
    let mut order: Vec<_> = players
        .iter()
        .map(|(entity, id, ..)| (id.0, entity))
        .collect();
    order.sort_unstable_by_key(|p| p.0);
    let present: std::collections::BTreeSet<_> = order.iter().map(|p| p.0).collect();
    let removed: Vec<_> = physics
        .0
        .states()
        .map(|p| p.0)
        .filter(|id| !present.contains(id))
        .collect();
    for id in removed {
        physics.0.remove_player(id);
    }
    // Populate every collider before stepping; newly spawned peers also block.
    for (_, id, motion, _, transform, _, _) in &players {
        physics.0.set_player(
            id.0,
            PlayerState {
                position: transform.translation.to_array(),
                motion: motion.0,
                velocity: [0.0; 3],
            },
        );
    }
    let mut sprinting = Vec::with_capacity(order.len());
    for (id, entity) in order {
        let Ok((_, _, mut motion, mut pose, mut transform, mut controls, local)) =
            players.get_mut(entity)
        else {
            continue;
        };
        survival.0.join(id);
        let mut command = if local.is_some() {
            input.0.next_command()
        } else {
            controls.0.next_command()
        };
        if !survival.0.can_sprint(id) {
            command.sprint = false;
        }
        pose.previous = pose.current;
        if let Some(state) = physics.0.step(id, command) {
            pose.current = Vec3::from_array(state.position);
            transform.translation = pose.current;
            motion.0 = state.motion;
        }
        sprinting.push((
            id,
            command.sprint && command.move_vec != [0.0; 2],
            pose.current.to_array(),
        ));
    }
    let prev_struct_count = survival.0.structures.len();
    let (respawned, _) = survival
        .0
        .step_with_positions(ServerTick(clock.tick), sprinting);
    if survival.0.structures.len() != prev_struct_count {
        physics.0.sync_structures(&survival.0.structures);
    }
    if !respawned.is_empty() {
        for mut marker in &mut nodes {
            if let Some(updated) = respawned.iter().find(|n| n.id == marker.0.id) {
                marker.0 = *updated;
            }
        }
    }
}

/// Deterministic clock advancement for CLI smoke runs and physics tests.
/// This does not promise bit-identical physics between CPU architectures.
pub fn build_headless_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, SharedSimulationPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / FIXED_HZ,
    )));
    app.finish();
    app.cleanup();
    app
}

#[cfg(test)]
mod tests {
    use super::*;

    fn advance(app: &mut App, count: u64) {
        let end = app.world().resource::<SimulationClock>().tick + count;
        let mut updates = 0;
        while app.world().resource::<SimulationClock>().tick < end {
            app.update();
            updates += 1;
            assert!(
                updates <= count * 3 + 10,
                "fixed schedule stopped advancing"
            );
        }
    }

    fn player_state(app: &mut App) -> (Vec3, MotionState) {
        let world = app.world_mut();
        let mut query = world.query_filtered::<(&Transform, &CharacterMotion), With<Player>>();
        let (transform, motion) = query.single(world).expect("one player");
        (transform.translation, motion.0)
    }

    #[test]
    fn capsule_lands_on_floor_and_cannot_pass_through_wall() {
        let mut app = build_headless_app();
        advance(&mut app, 120);
        let (position, motion) = player_state(&mut app);
        assert!(motion.grounded);
        assert!(
            (0.89..1.05).contains(&position.y),
            "floor position: {position:?}"
        );
        app.world_mut()
            .resource_mut::<SimulationInput>()
            .0
            .set_movement([0.0, 1.0], false);
        advance(&mut app, 360);
        let (position, _) = player_state(&mut app);
        assert!(
            (-4.3..-4.0).contains(&position.z),
            "wall position: {position:?}"
        );
    }

    #[test]
    fn jump_rises_then_lands_without_repeating() {
        let mut app = build_headless_app();
        advance(&mut app, 120);
        let floor = player_state(&mut app).0.y;
        app.world_mut()
            .resource_mut::<SimulationInput>()
            .0
            .queue_jump();
        advance(&mut app, 15);
        assert!(player_state(&mut app).0.y > floor + 0.5);
        advance(&mut app, 100);
        let (position, motion) = player_state(&mut app);
        assert!(motion.grounded);
        assert!((position.y - floor).abs() < 0.05);
    }

    #[test]
    fn controller_climbs_test_stairs() {
        let mut app = build_headless_app();
        advance(&mut app, 120);
        {
            let world = app.world_mut();
            let mut query =
                world.query_filtered::<(&mut Transform, &mut SimulationPose), With<Player>>();
            let (mut transform, mut pose) = query.single_mut(world).unwrap();
            transform.translation = Vec3::new(-10.0, 0.94, 6.0);
            pose.previous = transform.translation;
            pose.current = transform.translation;
        }
        advance(&mut app, 5);
        app.world_mut()
            .resource_mut::<SimulationInput>()
            .0
            .set_movement([0.0, 1.0], false);
        advance(&mut app, 90);
        let (position, _) = player_state(&mut app);
        assert!(
            position.z < 2.0 && position.y > 1.6,
            "stair position: {position:?}"
        );
    }
}
