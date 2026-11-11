//! Source controller definitions recovered from the supplied APK's native code.
//!
//! These are unpatched constructor values and scalar velocity requests, not
//! selected runtime presets or effective movement/camera/collider settings.
//! Provenance and native ranges: memory/evidence/player-rules-source-review.md.
//! No gravity, jump impulse, sprint speed, FOV or world transform is inferred.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceControllerError {
    NonFinite { field: &'static str },
    NegativeInputMagnitude,
    NonPositiveCapsuleDimension { field: &'static str },
    UnresolvedUnityCapsuleClamp,
}

impl fmt::Display for SourceControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite { field } => write!(f, "{field} must be finite"),
            Self::NegativeInputMagnitude => f.write_str("source input magnitude must be nonnegative"),
            Self::NonPositiveCapsuleDimension { field } => {
                write!(f, "capsule {field} must be positive")
            }
            Self::UnresolvedUnityCapsuleClamp => f.write_str(
                "source capsule height is below its diameter; Unity's effective clamping is unresolved",
            ),
        }
    }
}

impl std::error::Error for SourceControllerError {}

pub(crate) fn finite(value: f32, field: &'static str) -> Result<f32, SourceControllerError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(SourceControllerError::NonFinite { field })
    }
}

/// Original Unity direction values recovered for the source player capsules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SourceCapsuleDirection {
    Y = 1,
    Z = 2,
}

/// Raw dimensions in the source collider's local space, before transforms.
///
/// Raw source values can have height below diameter, as Jump does. They must
/// remain intact until Unity's effective geometry and scaling are established.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceCapsuleSettings {
    pub height: f32,
    pub center_y: f32,
    pub direction: SourceCapsuleDirection,
    pub radius: f32,
}

impl SourceCapsuleSettings {
    /// Computes an unscaled geometric segment only when no height clamp is needed.
    ///
    /// This does not establish an effective Unity or Rapier collider: parent
    /// transforms, scale and runtime overrides still require separate evidence.
    pub fn try_unscaled_half_segment(self) -> Result<f32, SourceControllerError> {
        finite(self.height, "capsule height")?;
        finite(self.center_y, "capsule center_y")?;
        finite(self.radius, "capsule radius")?;
        if self.height <= 0.0 {
            return Err(SourceControllerError::NonPositiveCapsuleDimension { field: "height" });
        }
        if self.radius <= 0.0 {
            return Err(SourceControllerError::NonPositiveCapsuleDimension { field: "radius" });
        }
        let diameter = finite(self.radius * 2.0, "capsule diameter")?;
        if self.height < diameter {
            return Err(SourceControllerError::UnresolvedUnityCapsuleClamp);
        }
        Ok((self.height - diameter) / 2.0)
    }
}

/// `BasePlayerController` stand capsule, static constructor `0x1e59600`.
pub const SOURCE_STAND_CAPSULE: SourceCapsuleSettings = SourceCapsuleSettings {
    height: 2.1,
    center_y: 1.04,
    direction: SourceCapsuleDirection::Y,
    radius: 0.42,
};

/// Base crouch and `CrouchState` share these raw defaults (`0x1e6200c`).
pub const SOURCE_CROUCH_CAPSULE: SourceCapsuleSettings = SourceCapsuleSettings {
    height: 1.5,
    center_y: 0.73,
    direction: SourceCapsuleDirection::Y,
    radius: 0.42,
};

/// `JumpState` static constructor `0x1e6f2bc`; height deliberately stays 0.5.
pub const SOURCE_JUMP_CAPSULE: SourceCapsuleSettings = SourceCapsuleSettings {
    height: 0.5,
    center_y: 1.2,
    direction: SourceCapsuleDirection::Y,
    radius: 0.42,
};

fn with_heel_height(
    source: SourceCapsuleSettings,
    heel_height: f32,
) -> Result<SourceCapsuleSettings, SourceControllerError> {
    let heel_height = finite(heel_height, "heel height")?;
    Ok(SourceCapsuleSettings {
        height: finite(source.height + heel_height, "adjusted capsule height")?,
        center_y: finite(
            source.center_y - heel_height / 2.0,
            "adjusted capsule center_y",
        )?,
        ..source
    })
}

