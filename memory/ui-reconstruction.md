# Native Bevy UI reconstruction

Updated: 2026-11-11. Source: `export/lite4.0.apk`, SHA-256
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.

The current Linux client uses Bevy 0.20.1 UI. The egui rendering dependency,
context passes and painters have been removed. Source images are presented with
Bevy `MaterialNode`, text with Bevy Text, controls with native `Button`,
`TextInput` and `ScrollArea`. Android builds and original Android runtime
observation are outside the current implementation stage. Static extraction
uses Rust; no Python or Waydroid is required.

## Verified source packages

The active nine-panel package is:

`assets/imported_ui/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1-panels-e1739e152490099d/`.

Its `source_ui.json` SHA-256 is
`ee1df91b8dbf610b108a55fd5530044a09104dd803d7ffe674db258181a3e883`.
Read [the expanded verification report](evidence/ui-expanded-verification.json)
for bounded inventory and verification results. The original three-panel
package remains intact, including its historical reproducibility evidence.

| Panel | Source nodes | Images | Texts | Masks | Current native use |
| --- | ---: | ---: | ---: | ---: | --- |
| BattleMinimap | 225 | 133 | 14 | 2 | Live player, map, waypoint and simulation markers |
| BattlePanel | 1,817 | 905 | 259 | 16 | Selected HUD groups, vitals, hotbar and source actions |
| BattleMapPanel | 416 | 214 | 73 | 14 | Map image, player/waypoint, zoom, pan, close and clear |
| BagAndBuildPanel | 1,685 | 932 | 288 | 11 | Bag page shell; inactive item/detail/crafting templates hidden |
| BattleLoadingPanel | 29 | 14 | 3 | 1 | Source layout with native world-loading state |
| MainPanel | 358 | 178 | 67 | 18 | Selected menu chrome and return/settings actions |
| SettingPanel | 2,836 | 1,169 | 606 | 11 | Selected settings chrome; back/editor actions |
| DeathPanel | 150 | 76 | 21 | 4 | Loaded; original death transition pending |
| BluePrintBuildPanel | 113 | 38 | 40 | 1 | Loaded; original blueprint transition pending |

The verified closure includes 70 bundles, 30,629 source objects, 555 sprites and
166 decoded textures. Verification checks 62,236 files / 831,070,387 bytes.
There are zero import errors and 90 explicit warnings: 80 unresolved resource
references, eight sprite preview variants, one font without embedded outlines,
and one empty source font atlas. Hierarchy completeness is verified; graphics
completeness and runtime equivalence are not.

All selected source nodes serialize as active. That is authored prefab state,
not evidence that all popups, templates, effects or account controls should be
visible during gameplay. The native bindings select an explicit current state.
Qualified `(bundle, CAB, signed PathID)` identities, references, source bytes,
hierarchy order, transforms, material fields and font settings remain retained.

## Renderer and binding contracts

`crates/ui_data` owns the shared package DTOs, package access, RectTransform
layout and sprite geometry. `client_core::source_ui::SourcePanelRenderer` loads
verified source textures/fonts and returns `PanelFrame` plus per-graphic reports.
`native_ui::update_source_frame` is the shared Bevy UI presentation path used by
both the game and the capture probe.

Source atlas color sampling stays on the GPU. Full source mip chains, color
space and sampler metadata are retained in Bevy `Image` assets. Source sprite
geometry is rasterized into RGBA32Float UV/coverage textures; these contain
coordinates and stencil coverage, rather than CPU-painted atlas colors.
Standard Mask alpha coverage uses threshold 0.001 at physical pixel centers.
RectMask2D uses the source rectangle. Nested source transforms and masks are
preserved. Text-mask coverage inside source text bounds is decomposed into
disjoint native overflow-clip rectangles. Identical horizontal spans merge
across pixel rows; each copy keeps the same Bevy text layout and rotation.
Coverage budgets are 16 million pixels per viewport, four million pixels per
graphic and 4,096 clip rectangles per text. Excessive fragmentation is reported
unsupported. Retired live-instance nodes, text clips and coordinate/material
assets are released.

Bindings accept qualified targets for RawImage texture/UV, text, visibility,
anchored position, rectangle anchors, scale, rotation, sprite, tint, fill amount
and repeated source instances. Invalid target types, duplicate instance keys,
invalid numeric input and invalid PPU are rejected. Binding evidence uses entry
arrays so qualified IDs remain structured JSON objects. Source component order
and runtime instance identities remain visible in each report.

