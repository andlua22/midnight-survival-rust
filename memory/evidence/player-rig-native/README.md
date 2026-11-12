# Player prefab, binding, and animation bridge evidence

Static native evidence recovered on 2026-11-12 for Last Island of Survival LITE
4.0. This report establishes the unpatched paths in the supplied IL2CPP memory
dump. It does not claim that the original game was executed, that IFix patches
were inactive in a live session, or that runtime pose/physics parity was observed.

## Artifact identities

| Artifact | SHA-256 |
| --- | --- |
| `export/lite4.0.apk` | `2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df` |
| `/tmp/codex-minimap-review/libil2cpp.so` (source dump) | `2e2efde1a103549694b17b361cacc18faa5714d06c444945e44b30f1377e2b3c` |
| `/tmp/codex-memory-dump-review/source/global-metadata.dat` | `548b4e3e990641716e6e368de780fbf8c82f9806c0238d01e3bc02d0d644f2e6` |
| `/tmp/codex-memory-dump-review/candidate/libil2cpp.so` (retained encrypted counterpart) | `ed3ec656863b106bca6bd62dc2a11283b47c2c3c3c1634e4070fd386447f8c68` |

The APK/counterpart relationship is recorded in
`memory/evidence/apk-runtime-member-comparison.json` and
`memory/apk-reconstruction.md`: 9,318 packaged assets/files match; ELF and metadata
are different artifacts. Counterpart tokens and type records are corroboration,
not bytes silently substituted into the source dump.

`inspect_slots.rs` reads the dynamic RELA table rather than the source dump's
damaged section inventory. Selected source/counterpart R_AARCH64_RELATIVE addends
must agree. Encoded IL2CPP usages use `kind = usage >> 29` and
`index = (usage & 0x1fffffff) >> 1`. The helper reports both raw usage words.
Some source usages are already initialized pointers; those literal/type names
are explicitly counterpart-corroborated. `inspect_generics.rs` uses decoded
source MethodSpec records, verifies the pointer chain against source relocations,
and reports both source and counterpart native type records. In the selected
MethodSpec records the counterpart's bytes are encrypted; it is not a valid
method-index source. Several source native type records have initialized data
pointers; the counterpart retains their metadata type-definition indices.

## Verified selection and bindings

| Claim | Native evidence | Scope |
| --- | --- | --- |
| Self player selects `role_new` in `role/role.ab` | `Battle.<LoadSceneFinish>d__299.MoveNext`, `0x2161018..0x216161c`; AssetMgr.LoadAsset call `0x2161318` | `load-scene-finish-next.asm`; `role_new` survives as a source token; bundle name needs counterpart corroboration |
| Instantiated player comes from `Battle.RolePrototype` | The same state machine reads field `+0x40`, calls Instantiate at `0x25b1ac0`, resets position/rotation | Static call/data flow |
| Camera initialization precedes InitSelfPlayer | The same path calls `vThirdPersonCamera.Init(player.transform, "Stand")` at target `0x16ee608` | Camera mode selection and order only; numerical camera behavior is separate |
| `HeadTop` field binds `GetOriChildObjByName("Head")` | `BasePlayerController.InitBodyParts`; field `+0x80` store at `0x1e4d960` | Source literal chain: slot `0x45cce78` → `0x47d6410` → `0xa0001db9` → literal 3804 |
| `RootBone` field binds `GetOriChildObjByName("Bip001")` | Same method; field `+0x98` store at `0x1e4d9f8` | Source literal chain: slot `0x45b5108` → `0x47d2f48` → `0xa0001081` → literal 2112 |
| PlayerCollider lookup uses child `Collider` | `BasePlayerController.InitPrototyle`, `0x1e4cf98..0x1e4d2c8` | Source literal: slot `0x463cc48` → `0x47d3e08` → `0xa0001433` → literal 2585; serialized geometry remains separate |

Do not bind HeadTop to the name `HeadTop` or to `Bip001 Head`. The field names
above are native fields; the literal lookup operands establish which prefab
objects supply them. Whether a live instance has overridden those references is
not observed here.

