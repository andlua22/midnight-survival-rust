# Player clearance and crouch callback Rust validation

2026-11-12. [validation.json](validation.json) records the final code and log
SHA-256 identities, retained native evidence references, commands and results.
Test commands are labeled as reproduction commands because their retained logs
omit the original shell invocation; check/Clippy invocations are recorded.
All **56 native artifact/source records (54 unique files)** matched their
recorded hashes. The [clearance audit](../player-clearance-native/README.md)
and [transition audit](../player-transitions-native/README.md) retain the
source/native contracts used by the implementation.

The shared query uses the explicitly supplied RootBone world position,
upward sphere radius 0.2, CrouchUp/PaToStand distances 1.5/1.9 and source mask
0x40080001. Imported prefab solid layers and skipped trigger layers survive
replacement/removal atomically. Missing collider layers, included
unrepresented triggers and unresolved initial overlap return explicit errors.
The host supplies the runtime `queriesHitTriggers` boolean used by UseGlobal.

The source-only OnClickCrouch2 continuation preserves the upper Attack gate,
fresh base-state reads, separate PaUp/CrouchUp verdicts, distinct panel/global
player targets and Play2D IDs 339/435 in their native call order. Each call
must return before the next request is emitted; acknowledging a SwitchState
return does not claim a committed state.

Validation retained here:

- `workspace-tests.log`: **199 passed, 0 failed, 10 ignored**, including all
  44 final data and 63 shared tests.
- `retained-supermarket-tests.log`: the separate ignored real-package test
  passed with **55 solid colliders and 1 skipped trigger**, including matching
  source layer metadata in independent shared worlds.
- `workspace-check.log`: all-target workspace compatibility check passed.
- `data-posture-clippy.log` and `shared-lib-clippy.log`: strict final data
  all-target and shared library Clippy checks passed.
- `data-shared-tests.log` and `data-clippy.log`: preserved earlier
  clearance-only validation, before the five callback tests were added.
- `data-posture-tests.log`: all 44 final data tests passed.

Scoped rustfmt and `git diff --check` passed. A broader formatting check of
`shared/physics.rs` still reports inherited formatting differences; its
introduced re-export order was corrected and the final hash refreshed without
a behavioral change.

Full strict workspace Clippy still has five inherited warnings from earlier
work; the scoped checks above do not claim that it is clean. Earlier GPU
captures and startup checks remain separate; this milestone did not execute
the original game or run a new GUI probe.

Rig RootBone/HeadTop transforms, terrain/player layer metadata, complete
trigger/collider coverage, distinct PaUp ray dependencies and source FSM
switch arbitration remain required for active runtime binding. Rapier cast
tests do not establish PhysX overlap/tolerance or IFix runtime parity. No
desktop crouch binding, physical posture transition or audio playback was
activated by this milestone.

[Implementation and continuation](../../implementation-player-clearance-2026-11-12.md).

