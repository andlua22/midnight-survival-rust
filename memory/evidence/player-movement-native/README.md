# Player direction and orientation: native audit

2026-11-11. Read-only followup for source-only player state/classification work.
No production code, Cargo build, original-game runtime, GUI, service or cache
was used. The artifacts are bounded local Rust/Capstone disassembly, not REA
Evidence IDs or observed gameplay. Prior reconstruction evidence remains intact.

## Provenance

The retained supplied ELF is `/tmp/codex-minimap-review/libil2cpp.so`,
76,608,224 bytes, SHA-256
`2e2efde1a103549694b17b361cacc18faa5714d06c444945e44b30f1377e2b3c`.
Its recorded APK identity is
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
The dump pairing and prior source caveats are retained in
[player-rules-source-review.md](../player-rules-source-review.md).

The unchanged Rust helper `/tmp/codex-minimap-review/native` is SHA-256
`000d6b773e4f8ed574b62b1803a1d4d1314bd0178262359cb3a176a5ac7fd8e9`;
its source `native.rs` is SHA-256
`91d59a9b3b42ca2e4af1880151decd98b5c00d982978c4e5a9848f8eea03bf18`.
It verifies ELF64 little-endian AArch64 and maps bounded ranges through
file-backed PT_LOAD segments. Every retained assembly request decoded fully.

Source declaration SHA-256 values:

- `export/PlayerController.cs`:
  `c835524a62f5b674c36cc3772ba5de1b3ba191de52699889266df34a2a34bd56`
- `export/BasePlayerController.cs`:
  `b1322444dcd7a3aa5a13218af6ba6b4359843410e861456036e995ea2632c541`
- `export/MathUtils.cs`:
  `a8649600651a77e21df23ceb3a4599c941a6cb7481a43c4da2f63997410d0d7e`
- `export/SC/ActorStatusType.cs`:
  `e01897c882025456f8e1956326c76829b7d5564d4bab3e296244aa5f4cc49f57`

## Raw input identity and magnitude

`StandState.Move` and `PlayerController.ChangeVelocity` read the two floats at
native PlayerController offsets 0x3a8/0x3ac. The constructor writes the entire
Vector2 at 0x1ecb15c and then another Vector2 at 0x3b0/0x3b4. These correspond
to source `Input` and `LastJumpUpInput` (PlayerController.cs lines 1350/1354 and
58118/58121). The source's external-value-type offsets are damaged and overlap;
its printed Input offset 0x398 is not the observed native offset.

This is distinct from BasePlayerController's `InputVector`: native getter
0x1e4fdb0 reads 0x2a4/0x2a8, as retained in `base-input-getter.asm`.

`StandState.Move` computes magnitude at 0x1e7b818..0x1e7b834 with separate
single-precision `fmul(x,x)`, `fmul(y,y)`, `fadd`, then `fsqrt`. The result is
converted from float to double before the rush-threshold comparison. There is
no fused multiply-add or double-precision square/sum. Finite raw inputs can
still overflow these operations; rejecting overflow is reconstruction admission
policy, not a recovered source guard.

## GetOrientation operation order

`orientation-overloads.asm` retains the complete scalar method
0x21205e4..0x2120668 and Vector2 overload 0x2120668..0x21206c8. Both contain
IFix gates, so this note describes their unpatched base paths.

The scalar signature `GetOrientation(float x, float y)` passes `(y,x)` to
`atan2f`. The Vector2 overload swaps its input components before calling the
scalar overload. Consequently the Vector2 contract used by StandState.Move is:

```text
angle = atan2f(raw_input.x, raw_input.y)
angle = angle * f32::from_bits(0x42652ee0)
if angle < 0: angle = angle + 360.0f
```

The multiplier at 0x3623834 is exactly bits 0x42652ee0, printed as 57.295776.
Use the retained bits rather than substituting a language's generic degrees
conversion. Multiply and add are separate float operations. The native method
does not clamp, apply a deadzone, perform modulo or add 360 to signed zero.
Native libm atan2f implementation/rounding was not executed or compared with
the host Rust implementation; mathematical correspondence does not establish
cross-platform bit equality. Preserve supplied orientation values when exact
runtime observations become available.

