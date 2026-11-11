# Player rules: bounded source/native review

2026-11-11. Initial read-only audit for the next player milestone, followed by
an authorized source-only definitions implementation in
`crates/game_data/src/controller.rs`. The initial audit changed no production
code. The followup changes no active gameplay behavior. No Cargo commands,
source-game execution, GUI, service or cache recovery was performed.

The supplied recovered source is `export/lite4.0.apk`, SHA-256
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
The user's memory-recovery provenance and distinct installed counterpart are
recorded in [minimap-dump-pairing.md](minimap-dump-pairing.md). The retained ELF
`/tmp/codex-minimap-review/libil2cpp.so` was rehashed in this pass: 76,608,224
bytes, SHA-256 `2e2efde1a103549694b17b361cacc18faa5714d06c444945e44b30f1377e2b3c`.

## Evidence boundary

`export/*.cs` preserves declarations, tokens and native RVAs, but its Cpp2IL
bodies are damaged. For example `PlayerController`'s constructor prints
`RotateSpeed = 1.0926162E+09f`; native instructions instead store bits
`0x41200000`, representing `10.0`. Numeric assignments in these bodies must
not be copied or reinterpreted as authenticated values without native bytes.

REA `binary_session` reported no open target. Opening a whole-ELF provider was
avoided. Host `objdump` could not decode this AArch64 ELF. With root's explicit
read-only fallback authorization, this review reused the existing Rust/Capstone
5.0 helper, `/tmp/codex-minimap-review/native`. Its `native.rs` verifies ELF64
AArch64 and maps each range through file-backed `PT_LOAD` segments; reads are
bounded to 256 KiB. Every requested range decoded completely. This is static
local evidence, not REA native Evidence IDs or observed game execution.

Helper SHA-256: `000d6b773e4f8ed574b62b1803a1d4d1314bd0178262359cb3a176a5ac7fd8e9`.
Helper source SHA-256: `91d59a9b3b42ca2e4af1880151decd98b5c00d982978c4e5a9848f8eea03bf18`.
Reproduce a range with `native disasm ELF RVA LENGTH` and a constant with
`native words ELF RVA LENGTH`; lengths and addresses accept hexadecimal.
No helper was rebuilt or modified.

## Capsule defaults and actual setter behavior

These are defaults passed to the source `CheatFloat`/`CheatInt` constructors,
not effective runtime values after IFix patches or other overrides. Directions
retain the original Unity numeric contract (`1` is Y, `2` is Z).

| Exact source fields/method | Height | Center Y | Direction | Radius | Native evidence |
|---|---:|---:|---:|---:|---|
| `BasePlayerController._cheatStandCapsule*` | 2.1 | 1.04 | 1 | 0.42 | static ctor `0x1e59600`; stores static offsets `0x88..0xa0` |
| `BasePlayerController._cheatBasePlayerCrouchCapsule*` | 1.5 | 0.73 | 1 | 0.42 | same ctor; offsets `0x68..0x80` |
| `CrouchState._cheatCrouchCapsule*` | 1.5 | 0.73 | 1 | 0.42 | static ctor `0x1e6200c`; offsets `0x0..0x18` |
| `JumpState._cheatJumpCapsule*` | 0.5 | 1.2 | 1 | 0.42 | static ctor `0x1e6f2bc`; offsets `0x0..0x18` |
| `BasePlayerController._cheatPaCapsule*` | 1.9 | 0.42 | 2 | 0.42 | base static ctor; offsets `0x28..0x40` |
| `BasePlayerController._cheatPaCapsuleLong*` | 1.9 | 0.42 | 2 | 0.42 | base static ctor; offsets `0x48..0x60` |

The float load locations are: `0x362196c = 0x40066666` (2.1),
`0x3622ccc = 0x3ff33333` (1.9), `0x3622cc4 = 0x3ed70a3d` (0.42),
`0x3622cd0 = 0x3f3ae148` (0.73), `0x3622cd4 = 0x3f851eb8` (1.04), and
`0x3621740 = 0x3f99999a` (1.2). Height 1.5 and 0.5 and direction 1/2 are
immediate constructor arguments. Static field-name/offset declarations are in
`export/BasePlayerController.cs:1219`, `CrouchState.cs:194`, and
`JumpState.cs:215`.

`BasePlayerController.StandCapsule` (`0x1e518cc`, length `0x218`) calls
`CharacterSetSkin.GetHeelHeight` (`0x1f7b644`; declaration in
`export/CharacterSetSkin.cs:12842`) and `CheatFloat` value getter `0x2258b00`.
The native setter computes `height = default_height + heel_height` and
`center_y = default_center_y - heel_height / 2`, then applies direction/radius.
`CrouchState.CrouchCapsule` (`0x1e615a8`, length `0x1ac`) does the same.
Neither is represented by one immutable capsule size.