Base `InitPrototyle` binds PlayerTransform from `this.transform`, obtains the
root Animator, obtains/adds `SC.AnimatonPlayer.AnimatorHelper`, adds a
SkinnedMeshRenderer on the root, binds Collider and other components, and calls
InitLayerIndex → InitBodyParts → InitGuaDian → InitAimationStates → InitIK.
PlayerController's override (`0x1eb3604..0x1eb3aec`) calls the base initializer,
adds event subscriptions, and obtains its Rigidbody at field `+0x418`.

## Lookup precedence and ordering

`BasePlayerController.Init` (`0x1e4c094..0x1e4c2ec`) calls SaveBones. SaveBones
(`0x1e4c2ec..0x1e4c67c`, end exclusive) obtains
`Component.GetComponent<GameObjectData>()` using MethodSpec 79506 at
`0x386127c` (`method=115114`, `methodInst=7305`). Its generic argument resolves
through `0x4490768` → `0x447d0a0` → `0x4755ad8` → native type `0x4814768`;
the counterpart type-definition index is 1583 (`GameObjectData`).

SaveBones reads `GameObjectData.GameObjects` at `+0x18`, allocates m_RootBones
(`+0x238`) with the list's count, and iterates from index zero upwards. It skips
non-live objects, stores each live object's transform at the same array index,
and adds name→transform to m_NameToBone (`+0x248`) only if that name is absent.
Duplicate names therefore retain the first live occurrence in the authored
list, not a name-sorted or inferred bone order. It then queries `Bip001` in that
dictionary for the private m_RootBone GameObject (`+0x240`).

`GetOriChildObjByName` (`0x1e4c7b0..0x1e4c91c`) first queries m_NameToBone and
returns the live Transform if present. Otherwise it calls
`Utils.GetChildObjByName(name, this.gameObject)` and returns that object's
transform or null. `Utils.GetChildObjByName` (`0x21581d0..0x21582f4`) invokes
`GameObject.GetComponentsInChildren<Transform>(true)` and returns the GameObject
of the first exact name match in the returned array. The generic is MethodSpec
80782 at `0x3864e4c` (`method=114904`, `methodInst=152`), resolving to native type
`0x47f69d8`, counterpart type-definition 15563 (`UnityEngine.Transform`). The
native code establishes array iteration order; this report does not invent a
Unity traversal order for duplicated fallback names.

## Animator layers are not physics layers

`InitLayerIndex` (`0x1e4d2c8..0x1e4d418`) calls Animator.GetLayerIndex
(`0x33f97f0`) and stores:

| Field offset | Animator layer name | Literal slot |
| --- | --- | --- |
| `+0x2b0` BaseLayer | `Base Layer` | `0x4626918` |
| `+0x2b4` UpperBodyLayer | `UpperBody Layer` | `0x460c388` |
| `+0x2b8` FullBodyLayer | `FullBody Layer` | `0x45c0070` |
| `+0x2bc` LeftArmLayer | `LeftArm Layer` | `0x45e99c8` |
| `+0x2c0` RightArmLayer | `RightArm Layer` | `0x46227f0` |

These names do not establish physics layer numbers, layer masks, effective
trigger settings, or runtime collider activation.

## Runtime clip replacement bridge

The source helper is `SC.AnimatonPlayer.AnimatorHelper`, not
AnimatorTransitionHelper. `Init` (`0x133f24c..0x133f3ac`) loads
`role/animationbundle.ab`. Its load callback (`0x13416d8..0x1341b8c`, bounded
excerpt in `animator-helper-cctor.asm`) loads TextAssets
`Assets/Roles/RoleAnimNameMap.json` and
`Assets/Roles/RoleAnimState2ClipMap.json`, reads TextAsset.text, and decodes JSON.
The literal slots are respectively `0x463b228` (literal 1920) and `0x45c3f78`
(literal 1921). The helper holds shared static fields m_controller (`+0x0`),
m_clipNameMap (`+0x8`), m_stateHash2ClipMap (`+0x10`), AnimationAb (`+0x18`), and
m_loadedClip (`+0x20`).