Embedded font payloads are supplied to Bevy Text. The retained Adobe font has
12,649,568 outline bytes. [Historical independent font coverage](evidence/minimap-font-coverage.json)
records missing nominal mappings for seven source characters. Bevy rendering
is not proof of Unity TextGenerator metrics, font fallback or text rasterization.
Zero-area source Text and nonpositive authored font sizes are omitted before
font submission. The Linux client enables Bevy's cached
`complex_script_segmentation` feature for source Chinese/Japanese text.

## Native runtime behavior

`NativeUiPlugin` loads panels in Startup, processes native actions after Bevy
picking in PreUpdate, and updates source panels in Update. Desktop input runs
after the native UI boundary. Modal screens, hovered UI buttons and focused
text fields block gameplay input. Editor-mode source hit targets are disabled
so dragging a widget does not activate a gameplay button.

The native menu retains server address/invitation input and Connect/Disconnect
through the existing Online resource. Bevy `TextInput` supplies Unicode editing,
selection, clipboard and IME behavior. Fields are bounded and single-line.
Editor Save/Cancel, persisted layout, storage error feedback, presets, reanchoring,
scale, opacity, visibility and Escape cancellation remain available. Native
buttons/text fields participate in tab navigation. The developer overlay keeps
fly, fog, HUD visibility and cloud-quality controls. Obsolete egui-only GM tab,
item-spawn selector, target-coordinate and timed-loading-tip state was removed;
source equivalents for those prototype tools are not claimed.

Source HUD groups are transformed using their resolved source bounds and parent
transform inverses. The editor modifies health bars, hotbar, joystick, fire,
aim, jump, crouch and inventory groups while preserving their internal geometry.
The source reload widget has not been identified; it is an explicit editor gap.
The minimap keeps its source envelope and applies saved placement/opacity.
Editor hit rectangles follow the rendered source groups.

Vitals text targets use full source paths, including `txt_mybloodNum`,
`txt_hunger` and `txt_bodyWater`. Filled images/slider anchors reflect current
native simulation values. Authored hotbar count, icon and durability placeholders
are hidden or replaced with native state. Item-to-source-sprite selection remains
unresolved, so guessed icons are not substituted. The source has seven hotbar
entries; the prototype has six. The seventh source entry has no invented native
slot action. The imported-island runtime does not infer original bag capacity
from prototype six/eighteen-slot inventories.

The inventory sidebar reads `LoadedWorld.gameplay` and presents bounded pages
of original item IDs/names and recovered blueprint requirements. It displays
DrawingCfg target IDs, output counts, material IDs/counts, time, workbench/level
and mode flags. This is a read-only catalog; it does not infer unlock eligibility
or map source IDs to prototype ItemKind labels. The separate CraftRecipeCfg table
contains only the special battery row; it is not the ordinary full crafting list.

Loading percentage follows native world/package readiness, without a fabricated
progress timer. Source account, equipment, selected-item, crafting and service
states that have not been reconstructed are hidden where explicitly identified.
Remaining source screens and graphics are reported without claiming complete
runtime coverage. Technical reconstruction notices belong in documentation,
logs or developer diagnostics rather than ordinary play screens.

## Map policies and evidence boundaries

The retained `Map_map` identity is `assets/texture/map_map.ab`, CAB
`CAB-6f665c88cf2187ff08df209ad5c96ca6`, PathID `-945003921129902140`.
Its assignment to BattleMapPanel is source-backed. Assigning it to the minimap
RawImage `910110631840687877` is an explicit native policy.

The minimap uses player-centered RawImage UVs, default zoom 15, native limits
3–60 and a 260-pixel source mask. The 360×363 envelope includes the header gutter.
North is Bevy -Z and east +X. Unity +Z is reflected once into Bevy -Z. Texture
crop and marker projection share `shared::map_projection` and the provisional
world overview half extent 5360. Exact original world/image calibration remains
unverified. Required ancestors are activated for native template instances;
far waypoints/sound markers clamp to the edge and inactive vehicle markers retire.
No fixed cargo marker is fabricated.

The large map uses the retained 8640×8640 source rectangle with native overview
scale `0.125 * zoom`, zoom limits 0.5–4, right-button pan and inverse-transform
click-to-waypoint. Native player/waypoint icon sizes and lifecycle are policy,
not recovered original setters. Selected source close, zoom and clear buttons
activate native actions. Other players/service templates are hidden until their
state exists. M toggles the map; scrolling over the minimap zooms it; clicking
it opens the map. F2 opens the HUD editor.

