Rejected screen-space contact-shadow presets captured on Linux/Vulkan with Bevy 0.20.1 and an NVIDIA GeForce RTX 4060 Laptop GPU. The original PNGs and [render.json](render.json) are retained as diagnostic evidence.

All enabled presets still produce visible dotted self-shadows on planar source surfaces. The finer preset looks quiet in the earlier narrow measurement windows but remains noisy on the front wall, crates and slab edges. [affected-wall-comparison.json](affected-wall-comparison.json) recomputes the measurements from the saved PNGs using the affected front-wall regions: near x `[637,667)`, y `[415,454)` (1,170 pixels), and close x `[625,720)`, y `[320,350)` (2,850 pixels).

| Preset | Steps | Thickness | Ray length | Near darkened pixels, first/repeat | Near repeat changed | Close darkened pixels, first/repeat | Close repeat changed |
|---|---:|---:|---:|---:|---:|---:|---:|
| Bevy default | 16 | 0.1 | 0.3 | 192 / — | — | 307 / — | — |
| Thin | 32 | 0.01 | 0.1 | 279 / 273 | 561 | 301 / 296 | 597 |
| Finer | 32 | 0.001 | 0.1 | 19 / 21 | 43 | 101 / 99 | 207 |

A pixel counts as darkened when `sum(current RGB) + 30 < sum(contact-off RGB)` at the same camera pose. A repeated pixel counts as changed when any RGB channel differs by more than 8. Darkening alone can include intended occlusion; the front-wall dots and their frame variation establish the visual failure here. These measurements do not compare unrelated camera views.

The accepted shared desktop policy disables the optional contact-shadow pass while retaining directional shadow maps. [Accepted captures](../prefab-shadow-fixed-bevy/README.md) have zero changed pixels in both repeated front-wall regions and still show directional cast shadows.

To reproduce diagnostics without overwriting this evidence, build `prefab_probe` and run:

```sh
ulimit -c 0
timeout 180s target/debug/examples/prefab_probe /tmp/midnight-prefab-contact-review --review-contact-shadows
```
