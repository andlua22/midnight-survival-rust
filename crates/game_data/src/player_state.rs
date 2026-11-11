//! Exact source state IDs and the unpatched `StandState.Move` classification.
//!
//! These definitions do not select a player state, desktop sprint binding,
//! physical speed, animation, weapon or camera behavior. The caller supplies
//! source PlayerController.Input or explicit magnitude/orientation scalars.
//! Evidence: memory/evidence/player-state-native/README.md and
//! memory/evidence/player-movement-native/README.md.

use crate::controller::{SourceControllerError, finite};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A raw ID outside the retained APK's exact `StateID` declarations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownSourceStateId(pub i32);

impl fmt::Display for UnknownSourceStateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown source StateID {}", self.0)
    }
}

impl std::error::Error for UnknownSourceStateId {}

macro_rules! source_state_ids {
    ($($name:ident = $raw:literal,)+) => {
        /// The complete source enum, with its original spellings and integers.
        ///
        /// Numeric Serde representation is a reconstruction DTO contract; it
        /// does not establish any original packet framing or state transitions.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[repr(i32)]
        #[serde(try_from = "i32", into = "i32")]
        pub enum SourceStateId {
            $($name = $raw,)+
        }

        impl SourceStateId {
            pub const ALL: &'static [Self] = &[$(Self::$name,)+];

            pub const fn raw(self) -> i32 {
                self as i32
            }

            pub const fn source_name(self) -> &'static str {
                match self {
                    $(Self::$name => stringify!($name),)+
                }
            }

            /// Retain unknown integers as errors instead of mapping to Stand/null.
            pub const fn from_raw(raw: i32) -> Result<Self, UnknownSourceStateId> {
                match raw {
                    $($raw => Ok(Self::$name),)+
                    _ => Err(UnknownSourceStateId(raw)),
                }
            }
        }

        impl TryFrom<i32> for SourceStateId {
            type Error = UnknownSourceStateId;

            fn try_from(raw: i32) -> Result<Self, Self::Error> {
                Self::from_raw(raw)
            }
        }

        impl From<SourceStateId> for i32 {
            fn from(state: SourceStateId) -> Self {
                state.raw()
            }
        }
    };
}

// Metadata declarations in export/StateID.cs. Do not translate source labels
// or correct MonsterGoBitrh: those changes would lose source identity.
source_state_ids! {
    NullStateID = 0,
    Stand = 1,
    Pa = 2,
    PaUp = 3,
    Crouch = 4,
    CrouchDown = 5,
    Jump = 6,
    Fall = 7,
    Land = 8,
    Aim = 9,
    Shoot = 10,
    Shouwuqi = 11,
    Nawuqi = 12,
    Huanzidan = 13,
    Zhuangtian = 14,
    InCar = 15,
    Attack = 16,
    Cross = 17,
    ThrowWeaponState = 18,
    HoldNearWeaponState = 19,
    Swim = 20,
    UseItem = 21,
    SkyDiving = 22,
    InStartPlane = 23,
    SavePeople = 24,
    DownWaitSave = 25,
    Climb = 26,
    InDolphin = 27,
    RidePet = 28,
    Rolling = 29,
    MonsterFreeMove = 30,
    MonsterAttack = 31,
    MonsterBackAttack = 32,
    MonsterDefence = 33,
    MonsterFollow = 34,
    MonsterGoBitrh = 35,
    MonsterCollect = 36,
    MonsterStiff = 37,
    DolphinWaitTaking = 38,
    Build = 39,
    Sit = 40,
    CutPlant = 41,
    Plant = 42,
    Op = 43,
    Frozen = 44,
    AddOil = 45,
    Fly = 46,
    FlyWuRenJi = 47,
    SitQiuQian = 48,
    Sleep = 49,
    AutoMove = 50,
    TankShoot = 51,
    PlayMusicPart = 52,
    PasswordControl = 53,
    SitShoot = 54,
    Eject = 55,
    UseBike = 56,
    RetrieveBike = 57,
    Freezy = 58,
    ParabolicMotion = 59,
    ParabolicWait = 60,
    ClearParking = 61,
}

