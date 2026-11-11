# APK reconstruction

Started: 2026-11-09. Updated: 2026-11-10.

Continuation on 2026-11-11: [the implementation ledger](../PLAN/source_reconstruction.md)
supersedes current-engine/UI claims in this dated snapshot. Native Bevy UI
replaces the egui runtime, nine source UI panels and the supermarket prefab
are extracted, six original gameplay tables are retained/loaded, and the island
uses a shared validated world descriptor. See [source imports](source-imports-2026-11-11.md),
[gameplay data](gameplay-data-2026-11-11.md) and [storage audit](storage-2026-11-11.md).
Historical extraction identities and captures below remain preserved evidence.

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
shared with terrain. The native HUD now uses the extracted BattleMinimap;
the other HUD widgets, compass and large map retain their existing egui painters.

Two fresh imports produced identical manifest SHA-256
`677424f9e43225f3677f5713bed526dd9099339f2a8b44dbaecee7d41928cae5`.
All 20,782 files in the canonical UI directory are referenced and verified;
no stale files remain. See `evidence/ui-reproducibility.json`.

The extractor and client now share `midnight_ui_data` (schema 1). Its verified
payload loader, RectTransform resolver and sprite geometry feed
`client_core::source_ui::SourcePanelRenderer`. The source BattleMinimap renderer
has standard alpha-mask clipping, source embedded-font text and explicit
qualified texture/text/visibility/transform/sprite/color/instance bindings.
Full verification after moving the schema preserved
the manifest digest and verified 394,890,740 bytes across all 20,782 files.

Updated minimap GPU previews at 1920×1080 and 1280×720 are in
`evidence/minimap-render/`. They use source bootstrap canvas settings as explicit
inputs and show serialized templates with no runtime map texture/visibility.
Per-graphic reports retain unsupported cases; they are not APK screenshots.
`evidence/minimap-canvas-review.md` recovers canvas/scalar calibration and a
definite large-map texture binding. `evidence/minimap-font-coverage.json` records
seven missing nominal source-font character mappings, with two independent parsers.

The optional initialization profile adds two captures applying only 21 verified
source GO hides (27 inactive graphics). It checks qualified IDs and source
APK/bundle/layout identities and retains native provenance. Its file hash covers
the exact parsed bytes. No texture/UV/scale/zoom or successful runtime init is
inferred. `evidence/minimap-runtime-review.md` records these source actions,
constructor zoom=15 and the unresolved 94-vs-86 binding-list discrepancy.

`client_core::source_minimap::NativeMinimap` now integrates this panel into the
production Arch Linux HUD. It initializes before the first egui pass and binds
the retained 2048×2048 `Map_map` texture, player-centered UV crop, image scale
260/2048 and simulation state. The source texture's BattleMapPanel assignment
is established; its minimap assignment, crop behavior, marker sizes/categories
and lifecycle are explicit native policies. The shared overview half extent
remains provisionally 5360. The adapter uses qualified source identities and
retains source geometry, sprite UVs, masks and embedded font data.

Native zoom defaults to 15 and is constrained to 3–60. Scroll over the minimap
to zoom; click to open the existing large map; F2 opens the HUD editor. The source
330×330 root fits a 360×363 HUD envelope including the header/time background.
Player position/yaw, markers, coordinates, height and simulation time are bound
live. The obsolete minimap painter and duplicate texture loads were removed.
There is no Waydroid dependency. See `ui-reconstruction.md` for exact policies,
binding identities and remaining Unity fidelity gaps.

The native probe has four controlled states at two resolutions and writes to
`evidence/native-minimap/`, separate from the preserved prefab captures. Its
per-graphic reports retain bindings and instance IDs; active unsupported graphics
or absent map/player submission fail the probe. These captures validate the Rust
adapter and do not prove equality with the original Unity runtime.

Waydroid's `com.herogame.gplay.survival.lite` 40 / 4.0 APK has a different
container digest. The Rust full ZIP comparison confirms 9,318 matching regular
file payloads, including every asset; only libil2cpp.so and global-metadata.dat
differ. The user identifies the files inside lite4.0.apk as recovered memory
dumps of the encrypted game. The installed package is the encrypted runtime
counterpart, not an established unrelated version. The comparison and
`evidence/minimap-dump-pairing.md` retain both identities and native method
coverage. Three of seven mapped method ranges match exactly; four differ on
disk. No Android app launch, installation change or APK screenshot was performed.
Effective live-memory/IFix/texture/parentage and matched gameplay captures remain pending.

## Checklist

