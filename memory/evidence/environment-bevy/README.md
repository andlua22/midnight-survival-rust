# Bevy environment validation, 2026-11-11

These Linux captures use Bevy 0.20.1 on NVIDIA GeForce RTX 4060 Laptop /
Vulkan, driver 615.71.09. They validate the new production environment plugin
and canonical terrain material with controlled camera and lighting inputs.
They are not screenshots from the original game or a pixel-parity comparison.

The flat cloud canopy and cirrus meshes have been replaced at runtime by a
half-resolution cloud pass that integrates a 64³ density texture through a
world-space layer. It uses Beer–Lambert attenuation, sun-ray self-shadowing,
world-anchored wind, and opaque depth clipping. The composite draws the sky
gradient and angular sun disc at infinity before tonemapping. Sunlight and
cloud scattering share one sun direction; direct sunlight is zero at night.
The cloud pass supports both single-sample depth and MSAA depth, with bounded
12/24/48 view steps and at most eight light steps.

Canonical terrain now participates in the depth prepass. Its source splat,
normal, tangent and mip-chain math is unchanged. Directional shadow maps and
contact shadows remain enabled. The source terrain receives a reference
caster's shadow. The production normal bias now uses Bevy's default 1.8;
the previous 0.60 produced more low-sun self-shadow speckling on slopes.
First-person held tools and placement ghosts do not cast or receive shadows.

## Captures and assertions

| Capture | What was checked |
| --- | --- |
| [Noon, 24 steps](noon-24.png) | Cloud shader, source terrain and normal depth path |
| [Noon, clouds disabled](noon-disabled.png) | Cloud toggle changes 403,390 sky pixels and zero foreground pixels with a one-byte tolerance |
| [Noon, 48 steps and MSAA4](noon-48-msaa.png) | Multisampled depth path and bounded higher quality |
| [Night, 12 steps and MSAA4](night-12-msaa.png) | Mean image luminance 15.27, versus 193.55 at noon; direct sunlight is zero |
| [Clear sunrise](sunrise-clear.png) | Sun-disc center RGB 250/249/248 versus adjacent sky 128/159/189 |
| [Inside the cloud layer](inside-cloud-48.png) | Camera inside the density slab still integrates toward the exit |
| [Shadow on](shadow-on.png), [shadow off](shadow-off.png) | Source terrain receives the reference cuboid shadow; 5,522 lower-image pixels darken by more than 30 summed RGB units |
| [Previous lower bias](sunrise-low-bias.png) | Retained comparison with normal bias 0.60; all other controlled settings match |

All eight capture cases completed without GPU pipeline errors. The probe also
rejects nearly empty captures and missing terrain textures. The report retains
the original APK SHA-256 and the count of 399 source render records.

[render.json](render.json) includes 20 warmed frame samples per view. This
run measured mean synchronous frame times of 6.22 ms for 24-step noon clouds,
5.89 ms with clouds disabled, and 6.88 ms for 48 steps with MSAA4. These are
`app.update` plus CPU work and a synchronous GPU drain in the dev capture
harness. They are not isolated cloud GPU timings or an in-game FPS benchmark.
[before-bias-render.json](before-bias-render.json) retains the earlier run.

## Reproduce

```bash
cargo build --offline -p client_core --example environment_probe
target/debug/examples/environment_probe memory/evidence/environment-bevy
```

Execution requires host Vulkan access: the sandbox could not load the
installed NVIDIA driver. The capture harness uses a 1280×720 target,
640×360 cloud attachment and a 180-second deadline.

## Remaining source-fidelity work

The source scene's serialized default material is `Skybox/Procedural`, with
no texture bindings. Runtime overrides by `DayNightSystem` remain possible.
Original day/night Animator curves, sky/lighting colors, source shadow flags,
lightmaps, source tonemapping and cloud shadows on terrain are not reconstructed.
The implemented cloud light march provides cloud self-shadowing only.
Low-sun steep terrain still has residual artifacts requiring a source-material
and renderer-metadata comparison; this test does not establish that every
shadow problem is resolved. The imported terrain package still lacks source
renderer cast/receive/lightmap metadata, and one terrain mesh has no tangents.
