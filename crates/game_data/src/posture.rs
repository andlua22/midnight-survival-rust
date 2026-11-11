//! Explicit continuation for the unpatched SC.UI.BattlePanel.OnClickCrouch2.
//!
//! Hosts supply each current-state read and clearance verdict and acknowledge
//! that requested calls returned. This preserves re-reads, actor targets and
//! call order without assuming switches committed or selecting active UI input.
//! Evidence: memory/evidence/player-transitions-native/README.md.

use crate::player_state::SourceStateId;
use std::fmt;

/// Native m_Self and Battle.Ins.SelfPlayer are distinct targets.
/// Their actual identities must be resolved by the host; aliasing is not assumed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceCrouchClickTarget {
    PanelSelf,
    BattleSelfPlayer,
}

/// A request from the recovered caller, not a committed player-state transition.
/// Audio requests retain observed ID/order; full playback settings remain external.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceCrouchClickRequest {
    /// PanelSelf.FSMUpBody.CurrentState.ID.
    ReadUpperBodyState,
    /// Re-read PanelSelf.FSM.CurrentState.ID at this point in the caller.
    ReadBaseState,
    /// PanelSelf.CheckCanPaUp, distinct from CheckCanPaToStand.
    CheckPaUp,
    /// PanelSelf.CheckCanCrouchUp, using the identified rig's RootBone pose.
    CheckCrouchUp,
    Play2D {
        audio_id: i32,
    },
    /// Native SwitchState(state, null); no state-change success is inferred.
    SwitchBaseState {
        target: SourceCrouchClickTarget,
        state: SourceStateId,
    },
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceCrouchClickObservation {
    UpperBodyState(SourceStateId),
    BaseState(SourceStateId),
    /// A completed source-specific verdict; unresolved adapter errors are not
    /// verdicts. PaUp is distinct from the RootBone PaToStand sphere cast.
    PaUpClearance(bool),
    CrouchUpClearance(bool),
    /// The native caller ignores any switch result. Returning does not mean
    /// successful state change; exceptions must not be acknowledged as returns.
    CallReturned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceCrouchClickError {
    pub request: SourceCrouchClickRequest,
    pub observation: SourceCrouchClickObservation,
}

impl fmt::Display for SourceCrouchClickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unexpected {:?} while awaiting {:?}",
            self.observation, self.request
        )
    }
}
impl std::error::Error for SourceCrouchClickError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Upper,
    PaState,
    PaClearance,
    PaSwitch,
    StandState,
    StandAudio,
    GlobalCrouchSwitch,
    CrouchState,
    CrouchClearance,
    StandSwitch,
    CrouchAudio,
    Finished,
}

/// Bounded caller control flow only. It performs no physics, audio, UI or FSM
/// mutation. A new value means an explicitly selected callback invocation,
/// not a default player posture. Wrong observation kinds leave it unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceCrouchClick2 {
    phase: Phase,
}

impl Default for SourceCrouchClick2 {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceCrouchClick2 {
    pub const fn new() -> Self {
        Self {
            phase: Phase::Upper,
        }
    }

    pub const fn request(self) -> SourceCrouchClickRequest {
        use SourceCrouchClickRequest as R;
        match self.phase {
            Phase::Upper => R::ReadUpperBodyState,
            Phase::PaState | Phase::StandState | Phase::CrouchState => R::ReadBaseState,
            Phase::PaClearance => R::CheckPaUp,
            Phase::CrouchClearance => R::CheckCrouchUp,
            Phase::StandAudio => R::Play2D { audio_id: 339 },
            Phase::CrouchAudio => R::Play2D { audio_id: 435 },
            Phase::PaSwitch => R::SwitchBaseState {
                target: SourceCrouchClickTarget::PanelSelf,
                state: SourceStateId::PaUp,
            },
            Phase::GlobalCrouchSwitch => R::SwitchBaseState {
                target: SourceCrouchClickTarget::BattleSelfPlayer,
                state: SourceStateId::Crouch,
            },
            Phase::StandSwitch => R::SwitchBaseState {
                target: SourceCrouchClickTarget::PanelSelf,
                state: SourceStateId::Stand,
            },
            Phase::Finished => R::Finished,
        }
    }

