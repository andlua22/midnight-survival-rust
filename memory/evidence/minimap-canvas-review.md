# BattleMinimap canvas and calibration review

Reviewed 2026-11-09. Static bootstrap canvas settings, prefab layout, map
configuration records and native fallback calculations are verified below.
BattleMinimap's instantiated parent, runtime image selection and final world/UV
calibration have not been observed. **Do not replace production `5360.0` from
this report.** No production code or imported package was changed for this review.

## Source identities and method

- APK: `export/lite4.0.apk`, SHA-256
  `2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
- Native member: `lib/arm64-v8a/libil2cpp.so`, 76,608,224 bytes, SHA-256
  `2e2efde1a103549694b17b361cacc18faa5714d06c444945e44b30f1377e2b3c`.
- UI package:
  `assets/imported_ui/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1/`.
- Unity version in the inspected serialized files: `2021.3.23f1c1`.
- Existing `unity-asset-binary 0.2.0`, ZIP and JSON Rust libraries were used.
  Native instructions were decoded through the installed Capstone 5.0 library
  by a temporary Rust program. No Python, downloads or tool installation.
- Bootstrap type trees are stripped. Their parser output is explicitly raw
  bytes, not a successful field projection. The interpretations below use
  little-endian Unity 2021 field layouts, exact object boundaries and resolved
  MonoScript identities. Prefab fields come from the retained source projections.
- Native RVAs below are ELF virtual addresses, mapped through `PT_LOAD`, not
  assumed file offsets. Native method names/RVAs come from the C# exports;
  the damaged pseudo-C# bodies were not translated into implementation.

## Bootstrap root canvases: source values recovered

`level0` and `level1` each have a layer-5 GameObject named **`UIRootCanvas`**.
This resolves the earlier lack of CanvasScaler evidence in the three selected
UI prefab bundles. It does not prove which root is the live minimap ancestor.

All IDs in this table are local signed PathIDs, with file ID `0`:

| APK member | Root GameObject | RectTransform | Canvas | CanvasScaler | AdjustResolution | Canvas camera |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `assets/bin/Data/level0` | 1 | 12 | 11 | 17 | 15 | 6 (`UICamera`) |
| `assets/bin/Data/level1` | 2 | 15 | 14 | 19 | 22 | 8 (`UICamera`) |

Root component order is `[12,11,17,16,15,14]` in level0 and
`[15,14,19,18,21,20,22]` in level1. Both root RectTransforms have a null parent,
zero authored local scale and zero rect fields; these are driven root values,
not evidence of a zero reference resolution. Their children are the event
system/camera transforms, plus the FPS RectTransform in level1.

Both 55-byte Canvas records contain:

| Object-relative offset | Field | Value |
| --- | --- | --- |
| 0 / 4 | GameObject PPtr file ID / PathID | `0` / root GO above |
| 12 | enabled, u8 | `1` |
| 16 | render mode, i32 | `1`, Screen Space Camera |
| 20 / 24 | camera PPtr file ID / PathID | `0` / camera above |
| 32 | plane distance, f32 | `100.0` |
| 36–39 | pixel perfect / receives events / override sorting / override pixel perfect | `0 / 1 / 0 / 0` |
| 40 | sorting bucket normalized size, f32 | `0.0` |
| 44 | additional shader channels, i32 | `25` |
| 48 | sorting layer ID, i32 | `0` |
| 52 / 54 | sorting order, i16 / target display, u8 | `10 / 0` |

The two CanvasScaler MonoBehaviours point to file ID `1`, PathID `483`.
External file `1` is `globalgamemanagers.assets` in both scenes. Joining the
seven APK members `.split0` through `.split6` resolves PathID `483` as native
class `115`, MonoScript `CanvasScaler`, namespace `UnityEngine.UI`, assembly
`UnityEngine.UI.dll`. This is a script identity proof, not a size-based guess.

Both 80-byte CanvasScaler records contain:

| Object-relative offset | Field | Value |
| --- | --- | --- |
| 12 | enabled, u8 | `1` |
| 16 / 20 | script PPtr file ID / PathID | `1 / 483` |
| 28 | empty name length, i32 | `0` |
| 32 | UI scale mode, i32 | `1`, Scale With Screen Size |
| 36 | reference pixels per unit, f32 | `100.0` |
| 40 | serialized scale factor, f32 | `1.0` |
| 44 / 48 | reference resolution, two f32 | **`1920.0 / 1080.0`** |
| 52 | screen match mode, i32 | `0`, Match Width Or Height |
| 56 | serialized match width/height, f32 | `0.0`, width |
| 60 | physical unit, i32 | `3`, Points |
| 64 / 68 | fallback screen DPI / default sprite DPI, f32 | `96.0 / 96.0` |
| 72 | dynamic pixels per unit, f32 | `1.0` |
| 76 | preset is world, u8 | `0`; padding 77–79 is zero |

Exact source boundaries and hashes follow. Byte starts are absolute offsets in
the named serialized file; sizes exclude alignment before the following object.

| File | Class / PathID | Byte start | Size | SHA-256 of object bytes |
| --- | --- | ---: | ---: | --- |
| level0 | GameObject / 1 | 4096 | 99 | `6776d851b84a0aa53fa0c2068e0e0a38b94cce6559d8db7388b5ff58cec68572` |
| level0 | Canvas / 11 | 5072 | 55 | `59da351b9674aa96ae6ebe419c46e66795c9d4ab574355f9fc4e4c8d81c1b88a` |
| level0 | RectTransform / 12 | 5128 | 132 | `50ce2b5d52228e8c2cf1f280eed0b9fcd90be24c2df45265d5d3bbe314dc6f50` |
| level0 | AdjustResolution / 15 | 5352 | 32 | `4d4cce87ca09f295c64b4a6ff485b3ac44bd28ab52c54190ec0a5cd9026fa344` |
| level0 | CanvasScaler / 17 | 5432 | 80 | `38e39b78665be78628bf1b3dd3334f7c018d1b54fac16a4c71a2be8dfd6f53f1` |
| level1 | GameObject / 2 | 4128 | 111 | `5953e10f25309e32fce21e472d169b20cbd5296a7c7b31930092b33c9c8b61f1` |
| level1 | Canvas / 14 | 5200 | 55 | `dd0388ac8cf67497162949622778ccb699596d5738f6b0c91d97a122406fb29f` |
| level1 | RectTransform / 15 | 5256 | 144 | `cb9c49693cc14a64010a212149ccc6760e556d6d21d11bfe92a54676108cab08` |
| level1 | CanvasScaler / 19 | 5616 | 80 | `a31c3b7afeda17d0a096dafa074e42b7b2316042e3a5165d12dff2e75cde0cea` |
| level1 | AdjustResolution / 22 | 5768 | 32 | `832429b0085ce3529a5abfe4faf442e8d4e9378107e3e37c66ca726719efc581` |
| globalgamemanagers.assets | MonoScript / 483 | 236544 | 96 | `25400ae838e92b62ec04d78369db3948b64e10c0ccab76d7d2e401508e378ef7` |
| globalgamemanagers.assets | MonoScript / 4105 | 624296 | 88 | `6ba831a40130520b5a9cc684036dfd7f701834a171e8036466c0a611bd4bbd67` |

Whole-file identities: level0 is 5732 bytes, SHA
`74f3f0f4a384ba393418bca24c461f19b610bff4aab36a05563f879fd303ae1b`;
level1 is 6124 bytes, SHA
`77ca83aa15a1cfe9de2a6b3741620c83c2270c6a6e80ff3b5c2e4bdf76c33de2`;
joined globalgamemanagers.assets is 6,297,320 bytes, SHA
`85711611bda972881ddb20c15b441b8fc7c04e57cc5233333675c7de29aa9754`.
Level2 has five objects and no Canvas/GameObject/MonoBehaviour; its 4600-byte
member SHA is `094dfa19ed91369d48ac1d3854bca35d568a8d32ebf5d0f5c8927b641343e34b`.

## AdjustResolution changes the serialized match setting

Both 32-byte AdjustResolution components are enabled and resolve script PPtr
`(1,4105)` to `AdjustResolution` in `Assembly-CSharp.dll` with an empty
namespace. See `export/AdjustResolution.cs`: Awake RVA **`0x20FAC64`**,
token `0x6003C63`. Its native fallback body occupies 236 bytes through
`0x20FAD4F` and does the following for ordinary positive screen dimensions:

1. Calls `Screen.get_width()` at `0x3417128`, whose icall string is
   `UnityEngine.Screen::get_width()` at `0x370312c`.
2. Calls `Screen.get_height()` at `0x3417150`, whose icall string is
   `UnityEngine.Screen::get_height()` at `0x370314c`.
3. Compares `width / height` against f32 `1.7777778`, bits `0x3fe38e39`,
   stored at `0x36236c0`.
4. Gets the CanvasScaler component on its own GameObject and writes the
   runtime float field at `+0x30`: `1.0` for aspect >= that threshold,
   otherwise `0.0`. The writes are at `0x20FAD34` and `0x20FAD3C`.

Thus the source bootstrap behavior selects height matching on wide screens
and width matching on narrower screens. With reference `[1920,1080]`, its
ordinary scale is `min(width / 1920, height / 1080)`. The serialized `1.0`
scale factor alone is not the final screen scale. `[1920,1080]` gives scale
`1`; `[1280,720]` gives `2/3`. These are useful explicit preview cases.
The f32 threshold and native branch should be preserved if implementing the
source behavior rather than replacing it with a different aspect policy.

Awake consults IFix patch ID `16400` first. Actual patch state and later
changes remain unobserved. This source behavior is verified independently of
the incomplete exported Awake body.

## BattleMinimap prefab layout and exact bindings

Bundle `assets/ui/battleminimap.ab`, CAB
`CAB-88df926bf45d4de159fb33b326042714`, panel
`panels/battle_minimap.json`: 225 nodes, four Canvas components, **zero
CanvasScaler components**, one RawImage. IDs below belong to this bundle/CAB.

| Node | GameObject PathID | RectTransform PathID | Source rect |
| --- | --- | --- | --- |
| BattleMinimap | `1749944185578701977` | `-3069557478462998359` | anchors min=max `[1,1]`, pivot `[1,1]`, position `[10,-32.399994]`, size `[330,330]` |
| first GameObject child | `6295463798849069898` | `-4656830139714386032` | centered, position `[0,0]`, size `[100,100]` |
| first GameObject/ditu | `-4394325854588574010` | `5845033069953767145` | centered, position `[0,0]`, size `[260,260]` |
| …/ditu/lefttop | `-6742757713922008959` | `-8394090200930732781` | centered, position `[0,0]`, size `[272.1,281]` |
| …/lefttop/m_lefttop | `-7917619469819177287` | `-3798366094520507772` | centered, position `[0,0]`, size `[2048,2048]` |

These transforms have source local scale `[1,1,1]`; the prefab root has a
null serialized parent `(fileID 0, PathID 0)`. Do not silently negate the root's
positive X offset or change its pivot to center. Duplicate `GameObject` names
are distinguished by source IDs and ordered children.

The enabled RawImage on `m_lefttop` has component PathID
**`910110631840687877`**, source field document
`source_objects/eecda970a4f05d503d92b5752036e767ab509da78209d67e9af19cfe72527742.json`.
Its texture and material PPtrs are both null, UV rect `[0,0,1,1]`, graphic
color white, maskable true, raycast target true. **There is no serialized
minimap background texture binding to follow from this component.**

The four prefab Canvas PathIDs are:

| Node | Canvas PathID | Retained field JSON hash |
| --- | --- | --- |
| …/lefttop/m_lefttop | `992407269149794703` | `558e552904d7066d8e1a392e6879bb35c47546bb25c8f7eee02619ba7191d772` |
| …/lefttop/m_mapShowIcon | `9202849199775789067` | `b3092933cde14401209184db9398a42b5c480e9759fdcc82894d1ab152ae731e` |
| …/m_mapShowIcon/m_team_members | `1269670219451764558` | `63660ae3df2e6cca030f77a6f9de910fba3ede740b12a3c7dae576b5b20a857e` |
| BattleMinimap/zhou_ye | `8119633726226858880` | `979dda2fafc234201aae7e8584794854059cfcb9d22c700cdb78259b6e42f8fe` |

All four serialize render mode `2` (World Space), camera null, plane distance
`100`, pixel perfect false, override pixel perfect true, override sorting
false, sorting order/layer `0`, receives events true. These authored nested
canvas fields do not establish the effective mode or scale after runtime
parenting under a root canvas.

The root's `GroupName` component PathID `-5266315798943346194` explicitly stores
`groupName = "BattleMinimap"` in source document hash
`8990e67ad76aa5952b987763306eaaa89d78574ae57a04705249db195d9d2202`.
`GameObjectData` component `-5063542670030210478` retains 86 ordered GO PPtrs
in source document hash
`92d6e2bb872973a4e66cb2487cff21922983a42957d1a848101f55e8ffcb2cb4`.
Useful exact bindings: index `37` is `m_lefttop`, index `43` is
`m_mapShowIcon` (`-3618866192601094609`), index `58` is `m_team_members`
(`7234045134679076829`), index `85` is `m_StencilMask`
(`-4451105165023622346`). Field names and the binding order are source facts;
the component list itself is not the runtime script's complete behavior.

`ditu`'s Unity UI Mask is enabled, `m_ShowMaskGraphic = 0`, component
`1455664145696737228`, source field document hash
`ece68cc0fdca0b3277029a1be17a678428094efd6a8d75ab927d7c9e1564e6bd`.
The separate StencilMask and its Light/Fog references remain retained; they
must not be treated as interchangeable with that Unity UI Mask.

## MapCfg records recovered from the APK

Member **`assets/cfg/cfg.MapCfg.oc`**, 1465 bytes, SHA-256
`c77f8b15e3d46ec980333b7ef8f5ae1c2246824f56e4770f18888efa5de68896`.
It is a concatenation of six records, with little-endian i32 values,
i32-length UTF-8 strings and one-byte booleans. The Rust decoder rejected
invalid lengths/UTF-8/boolean bytes and consumed **1465/1465 bytes**.
Record field types/order are corroborated by declarations and the `pop_*`
sequence in `export/cfg/MapCfg.cs`, Unmarshal RVA `0x2247468`.

| ID | Byte range, end exclusive | length | mapSize | offset | sceneSize | scene rows × cols | mdituSceneName |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| 1 | 0–240 | 10000 | 10000 | 5000 | 400 | 25 × 25 | `mditu` |
| 2 | 240–497 | 7168 | 8000 | 0 | 512 | 14 × 14 | `Mditu_I` |
| 3 | 497–735 | 4096 | 4000 | 0 | 512 | 8 × 8 | `mditu_jidi` |
| 4 | 735–975 | 3072 | 3600 | 0 | 512 | 6 × 6 | `Mditu_Moba` |
| 5 | 975–1237 | 10240 | 10240 | 5120 | 512 | 20 × 20 | `Mditu_Island` |
| 6 | 1237–1465 | 512 | 512 | 256 | 512 | 20 × 20 | empty |

Map 1 explicitly selects `gameconfig/allscenelist.obj` and
`gameconfig/scenecell.obj`, with `battleAbName = "battle_map1"`.
Its map-selection thumbnail field is `texture/warchoiceconfirmpanel_1`;
**this field is not the minimap background binding**.

The source `assets/gameconfig/allscenelist.obj` has 243 distinct scene names.
All **241** distinct scene bundles represented by the canonical terrain
manifest match this list, case-insensitively; there are zero imported bundles
outside it. The two listed but unrepresented names are `z-01` and
`zhujiemian`. This directly associates the retained terrain inventory with
the map-1 scene list. It does not observe a live `CurMapID` or prove what
other two scenes contain. The native source mesh grid also retains 400-unit
cells, consistent with this record.

## Native map calculations: verified fallback data flow

See `export/SC.UI/BattleMinimap.cs` and `export/MapMgr.cs` for method RVAs.
Opaque external types cause overlapping/incorrect-looking field offsets in
some pseudo-C# declarations. The instructions below, rather than those damaged
vector/rect declarations, establish the scalar calculations.

- `MapMgr` static constructor RVA **`0x1DFD83C`** uses metadata slot
  `0x45d9c60`. At `0x1DFD874`–`0x1DFD87C` it copies the two literal i32 words
  at `0x3622938` into static fields `+0` and `+4`: **MapSize = 10000,
  offset = 0**. This is initialization, not a proof that no runtime write occurs.
- `BattleMinimap.OnInitMap`, RVA **`0x18B09E4`**, uses that same metadata
  slot, reads its first static integer, converts it to float and stores
  instance `+0xF4` at `0x18B0BDC` (`MAP_SIZE`). The denominator is therefore
  **MapMgr.MapSize**, not an inferred texture dimension or `ConstsBs.MAP_SIZE`.
- `0x18B0C60` calls `BattleBaseMgr.get_CurMapID`; `0x18B0C68` calls
  `MapCfg.Get`. Instructions `0x18B0C70`–`0x18B0C94` read config fields `+0x20`
  (`length`) and `+0x34` (`offset`) and compute:
  `offsetForBase = trunc_toward_zero(length / 2) - offset`.
- `0x18B0C98`–`0x18B0CB8` compute
  **`battleMapRate = 2048.0 / (0.5 * MapMgr.MapSize)`**, stored at instance
  `+0x2A0`. The immediate float bits are `0x45000000` = `2048.0`.
- `GetPosOnMap`, RVA **`0x18BE7F4`**, native fallback
  `0x18BE840`–`0x18BE870`: if instance float `+0x24C` (`_mapRate`) is zero,
  returns zero. Otherwise returns local XY
  **`[(UnityX - offsetForBase) * battleMapRate,
  (UnityZ - offsetForBase) * battleMapRate]`**, with output Z zero.
- `UpdateMapCenter`, RVA **`0x18B4F68`**, fallback
  `0x18B4FA0`–`0x18B4FC8`, computes the negative of that scaled center
  position and applies it to both the map image and icon-parent transforms.
  Treat image movement and marker movement as one calibration.
- In `InitCompassAndMinimap`, RVA **`0x18B1664`**, instructions
  `0x18B268C`–`0x18B26A0` double a shared three-component scale before setting
  `m_lefttop`'s transform scale. Its final visible image size is not established
  by the prefab's 2048-unit rect alone. At `0x18B26E4`–`0x18B2700`, the
  fallback also creates a UV extent of `1 / zoom` in both directions.

For map 1 with the initialized MapMgr value, the verified formulas yield
`offsetForBase = 0` and `battleMapRate = 0.4096`. A `5000` half-extent is a
source-based map-1 candidate, conditional on the actual image, scale and
MapSize selection. It is **not** a verified replacement for the current
overview projection constant. In Bevy's already established coordinate frame,
Unity Z in these equations is `-BevyZ`; do not apply another map mirror.

These methods check IFix first (OnInitMap patch `45844`, GetPosOnMap `4848`).
Their fallback data flow is verified; actual patch/zoom/player-center states
are unobserved. Large-map inverse methods are identifiable at `0x1893930`
and `0x1893A9C`, but no complete live world/UV inversion is claimed here.

## Source map image: one definite static binding

`BattleMapPanel/GameObject/m_map_scroll/m_Map` has:

- GameObject `4479137119172950043`, RectTransform `-477783636111715980`,
  RawImage component **`8248724791771182420`** in
  `assets/ui/battlemappanel.ab`, CAB
  `CAB-c1a7bc1dffdbd8cbd784a4bc95173eab`.
- Centered authored size `[8640,8640]`, local scale `[1,1,1]`, UV
  `[0,0,1,1]`; these are UI units, not world meters.
- Texture PPtr `(fileID 12, PathID -945003921129902140)`, resolved to
  `assets/texture/map_map.ab`, CAB
  `CAB-6f665c88cf2187ff08df209ad5c96ca6`, Texture2D **`Map_map`**.
- Source field document hash
  `e7cc8ccbb798d2444c696b457415d6566e6982dc0de7bd3896e0d51125625131`;
  serialized component SHA
  `d76f66d0c883cefaad32f59aad10451dc6d9b1ec4f8462b147bad195d47e5378`.

The retained texture is 2048 × 2048, one mip, Unity format 50, color space 1,
bilinear sampling, clamp on all axes. Its package PNG is
`textures/0f815605f23e7d5b5f5ea5704a7b02432f029dbab43090465a00864be94b54e7.png`,
SHA `458b84c2041aa49138e99c9ce1a25973f873012929fee5650c117e18c67cee54`.
This binds the authored large-map image. BattleMinimap's null texture still
requires its runtime binding/selection to be recovered or observed.

## Remaining verification and temporary probe paths

Before calling this runtime-equivalent, observe the instantiated minimap's
full ancestor chain, effective root Canvas/CanvasScaler settings after Awake
and IFix, `CurMapID`, MapMgr.MapSize, actual RawImage texture, image/marker
parent scales, zoom, UV rect and world center. Compare at least two retained
world landmarks against a source screenshot to confirm origin, extent and
orientation. Bootstrap settings can support explicit preview inputs now;
they should not silently turn an unknown live ancestor into a verified one.

Temporary Rust sources and artifacts are in `/tmp/codex-minimap-review/`:

- `src/main.rs`, `Cargo.toml`, `target/debug/codex-minimap-review`:
  selective JSON, strict MapCfg, scene-list comparison, split serialized-file
  inspection, and bounded bootstrap field interpretation.
- `native.rs`, `native`: Rust/Capstone inspection with checked Capstone 5 ABI,
  ELF address mapping and bounded disassembly; `libil2cpp.so`, `mapcfg.oc`.
- `bootstrap.jsonl`, `bootstrap_scripts.jsonl`, `allscenelist.json`,
  `on_init_map.asm`, `minimap_ctor.asm`, `mapmgr_init.asm`, `mapmgr_enter.asm`.
  `on_init_map.asm` covers `0x18B09E4`–`0x18B330F` and includes additional
  methods after OnInitMap ends at `0x18B1663`; it is not one giant method.

Useful commands while those temporary probes remain available:

```sh
/tmp/codex-minimap-review/target/debug/codex-minimap-review bootstrap-proof /tmp/codex-minimap-review/bootstrap.jsonl
/tmp/codex-minimap-review/target/debug/codex-minimap-review mapcfg /tmp/codex-minimap-review/mapcfg.oc
/tmp/codex-minimap-review/native disasm /tmp/codex-minimap-review/libil2cpp.so 0x20FAC64 0xec
/tmp/codex-minimap-review/native disasm /tmp/codex-minimap-review/libil2cpp.so 0x18BE7F4 0x90
```

Temporary artifacts are convenience files; the pinned APK, object identities,
hashes, byte offsets and native addresses above are the durable evidence.