`JumpState.JumpCapsule` (`0x1e6f0f0`, length `0x1cc`) applies its own height,
center Y and transforms. Its declared height 0.5 is below its diameter 0.84.
Unity's effective collider geometry, scaling and parent transforms must be
verified before converting this into a Rapier segment length; do not supply a
negative half-segment or silently claim a matched sphere. Source methods
`PaCapsule`, `PaCapsuleLong`, `CrouchCapsule`, `StandCapsule` and
`Stand3rdCapsule` remain distinct labels/contracts.

## Camera and controller constructor defaults

`vThirdPersonCameraState(string name)` is at `0x16ee930`; its complete body ends
at `0x16eea08`. It loads three vectors from `0x3621d00..0x3621d2f` and stores
them at instance offsets `0x24`, `0x34`, `0x5c`. This authenticates the following
generic constructor values against the declaration in
`export/vThirdPersonCameraState.cs:19`:

| Field | Value |
|---|---:|
| `defaultDistance` | 1.5 |
| `xMouseSensitivity`, `yMouseSensitivity` | 3, 3 |
| `yMinLimit`, `yMaxLimit` | -40, 80 |
| `xMinLimit`, `xMaxLimit` | -360, 360 |
| `cullingHeight`, `cullingMinDist` | 0.2, 0.1 |
| `followSmooth`, `changeSmooth` | 3, 8 |
| `NearClippingPlane`, `FarClippingPlane` | 0.1, 800 |

These are generic constructor defaults, not the selected player-camera preset.
The constructor does not establish an active FOV, eye offset, look-at offset,
selected preset, mouse pixel/radian conversion or viewport-dependent behavior.
Several export offsets for external struct/enum types overlap; do not use those
damaged offsets as a Rust memory layout. Read retained prefab/configuration
fields and native setter call sites before selecting the effective camera.

`PlayerController()` (`0x1ecafb8`, body ends at `0x1ecb690`) stores:
`RotateSpeed = 10` (`0x394`, instructions `0x1ecb10c/118`),
`MoveSpeedAdditive = 1` (`0x39c`, `0x1ecb110/11c`),
`GroundMinDistance = GroundMaxDistance = 0.2` (`0x568/56c`, `0x1ecb298..2d0`),
and `GroundDistance = 0.1` (`0x574`, `0x1ecb2d8..2e8`). These do not identify
walk/sprint/jump speeds or a ground-snap policy by themselves.

## Speed, jump and source state contracts

`GameConst.CanRushInputValue = 0.917f` is a literal metadata constant
(`export/GameConst.cs:140`). The ordinary `StandState.Move` path (`0x1e7b748`,
body ends `0x1e7b974`) compares the input-vector magnitude against a native
double `0.917` (`0x3622d68`, bits `0x3fed5810624dd2f2`). Rush also requires
`PlayerController.m_CanRush` and a `MathUtils.GetOrientation` result in
`[0,75)` or `(285,360]` degrees. That path sets `StandRush`, `FreeType` and
`m_SpeedMultiple=3`. Other branches use `m_SpeedMultiple=2`; the usual
non-Rush branch sets `StandRun`, but an out-of-range orientation with high
input magnitude and `m_CanRush` true leaves both flags cleared. These are
source state/animation inputs, not an authenticated physical speed pair or
the current desktop Shift binding.

`StateID` declarations retain `Stand=1`, `Pa=2`, `PaUp=3`, `Crouch=4`,
`CrouchDown=5`, `Jump=6`, `Fall=7`, `Land=8`, `Aim=9`, `Swim=20`
(`export/StateID.cs:12`). `StandState` has separate `StandRush` and `StandRun`;
`CrouchState` has separate `CrouchRush`, `CrouchRun`, `CrouchWalk`. A bool named
`sprint` cannot be assigned all these source meanings by label similarity.

The ordinary, unpatched `CrouchState.FixedUpdate` (`0x1e61f20`, length `0xec`)
gets `Gun.SpeedChange` (`0x1380000`) or defaults its multiplier to 1. One branch
calls `ChangeVelocity(multiplier * 3.4)` (`0x1e61fcc..1e61fe4`, constant
`0x3622d38 = 0x4059999a`). Its free-mode branch, selected by `FreeType` and
`IsJiMiao`/`InBuildState`, instead calls
`ChangeForwardVelocity(multiplier * 5)` (`0x1e61fe8..1e62004`). This proves
conditional speed rules; it does not establish the active mode/preset.

