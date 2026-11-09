# APK reconstruction

Started and updated: 2026-11-09.

## Objective and constraints

Reconstruct `export/lite4.0.apk` faithfully, including its map, UI and gameplay.
The first stage establishes one source-derived map and maintainable Rust
pipelines. Communicate in English. Use Rust and installed extraction tools;
no Python was executed. Preserve unrelated changes and legacy assets.

APK size: 1,115,839,572 bytes. SHA-256:
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
Observed Unity: `2021.3.23f1c1`. Project: Bevy `0.20.1`,
`bevy_egui` `0.43.1`, `bevy_rapier3d` `0.37.0`.

`tools/asset_unpacker` uses installed Rust Unity libraries/codecs and its own
Cargo.lock. Use offline Cargo. REA setup is installed, but its MCP tools are
absent from this live session. These extractor reports are ordinary Rust-tool
observations, not REA Evidence IDs. Do not repeat setup.

The worktree contained unrelated changes before this task, and some work was
committed externally during the session. Inspect current Git state rather
than attributing every diff to this task. Legacy `assets/ui`, model/placement
archives, Python scripts and recovered C# exports remain available.

## Canonical map

Package: `assets/imported_map/terrain/`. Geometry magic `MSMAP003`, manifest
schema 2, source mip archive `MSRGBA01` (`.msrgba`). Geometry SHA-256:
`89febd1decfaa5ed58089e9912fd61dedcb5285d51acb86f665e35c23413e8ee`.

Verified coverage: all 241 terrain scene bundles; 399 distinct mesh instances;
287 terrain (160 three-layer and 127 four-layer), 95 LambertLit and 17 Yo/Add;
382 enabled non-trigger source colliders, 289 materials and 316 textures.
The instance count is not inflated by submeshes.

Source parent transforms are resolved before Unity `(x,y,z)` becomes Bevy
`(x,y,-z)` once. North is **-Z**, east **+X**. Geometry preserves supplied
normals/tangents, indices and UV0. Each original source mip is independently
decoded; PNGs are base-level previews. Source color-space and sampler metadata
are retained, and absent source tangents remain explicitly absent.

Renderer and Rapier share the validated package and source collider flags.
Invalid canonical data fails validation instead of activating an older archive.
Player placement queries source surfaces. Legacy guessed placements, duplicate
placement streaming, legacy water/fabricated ocean geometry and arbitrary
terrain distance hiding were removed from the active island path.

Bevy/Vulkan captures in `evidence/terrain-render/` verify all 399 entities and
312 uploaded source textures/mip chains on the RTX 4060 Laptop GPU at
1280×1024. These are not APK screenshots or proof of original lighting.
See `map-evidence.md` for exact shader math, encoded-N0 residual and TBN limits.

## UI source foundation

Pass 3 now extracts the supplied APK rather than copying legacy PNGs.
Source package:
`assets/imported_ui/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1/`.

It retains three complete static prefab hierarchies (2,458 nodes), qualified
references, component/child order, anchors, rotations, enablement, masking,
canvas/text/font settings, atlas/sprite metadata and original object bytes.
Coverage: 36 dependency bundles, 10,047 source objects, 322 sprites, 88 decoded
textures, one deferred empty font atlas and two fonts. Import: zero errors,
61 explicit warnings. `FogLightMask` was recovered from the APK.

A bounded Rust reader retains the complete 12,649,568-byte font without
raising the parser's general array limit. Non-finite floats receive explicit
JSON tags, with original binary bytes retained. An empty source Font Texture
is deferred without invented pixels; font atlas runtime contents are unknown.

`--verify-ui` checks 20,782 referenced files, nested source byte blobs, raw
object bytes against the APK, CAB metadata, hierarchy links and decoded mip
metadata/base PNG consistency. Imports use fresh staging, stable dependency
index serialization, verification before publication and backups/rollback
shared with terrain. The runtime still uses the hand-built egui HUD.

## Checklist

- [done] Source geometry, coordinates, materials and source mip retention.
- [done] One canonical map for rendering and source collider physics.
- [done] Reproducible Rust import and integrity/coverage verification.
- [done] Static extraction of the three selected UI panel hierarchies.
- [done] Focused tests, workspace checks and Bevy GPU map captures.
- [pending] Source UI rendering, masks, fonts, dynamic bindings/transitions.
- [pending] Full prefab placements, water and original lighting/shaders.
- [pending] Map-image calibration and comparisons with the running APK.
- [pending] Full gameplay/server behavior and complete-game fidelity.

## Reproduction and verification

Run from the repository root. `--verify` reads the map without re-importing it.

```bash
cargo run --release --offline --manifest-path tools/asset_unpacker/Cargo.toml -- --verify --report memory/evidence/terrain-verification.json
cargo run --release --offline --manifest-path tools/asset_unpacker/Cargo.toml -- --pass3
cargo run --release --offline --manifest-path tools/asset_unpacker/Cargo.toml -- --verify-ui assets/imported_ui/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1 --report memory/evidence/ui-verification.json
cargo check --offline --workspace --examples
cargo test --offline -p midnight_map_data -p shared -p client_core
cargo test --release --offline --manifest-path tools/asset_unpacker/Cargo.toml
```

These checks pass: map data 19 tests, client 23, shared 46 with one intentional
stress-test ignore, extractor 24. GPU captures remain in the evidence directory.

`--pass2` regenerates/verifies/publishes terrain. `--verify-package PATH` checks
a completed candidate; `--promote-terrain PATH` activates it after verification.
Predecessors are retained under `.local/map-import-backups/` or
`.local/ui-import-backups/`. Failed candidates are preserved for investigation
and are not fallback inputs. Generated packages/local backups are Git-ignored.

Use `ui-reconstruction.md` for precise UI gaps. Static extraction and Bevy
verification do not establish 1:1 gameplay or pixel equality to the APK.
