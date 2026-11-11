# Inventory native getType observations

Read-only audit, 2026-11-11. Selected methods were disassembled offline with the existing Rust/Capstone helper. No target execution, REA session changes, Python, GUI, service changes, or production edits were used.

- ELF: `/tmp/codex-minimap-review/libil2cpp.so`
- ELF SHA-256: `2e2efde1a103549694b17b361cacc18faa5714d06c444945e44b30f1377e2b3c`
- ELF BuildID reported by file: `72e81cfbbeafb92cd6a300cd198efb468761f0b0`
- Helper: `/tmp/codex-minimap-review/native`
- Helper SHA-256: `000d6b773e4f8ed574b62b1803a1d4d1314bd0178262359cb3a176a5ac7fd8e9`
- Helper source: `/tmp/codex-minimap-review/native.rs`; inspected before execution.
- Disassembler: Capstone 5.0, ARM64, little-endian; each method range decoded 19 instructions / 76 bytes.
- Command pattern: `native disasm /tmp/codex-minimap-review/libil2cpp.so ADDRESS 0x4c`
- Method addresses follow the corresponding `export/gs.bag.scmsg/NAME.cs` getType RVA. The 0x4c-byte range ends immediately before the next marshal method. It includes the default ret and the terminal error-call block, without entering the next method.

Observed unpatched return paths load `w0` with the lower 16 bits, then execute `movk w0, #0x390, lsl #16`, restore the stack and return. Values below are thus native base-method protocol IDs, not metadata Tokens or IFix patch IDs.

| Bean | getType RVA | ID hex | ID decimal |
|---|---|---|---:|
| SItems | `0x10D909C` | `0x03900c3c` | 59771964 |
| SItemChanged | `0x10D7E98` | `0x03900bdb` | 59771867 |
| SItemNumberChange | `0x10D8A64` | `0x03900bc0` | 59771840 |
| SItemDurationChanged | `0x10D8580` | `0x03900bc2` | 59771842 |
| SQuickUseItemChanged | `0x10E1828` | `0x03900be0` | 59771872 |
| SSetQuickUseItem | `0x10E790C` | `0x03900c0a` | 59771914 |
| SBagSizeChange | `0x10D13B4` | `0x03900be6` | 59771878 |
| SSyncBagItem | `0x10E9998` | `0x03900bc4` | 59771844 |
| SClearAllBag | `0x10D2DF4` | `0x03900bee` | 59771886 |
| STimeoutItems | `0x10EA914` | `0x03900bf7` | 59771895 |
| SPutEquip | `0x10DEF94` | `0x03900c12` | 59771922 |
| SRemoveEquip | `0x10E1F60` | `0x03900bb9` | 59771833 |
| SSafeItems | `0x10E608C` | `0x03900bef` | 59771887 |
| SUnlockSafeSlot | `0x10EB160` | `0x03900c27` | 59771943 |
| SPutToHand | `0x10E1334` | `0x03900bf4` | 59771892 |

Every method first checks IFix.IsPatched and can tail-call a patch wrapper. The native observation establishes the base-method result; runtime IFix state and override results remain unknown. The five core methods retain selected exact branch/return instructions in `core-return-paths.asm`; the ten remaining files retain the complete bounded helper output. No identical disassembly was repeated merely to persist evidence.

SItems default unmarshal masks are corroborated by source only: `export/gs.bag.scmsg/SItems.cs` lines 921 and 952: bagItems count/nested records code 123; 1018 and 1047: skinItems code 14; 1105, 1119 and 1135: quickUseItems count/key/nested record code 7; 1165 and 1190: equipItems code 63; 1245: showEquip bool code 81; 1247: capacity int code 106. Its code-aware overload applies the caller-supplied code throughout. This audit did not request full native snapshot decoding.

