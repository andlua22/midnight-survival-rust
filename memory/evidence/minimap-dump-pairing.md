# Supplied memory recovery and installed counterpart: minimap pairing

2026-11-10. Bounded static comparison of the seven native ranges already
traced in the [canvas review](minimap-canvas-review.md) and
[initialization review](minimap-runtime-review.md).

**Result:** three requested method ranges are byte-identical; four differ.
The constructor's final 1,592 bytes also match. The supplied APK and
installed counterpart retain distinct identities. The user's clarification
attests that the supplied APK contains recovered memory artifacts from the
original game with encrypted native/metadata files.

This report does not observe the recovery process, decryption, loaded
process memory, IFix state, method execution or live UI behavior.

## Correct recovery direction and pinned identities

The clarified direction is:

- **Supplied recovered source:** export/lite4.0.apk, APK SHA-256
  2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df.
  The user attests that its libil2cpp.so and global-metadata.dat were
  recovered from memory because the original game's files are encrypted.
- **Installed counterpart:** preserved at
  .local/apk-runtime-candidates/waydroid-527ae0a1.apk, APK SHA-256
  527ae0a10681fd0dfdbe957b65fee79729cc665d0e7afcce43f25054366fc37b.
  Do not label this member set as the decrypted source.

The earlier reports' source/original native member refers to the member
inside the supplied 2e480f... APK. It should now be understood in light of
this recovery clarification. Those source-specific addresses and hashes
remain valid.

Root's [complete ZIP payload comparison](apk-runtime-member-comparison.json),
SHA-256 as read
37a5c5c0f93ee7ddb59eaaf8dcde85266bed764789dfde14d9d5bde9f6b6ac38,
reports 9,318 of 9,320 regular file payloads identical. Only the two members
below differ. That comparison covers all map, UI and Unity asset payloads;
it excludes ZIP layout/compression, directory entries and APK signing
blocks. This review reused that inventory instead of repeating it.

| Member | Supplied recovered source | Installed counterpart |
|---|---|---|
| lib/arm64-v8a/libil2cpp.so | 76,608,224 bytes; 2e2efde1a103549694b17b361cacc18faa5714d06c444945e44b30f1377e2b3c | 76,609,760 bytes; ed3ec656863b106bca6bd62dc2a11283b47c2c3c3c1634e4070fd386447f8c68 |
| assets/bin/Data/Managed/Metadata/global-metadata.dat | 16,788,272 bytes; 548b4e3e990641716e6e368de780fbf8c82f9806c0238d01e3bc02d0d644f2e6 | 16,788,272 bytes; 987af3ecbdd8466f5951cd1cf62f68a0a7d4ebe6a445872d5ede740ba6d5b566 |

Both newly extracted counterpart members and the reused source ELF were
hashed independently and match those inventory identities. The source
metadata was also extracted and independently checked.

## ELF mapping and comparison method

Both native files are ELF64, little-endian, AArch64, ET_DYN, with entry point
0xA4EFC0 and 11 program headers beginning at file offset 64.
Their PT_LOAD mappings are identical. Every requested range is wholly
file-backed in the first executable segment:

    p_offset = 0
    p_vaddr = 0
    p_filesz = p_memsz = 0x4414BE0
    flags = R E
    alignment = 0x1000

For these ranges, the ELF mapping therefore yields file offset equal to
virtual address. This equality was checked from both program-header
tables; it was not inferred from the C# export's Offset strings.
Actual loaded addresses would additionally depend on a process load base.

The supplied ELF has section-header offset/count/string-table-index
0/0/0. The counterpart has section-header offset 76,608,224, count 24 and
string-table index 23. The 24 entries of 64 bytes account for its additional
1,536 trailing bytes. This header/table observation does not imply that
the preceding native payload is identical.

The already existing source ELF at
/tmp/codex-minimap-review/libil2cpp.so was reused. The two counterpart
members were selectively extracted with installed unzip. Installed
readelf and dd provided the checked mappings and exact slices; sha256sum
hashed all slices, cmp compared their bytes, and an installed text tool
summarized differing byte positions. The existing Rust JSON probe read
bounded fields of the ZIP comparison report.

All 14 slices have the expected lengths: seven methods and 20,488 compared
bytes per ELF. No Python, downloads, installations, Cargo builds, ADB,
target execution or new reverse-engineering provider sessions were used.

## Requested method ranges

Ranges are inclusive ELF virtual addresses. A differing 4-byte slot means
different bytes in an aligned ARM64-sized slot; it does not assert that the
counterpart's on-disk bytes decode to a valid plaintext instruction.

| Method | Range | Bytes | Result | Differing bytes | Differing 4-byte slots |
|---|---|---:|---|---:|---:|
| BattleMinimap.OnInitMap | 0x18B09E4–0x18B1663 | 3200 | Different | 3169 | 800 |
| InitCompassAndMinimap | 0x18B1664–0x18B2C6F | 5644 | Different | 5598 | 1411 |
| OnShowCompassAndMinimap | 0x18B3358–0x18B420F | 3768 | Identical | 0 | 0 |
| BattleMinimap.Awake | 0x18D18F0–0x18D297F | 4240 | Different | 4240 | 1060 |
| BattleMinimap constructor | 0x18D2980–0x18D3637 | 3256 | Different | 1664 | 416 |
| GetPosOnMap | 0x18BE7F4–0x18BE883 | 144 | Identical | 0 | 0 |
| AdjustResolution.Awake | 0x20FAC64–0x20FAD4F | 236 | Identical | 0 | 0 |

