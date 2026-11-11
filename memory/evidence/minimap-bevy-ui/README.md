# Native Bevy UI minimap captures

Captured 2026-11-11 using Bevy 0.20.1, the production
`native_ui::update_source_frame` presenter and retained APK-derived UI assets.
Linux: NVIDIA RTX 4060 Laptop GPU, Vulkan, driver 615.71.09.
Final captures use the segmentation-enabled client build and omit zero-area
source Text before font submission.

`render.json` records 12 passing scenarios: north, east, movement and zoom/night
at 1920×1080 DPI 1, 1280×720 DPI 1 and 1920×1080 DPI 1.5. Every frame submits the
map and player, has no unsupported active graphics and has 311–413 distinct
quantized RGB buckets. Full frames, detail crops and per-graphic reports retain
the exact bindings and qualified source identities. East/north heading changes
also pass a pixel-difference check inside player-arrow bounds: 296 pixels at
1920×1080, 137 at 1280×720 and 296 at 1920×1080 DPI 1.5.

Source atlas sampling and mip chains stay on the GPU through native Bevy
`MaterialNode`. Embedded font payloads use Bevy Text. Binary source mask coverage
inside each text rectangle is preserved with disjoint native overflow clips;
the coordinate label uses four clips at 1920×1080 DPI 1. Coordinate-image updates
refresh cached material bind groups so GPU views follow movement and rotation.
The capture camera uses
`IsDefaultUiCamera`, and assets resolve from the project root.

These captures validate the current native presentation and explicit simulation
inputs. Original crop/zoom calibration, runtime setters, Unity font metrics,
custom stencil/fog behavior and pixel equality remain unverified. Historical
egui captures under `../native-minimap/` and `../minimap-render/` are separate
dated evidence.

Independent representative-frame review passes heading, recentering, zoom,
moon/time, coordinate/height text and masks at the covered resolutions/DPI.
Bevy text baselines sit approximately ten pixels lower at 1920 and the yellow
player marker is brighter than the historical renderer. Original Unity font
and color behavior remains an explicit parity limit.

Reproduce with:

```sh
cargo build --offline -p client_core --example minimap_probe
target/debug/examples/minimap_probe memory/evidence/minimap-bevy-ui --native
```

The host Vulkan driver must be accessible for the render app. The probe retains
failed-frame diagnostics when its active-graphic acceptance fails.
