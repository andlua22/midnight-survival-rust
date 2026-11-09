# BattleMinimap initialization: native fallback review

2026-11-10. This is a static ARM64 review of the pinned APK. It recovers
initialization instructions and corresponding retained source objects. It
does not observe execution, IFix state, successful runtime binding, a final
RawImage texture or a live screenshot.

**Usable result:** source-backed marker-template visibility overrides are
listed below with qualified GO/Transform identities. The constructor's zoom
default is 15. The internal reciprocal-zoom Rect is **not a verified
RawImage UV assignment**. Texture selection remains unresolved. Do not
automatically apply 1/15 UVs, infer a final image scale, or change the
production overview extent from this report.

The [canvas and calibration review](minimap-canvas-review.md) retains the
bootstrap, MapCfg, map inventory and world-coordinate evidence. Those
investigations were not repeated here.

## Source and tool identities

- APK: export/lite4.0.apk, SHA-256
  2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df.
- Native member: lib/arm64-v8a/libil2cpp.so, 76,608,224 bytes, SHA-256
  2e2efde1a103549694b17b361cacc18faa5714d06c444945e44b30f1377e2b3c.
- Method/field names: export/SC.UI/BattleMinimap.cs, SHA-256
  00124db6b6b2041d06c9d188253b579f8ccc243ccd5408425f7192e54e2f6090.
  These exports supply tokens and native RVAs. Their damaged pseudo-C#
  bodies and overlapping external-value-type offsets are not executable
  reconstruction specifications.
- UI package:
  assets/imported_ui/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1/.
- Selected layout: panels/battle_minimap.json, SHA-256
  0239ce5e792711fd7b850bb97b58c72122456b75e8596310bf9aea46adab44de.
- Retained root GameObjectData: component PathID -5063542670030210478,
  source_objects/92d6e2bb872973a4e66cb2487cff21922983a42957d1a848101f55e8ffcb2cb4.json.
  Its hash matches that filename; its GameObjects array has 86 entries.
- Existing Rust probes in /tmp/codex-minimap-review were reused with the
  installed Capstone 5.0 library. The native probe maps ELF virtual addresses
  through PT_LOAD and checks the Capstone ABI. No Python, downloads,
  installations, Cargo builds, target execution or ADB calls were used.

Native ranges below are inclusive ARM64 ELF virtual-address ranges.

| Method | Token | Native range | Relevant IFix check |
|---|---|---|---|
| BattleMinimap.OnInitMap | 0x6012ACF | 0x18B09E4–0x18B1663 | 45844 |
| InitCompassAndMinimap | 0x6012B08 | 0x18B1664–0x18B2C6F | 45845 |
| OnShowCompassAndMinimap | 0x6012AE5 | 0x18B3358–0x18B420F | 46115 |
| Awake | 0x6012B6D | 0x18D18F0–0x18D297F | Actual patch state unobserved |
| BattleMinimap constructor | 0x6012B6E | 0x18D2980–0x18D3637 | Later initialization can change defaults |

OnInitMap calls InitCompassAndMinimap at 0x18B1038 and SetMapMask at
0x18B1040. The source visibility actions below occur in that fallback
initialization phase. Later show/update calls can change visibility.

## Texture and scale: preserve the unresolved boundary

The qualified minimap image source is:

    bundle = assets/ui/battleminimap.ab
    cab = CAB-88df926bf45d4de159fb33b326042714
    GO = -7917619469819177287
    Transform = -3798366094520507772
    RawImage = 910110631840687877
    path = BattleMinimap/GameObject/ditu/lefttop/m_lefttop

The RawImage source fields document is
source_objects/eecda970a4f05d503d92b5752036e767ab509da78209d67e9af19cfe72527742.json.
Its script is UnityEngine.UI.RawImage, UnityEngine.UI.dll. Its authored
texture and material references are null; authored UV rect is [0,0,1,1].
The node also has RectTransform, CanvasRenderer, Canvas and GraphicRaycaster
components. No additional texture-loader script was found on this selected
node.

InitCompassAndMinimap loads m_lefttop from instance +0x508 at 0x18B26A8,
gets a component using the type metadata slot 0x45EA1C0, and caches it into
_mBattleImage (+0x118) at 0x18B26C0. The acquired graphic's RectTransform
is then queried. **This trace does not establish a texture setter or a
texture name for this component.**