/// Native `StandCapsule` (`0x1e518cc`): H += heel; center_y -= heel / 2.
pub fn stand_capsule_with_heel_height(
    heel_height: f32,
) -> Result<SourceCapsuleSettings, SourceControllerError> {
    with_heel_height(SOURCE_STAND_CAPSULE, heel_height)
}

/// Native `CrouchCapsule` (`0x1e615a8`); Jump has no corresponding heel rule.
pub fn crouch_capsule_with_heel_height(
    heel_height: f32,
) -> Result<SourceCapsuleSettings, SourceControllerError> {
    with_heel_height(SOURCE_CROUCH_CAPSULE, heel_height)
}

/// Generic `vThirdPersonCameraState` constructor values (`0x16ee930`).
/// No selected preset, eye offset, FOV or mouse-unit conversion is established.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceCameraDefaults {
    pub default_distance: f32,
    pub x_mouse_sensitivity: f32,
    pub y_mouse_sensitivity: f32,
    pub y_min_limit_degrees: f32,
    pub y_max_limit_degrees: f32,
    pub x_min_limit_degrees: f32,
    pub x_max_limit_degrees: f32,
    pub culling_height: f32,
    pub culling_min_dist: f32,
    pub follow_smooth: f32,
    pub change_smooth: f32,
    pub near_clipping_plane: f32,
    pub far_clipping_plane: f32,
}

pub const SOURCE_GENERIC_CAMERA_DEFAULTS: SourceCameraDefaults = SourceCameraDefaults {
    default_distance: 1.5,
    x_mouse_sensitivity: 3.0,
    y_mouse_sensitivity: 3.0,
    y_min_limit_degrees: -40.0,
    y_max_limit_degrees: 80.0,
    x_min_limit_degrees: -360.0,
    x_max_limit_degrees: 360.0,
    culling_height: 0.2,
    culling_min_dist: 0.1,
    follow_smooth: 3.0,
    change_smooth: 8.0,
    near_clipping_plane: 0.1,
    far_clipping_plane: 800.0,
};

/// `PlayerController` constructor values (`0x1ecafb8`), before overrides.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourcePlayerControllerDefaults {
    pub rotate_speed: f32,
    pub move_speed_additive: f32,
    pub ground_min_distance: f32,
    pub ground_max_distance: f32,
    pub ground_distance: f32,
}

pub const SOURCE_PLAYER_CONTROLLER_DEFAULTS: SourcePlayerControllerDefaults =
    SourcePlayerControllerDefaults {
        rotate_speed: 10.0,
        move_speed_additive: 1.0,
        ground_min_distance: 0.2,
        ground_max_distance: 0.2,
        ground_distance: 0.1,
    };

/// Distinct `talent.StatusMgr` constructor fields (`0x22f1638`).
/// `LandSpeed` 6.3 is not the factor 5 in the ordinary Stand velocity request.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceStatusMovementDefaults {
    pub land_speed: f32,
    pub swim_speed: f32,
    pub fly_speed: f32,
    pub land_speed_rate: f32,
    pub swim_speed_rate: f32,
    pub speed_multiple_rate: f32,
}

pub const SOURCE_STATUS_MOVEMENT_DEFAULTS: SourceStatusMovementDefaults =
    SourceStatusMovementDefaults {
        land_speed: 6.3,
        swim_speed: 3.0,
        fly_speed: 6.3,
        land_speed_rate: 1.0,
        swim_speed_rate: 1.0,
        speed_multiple_rate: 1.0,
    };

/// A finite source multiplier or additive modifier. Signed values are retained;
/// native evidence does not establish a general positivity constraint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FiniteMovementModifier(f32);

