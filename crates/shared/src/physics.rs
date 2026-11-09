//! Shared character queries. Updating the BVH after every character prevents
//! two kinematic capsules from moving into the same gap in a single tick.
use std::collections::BTreeMap;

use bevy::prelude::Vec3;
use bevy_rapier3d::rapier::{
    control::{CharacterAutostep, CharacterLength, KinematicCharacterController},
    parry::{
        math::Pose,
        partitioning::{Bvh, BvhWorkspace},
        query::ShapeCastOptions,
    },
    prelude::*,
};

use crate::{
    capsule_query::CharacterQueryDispatcher,
    input::PlayerInputCommand,
    level::{MapId, map_geometry},
    movement::{FIXED_DT, PLAYER_HALF_SEGMENT, PLAYER_RADIUS},
    protocol::{PlayerId, PlayerState},
};

pub struct CollisionWorld {
    #[cfg(test)]
    pub query_nanos: u128,
    #[cfg(test)]
    pub index_nanos: u128,
    pub map: MapId,
    colliders: ColliderSet,
    bodies: RigidBodySet,
    soft_bodies: SoftBodySet,
    islands: IslandManager,
    bvh: Bvh,
    bvh_workspace: BvhWorkspace,
    refreshes: u32,
    controller: KinematicCharacterController,
    capsule: SharedShape,
    players: BTreeMap<PlayerId, (ColliderHandle, PlayerState)>,
    structures: BTreeMap<u32, Vec<ColliderHandle>>,
    initial_colliders: Vec<ColliderHandle>,
}

