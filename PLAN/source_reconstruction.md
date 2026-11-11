# Source reconstruction plan and implementation ledger

Updated 2026-11-11. Target: `export/lite4.0.apk`, SHA-256
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
Use Rust and Bevy 0.20.1. Test on this Linux machine first; Android comes after
a stable desktop slice. Preserve source exports, packages, evidence and rollback
copies. A source import, a working Bevy feature and original-runtime parity are
separate acceptance stages.

## Implemented in this pass

| Work | Concrete result | Evidence / code |
|---|---|---|
| Bounded import pipeline | Selected deterministic scans, bounded ZIP/bundle reads and caches, explicit CLI actions, verification before publishing | `memory/source-imports-2026-11-11.md` |
| UI extraction | Nine source panels published separately; original three-panel package preserved | `memory/evidence/ui-expanded-verification.json` |
| Native Bevy UI | Source atlas/mip rendering, source geometry/masks, Bevy text, live HUD/minimap, inventory/catalog pages, menu and editor controls | `crates/client_core/src/native_ui.rs`, `source_ui/native.rs` |
| World selection | Imported island selected explicitly and by client default; identical validated source collision on client/server; content identity checked at connection | `crates/shared/src/world.rs`, `content.rs` |
| Source gameplay definitions | Six strict Octets readers, original raw tables, hash-verified catalog, item IDs and blueprint requirements | `crates/game_data/`, `memory/evidence/gameplay-verification.json` |
| Inventory model support | Source-ID stacking and completed-craft transactions roll back on missing materials or insufficient capacity | `crates/game_data/src/lib.rs` |
| Environment | Half-resolution world-space volume ray marching, opaque-depth clipping, bounded quality, below-horizon sun/night handling, cascaded shadow maps; artifact-producing contact shadows disabled on desktop | `crates/client_core/src/environment.rs`, `assets/shaders/volumetric_clouds.wesl` |
| MapObject extraction pilot | Verified supermarket hierarchy, render parts/materials and exact mesh-collider data | `memory/evidence/prefab-supermarket-import.json` |
| Omitted terrain scene | Source Z-01 geometry, four-layer material/mips and solid MeshCollider added; 400 render instances/383 colliders across 242 terrain scenes; previous package preserved | `memory/terrain-z01-2026-11-11.md` |
| Shared prefab runtime boundary | Hash-verified reader, source-field/hierarchy/material/mesh checks, Bevy diagnostic renderer and LOD selection, atomic shared Box/Mesh query collision | `crates/map_data/src/prefab.rs`, `crates/client_core/src/source_prefabs.rs`, `crates/shared/src/physics/source_prefabs.rs` |
| Placement discovery/readers | Bounded APK/metadata/cache audit; strict SceneInfo/CellSceneInfo reader preserves signed indices, typed lists, source values and byte spans | `memory/world-placements-2026-11-11.md`, `memory/mapobjects-source-2026-11-11.md` |
| Low-disk operation | Compact dev profiles; read-only storage/build-reserve gate; precise cache inventory | `tools/storage_audit.sh`, `memory/storage-2026-11-11.md` |

The source catalog is loaded and retained by the island simulation/server. Its
identity must match the terrain APK. It can be browsed in the Bevy interface.
The completed-craft API applies only externally authorized completion: it does
not infer unlocks, workbench rules or time units. Fixture gameplay remains
available through `--map polygon` and `--map load-arena`.

## Next milestones, in dependency order

Check results and capture evidence are in
[desktop validation](../memory/implementation-2026-11-11.md) and the later
[MapObject validation](../memory/implementation-mapobjects-2026-11-11.md).

### 1. Complete world placement and the missing island

Z-01 was missing from the terrain import and is now published at its exact
source bounds X[-3639,-3127], Y[-7.101078,26.990854], Z[-1269,-757]. All 16
military terrain bundles were already present. The remaining source-list entry
is a main-menu scene. Identification of Z-01 as the described neighboring
island still requires the original landmark/view comparison.

No authenticated SceneInfo body was found in the inspected APK, FileList or
task-owned local caches. The original game's protected Waydroid cache could
not be enumerated: direct reads were denied, noninteractive sudo required a
password, and the known local ADB endpoint was unavailable. Cache absence is
not established. The supermarket can now be verified and previewed in
Bevy, and its 55 solid colliders rebuilt through shared queries; one trigger
is reported. Exact world placement, sensor behavior, source glass/cubemap
shading, PhysX cooking and gameplay activity remain separate acceptance work.

