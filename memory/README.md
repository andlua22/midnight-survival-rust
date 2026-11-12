# Reconstruction handoff

This directory records evidence, decisions, commands and unfinished work for
the Rust/Bevy reconstruction of `export/lite4.0.apk`.

Current continuation (2026-11-12): use [the implementation ledger](../PLAN/source_reconstruction.md),
[MapObject implementation](implementation-mapobjects-2026-11-11.md),
[source imports](source-imports-2026-11-11.md), [gameplay data](gameplay-data-2026-11-11.md)
and [storage audit](storage-2026-11-11.md). The latest step is
[independent terrain collision](implementation-terrain-collision-2026-11-12.md):
schema-4 source table with 404 components, 389 active solids, 12 skipped triggers
and three inactive records. Five omitted Boxes and B-32 MeshCollider 10 are
restored with saved ancestor activity; B-32 components 10/11 retain distinct
identities while sharing one mesh. All 400 render records and 970 retained
payloads remain unchanged; 364 unique collision meshes have a separate archive.
The first import, fresh source comparison, repeat preservation and schema-2/3
compatibility passed, as did 50 extractor tests, strict extractor Clippy and
three selected real-package shared collision/world tests. Workspace tests
passed: 218 passed, 12 ignored; all-target workspace check and strict shared
Clippy passed. Map all-target Clippy used the documented allowance for three
inherited `collapsible_if` warnings. Exact scope is recorded in
[the validation ledger](evidence/terrain-collision-source/README.md).
Its predecessor is retained at
`.local/map-import-backups/1794438051734897066/terrain`.
The preceding [terrain source layers](implementation-terrain-layers-2026-11-12.md) added
schema-3 enrichment with 400 verified GameObject layers, unchanged geometry/
textures and 383 classified shared solid handles. Source collider inventory
found 21 omissions: 12 triggers, six solids with active local flags and three
solids with inactive local flags. Workspace validation passed 208 tests
(11 ignored); extractor validation passed 46 tests, with fresh source/package
verification and idempotent repeat. Its predecessor is retained at
`.local/map-import-backups/1794435897062652886/terrain`.
The preceding [standing clearance and crouch callback](implementation-player-clearance-2026-11-12.md) retains
exact RootBone cast requests, shared queries with explicit unresolved results,
atomic prefab layer/trigger metadata and ordered OnClickCrouch2 continuation.
Workspace validation passed 199 tests (10 ignored), with the separately selected
real supermarket-package test and all-target check also passing.
[Native clearance](evidence/player-clearance-native/README.md) and
[transition evidence](evidence/player-transitions-native/README.md) retain Move
invocation/FSM order, cast constants and callback targets/gates/audio order.
The preceding [player state/input definitions](implementation-player-state-2026-11-11.md)
retain all 62 exact StateIDs, source magnitude/orientation and Stand Run/Rush
classification.
The preceding completed boundary is
[player/inventory implementation](implementation-player-inventory-2026-11-11.md):
strict source bodies, atomic shared state, lifecycle, Bevy UI binding and a
bounded Rust inspector. [Inventory rules](evidence/inventory-rules-source-review.md)
and [player definitions](evidence/player-rules-source-review.md) retain native
provenance and unresolved effects. No authentic source snapshot/session adapter
is supplied; active source movement and animation remain unported. Continue
with the actual rig, source runtime binding and effective RootBone/HeadTop
transforms/player layers, runtime trigger/overlap policy, the distinct PaUp query
and FSM switch arbitration before active posture/UI/audio bindings, then unresolved item-use
rules. The latest disk check found about 126 GiB free; no cleanup was required.
The client now uses native Bevy UI,
a ray-marched atmosphere and shared validated island collision/content identity.
The Bevy/egui notes and older captures below describe the preserved earlier
implementation. Original-runtime 1:1 gameplay and pixel parity remain open.