`StandState.FixedUpdate` (`0x1e7c70c`, complete body ends `0x1e7cb14`) initializes
`m_AddSpeed=1`, `m_SpeedPlus=0` (`0x361f7a0`, stores at `0x1e7c7b4`). Its ordinary
branch computes `(m_AddSpeed * 5 * PlayerController.MoveSpeedAdditive +
m_SpeedPlus) * talent.StatusMgr.LandSpeedRate` before calling `ChangeVelocity`
(`0x1e7c950..1e7c9f4`). `LandSpeedRate` is the declared float at offset `0x4c`
(`export/talent/StatusMgr.cs:60`). The class slot `0x462f620` is shared with the
verified `StatusMgr` constructor; singleton access uses slot `0x461d798`.
An independent native cross-check is `HuanzidanState.SpeedUpSkill`
(`0x1e8e000`, body ends `0x1e8e140`): the identical slots and singleton pointer
chain lead to a call to concrete `StatusMgr.GetFAttribute` (`0x22f1594`) at
`0x1e8e110`, establishing the returned object's type.

`StatusMgr()` (`0x22f1638`, body ends `0x22f16b8`) loads a 16-byte constant
from `0x3624180` and stores it at `0x40..0x4f`; it stores two immediate float
ones at `0x50/0x54`. Together with the source field declarations, this verifies:

| Constructor field | Default | Instance offset |
|---|---:|---:|
| `LandSpeed` | 6.3 | `0x40` |
| `SwimSpeed` | 3 | `0x44` |
| `FlySpeed` | 6.3 | `0x48` |
| `LandSpeedRate` | 1 | `0x4c` |
| `SwimSpeedRate` | 1 | `0x50` |
| `speedMultipleRate` | 1 | `0x54` |

With all listed defaults and no intervening modifiers, the ordinary Stand
formula evaluates to 5. `LandSpeed=6.3` is a separate field, not the multiplier
in this formula. `StatusMgr.OnSSpeedChange` (`0x22f66d4`, body ends `0x22f67cc`)
conditionally writes incoming data into `LandSpeed`, `SwimSpeed` or `FlySpeed`
for the self player. Other Stand branches read gun/status/config data or call
`ChangeForwardVelocity`; animation/root-motion coupling and effective runtime
overrides still prevent one authoritative walk/sprint pair.

`PlayerController.ChangeVelocity(float speed=5f)` is `0x1ebdf20` (body ends
`0x1ebe12c`); `ChangeForwardVelocity(float speed=5f)` is `0x1ebe220` and
`ChangeYVelocity(float ySpeed)` is `0x1ebe35c`. The ordinary ChangeVelocity path
rotates and normalizes the input direction, sets horizontal Rigidbody velocity,
and preserves current vertical velocity. Default argument 5 is not a universal
walk speed. `JumpState`'s native paths include conditional calls with 5 to
`ChangeVelocity` (`0x1e6e6a0..6a8`) and `ChangeYVelocity`
(`0x1e6e8a4..8ac`), surrounded by animation/geometry tests; these do not prove an
immediate fixed jump impulse of 5 or the prototype's 6.

## Implemented source-only definitions boundary

`crates/game_data/src/controller.rs` now exposes the following frozen public API.
It uses only the standard library and does not integrate with Bevy, Rapier,
shared movement or an active camera.

