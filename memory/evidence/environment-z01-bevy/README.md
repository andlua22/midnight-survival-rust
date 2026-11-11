# Restored terrain and environment GPU validation — 2026-11-11

The production Bevy 0.20.1 terrain/material/environment code passed all ten
bounded capture scenarios on Linux with NVIDIA GeForce RTX 4060 Laptop GPU,
Vulkan driver 615.71.09. [Machine-readable results](render.json) retain camera,
source APK identity, sample count, image statistics and synchronized timings.
Earlier captures remain in `../environment-bevy/`.

The canonical package now has 400 source mesh instances and 383 enabled solid
terrain colliders. The added `Z_01_ID_639504_` scene is at source-derived Bevy
bounds X[-3639,-3127], Y[-7.101078,26.990854], Z[-1269,-757].

- [Z-01 overview](z01-overview.png) versus [Z-01 hidden](z01-hidden.png):
  toggling only the new terrain mesh changes **118,879 pixels**. The adjacent
  imported military terrain remains visible in both views. These are diagnostic
  terrain views; water and source scene objects are not reconstructed here.
- Cloud toggling changes **403,390 sky pixels** and **zero opaque foreground
  pixels**. Single-sample and four-sample MSAA cases pass.
- The reference caster darkens **5,522 ground pixels** with shadows enabled.
- Noon mean luminance is **193.546**, night **15.272**. Sun-disc contrast and
  a camera inside the cloud volume pass their assertions.

Only the inherited airport missing-tangent warning was emitted: source mesh
absence is retained and the renderer uses geometric normals for that record.
There were no GPU pipeline failures. Timings include synchronous CPU/GPU drain;
they are not in-game FPS or isolated GPU-pass measurements.

Reproduce from the repository root on the host Vulkan driver:

```sh
cargo build --offline --locked -p client_core --example environment_probe
ulimit -c 0
target/debug/examples/environment_probe .local/captures/environment-z01
```

Source import details, hashes and predecessor path are in
[the terrain handoff](../../terrain-z01-2026-11-11.md). Z-01's terrain role and
coordinates are verified. Its identity as the particular island described by
the user, original lighting/shader execution and source-runtime pixel parity
still need a matched original-game view.