/// The native double constant, compared against an input magnitude promoted
/// from f32. The metadata's `0.917f` is a distinct, slightly smaller value.
pub const SOURCE_STAND_RUSH_THRESHOLD: f64 = f64::from_bits(0x3fed_5810_624d_d2f2);

/// Exact f32 multiplier used by the native MathUtils orientation helper.
pub const SOURCE_ORIENTATION_RADIANS_TO_DEGREES_FACTOR: f32 = f32::from_bits(0x4265_2ee0);

/// Source Stand Move magnitude: separate f32 squares, addition and square root.
/// No clamp/deadzone/hypot substitution; finite-input arithmetic overflow is
/// rejected as reconstruction admission policy.
pub fn source_stand_input_magnitude(input: [f32; 2]) -> Result<f32, SourceControllerError> {
    finite(input[0], "source Input.x")?;
    finite(input[1], "source Input.y")?;
    let x_squared = input[0] * input[0];
    let y_squared = input[1] * input[1];
    finite((x_squared + y_squared).sqrt(), "Stand Move input magnitude")
}

/// Native Vector2 MathUtils.GetOrientation uses atan2f(input.x, input.y),
/// multiplies by the exact source f32 constant and adds 360 only if negative.
/// This is PlayerController.Input, not the distinct InputVector property or
/// an inferred desktop/camera coordinate transform. Host libm atan2 results
/// have not been compared bit-for-bit against the original Android runtime.
pub fn source_input_orientation(input: [f32; 2]) -> Result<f32, SourceControllerError> {
    finite(input[0], "source Input.x")?;
    finite(input[1], "source Input.y")?;
    let angle = input[0].atan2(input[1]) * SOURCE_ORIENTATION_RADIANS_TO_DEGREES_FACTOR;
    finite(
        if angle < 0.0 { angle + 360.0 } else { angle },
        "source orientation",
    )
}

fn rush_eligible(input_magnitude: f32, can_rush: bool) -> bool {
    f64::from(input_magnitude) > SOURCE_STAND_RUSH_THRESHOLD && can_rush
}

/// Mutually exclusive flags assigned by the ordinary Stand Move base path.
/// Neither also occurs with high input directed outside the Rush sectors.
/// This is not the source `StateID` or an idle/walk/sprint speed selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceStandMotion {
    Neither,
    Run,
    Rush,
}

/// Source flag/field writes only, before weapon and camera callbacks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceStandMoveDecision {
    motion: SourceStandMotion,
    orientation_update: Option<f32>,
}

impl SourceStandMoveDecision {
    pub const fn motion(self) -> SourceStandMotion {
        self.motion
    }

    pub const fn stand_run(self) -> bool {
        matches!(self.motion, SourceStandMotion::Run)
    }

    pub const fn stand_rush(self) -> bool {
        matches!(self.motion, SourceStandMotion::Rush)
    }

    /// Native resets FreeType on entry, then sets it only in the Rush branch.
    pub const fn free_type(self) -> bool {
        self.stand_rush()
    }

    /// The written m_SpeedMultiple field, not a physical velocity multiplier.
    pub const fn speed_multiple(self) -> f32 {
        if self.stand_rush() { 3.0 } else { 2.0 }
    }

    /// Some means write YaoGanAngle; None means preserve the previous angle.
    pub const fn orientation_update(self) -> Option<f32> {
        self.orientation_update
    }
}

/// Project the recovered ordinary `StandState.Move` branch from explicit
/// scalar inputs. Finite, nonnegative magnitude admission is reconstruction
/// policy. Negative magnitudes are rejected; out-of-range finite angles are
/// retained without clamping or wrapping.
///
/// Even zero magnitude takes the native Run fallback. The caller must recover
/// when Move is invoked; this function does not declare zero input to be running
/// in the complete player state machine. Angle is supplied, never computed from
/// desktop input or an assumed coordinate frame. It is ignored unless input
/// exceeds the double threshold and m_CanRush is true.
pub fn classify_source_stand_move(
    input_magnitude: f32,
    can_rush: bool,
    orientation_degrees: f32,
) -> Result<SourceStandMoveDecision, SourceControllerError> {
    finite(input_magnitude, "Stand Move input magnitude")?;
    finite(orientation_degrees, "Stand Move orientation")?;
    if input_magnitude < 0.0 {
        return Err(SourceControllerError::NegativeInputMagnitude);
    }
    let eligible = rush_eligible(input_magnitude, can_rush);
    let (motion, orientation_update) = if eligible {
        let forward = (0.0..75.0).contains(&orientation_degrees)
            || (orientation_degrees > 285.0 && orientation_degrees <= 360.0);
        (
            if forward {
                SourceStandMotion::Rush
            } else {
                SourceStandMotion::Neither
            },
            Some(orientation_degrees),
        )
    } else {
        (SourceStandMotion::Run, None)
    };
    Ok(SourceStandMoveDecision {
        motion,
        orientation_update,
    })
}

