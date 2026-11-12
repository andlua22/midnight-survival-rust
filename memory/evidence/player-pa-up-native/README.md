# CheckCanPaUp native continuation — 2026-11-12

This audit reuses the retained Rust/Capstone 5 helper and recovered AArch64 ELF.
The ELF SHA-256 remains
`2e2efde1a103549694b17b361cacc18faa5714d06c444945e44b30f1377e2b3c`.
[manifest.json](manifest.json) records source/helper/artifact hashes and exact
bounded reads. These are static observations of unpatched procedures; original
execution, IFix state and physics parity were not observed. Existing evidence
and imported packages were preserved.

The complete CheckCanPaUp body is already retained in
[the preceding audit](../player-clearance-native/check-can-pa-up.asm), range
`0x1ec332c..0x1ec35d0`, IFix ID 3312. It establishes this exact order:

1. Read `HeadTop.position` (controller field `0x80`). SphereCast upwards with
   radius `0.1`, distance `1.3`, mask `0x40080001`, and UseGlobal. A hit returns
   false immediately.
2. Only after a miss, read `PlayerTransform.position` (field `0x40`). Form a
   ray origin by adding world up times `0.3`; cast down for distance `1.0`.
3. The ray mask is `1 << BattleScMgr.DefaultLayer`. The native class slot is
   `0x46283d0`; the static field offset is `0x28`. The class's own constructor
   at `0x217352c` uses that slot and writes `SceneLayerMask` at `0x5c`, agreeing
   with the retained declarations. Its effective DefaultLayer value is still
   a runtime input; the name alone does not authorize substituting layer zero.
   AArch64 `LSL W` and C# int shifts mask the shift count to five bits.
4. A ray miss returns true. A hit reads `RaycastHit.transform`, then
   `GetComponentInParent<ElevatorCabin>()`, and returns false when that component
   compares non-null under Unity's Object equality.

The [rig metadata audit](../player-rig-native/lookup-generics.txt) resolves
method slot `0x45d63f0` to token `0xc0026e63`, MethodSpec 79665, source method
115124 (`UnityEngine.Component.GetComponentInParent`) and method instantiation
7090. Its single argument resolves to `ElevatorCabin` through the retained
counterpart type-definition index `0x259`; the exported declaration's token is
`0x200025A`. The decoded source MethodSpec is required because counterpart
MethodSpecs are encrypted. See the
[slot proof](../player-rig-native/lookup-slots.tsv) and the
[artifact/method provenance](../player-rig-native/README.md). Runtime presence
and active-parent ancestry of that component still require supplied evidence.

The additional complete native wrappers establish:

- [Raycast](raycast-default-trigger.asm), `0x3477b60..0x3477c2c`, uses the
  default PhysicsScene and explicitly passes trigger interaction zero. The
  [PhysicsScene wrapper](physics-scene-raycast.asm) normalizes the nonzero
  direction and forwards layer/trigger arguments unchanged to the exact
  [Internal_Raycast_Injected binding](raycast-injected-name.txt).
- [RaycastHit.transform](raycasthit-transform.asm),
  `0x347aaac..0x347ab88`, prefers the attached Rigidbody's Transform. Only when
  that is Unity-null does it use the collider Transform. The attached-body
  binding is independently retained in [this wrapper](raycasthit-rigidbody.asm)
  and [the exact native name](attached-rigidbody-name.txt). A collider's own
  hierarchy therefore cannot always substitute for the hit transform.
- [Generic GetComponentInParent](getcomponentinparent-generic.asm),
  `0x25811ec..0x25812b4`, resolves its generic type and passes
  `includeInactive = false` to `0x342a67c`. The
  [type-taking wrapper](getcomponentinparent-type.asm) gets the Component's
  GameObject and forwards that flag to the exact
  [GameObject.GetComponentInParent(Type, bool) binding](getcomponentinparent-name.txt).

The shared implementation keeps the HeadTop sweep and ground ray separate.
It requests PlayerTransform/layer inputs only after the head sweep misses and
requests the parent-component result only after a unique nearest ray hit.
Unknown global trigger state, hit-transform/component identity, coverage,
initial overlaps and equal-distance hit arbitration remain explicit errors.
No self exclusion or posture transition is inferred. Original PhysX tolerances,
backface rules, runtime activation and full collider coverage remain separate
acceptance work.
