//! Shared source clearance adapter over the explicitly classified collision scene.
//!
//! Layer/trigger coverage and initial overlap must be resolved before this
//! adapter returns a clearance result. This does not select a posture, rig pose
//! or runtime Physics.queriesHitTriggers value, and does not prove PhysX parity.

use super::*;
use midnight_game_data::clearance::{
    SourcePaUpGroundRay, SourcePaUpHeadCast, SourceStandClearanceCast,
};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceClearanceError {
    UnknownQueryTriggerPolicy,
    UnknownColliderLayers { count: usize },
    UnknownTriggerLayers { count: usize },
    UnrepresentedTriggers { count: usize },
    UnresolvedInitialOverlap { count: usize },
    UnknownPaUpPlayerTransform,
    UnknownPaUpRayLayer,
    UnknownPaUpElevatorCabin,
    UnresolvedInitialRayOverlap { count: usize },
    AmbiguousPaUpGroundHit { count: usize },
}

impl fmt::Display for SourceClearanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownQueryTriggerPolicy => {
                f.write_str("source Physics.queriesHitTriggers is unresolved")
            }
            Self::UnknownColliderLayers { count } => {
                write!(f, "{count} enabled colliders lack source layers")
            }
            Self::UnknownTriggerLayers { count } => {
                write!(f, "{count} enabled source triggers lack source layers")
            }
            Self::UnrepresentedTriggers { count } => {
                write!(f, "{count} included source triggers lack query geometry")
            }
            Self::UnresolvedInitialOverlap { count } => write!(
                f,
                "{count} initial sphere overlaps need source PhysX comparison"
            ),
            Self::UnknownPaUpPlayerTransform => {
                f.write_str("source PlayerTransform position is unresolved")
            }
            Self::UnknownPaUpRayLayer => {
                f.write_str("effective BattleScMgr.DefaultLayer is unresolved")
            }
            Self::UnknownPaUpElevatorCabin => {
                f.write_str("PaUp RaycastHit.transform ElevatorCabin parent query is unresolved")
            }
            Self::UnresolvedInitialRayOverlap { count } => write!(
                f,
                "{count} initial ray overlaps need source PhysX comparison"
            ),
            Self::AmbiguousPaUpGroundHit { count } => write!(
                f,
                "{count} equally near PaUp ray hits need source hit-order comparison"
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SourcePaUpClearanceResult {
    Clear,
    BlockedByHead { distance: f32 },
    BlockedByGroundComponent { distance: f32 },
}

impl SourcePaUpClearanceResult {
    pub const fn allows_pa_up(self) -> bool {
        matches!(self, Self::Clear)
    }
}

#[derive(Clone, Copy)]
struct SphereClearanceGeometry {
    origin: [f32; 3],
    radius: f32,
    distance: f32,
    layer_mask: u32,
}

impl CollisionWorld {
    /// Queries a separately supplied RootBone request against this shared scene.
    /// `queries_hit_triggers` is an explicit host value resolving source UseGlobal.
    /// No collider/self exclusion is inferred. Source prefabs and schema-3 terrain
    /// retain known layers. Legacy terrain, fixture and prototype player colliders
    /// remain unclassified until their original layer metadata is supplied.
    ///
    /// Initial overlap (including contact reported by Rapier) is rejected because
    /// original PhysX sweep/contact handling has not been compared. No scene or
    /// player state is mutated by successful or failed queries.
    pub fn check_source_stand_clearance(
        &self,
        cast: SourceStandClearanceCast,
        queries_hit_triggers: bool,
    ) -> Result<SourceClearanceResult, SourceClearanceError> {
        self.check_source_sphere_clearance(
            SphereClearanceGeometry {
                origin: cast.origin(),
                radius: cast.radius(),
                distance: cast.max_distance(),
                layer_mask: cast.layer_mask(),
            },
            queries_hit_triggers,
        )
    }

    /// Execute the two distinct CheckCanPaUp queries in source order.
    ///
    /// The source reads PlayerTransform and BattleScMgr.DefaultLayer only after
    /// the HeadTop sweep misses. `ground_ray` must sample those authoritative
    /// inputs at that point; errors for missing inputs are therefore lazy.
    /// `has_elevator_cabin` is called only for a unique nearest ground hit. It
    /// must resolve the source RaycastHit.transform and its
    /// GetComponentInParent<ElevatorCabin>(includeInactive: false) result.
    /// Native MethodSpec 79665 at source slot 0x45d63f0 establishes the type;
    /// dynamic hit-transform/component presence is not inferred from names.
    /// Return UnknownPaUpElevatorCabin when that runtime result is unavailable.
    ///
    /// Both query stages use the supplied UseGlobal trigger value, with no
    /// inferred player/self exclusion. No FSM, player, or presentation changes
    /// occur. These shared queries do not establish Unity/PhysX parity.
    pub fn check_source_pa_up_clearance(
        &self,
        head: SourcePaUpHeadCast,
        queries_hit_triggers: Option<bool>,
        ground_ray: impl FnOnce() -> Result<SourcePaUpGroundRay, SourceClearanceError>,
        has_elevator_cabin: impl FnOnce(ColliderHandle) -> Result<bool, SourceClearanceError>,
    ) -> Result<SourcePaUpClearanceResult, SourceClearanceError> {
        let queries_hit_triggers =
            queries_hit_triggers.ok_or(SourceClearanceError::UnknownQueryTriggerPolicy)?;
        if let SourceClearanceResult::Blocked { distance } = self.check_source_sphere_clearance(
            SphereClearanceGeometry {
                origin: head.origin(),
                radius: head.radius(),
                distance: head.max_distance(),
                layer_mask: head.layer_mask(),
            },
            queries_hit_triggers,
        )? {
            return Ok(SourcePaUpClearanceResult::BlockedByHead { distance });
        }
        let ground = ground_ray()?;
        self.validate_source_query_coverage(ground.layer_mask(), queries_hit_triggers)?;
        let selected = self.source_clearance_filter(ground.layer_mask(), queries_hit_triggers);
        let query = self.query(QueryFilter {
            predicate: Some(&selected),
            ..Default::default()
        });
        let origin = Vec3::from_array(ground.origin());
        let count = query.intersect_point(origin).count();
        if count > 0 {
            return Err(SourceClearanceError::UnresolvedInitialRayOverlap { count });
        }
        let ray = Ray::new(origin, Vec3::from_array(ground.direction()));
        let mut nearest: Option<(ColliderHandle, f32)> = None;
        let mut equal_hits = 0;
        for (handle, _, hit) in query.intersect_ray(ray, ground.max_distance(), true) {
            match nearest {
                Some((_, distance)) if hit.time_of_impact > distance => {}
                Some((_, distance)) if hit.time_of_impact == distance => equal_hits += 1,
                _ => {
                    nearest = Some((handle, hit.time_of_impact));
                    equal_hits = 1;
                }
            }
        }
        if equal_hits > 1 {
            return Err(SourceClearanceError::AmbiguousPaUpGroundHit { count: equal_hits });
        }
        if let Some((handle, distance)) = nearest
            && has_elevator_cabin(handle)?
        {
            return Ok(SourcePaUpClearanceResult::BlockedByGroundComponent { distance });
        }
        Ok(SourcePaUpClearanceResult::Clear)
    }

    fn validate_source_query_coverage(
        &self,
        layer_mask: u32,
        queries_hit_triggers: bool,
    ) -> Result<(), SourceClearanceError> {
        if queries_hit_triggers {
            let count = self
                .source_terrain_trigger_layers
                .iter()
                .filter(|layer| layer.is_none())
                .count();
            if count > 0 {
                return Err(SourceClearanceError::UnknownTriggerLayers { count });
            }
        }
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
        let in_mask = |layer: u8| layer_mask & (1u32 << layer) != 0;
        if queries_hit_triggers {
            let count = self
                .source_unrepresented_trigger_layers
                .values()
                .flatten()
                .filter(|&&layer| in_mask(layer))
                .count()
                + self
                    .source_terrain_trigger_layers
                    .iter()
                    .flatten()
                    .filter(|&&layer| in_mask(layer))
                    .count();
            if count > 0 {
                return Err(SourceClearanceError::UnrepresentedTriggers { count });
            }
        }
        Ok(())
    }

    fn source_clearance_filter(
        &self,
        layer_mask: u32,
        queries_hit_triggers: bool,
    ) -> impl Fn(ColliderHandle, &Collider) -> bool + '_ {
        move |handle: ColliderHandle, collider: &Collider| {
            collider.is_enabled()
                && (queries_hit_triggers || !collider.is_sensor())
                && self
                    .source_collision_layers
                    .get(&handle)
                    .is_some_and(|&layer| layer_mask & (1u32 << layer) != 0)
        }
    }

    fn check_source_sphere_clearance(
        &self,
        geometry: SphereClearanceGeometry,
        queries_hit_triggers: bool,
    ) -> Result<SourceClearanceResult, SourceClearanceError> {
        self.validate_source_query_coverage(geometry.layer_mask, queries_hit_triggers)?;
        let selected = self.source_clearance_filter(geometry.layer_mask, queries_hit_triggers);
        let query = self.query(QueryFilter {
            predicate: Some(&selected),
            ..Default::default()
        });
        let origin = geometry.origin;
        let pose = Pose::translation(origin[0], origin[1], origin[2]);
        let sphere = bevy_rapier3d::rapier::parry::shape::Ball::new(geometry.radius);
        let count = query.intersect_shape(pose, &sphere).count();
        if count > 0 {
            return Err(SourceClearanceError::UnresolvedInitialOverlap { count });
        }
        let hit = query.cast_shape(
            &pose,
            Vec3::Y,
            &sphere,
            ShapeCastOptions {
                max_time_of_impact: geometry.distance,
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

    fn pa_head(position: [f32; 3]) -> SourcePaUpHeadCast {
        SourcePaUpHeadCast::from_bevy_head_top_position(position).unwrap()
    }

    fn pa_ground(position: [f32; 3], layer: i32) -> SourcePaUpGroundRay {
        SourcePaUpGroundRay::from_bevy_player_transform_position(position, layer).unwrap()
    }

    #[test]
    fn pa_up_head_hit_precedes_player_transform_layer_and_component_reads() {
        let world = ceiling_world(0, 1.0, false);
        let result = world
            .check_source_pa_up_clearance(
                pa_head([0.0; 3]),
                Some(false),
                || panic!("a blocked HeadTop cast must not sample PlayerTransform or ray layer"),
                |_| panic!("a blocked HeadTop cast must not sample a ground component"),
            )
            .unwrap();
        let SourcePaUpClearanceResult::BlockedByHead { distance } = result else {
            panic!("head should hit the ceiling");
        };
        assert!((distance - 0.8).abs() < 0.001);
        assert!(!result.allows_pa_up());
    }

    #[test]
    fn pa_up_is_not_the_root_bone_pa_to_stand_cast() {
        let world = ceiling_world(0, 1.6, false);
        assert_eq!(
            world.check_source_pa_up_clearance(
                pa_head([0.0; 3]),
                Some(false),
                || Ok(pa_ground([0.0; 3], 0)),
                |_| panic!("there is no ground hit"),
            ),
            Ok(SourcePaUpClearanceResult::Clear)
        );
        assert!(
            !world
                .check_source_stand_clearance(
                    cast(SourceStandClearanceKind::PaToStand, [0.0; 3]),
                    false,
                )
                .unwrap()
                .allows_standing()
        );
    }

    #[test]
    fn pa_up_ground_ray_uses_its_own_layer_and_conditional_component_result() {
        let world = ceiling_world(17, -0.1, false);
        let expected_handle = world.source_prefabs["ceiling"][0];
        for present in [false, true] {
            let result = world
                .check_source_pa_up_clearance(
                    pa_head([8.0, 2.0, 8.0]),
                    Some(false),
                    || Ok(pa_ground([0.0; 3], 17)),
                    |handle| {
                        assert_eq!(handle, expected_handle);
                        Ok(present)
                    },
                )
                .unwrap();
            if present {
                let SourcePaUpClearanceResult::BlockedByGroundComponent { distance } = result
                else {
                    panic!("observed excluded parent component should block PaUp");
                };
                assert!((distance - 0.3).abs() < 0.001);
            } else {
                assert!(result.allows_pa_up());
            }
        }
        assert!(
            world
                .check_source_pa_up_clearance(
                    pa_head([0.0, 2.0, 0.0]),
                    Some(false),
                    || Ok(pa_ground([0.0; 3], 18)),
                    |_| panic!("ground on a different layer must not run the component check"),
                )
                .unwrap()
                .allows_pa_up()
        );
    }

    #[test]
    fn pa_up_missing_dependencies_are_errors_only_when_their_stage_is_reached() {
        let world = ceiling_world(17, -0.1, false);
        let head = pa_head([0.0, 2.0, 0.0]);
        assert_eq!(
            world.check_source_pa_up_clearance(
                head,
                None,
                || panic!("unknown trigger policy must stop the first query"),
                |_| panic!("unknown trigger policy must stop the first query"),
            ),
            Err(SourceClearanceError::UnknownQueryTriggerPolicy)
        );
        for error in [
            SourceClearanceError::UnknownPaUpPlayerTransform,
            SourceClearanceError::UnknownPaUpRayLayer,
        ] {
            assert_eq!(
                world.check_source_pa_up_clearance(
                    head,
                    Some(false),
                    || Err(error),
                    |_| panic!("missing ground-ray inputs must stop before component lookup"),
                ),
                Err(error)
            );
        }
        assert_eq!(
            world.check_source_pa_up_clearance(
                head,
                Some(false),
                || Ok(pa_ground([0.0; 3], 17)),
                |_| Err(SourceClearanceError::UnknownPaUpElevatorCabin),
            ),
            Err(SourceClearanceError::UnknownPaUpElevatorCabin)
        );
    }

    #[test]
    fn pa_up_ground_triggers_are_checked_after_the_head_query() {
        let world = ceiling_world(17, -0.1, true);
        let sampled_ground = std::cell::Cell::new(false);
        assert_eq!(
            world.check_source_pa_up_clearance(
                pa_head([0.0, 2.0, 0.0]),
                Some(true),
                || {
                    sampled_ground.set(true);
                    Ok(pa_ground([0.0; 3], 17))
                },
                |_| panic!("unrepresented ground trigger must not imply a component result"),
            ),
            Err(SourceClearanceError::UnrepresentedTriggers { count: 1 })
        );
        assert!(sampled_ground.get());
        assert!(
            world
                .check_source_pa_up_clearance(
                    pa_head([0.0, 2.0, 0.0]),
                    Some(false),
                    || Ok(pa_ground([0.0; 3], 17)),
                    |_| panic!("ignored ground trigger must not hit"),
                )
                .unwrap()
                .allows_pa_up()
        );
    }

    #[test]
    fn pa_up_ray_overlap_and_equal_hits_remain_unresolved() {
        let overlap = ceiling_world(17, 0.3, false);
        assert_eq!(
            overlap.check_source_pa_up_clearance(
                pa_head([0.0, 2.0, 0.0]),
                Some(false),
                || Ok(pa_ground([0.0; 3], 17)),
                |_| panic!("overlapping ray origin needs original physics evidence"),
            ),
            Err(SourceClearanceError::UnresolvedInitialRayOverlap { count: 1 })
        );
        let mut world = ceiling_world(17, -0.1, false);
        let mut duplicate = package();
        duplicate.manifest.nodes[0].layer = 17;
        duplicate.manifest.nodes[0].components[0].fields["m_Center"] =
            json!({"x":0.0,"y":-0.1,"z":0.0});
        duplicate.manifest.nodes[0].components[0].fields["m_Size"] =
            json!({"x":4.0,"y":0.2,"z":4.0});
        world
            .replace_source_prefab("duplicate", &duplicate, &source(1), IDENTITY)
            .unwrap();
        assert_eq!(
            world.check_source_pa_up_clearance(
                pa_head([0.0, 2.0, 0.0]),
                Some(false),
                || Ok(pa_ground([0.0; 3], 17)),
                |_| panic!("equal nearest hits must not select an arbitrary parent component"),
            ),
            Err(SourceClearanceError::AmbiguousPaUpGroundHit { count: 2 })
        );
    }

    #[test]
    fn pa_up_does_not_infer_self_exclusion_or_classify_prototype_players() {
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        world.set_player(PlayerId(8), PlayerState::at([0.0; 3]));
        assert_eq!(
            world.check_source_pa_up_clearance(
                pa_head([0.0, 0.8, 0.0]),
                Some(false),
                || panic!("unknown layers must stop the first query"),
                |_| panic!("unknown layers must stop the first query"),
            ),
            Err(SourceClearanceError::UnknownColliderLayers { count: 1 })
        );
        let handle = world.players[&PlayerId(8)].0;
        // Test-only source classification, not an inferred original player layer.
        world.source_collision_layers.insert(handle, 0);
        assert_eq!(
            world.check_source_pa_up_clearance(
                pa_head([0.0, 0.8, 0.0]),
                Some(false),
                || panic!("the caller's player is not automatically excluded"),
                |_| panic!("the caller's player is not automatically excluded"),
            ),
            Err(SourceClearanceError::UnresolvedInitialOverlap { count: 1 })
        );
    }

    #[test]
    fn pa_up_checks_only_the_nearest_ground_hit_and_shared_worlds_agree() {
        for _ in 0..2 {
            let mut world = ceiling_world(17, -0.1, false);
            let nearest_handle = world.source_prefabs["ceiling"][0];
            let mut lower = package();
            lower.manifest.nodes[0].layer = 17;
            lower.manifest.nodes[0].components[0].fields["m_Center"] =
                json!({"x":0.0,"y":-0.5,"z":0.0});
            lower.manifest.nodes[0].components[0].fields["m_Size"] =
                json!({"x":4.0,"y":0.2,"z":4.0});
            world
                .replace_source_prefab("lower", &lower, &source(1), IDENTITY)
                .unwrap();
            let calls = std::cell::Cell::new(0);
            assert_eq!(
                world.check_source_pa_up_clearance(
                    pa_head([0.0, 2.0, 0.0]),
                    Some(false),
                    || Ok(pa_ground([0.0; 3], 17)),
                    |handle| {
                        calls.set(calls.get() + 1);
                        assert_eq!(handle, nearest_handle);
                        Ok(false)
                    },
                ),
                Ok(SourcePaUpClearanceResult::Clear)
            );
            assert_eq!(calls.get(), 1);
            assert_eq!(world.colliders.len(), 2);
        }
    }
}