At 0x18B2630–0x18B26A0 the initializer gets m_lefttop's Transform and
sets localScale using twice the three-float static value at +0xC through
metadata slot 0x462E2F8. The source vector's type/value was not independently
resolved from metadata in this review. Its layout is consistent with the
ordinary Unity Vector3.one static field, but [2,2,2] is consequently an
interpretation, not a proven preview binding. Final inherited scale also
depends on the actual parent hierarchy.

The definite Map_map binding belongs to BattleMapPanel's large-map
RawImage, as recorded in the earlier report. It does not prove m_lefttop's
runtime texture. An explicitly chosen Map_map preview texture would remain
a documented preview choice pending a direct assignment or source-runtime
observation.

## Zoom, internal Rect and _mapRate

At 0x18D2EB4–0x18D2EF0 the constructor copies literal vectors from
0x3622110 and 0x3622120 into instance +0xF0 and +0x100.
The first two literal words are:

| Literal VA | Bits | Float | Corresponding scalar |
|---|---|---:|---|
| 0x3622110 | 0x41700000 | 15.0 | _mZoomLevel, +0xF0 |
| 0x3622114 | 0x461C4000 | 10000.0 | MAP_SIZE, +0xF4 |

OnInitMap later updates MAP_SIZE from MapMgr; retain the earlier report's
source-selection conditions rather than treating 10000 as immutable.

The initializer constructs an **instance-owned Rect at +0x130**:

- 0x18B26BC gets its address.
- 0x18B26C4–0x18B26E0 set its X and Y to zero.
- 0x18B26E4–0x18B26F4 read zoom and calculate the float reciprocal.
- 0x18B26FC also stores that reciprocal into floats at +0x6C/+0x70.
- 0x18B2700 sets the Rect size to that reciprocal in both directions.

For the constructor default this is approximately [0,0,0.06666667,0.06666667].
The size helper at 0x3422A68 writes two floats at Rect +8/+12.
It receives the address of the instance Rect, **not the RawImage**.
No RawImage.uvRect setter has been established here. Applying these values
as a RawImage preview crop would overstate the evidence.

_mapRate (+0x24C) is not explicitly initialized by the inspected constructor
or InitCompassAndMinimap. The constructor's writes at +0x244/+0x248 set
two integers to 1; its color-vector writes begin at +0x258 and do not
initialize +0x24C. A fresh managed object's ordinary zero initialization
therefore leaves _mapRate zero absent deserialization or other writes.

OnShowCompassAndMinimap computes it at 0x18B3730–0x18B379C:

    H = _mBattleImage.rectTransform.rect.height
    Z = _mZoomLevel
    M = MAP_SIZE
    A = H * Z
    B = A + (A * (-1 / 4096)) * 204
    _mWidthRate  = B / M
    _mHeightRate = B / M
    _mapRate = (B / A) / M

The two immediate float bit patterns in this calculation are 0xB9800000
(-1/4096) and 0x434C0000 (204). Stores are at 0x18B3794 (+0x120),
0x18B3798 (+0x24C) and 0x18B379C (+0x124).
Rect height getter 0x3422A50 loads the float at Rect +0xC.

Conditional on the retained 2048-unit rect, zoom 15 and M=10000:
A=30720, B=29190, width/height rates are approximately 2.919, and
_mapRate is approximately 0.00009501953125. These calculations are native
float operations; decimal expressions are descriptive, not bit-exact
reimplementations.

As proved in the earlier report, GetPosOnMap uses _mapRate as its zero
gate and uses _battleMapRate for the coordinate multiplication. **Do not
replace the conditional 0.4096 world-to-map factor with _mapRate.**
Production 5360 remains unchanged.

## Source-backed template visibility

Every GO and Transform PathID in the following tables is qualified by:

    bundle = assets/ui/battleminimap.ab
    cab = CAB-88df926bf45d4de159fb33b326042714

Paths are relative to the BattleMinimap root. Use the full bundle/CAB/GO
identity for a preview override. The Transform ID identifies the associated
geometry; it is not the SetActiveBetter argument.

The source association uses the retained GameObjectData index, exported
field name/offset, native indexed field binding and selected layout node.
This establishes the corresponding source objects and fallback false
arguments. **Actual successful runtime binding remains unobserved**, including
the additional-entry mismatch documented below. Each identified source GO
is an appropriate explicit initialization-preview override, not proof that
the same GO is inactive in an arbitrary gameplay frame.

### Direct InitCompassAndMinimap false arguments

SetActiveBetter is export/Extensions.cs, token 0x6003701, RVA 0x207D308.
The following transfers pass w1=0.

