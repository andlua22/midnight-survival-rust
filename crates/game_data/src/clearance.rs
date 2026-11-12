//! Source posture clearance requests, with explicitly supplied authoritative pose.
//!
//! The standing methods cast from RootBone.position. CheckCanPaUp instead
//! casts from HeadTop, then conditionally raycasts below PlayerTransform.
//! Actor feet/capsule center and active skeleton pose are not inferred. Native
//! PhysX results remain a separate acceptance boundary. Evidence:
//! memory/evidence/player-clearance-native/ and player-pa-up-native/.

use crate::controller::{SourceControllerError, finite};

/// The two distinct native callers retain different maximum cast distances.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceStandClearanceKind {
    CrouchUp,
    PaToStand,
}

impl SourceStandClearanceKind {
    pub const fn source_method(self) -> &'static str {
        match self {
            Self::CrouchUp => "PlayerController.CheckCanCrouchUp",
            Self::PaToStand => "PlayerController.CheckCanPaToStand",
        }
    }
}

/// Numeric Unity contract observed in the recovered SphereCast overload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SourceQueryTriggerInteraction {
    UseGlobal = 0,
}

pub const SOURCE_STAND_CLEARANCE_RADIUS: f32 = 0.2;
pub const SOURCE_STAND_CLEARANCE_LAYER_MASK: u32 = 0x4008_0001;
/// Native MethodSpec 79665 resolves GetComponentInParent<ElevatorCabin>.
pub const SOURCE_PA_UP_EXCLUDED_PARENT_COMPONENT: &str = "ElevatorCabin";

/// A validated request in Bevy world coordinates. RootBone must be supplied
/// by the caller from an identified rig pose, not substituted with player
/// position/collider center. Finite validation is reconstruction admission.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceStandClearanceCast {
    kind: SourceStandClearanceKind,
    root_bone_position: [f32; 3],
}

impl SourceStandClearanceCast {
    pub fn from_bevy_root_bone_position(
        kind: SourceStandClearanceKind,
        position: [f32; 3],
    ) -> Result<Self, SourceControllerError> {
        for (value, field) in position
            .into_iter()
            .zip(["RootBone.x", "RootBone.y", "RootBone.z"])
        {
            finite(value, field)?;
        }
        Ok(Self {
            kind,
            root_bone_position: position,
        })
    }

    /// Reflect raw Unity world Z exactly once. Already converted Bevy poses
    /// must use the other constructor. Both engines use world Y as up here.
    pub fn from_unity_root_bone_position(
        kind: SourceStandClearanceKind,
        position: [f32; 3],
    ) -> Result<Self, SourceControllerError> {
        Self::from_bevy_root_bone_position(kind, [position[0], position[1], -position[2]])
    }

    pub const fn kind(self) -> SourceStandClearanceKind {
        self.kind
    }
    pub const fn origin(self) -> [f32; 3] {
        self.root_bone_position
    }
    pub const fn direction(self) -> [f32; 3] {
        [0.0, 1.0, 0.0]
    }
    pub const fn radius(self) -> f32 {
        SOURCE_STAND_CLEARANCE_RADIUS
    }
    pub const fn max_distance(self) -> f32 {
        match self.kind {
            SourceStandClearanceKind::CrouchUp => 1.5,
            SourceStandClearanceKind::PaToStand => 1.9,
        }
    }
    pub const fn layer_mask(self) -> u32 {
        SOURCE_STAND_CLEARANCE_LAYER_MASK
    }
    pub const fn query_trigger_interaction(self) -> SourceQueryTriggerInteraction {
        SourceQueryTriggerInteraction::UseGlobal
    }
}

/// The first query in CheckCanPaUp (0x1ec332c), sampled from HeadTop.
/// Its smaller radius and shorter travel are distinct from PaToStand.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourcePaUpHeadCast {
    head_top_position: [f32; 3],
}