For OnInitMap and InitCompassAndMinimap, the first and last bytes differ
and every aligned 4-byte slot contains at least one difference. All 4,240
Awake bytes differ. These are dense differences across those requested
ranges, rather than a proven small isolated entry-point replacement.

The constructor has a precise split:

- 0x18D2980–0x18D2FFF: all 1,664 bytes differ;
- 0x18D3000–0x18D3637: all 1,592 bytes match.

Total differing bytes across the requested ranges: 14,671. This total
describes these selected slices, not the whole native library.

### Raw slice SHA-256

For rows marked identical, the counterpart has the same listed digest.

| Method | Supplied recovered source digest | Installed counterpart digest |
|---|---|---|
| OnInitMap | a5039f84dac39d8a11dbc2b321701a695924a21376d07da5d54f665b790770eb | b4a7e7c7221a56abcb3ffb306689232bb698dd0199c337899d86a09f4832b7b5 |
| InitCompassAndMinimap | c9ade1d25360b06f2a61c87faf1cb8bfb2177b18add9171bca000b4b3c93dbd9 | 25238c7464720a1eb749b55a86729bd3e8f93e8421b3104e2f62b196487d32d6 |
| OnShowCompassAndMinimap | 21d8f1fd8b3ae841ec28dbff281d79d72136f76d6186691d9760ed6851e77785 | Identical digest |
| Awake | 32a477b9bcac9bdb61d4fbf114fcca9c3a87feb133372508df81d0fda2d72b33 | 18aee4fc5bae4f02d6f5cf787a093e6e62df5ad6911abd22250e30b89950367b |
| Constructor | 3bc94bb72dfd3eabd005da7f01a437b7ff98266914263c6d82c217f108c8de82 | b9ea2d83594ea5c854162d5d38a60727db410a4cb9d9155e361bc556cf9e6224 |
| GetPosOnMap | 246dbfcafed01f053f46da69277022794141c413762454f8bc1eb1c960693a5b | Identical digest |
| AdjustResolution.Awake | e8612461b85d079c5148fd253503fa709e6fb9989efe195bd4d0d7e53d81fd3f | Identical digest |

## Bounded metadata header observations

Only the first 64 bytes were interpreted as little-endian words.

| Offset | Supplied recovered source | Installed counterpart |
|---|---|---|
| 0 | 0xFAB11BAF | 0x241A8351 |
| 4 | 0x0000001D, decimal 29 | 0x33367684 |
| 8 | 0x00000100 | 0x65ABFFBC |
| 12 | 0x0001B688 | 0xBE96D725 |

The supplied member begins with the ordinary IL2CPP sanity word and
metadata version 29. The counterpart does not contain that ordinary
header at offset zero. These observations fit the clarified direction of
a supplied recovery and an installed counterpart retaining transformed
original files. The header observations alone do not verify a decryption
algorithm, the capture provenance or complete metadata-table validity.
No full type or method-registration resolution was attempted.

## What the attested pairing supports

The user's recovery attestation, the complete matching map/UI/Unity
payload inventory, the identical ELF mappings and the three identical
method ranges provide concrete support for pairing the supplied recovered
artifacts with the installed counterpart. Preserve both hashes and this
attested direction instead of treating file identity as interchangeable.

The retained map and UI asset bytes can be reused for this counterpart
without a second imported asset variant. The three identical ranges
support the presence of the same instruction bytes at the same file-backed
virtual addresses. They do not prove that a process invoked those methods,
used the fallback instead of IFix, or held the source-derived scalar values.

For the four differing ranges, the earlier analyses remain facts about
the supplied recovered code, with the user's stated original-game relation.
This comparison does not independently prove the transformation from the
counterpart's on-disk bytes to those plaintext recovered methods.
Their current live-memory equality would require observation of the
recovery/runtime path or a pinned process-memory comparison.

The 21 source-qualified template hides remain justified as an explicit
recovered-source initialization preview. This report does not certify that
the installed process executed every hide. The actual minimap texture,
effective RawImage UVs, inherited image/marker scale, live zoom/center,
IFix state and successful binding still require their own evidence.

In particular, byte pairing does not settle the earlier 94 native indexed
binding requests versus 86 retained GameObjectData entries. It also does
not justify automatically applying the internal reciprocal-zoom Rect as
RawImage UVs, resolving the static vector by assumption, or changing
production 5360.

## Temporary artifacts and reproduction

New files are confined to /tmp/codex-memory-dump-review/:

- candidate/libil2cpp.so and candidate/global-metadata.dat;
- source/global-metadata.dat, with the source ELF reused from its earlier
  temporary location;
- source/*-method.bin and candidate/*-method.bin, the 14 raw mapped slices;
- four *-byte-differences.txt files containing complete cmp byte positions
  for the differing methods.

Example exact extraction after checking both ELF program-header tables:

    dd if=/tmp/codex-minimap-review/libil2cpp.so of=/tmp/codex-memory-dump-review/source/on-init-map-method.bin bs=65536 skip=25889252 count=3200 iflag=skip_bytes,count_bytes status=none
    dd if=/tmp/codex-memory-dump-review/candidate/libil2cpp.so of=/tmp/codex-memory-dump-review/candidate/on-init-map-method.bin bs=65536 skip=25889252 count=3200 iflag=skip_bytes,count_bytes status=none
    cmp -l /tmp/codex-memory-dump-review/source/on-init-map-method.bin /tmp/codex-memory-dump-review/candidate/on-init-map-method.bin

The decimal skip above is 0x18B09E4. Other start addresses and lengths are
in the comparison table. cmp exit code 0 means identical, 1 means byte
differences, and other codes indicate an error.

No earlier report, main handoff document, imported asset or production code
was modified by this review.
