Linux/Vulkan render acceptance captured on 2026-11-11 with Bevy 0.20.1 and an NVIDIA GeForce RTX 4060 Laptop GPU. All seven views passed the bounded 180-second probe. Images are 1280×720 with 4× MSAA.

The production camera and both render probes share `environment::DESKTOP_CONTACT_SHADOWS = None`. Directional shadow maps remain enabled. Disabling the optional screen-space pass removes the frame-varying dotted self-shadows on the verified supermarket walls, crates and slab edges.

| Capture | Camera distance | Visible source parts | Purpose |
|---|---:|---:|---|
| [near.png](near.png) | 45 | 296 | Accepted overview |
| [near-repeat.png](near-repeat.png) | 45 | 296 | Wall stability across later frames |
| [near-shadows-off.png](near-shadows-off.png) | 45 | 296 | Directional shadow-map control |
| [close.png](close.png) | 22 | 296 | Accepted wall, crates and air-conditioner view |
| [close-repeat.png](close-repeat.png) | 22 | 296 | Close wall stability across later frames |
| [opposite.png](opposite.png) | 45 | 296 | Opposite-facing geometry |
| [far-lod.png](far-lod.png) | 180 | 2 | Source LOD selection |

[render.json](render.json) records zero changed pixels in both repeated wall regions, using an absolute difference greater than 8 in any RGB channel. The front-wall regions are near `(637,415,30,39)` and close `(625,320,95,30)`, expressed as x, y, width, height. Disabling directional shadow maps changes 10,819 lower-image pixels from shadowed to brighter, using an RGB-sum difference greater than 30. Cast shadows from the source prefab remain visible on the diagnostic stage.

Every view also verifies all 303 source node poses against the independently verified package transforms within 0.002, and preserves dimensions and mip counts for all 12 source textures. Strict preparation rejects the unsupported glass/cubemap part before mutating Bevy asset collections; a partial preview cannot enter the runtime placement path. The prepared preview contains 297 supported parts and one explicitly unsupported part.

The original artifact-bearing captures remain in [prefab-bevy](../prefab-bevy/), [prefab-shadow-review](../prefab-shadow-review/) and [prefab-contact-presets](../prefab-contact-presets/). The preset review documents why thinner contact rays were rejected.

This is a standalone diffuse preview of source prefab geometry on an explicit diagnostic stage. It authenticates no map placement. Original shader execution, unsupported glass, and baked/probe lighting are not reproduced; non-fading source LOD thresholds use one perspective camera and native LOD bias 1, while the source QualitySettings bias remains unknown.

After building `prefab_probe`, reproduce to a new directory with host NVIDIA access:

```sh
ulimit -c 0
timeout 180s target/debug/examples/prefab_probe /tmp/midnight-prefab-shadow-acceptance
```