impl SourcePaUpHeadCast {
    pub fn from_bevy_head_top_position(position: [f32; 3]) -> Result<Self, SourceControllerError> {
        for (value, field) in position
            .into_iter()
            .zip(["HeadTop.x", "HeadTop.y", "HeadTop.z"])
        {
            finite(value, field)?;
        }
        Ok(Self {
            head_top_position: position,
        })
    }

    /// Reflect raw Unity Z exactly once; use the other constructor for Bevy poses.
    pub fn from_unity_head_top_position(position: [f32; 3]) -> Result<Self, SourceControllerError> {
        Self::from_bevy_head_top_position([position[0], position[1], -position[2]])
    }

    pub const fn origin(self) -> [f32; 3] {
        self.head_top_position
    }
    pub const fn direction(self) -> [f32; 3] {
        [0.0, 1.0, 0.0]
    }
    pub const fn radius(self) -> f32 {
        0.1
    }
    pub const fn max_distance(self) -> f32 {
        1.3
    }
    pub const fn layer_mask(self) -> u32 {
        SOURCE_STAND_CLEARANCE_LAYER_MASK
    }
    pub const fn query_trigger_interaction(self) -> SourceQueryTriggerInteraction {
        SourceQueryTriggerInteraction::UseGlobal
    }
}

/// The second query in CheckCanPaUp. The source reads PlayerTransform only
/// after the HeadTop cast misses, then reads BattleScMgr.DefaultLayer.
/// Supply the effective source layer; its name does not establish its value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourcePaUpGroundRay {
    player_transform_position: [f32; 3],
    default_layer: i32,
}

impl SourcePaUpGroundRay {
    pub fn from_bevy_player_transform_position(
        position: [f32; 3],
        default_layer: i32,
    ) -> Result<Self, SourceControllerError> {
        for (value, field) in position.into_iter().zip([
            "PlayerTransform.x",
            "PlayerTransform.y",
            "PlayerTransform.z",
        ]) {
            finite(value, field)?;
        }
        Ok(Self {
            player_transform_position: position,
            default_layer,
        })
    }

    pub fn from_unity_player_transform_position(
        position: [f32; 3],
        default_layer: i32,
    ) -> Result<Self, SourceControllerError> {
        Self::from_bevy_player_transform_position(
            [position[0], position[1], -position[2]],
            default_layer,
        )
    }

