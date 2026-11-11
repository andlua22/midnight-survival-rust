# Reconstruction handoff

This directory records evidence, decisions, commands and unfinished work for
the Rust/Bevy reconstruction of `export/lite4.0.apk`.

Current continuation (2026-11-11): use [the implementation ledger](../PLAN/source_reconstruction.md),
[MapObject implementation](implementation-mapobjects-2026-11-11.md),
[source imports](source-imports-2026-11-11.md), [gameplay data](gameplay-data-2026-11-11.md)
and [storage audit](storage-2026-11-11.md). The client now uses native Bevy UI,
a ray-marched atmosphere and shared validated island collision/content identity.
The Bevy/egui notes and older captures below describe the preserved earlier
implementation. Original-runtime 1:1 gameplay and pixel parity remain open.

Start with [the requested handoff](<щоб інші ші знали, про що це.md>).
[apk-reconstruction.md](apk-reconstruction.md) records implementation status
and commands. [map-evidence.md](map-evidence.md) retains map/shader evidence;
[ui-reconstruction.md](ui-reconstruction.md) covers source UI and runtime gaps.

The canonical map is `assets/imported_map/terrain/` (MSMAP003, schema 2).
The verified UI source package is under `assets/imported_ui/`, keyed by APK
SHA-256 and schema version. The current Linux runtime uses native Bevy UI.
The terrain package now includes the omitted Z-01 scene: 400 render instances
and 383 solid colliders. Its predecessor and earlier capture evidence remain
available. The older egui implementation reports below are historical.

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
executed during this work; do not use it without a concrete missing capability.

Preserve user changes and old assets. Record static observations, Bevy renderer
validation and unverified APK runtime behavior separately. REA is already
installed; do not repeat setup to reconnect absent live MCP tools.
