# Rust source imports — 2026-11-11

The extractor reads `export/lite4.0.apk` offline on Linux using
`unity-asset-binary/core/decode` 0.2.0. This pass used no Python, Android runtime,
or Unity installation. APK identity:
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`
(1,115,839,572 bytes). Canonical terrain and the original three-panel UI package
were preserved.

## Implemented safeguards

- An explicit CLI action is required; omitting flags no longer starts `--all`.
- Indexing uses ordered maps and bounded worker batches (default 2, allowed 1–8).
  Scan failures are reported and prevent replacing a usable index. Selected
  scans publish `selected_asset_index.json` separately. Ambiguous bare PathIDs
  are omitted; cross-file references use CAB-qualified identities.
- ZIP duplicate names are rejected. Member reads are bounded to 256 MiB;
  decompressed Unity bundles to 512 MiB, with metadata and block-count bounds.
  Mesh streams are validated before allocation. Resolver caches hold at most
  16 bundles and 256 MiB of cache-owned payloads; caller handles and parsed
  metadata are outside this cache bound.
- Selected prefab imports retain hierarchy, component fields and original
  bytes, converted local transforms, render meshes/materials/2D texture mips,
  exact collision triangles, and source collider/shadow/lightmap metadata.
  Unsupported non-2D material samplers remain explicit with their source bytes.
- UI panel supersets publish independently with a hashed `panel_selection.json`.
  Packages without this field retain the original strict three-panel contract.
- The extractor dev profile disables debug/incremental output and optimizes
  dependencies to reduce disk usage and offline import time.

## Published packages

Expanded UI:
`assets/imported_ui/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1-panels-e1739e152490099d`.
It contains BattleMinimap, BattlePanel, BattleMapPanel, BagAndBuildPanel,
BattleLoadingPanel, MainPanel, SettingPanel, DeathPanel and BluePrintBuildPanel.
Full verification covered 70 bundles, 30,629 source objects, 555 sprites,
166 decoded textures and 62,236 retained files (831,070,387 bytes).
Import reported 0 errors and 90 warnings: 80 unresolved references, 8 sprite
preview limitations, one empty source font atlas and one font without embedded
outline data. Manifest SHA-256:
`ee1df91b8dbf610b108a55fd5530044a09104dd803d7ffe674db258181a3e883`.
See [ui-expanded-verification.json](evidence/ui-expanded-verification.json).

Supermarket prefab:
`assets/imported_prefabs/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df/building_chaoshi_01-3be4bce3aeca`.
Source `assets/sceneprefab/building_chaoshi_01.ab` yielded 303 nodes,
298 renderers, 46 mesh parts, 3 collision meshes and 1,314 verified files.
One Cubemap sampler is retained with explicit unsupported status.
See [prefab-supermarket-import.json](evidence/prefab-supermarket-import.json).

## Reproduction

Run from the repository root. Existing package directories are verified before
publication; failed staging does not replace a completed package.

```sh
cargo run --offline --locked --manifest-path tools/asset_unpacker/Cargo.toml -- \
  --prefab-bundle assets/sceneprefab/building_chaoshi_01.ab \
  --report memory/evidence/prefab-supermarket-import.json

cargo run --offline --locked --manifest-path tools/asset_unpacker/Cargo.toml -- \
  --pass3 \
  --ui-panel BagAndBuildPanel=assets/ui/bagandbuildpanel.ab \
  --ui-panel BattleLoadingPanel=assets/ui/battleloadingpanel.ab \
  --ui-panel MainPanel=assets/ui/mainpanel.ab \
  --ui-panel SettingPanel=assets/ui/settingpanel.ab \
  --ui-panel DeathPanel=assets/ui/deathpanel.ab \
  --ui-panel BluePrintBuildPanel=assets/ui/blueprintbuildpanel.ab

cargo run --offline --locked --manifest-path tools/asset_unpacker/Cargo.toml -- \
  --verify-ui assets/imported_ui/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1-panels-e1739e152490099d \
  --report memory/evidence/ui-expanded-verification.json
```

The selected index validation used exact members
`assets/sceneprefab/building_chaoshi_01.ab` and `assets/scene/a-01.ab`
(zero padded). Both 1-worker and 2-worker runs scanned 2/2 selected bundles out
of 6,831 APK bundles, with 12 qualified entries and no failures or ambiguities.
The selected index SHA-256 was identical across worker counts:
`52e4e88b03223aa78f079c2b90334579573b06e7757f3c7bb1f2e6318ed70b00`.
The optimized 1-worker run took 4.62 seconds; the earlier unoptimized 2-worker
run took 157.89 seconds, mostly APK hashing. This timing is a profile comparison,
not a worker-scaling benchmark.
See [index-selected-validation.json](evidence/index-selected-validation.json)
and [index-selected-validation-one-worker.json](evidence/index-selected-validation-one-worker.json).

Final validation: 35 extractor tests passed; all-target Clippy with
`-D warnings`, scoped formatting and `git diff --check` passed.

## Subsequent terrain completion

The original package preservation above describes the initial source import
pass. A later authenticated augmentation added the omitted Z-01 terrain
scene while retaining the predecessor package. Canonical terrain now contains
242 terrain scenes, 400 source instances and 383 enabled collision records.
All 41 current extractor tests and strict all-target Clippy passed.
See [terrain-z01-2026-11-11.md](terrain-z01-2026-11-11.md) for source identities,
verified hashes, backup path and idempotence evidence. SceneInfo discovery and
prefab-loader verification are recorded separately in
[world-placements-2026-11-11.md](world-placements-2026-11-11.md) and
[prefab-supermarket-loader-verification.json](evidence/prefab-supermarket-loader-verification.json).

## Remaining boundaries

Serialized prefab positions do not authenticate world placement. The shared
Box/Mesh collider conversion and geometry queries, validated prefab package
loader, and diagnostic Bevy prefab spawning are now implemented. Authored
world placement, complete gameplay activation, sensor behavior, original
PhysX cooking and source shader parity remain unfinished. Cubemap fidelity,
complete character/animation import and custom component behavior are also
pending. UI verification checks hashes,
APK raw bytes, hierarchy, and texture metadata/PNG consistency; it does not
independently repeat every decoder or establish rendered/runtime equivalence.
The 90 UI warnings and unsupported Cubemap remain visible evidence, not silently
replaced source behavior.