impl FiniteMovementModifier {
    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);

    pub fn new(value: f32) -> Result<Self, SourceControllerError> {
        Self::for_field(value, "movement modifier")
    }

    fn for_field(value: f32, field: &'static str) -> Result<Self, SourceControllerError> {
        Ok(Self(finite(value, field)?))
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrdinaryStandModifiers {
    pub add_speed: FiniteMovementModifier,
    pub move_speed_additive: FiniteMovementModifier,
    pub speed_plus: FiniteMovementModifier,
    pub land_speed_rate: FiniteMovementModifier,
}

impl OrdinaryStandModifiers {
    pub fn new(
        add_speed: f32,
        move_speed_additive: f32,
        speed_plus: f32,
        land_speed_rate: f32,
    ) -> Result<Self, SourceControllerError> {
        Ok(Self {
            add_speed: FiniteMovementModifier::for_field(add_speed, "m_AddSpeed")?,
            move_speed_additive: FiniteMovementModifier::for_field(
                move_speed_additive,
                "MoveSpeedAdditive",
            )?,
            speed_plus: FiniteMovementModifier::for_field(speed_plus, "m_SpeedPlus")?,
            land_speed_rate: FiniteMovementModifier::for_field(land_speed_rate, "LandSpeedRate")?,
        })
    }
}

impl Default for OrdinaryStandModifiers {
    fn default() -> Self {
        Self {
            add_speed: FiniteMovementModifier::ONE,
            move_speed_additive: FiniteMovementModifier::ONE,
            speed_plus: FiniteMovementModifier::ZERO,
            land_speed_rate: FiniteMovementModifier::ONE,
        }
    }
}

/// Finite scalar passed to source `ChangeVelocity`, before its mode-specific
/// application. It is not an observed displacement, sprint speed or jump impulse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceVelocityRequest(f32);

impl SourceVelocityRequest {
    pub const fn get(self) -> f32 {
        self.0
    }
}

pub const SOURCE_ORDINARY_STAND_REQUEST_FACTOR: f32 = 5.0;
pub const SOURCE_ORDINARY_CROUCH_REQUEST_FACTOR: f32 = 3.4;

/// Ordinary `StandState.FixedUpdate` (`0x1e7c950..0x1e7c9f4`).
/// Preserves native float operation order without fusing, clamping or guessing
/// the Run/Rush/root-motion interaction or current runtime modifiers.
pub fn ordinary_stand_velocity_request(
    modifiers: OrdinaryStandModifiers,
) -> Result<SourceVelocityRequest, SourceControllerError> {
    let value = modifiers.add_speed.get() * SOURCE_ORDINARY_STAND_REQUEST_FACTOR;
    let value = value * modifiers.move_speed_additive.get();
    let value = value + modifiers.speed_plus.get();
    let value = value * modifiers.land_speed_rate.get();
    Ok(SourceVelocityRequest(finite(
        value,
        "ordinary Stand velocity request",
    )?))
}

/// Ordinary non-free `CrouchState.FixedUpdate` (`0x1e61fcc..0x1e61fe4`).
/// With no gun, source code supplies a multiplier of one. Free mode is a distinct
/// `ChangeForwardVelocity` path and is deliberately not represented here.
pub fn ordinary_crouch_velocity_request(
    gun_speed_change: FiniteMovementModifier,
) -> Result<SourceVelocityRequest, SourceControllerError> {
    Ok(SourceVelocityRequest(finite(
        gun_speed_change.get() * SOURCE_ORDINARY_CROUCH_REQUEST_FACTOR,
        "ordinary Crouch velocity request",
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.000_001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn ordinary_requests_use_each_modifier_and_keep_status_fields_separate() {
        let baseline = ordinary_stand_velocity_request(OrdinaryStandModifiers::default()).unwrap();
        near(baseline.get(), 5.0);
        let modified = OrdinaryStandModifiers::new(2.0, 1.5, 3.0, 0.5).unwrap();
        near(
            ordinary_stand_velocity_request(modified).unwrap().get(),
            9.0,
        );
        near(
            ordinary_crouch_velocity_request(FiniteMovementModifier::ONE)
                .unwrap()
                .get(),
            3.4,
        );
        near(
            ordinary_crouch_velocity_request(FiniteMovementModifier::new(0.5).unwrap())
                .unwrap()
                .get(),
            1.7,
        );
        assert_ne!(baseline.get(), SOURCE_STATUS_MOVEMENT_DEFAULTS.land_speed);
        let signed = OrdinaryStandModifiers::new(1.0, 1.0, -7.0, 1.0).unwrap();
        near(ordinary_stand_velocity_request(signed).unwrap().get(), -2.0);
    }

    #[test]
    fn every_stand_modifier_rejects_non_finite_input() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(FiniteMovementModifier::new(invalid).is_err());
            for index in 0..4 {
                let mut values = [1.0, 1.0, 0.0, 1.0];
                values[index] = invalid;
                assert!(
                    OrdinaryStandModifiers::new(values[0], values[1], values[2], values[3])
                        .is_err()
                );
            }
        }
    }

    #[test]
    fn finite_inputs_that_overflow_are_rejected_at_the_request_boundary() {
        let modifiers = OrdinaryStandModifiers::new(f32::MAX, 1.0, 0.0, 1.0).unwrap();
        assert!(matches!(
            ordinary_stand_velocity_request(modifiers),
            Err(SourceControllerError::NonFinite { .. })
        ));
        assert!(matches!(
            ordinary_crouch_velocity_request(FiniteMovementModifier::new(f32::MAX).unwrap()),
            Err(SourceControllerError::NonFinite { .. })
        ));
        let modifiers = OrdinaryStandModifiers::new(f32::MAX, 0.0, 0.0, 1.0).unwrap();
        assert!(ordinary_stand_velocity_request(modifiers).is_err());
    }

    #[test]
    fn heel_adjustment_preserves_top_and_lowers_bottom_for_stand_and_crouch() {
        for (source, adjusted) in [
            (
                SOURCE_STAND_CAPSULE,
                stand_capsule_with_heel_height(0.2).unwrap(),
            ),
            (
                SOURCE_CROUCH_CAPSULE,
                crouch_capsule_with_heel_height(0.2).unwrap(),
            ),
        ] {
            near(adjusted.height, source.height + 0.2);
            near(adjusted.center_y, source.center_y - 0.1);
            near(
                adjusted.center_y + adjusted.height / 2.0,
                source.center_y + source.height / 2.0,
            );
            near(
                adjusted.center_y - adjusted.height / 2.0,
                source.center_y - source.height / 2.0 - 0.2,
            );
            assert_eq!(adjusted.direction, source.direction);
            assert_eq!(adjusted.radius, source.radius);
            near(
                adjusted.try_unscaled_half_segment().unwrap(),
                source.try_unscaled_half_segment().unwrap() + 0.1,
            );
        }
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(stand_capsule_with_heel_height(invalid).is_err());
            assert!(crouch_capsule_with_heel_height(invalid).is_err());
        }
        let signed_raw = stand_capsule_with_heel_height(-3.0).unwrap();
        assert!(signed_raw.height < 0.0);
        assert_eq!(
            signed_raw.try_unscaled_half_segment(),
            Err(SourceControllerError::NonPositiveCapsuleDimension { field: "height" })
        );
    }

    #[test]
    fn raw_jump_geometry_is_retained_and_unresolved_conversion_is_rejected() {
        assert_eq!(SOURCE_JUMP_CAPSULE.height, 0.5);
        assert_eq!(SOURCE_JUMP_CAPSULE.radius, 0.42);
        assert_eq!(
            SOURCE_JUMP_CAPSULE.try_unscaled_half_segment(),
            Err(SourceControllerError::UnresolvedUnityCapsuleClamp)
        );
        near(
            SOURCE_STAND_CAPSULE.try_unscaled_half_segment().unwrap(),
            0.63,
        );
        near(
            SOURCE_CROUCH_CAPSULE.try_unscaled_half_segment().unwrap(),
            0.33,
        );
        let diameter_height = SourceCapsuleSettings {
            height: SOURCE_STAND_CAPSULE.radius * 2.0,
            ..SOURCE_STAND_CAPSULE
        };
        assert_eq!(diameter_height.try_unscaled_half_segment(), Ok(0.0));
        let invalid = SourceCapsuleSettings {
            radius: f32::NAN,
            ..SOURCE_STAND_CAPSULE
        };
        assert!(invalid.try_unscaled_half_segment().is_err());
    }
}