| API | Contract |
|---|---|
| `SourceCapsuleDirection`, `SourceCapsuleSettings` | Raw Unity-local height, center Y, direction and radius |
| `SOURCE_STAND_CAPSULE`, `SOURCE_CROUCH_CAPSULE`, `SOURCE_JUMP_CAPSULE` | Exact recovered defaults; Jump retains height 0.5 and radius 0.42 |
| `stand_capsule_with_heel_height(f32)`, `crouch_capsule_with_heel_height(f32)` | Finite raw heel adjustment `H + heel`, `center_y - heel / 2`; no Jump heel rule |
| `SourceCapsuleSettings::try_unscaled_half_segment()` | Local geometric calculation only; rejects non-finite/non-positive dimensions and unresolved height-below-diameter clamping |
| `SourceCameraDefaults`, `SOURCE_GENERIC_CAMERA_DEFAULTS` | Generic camera constructor values only |
| `SourcePlayerControllerDefaults`, `SOURCE_PLAYER_CONTROLLER_DEFAULTS` | Verified controller constructor values |
| `SourceStatusMovementDefaults`, `SOURCE_STATUS_MOVEMENT_DEFAULTS` | StatusMgr's separate 6.3/3/6.3 speed fields and rate fields |
| `FiniteMovementModifier::new(f32)`, `get()`, `ZERO`, `ONE` | Private finite representation; signed modifiers remain allowed |
| `OrdinaryStandModifiers::new(f32,f32,f32,f32)`, `Default` | Ordered add-speed, move-additive, speed-plus and land-rate modifiers; default 1/1/0/1 |
| `SOURCE_ORDINARY_STAND_REQUEST_FACTOR`, `SOURCE_ORDINARY_CROUCH_REQUEST_FACTOR` | Scalar factors 5 and 3.4, distinct from StatusMgr speed fields |
| `ordinary_stand_velocity_request(OrdinaryStandModifiers)` | Exact source float operation order; finite scalar output |
| `ordinary_crouch_velocity_request(FiniteMovementModifier)` | Ordinary non-free 3.4-times-gun-modifier request |
| `SourceVelocityRequest::get()` | Finite scalar request, not effective displacement or a sprint/jump rule |
| `SourceControllerError` | Typed non-finite, non-positive geometric dimension and unresolved Unity-clamp failures |

Validation used `rustfmt --edition 2024` on this file, then direct
`rustc --edition 2024 --test crates/game_data/src/controller.rs -o
/tmp/codex-player-controller-tests` and the resulting test executable. Five
tests passed: distinct modifier outputs and signed modifiers, all four input
fields rejecting NaN/both infinities, finite-input arithmetic overflow, heel
top/bottom geometry including signed raw values, and unresolved Jump conversion
with the height-equals-diameter boundary. Independent read-only review found no
correctness blockers. Root owns module declaration and workspace Cargo checks.

## Current Rust behavior and next implementable work

`crates/shared/src/movement.rs` currently uses walk 4, sprint 7, gravity -18,
jump 6, radius 0.35, half-segment 0.55, eye offset 0.7 and pitch limit 89 degrees.
This source/native audit authenticates none of those as original defaults.
Its SHA-256 at review is
`ab787db6bde8d272b70695d649eee6dfa2ad6c34dda5b9eab51dde4567903033`.
`shared/src/input.rs` transports movement/look/jump/sprint but no posture or
original state ID. Local and server share this prototype controller; preserving
their agreement does not prove source parity.

The new module establishes the first source-identified definitions boundary.
The next data step retains exact `StateID` values, these posture-specific capsule
defaults, camera constructor defaults, heel-height modifier and provenance /
override status in a validated package. Keep physical movement constants
explicitly provisional until effective presets, run/rush integration and runtime
multipliers are recovered.
Then add posture/clearance state to shared simulation and collision using the
same validated rules on client/server, with a matched-input acceptance scenario.
Do not replace the active camera with the generic constructor defaults, change
the Jump collider, or rename `Pa`/`Rush` during this data step.

Remaining evidence: player/camera prefabs and transforms, IFix overrides,
runtime speed-rate overrides, full Run/Rush integration, animation displacement,
jump timing/air control/landing, original gravity and collision skin/step/slope
settings. No terrain/world-placement cache access was retried.

## Later StateID and input-classification implementation

The next definitions step is implemented in
`crates/game_data/src/player_state.rs`. It retains the full 62-entry StateID
enum, safe numeric conversion/Serde DTOs, explicit scalar Stand classification
and raw PlayerController.Input magnitude/orientation helpers. The full enum
includes source spellings such as MonsterGoBitrh; the earlier posture list
above was a subset. Raw magnitude uses separate f32 squares/add/sqrt and the
native double threshold. Vector2 orientation uses atan2f(x,y), exact factor
bits 0x42652ee0 and negative-only addition of 360.

[Stand evidence](player-state-native/README.md) and
[direction/orientation evidence](player-movement-native/README.md) retain
bounded slices and source/hash provenance. Independent review caught the
unreachable-for-real-magnitude negative scalar branch; the API explicitly
rejects negative magnitude instead of inventing a multiplier write. Nine new
tests pass, including zero input's native Run fallback, exact thresholds,
signed zero, overflow, unknown IDs and numeric DTO representation.

[Implementation validation](../implementation-player-state-2026-11-11.md)
records the 36 passing data tests, strict scoped Clippy and workspace check.
The public data API still does not activate source movement or a state machine.
Next trace invocation/transition gates and effective runtime inputs before
shared posture/physics integration. Host atan2 rounding equality, source
runtime patches and original movement/collision parity remain unverified.
