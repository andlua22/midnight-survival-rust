# Posture clearance: bounded native evidence

2026-11-11. This source audit reuses the unchanged local Rust/Capstone 5.0
helper and supplied recovered AArch64 ELF. It changes no production code and
executes no original game, GUI, service or protected-cache action. These are
static local observations, not REA provider Evidence IDs or runtime parity.

The source APK is `export/lite4.0.apk`, SHA-256
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
The retained ELF has 76,608,224 bytes and SHA-256
`2e2efde1a103549694b17b361cacc18faa5714d06c444945e44b30f1377e2b3c`.
`manifest.json` records source/helper hashes and all inspected ranges. Every
assembly range decoded completely. Prior evidence was preserved.

## Crouch and Pa to standing clearance

| Base method | Exact range (exclusive end) | IFix ID | Origin | Radius | Maximum distance |
|---|---|---:|---|---:|---:|
| PlayerController.CheckCanCrouchUp | 0x1eb3344..0x1eb3434 | 3264 | RootBone.position | 0.2 | 1.5 |
| PlayerController.CheckCanPaToStand | 0x1ec35d0..0x1ec36c4 | 3263 | RootBone.position | 0.2 | 1.9 |

Both unpatched paths load the Transform at source/native offset 0x98,
`BasePlayerController.RootBone` (not `PlayerTransform`, which is at 0x40).
They pass its directly read world position without any added height/offset.
The position wrapper at 0x3438a14 resolves the exact native string
`UnityEngine.Transform::get_position_Injected(UnityEngine.Vector3&)`.
Null RootBone/patch references take the native failure path, not a clear
result. Do not substitute actor feet, a collider center or camera eye position
for the source rig's RootBone position.

Direction is the Vector3 static slot at 0x18/0x20. Its initializer at
0x34309d8 writes the 64-bit constant from 0x361f7a8 (float components 0,1)
and zero Z at 0x20, proving direction **(0,1,0)**. The same initializer writes
zeroVector at offsets 0/8; this corroborates the preceding movement audit's
previously unexamined zero normalization fallback. The up getter at 0x3430580
reads the same slot. No standing capsule geometry or RootBone scale affects
the clearance radius/distance in these base methods.

Radius is source bits `0x3e4ccccd` at 0x361f584. Crouch distance is an immediate
float 1.5; Pa-to-Stand distance is source bits `0x3ff33333` at 0x3622ccc.
Both calls supply integer layer mask **0x40080001 / 1074266113**, containing
exact layer indices **0,19,30**. These are numeric source filter identities;
the audit does not invent layer names or infer every imported collider's layer.

The SphereCast overload at 0x34788b8 explicitly writes query-trigger value 0
and calls 0x34787e4. The latter obtains the default PhysicsScene and passes
mask and query-trigger value unchanged to PhysicsScene.SphereCast at
0x347d9d4. Its query wrapper at 0x347d8ac resolves:

```text
UnityEngine.PhysicsScene::Query_SphereCast_Injected(UnityEngine.PhysicsScene&,UnityEngine.Vector3&,System.Single,UnityEngine.Vector3&,System.Single,UnityEngine.RaycastHit&,System.Int32,UnityEngine.QueryTriggerInteraction)
```

Numeric query-trigger 0 follows Unity's UseGlobal contract; the selected global
queriesHitTriggers value is not supplied by these methods or observed here.
The generic PhysicsScene wrapper normalizes a nonzero direction; fixed up has
unit length and passes. Neither player method requests collider/self exclusion,
a second overlap check, a capsule cast or an endpoint-only test.

The returned boolean is **not SphereCast hit**: eor #1 at 0x1eb3414/0x1ec36a4,
then final bit-zero masking. Any matching hit blocks the respective clearance
check. Original PhysX initial-overlap/contact/tolerance/cooking semantics and
runtime full collider coverage are not established by these call arguments.
An unresolved initial overlap must not be used to claim clearance in a
reconstruction adapter.

## Distinct PaUp contract

`CheckCanPaUp` at 0x1ec332c..0x1ec35d0 is retained for the next transition
review. It first SphereCasts from **HeadTop.position** (field 0x80) upward,
radius **0.1**, maximum distance **1.3**, the same mask/trigger policy. A hit
immediately rejects. If clear, it casts a downward ray of distance 1 from
**PlayerTransform.position + up * 0.3**. That ray's layer mask comes from
`1 <<` a static value at a distinct class slot; a hit can additionally reject
based on GetComponentInParent of an unresolved generic component type.
Those layer/component identities were not established by this bounded audit.
It is therefore not modeled as either simple RootBone standing cast.

## Implementable boundary

A source request can safely retain the exact cast kind, explicitly supplied
RootBone world origin, up direction, radius/distance, integer layer mask and
query-trigger 0. Its result projection is !hit when an authoritative complete
cast result is supplied. A shared physics adapter must explicitly bind source
layers and UseGlobal state and report incomplete collider/trigger coverage or
initial overlap as unresolved. This does not activate a complete posture
transition or prove original-runtime physics equality.

Reproduce a retained slice with:

```sh
/tmp/codex-minimap-review/native disasm /tmp/codex-minimap-review/libil2cpp.so 0x1eb3344 0xf0
```

The helper validates ELF64 little-endian AArch64 and maps bounded requests
through file-backed PT_LOAD segments. It was not rebuilt. No Cargo or Python
was used; root owns implementation and validation.
