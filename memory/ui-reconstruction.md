# APK UI reconstruction

Updated: 2026-11-10. Target: `export/lite4.0.apk`, SHA-256
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.

All three selected prefabs are now extracted directly from the APK in Rust.
This is verified static source data. The client still renders a hand-built
egui HUD. Communicate in English; use Rust and installed tools. No Python ran.

## Current source package

Root:
`assets/imported_ui/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1/`.
Schema-1 manifest: `source_ui.json`. Layouts: `panels/*.json`.
Read `evidence/ui-verification.json` instead of dumping the enormous manifest.

| Prefab | Nodes | Components | Images | RawImages | Texts | Disabled components | Canvases | CanvasScalers | Masks |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| BattleMinimap | 225 | 593 | 133 | 1 | 14 | 2 | 4 | 0 | 2 |
| BattlePanel | 1817 | 5213 | 905 | 4 | 259 | 36 | 16 | 0 | 16 |
| BattleMapPanel | 416 | 1181 | 214 | 7 | 73 | 4 | 1 | 0 | 14 |

All 2,458 source nodes have `active_self=true`, read from actual GameObjects
without defaults. This does not establish runtime visibility: scripts and
parent canvases can change it. The 42 disabled components remain disabled.
Custom/native components without a projection retain complete fields/bytes;
Image/Text totals therefore differ from the old exporter.

The 36-bundle dependency closure retains 10,047 source objects, 322 sprites,
88 decoded Texture2D resources, one deferred font atlas and two fonts.
Texture mip counts: `1:83, 7:3, 8:1, 9:1`. `FogLightMask`, missed by the old
export, was recovered from the actual APK.

The data preserves qualified `(bundle, CAB, signed PathID)` identities, PPtr
FileIDs and explicit resolution states; hierarchy/component order; quaternions,
anchors/pivots; zero fill values; graphic/material/raycast/mask settings;
canvas metadata; font/text settings; atlas packing, trim, borders and sprite
mesh data. Large integers and non-finite floats use explicit JSON encodings;
original binary objects remain authoritative for float width/NaN payloads.

Imports build fresh staging directories, serialize the dependency index with
stable key order and verify before publishing. Prior packages are retained
under `.local/ui-import-backups/`; failed reruns preserve their predecessor.
Legacy `assets/ui/` and the current runtime HUD remain available.

## Verified extraction and remaining gaps

The current import completes with **zero errors and 61 warnings**:

- 56 unresolved Unity built-in/default-resource references.
- Three sprite previews requiring source mesh geometry or trim handling.
- One Font without embedded outlines (`hitFont`); glyph/material data is retained.
- One explicitly empty source `Font Texture`, retained as deferred.

Reference counts: 40,071 resolved, 11,385 null and 56 unresolved.
Hierarchy completeness is true; graphics completeness, reference coverage
and runtime equivalence are false. Success verifies the stated static scope.

`ADOBEHEITISTD-REGULAR` has 12,649,568 embedded outline bytes. A bounded Rust
Font reader uses its shipped type tree and retains the exact original object
bytes while restoring the complete font array. The parser's general
one-million-element array limit was not raised.

The touched empty font Texture2D is in `assets/fonts/fonts.ab`, CAB
`CAB-096add6cdbb7663c007ed458ab0c7475`, PathID `6707847669886887298`.
Its dimensions, image count and complete image size are zero; format is RGBA32
(4), mip count is one, image bytes are empty and no stream exists. Another
empty object, PathID `1493665959590621418`, is in that bundle's inventory
outside the touched graphics closure. See `evidence/font-textures.json`.
No blank PNG was invented. Runtime font-atlas generation/content is unverified.

`--verify-ui` checks 20,782 referenced files: hashes and nested blob lengths,
raw object bytes against the APK, CAB metadata/dependency coverage, hierarchy
parents/ordered children/components and decoded mip dimensions/counts/base
PNG consistency. It does not independently rerun every decoder, rasterize
Unity sprite meshes or compare Unity pixels. Commands are in
`apk-reconstruction.md`; `evidence/ui-verification.json` is the compact report.

Two fresh local imports produce the same manifest digest, recorded in
`evidence/ui-reproducibility.json`. The canonical folder contains exactly the
20,782 verified files, with no unreferenced output files. This establishes
local static repeatability rather than cross-platform or runtime equivalence.

## Shared schema and source minimap renderer