impl CollisionWorld {
    pub fn new(map: MapId) -> Self {
        let mut world = Self {
            #[cfg(test)]
            query_nanos: 0,
            #[cfg(test)]
            index_nanos: 0,
            map,
            colliders: ColliderSet::new(),
            bodies: RigidBodySet::new(),
            soft_bodies: SoftBodySet::new(),
            islands: IslandManager::new(),
            bvh: Bvh::new(),
            bvh_workspace: BvhWorkspace::default(),
            refreshes: 0,
            controller: KinematicCharacterController {
                offset: CharacterLength::Absolute(0.02),
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(0.25),
                    min_width: CharacterLength::Absolute(0.2),
                    include_dynamic_bodies: false,
                }),
                max_slope_climb_angle: 45.0_f32.to_radians(),
                min_slope_slide_angle: 50.0_f32.to_radians(),
                snap_to_ground: Some(CharacterLength::Absolute(0.25)),
                ..Default::default()
            },
            capsule: SharedShape::capsule_y(PLAYER_HALF_SEGMENT, PLAYER_RADIUS),
            players: BTreeMap::new(),
            structures: BTreeMap::new(),
            initial_colliders: Vec::new(),
        };
        let mut handles = Vec::new();
        for shape in map_geometry(map) {
            let handle = world.colliders.insert(
                ColliderBuilder::cuboid(
                    shape.half_extents[0],
                    shape.half_extents[1],
                    shape.half_extents[2],
                )
                .translation(Vec3::from(shape.center))
                .rotation(Vec3::new(0.0, 0.0, shape.rotation_z)),
            );
            handles.push(handle);
            world.initial_colliders.push(handle);
        }
        for node in crate::survival::map_resource_nodes(map) {
            let r = node.kind.radius() * 0.65;
            let h = node.kind.height() * 0.5;
            handles.push(
                world
                    .colliders
                    .insert(ColliderBuilder::cuboid(r, h, r).translation(Vec3::new(
                        node.position[0],
                        node.position[1] + h,
                        node.position[2],
                    ))),
            );
        }
        world.refresh(&handles, &[]);
        world
    }

    fn refresh(&mut self, changed: &[ColliderHandle], removed: &[ColliderHandle]) {
        #[cfg(test)]
        let started = std::time::Instant::now();
        // This world only runs scene queries: maintaining a physics pair/event
        // graph for every sequential character move would be wasted work.
        for handle in removed {
            self.bvh.remove(handle.into_raw_parts().0);
        }
        let parameters = IntegrationParameters {
            dt: FIXED_DT,
            ..Default::default()
        };
        for handle in changed {
            let bounds =
                self.colliders[*handle].compute_broad_phase_aabb(&parameters, &self.bodies);
            self.bvh
                .insert_or_update_partially(bounds, handle.into_raw_parts().0, 0.01);
        }
        self.refreshes = self.refreshes.wrapping_add(1);
        if self.refreshes.is_multiple_of(32) || changed.len() > 1 {
            self.bvh.optimize_incremental(&mut self.bvh_workspace);
            self.bvh.refit(&mut self.bvh_workspace);
        } else {
            // Bounds are committed after EVERY move, even between periodic
            // topology optimizations, so later players see earlier movement.
            self.bvh.refit_without_opt();
        }
        // This world uses explicit query updates, not PhysicsPipeline::step,
        // so consume Rapier's mutation journals here, including despawn churn.
        self.colliders.take_modified();
        self.colliders.take_removed();
        #[cfg(test)]
        {
            self.index_nanos += started.elapsed().as_nanos();
        }
    }

    pub fn state(&self, id: PlayerId) -> Option<PlayerState> {
        self.players.get(&id).map(|p| p.1)
    }

    fn query<'a>(&'a self, filter: QueryFilter<'a>) -> QueryPipeline<'a> {
        QueryPipeline {
            dispatcher: &CharacterQueryDispatcher,
            bvh: &self.bvh,
            bodies: &self.bodies,
            colliders: &self.colliders,
            filter,
        }
    }

    pub fn states(&self) -> impl Iterator<Item = (PlayerId, PlayerState)> + '_ {
        self.players.iter().map(|(&id, &(_, state))| (id, state))
    }

    pub fn set_player(&mut self, id: PlayerId, state: PlayerState) {
        if let Some(handle) = self.upsert(id, state) {
            self.refresh(&[handle], &[]);
        }
    }

    /// Collision proxies can be refreshed together; authoritative movement
    /// still commits each player separately before stepping the next one.
    pub fn set_players(&mut self, states: impl IntoIterator<Item = (PlayerId, PlayerState)>) {
        let handles: Vec<_> = states
            .into_iter()
            .filter_map(|(id, state)| self.upsert(id, state))
            .collect();
        if !handles.is_empty() {
            self.refresh(&handles, &[]);
        }
    }

    fn upsert(&mut self, id: PlayerId, state: PlayerState) -> Option<ColliderHandle> {
        let handle = if let Some((handle, old)) = self.players.get_mut(&id) {
            let changed = old.position != state.position;
            *old = state;
            if !changed {
                return None;
            }
            self.colliders[*handle].set_translation(Vec3::from(state.position));
            *handle
        } else {
            let handle = self.colliders.insert(
                ColliderBuilder::new(self.capsule.clone())
                    .translation(Vec3::from(state.position)),
            );
            self.players.insert(id, (handle, state));
            handle
        };
        Some(handle)
    }

    pub fn remove_player(&mut self, id: PlayerId) {
        if let Some((handle, _)) = self.players.remove(&id) {
            self.colliders
                .remove(handle, &mut self.islands, &mut self.bodies, &mut self.soft_bodies, false);
            self.refresh(&[], &[handle]);
        }
    }

    pub fn insert_structure(&mut self, structure: crate::survival::PlacedStructure) {
        if self.structures.contains_key(&structure.id) {
            return;
        }
        let mut handles = Vec::new();
        let pos = structure.position;
        if structure.kind == crate::survival::StructureKind::Doorway {
            if structure.rotation % 2 == 1 {
                for offset_z in [-1.05, 1.05] {
                    handles.push(self.colliders.insert(
                        ColliderBuilder::cuboid(0.15, 1.25, 0.45).translation(Vec3::new(
                            pos[0],
                            pos[1],
                            pos[2] + offset_z,
                        )),
                    ));
                }
            } else {
                for offset_x in [-1.05, 1.05] {
                    handles.push(self.colliders.insert(
                        ColliderBuilder::cuboid(0.45, 1.25, 0.15).translation(Vec3::new(
                            pos[0] + offset_x,
                            pos[1],
                            pos[2],
                        )),
                    ));
                }
            }
        } else {
            let ext = structure.kind.oriented_half_extents(structure.rotation);
            handles.push(self.colliders.insert(
                ColliderBuilder::cuboid(ext[0], ext[1], ext[2]).translation(Vec3::from(pos)),
            ));
        }
        self.refresh(&handles, &[]);
        self.structures.insert(structure.id, handles);
    }

    pub fn sync_structures(&mut self, structures: &[crate::survival::PlacedStructure]) {
        let mut removed_handles = Vec::new();
        self.structures.retain(|id, handles| {
            let keep = structures.iter().any(|s| s.id == *id);
            if !keep {
                for &h in handles.iter() {
                    self.colliders
                        .remove(h, &mut self.islands, &mut self.bodies, &mut self.soft_bodies, false);
                    removed_handles.push(h);
                }
            }
            keep
        });
        if !removed_handles.is_empty() {
            self.refresh(&[], &removed_handles);
        }
        for &structure in structures {
            self.insert_structure(structure);
        }
    }

    pub fn insert_imported_terrain(&mut self, chunks: &[crate::level::ImportedMeshRecord]) {
        let removed = std::mem::take(&mut self.initial_colliders);
        for &h in &removed {
            self.colliders
                .remove(h, &mut self.islands, &mut self.bodies, &mut self.soft_bodies, false);
        }
        let mut handles = Vec::new();
        for chunk in chunks {
            let lower = chunk.name.to_ascii_lowercase();
            if lower.contains("road")
                || lower.contains("jichang")
                || lower.contains("gangkou")
                || lower.contains("feiqichangfang")
                || lower.contains("maoyiqu")
                || lower.contains("shiyanshi")
                || lower.contains("object006")
                || chunk.indices.len() < 3
            {
                continue;
            }
            let vertices: Vec<Vec3> = chunk
                .positions
                .iter()
                .map(|p| Vec3::new(p[0], p[1], p[2]))
                .collect();
            let tris: Vec<[u32; 3]> = chunk.indices.as_chunks::<3>().0.to_vec();
            if let Ok(builder) = ColliderBuilder::trimesh(vertices, tris) {
                handles.push(self.colliders.insert(builder));
            }
        }
        if !handles.is_empty() || !removed.is_empty() {
            self.refresh(&handles, &removed);
        }
    }

    pub fn free_at(&self, position: [f32; 3], exclude: Option<PlayerId>) -> bool {
        let filter = QueryFilter {
            exclude_collider: exclude.and_then(|id| self.players.get(&id).map(|p| p.0)),
            ..Default::default()
        };
        let query = self.query(filter);
        query
            .intersect_shape(
                Pose::translation(position[0], position[1], position[2]),
                &*self.capsule,
            )
            .next()
            .is_none()
    }

    pub fn cast_ray(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
        max_distance: f32,
        exclude: Option<PlayerId>,
    ) -> Option<([f32; 3], [f32; 3], f32)> {
        let dir = Vec3::new(direction[0], direction[1], direction[2]);
        let norm_dir = dir.normalize_or_zero();
        if !norm_dir.x.is_finite() || !norm_dir.y.is_finite() || !norm_dir.z.is_finite() || norm_dir == Vec3::ZERO {
            return None;
        }
        let ray = Ray::new(Vec3::new(origin[0], origin[1], origin[2]), norm_dir);
        let filter = QueryFilter {
            exclude_collider: exclude.and_then(|id| self.players.get(&id).map(|p| p.0)),
            ..Default::default()
        };
        let query = self.query(filter);
        if let Some((_handle, intersection)) =
            query.cast_ray_and_get_normal(&ray, max_distance, true)
        {
            let hit_point = ray.point_at(intersection.time_of_impact);
            let normal = intersection.normal;
            Some((
                [hit_point.x, hit_point.y, hit_point.z],
                [normal.x, normal.y, normal.z],
                intersection.time_of_impact,
            ))
        } else {
            None
        }
    }

    pub fn spawn_position(&self, exclude: Option<PlayerId>) -> Option<[f32; 3]> {
        // The south of the polygon is deliberately clear of test obstacles.
        for row in 0..12 {
            for col in 0..20 {
                let p = [-14.25 + col as f32 * 1.5, 2.0, 9.0 + row as f32 * 1.5];
                if self.free_at(p, exclude) {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn step(&mut self, id: PlayerId, command: PlayerInputCommand) -> Option<PlayerState> {
        let (handle, mut state) = *self.players.get(&id)?;
        let requested = state.motion.plan_step(command);
        #[cfg(test)]
        let started = std::time::Instant::now();
        let query = self.query(QueryFilter::default().exclude_collider(handle));
        let position =
            Pose::translation(state.position[0], state.position[1], state.position[2]);
        let mut controller = self.controller;
        // Autostep performs several expensive extra sweeps per contact. Only
        // enable it when static geometry actually obstructs this horizontal
        // step; other player capsules must be blockers, not automatic stairs.
        let horizontal = Vec3::new(requested[0], 0.0, requested[2]);
        let distance = horizontal.length();
        let static_geometry =
            |_: ColliderHandle, collider: &Collider| collider.shape().as_capsule().is_none();
        let mut stair_query = query;
        stair_query.filter.predicate = Some(&static_geometry);
        if distance < 1e-5
            || stair_query
                .cast_shape(
                    &position,
                    horizontal / distance,
                    &*self.capsule,
                    ShapeCastOptions {
                        max_time_of_impact: distance,
                        target_distance: 0.02,
                        stop_at_penetration: false,
                        compute_impact_geometry_on_penetration: true,
                    },
                )
                .is_none()
        {
            controller.autostep = None;
        }
        let movement = controller.move_shape(
            FIXED_DT,
            &query,
            &*self.capsule,
            &position,
            Vec3::from(requested),
            |_| {},
        );
        #[cfg(test)]
        {
            self.query_nanos += started.elapsed().as_nanos();
        }
        state
            .motion
            .finish_step(movement.grounded, requested[1], movement.translation.y);
        for axis in 0..3 {
            state.position[axis] += movement.translation[axis];
            state.velocity[axis] = movement.translation[axis] / FIXED_DT;
        }
        if state.position[1] < -80.0 || !state.position.iter().all(|v| v.is_finite()) {
            state = PlayerState::at(self.spawn_position(Some(id)).unwrap_or([0.0, 2.0, 12.0]));
        }
        self.set_player(id, state);
        Some(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposing_capsules_cannot_exchange_sides_or_overlap() {
        let mut world = CollisionWorld::new(MapId::Polygon);
        let walls: Vec<_> = [-0.6, 0.6]
            .into_iter()
            .map(|offset| {
                world.colliders.insert(
                    ColliderBuilder::cuboid(8.0, 2.0, 0.2).translation(Vec3::new(
                        0.0,
                        2.0,
                        12.0 + offset,
                    )),
                )
            })
            .collect();
        world.refresh(&walls, &[]);
        world.set_player(PlayerId(1), PlayerState::at([-3.0, 2.0, 12.0]));
        world.set_player(PlayerId(2), PlayerState::at([3.0, 2.0, 12.0]));
        for _ in 0..300 {
            for (id, direction) in [(PlayerId(1), 1.0), (PlayerId(2), -1.0)] {
                world.step(
                    id,
                    PlayerInputCommand {
                        tick_id: 0,
                        move_vec: [direction, 0.0],
                        look_delta: [0.0, 0.0],
                        jump: false,
                        sprint: true,
                    },
                );
            }
            let p1 = world.state(PlayerId(1)).unwrap().position;
            let p2 = world.state(PlayerId(2)).unwrap().position;
            assert!(p1[0] < p2[0], "p1={p1:?} p2={p2:?}");
            let dx = p2[0] - p1[0];
            let dy = p2[1] - p1[1];
            let dz = p2[2] - p1[2];
            assert!(
                dx * dx + dy * dy + dz * dz > (PLAYER_RADIUS * 2.0).powi(2),
                "overlap p1={p1:?} p2={p2:?}"
            );
        }
    }

    #[test]
    fn step_benchmark_reports_timings() {
        let mut world = CollisionWorld::new(MapId::Polygon);
        for id in 0..64 {
            let pid = PlayerId(id + 1);
            let state = PlayerState::at([
                -12.0 + (id % 16) as f32 * 1.5,
                2.0,
                10.0 + (id / 16) as f32 * 1.5,
            ]);
            world.set_player(pid, state);
        }
        for tick in 0..60 {
            for id in 0..64 {
                let pid = PlayerId(id + 1);
                let angle = (tick + id) as f32 * 0.1;
                world.step(
                    pid,
                    PlayerInputCommand {
                        tick_id: tick as u64,
                        move_vec: [angle.cos(), angle.sin()],
                        look_delta: [0.0, 0.0],
                        jump: tick % 20 == 0,
                        sprint: id % 2 == 0,
                    },
                );
            }
        }
        println!(
            "physics 64 players 60 ticks: query={:.2}ms index={:.2}ms",
            world.query_nanos as f64 / 1_000_000.0,
            world.index_nanos as f64 / 1_000_000.0
        );
    }
}
