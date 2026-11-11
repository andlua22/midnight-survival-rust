# Source reconstruction plan and implementation ledger

Updated 2026-11-12. Target: `export/lite4.0.apk`, SHA-256
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
| Native Bevy UI | Source atlas/mip rendering, source geometry/masks, Bevy text, live HUD/minimap, inventory/catalog pages, menu/editor controls and deterministic Tab focus routing | `crates/client_core/src/native_ui.rs`, `source_ui/native.rs` |
| World selection | Imported island selected explicitly and by client default; identical validated source collision on client/server; content identity checked at connection | `crates/shared/src/world.rs`, `content.rs` |
| Source gameplay definitions | Six strict Octets readers, original raw tables, hash-verified catalog, item IDs and blueprint requirements | `crates/game_data/`, `memory/evidence/gameplay-verification.json` |
| Inventory model support | Source-ID stacking and completed-craft transactions roll back on missing materials or insufficient capacity | `crates/game_data/src/lib.rs` |
| Source inventory bodies and state | Ten recovered body decoders, atomic snapshot/delta projection shared by authority/client, player lifecycle, bounded Bevy UI preview and Rust inspection CLI | `memory/implementation-player-inventory-2026-11-11.md` |
| Source player definitions | Verified posture/constructor defaults, ordinary velocity requests, all 62 exact StateIDs and native input magnitude/orientation/Stand Run-Rush classification; active controller remains provisional | `memory/implementation-player-state-2026-11-11.md`, `crates/game_data/src/controller.rs`, `player_state.rs` |
| Source clearance and crouch callback | Exact RootBone standing casts, shared layer-filtered queries with explicit unresolved results, atomic prefab layer/trigger metadata, and ordered OnClickCrouch2 continuation | `memory/implementation-player-clearance-2026-11-12.md`, `crates/game_data/src/posture.rs`, `crates/shared/src/physics/source_clearance.rs` |
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

The first boundary is implemented: ten native message IDs and default XOR
body layouts; snapshot, count, quick-change, capacity, durability, count-alert
and held-instance projection; shared authority/client state and same-frame
Bevy UI updates. The Rust inspector verifies the real catalog, hashes explicit
bodies and refuses existing reports. Validation is recorded in
[player/inventory implementation](../memory/implementation-player-inventory-2026-11-11.md).
No captured source snapshot or original frame/session adapter is available.
Registered players therefore remain uninitialized until a validated snapshot
arrives; no source starter contents or capacity are invented.

The next definitions step is also complete: every StateID name/integer matches
the retained declarations, and the source-only raw Input magnitude/orientation
helpers and Stand Run/Rush classifier preserve the verified native thresholds,
gates, flags and angle writes. [Player state validation](../memory/implementation-player-state-2026-11-11.md)
records nine new tests and the bounded native direction audit. These rules are
available to subsequent simulation work; no active state transitions or physical
sprint speed are inferred from the classification.

The [clearance/crouch continuation](../memory/implementation-player-clearance-2026-11-12.md)
is now implemented and validated with 199 passing workspace tests. The native
audit establishes Move invocation and FSM update order; source requests retain
the exact RootBone origin, cast dimensions, layers and UseGlobal contract.
Shared queries preserve imported prefab layer/trigger metadata and return
explicit unresolved errors for missing layers, selected unrepresented triggers
or initial overlaps. The callback preserves fresh state reads, separate player
targets and audio/switch order without committing a state. CheckCanPaUp remains
distinct and externally supplied. No active source posture binding is inferred.

`SetQuickUseItem`, `SyncBagItem` and `ClearAllBag` decode but application is
explicitly rejected. Their full swap/coffer, consumer and weapon/extra-data
effects remain unresolved. Expiry, use-item effects and original crafting
completion are not provided by the body projection. Static ItemCfg pile limits
are not imposed on incoming state because community overrides are observed.

Continue with these concrete steps:

1. Identify/import the actual player rig and RootBone/HeadTop references;
   recover effective transforms/scaling and retain terrain GameObject layers
   through a verified compatible collision-package import. Current terrain
   and provisional player colliders lack source layers for standing queries.
2. Resolve runtime queriesHitTriggers, required collider/trigger coverage and
   PhysX initial-overlap/contact semantics. Finish CheckCanPaUp's separate
   HeadTop sphere cast and downward ray layer/component conditions.
3. Recover FSM switch arbitration, active modifiers, capsule scaling,
   jump/landing and animation/root-motion behavior, then wire admitted source
   controller/posture state and ordered callback requests into shared physics,
   Bevy UI and audio. Velocity bodies, Move invocation and callback order are
   audited; full physical movement and successful state commits remain open.
4. Finish quick-slot swap/coffer and clear weapon/extra-data dependencies;
   establish the actual SyncBagItem consumer and use-item callbacks.
5. Bind authenticated snapshots/deltas through a recovered source admission
   adapter, then original UI slots/icons/actions and matched source scenarios.

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