`crates/ui_data` (`midnight_ui_data`) now owns the schema-1 DTO, package loader,
RectTransform layout and sprite geometry. The extractor re-exports this schema;
it does not maintain a second copy. Full UI verification after that move checks
20,782 files / 394,890,740 bytes and preserves manifest SHA-256
`677424f9e43225f3677f5713bed526dd9099339f2a8b44dbaecee7d41928cae5`.
Consumed payloads are hashed from the exact byte buffer returned to the caller.

The layout resolver applies anchors, stretch, pivots, inherited transforms,
activity and source child order in a supplied parent canvas. Sprite geometry
supports Simple, Sliced and horizontal/vertical Filled images, fractional atlas
UVs, packing rotations 0–4, trim, aspect and source PPU. All 120 minimap images
with static sprite bindings pass geometry generation; 13 source Images have
null sprite references. Unsupported radial/tiled/source-mesh/atlas variants
return explicit errors. Collapsed Sliced borders share one inner split to avoid
float32 rounding reversing the center; a regression test covers the source case.

`crates/client_core/src/source_ui/` renders these layouts through the existing
Bevy/egui backend. It prepares verified full-atlas base pixels and the embedded
source font, clips triangles through standard Mask alpha coverage at physical
pixel centers (threshold 0.001), and preserves interpolated UVs across clipping.
`SourceUiBindings` accepts qualified RawImage textures, text, node visibility
and RectTransform anchored positions, with target/type validation. Binding
evidence uses entry arrays, preserving qualified IDs as JSON objects.
Font atlas UVs normalize after all glyph allocations, including the first frame.
Vertical text truncation limits rows without an ellipsis; it does not add an
invented rectangle stencil around each Text.

`examples/minimap_probe.rs` was rebuilt with stripped debug information and
recaptured after the text/binding-report fixes. See
[render.json](evidence/minimap-render/render.json) and the
[1920×1080 detail](evidence/minimap-render/prefab-1920x1080-detail.png).
Both captures use NVIDIA RTX 4060 Laptop / Vulkan, six prepared atlases, one
embedded font and one standard mask. Each reports 125 submitted graphics,
20 unsupported, two disabled, two empty and one mask-only graphic.
These are Bevy prefab previews, not APK screenshots or a production HUD replacement.
Templates overlap because the serialized nodes are active and runtime visibility
has not been applied. The minimap RawImage remains explicitly unbound.

The optional `initialization-visibility.json` profile now adds two more
captures at the same sizes. It applies only the 21 qualified GO false overrides
in [the native review](evidence/minimap-runtime-review.md), hiding their
descendants without altering source records. Each initialization preview has
103 submitted graphics, 27 inactive, 15 unsupported, two disabled, two empty
and one mask-only. See the
[initialization detail](evidence/minimap-render/initialization-1920x1080-detail.png).
These are partial lifecycle previews, not successful native Awake/init or
gameplay frames. The profile includes APK/bundle/layout identity, GO/Transform
pairs, retained indices and native provenance; malformed/foreign/duplicate IDs
and non-false overrides are rejected. Its evidence hash covers the parsed byte
buffer. Texture, UV, local scale and zoom bindings are not inferred from it.

Remaining renderer gaps include custom stencil/fog/materials, line renderers,
particles, additional sprite variants, source mip sampling and Unity text
metrics/rasterization. egui shaping, line spacing, best-fit, color/tint and
premultiplied blending remain preview approximations. Per-graphic JSON reports
retain the exact source identities and reasons for omissions.

[Font coverage](evidence/minimap-font-coverage.json) independently compares
skrifa (the parser used by egui 0.36) and ttf-parser. The shipped OTF has 30,284
glyphs but lacks nominal mappings for `丧 尸 时 秒 解 计 领`, affecting three
minimap templates. This does not prove there are no corresponding outlines;
Unity fallback and runtime atlas behavior still require evidence. No substitute
font was introduced to conceal the gap.

## Historical export boundary

Old `assets/ui` layouts had 1817/225/416 nodes and 906/134/215 Images, but
discarded FileID, defaulted missing active flags to true, could replace zero
fill with one and ignored component failures. The former pass3 copied PNGs
from those layouts and ignored its APK argument; that code is replaced.

The historical audit found 33 distinct missing sprite names and redundant
panel stores. Those findings describe the old export and are not proof that
the APK lacks the assets. Large prior import evidence is retained under
`.local/ui-import-history/`.

## Map projection and C# boundaries

`crates/shared/src/map_projection.rs` owns overview projection/inversion,
radar offsets and bearings. Canonical Bevy north is **-Z**, east **+X**.
Camera yaw zero moves toward -Z and points upward. The importer reflects
Unity +Z into Bevy -Z once.

