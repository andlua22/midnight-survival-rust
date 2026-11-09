# APK UI reconstruction: evidence and first correction

Updated: 2026-11-09. Target: `export/lite4.0.apk`.

The user wants the original game and UI reproduced faithfully, with one canonical map pipeline. Communicate in English. Write new extraction and validation tools in Rust and reuse existing tools; do not run Python unless Rust and the available tools are insufficient.

This is a static audit of the existing extraction and code, followed by a narrow map-projection correction. It does not establish a completed 1:1 UI recreation or a screenshot match to the running APK. No Python was executed for this audit or correction.

## Existing UI evidence

A read-only Rust traversal of the JSON files and PNG directories produced these baseline counts:

| Layout | Nodes | Images | Texts | Unique sprite names | Missing PNG and metadata | Unreferenced PNGs in panel directory |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `assets/ui/battle_panel_layout.json` | 1817 | 906 | 259 | 245 | 30 | 99 |
| `assets/ui/battle_minimap_layout.json` | 225 | 134 | 14 | 56 | 5 | 0 |
| `assets/ui/battle_map_panel_layout.json` | 416 | 215 | 75 | 81 | 4 | 36 |

`assets/ui/sprites_meta.json` contains 987 entries, each with a corresponding PNG in `assets/ui/sprites`. The layouts reference 318 distinct sprite names in total, with 33 distinct missing names. The most relevant missing map asset is `FogLightMask`, referenced by all three panels. Missing references are an extraction gap, not proof that the APK lacks the asset or that every missing sprite should be visible in the default game state.

All 2458 exported hierarchy nodes have `active: true`. This is an observation about the exported JSON, not a verified claim about APK runtime visibility. The exporter defaults missing `m_IsActive` to true and does not retain component enablement or script-controlled visibility.

The layouts retain only image and text components. Rectangles retain anchors, anchored position, size delta, pivot, local position, and local scale. Text retains only content, font size, color, and alignment. Canvas scaling, font identity, local rotation, buttons/listeners, mask components, animation bindings, and original component references are absent. There are 244 sliced images and 18 filled images across the layouts, making nine-slice and fill semantics necessary for faithful reproduction.

Relevant original hierarchy paths in the existing JSON:

- `BattleMinimap/GameObject/Image` uses `BattlePanel2_dt_bg_1`, with a 260×260 size delta.
- `BattleMinimap/GameObject/ditu/Image` uses `1BattlePanel_ann_f`, with a 330×330 size delta.
- The minimap root has anchors and pivot `[1, 1]`, anchored position `[10, -32.39999]`, and size delta `[330, 330]`.
- `BattleMapPanel/GameObject/you/btn_close` uses `com_ty_gb_1`.

These are source-layout coordinates. They cannot be treated as final screen pixels until the original CanvasScaler and reference resolution are recovered.

## Extraction problems

`tools/asset_unpacker/src/pass3_ui.rs` currently ignores the APK argument. It reads previously generated layouts, gathers sprite names, and copies existing PNGs into three panel directories. It does not extract Unity prefabs or sprites from the APK. It returns success when layouts are absent or sprite references cannot be copied, and merely checks that the metadata file exists. Existing destination PNGs are accepted without checking whether they match the current source. See the function beginning at `pass3_ui.rs:83` and the copy loop near line 134.

The older exporter, read for context but not executed, has additional problems in `tools/unpack_all_assets.py`:

- Sprite lookup around lines 673–679 uses PathID alone and discards the PPtr FileID, so it cannot safely distinguish local objects from external CAB references.
- `float(tree.get("m_FillAmount", 1.0) or 1.0)` around line 688 changes a genuine zero fill amount into one.
- Unrecognized component parsing and hierarchy failures are silently ignored around lines 704 and 723.
- The exported schema omits the fidelity-critical data listed above.

Prefer one sprite store with qualified source identities and referenced manifests; the three copied PNG directories are redundant and can accumulate stale files. Any replacement must regenerate from source rather than present existing files as newly extracted evidence.

## C# evidence boundary

The files in `export/SC.UI/` are recovered pseudo-C#, not original compilable Unity source and not uniformly empty stubs. They retain useful type, field, method, native address, and recovered call clues. They also contain unresolved `__External_*` types, invalid object arithmetic, failed IL conversions, and placeholder diagnostics such as `call to unresolved address`.

