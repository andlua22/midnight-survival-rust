# World placement source boundary — 2026-11-11

The selected source audit has **zero authenticated world placements**. No world
placement package was created. The supermarket prefab remains a verified asset
package; its saved root position is not evidence of a runtime instance.

## Observed source data

[world-discovery-2026-11-11.json](evidence/world-discovery-2026-11-11.json) binds
the report to `export/lite4.0.apk`, SHA-256
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
The APK was hashed before and after discovery. The report retains hashes of the
selected raw APK members and recovered C# reader files.

- All **9,430 ZIP member names** and all `assets/version/FileList.txt` path
  entries were inspected. Neither inventory contains a `sceneinfo` path.
- `assets/gameconfig/allscenelist.obj` lists **243 scene names**.
  `assets/gameconfig/scenecell.obj` has **10,000 cells**, **5,029 nonempty cells**
  and **242 distinct referenced scene names**. These are map-1 loading
  inventories. Cell coordinates and scene membership do not place individual
  prefabs.
- Strict class inspection of `assets/scene/a-01.ab` found terrain, renderer,
  transform and collider objects, with no TextAsset or MonoBehaviour records.
  [A-01 inspection](evidence/placements-a01-inspection.json) retains the details.
- `assets/StreamingAssets` is a **385,726-byte UnityFS bundle** despite lacking
  an `.ab` suffix. It contains one AssetBundle and one AssetBundleManifest,
  with no placement/TextAsset/MonoBehaviour records.
  [StreamingAssets inspection](evidence/placements-streamingassets-inspection.json)
  retains its CAB and source fields.
- The exact selected candidate `assets/gameconfig/scenemaxmin.oc`
  (**4,000,004 bytes**, SHA-256
  `4e3c3bdddf1e4cdd149cbf014c80361db615fe80f178e094fa52a5338a0ed813`)
  **failed the plain SceneInfo decoder at byte 9** with
  `count exceeds remaining payload or limit`. Its contents were not substituted
  or interpreted as placements.

This is a bounded negative search. It does not inspect every serialized object
in every bundle or establish absence from downloaded/cached resources outside
the declared APK. Legacy XAPK-derived placements remain outside this source
authority.

## Metadata and reader contracts

A bounded Rust read of the original IL2CPP v29 metadata confirms **14,033 literal
rows**, with metadata SHA-256
`548b4e3e990641716e6e368de780fbf8c82f9806c0238d01e3bc02d0d644f2e6`.
The report retains exact literal indices and raw byte spans for:

| Literal | Index | Metadata byte offset |
|---|---:|---:|
| `.oc` | 1085 | 145121 |
| `objectInfo.obj` | 12046 | 419741 |
| `sceneinfo/` | 12471 | 425047 |

These match the independent REA literal evidence in
[mapobjects-metadata.json](evidence/mapobjects-metadata.json), Evidence ID
`ev_3951759b019f5da2bfa825a1f55f2979455cfb8e4c1c624c2525e5482123376e`.
Literal presence does not establish a complete loader path or remote URI.
Recovered `SmallSceneMgr.LoadSceneData` fetches data through UnityWebRequest
and checks `CheckSceneInfoMD5`, but its exact URI construction remains unknown.

The new `tools/asset_unpacker/src/world_scene.rs` parser follows recovered plain
`gs.battle.scmsg` field/pop sequences. It retains the complete original payload
in memory and assigns offset/byte-count/SHA-256 spans to each decoded record.
The bounded discovery report emits counts and at most 16 tree-record samples;
it does not dump complete decoded scenes or publish runtime data.

| Record | Preserved source fields | Minimum bytes |
|---|---|---:|
| GameObjectInfo / SceneObject base | position, nonuniform scale, Angles, signed i16 prefab index | 38 |
| GrassInfo | base plus signed i16 **brightness** | 40 |
| TreeObjectInfo | base plus u8 treeType and i32 id | 43 |
| StealthGrassMapObjectInfo | base plus i32 id | 42 |
| GameObjectWithLightmap | base, u8 lightmap index, Vec4 scale/offset, i32 uvSize, counted exact ByteVec2 data | 63 |
| CellSceneInfo | terrainX/Y, its own prefab-name table, grass/tree/stealth/things-without-lightmaps lists | 28 |
| OcclusionGroup | group transform, box size/center, three lightmapped-object lists | 72 |
| SceneInfo | terrain lightmap offset, global prefab table, cells, big-size objects, occlusion groups | 17 |