Start with [the requested handoff](<щоб інші ші знали, про що це.md>).
[apk-reconstruction.md](apk-reconstruction.md) records implementation status
and commands. [map-evidence.md](map-evidence.md) retains map/shader evidence;
[ui-reconstruction.md](ui-reconstruction.md) covers source UI and runtime gaps.

The canonical map is `assets/imported_map/terrain/` (MSMAP003, schema 4), with
a separate MSCOL001 collision archive. Legacy schemas 2/3 remain readable.
The verified UI source package is under `assets/imported_ui/`, keyed by APK
SHA-256 and schema version. The current Linux runtime uses native Bevy UI.
The terrain package includes the omitted Z-01 scene: 400 render instances
and an independent table with 389 active solid components. Its predecessor
and earlier capture evidence remain available. The current desktop preset
retains directional shadow maps and disables the contact effect that caused
prefab wall dots. The final
[prefab](evidence/prefab-shadow-fixed-bevy/README.md),
[environment](evidence/environment-shadow-fixed-bevy/README.md) and
[game-window startup](evidence/desktop-shadow-fixed-bevy/README.md) evidence
includes the native Tab routing correction. The older egui implementation
reports below are historical.

Compact reports: [terrain](evidence/terrain-verification.json),
[UI](evidence/ui-verification.json), and
[GPU captures](evidence/terrain-render/render.json). Do not print the enormous
`source_ui.json`; use the compact report or bounded package queries.

[UI repeatability](evidence/ui-reproducibility.json) records matching fresh
imports and a file inventory without stale files. [Archive index](evidence/archive-index.md)
locates preserved candidates, earlier reports and code/package backups.

The earlier source renderer used `crates/ui_data` and the client's egui backend.
[Prefab minimap captures](evidence/minimap-render/render.json) record
1920×1080 and 1280×720 authored and initialization-visibility previews with
explicit canvas inputs. The optional profile applies 21 source-backed template
hides. These historical previews keep their original scope and evidence.

[NativeMinimap](../crates/client_core/src/source_minimap.rs) supplies the live
HUD texture, UV crop, scale, player/marker instances, coordinates and simulation
time. Wheel scrolling over the panel changes zoom; clicking opens the map;
F2 opens the HUD editor. It runs without Waydroid. The retained `Map_map` texture
is bound with explicit native policies: zoom 15 (3–60), image scale 260/2048,
and the shared provisional overview half extent 5360. Original Unity minimap
assignment/calibration and APK pixel equality remain unverified. The native
probe produced [eight native captures](evidence/native-minimap/render.json)
for heading, movement and zoom/night states at both resolutions on RTX 4060
Laptop/Vulkan. [Native capture notes](evidence/native-minimap/README.md) include
the exact binding policies, visual review and controls. Commands and validation are in
[apk-reconstruction.md](apk-reconstruction.md).

[Canvas/calibration evidence](evidence/minimap-canvas-review.md) and
[font coverage](evidence/minimap-font-coverage.json) separate recovered source
values from missing runtime state. [Native initialization](evidence/minimap-runtime-review.md)
records the template hides, zoom default and unresolved binding entries.
[Android runtime identity](evidence/android-runtime-check.json),
[APK payload comparison](evidence/apk-runtime-member-comparison.json), and
[native dump pairing](evidence/minimap-dump-pairing.md) relate the recovered
analysis APK to its encrypted Waydroid runtime counterpart. All 9,318 file
payloads outside native code/metadata match; the user identifies the two
recovered files inside lite4.0.apk as memory dumps of the encrypted game.

The user writes Ukrainian and wants English responses. Write new tools in
Rust and reuse installed Unity/REA utilities and C# evidence. No Python was
executed during this work; keep the extraction and investigation in Rust.

Preserve user changes and old assets. Record static observations, Bevy renderer
validation and unverified APK runtime behavior separately. REA is already
installed; do not repeat setup to reconnect absent live MCP tools.