[Canvas evidence](evidence/minimap-canvas-review.md) establishes bootstrap
1920×1080 / PPU 100 scaling and recovered scalar formulas. It does not establish
the instantiated minimap ancestor chain, IFix state or final effective map scale.
[Native method review](evidence/minimap-runtime-review.md) and
[dump pairing](evidence/minimap-dump-pairing.md) retain exact static boundaries.
Recovered `export/SC.UI/*.cs` is pseudo-C# with incomplete decompilation, rather
than original compilable Unity source. Historical Android member comparisons
remain read-only evidence and are not a dependency of current Linux work.

## Validation and next work

The native minimap probe uses the same Bevy UI material/text presenter as the
game. It covers north/east headings, movement, waypoint/resource/sound markers,
zoom/night, 1920×1080, 1280×720 and DPI 1.5. Reports retain source/instance IDs,
bindings, GPU metadata and preview caveats. It rejects a missing map/player,
unsupported active graphics and captures without enough texture variation.

The native GPU probe completed all 12 captures on Linux / NVIDIA RTX 4060 Laptop
/ Vulkan, driver 615.71.09. Final captures were rerun after enabling source CJK
segmentation and omitting zero-area Text. [The current report](evidence/minimap-bevy-ui/render.json)
records 311–413 distinct RGB buckets per frame and no unsupported active graphics.
Map, player, coordinate/height text, clock and marker bindings are submitted.
The coordinate text uses four clip rectangles at 1920×1080 DPI 1. This validates
the native renderer and explicit simulation bindings, without proving original
Unity pixel equality or behavior.

Seven focused `native_` tests passed, including the retained asset check, exact
binary text-mask holes/nested intersections, clip fragmentation budget, modal
input boundary and deferred overlay creation/retirement across Loading, Menu,
Inventory, Map, Editor and developer controls. The cached material test checks
invalidation when UV/coverage pixels change and avoids invalidation for unchanged
pixels. Menu fields retain bounded single-line editing state. The strict client
library Clippy check passes with `--lib --no-deps -- -D warnings`; broader
workspace/example lint baselines are recorded separately. These tests do not
exercise platform IME/clipboard
interaction. Full-client Linux launch validation is recorded separately.

The capture harness explicitly uses the project's asset root and marks its
headless Camera2d as `IsDefaultUiCamera`. The full-client smoke test exposed a
duplicate Node in a label bundle; that was removed and the overlay regression
test now exercises deferred bundle insertion.

The visual review also caught stale player heading after coordinate-image
updates: replacing a GPU image view did not refresh the cached UiMaterial bind
group. Coordinate changes now invalidate the material. The probe explicitly
checks east/north pixel differences inside player bounds: 296 changed pixels
at 1920×1080 and 137 at 1280×720, including the 1920 DPI 1.5 run.

Independent visual review passes heading, movement/recentering, zoom, moon/time,
height/coordinates and map masks in representative resolution/DPI captures.
Against the historical renderer, Bevy text baselines are approximately ten
pixels lower at 1920 and the yellow player marker is brighter. These are retained
migration parity limits; neither capture set proves original Unity text/color
equivalence.

Historical egui captures under `evidence/minimap-render/` and
`evidence/native-minimap/` remain dated evidence of the former renderer. They are
not baselines or passing checks for the current Bevy UI backend. New captures use
`evidence/minimap-bevy-ui/` so the historical artifacts remain intact.

Useful checks after a material or interaction change:

```sh
cargo check --offline -p client_core --all-targets
cargo test --offline -p client_core native_ui::tests
cargo test --offline -p client_core native_bevy_ui_asset -- --ignored
cargo run --offline -p client_core --example minimap_probe -- memory/evidence/minimap-bevy-ui --native
```

The ignored asset test explicitly requires the retained APK-derived package; it
checks native map/player submission and rejection of foreign binding targets.
The modal test checks gameplay boundaries and Escape/editor cancellation.
The overlay test checks native screen controls and retirement. Source geometry
and projection tests retain their separate contracts.

Next work is source inventory/item-use/equipment rules and source icon selection;
full HUD/compass/ammo bindings and original screen transitions; settings state;
death/blueprint screens; source particles, custom fog/stencil/materials and lines;
additional sprite variants; and Unity rich text, best-fit, font styles,
line spacing and vertical alignment. Pixel equality and complete
UI behavior remain unverified. Use matched static evidence and deterministic
Bevy captures while continuing Linux-first development.