Examples: `BattleMinimap.cs:15` has an unresolved base type; lines 4350–4383 contain recovered calls mixed with invalid object operations and unresolved-call diagnostics. `BattleMapPanel.cs:1188` contains placeholder RectTransform operations. Use names and RVA annotations to guide focused native analysis; do not translate these bodies literally or infer exact behavior from failed decompilation.

## Projection correction implemented

The current presentation convention is +X to the right and +Z at the top. `crates/shared/src/map_projection.rs` now owns pure Rust forward/inverse overview projection, radar point offsets, world bearings, and camera-yaw-to-map bearing. `crates/shared/src/lib.rs` exports it.

`crates/client_core/src/hud_painter.rs` now uses these helpers for tactical map positions and clicks, the radar overview crop, every radar marker, both player headings, compass sound cues, and compass waypoint bearings. Before this correction, the tactical map used `v = (half - Z) / span`, while the radar background used `v = (Z + half) / span` and markers used positive Z for positive screen Y. A player away from the world origin consequently sampled the opposite vertical map location. Radar crops are now left unclamped so approaching an overview edge does not rescale the background independently of the markers.

Camera yaw zero moves forward toward world -Z (`crates/shared/src/movement.rs`, displacement calculation), so it must point downward on this map. The old arrow headed upward. The correction projects camera motion and headings under the same convention.

The textured arrow/cone code previously assigned corners in TL, TR, BR, BL order to an egui rectangle whose native vertex order is TL, TR, BL, BR, creating twisted quads. A shared image-rotation helper now rotates the native vertices in place. Pixel inspection confirmed that the arrow sprite points upward and the cone sprite downward, so the cone receives its intrinsic 180° adjustment.

The imported `1BattlePanel_ann_f` backplate is now drawn behind the radar. It is a translucent black 62×62 sprite, with center alpha 102, not a separate decorative border. This addition uses the existing widget rectangle and does not claim that the full original minimap layout has been recreated.

The existing `MAP_OVERVIEW_HALF_EXTENT = 5360.0` is a calibration value. Its exact relationship to the APK's map image, terrain bounds, padding, and origin has not been established by this audit. Do not describe 5360 m as an APK-proven extent. The helpers receive extents explicitly so a verified calibration can replace it without new sign conversions.

Validation performed:

```bash
rustc --edition 2024 --test crates/shared/src/map_projection.rs -o /tmp/midnight-map-projection-tests
/tmp/midnight-map-projection-tests
```

All six tests passed. They check independent north/east landmark placement, forward/inverse coordinates inside and outside the image, off-center radar positions, marker alignment with the overview crop, cardinal bearings, and heading agreement with simulation forward motion. This is numerical validation; it does not replace integrated rendering checks or APK screenshots.

## Remaining UI work

The client still draws a hand-built egui HUD. Its production crates do not load any of the extracted layout JSON files or sprite metadata. `HudLayout::preset` and `WidgetId::size` define hand-tuned positions and sizes; `UiSpriteAssets` loads a small fixed sprite subset; `hud_painter.rs` draws those into custom geometry. Exact hierarchy composition, nine-slice borders, original fonts, visibility rules, interactions, and animations are therefore not reproduced.

The radar image still uses rectangular clipping; painting a circle behind an image does not mask its corners. Actual circular/Unity mask behavior remains to be implemented from recovered mask semantics. `FogLightMask` needs explicit investigation. Source-derived visibility, gameplay state, and dynamic binding targets also remain unresolved.

The next concrete UI slice should be `BattleMinimap`, after canonical map calibration:

1. Extract its source prefab, qualified sprite/component references, CanvasScaler, enabled/active flags, mask data, and text/font settings in Rust.
2. Define one versioned UI DTO shared by extraction and rendering, retaining source identity and hierarchy order. Avoid another unrelated HUD implementation.
3. Resolve stretch anchors and pivots under the recovered canvas scale; preserve source sprite, nine-slice, fill, color, and draw order.
4. Bind the map image, player arrow, compass, coordinates, waypoint, and visible gameplay icons to the existing simulation through explicit source identities.
5. Compare the same scene and camera against the running APK at a recorded resolution. Check off-center landmarks, heading, clipping, edge crops, and opening/closing the full map. Keep the UI labeled incomplete until this comparison exists.

Complete-game fidelity, full UI coverage, exact source map calibration, and runtime screenshot evidence remain open work.
