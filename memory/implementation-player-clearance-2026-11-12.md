# Source standing clearance and crouch callback — 2026-11-12

The next player boundary is implemented: explicit source stand-up casts,
shared Rapier queries with retained prefab layers, and the ordered
`SC.UI.BattlePanel.OnClickCrouch2` continuation. The existing Rust/Bevy 0.20.1
workspace builds without new dependencies or lockfile changes. Earlier
inventory, terrain, UI, environment and player-definition changes are preserved.

## Concrete result

[clearance.rs](../crates/game_data/src/clearance.rs) retains the two distinct
unpatched source requests:

| Source method | World origin | Direction | Radius | Maximum distance | Unity layer mask |
|---|---|---|---:|---:|---|
| CheckCanCrouchUp | RootBone.position | (0,1,0) | 0.2 | 1.5 | 0x40080001 |
| CheckCanPaToStand | RootBone.position | (0,1,0) | 0.2 | 1.9 | 0x40080001 |

The caller supplies the identified rig's RootBone world position. Constructors
accept already converted Bevy coordinates or raw Unity coordinates with one
Z reflection. Finite-position validation is a reconstruction admission rule.
The mask contains numeric Unity layers 0, 19 and 30. Trigger interaction is
the observed `UseGlobal = 0`; its actual `queriesHitTriggers` value must be
supplied explicitly by the runtime adapter.

[source_clearance.rs](../crates/shared/src/physics/source_clearance.rs) adds
`CollisionWorld::check_source_stand_clearance`. It casts the sphere against
the installed shared query world and projects a hit to blocked standing.
It returns explicit errors for unknown layers on participating installed
colliders, selected unrepresented triggers, and initial overlap/contact whose
PhysX behavior has not been matched. It performs no player/posture mutation
and adds no inferred self exclusion. This API is shared by authority and
prediction; full original collider coverage remains a separate prerequisite.

[source_prefabs.rs](../crates/shared/src/physics/source_prefabs.rs) now retains
each enabled solid collider's source GameObject layer and the layers of
skipped triggers. Layers must be in 0..31. Shape/layer preparation completes
before instance replacement, so invalid layers or a later unsupported shape
leave the old collision and metadata intact. Replacement and removal clean
the corresponding metadata. Existing physical trigger behavior is preserved.

[posture.rs](../crates/game_data/src/posture.rs) models one explicitly selected
`OnClickCrouch2` invocation as a bounded continuation. Hosts supply each fresh
FSM state read, the separate PaUp/CrouchUp clearance verdicts, and acknowledgments
that requested audio/switch calls returned. It preserves these source branches:

- Upper-body Attack (16) finishes before reading the base FSM.
- Pa (2) requests the distinct CheckCanPaUp; true requests panel-self PaUp (3),
  while false proceeds to a fresh base-state read.
- Stand (1) requests Play2D(339), then the battle-self player's Crouch (4).
- Crouch (4) requests CheckCanCrouchUp; true requests panel-self Stand (1),
  then Play2D(435) only after the switch call returns. False finishes.
- Other base states finish. Wrong observation kinds leave the continuation
  unchanged. Panel-self and battle-self player identities remain distinct.

Switch requests preserve null arguments and caller order; a returned switch
does not establish a committed state. Audio IDs/order are retained, with
assets and full playback settings still external. No desktop posture binding
or active source FSM/controller is added by this boundary.

## Evidence and validation

The bounded native audits retain complete clearance/callback bodies, their
wrappers and selected invocation dependencies:
[clearance evidence](evidence/player-clearance-native/README.md) and
[transition evidence](evidence/player-transitions-native/README.md).
All 56 manifest artifact/source records, representing 54 unique files,
matched their retained SHA-256 identities. The unchanged Rust/Capstone helper
was reused; these are static local observations of unpatched procedures.

The transition audit also establishes unconditional Move invocation within
ordinary admitted Stand/Crouch Act paths, death/load gates in Player.Update,
and FSM Ticker → fresh CurrentState.Act → fresh CurrentState.Reason ordering.
The Vector3 initializer proves both the up direction and zero fallback used
by the preceding movement audit. It does not establish runtime patch state,
root-motion displacement or physical movement parity.

- Fourteen new tests cover cast geometry/conversion/admission, shared mask and
  unresolved-query behavior, independent worlds, metadata lifecycle/rollback,
  and callback gates, targets, ordering and fresh reads.
- Final workspace tests: **199 passed, 10 ignored**, with no failures.
- The separately selected real supermarket-package test passed: **55 solid
  colliders, 0 inactive/disabled, 1 trigger**, including matching layer metadata.
- All-target workspace check, strict all-target game_data Clippy and strict
  shared-library Clippy passed. Scoped rustfmt and `git diff --check` passed.
  Earlier full-workspace strict Clippy has five inherited warnings; this
  milestone does not claim a warning-free full workspace.

[Rust validation](evidence/player-clearance-rust/README.md) retains logs,
commands, code identities and machine-readable results. Earlier Linux/GPU
startup captures keep their original scope; rendering code was unchanged.

## Next implementation work

1. Recover the actual player rig hierarchy, RootBone/HeadTop references and
   effective world transforms/scaling. Retain the source terrain GameObject
   layers in a verified compatible collision-package import; current terrain
   and provisional player colliders have unknown layers for this query.
2. Resolve the selected global trigger policy and collider/trigger coverage.
   Match relevant PhysX overlap/contact/tolerance behavior before asserting
   original clearance parity.
3. Finish the distinct CheckCanPaUp query. Its first cast uses HeadTop, radius
   0.1 and distance 1.3; its subsequent downward ray uses PlayerTransform and
   unresolved dynamic layer/component conditions. PaToStand is not a substitute.
4. Recover FSM switch arbitration, effective modifiers, capsule scaling and
   animation/root-motion/jump/landing dependencies, then bind admitted source
   state and callback requests to shared simulation, Bevy UI and audio.

The terrain importer already resolves each renderer's qualified GameObject in
[pass2_terrain.rs](../tools/asset_unpacker/src/pass2_terrain.rs). That is the
concrete extraction point for strict class-1 `m_Layer` values. The current
[terrain model](../crates/map_data/src/lib.rs) uses manifest schema 2 with
MSMAP003 geometry; a compatible metadata extension must keep schema-2 layers
explicitly unknown. Existing geometry can be retained while deduplicated
GameObject metadata is resolved, avoiding mesh/texture re-extraction.
The current collection is renderer-driven and retains one collider per
GameObject. Audit collider-only objects and multiple colliders separately;
metadata enrichment alone does not establish complete source coverage.

Authentic world placements and inventory snapshots/session admission remain
open parallel dependencies. Active movement/camera still use provisional
values. No original game, GUI, Android build, source service, protected-cache
retry, Python, cleanup or system change was needed for this milestone. The
read-only disk check found about 127 GiB free; no cleanup was required.
