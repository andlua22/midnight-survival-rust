# APK UI reconstruction

Updated: 2026-11-09. Target: `export/lite4.0.apk`, SHA-256
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
`MAP_OVERVIEW_HALF_EXTENT = 5360.0` remains provisional. The source map image,
origin/padding and world calibration have not been established.

`export/SC.UI/*.cs` is recovered pseudo-C#, with useful names/native addresses
and partial call clues, plus unresolved external types, invalid operations
and decompilation placeholders. Do not translate its bodies literally or
describe it as original compilable Unity source.

## Next runtime slice: BattleMinimap

Use the new source package as the single layout/asset input. Move its shared
DTO into a reusable Rust crate when the renderer consumes it.

1. Recover the runtime parent canvas/reference resolution and map calibration.
   No CanvasScaler is present in these selected prefabs.
2. Resolve anchors, pivots, rotation, sorting/draw order, sprite mesh/trim,
   nine-slice borders and fill behavior from retained source identities.
3. Implement source mask/stencil semantics. Current radar clipping is
   rectangular; a circle painted behind the image does not clip its corners.
4. Interpret font/glyph resources and bind headings, compass, coordinates,
   waypoints, map image and state-dependent icons to the simulation.
5. Compare matched scene/camera/resolution captures with the running APK,
   including off-center landmarks, map edges, clipping and panel transitions.

Full UI behavior, interactions/animations, placements, water, original
lighting, gameplay and complete-game fidelity remain unfinished.