Source table scopes stay distinct: cell records resolve against their cell's
prefab table; big-size/occlusion records use the scene table. Native review
corrected misleading recovered C# unsigned-mask output: authored prefab indices
are retained as **signed i16**, and negative/out-of-table references are rejected.
`BuildingObject.Unmarshal` has a separate base + lightmap-list + building-ID
contract; it is not conflated with `GameObjectWithLightmap` and is not implemented
as a standalone placement reader by this pass.

The parser bounds payloads to **16 MiB**, strings to **4,096 bytes**, and list
counts to both the remaining bytes and one million entries before allocating.
It rejects non-finite floats, malformed UTF-8, truncated records, invalid table
references and trailing bytes. Exact position/scale/Angles are preserved;
coordinate frame, Euler order, hierarchy composition and IFix/encrypted-layout
behavior are not guessed. No converted world matrix is emitted before source
payload and native consumer/frame corroboration.

## Commands and verification

```sh
cargo run --offline --locked --manifest-path tools/asset_unpacker/Cargo.toml -- \
  --discover-world --world-member assets/gameconfig/scenemaxmin.oc \
  --report memory/evidence/world-discovery-2026-11-11.json

cargo run --offline --locked --manifest-path tools/asset_unpacker/Cargo.toml -- \
  --verify-prefab assets/imported_prefabs/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df/building_chaoshi_01-3be4bce3aeca \
  --report memory/evidence/prefab-supermarket-loader-verification.json
```

Discovery allows at most eight explicitly selected candidate members, reads
bounded metadata/topology, and activates no game data. The extractor now uses
shared prefab DTOs from `midnight_map_data` and calls
`load_prefab_package(candidate)` before publication, with complete mesh-record
roundtrip comparison. `--verify-prefab` verified the retained supermarket's
**303 nodes, 298 renderers, 46 mesh parts, 3 collision meshes and 1,314 files**.
See [loader verification](evidence/prefab-supermarket-loader-verification.json).

Validation: **41 extractor tests passed**, including source table scopes, signed
IDs/brightness, nonuniform scales, every truncation boundary, invalid counts and
indices, non-finite values, lightmap/UV bytes and exact record spans. All-target
Clippy with `-D warnings`, scoped formatting and `git diff --check` passed.
No full dump, Python extraction, Android execution, canonical asset replacement
or source/backups deletion was performed.

## Follow-up local cache access

The bounded filename search of project and task-owned source-review roots found
no authentic `sceneinfo/*.oc` or `objectInfo.obj` filenames. The existing
`assets/imported_map/placements.bin` has the legacy `MSPLC002` header and is
not accepted as recovered SceneInfo evidence. See the
[local cache-search note](evidence/placement-cache-search.md).

The exact Waydroid package roots for `com.herogame.gplay.survival.lite` were
also attempted. Host filename traversal was denied; the root agent's
passwordless `sudo -n rg` attempt required a password and read no files.
No password was requested and no permissions were changed.

The final authorized read-only route observed an active container service but
a stopped graphical Waydroid session. `adb devices -l` started only the
permitted incidental host ADB daemon and listed no devices. A bounded attempt
to connect to the sole prior task-known local endpoint
`192.168.240.112:5555` timed out after 12 seconds with no output; the final
device list remained empty. No game, container, graphical session or system
service was started, and no package contents were read. The
[complete retained evidence](evidence/placement-waydroid-cache-search.json)
records these observations. **The protected cache remains inaccessible;
its contents and absence are unverified.** Exploration stopped when these
known cache routes were exhausted.

The next source-dependent step is to recover the exact scene payload together
with its original identity and loader association. Only then can selected
world-placement packages authenticate prefab references, per-instance transforms
and IDs for runtime spawning.