| Field / retained index | Relative source path | GO PathID | Transform PathID | Transfer to SetActiveBetter |
|---|---|---:|---:|---|
| m_map_line / 41 | GameObject/ditu/lefttop/m_lefttop/m_map_line_root/m_map_line | 7485614794676456502 | 3514403655517287257 | 0x18B1910 |
| m_tag_image_orange / 80 | GameObject/ditu/lefttop/m_mapShowIcon/m_tagging_player_root/m_tag_image_orange | -1797627085918124301 | -3256156571010530090 | 0x18B194C |
| m_tag_image_red / 81 | GameObject/ditu/lefttop/m_mapShowIcon/m_tagging_player_root/m_tag_image_red | -3543335029901358633 | -331953300422487428 | 0x18B195C |
| m_tag_image_box / 79 | GameObject/ditu/lefttop/m_mapShowIcon/m_tagging_player_root/m_tag_image_box | -9204725439304705299 | 6891755472039237723 | 0x18B196C |

### Sound-template dictionaries

The initializer inserts the following source objects and then iterates
the dictionary values with SetActiveBetter(value, false).
Paths below are relative to
BattleMinimap/GameObject/ditu/lefttop/m_mapShowIcon/m_mapSound_parent/.

| Field / retained index | Source node | GO PathID | Transform PathID | Insertion VA | False transfer VA |
|---|---|---:|---:|---|---|
| m_gun_sound_pos_map / 27 | m_gun_sound_pos_map | 8869356163041564258 | -9221279518444326841 | 0x18B19A8 | 0x18B1A5C |
| m_airdrop_pos_map / 5 | m_airdrop_pos_map | 6360961865288636768 | -8193319318223761873 | 0x18B19C8 | 0x18B1A5C |
| m_footSoundInMap / 21 | m_footSoundInMap | 8771581050433998562 | 8494517497541698438 | 0x18B2084 | 0x18B2198 |
| m_gunSoundInMap / 28 | m_gunSoundInMap | 5985292754087571339 | 7559449315723006120 | 0x18B20A4 | 0x18B2198 |
| m_vehicleSoundInMap / 68 | m_vehicleSoundInMap | 5159176092881812020 | -4399458412663406298 | 0x18B20C4 | 0x18B2198 |
| m_airdrop_dir_map / 3 | m_airdrop_dir_map | 8363443611025557459 | 9008430909113097640 | 0x18B20E4 | 0x18B2198 |
| m_gun_silence_sound_pos / 26 | m_gun_silence_sound_pos | 1651694541040224185 | 6915414000386161051 | 0x18B2104 | 0x18B2198 |
| m_wurenji_sound_pos / 70 | m_wurenji_sound_pos | 2736434523089675485 | -3938415719302913455 | 0x18B2124 | 0x18B2198 |

The first two values populate _mSound2IconInMap (+0x88), whose enumeration
is 0x18B1A2C–0x18B1A60. The remaining six populate
_mSound2IconInMapDir (+0x90), whose enumeration is 0x18B2168–0x18B219C.
These are template objects; pooled or newly generated sound objects and
later sound events have separate lifetimes.

### Additional direct OnInitMap false arguments

| Field / retained index | Relative source path | GO PathID | Transform PathID | Transfer to SetActiveBetter |
|---|---|---:|---:|---|
| m_badge_img / 8 | GameObject/ditu/lefttop/m_mapShowIcon/m_resource_tags/m_badge_pos_root/m_badge_img | 4458685810974747162 | 7407897960620516250 | 0x18B1050 |
| m_vehicle_icon / 67 | GameObject/ditu/lefttop/m_mapShowIcon/m_self_vehicle_parent/m_vehicle_icon | 9090670034721949667 | 3071919443331380935 | 0x18B1094 |
| m_shouHuoJi_imgObj / 56 | GameObject/ditu/lefttop/m_mapShowIcon/m_shouHuoJi_parent/m_shouHuoJi_img | 4820699717724347823 | -8387213359613818777 | 0x18B10D0 |
| m_fanmaiji_imgObj / 19 | GameObject/ditu/lefttop/m_mapShowIcon/m_fanmaiji_parent/m_fanmaiji_img | -8294374823165935752 | 2837093518875009007 | 0x18B10F8 |
| m_common_icon / 10 | GameObject/ditu/lefttop/m_mapShowIcon/m_resource_tags/m_common_image_root/m_common_icon | -7996231445121348839 | 6353688308257557609 | 0x18B11B8 |
| m_above_icon / 0 | GameObject/ditu/lefttop/m_mapShowIcon/m_above_image_root/m_above_icon | -5262363387168547893 | -8623905796289216940 | 0x18B11C8 |
| m_airDrop_Mark / 4 | GameObject/GameObject/image1/m_airDrop_Mark | 2391313024899094591 | 2291175960084559457 | 0x18B1244 |
| m_freighter_mark / 23 | GameObject/GameObject/image2/m_freighter_mark | -5063654596796734279 | -7325922068362126442 | 0x18B1254 |
| m_cardBuildingTag_img / 78 | GameObject/ditu/lefttop/m_mapShowIcon/m_cardBuilding_parent/m_cardBuildingTag_img | -7139541613775318912 | 3929743745713679514 | 0x18B165C, tail transfer; w1=0 at 0x18B1650 |