    /// Supply one observation for the currently requested operation. State reads
    /// remain separate: a failed PaUp check may have changed the base FSM before
    /// the Stand/Crouch checks. This does not collapse them to one stale snapshot.
    pub fn observe(
        &mut self,
        observation: SourceCrouchClickObservation,
    ) -> Result<SourceCrouchClickRequest, SourceCrouchClickError> {
        use SourceCrouchClickObservation as O;
        let next = match (self.phase, observation) {
            (Phase::Upper, O::UpperBodyState(SourceStateId::Attack)) => Phase::Finished,
            (Phase::Upper, O::UpperBodyState(_)) => Phase::PaState,
            (Phase::PaState, O::BaseState(SourceStateId::Pa)) => Phase::PaClearance,
            (Phase::PaState, O::BaseState(_)) => Phase::StandState,
            (Phase::PaClearance, O::PaUpClearance(true)) => Phase::PaSwitch,
            (Phase::PaClearance, O::PaUpClearance(false)) => Phase::StandState,
            (Phase::StandState, O::BaseState(SourceStateId::Stand)) => Phase::StandAudio,
            (Phase::StandState, O::BaseState(_)) => Phase::CrouchState,
            (Phase::StandAudio, O::CallReturned) => Phase::GlobalCrouchSwitch,
            (Phase::PaSwitch | Phase::GlobalCrouchSwitch, O::CallReturned) => Phase::Finished,
            (Phase::CrouchState, O::BaseState(SourceStateId::Crouch)) => Phase::CrouchClearance,
            (Phase::CrouchState, O::BaseState(_)) => Phase::Finished,
            (Phase::CrouchClearance, O::CrouchUpClearance(true)) => Phase::StandSwitch,
            (Phase::CrouchClearance, O::CrouchUpClearance(false)) => Phase::Finished,
            (Phase::StandSwitch, O::CallReturned) => Phase::CrouchAudio,
            (Phase::CrouchAudio, O::CallReturned) => Phase::Finished,
            _ => {
                return Err(SourceCrouchClickError {
                    request: self.request(),
                    observation,
                });
            }
        };
        self.phase = next;
        Ok(self.request())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use SourceCrouchClickObservation as O;
    use SourceCrouchClickRequest as R;

    fn begin() -> SourceCrouchClick2 {
        let mut callback = SourceCrouchClick2::new();
        assert_eq!(
            callback
                .observe(O::UpperBodyState(SourceStateId::NullStateID))
                .unwrap(),
            R::ReadBaseState
        );
        callback
    }

    #[test]
    fn upper_body_attack_stops_before_any_base_read_or_effect() {
        let mut callback = SourceCrouchClick2::new();
        assert_eq!(
            callback
                .observe(O::UpperBodyState(SourceStateId::Attack))
                .unwrap(),
            R::Finished
        );
    }

    #[test]
    fn standing_plays_audio_before_switching_the_distinct_global_target() {
        let mut callback = begin();
        assert_eq!(
            callback
                .observe(O::BaseState(SourceStateId::Stand))
                .unwrap(),
            R::ReadBaseState
        );
        assert_eq!(
            callback
                .observe(O::BaseState(SourceStateId::Stand))
                .unwrap(),
            R::Play2D { audio_id: 339 }
        );
        assert_eq!(
            callback.observe(O::CallReturned).unwrap(),
            R::SwitchBaseState {
                target: SourceCrouchClickTarget::BattleSelfPlayer,
                state: SourceStateId::Crouch
            }
        );
        assert_eq!(callback.observe(O::CallReturned).unwrap(), R::Finished);
    }

    #[test]
    fn crouched_clearance_blocks_or_switches_self_then_plays_audio_after_return() {
        for clear in [false, true] {
            let mut callback = begin();
            for _ in 0..2 {
                assert_eq!(
                    callback
                        .observe(O::BaseState(SourceStateId::Crouch))
                        .unwrap(),
                    R::ReadBaseState
                );
            }
            assert_eq!(
                callback
                    .observe(O::BaseState(SourceStateId::Crouch))
                    .unwrap(),
                R::CheckCrouchUp
            );
            let next = callback.observe(O::CrouchUpClearance(clear)).unwrap();
            if clear {
                assert_eq!(
                    next,
                    R::SwitchBaseState {
                        target: SourceCrouchClickTarget::PanelSelf,
                        state: SourceStateId::Stand
                    }
                );
                // CallReturned acknowledges any return, even an internally denied
                // switch. This caller has no successful-commit condition.
                assert_eq!(
                    callback.observe(O::CallReturned).unwrap(),
                    R::Play2D { audio_id: 435 }
                );
                assert_eq!(callback.observe(O::CallReturned).unwrap(), R::Finished);
            } else {
                assert_eq!(next, R::Finished);
            }
        }
    }

    #[test]
    fn pa_requires_its_distinct_check_and_failure_re_reads_the_current_state() {
        let mut callback = begin();
        assert_eq!(
            callback.observe(O::BaseState(SourceStateId::Pa)).unwrap(),
            R::CheckPaUp
        );
        assert_eq!(
            callback.observe(O::PaUpClearance(false)).unwrap(),
            R::ReadBaseState
        );
        // Observe a changed state, not the stale Pa value supplied before check.
        assert_eq!(
            callback
                .observe(O::BaseState(SourceStateId::Stand))
                .unwrap(),
            R::Play2D { audio_id: 339 }
        );
        let mut clear = begin();
        assert_eq!(
            clear.observe(O::BaseState(SourceStateId::Pa)).unwrap(),
            R::CheckPaUp
        );
        assert_eq!(
            clear.observe(O::PaUpClearance(true)).unwrap(),
            R::SwitchBaseState {
                target: SourceCrouchClickTarget::PanelSelf,
                state: SourceStateId::PaUp
            }
        );
        assert_eq!(clear.observe(O::CallReturned).unwrap(), R::Finished);
    }

    #[test]
    fn invalid_observations_and_completed_callbacks_preserve_control_state() {
        let mut callback = SourceCrouchClick2::new();
        let original = callback;
        assert!(
            callback
                .observe(O::BaseState(SourceStateId::Stand))
                .is_err()
        );
        assert_eq!(callback, original);
        let mut pa = begin();
        pa.observe(O::BaseState(SourceStateId::Pa)).unwrap();
        let paused = pa;
        assert!(pa.observe(O::CrouchUpClearance(true)).is_err());
        assert_eq!(pa, paused);
        callback
            .observe(O::UpperBodyState(SourceStateId::Attack))
            .unwrap();
        let finished = callback;
        assert!(callback.observe(O::CallReturned).is_err());
        assert_eq!(callback, finished);
    }
}
