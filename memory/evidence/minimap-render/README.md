# Source BattleMinimap preview

These captures render the verified APK prefab through Bevy 0.20.1,
bevy_egui 0.43.1 and egui 0.36.2 on NVIDIA RTX 4060 Laptop / Vulkan.
They were refreshed after the font atlas, text truncation and binding-evidence
fixes. No Python was used.

[Full 1920×1080 capture](prefab-1920x1080.png),
[detail](prefab-1920x1080-detail.png), and
[graphic report](prefab-1920x1080.json).
The [1280×720 capture](prefab-1280x720.png) uses scale 2/3.
[render.json](render.json) records adapter, canvas inputs, status totals and limitations.

The optional [initialization profile](initialization-visibility.json) adds
[1920×1080](initialization-1920x1080.png) and
[1280×720](initialization-1280x720.png) previews using only 21 source-qualified
GO false overrides from [the native review](../minimap-runtime-review.md).
Each has 103 submitted graphics, 27 inactive, 15 unsupported, two disabled,
two empty and one mask-only. Descendant graphics inherit the template hides.
This does not establish successful native Awake/init: the retained binding
list has 86 entries while native Awake requests 94. Zoom 15 is recorded as
a source default without an inferred texture, UV or scale binding.

The parent canvas inputs are 1920×1080 reference resolution and PPU 100,
with scale min(width/1920,height/1080), recovered from bootstrap source/native
fallback evidence in [the canvas review](../minimap-canvas-review.md).
The effective runtime minimap ancestry and IFix state are still unknown.

Each frame has six prepared atlases, one embedded source font and one
standard mask: 125 submitted graphics, 20 unsupported, two disabled,
two empty and one mask-only. The background map texture is unbound; icons and
placeholders overlap because this is the serialized prefab state. Custom
stencil/fog, line renderers and missing font mappings are reported explicitly.

Rebuild from the repository root, then run with host GPU access:

```sh
cargo rustc --offline -p client_core --example minimap_probe -- -C strip=debuginfo
target/debug/examples/minimap_probe memory/evidence/minimap-render
target/debug/examples/minimap_probe memory/evidence/minimap-render memory/evidence/minimap-render/initialization-visibility.json
```

Capture SHA-256:

| File | SHA-256 |
| --- | --- |
| prefab-1920x1080.png | `a5169680d26784ae7e5dab231d91a880328e8c03b8a723f9c4528b80a0cb5be2` |
| prefab-1280x720.png | `1710ef156fc5cab633a6768c6bd72341ab3916578e4db79093a95b8c4f956649` |
| initialization-1920x1080.png | `076067e292fd66625167e26acc651286fa6c77558ac22047213fde169b035c67` |
| initialization-1280x720.png | `d2d80d18fba1936bdead19166a0e69a437a28e04e3251616a7170de466732a44` |
| initialization-visibility.json | `0d08da97de8c9f8d92611077fe0182c70d21451dd2053ad8d9001c19b459b65c` |
| render.json | `c439458f3f6387a560d4572ef0a1eb2985ffca9aa07199d40db292b44b5a32a1` |

These are local renderer observations, not screenshots of the APK or proof
of pixel equality. Production still uses the hand-built HUD. A connected
Waydroid installation has a different APK digest, but
[payload comparison](../apk-runtime-member-comparison.json) confirms all
9,318 files outside the native library/metadata pair match. The user identifies
lite4.0.apk's native/metadata files as memory dumps of the encrypted counterpart.
See [the runtime identity check](../android-runtime-check.json) and
[native pairing coverage](../minimap-dump-pairing.md). Live state and the recovery
transformation remain outside these captures.