`atan2f-plt.asm` shows the call at 0xa4ea40 loading GOT slot 0x45ac880.
`readelf --use-dynamic -rW` identifies that slot's symbol as atan2f, and
`readelf --use-dynamic -sW` identifies dynamic symbol 247 as undefined atan2f.
The ELF has no section table. The dynamic relocation scan reports bad symbol
indices for other entries, retained in `atan2-relocations.txt`; this is a
selected valid-row corroboration, not a claim that all dynamic symbols or
relocations were recovered successfully.

## ChangeVelocity base behavior

`change-velocity.asm` retains 0x1ebdf20..0x1ebe12c, including IFix gate 11656.
The base path:

1. Reads `PlayerController.Status` at native offset 0x48c. Getter/setter
   0x1eab794/0x1eab79c corroborate the field. Source return type matches
   `SC.ActorStatusType`; its declared `CannotMove` value is 1. Status mask 1
   changes the requested horizontal speed to zero; other bits do not trigger
   that particular test.
2. Reads the component transform, gets Euler angles and uses only its Y
   rotation. It multiplies yaw by the float radians constant 0x3c8efa35 at
   0x361f58c, then calls Unity's `Internal_FromEulerRad_Injected` with
   `(0,yaw_radians,0)`.
3. Multiplies that quaternion by source vector `(Input.x,0,Input.y)` using
   the retained quaternion/vector arithmetic at 0x34320cc.
4. Computes the rotated vector's full magnitude in float. If magnitude is
   greater than the float threshold bits 0x3727c5ac (printed 0.00001), it divides
   rotated X and Z by that magnitude. Otherwise it takes X and Z from the
   Vector3 static path at slot 0x462e2f8, offsets 0/8 (the conventional zero
   fallback; its runtime static initializer was not inspected here).
   There is no proportional analog-magnitude
   scaling above the normalization threshold.
5. Writes velocity X/Z as normalized direction times the gated request speed,
   reads the current Rigidbody velocity Y and preserves it, then calls
   `UnityEngine.Rigidbody::set_velocity_Injected`.

The retained Unity wrappers resolve these exact strings:
`UnityEngine.Component::get_transform()`,
`UnityEngine.Transform::get_rotation_Injected(UnityEngine.Quaternion&)`,
`UnityEngine.Quaternion::Internal_FromEulerRad_Injected(UnityEngine.Vector3&,UnityEngine.Quaternion&)`,
`UnityEngine.Rigidbody::get_velocity_Injected(UnityEngine.Vector3&)`, and
`UnityEngine.Rigidbody::set_velocity_Injected(UnityEngine.Vector3&)`.

There is no FreeType branch inside this ChangeVelocity base body.
`ChangeForwardVelocity` at 0x1ebe220..0x1ebe35c is a distinct method: it applies
the same CannotMove mask gate, takes source Transform.forward X/Z times speed,
and preserves current Rigidbody Y. Its selection by state/mode remains a
separate calling-path contract.

## Safe implementation boundary and remaining evidence

A source-only raw-input magnitude and Vector2 orientation helper can preserve
the observed operation order and constants, with explicit finite/overflow
admission policy. Feed its outputs into the separately verified Stand Run/Rush
classification. Such helpers do not select active physical speed or a desktop
Shift binding.

Activating physical movement still requires caller/mode selection, effective
source transform/yaw and coordinate conversion, IFix/runtime input overrides,
Unity rotation/euler implementation where exact float equality matters,
collision/clearance/capsule scaling, vertical acceleration, jump/landing and
animation/root-motion interactions. The verified horizontal setter alone
does not establish these dependencies or active original-runtime parity.

Reproduce a slice with `native disasm ELF RVA LENGTH`; reproduce constants
with `native words ELF RVA LENGTH`. `manifest.json` records retained artifact
hashes and source identities. No whole-program native provider was opened.