This is a source-qualified list of 21 overrides. Hiding a GO must hide its
descendants in the preview, including the Images under the sound and
UIGameObjectList templates. It should not erase the retained source records.

## Additional native binding entries are unresolved

The inspected Awake body requests entries 86 through 93, at
0x18D2850–0x18D2950, from its indexed binding list. For example:

- index 86 is stored into m_fanzhongli_img (+0x6C8) at 0x18D2864;
- index 90 is stored into m_redUserIconTemp (+0x6E8) at 0x18D28F4;
- index 92 is stored into m_cardIcon (+0x6F8) at 0x18D293C;
- index 93 is stored into the last field (+0x700) at 0x18D2960.

The selected retained GameObjectData only has indices 0–85.
Complete runtime-list construction, alternative prefabs, asset revisions and
IFix handling are not established. Do not assume a successful 94-entry
native initialization from this 86-entry retained source array, or invent
source IDs for the additional members. This discrepancy is a reason to
preserve source identity and runtime identity separately.

## Explicit preview inputs and remaining verification

The 21 GO false overrides above can define a clearly named initialization
preview profile using retained source identities. This is the concrete
addition justified by this review. A profile should preserve authored mode
as well, because these are lifecycle changes.

Record zoom=15 as a constructor default. Do not automatically apply it as
a texture crop. Leave the authored RawImage UVs and source transform scale
in place unless an explicit preview scenario changes them; the effective
runtime UV assignment and source static vector remain unresolved.

Still required for source-runtime equivalence:

- confirm the actual binding list/prefab revision and successful Awake/init
  path, including IFix;
- follow a direct m_lefttop texture assignment or observe its actual texture;
- resolve metadata slot 0x462E2F8 and observe parent/image/icon transforms;
- identify any RawImage UV setter and the effective zoom/center state;
- observe show/update visibility, map selection and calibration with pinned
  source runtime identity and multiple landmarks.

## Temporary artifacts and reproduction

New temporary files are confined to /tmp/codex-minimap-runtime/.

| Artifact | SHA-256 / purpose |
|---|---|
| init_compass_minimap.asm | 2a9aab4adb9b405880aac61619f47f1a7c76d1191a5df8aec3becb2fd1cc15ab |
| battle_minimap_range.asm | 5ca6dd6f74e3b3b998d0b4c17738163f65953ada197d376bbc9f3c05cb1c61a5 |
| visibility_nodes.jsonl | c39cb8a7d319938c9b1a82521b4962ea58555006fa64608519b9c3bc6f28c0ad; 21 qualified source-node/transform records |
| bindings.jsonl | Selected layout's GroupName/GameObjectData/UIGameObjectList binding projections |

The broad assembly artifact contains a bounded 144,212-byte native interval
with multiple methods. It is not one giant function. Capstone decoded all
36,053 instructions in that interval; the initializer artifact decoded all
1,411 instructions / 5,644 bytes. Durable identities and instruction
addresses are retained above even if temporary files are later removed.

    /tmp/codex-minimap-review/native disasm /tmp/codex-minimap-review/libil2cpp.so 0x18B1664 0x160c
    /tmp/codex-minimap-review/native disasm /tmp/codex-minimap-review/libil2cpp.so 0x18B0338 0x23354
    /tmp/codex-minimap-review/native words /tmp/codex-minimap-review/libil2cpp.so 0x3622110 0x48
    /tmp/codex-minimap-review/native disasm /tmp/codex-minimap-review/libil2cpp.so 0x3422A40 0x38

The existing Rust UI probe's nodes mode on panels/battle_minimap.json emits
compact qualified geometry records. Its bindings mode emits the ordered
retained references. These bounded layout queries do not dump source_ui.json.
No production code, imported assets, earlier evidence or other memory files
were edited by this review.
