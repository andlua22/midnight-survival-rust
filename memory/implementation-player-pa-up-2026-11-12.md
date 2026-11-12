# Distinct PaUp clearance boundary — 2026-11-12

`CheckCanPaUp` now has a separate source request and shared query path. It
preserves the HeadTop sphere cast, the conditional PlayerTransform ground ray,
and the conditional parent-component check. The existing source stand-up
requests and active prototype controller remain as before.

[Source request types](../crates/game_data/src/clearance.rs) retain the sphere
radius `0.1`, distance `1.3`, mask `0x40080001`, the ground-ray offset `0.3`,
downward distance `1.0`, and exact signed C# layer-shift semantics. Unity and
Bevy position constructors explicitly distinguish the single Z reflection.
HeadTop and PlayerTransform are separate inputs; neither is inferred from
the actor capsule or a render-frame transform.

[The shared adapter](../crates/shared/src/physics/source_clearance.rs) samples
ground inputs only after the head sweep misses. It asks for the source parent
`ElevatorCabin` parent component only for the unique nearest ground hit. A miss
allows PaUp; a matching component blocks it. Unknown global trigger policy, source layers,
unrepresented triggers, missing ground inputs, initial overlaps and equal
nearest hits remain explicit errors. No self exclusion is added. The caller
must resolve the original hit transform and generic component query; arbitrary
collider names or broad component categories do not establish that result.

[Native continuation evidence](evidence/player-pa-up-native/README.md) proves
the ray's layer field is `BattleScMgr.DefaultLayer`, the ground ray also uses
UseGlobal, `RaycastHit.transform` prefers an attached Rigidbody, and
GetComponentInParent passes `includeInactive = false`. The accompanying
[rig metadata audit](evidence/player-rig-native/lookup-generics.txt) resolves
the exact generic argument to `ElevatorCabin`. Its per-hit presence and the
effective runtime layer/trigger values remain explicit dependencies. No
original game or GUI was run for this query boundary.

[Validation](evidence/player-pa-up-rust/validation.json) records 11 added tests:
three request/conversion tests and eight shared query tests. The focused runs
pass seven game-data clearance-related tests and twelve shared clearance
tests, including the retained stand-query tests. Strict focused Clippy with
`--no-deps` and scoped formatting/diff checks pass. Strict Clippy including
local dependencies still reports three inherited `collapsible_if` findings in
`map_data/src/package.rs`; this change does not claim a clean whole-workspace
lint result.

This is a verified request/query boundary. Source rig pose admission, runtime
component coverage and PhysX overlap/contact/tolerance semantics remain
required before source FSM activation or original-game movement parity.