    pub fn origin(self) -> [f32; 3] {
        let [x, y, z] = self.player_transform_position;
        [x, y + 0.3, z]
    }
    pub const fn direction(self) -> [f32; 3] {
        [0.0, -1.0, 0.0]
    }
    pub const fn max_distance(self) -> f32 {
        1.0
    }
    pub const fn default_layer(self) -> i32 {
        self.default_layer
    }
    pub const fn excluded_parent_component(self) -> &'static str {
        SOURCE_PA_UP_EXCLUDED_PARENT_COMPONENT
    }
    pub const fn include_inactive_parents(self) -> bool {
        false
    }
    pub const fn layer_mask(self) -> u32 {
        // C# int shifts and AArch64 LSL W both mask the shift count to five bits.
        1_u32.wrapping_shl(self.default_layer as u32)
    }
    pub const fn query_trigger_interaction(self) -> SourceQueryTriggerInteraction {
        SourceQueryTriggerInteraction::UseGlobal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_methods_preserve_rig_origin_and_distinct_cast_lengths() {
        for (kind, length) in [
            (SourceStandClearanceKind::CrouchUp, 1.5),
            (SourceStandClearanceKind::PaToStand, 1.9),
        ] {
            let cast =
                SourceStandClearanceCast::from_bevy_root_bone_position(kind, [7.0, 11.0, -5.0])
                    .unwrap();
            assert_eq!(cast.origin(), [7.0, 11.0, -5.0]);
            assert_eq!(cast.direction(), [0.0, 1.0, 0.0]);
            assert_eq!(cast.radius().to_bits(), 0x3e4c_cccd);
            assert_eq!(cast.max_distance(), length);
            assert_eq!(cast.layer_mask(), (1 << 0) | (1 << 19) | (1 << 30));
            assert_eq!(cast.query_trigger_interaction() as i32, 0);
        }
    }

    #[test]
    fn unity_root_position_is_reflected_once_without_an_actor_offset() {
        let unity = SourceStandClearanceCast::from_unity_root_bone_position(
            SourceStandClearanceKind::CrouchUp,
            [3.0, 4.0, 6.0],
        )
        .unwrap();
        let bevy = SourceStandClearanceCast::from_bevy_root_bone_position(
            SourceStandClearanceKind::CrouchUp,
            [3.0, 4.0, -6.0],
        )
        .unwrap();
        assert_eq!(unity, bevy);
    }

    #[test]
    fn non_finite_rig_pose_is_rejected_in_each_coordinate_frame() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for axis in 0..3 {
                let mut position = [0.0; 3];
                position[axis] = invalid;
                assert!(
                    SourceStandClearanceCast::from_bevy_root_bone_position(
                        SourceStandClearanceKind::CrouchUp,
                        position
                    )
                    .is_err()
                );
                assert!(
                    SourceStandClearanceCast::from_unity_root_bone_position(
                        SourceStandClearanceKind::PaToStand,
                        position
                    )
                    .is_err()
                );
            }
        }
    }

    #[test]
    fn pa_up_keeps_distinct_head_and_player_transform_queries() {
        let head = SourcePaUpHeadCast::from_unity_head_top_position([2.0, 4.0, 6.0]).unwrap();
        assert_eq!(head.origin(), [2.0, 4.0, -6.0]);
        assert_eq!(head.direction(), [0.0, 1.0, 0.0]);
        assert_eq!(head.radius().to_bits(), 0x3dcc_cccd);
        assert_eq!(head.max_distance().to_bits(), 0x3fa6_6666);
        assert_eq!(head.layer_mask(), 0x4008_0001);
        assert_eq!(head.query_trigger_interaction() as i32, 0);
        let ray =
            SourcePaUpGroundRay::from_unity_player_transform_position([8.0, 1.0, 9.0], 19).unwrap();
        assert_eq!(ray.origin(), [8.0, 1.3, -9.0]);
        assert_eq!(ray.direction(), [0.0, -1.0, 0.0]);
        assert_eq!(ray.max_distance(), 1.0);
        assert_eq!(ray.default_layer(), 19);
        assert_eq!(ray.excluded_parent_component(), "ElevatorCabin");
        assert!(!ray.include_inactive_parents());
        assert_eq!(ray.layer_mask(), 1 << 19);
        assert_eq!(ray.query_trigger_interaction() as i32, 0);
        assert_eq!(
            ray,
            SourcePaUpGroundRay::from_bevy_player_transform_position([8.0, 1.0, -9.0], 19).unwrap()
        );
    }

    #[test]
    fn pa_up_preserves_signed_source_layer_shift_without_guessing_a_default() {
        for (layer, expected) in [(-1, 1 << 31), (0, 1), (31, 1 << 31), (32, 1), (65, 2)] {
            let ray =
                SourcePaUpGroundRay::from_bevy_player_transform_position([0.0; 3], layer).unwrap();
            assert_eq!(ray.layer_mask(), expected);
        }
    }

    #[test]
    fn pa_up_rejects_non_finite_pose_in_both_spaces() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for axis in 0..3 {
                let mut position = [0.0; 3];
                position[axis] = invalid;
                assert!(SourcePaUpHeadCast::from_bevy_head_top_position(position).is_err());
                assert!(SourcePaUpHeadCast::from_unity_head_top_position(position).is_err());
                assert!(
                    SourcePaUpGroundRay::from_bevy_player_transform_position(position, 0).is_err()
                );
                assert!(
                    SourcePaUpGroundRay::from_unity_player_transform_position(position, 0).is_err()
                );
            }
        }
    }
}