/// Project source flags from an explicitly supplied PlayerController.Input
/// vector. As in native Move, compute orientation only after the Rush gates
/// pass. Call scheduling, patches and downstream callbacks remain external.
pub fn classify_source_stand_input(
    input: [f32; 2],
    can_rush: bool,
) -> Result<SourceStandMoveDecision, SourceControllerError> {
    let magnitude = source_stand_input_magnitude(input)?;
    let orientation = if rush_eligible(magnitude, can_rush) {
        source_input_orientation(input)?
    } else {
        0.0
    };
    classify_source_stand_move(magnitude, can_rush, orientation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_ids_round_trip_numerically_without_losing_source_names() {
        assert_eq!(SourceStateId::ALL.len(), 62);
        for raw in 0..=61 {
            let state = SourceStateId::from_raw(raw).unwrap();
            assert_eq!(state.raw(), raw);
            assert_eq!(serde_json::to_string(&state).unwrap(), raw.to_string());
            assert_eq!(
                serde_json::from_str::<SourceStateId>(&raw.to_string()).unwrap(),
                state
            );
        }
        assert_eq!(SourceStateId::from_raw(2).unwrap().source_name(), "Pa");
        assert_eq!(
            SourceStateId::from_raw(35).unwrap().source_name(),
            "MonsterGoBitrh"
        );
        assert_ne!(SourceStateId::Frozen, SourceStateId::Freezy);
        assert_eq!(SourceStateId::ClearParking.raw(), 61);
    }

    #[test]
    fn unknown_ids_and_non_integer_serialization_never_become_a_default_state() {
        for raw in [i32::MIN, -1, 62, i32::MAX] {
            assert_eq!(SourceStateId::from_raw(raw), Err(UnknownSourceStateId(raw)));
            assert!(serde_json::from_str::<SourceStateId>(&raw.to_string()).is_err());
        }
        for invalid in ["\"Stand\"", "null", "1.5", "true", "2147483648"] {
            assert!(serde_json::from_str::<SourceStateId>(invalid).is_err());
        }
    }

    #[test]
    fn rush_threshold_uses_promoted_float_against_native_double() {
        let metadata_float = 0.917_f32;
        let immediately_above = f32::from_bits(metadata_float.to_bits() + 1);
        assert!(f64::from(metadata_float) < SOURCE_STAND_RUSH_THRESHOLD);
        assert!(f64::from(immediately_above) > SOURCE_STAND_RUSH_THRESHOLD);
        assert_eq!(
            classify_source_stand_move(metadata_float, true, 0.0)
                .unwrap()
                .motion(),
            SourceStandMotion::Run
        );
        let decision = classify_source_stand_move(immediately_above, true, 0.0).unwrap();
        assert!(decision.stand_rush());
        assert!(!decision.stand_run());
        assert!(decision.free_type());
        assert_eq!(decision.speed_multiple(), 3.0);
    }

    #[test]
    fn forward_sector_boundaries_are_not_wrapped_or_widened() {
        for (angle, expected) in [
            (-0.01, SourceStandMotion::Neither),
            (-0.0, SourceStandMotion::Rush),
            (0.0, SourceStandMotion::Rush),
            (74.999, SourceStandMotion::Rush),
            (75.0, SourceStandMotion::Neither),
            (180.0, SourceStandMotion::Neither),
            (285.0, SourceStandMotion::Neither),
            (285.001, SourceStandMotion::Rush),
            (360.0, SourceStandMotion::Rush),
            (360.001, SourceStandMotion::Neither),
            (720.0, SourceStandMotion::Neither),
        ] {
            let decision = classify_source_stand_move(1.0, true, angle).unwrap();
            assert_eq!(decision.motion(), expected, "angle {angle}");
            assert_eq!(decision.orientation_update(), Some(angle));
        }
    }

    #[test]
    fn gating_and_zero_input_preserve_native_fallback_and_angle_writes() {
        for (magnitude, can_rush) in [(1.0, false), (0.5, true), (0.0, true), (-0.0, false)] {
            let decision = classify_source_stand_move(magnitude, can_rush, 180.0).unwrap();
            assert!(decision.stand_run());
            assert!(!decision.stand_rush());
            assert!(!decision.free_type());
            assert_eq!(decision.speed_multiple(), 2.0);
            assert_eq!(decision.orientation_update(), None);
        }
        assert_eq!(
            classify_source_stand_move(-0.1, true, 0.0),
            Err(SourceControllerError::NegativeInputMagnitude)
        );
        let sideways = classify_source_stand_move(1.0, true, 180.0).unwrap();
        assert!(!sideways.stand_run());
        assert!(!sideways.stand_rush());
        assert!(!sideways.free_type());
        assert_eq!(sideways.speed_multiple(), 2.0);
    }

    #[test]
    fn non_finite_inputs_are_rejected_by_explicit_admission_policy() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(classify_source_stand_move(invalid, true, 0.0).is_err());
            assert!(classify_source_stand_move(1.0, true, invalid).is_err());
            assert!(classify_source_stand_move(0.0, false, invalid).is_err());
        }
    }

    #[test]
    fn raw_source_axes_and_gates_produce_expected_classification() {
        for (input, expected) in [
            ([0.0, 1.0], SourceStandMotion::Rush),
            ([1.0, 1.0], SourceStandMotion::Rush),
            ([1.0, 0.0], SourceStandMotion::Neither),
            ([0.0, -1.0], SourceStandMotion::Neither),
            ([-1.0, 0.0], SourceStandMotion::Neither),
            ([0.0, 0.0], SourceStandMotion::Run),
            ([0.0, 0.5], SourceStandMotion::Run),
        ] {
            let decision = classify_source_stand_input(input, true).unwrap();
            assert_eq!(decision.motion(), expected, "input {input:?}");
        }
        let gated = classify_source_stand_input([0.0, 1.0], false).unwrap();
        assert_eq!(gated.motion(), SourceStandMotion::Run);
        assert_eq!(gated.orientation_update(), None);
        assert_eq!(source_stand_input_magnitude([3.0, 4.0]).unwrap(), 5.0);
    }

    #[test]
    fn orientation_preserves_source_factor_and_negative_zero_addition_rule() {
        for (input, expected) in [
            ([0.0, 1.0], 0.0),
            ([1.0, 0.0], 90.0),
            ([0.0, -1.0], 180.0),
            ([-1.0, 0.0], 270.0),
            ([-1.0, 1.0], 315.0),
        ] {
            let angle = source_input_orientation(input).unwrap();
            assert!(
                (angle - expected).abs() <= 0.000_02,
                "input {input:?}, angle {angle}"
            );
        }
        let negative_zero = source_input_orientation([-0.0, 1.0]).unwrap();
        assert_eq!(negative_zero.to_bits(), (-0.0_f32).to_bits());
        let rounded = classify_source_stand_input([-f32::MIN_POSITIVE, 1.0], true).unwrap();
        assert_eq!(rounded.orientation_update(), Some(360.0));
        assert_eq!(rounded.motion(), SourceStandMotion::Rush);
    }

    #[test]
    fn raw_input_rejects_non_finite_values_and_native_magnitude_overflow() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for input in [[invalid, 0.0], [0.0, invalid]] {
                assert!(source_stand_input_magnitude(input).is_err());
                assert!(source_input_orientation(input).is_err());
                assert!(classify_source_stand_input(input, false).is_err());
            }
        }
        // hypot(MAX, 0) would be finite; native's separate square is not.
        assert!(source_stand_input_magnitude([f32::MAX, 0.0]).is_err());
        assert!(classify_source_stand_input([f32::MAX, 0.0], true).is_err());
        assert!(source_input_orientation([f32::MAX, 0.0]).is_ok());
    }
}
