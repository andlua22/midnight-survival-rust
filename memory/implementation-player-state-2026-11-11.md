# Source player state and input definitions — 2026-11-11

The next player definitions step is implemented in
[player_state.rs](../crates/game_data/src/player_state.rs). It is exported by
`midnight_game_data` and builds with the existing Rust/Bevy workspace without
new dependencies or lockfile changes. Earlier inventory, terrain and UI changes
and evidence were preserved.

## Concrete result

The complete `SourceStateId` enum retains all 62 original names/integers,
0 through 61, including `Pa`, `MonsterGoBitrh`, `Frozen` and the distinct
`Freezy`. Safe conversion rejects unknown integers instead of selecting a
default. Numeric Serde is an explicit reconstruction DTO contract, not a
recovered original network frame. A direct source-versus-Rust comparison
verified every name/value against `export/StateID.cs`.

Source input helpers preserve separate f32 squares/addition/square root and
`atan2f(Input.x, Input.y)` with the exact native f32 degrees factor
`0x42652ee0`. They add 360 only for negative orientation, preserving signed
zero. No hypot, generic degrees conversion, clamp, modulo or deadzone is
substituted. Host libm rounding has not been matched against the Android runtime.

The Stand classifier preserves the unpatched base procedure's field writes:

| Input condition | Flags | FreeType | m_SpeedMultiple | YaoGanAngle |
|---|---|---|---:|---|
| Promoted magnitude > native f64 0.917, canRush, angle [0,75) or (285,360] | Rush | true | 3 | write supplied/computed angle |
| Same magnitude/gate, other finite angle | Neither Run nor Rush | false | 2 | write supplied/computed angle |
| Other admitted magnitude/gate, including zero | Run | false | 2 | preserve previous angle |

`classify_source_stand_move` accepts explicit magnitude/orientation values.
`classify_source_stand_input` accepts source `PlayerController.Input` and
computes orientation only after the Rush gates pass. This vector is distinct
from BasePlayerController.InputVector; damaged exported struct offsets are
not adopted as a Rust binary layout.

Negative scalar magnitude is rejected, while -0 is admitted. Finite input and
arithmetic-overflow rejection are reconstruction admission policies, not
source invalid-input parity. A finite true square-root magnitude cannot be
negative; the native hypothetical failed >=0 fallback skips the multiplier
write, so the API does not fabricate that outcome. Flags classify the Move
procedure only; its invocation gates and downstream weapon/camera callbacks
are still outside this data boundary.

## Validation and evidence

- Nine new tests cover numeric IDs/unknown input, native f64 threshold versus
  adjacent f32 values, orientation sectors, zero/gated input, signed-zero
  orientation, raw vector directions and non-finite/overflow admission.
- All 36 `midnight_game_data` tests passed; existing inventory/controller tests
  continue to pass. Final all-target workspace check passed.
- Strict all-target game_data Clippy passed with `--no-deps -- -D warnings`.
  Scoped rustfmt and `git diff --check` passed.
- All 28 retained native artifacts/source files checked against the two audit
  manifests matched their SHA-256 identities. The 62 source/Rust ID pairs
  matched independently of the Rust round-trip tests.

Logs and comparison results are under
[player-state-rust](evidence/player-state-rust/README.md). Static disassembly
and exact source identities are under
[player-state-native](evidence/player-state-native/README.md) and
[player-movement-native](evidence/player-movement-native/README.md). The local
Rust/Capstone helper and retained ELF were reused unchanged. These are static
local observations, not REA provider Evidence IDs or original-game execution.

The movement audit additionally authenticates CannotMove mask 1,
ChangeVelocity's yaw-only rotation, 1e-5 normalization threshold and preserved
Rigidbody Y; ChangeForwardVelocity uses Transform.forward and preserves Y.
That audit records the complete bodies and remaining runtime dependencies.

## Next implementation work

Trace when Move is invoked, Stand/Crouch transitions and stand-clearance rules;
recover selected modifiers and effective transforms/capsule scaling. Then
convert source coordinates and connect the validated controller/posture state
to shared physics with matched client/server scenarios. Jump/landing,
animation displacement and runtime overrides remain prerequisites for movement
parity. Inventory quick-swap/clear/use effects and authentic snapshot/session
admission remain the separate inventory continuation.

Active movement/camera still use the earlier provisional controller. This pass
changes data APIs only; the preceding Linux/GPU startup evidence keeps its
original executable/scope and is not presented as a validation of these new
source movement rules. No Python, Android build, source services, protected
cache retry, cleanup or system changes were used.
