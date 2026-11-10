# Native Linux minimap verification

Captured on 2026-11-10 with Bevy 0.20.1, bevy_egui 0.43.1 and egui 0.36.2,
using NVIDIA GeForce RTX 4060 Laptop / Vulkan, driver 615.71.09.

The Arch Linux HUD now uses `client_core::source_minimap::NativeMinimap`.
This probe renders that same adapter with controlled simulation inputs.
The asset-backed HUD test also executes the real `draw_interface` system and
checks initialization, wheel zoom, clicking to open the map and editor dragging.
Waydroid is not required. These are native renderer captures, not APK screenshots.

## Retained source and native bindings

Package: `assets/imported_ui/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1/`.
The source hierarchy, RectTransform anchors/pivots, sprite atlas UVs, standard
alpha mask and embedded font come from the verified package. Runtime changes
use qualified bindings and instances; the retained package is not rewritten.

| Item | Binding or policy |
| --- | --- |
| Texture | Retained `Map_map`, 2048×2048, bilinear/clamp |
| Texture identity | `assets/texture/map_map.ab`, `CAB-6f665c88cf2187ff08df209ad5c96ca6`, PathID `-945003921129902140` |
| RawImage target | `assets/ui/battleminimap.ab`, `CAB-88df926bf45d4de159fb33b326042714`, PathID `910110631840687877` |
| Image scale | Native `260 / 2048` on both axes, filling the source 260×260 map area |
| World extent | Shared provisional half extent `5360`, also used by the overview |
| Zoom | Default `15`, limited to `3..=60`; wheel input over the panel changes it |
| UV crop | Unity bottom-left `[u - 0.5/zoom, 1 - v - 0.5/zoom, 1/zoom, 1/zoom]`, with `(u,v)` from the shared top-left overview projection |
| Orientation | Bevy north is `-Z`, east is `+X`; source marker XY uses `[delta_x, -delta_z]` |
| Canvas | Reference 1920×1080, scale `min(width/1920, height/1080)`; editor/DPI scaling is preserved |
| HUD placement | Source 330×330 root inside a 360×363 envelope, including the 32.5-unit header gutter |
| Live data | Player position/yaw, waypoint, resources, structures, NPCs, vehicles, cargo, sound cues, Unity XZ coordinates, height and simulation time |

The texture's original **BattleMapPanel** binding is established. Assigning it
to **BattleMinimap**, the crop/zoom and scale above, marker sizes/categories and
visibility lifecycle are explicit native policies. Original Unity minimap
assignment, live UV/scale/zoom behavior and exact world calibration remain open.

The source view-cone sprite widens upward from its player-facing tip. Its broad
end and the arrow both follow player yaw. Visual review caught an extra half-turn
on the cone; the cardinal/DPI test reproduced the error and now checks both
graphics. The captures in this directory use the corrected orientation.

## Capture cases

Each case has a full PNG, a detail crop and a per-graphic JSON report at both
1920×1080 and 1280×720. [render.json](render.json) records exact state, bindings,
qualified source IDs, GPU metadata and known limitations.

| Case | Input and expected change | Detail at 1920×1080 |
| --- | --- | --- |
| North | Position `(2250,84,-700)`, yaw 0, 10:00, zoom 15 | [North](native-north-1920x1080-detail.png) |
| East | Same position; yaw `-PI/2`; arrow/cone turn east | [East](native-east-1920x1080-detail.png) |
| Moved | Position `(2300,84,-750)`; map and landmark offsets move together | [Moved](native-moved-1920x1080-detail.png) |
| Zoomed night | Moved position; zoom 30, 23:00; tighter crop and night indicator | [Zoom/night](native-zoomed-night-1920x1080-detail.png) |

Ordinary cases submit 17 graphics; zoom/night submits 15 after distant markers
leave range. Each reports 141 inactive graphics and one mask-only graphic, with
zero unsupported active graphics. The probe rejects missing map/player submission,
unsupported active graphics, failed GPU pipelines and nearly empty captures.

All 33 client tests pass, including the three tests requiring extracted assets.
They cover source geometry, all four headings for arrow and cone, movement, DPI,
parent activity, malformed bindings and real HUD input. The shared UI-data suite
has 15 passing tests. Native runner compilation and its `--help` entry point also
pass. This does not constitute a complete game-window or APK comparison.

## Reproduce

From the repository root, with the extracted package present:

```bash
cargo rustc --offline -p client_core --example minimap_probe -- -C strip=debuginfo
target/debug/examples/minimap_probe memory/evidence/native-minimap --native
cargo rustc --offline -p client_runner --bin client_runner -- -C strip=debuginfo
target/debug/client_runner
```

The capture command needs host Vulkan access; the sandbox could not load the
installed NVIDIA driver. Use stripped builds on this nearly full volume.

In the client, F2 opens the HUD editor; M opens the overview. With the pointer
available, scrolling over the minimap zooms and clicking it opens the overview.
The native integration is enabled by default. Other HUD panels still use their
existing painters. Custom Unity fog/stencil/materials, particles and exact
font/color equivalence remain unverified; full APK UI equality is not claimed.