- [done] Source geometry, coordinates, materials and source mip retention.
- [done] One canonical map for rendering and source collider physics.
- [done] Reproducible Rust import and integrity/coverage verification.
- [done] Static extraction of the three selected UI panel hierarchies.
- [done] Shared source UI schema/layout and BattleMinimap Bevy/egui preview.
- [done] Production native BattleMinimap with texture/UV/scale and simulation bindings.
- [done] Bootstrap canvas/scalar calibration and independent font coverage.
- [done] Focused tests, workspace checks and Bevy GPU map captures.
- [pending] Remaining source HUD panels, custom graphics and original runtime transitions.
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
cargo test --offline -p midnight_ui_data -p midnight_map_data -p shared -p client_core
cargo test --release --offline --manifest-path tools/asset_unpacker/Cargo.toml
cargo clippy --offline -p midnight_ui_data -p client_core --all-targets
cargo clippy --offline --workspace --all-targets
cargo rustc --offline -p client_core --example minimap_probe -- -C strip=debuginfo
target/debug/examples/minimap_probe memory/evidence/minimap-render
target/debug/examples/minimap_probe memory/evidence/minimap-render memory/evidence/minimap-render/initialization-visibility.json
target/debug/examples/minimap_probe memory/evidence/native-minimap --native
cargo rustc --offline -p client_runner --bin client_runner -- -C strip=debuginfo
target/debug/client_runner --help
cargo test --release --offline --manifest-path tools/asset_unpacker/Cargo.toml --example apk_member_compare
cargo run --release --offline --manifest-path tools/asset_unpacker/Cargo.toml --example apk_member_compare -- export/lite4.0.apk .local/apk-runtime-candidates/waydroid-527ae0a1.apk --report memory/evidence/apk-runtime-member-comparison.json
```

Current native minimap checks pass: 30 ordinary client tests and all three
explicitly executed asset-backed tests for source geometry/headings/DPI, binding
validation/parent visibility and actual HUD initialization/zoom/map/editor input.
Shared UI data has 15 passing tests. The asset tests remain marked ignored for
installations without the extracted package; this workspace's stripped test
executable was explicitly run with `--ignored --nocapture`.

Offline workspace all-target Clippy completes with only existing map package,
compass, terrain probe and shared test warnings; it is not a clean workspace-wide
`-D warnings` run. The new minimap code has no reported warnings. Scoped formatting
and `git diff --check` pass. GPU capture execution needs host Vulkan access because
the sandbox cannot load the installed NVIDIA driver; Cargo builds/checks are offline.
The native probe completed eight captures at 1920×1080 and 1280×720 on NVIDIA
RTX 4060 Laptop/Vulkan with zero unsupported active graphics and verified
map/player submission. Its exact inputs, bindings and per-graphic statuses
are retained in `evidence/native-minimap/render.json` and the capture reports.
Visual review corrected a reversed player cone and the heading regression test
now checks arrow and cone at all four cardinal headings. All eight final captures
were refreshed and inspected. The stripped native runner was rebuilt after this
fix; its `--help` entry point succeeds. A full interactive game-window comparison
has not been performed. See `evidence/native-minimap/README.md`.

The final local client test run used:

```bash
cargo rustc --offline -p client_core --tests -- -C strip=debuginfo
target/debug/deps/client_core-db0409942cc3474f --include-ignored --nocapture
```

All 33 passed; the test executable's Cargo hash may differ on another build.

Earlier foundation checks recorded map data 19, shared 46 with one intentional
stress-test ignore, extractor 24 and the APK comparison example's three passing
tests. The comparison example has a clean scoped release Clippy check. It hashes decoded
members through streaming I/O, checks lengths/CRC, rejects duplicate/oversized
members, and verifies both input file hashes before and after the inventory.
Directory/order/compression/signing-block equality and live execution are outside
that check. The preserved installed APK copy is Git-ignored under
`.local/apk-runtime-candidates/`; it is evidence, not a replacement runtime asset package.

Debug linking filled the disk during this stage. This turn's earlier generated
probe binaries were preserved in `/tmp/codex-minimap-probe-before-sprite-fix`
and `/tmp/codex-minimap-link-failed-2b1c49b459b68ea3`. Use the stripped probe
build above and check free space before large links. Do not purge user assets,
history or unrelated caches to make space.

`--pass2` regenerates/verifies/publishes terrain. `--verify-package PATH` checks
a completed candidate; `--promote-terrain PATH` activates it after verification.
Predecessors are retained under `.local/map-import-backups/` or
`.local/ui-import-backups/`. Failed candidates are preserved for investigation
and are not fallback inputs. Generated packages/local backups are Git-ignored.

Use `ui-reconstruction.md` for precise UI gaps. Static extraction and Bevy
verification do not establish 1:1 gameplay or pixel equality to the APK.