Inventory actual scene/placement payloads alongside `MapObject`, `TreeObject`,
`BuildingObject`, `GrassObject` and related C# readers. Determine serialized
coordinates, source hierarchy, IDs, active flags and streaming ownership.
Preserve every source record and classify unsupported ones. Locate the island
beside Military Base in the original scene coverage; confirm its extent and
connections before changing terrain.

Use the implemented shared verified prefab reader, Bevy renderer and collision
boundary to instantiate authenticated placement records. Convert source
placement coordinates exactly once and preserve their streaming ownership and
source renderer/collider flags. Start with the supermarket package, then trees, buildings, grass, rocks,
props and water. Share stable IDs and collision with server/save data.

Acceptance: source inventory versus imported counts/hashes, known landmark
positions, visual and collision checks at Military Base and its neighboring
island, no guessed fallback placements. Prefab extraction alone does not meet
this milestone.

### 2. Recover player and inventory rules

Trace the original controller, bag-size messages, equipment/quick slots,
stack limits, starter contents, use-item callbacks and durability. Recover
movement/crouch/jump/sprint values, camera offsets and collision geometry.
Use `DrawingCfg` for normal blueprints; `CraftRecipeCfg` has only one special
battery recipe in this APK. Retain mode/workbench/unlock restrictions and
verify how the runtime uses `baseNumber`, `productNum` and timing fields.

Import character skeletons, skinned meshes and animation clips; connect Bevy
animation states to validated movement and action state. Port authoritative
source-ID inventory changes, harvesting, consumption and crafting before
combat/building. Do not map fixture `ItemKind` values to source IDs by labels.

Acceptance: repeatable movement and item transactions from matched source
inputs; no duplication or item loss, identical local/server results, exact
source rules documented with unresolved overrides separated.

### 3. Finish original UI behavior

Complete the extracted BattlePanel, inventory/building, map, loading, main,
settings, death and blueprint screens. Bind actual source state and callbacks,
then original transitions, scrolling, focus, localization, equipment icons,
slot counts and contextual controls. Finish custom graphics, text masking,
stencil behavior, rich text, font fallback/shaping, best fit and clipping.

Acceptance: keyboard/mouse and controller input routes correctly through Bevy;
menus block gameplay and restore focus; save/cancel works; captures at
1920×1080, 1280×720 and non-unit DPI; then matched Unity screenshots and action
scenarios. Current desktop controls and static prefab previews are intermediate
artifacts.

### 4. Match lighting and environment

Recover `DayNightAnimator`/weather curves, light/exposure settings and the
original material/shader parameters. The implemented volume is world-space
and depth-clipped; retain it while comparing source sky, sun disc, sunrise,
sunset and night. Match source terrain/object shading and cutout shadows,
then add clouds' influence on terrain lighting, water and weather effects.

Acceptance: GPU pipeline checks for single-sample and MSAA depth, foreground
occlusion, visible cloud-quality/toggle changes, coherent sun direction and
night luminance, controlled shadow caster comparison; then scene/reference
captures at fixed times and camera positions. Measure frame time and memory
before increasing volume resolution or shadow coverage.

### 5. Port remaining systems with shared authority

After player/inventory/world data are stable, port weapons/projectiles/reload,
building placement/damage, resource respawns, interaction/loot, NPCs, vehicles,
audio and persistence in dependency order. Each feature needs source data,
component/state mapping, simulation rules, authoritative commands, UI/animation
bindings and acceptance scenarios. Existing prototype versions provide
infrastructure and test fixtures, not verified source behavior.

Acceptance: record and replay fixed-step commands, reject invalid actions,
reconnect with consistent content/state, save/load stable source IDs, and test
representative multi-player scenarios before full capacity acceptance.

### 6. Validate the desktop slice, then Android

Keep a compact ledger of APK/C# identities, imported packages, Rust checks,
GPU captures and source-runtime comparisons. Capture representative terrain,
players, dense props, UI and day/night scenes; profile bounded streaming,
CPU/GPU time and memory. Preserve the last working binary before replacement.

Android begins only after the desktop slice passes these scenarios. Add the
Android runner, lifecycle/storage/input adapter and mobile quality presets
without replacing the shared simulation, source packages or Bevy UI.

## Verification commands

Current commands and controls are in `README.md`; import commands in
`memory/source-imports-2026-11-11.md` and `memory/gameplay-data-2026-11-11.md`.
Re-run native capture probes after shader/UI changes. Store new captures in
separate directories so earlier evidence remains intact. Update this ledger
with observed results; do not mark 1:1 parity complete from static extraction
or successful compilation alone.