The HUD uses the same helpers for tactical-map positions/clicks, radar
background crops/markers, headings and compass bearings. Edge crops remain
unclamped to avoid stretching the background independently of markers.
Arrow/cone rotation keeps egui's native quad vertex order. Six projection
tests cover off-center crops, cardinal axes and simulation heading agreement.

`1BattlePanel_ann_f` is a translucent black 62×62 backplate (center alpha
102); its existing HUD use does not reproduce the source layout.
`MAP_OVERVIEW_HALF_EXTENT = 5360.0` remains provisional. The large-map prefab's
2048² `Map_map` texture binding is established, but the minimap runtime image,
effective scales/UVs and matched world calibration remain unverified.

`export/SC.UI/*.cs` is recovered pseudo-C#, with useful names/native addresses
and partial call clues, plus unresolved external types, invalid operations
and decompilation placeholders. Do not translate its bodies literally or
describe it as original compilable Unity source.

## Calibration recovered and next runtime work

[The canvas review](evidence/minimap-canvas-review.md) recovers enabled
CanvasScaler components in bootstrap `level0` and `level1`: reference
1920×1080, reference PPU 100. Native `AdjustResolution.Awake` ordinarily selects
`min(width/1920,height/1080)`. This supports the supplied preview inputs; the
live minimap ancestor chain and IFix state remain unobserved.

The minimap root preserves top-right anchors/pivot, 330×330 size and position
`[10,-32.399994]`; its standard mask is 260×260. MapCfg decoding consumes all
1465 bytes and associates all 241 terrain bundles with map 1's scene list.
For map 1 and initialized MapMgr.MapSize=10000, native fallback formulas yield
offsetForBase=0 and battleMapRate=0.4096. The source image and icon-parent
translation must use one calibration. Do not change the production 5360 extent
to 5000 from these scalar values alone.

The native review records constructor zoom=15. An internal reciprocal-zoom Rect
at +0x130 is not a verified RawImage UV setter; the scale vector's metadata
identity remains unresolved. OnShowCompassAndMinimap's `_mapRate` is the zero
gate, not the coordinate multiplier `_battleMapRate`. Native Awake requests
indices 0–93 while the retained GameObjectData has 86 entries (0–85): actual
list construction, prefab revision and IFix execution still need runtime evidence.

1. Trace runtime image selection, zoom/scales, UVs and initialization visibility
   from native bodies, then expose those changes as explicit source bindings.
2. Bind headings, compass, coordinates, waypoints and event icons to simulation
   state using qualified IDs. Keep static prefab state available for inspection.
3. Interpret custom stencil/fog and font fallback, and implement required graphics
   omitted from the current preview before integrating the source panel into the HUD.
4. Compare matched scene/camera/resolution captures against the exact APK,
   including off-center landmarks, map edges, clipping and panel transitions.

ADB found Waydroid with package/version `com.herogame.gplay.survival.lite`,
40 / 4.0. Its base APK SHA-256 is
`527ae0a10681fd0dfdbe957b65fee79729cc665d0e7afcce43f25054366fc37b`.
The Rust full-member comparison hashes all 9,320 regular files: 9,318 match,
including all assets, Java/manifest and libunity.so. Only libil2cpp.so and
global-metadata.dat differ. The user clarified that **lite4.0.apk contains the
recovered memory artifacts**, whereas the installed counterpart has the
encrypted originals. See `evidence/android-runtime-check.json` and
`evidence/apk-runtime-member-comparison.json`; the cached read-only APK copy is
under `.local/apk-runtime-candidates/`, ignored by Git.

[Dump pairing](evidence/minimap-dump-pairing.md) compares 20,488 mapped bytes
across seven native methods. OnShowCompassAndMinimap, GetPosOnMap and
AdjustResolution.Awake match exactly. The four initialization/constructor
ranges differ on disk; the constructor's final 1,592 bytes match. This supports
using the recovered source for analysis and the matching assets for previews,
without claiming whole-method or live-state equivalence. No app launch,
installation change or APK screenshot was performed. Continue with the paired
runtime and recovered artifacts; do not discard it merely because the outer
APK hashes differ.

Validation: shared UI 15 tests, client 25, map data 19, shared 46 with one
intentional stress-test ignore, extractor 24: 129 passed total. Offline
workspace/example checks and scoped formatting pass. Clippy reports no warnings
in the new UI code; existing map package, old HUD and terrain probe warnings remain.
The new streaming APK comparison example adds three passing tests for packaging
differences, changed/missing/extra payloads and CRC corruption: 132 passing tests
across the recorded suites. Its scoped release Clippy check passes.

Full UI behavior, interactions/animations, placements, water, original
lighting, gameplay and complete-game fidelity remain unfinished.
