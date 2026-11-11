# MapObject source audit — 2026-11-11

This bounded, read-only audit supports the next Linux/Bevy reconstruction
milestone. It does not establish original-game execution or 1:1 runtime
equivalence. No Python, Android execution, installations, whole-APK dump,
Cargo builds, UI changes or importer edits were used in this audit.

**Later verified update:** Z-01 terrain has since been added to the canonical
package, which now contains 242 scenes and 400 render records. The independent
[omitted-scene review](evidence/omitted-scenes-review.md) establishes Z-01's
terrain role and the CommonScene background role of `zhujiemian_01`.
[The augmentation handoff](terrain-z01-2026-11-11.md) records publication,
hashes and preservation. Z-01's identity as the island near Military Base
remains unknown. Inventory values below describe the pre-augmentation audit.

## Source identity and evidence

- Supplied recovered APK: `export/lite4.0.apk`, SHA-256
  `2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
- Reused recovered AArch64 ELF: `/tmp/codex-minimap-review/libil2cpp.so`,
  SHA-256
  `2e2efde1a103549694b17b361cacc18faa5714d06c444945e44b30f1377e2b3c`.
- Reused metadata: `/tmp/codex-memory-dump-review/source/global-metadata.dat`,
  SHA-256
  `548b4e3e990641716e6e368de780fbf8c82f9806c0238d01e3bc02d0d644f2e6`.
- Historical pre-augmentation terrain manifest SHA-256:
  `89cd215635b0de1ab721a4ab021b315fd6feb9532328e535d5c21fe8293a5c82`.
- [Bounded native instructions](evidence/mapobjects-native.log), SHA-256
  `adcbe7532cdce0a55b0c5f52632ba82d89137a8fc2b42d1b5886cee17fffc631`.
- [REA metadata evidence](evidence/mapobjects-metadata.json) retains complete
  summary and selected views, including native-link failure.
- [Scene inventory and military region](evidence/mapobjects-terrain-coverage.json).

The recovery direction and matching asset provenance are already recorded in
[evidence/minimap-dump-pairing.md](evidence/minimap-dump-pairing.md).
Candidate metadata/native files from the installed counterpart are distinct;
they were not substituted.

Recovered C# is damaged pseudocode: some vector assignments, opaque external
types and literal arguments are lost. Do not interpret `(object)0` as an
authored transform, or overlapping `FieldOffset` annotations as a reliable
runtime layout. Native observations below describe the unpatched fallback;
IFix patch selection and current live state remain unknown.

## Placement records and field order

`MapObject` and `SceneObject` are separate concepts. The gameplay component
has `InsId:i64`, `CfgId:i32`, name, HP, MaxHP and renderer references
(`export/MapObject.cs:11–31`, Awake RVA `0x2417058`). The placement hierarchy
has position, scale, Euler angles, prefab name, holder and scene name
(`export/SceneObject.cs:13–34`). A render prefab alone authenticates neither
world placement nor health/resource behavior.

The common object record is position Vec3, scale Vec3, angles Vec3, then
`prefabIndex:i16`: nine f32 values plus two bytes, 38 bytes total.
`SceneObject.Unmarshal` at `0x1FB7E40` reads the floats into offsets
0x10–0x30 in exactly that order, then calls `Octets.pop_short`.
Native `0x1FB7F64` uses `sxth w1,w0` before prefab-table lookup.
This contradicts the pseudo-C# mask-to-unsigned presentation at
`SceneObject.cs:154–160`: retain the authored signed value and reject
negative/out-of-range indices instead of claiming 32768–65535 are valid.
The DTO declares `short` too (`GameObjectInfo.cs:25–29`, unmarshal RVA
`0xF83D18`).

| Record | Ordered content after common transform/index | Minimum bytes | Source |
| --- | --- | ---: | --- |
| GameObjectInfo | none | 38 | `GameObjectInfo.cs:145`, RVA `0xF83D18` |
| GrassInfo | brightness i16 | 40 | `GrassInfo.cs:29,150,182–183`, RVA `0xF84B0C` |
| TreeObjectInfo | treeType u8; id i32 | 43 | `TreeObjectInfo.cs:29–33,155,187–189`, RVA `0x100E770` |
| StealthGrassMapObjectInfo | id i32 | 42 | `StealthGrassMapObjectInfo.cs:29,150,182–183`, RVA `0xFFC538` |
| GameObjectWithLightmap | lightmapIndex u8; scale/offset Vec4; uvSize i32; i32 count plus ByteVec2 UV records | 63 | `GameObjectWithLightmap.cs:29–41,316,408–441`, RVA `0xF842D0` |
| BuildingObjectInfo | i32 lightmap-record count; (u8 index + Vec4) records; buildingId i32 | 46 | `BuildingObjectInfo.cs:14–22,236,322–351,403`, RVA `0xF337BC` |

The second GrassInfo short is **brightness**, not a grass ID or sequence
index. Runtime `GrassObject.m_GrassIndex` is separate and set through
`SetGrassIndex` (`GrassObject.cs:25,35`). Runtime grass Unmarshal
(`0x1FB90C8`) calls the common reader then consumes one short; that fallback
does not visibly store the brightness result. Preserve the source value
regardless.

`LightmapInfo` contains only u8 index and Vec4 scale/offset
(`LightmapInfo.cs:14–18,100,128–129`, RVA `0xF861D8`).
Do not mix this record with `GameObjectWithLightmap`'s uvSize and ByteVec2
payload. The native `BuildingObject.Unmarshal` (`0x1FB7CAC`) independently
confirms common object + i32 count + (u8,4f32) per record + i32 building ID.

`SceneInfo` is, in order:

1. terrainlightmapOffset u8;
2. i32 count plus prefab strings;
3. i32 count plus CellSceneInfo records (`separatedThingList`);
4. i32 count plus GameObjectInfo records (`bigSizeObject`);
5. i32 count plus OcclusionGroup records.

See `SceneInfo.cs:14–30,760,930–944,987–1010,1059–1082,1131–1154`,
unmarshal RVA `0xF9E7FC`.

`CellSceneInfo` is terrainX f32, terrainY f32, its own counted prefab string
table, then counted GrassInfo, TreeObjectInfo, StealthGrassMapObjectInfo and
GameObjectInfo (`thingsNoLightMap`) lists. See
`CellSceneInfo.cs:14–38,939,1143–1159,1202,1274,1346,1418`, RVA
`0xF49434`. Scene and cell prefab-table scopes are distinct.

`StaticGroup` has position/scale/angles, then counted lod0things and
lod1things, each GameObjectWithLightmap.
See `StaticGroup.cs:14–30,424,549–581,636–668`, RVA `0xFFB8AC`.
`OcclusionGroup` has position/scale/angles, boxsize, boxcenter, then
counted waikes, kuangjias and wujians lists, all GameObjectWithLightmap.
See `OcclusionGroup.cs:14–42,595,750–773,842,921`, RVA `0xF8C214`.

No numeric serialized kind discriminator was established. These DTOs use
typed lists. Earlier legacy kind buckets and randomized substitutions do
not establish the recovered source's schema. Normal and XOR-code marshal
overloads exist; their existence does not prove an actual payload is encrypted.

## Loading path: observed steps and unresolved strings

`SmallSceneMgr.LoadSceneData` creates a coroutine
(`SmallSceneMgr.cs:12178`, RVA `0x1FCBB74`).
Its MoveNext (`SmallSceneMgr.cs:2243`, RVA `0x1FD079C`) shows:

- `0x1FD08E0`: scene name getter;
- `0x1FD08EC/08F0`: prefix/suffix literal slots;
- `0x1FD08F8`: three-string concatenation;
- `0x1FD0B64`: Utils.GetFilePathForWWW;
- `0x1FD0B6C`: UnityWebRequest.Get, followed by SendWebRequest;
- `0x1FD0C8C`: LargeSceneManager.CheckSceneInfoMD5;
- `0x1FD0D7C`: SmallScene.InitDateToMember with an Octets instance;
- `0x1FD0D88`: SmallScene.ReadDependedABs;
- `0x1FD0D94`: PrepareAllABOfScene (`0x1FCC3DC`).

`SmallScene.InitDateToMember` (`SmallScene.cs:1260`, `0x1FC4DAC`)
allocates SceneInfo (`0xF9E168`), stores it and calls Octets.pop.
Thus topology JSON and placement Octets must remain separate inputs.

The metadata-only REA inspection decoded version 29 and 14,033 string
literals. Selected Evidence
`ev_3951759b019f5da2bfa825a1f55f2979455cfb8e4c1c624c2525e5482123376e`
observes `sceneinfo/` at index12471, `.oc` at1085,
`objectInfo.obj` at12046 and the gameconfig topology/preabs paths.
A plausible path is `sceneinfo/<scene-name>.oc`; **it is not yet an exact
verified URL or member path**.

The loaded prefix slot `0x4636258` contains
`0x00004000283E72C8`; suffix slot `0x45C66E0` contains
`0x00004000283D0F30`. They are not file-backed ELF addresses. No relocation
mapping was established. REA's binary/metadata registration link failed with
an out-of-range type-definition index2019059728; metadata-only decoding works.
That failure is retained, and no unchanged repeated native-link analysis
was attempted. The static slots' exact metadata association is unresolved.

`sceneInfoPath` and `sceneInfoPath_jidi` belong to
`SC.LargeScene.SmallSceneList` (metadata type8983, fields32551/32552;
`SmallSceneList.cs:15,19`), not the SmallSceneMgr placement coroutine.
`LargeSceneManager.LoadSceneInfoFile/LoadSceneCellInfoFile`
(`LargeSceneManager.cs:2609/2750`, `0x1339464/0x13395C0`)
select MapCfg paths and load text. These topology loaders do not supply
the placement bodies.

The extractor's bounded A-01 and StreamingAssets inspections are retained
in [placements-a01-inspection.json](evidence/placements-a01-inspection.json)
and [placements-streamingassets-inspection.json](evidence/placements-streamingassets-inspection.json).
StreamingAssets is an extensionless UnityFS manifest bundle with classes142
and290; that inspected bundle has no TextAsset/MonoBehaviour placement body.
The APK/member and FileList scans reported no sceneinfo/objectInfo path.
This supports an absent inspected placement source, not proof that every
possible downloaded/external carrier is absent.

## Runtime transforms, state and streaming

Native TreeObject.AddToScene (`0x1FD5A6C`; `TreeObject.cs:107`)
borrows the named prefab holder, initializes TreeInfo with tree ID/type,
registers it, chooses activity based partly on an existing tree-state set,
then sets **world position**, **world Euler angles**, and **local scale**.
Calls `0x3438AB4/0x3438D0C/0x34392E4` correspond to the setters shown at
`TreeObject.cs:478,499,519`. All three vectors come from the placement object;
the fallback does not clamp scale or substitute random variants.
TreeInfo Init/Add/Remove methods are at
`TreeInfo.cs:334/41/214` (`0xFBC3B4/0xFBC13C/0xFBC298`).

BuildingObject uses the same setter pattern and applies renderer-specific
lightmap indices/scale-offsets
(`BuildingObject.cs:1137–1139,1208,1227,1246`, AddToScene `0x1FB6F28`).
All-zero lightmap defaults would lose authored behavior.

StaticGroupObject.AddToScene (`0x1FD3140`, `StaticGroupObject.cs:228`)
sets the group transform, calls child AddToScene, then
`Transform.SetParent(parent,true)` at native `0x1FD32F0`
(`StaticGroupObject.cs:503`). The explicit true argument preserves the
child's world transform. Simply multiplying child placements by group
transform again would be wrong for this observed path.

Grass retains a matrix and GPU-instancing state
(`GrassObject.cs:12–33`). Native Unmarshal converts angle values with a
degrees-to-radians factor and builds a TRS matrix. Exact Unity Euler
composition order, Bevy quaternion conversion and parity under nonuniform
or negative scale remain to be verified; do not silently choose glam XYZ.

Grass culling fallback (`0x1FB934C`) dispatches RemoveFromScene/AddToScene
through the SceneObject vtable and retained scene pointer.
OnRenderQualityChanged fallback (`0x1FB93EC`) returns without work.
needShow (`0x1FB9444`) uses runtime grass sequence index:
quality>=3 selects index divisible by4; quality<3 selects index divisible
by6. This is an observed fallback thinning rule, not a universal live
quality setting or brightness rule.

SmallScene stores pending-load/deletion and per-part object collections
(`SmallScene.cs:14–46`); its static loadDistance is450
(native `0x1FC8094`, `SmallScene.cs:7212`).
CalToLoadPart/`_DoLoad`/`_DoLoadPart`/`_DoUnload` exist at
`0x1FC5EA4/0x1FC6BCC/0x1FC7024/0x1FC7D48`
(`SmallScene.cs:2628/4045/4709/6828`).
The exact distance metric, hysteresis and per-frame budget were not traced.

Unity inactive/pooled objects and hidden renderers are different states.
SceneObjectPool holds prototypes and recycles holders
(`SceneObjectPool.cs:15,169,301`).
A Bevy implementation must synchronize render visibility, shadow casting,
collision and interactability with imported source activity and gameplay
state. Renderer-hidden alone does not reproduce Unity GameObject deactivation;
neither every tree nor every grass mesh should become a solid collider.

## Terrain and the island near Military Base

The pre-augmentation manifest had399 render instances from241 distinct source scene
bundles. Reusing the existing Rust matcher against the retained source
`allscenelist.json` returned exactly:

```json
{"imported_bundle_count":241,"imported_not_listed":[],"intersection_count":241,"listed_count":243,"listed_not_imported":["z-01","zhujiemian"]}
```

The retained source list JSON SHA-256 is
`6fce42f16e784666024943bd91bc9717ce38ef5928a1caca19d73dbe7f2be2b1`.
MapCfg1 selects this list and uses400-unit cells; MapCfg5 is a separate
512-unit `Mditu_Island` map
([minimap-canvas-review.md:204–239](evidence/minimap-canvas-review.md)).
The current project's ImportedIsland label does not prove source MapCfg5.

All16 `assets/scene/junshijidi_terrain_01.ab` through `_16.ab` bundles
are already in the canonical package, with20 render instances. Their combined
Bevy-frame AABB is X[-4600,-2999.9998], Y[-1.314341,60.737804],
Z[-600.0003,1000.0001]. Some `junshijidi`-named objects also occur in C-18
and C-30, so a name match alone is not a unique landmark identifier.
Full map AABB is X[-4600,3000], Y[-45.363487,255.04572],
Z[-3031.4,3000.0002].

**Historical unknown, subsequently narrowed:** retained inspection now
establishes Z-01 terrain and `zhujiemian_01` CommonScene background content.
The remaining unknown is whether the described neighboring island is Z-01, another
map, geometry already imported but not visible, or data outside the current
package. Included AABBs do not establish water/land topology or runtime
streaming visibility. Do not identify z-01 as the missing island or synthesize
land solely from the historical243-versus241 count.

The subsequent [bounded local cache search](evidence/placement-cache-search.md)
found no authentic placement-cache filenames in the authorized project and
source-review roots. The legacy `MSPLC002` binary is excluded as SceneInfo
evidence. This leaves the placement-source milestone below open.
The later package-scoped Waydroid extension encountered host permission
denials. The final authorized ADB check listed no devices, and the prior local
Waydroid endpoint timed out; it does not establish absence of cached
placement data. See
[the retained extension](evidence/placement-waydroid-cache-search.json).

## Next milestone and acceptance criteria

1. Find the authentic placement body in a bounded candidate/source pass.
   Pin APK/source digest and exact member/payload path; keep absent results
   explicit. External/downloaded placement data may be needed.
2. Validate Rust readers against a real complete payload, preserving byte
   spans, signed prefab indices, independent tables, brightness, IDs,
   nonuniform scales, lightmaps, UV records and typed child lists.
   Synthetic tests establish reader mechanics, not source placement parity.
3. Instantiate one source cell in Bevy using authentic placements and complete
   prefab hierarchy. Check a tree, building, grass and ordinary object against
   source transforms, materials and collider activity.
4. Add resource/HP/harvest/destruction identity using source MapObject/TreeInfo
   and authoritative content rules. Preserve changes across streaming unload.
5. Add bounded scene/cell streaming and pooling, then validate load/unload
   collisions, retained destroyed states, quality/culling and shadow behavior.
6. Investigate the island separately by source scene inventory, spatial
   geometry and a fixed-camera comparison once its landmark is established.

Success means source-backed placement counts and transforms, visible
hierarchy/material fidelity, correct collider state and stable streaming.
A hand-filled landscape with plausible props cannot pass that source parity
milestone.

