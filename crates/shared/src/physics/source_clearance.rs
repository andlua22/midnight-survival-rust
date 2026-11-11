//! Shared source clearance adapter over the explicitly classified collision scene.
//!
//! Layer/trigger coverage and initial overlap must be resolved before this
//! adapter returns a clearance result. This does not select a posture, rig pose
//! or runtime Physics.queriesHitTriggers value, and does not prove PhysX parity.

use super::*;
use midnight_game_data::clearance::SourceStandClearanceCast;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceClearanceError {
    UnknownColliderLayers { count: usize },
    UnrepresentedTriggers { count: usize },
    UnresolvedInitialOverlap { count: usize },
}

impl fmt::Display for SourceClearanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownColliderLayers { count } => {
                write!(f, "{count} enabled colliders lack source layers")
            }
            Self::UnrepresentedTriggers { count } => {
                write!(f, "{count} included source triggers lack query geometry")
            }
            Self::UnresolvedInitialOverlap { count } => write!(
                f,
                "{count} initial sphere overlaps need source PhysX comparison"
            ),
        }
    }
}
impl std::error::Error for SourceClearanceError {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SourceClearanceResult {
    Clear,
    Blocked { distance: f32 },
}

impl SourceClearanceResult {
    /// The recovered callers return the inverse of the SphereCast hit bool.
    pub const fn allows_standing(self) -> bool {
        matches!(self, Self::Clear)
    }
}

impl CollisionWorld {
    /// Queries a separately supplied RootBone request against this shared scene.
    /// `queries_hit_triggers` is an explicit host value resolving source UseGlobal.
    /// No collider/self exclusion is inferred. Source prefabs retain known layers;
    /// terrain, fixture and prototype player colliders remain unclassified until
    /// their original layer metadata is supplied by a future importer/adapter.
    ///
    /// Initial overlap (including contact reported by Rapier) is rejected because
    /// original PhysX sweep/contact handling has not been compared. No scene or
    /// player state is mutated by successful or failed queries.
    pub fn check_source_stand_clearance(
        &self,
        cast: SourceStandClearanceCast,
        queries_hit_triggers: bool,
    ) -> Result<SourceClearanceResult, SourceClearanceError> {
        let unknown = self
            .colliders
            .iter()
            .filter(|(handle, collider)| {
                collider.is_enabled()
                    && (queries_hit_triggers || !collider.is_sensor())
                    && !self.source_collision_layers.contains_key(handle)
            })
            .count();
        if unknown > 0 {
            return Err(SourceClearanceError::UnknownColliderLayers { count: unknown });
        }
        let in_mask = |layer: u8| cast.layer_mask() & (1u32 << layer) != 0;
        if queries_hit_triggers {
            let count = self
                .source_unrepresented_trigger_layers
                .values()
                .flatten()
                .filter(|&&layer| in_mask(layer))
                .count();
            if count > 0 {
                return Err(SourceClearanceError::UnrepresentedTriggers { count });
            }
        }
        let selected = |handle: ColliderHandle, collider: &Collider| {
            collider.is_enabled()
                && (queries_hit_triggers || !collider.is_sensor())
                && self
                    .source_collision_layers
                    .get(&handle)
                    .is_some_and(|&layer| in_mask(layer))
        };
        let query = self.query(QueryFilter {
            predicate: Some(&selected),
            ..Default::default()
        });
        let origin = cast.origin();
        let pose = Pose::translation(origin[0], origin[1], origin[2]);
        let sphere = bevy_rapier3d::rapier::parry::shape::Ball::new(cast.radius());
        let count = query.intersect_shape(pose, &sphere).count();
        if count > 0 {
            return Err(SourceClearanceError::UnresolvedInitialOverlap { count });
        }
        let hit = query.cast_shape(
            &pose,
            Vec3::from_array(cast.direction()),
            &sphere,
            ShapeCastOptions {
                max_time_of_impact: cast.max_distance(),
                target_distance: 0.0,
                stop_at_penetration: true,
                compute_impact_geometry_on_penetration: true,
            },
        );
        Ok(match hit {
            Some((_, hit)) => SourceClearanceResult::Blocked {
                distance: hit.time_of_impact,
            },
            None => SourceClearanceResult::Clear,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::source_prefabs::tests::{IDENTITY, package, source};
    use super::*;
    use midnight_game_data::clearance::SourceStandClearanceKind;
    use serde_json::json;

    fn cast(kind: SourceStandClearanceKind, origin: [f32; 3]) -> SourceStandClearanceCast {
        SourceStandClearanceCast::from_bevy_root_bone_position(kind, origin).unwrap()
    }
    fn ceiling_world(layer: i64, height: f32, trigger: bool) -> CollisionWorld {
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        let mut prefab = package();
        prefab.manifest.nodes[0].layer = layer;
        prefab.manifest.nodes[0].components[0].fields["m_Center"] =
            json!({"x":0.0,"y":height,"z":0.0});
        prefab.manifest.nodes[0].components[0].fields["m_Size"] = json!({"x":4.0,"y":0.2,"z":4.0});
        prefab.manifest.nodes[0].components[0].fields["m_IsTrigger"] = json!(trigger);
        world
            .replace_source_prefab("ceiling", &prefab, &source(1), IDENTITY)
            .unwrap();
        world
    }

    #[test]
    fn source_cast_radius_lengths_and_root_bone_origin_determine_clearance() {
        // Lower face 1.7; sphere radius .2 hits after travel 1.5. A .02 margin
        // keeps this fixture away from exact-touch floating-point comparisons.
        let world = ceiling_world(0, 1.82, false);
        let crouch = cast(SourceStandClearanceKind::CrouchUp, [0.0; 3]);
        let pa = cast(SourceStandClearanceKind::PaToStand, [0.0; 3]);
        assert!(
            world
                .check_source_stand_clearance(crouch, false)
                .unwrap()
                .allows_standing()
        );
        let SourceClearanceResult::Blocked { distance } =
            world.check_source_stand_clearance(pa, false).unwrap()
        else {
            panic!("Pa cast should hit ceiling")
        };
        assert!((distance - 1.52).abs() < 0.001);
        let raised = cast(SourceStandClearanceKind::CrouchUp, [0.0, 0.4, 0.0]);
        assert!(
            !world
                .check_source_stand_clearance(raised, false)
                .unwrap()
                .allows_standing()
        );
        let moved = cast(SourceStandClearanceKind::PaToStand, [8.0, 0.0, 0.0]);
        assert!(
            world
                .check_source_stand_clearance(moved, false)
                .unwrap()
                .allows_standing()
        );
    }

    #[test]
    fn exact_unity_layers_filter_both_sweep_and_initial_overlap() {
        for layer in [0, 19, 30] {
            let world = ceiling_world(layer, 1.0, false);
            let request = cast(SourceStandClearanceKind::CrouchUp, [0.0; 3]);
            assert!(
                !world
                    .check_source_stand_clearance(request, false)
                    .unwrap()
                    .allows_standing()
            );
        }
        for layer in [1, 18, 20, 29, 31] {
            let world = ceiling_world(layer, 0.0, false);
            let request = cast(SourceStandClearanceKind::CrouchUp, [0.0; 3]);
            assert!(
                world
                    .check_source_stand_clearance(request, false)
                    .unwrap()
                    .allows_standing()
            );
        }
    }

    #[test]
    fn unknown_layers_triggers_and_initial_overlap_never_imply_clearance() {
        let mut unknown = CollisionWorld::new(MapId::ImportedIsland);
        unknown.set_player(PlayerId(3), PlayerState::at([20.0, 0.0, 0.0]));
        let request = cast(SourceStandClearanceKind::CrouchUp, [0.0; 3]);
        assert_eq!(
            unknown.check_source_stand_clearance(request, false),
            Err(SourceClearanceError::UnknownColliderLayers { count: 1 })
        );
        let trigger = ceiling_world(19, 1.0, true);
        assert!(
            trigger
                .check_source_stand_clearance(request, false)
                .unwrap()
                .allows_standing()
        );
        assert_eq!(
            trigger.check_source_stand_clearance(request, true),
            Err(SourceClearanceError::UnrepresentedTriggers { count: 1 })
        );
        let ignored_trigger = ceiling_world(31, 1.0, true);
        assert!(
            ignored_trigger
                .check_source_stand_clearance(request, true)
                .unwrap()
                .allows_standing()
        );
        let overlapping = ceiling_world(0, 0.0, false);
        assert_eq!(
            overlapping.check_source_stand_clearance(request, false),
            Err(SourceClearanceError::UnresolvedInitialOverlap { count: 1 })
        );
    }

    #[test]
    fn independent_shared_worlds_agree_and_queries_leave_players_unchanged() {
        let mut first = ceiling_world(30, 1.0, false);
        let mut second = ceiling_world(30, 1.0, false);
        let request = cast(SourceStandClearanceKind::CrouchUp, [0.0; 3]);
        assert_eq!(
            first.check_source_stand_clearance(request, false),
            second.check_source_stand_clearance(request, false)
        );
        let player = PlayerId(5);
        let original = PlayerState::at([20.0, 10.0, 0.0]);
        for world in [&mut first, &mut second] {
            world.set_player(player, original);
            assert_eq!(
                world.check_source_stand_clearance(request, false),
                Err(SourceClearanceError::UnknownColliderLayers { count: 1 })
            );
            assert_eq!(world.state(player), Some(original));
            assert_eq!(world.states().count(), 1);
            assert_eq!(world.colliders.len(), 2);
        }
    }
}
