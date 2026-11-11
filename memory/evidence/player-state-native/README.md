# Stand state classification — static native evidence

2026-11-11. This focused review reuses the retained Rust/Capstone 5.0 helper
and the selected recovered APK's AArch64 ELF. No target was executed and no
REA provider was opened. These are static local observations, not REA Evidence
IDs, observed gameplay or verified IFix runtime behavior. Earlier evidence was
preserved and no production code changed during this review.

The source APK SHA-256 is
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
The retained 76,608,224-byte `/tmp/codex-minimap-review/libil2cpp.so` hashes to
`2e2efde1a103549694b17b361cacc18faa5714d06c444945e44b30f1377e2b3c`.
Source declarations and native RVAs come from the paired recovered exports;
Cpp2IL statements and external vector layout offsets remain damaged.

## Complete base Move path

`StandState.Move` is `0x1e7b748..0x1e7b974` (exclusive end), with 139 decoded
instructions covering all 556 bytes. `stand-move.asm` retains instruction
bytes, addresses, branches and the Capstone completion count. The opening
IFix gate is patch ID 11996; this contract concerns its unpatched base path.
Null references take the observed native failure path rather than a default
classification.

The base path clears both static flags `StandRush` and `StandRun` with one
halfword store at `0x1e7b7c8`, then clears `PlayerController.FreeType` at
`0x1e7b7d4`. It reads two f32 input components at native offsets 0x3a8/0x3ac,
computes separate f32 multiplies/addition and f32 square root, promotes the
result to f64, and compares against f64 bits `0x3fed5810624dd2f2` (0.917).
The branch at `0x1e7b83c` enters fallback when magnitude is <= that threshold.
There is no input-vector clamping, normalized-magnitude replacement, hypot
operation or fused multiply-add in this sequence.

When magnitude is strictly above the f64 threshold and native `m_CanRush`
at 0x37c is true, `MathUtils.GetOrientation(input)` (`0x2120668`) is called
and its f32 result is stored in `YaoGanAngle` at 0x470. The source-only
classifier should accept that orientation result explicitly until this
helper's behavior is recovered. Other branches preserve existing YaoGanAngle.

For nonnegative finite magnitude and finite orientation the directly observed
outcomes are:

| Condition | StandRush | StandRun | FreeType | m_SpeedMultiple | Orientation write |
|---|---|---|---|---:|---|
| High magnitude, canRush, angle in [0,75) or (285,360] | true | false | true | 3 | supplied helper result |
| High magnitude, canRush, other angle | false | false | false | 2 | supplied helper result |
| Nonnegative magnitude <= threshold, or canRush false | false | true | false | 2 | none |

Zero magnitude enters fallback and **sets StandRun true**. Its comparison at
`0x1e7b90c` is against zero and `b.ge` at `0x1e7b910` reaches the true store
at `0x1e7b92c`. A source-only implementation must not invent an idle outcome
based on the flag's name. When the fallback's magnitude >= 0 test fails,
`0x1e7b914..0x1e7b91c` instead jumps directly to the callbacks at `0x1e7b940`;
it skips both the StandRun true store and the multiplier 2 store, preserving
the prior multiplier. A negative finite scalar magnitude cannot be produced by
the observed input-square-root sequence. Rejecting negative scalar magnitude
(while accepting -0), NaN/infinity or arithmetic overflow at an API admission
boundary is a reconstruction policy, not recovered invalid-input parity.

Orientation 0 is admitted, 75 and 285 are excluded, and 360 is admitted.
The latter is a native finite comparison result, not an instruction to wrap
360 to 0 or normalize arbitrary angles. The 75/360 constants are immediate
f32 bit patterns `0x42960000`/`0x43b40000`; the retained 285 constant is
`0x438e8000` at `0x3622de4`. The nearest f32 to 0.917 is below the native
f64 threshold. Preserve the source float operation order and promoted compare.

After classification the method calls `PlayGunChangeAimByFreeType`
(`0x1ec4f34`), copies current FreeType into `m_lastFrameFreeType` at 0x54c,
then tail-calls `ChangeCameraState` (`0x1e7c5e8`). Their complete native slices
are retained. The weapon callback branches on old/new FreeType and has its
own IFix gate; camera selection reads the source flags/AutoRun and aiming/drone
gates. These effects are outside the classification contract, as are selected
runtime camera presets, weapon animation and effective physical velocity.

The input vector's exported external-type offsets overlap adjacent fields
(`Input` is printed at 0x398). Native reads and source field labels therefore
do not establish a valid Rust memory layout. No binary-layout overlay is
proposed by this evidence.

## Exact state ID declarations

`state-id-declarations.txt` retains all 62 `StateID.cs` declarations and their
source line numbers. Values are contiguous 0..61, from `NullStateID` through
`ClearParking`. This includes weapon, vehicle, NPC and other source states;
the earlier posture-only subset was not the complete enum. Preserve names
such as `Pa`, `Huanzidan`, `MonsterGoBitrh` (source spelling), `Freezy` and
`FlyWuRenJi`. The declaration inventory does not prove all states are
registered in a particular player's active state machine.

`manifest.json` records source/helper identities, inspected ranges and hashes
of every retained slice. Reproduce an assembly slice with:

```sh
/tmp/codex-minimap-review/native disasm /tmp/codex-minimap-review/libil2cpp.so 0x1e7b748 0x22c
```

The helper validates ELF64 little-endian AArch64 and maps through file-backed
PT_LOAD segments; ranges are bounded to 256 KiB. It was neither rebuilt nor
modified. No Cargo, Python, GUI, services or protected-cache access was used.