`Awake` (`0x133f3ac..0x133f6ac`) obtains the Animator; if the shared override
controller does not exist, it constructs one from the current
runtimeAnimatorController. It assigns the shared override to the Animator and
subscribes animation events.

On the normal mapped path, `ReplaceAnimationCore(_hash, layer, callback)`
(`0x133f7c8..0x133fc10`) checks `m_stateHash2ClipMap.ContainsKey(_hash)`, records
the hash at the selected layer's current-state index, gets
`idx=m_stateHash2ClipMap[_hash]`, gets `clipName=m_clipNameMap[idx]`, and calls
`LoadClipCore(idx, clipName, wrappedCallback)`. Its early guards and exception
fallback must not be replaced by an assumed general FSM policy.

`LoadClipCore` (`0x133ff38..0x13400dc`) checks m_loadedClip by **clipName**. A
previously loaded name invokes the callback if supplied, then returns. A new
name is added to the set before `AnimationAb.LoadAsset(clipName)`; the callback
runs **before** `ReplaceClip(idx, loadedClip)`. `ReplaceClip`
(`0x13400dc..0x13401b0`) writes the shared AnimatorOverrideController's named
entry at `0x1340188` (target `0x33f86d0`), then calls **Animator.Update(0)** at
`0x13401a8` (target `0x33fa1f4`). This makes replacement/callback ordering
material to a later timing implementation. JSON content, controller state
evaluation, clip decompression/retargeting, root motion and animation events
must be verified independently before source animation activation.

## PaUp generic component condition

Slot `0x45d63f0` → `0x47b15e0` contains source token `0xc0026e63` → MethodSpec
79665. Decoded source spec at `0x38619f0` is
`(method=115124, classInst=-1, methodInst=7090)`:
`UnityEngine.Component.GetComponentInParent<ElevatorCabin>`. Generic instance
7090 resolves `0x44900b0` → `0x447c330` → `0x4755420` → native type
`0x480d288`. The source type data is initialized; the counterpart preserves
type-definition index 601 (`ElevatorCabin`) and the same type attributes. PaUp
cast/ray semantics and the still-unresolved effective DefaultLayer are tracked
separately under `memory/evidence/player-pa-up-native/`.

## Boundaries and remaining work

- Preserve authored collider radius/height/center/direction/trigger/enabled
  values separately from runtime posture writes. This report does not establish
  effective capsule scaling/clamping, live layers, or initial-overlap behavior.
- RootBone/HeadTop query origins require authoritative sampled hierarchy and
  pose; no render-frame Transform is an authoritative source.
- IFix guards exist in the inspected methods. Native unpatched paths are
  established; active hotfix scripts and live dispatch are not.
- `save-bones.asm` also includes a small next-function prefix after `0x1e4c67c`.
  `animator-helper-cctor.asm` includes the cctor, ending at `0x134166c`, and
  subsequent compiler-generated callbacks; the file is not one function.
- Ghidra 12.1.2's initial cold import returned provider_timeout at 330 s. After
  raising its startup budget to 900 s, the retry hit the host's 450 s tools/call
  deadline. No pseudocode was obtained and no OOM was confirmed. On resume the
  session reported cleanup_pending=false and analysis_activity=not_observed.
  A repeated cold import is not required for the byte-level claims above.
- Metadata-only REA inspection succeeded (parent
  `ev_5849ca51956c07987800616c6ab46655dbe3004f7fe539d3c71f9c9dae988d88`,
  summary `ev_5730e067f9baf0392c87819fb7eeb2bba97d052b51e8c025dbaf95beedf36978`).
  Binary-linked attempts were partial failures: source dump invalid initialized
  type index and counterpart code-gen modules not found. Those failures do not
  erase direct, bounded native and metadata observations.
