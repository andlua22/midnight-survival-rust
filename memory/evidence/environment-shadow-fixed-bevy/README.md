Linux/Vulkan acceptance captured on 2026-11-11 with Bevy 0.20.1 and an NVIDIA GeForce RTX 4060 Laptop GPU after the shared desktop contact-shadow correction. All ten cases passed within the bounded 180-second probe. Original environment captures remain in [environment-bevy](../environment-bevy/) and [environment-z01-bevy](../environment-z01-bevy/).

The main camera, prefab probe and environment probe use `environment::DESKTOP_CONTACT_SHADOWS = None`. Directional shadow maps remain enabled with Bevy's default directional normal bias (approximately 1.8). `CloudSettings` still requires the depth prepass, so removing the contact-shadow component preserves depth rejection for volumetric clouds.

| Captures | Acceptance result |
|---|---|
| [noon-24.png](noon-24.png), [noon-disabled.png](noon-disabled.png) | Toggling clouds changes 403,344 sky pixels and zero foreground pixels |
| [noon-48-msaa.png](noon-48-msaa.png) | 48 cloud steps and 4× MSAA render successfully |
| [night-12-msaa.png](night-12-msaa.png) | Night mean luminance 15.27 versus noon 193.56; 12 cloud steps and 4× MSAA |
| [sunrise-clear.png](sunrise-clear.png) | Sun-disc center RGB `[250,249,248]` versus adjacent sky `[128,159,189]` |
| [inside-cloud-48.png](inside-cloud-48.png) | Camera inside the cloud volume renders successfully with 48 steps |
| [shadow-on.png](shadow-on.png), [shadow-off.png](shadow-off.png) | Directional shadow maps darken 5,522 reference-ground pixels |
| [z01-overview.png](z01-overview.png), [z01-hidden.png](z01-hidden.png) | Hiding verified Z-01 terrain changes 118,602 pixels |

[render.json](render.json) pins source APK SHA-256 `2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`, 400 imported terrain records, and Z-01 source mesh `Z_01_ID_639504_`. Target images are 1280×720, the cloud pass is 640×360, and the density volume is 64³. The only runtime warning states that `Terrain_jichang` has no source tangents and therefore uses geometric normals.

The recorded frame measurements cover 20 warmed synchronous `App::update` calls plus GPU completion per view on the development build. They include CPU work and do not measure in-game FPS or isolate GPU-pass cost.

These cases validate Bevy rendering, depth rejection, shadow maps and source terrain visibility. Original-runtime pixel parity, terrain cloud shadows, and original DayNightSystem curves remain unverified.

After building `environment_probe`, reproduce to a new directory with host NVIDIA access:

```sh
ulimit -c 0
timeout 180s target/debug/examples/environment_probe /tmp/midnight-environment-shadow-acceptance
```
