//! Source stand-up clearance requests, with explicitly supplied rig-root pose.
//!
//! Both recovered base methods cast from RootBone.position, using a fixed
//! world-space sphere and up direction. Actor feet/capsule center and active
//! skeleton pose are not inferred. Native PhysX results remain a separate
//! acceptance boundary. Evidence: memory/evidence/player-clearance-native/.

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
}
