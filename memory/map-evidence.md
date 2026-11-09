# Map evidence

Updated: 2026-11-09. The runtime now consumes one validated APK scene package
for rendering and terrain physics. This handoff combines static APK evidence
with GPU validation of the Rust renderer. The Android game's execution and
pixel equivalence remain unobserved; these records are not REA Evidence IDs.

APK: `export/lite4.0.apk`.
SHA-256: `2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
Sampled bundles report Unity `2021.3.23f1c1`.

## Current canonical package

Package root: `assets/imported_map/terrain/`.
`manifest.json` uses schema 2 and identifies `meshes.bin`, whose archive magic
is `MSMAP003`. Geometry SHA-256:
`89febd1decfaa5ed58089e9912fd61dedcb5285d51acb86f665e35c23413e8ee`.
The current source inventory and validation results are retained in
`evidence/terrain-verification.json`.

| Verified inventory | Count |
| --- | ---: |
| Imported scene bundles | 241 |
| Distinct source mesh instances / render records | 399 |
| Terrain instances | 287 |
| Three-layer terrain instances | 160 |
| Four-layer terrain instances | 127 |
| `_URP/LambertLit` instances | 95 |
| `Yo/Add` instances | 17 |
| Enabled, non-trigger source colliders | 382 |
| Distinct source materials | 289 |
| Retained source texture assets | 316 |

The 399 records include multiple terrain and other mesh instances within some
scene bundles. Preserve the qualified bundle/CAB/PathID and GameObject
identity; a scene filename or a shared Mesh PathID alone does not identify an
instance. The archive also preserves supplied normals, tangents, UV0 and
triangle indices. Its aggregate bounds are approximately
`X=[-4600,3000]`, `Y=[-45.3635,255.0457]`, `Z=[-3031.4,3000.0002]`.

Scene-parent transforms are resolved before applying Unity world
`(x,y,z) -> Bevy (x,y,-z)` once. `MSMAP003` already stores the resulting Bevy
coordinates; renderer and physics do not repeat this reflection. Bevy map
north is `-Z` and east is `+X`. Mirrored triangle winding and tangent signs,
plus top-left UV/image conventions, are handled in the Rust import.

The client loads the package through `crates/shared/src/level.rs`, gives the
same records to `crates/client_core/src/terrain.rs`, and builds Rapier meshes
through `PhysicsWorld::insert_canonical_terrain`. Source enabled/trigger
flags select collision records. Package validation checks geometry and
texture identities and refuses corrupt data. Legacy `island_terrain.bin`
does not serve as a runtime fallback.

The canonical island path no longer spawns the legacy guessed scene-info
placements, legacy `water.bin` geometry, or the synthetic ocean floor; it
also removes terrain distance hiding. Source lake, road and other surfaces
included among the 399 imported records retain their source identity.
Demo island NPCs, vehicles and resource nodes are cleared. Old extracted
archives remain available as evidence; full prefab reconstruction and their
placement provenance still need separate work.

### Source mip levels

Every retained texture has a decoded source mip archive with magic
`MSRGBA01`, alongside its source metadata, complete encoded payload and PNG
preview. The archive stores top-left RGBA8 pixels for each original mip,
largest level first, without regenerating levels from the preview.
`crates/map_data/src/texture.rs` validates dimensions and complete level
lengths. The runtime loader in `crates/client_core/src/map_texture.rs`
uploads that chain; terrain materials select it through `TextureAsset.gpu`.

The importer handles the shipped ASTC channel order explicitly. Texture
sampling retains source color-space, wrap and filter settings. The 316
package textures have mip-count distribution `1:100, 9:1, 10:211, 11:4`;
315 use source Unity format 50 and one uses format 4. Exact GPU sampling and
Unity's global mip bias still require a runtime comparison.

## Findings that affect the import

1. The sampled A-57, C-10 and military16 scene transforms have position 0,
   scale 1 and quaternion `(-0.7071068,0,0,0.7071068)` (minus 90 degrees about X).
   The canonical import resolves and applies that mesh-to-scene change.
   Do not apply it a second time merely because the raw mesh uses another up axis.
   The earlier `MSMAP001` audit is retained in
   `evidence/map-evidence-audit.log`; its record inventory is historical and
   has been superseded by the 399-instance canonical package. Sampled
   source transforms are recorded at lines 21–22, 36–37 and 50. This sample
   does not prove every scene transform is identical.
2. `assets/sceneprefab/building_chaoshi_01.ab` contains 303 GameObjects,
   303 Transforms, 298 MeshRenderers, 298 MeshFilters and 5 Mesh objects.
   Selecting one mesh cannot represent this prefab. Its root has its own
   minus 90-degree X rotation, and its glass child has position
   `(0.021478087,0,0)` relative to the root. The root's saved position is
   `(2339.2349,32.92,666.88544)`; this alone is not proof of a runtime instance.
   See audit log, lines 51–59.
3. The bounded APK inventory has 248 `assets/scene/` members, 1129
   `assets/sceneprefab/` members, and no `assets/sceneinfo/` members. The
   legacy placement archive comes from a separate XAPK extraction path. Its
   names, kind buckets, clamped uniform scales and randomized substitutions
   are not evidence of the supplied APK's exact scene placement semantics.
4. Terrain layer count comes from the resolved shader, not the saved `_Layer4`
   float. A-57 points to the 4-texture shader despite `_Layer4=0`; C-10 points
   to the 3-texture shader despite `_Layer4=1`. Neither shader declares that
   property, and none of the 40 extracted GLES3 program strings uses it.

## Compiled terrain shader evidence

The exact shader bundle is `assets/shader/shaders.ab`, CAB
`CAB-b99e201845a48115c6b6a24e89d01efe`.

| PathID | Parsed shader name | Compressed bytes | Decompressed bytes |
| --- | --- | ---: | ---: |
| `-1302531423149963743` | `_URP/TerrainMeshBase/4 Textures` | 15303 | 304356 |
| `8944411127715196945` | `_URP/TerrainMeshBase/3 Textures` | 17185 | 317688 |

Both advertise one platform 9 program block at offset 0, compressed with LZ4.
Strict type-tree metadata, compressed bytes, complete decompressed blobs,
subprogram manifests, and 40 exact GLSL sources are saved in
`evidence/terrain-glsles/`. Each blob has 40 table entries with version 202012090:
20 intact GLES3 source strings (`gpu_type=4`) and 20 non-text program entries
(`gpu_type=5`). The latter remain in the complete blob and are not decoded by
the evidence probe. All text ranges are bounded by their subprogram table
entries; no source is truncated or edited. No source has trailing NUL bytes,
so the `.glsl` and corresponding `.program-code.bin` files are byte-identical.

The strict metadata maps `ForwardLit` programs 08–15 to LOD 300, 24–31 to LOD 250,
and 36–39 to LOD 200. Keyword variants include `_NORMALMAP`, `LIGHTMAP_ON`,
`_MAIN_LIGHT_SHADOWS` and `_MAIN_LIGHT_SHADOWS_CASCADE`. The actual runtime's
chosen shader LOD and global keyword state remain unobserved.

### UVs and saved material values

The compiled vertex programs pass `in_TEXCOORD0.xy` straight to
`vs_TEXCOORD0.xy`. The fragment programs sample `_Control` at that UV and
sample layer N at `UV0 * _SplatN_S`. `_Control_ST` is declared but unused.
No extracted source uses `_SplatN_ST`, `_Tiling3`, or a saved texture offset.
Therefore scalar `_SplatN_S` is the observed layer tiling parameter.

| Material | Shader layers | Scalar tiling 0,1,2,3 | Saved legacy `_Layer4` |
| --- | ---: | --- | ---: |
| A-57 | 4 | 40,60,60,90 | 0 |
| C-10 | 3 | 60,90,50; fourth unused | 1 |
| military16 | 4 | 60,90,50,120 | absent |

Military16 also saves `_Splat3` texture offset X=1.2999999523 and
`_Tiling3=(120,120,0,0)`. These saved values do not enter the extracted
compiled UV calculation. Raw material values are in the audit log, lines 10,
25 and 40. Avoid interpreting unused saved material properties as live behavior.

### Exact control blend

Let `C` be the raw RGBA sample of `_Control`, and `LN` the RGB sample of
`_SplatN` at the scalar-tiled UV.

```text
W  = C.r + C.g + C.b + C.a
R  = clamp(1 - W, 0, 1)
B4 = C.r*L0 + C.g*L1 + C.b*L2 + C.a*L3
B3 = C.r*L0 + C.g*L1 + C.b*L2
Albedo = B + R*(L0 - B)
```

There is no weight normalization or alpha reconstruction in this calculation.
The 3-texture shader uses alpha in `W` and the residual but contributes no
alpha-weighted fourth layer color. Replacing alpha with zero or `1-r-g-b`
changes that behavior. The residual is an interpolation toward L0, not an
additive missing weight: for `C=(0.25,0.25,0,0)`, final albedo is
`0.625*L0 + 0.125*L1`, not `0.5*L0 + 0.5*L1`.

Direct references:

- Four layers: `terrain-4-program-08.glsl`, lines 260–272 and 302–314.
- Three layers: `terrain-3-program-08.glsl`, lines 270–280 and 303–315.
- LOD 250 corroboration: `terrain-4-program-24.glsl`, lines 258–277.
- LOD 200 corroboration: `terrain-4-program-36.glsl`, lines 231–249.
- Normal-map blend uses the same raw control and residual:
  `terrain-4-program-09.glsl`, lines 269–291 and 320–365.

LOD 300 and 250 set output alpha to `W`; LOD 200 sets it to 1. LOD 300 also
multiplies diffuse albedo by 0.959999979 as part of nonmetallic lighting,
samples the layer texture alpha for smoothness, and includes reflection/light
calculations. The current terrain WESL samples and blends source layers at
runtime, using the pre-lighting source formula. Its lighting remains Bevy's;
the source lighting factor must not also be baked into the texture.

### Encoded normal residual and tangent basis

The normal-map programs have an unusual residual target. Let `EN` be the
encoded RGB normal sample for layer N. They compute

```text
D4 = sum(C.rgba * (2*E0..3 - 1))
D3 = sum(C.rgb  * (2*E0..2 - 1))
TangentNormal = D + R*(E0 - D)
```

The residual targets **encoded E0**, while the weighted contributions are
decoded. Both variants keep control alpha in `R`. The CPU reference in
`crates/map_data/src/terrain.rs` and `assets/shaders/terrain_splat.wesl`
preserve this behavior. The importer applies
`w_imported=-sign(det(world))*w_source` for the world reflection and V change;
WESL negates the completed tangent-normal Y afterward. This preserves the
sampled source basis without changing the encoded residual.

The bounded C-27 review is retained in
`evidence/terrain-normal-review.md`. It matched the source stream hash,
confirmed all 1067 supplied terrain normals and tangents, and found correct
normal orientation after conversion. It also measured a remaining basis
difference: Unity interpolates a bitangent built per vertex, while Bevy
recomputes it from interpolated N/T in the fragment. A diagnostic C-27
tangent normal produced a maximum 0.189194607-degree difference. This is a
static counterexample, not a measured APK pixel difference.

`Terrain_jichang` has a normal-map material but no supplied tangents. Its
absence is retained, package validation accepts it, and the renderer reports
its geometric-normal fallback. LOD 200 source programs do not need tangents;
the original runtime LOD or generated/default attributes remain unknown.

## Existing GPU validation

Current captures and metadata are in `evidence/terrain-render/`:
`overview.png`, `spawn.png`, and `render.json`. The Vulkan probe used Bevy
0.20.1 on an NVIDIA GeForce RTX 4060 Laptop GPU, the current schema-2
geometry identity, all 399 source mesh entities and 382 collision records.
All 312 referenced source mip textures were uploaded; the report records
mip-count distribution `1:99, 9:1, 10:208, 11:4`. The package retains 316
source texture assets, including bindings not used by active rendering.

The current two 1280x1024 views use AgX and fixed Bevy lighting with shadows
disabled. They show actual Rust renderer output and confirm package/shader
upload, not equality to an APK screenshot. Earlier captures in
`evidence/terrain-render-before-mips/` remain comparison evidence from the
preview-texture path and should not be mistaken for the current mip path.

## Reproduction and verification

The installed Rust import/verification tool is `tools/asset_unpacker`.
To verify the current canonical package against the supplied APK without
importing or promoting assets:

```bash
cargo run --offline --manifest-path tools/asset_unpacker/Cargo.toml -- --verify --apk export/lite4.0.apk --out-dir assets --report /tmp/terrain-verification.json
```

Use `evidence/terrain-verification.json` for the completed verification
results and `evidence/terrain-render/render.json` for the existing GPU probe.
The following commands reproduce the independently retained shader evidence;
they are separate from canonical package generation.

`evidence/dump_terrain_shader_metadata.rs` reads only the shader bundle and
the two specified shader PathIDs with strict type-tree parsing.
`evidence/extract_terrain_glsles.rs` extracts validated length-prefixed strings
from the complete blobs. Their independent Cargo manifest is
`evidence/terrain_shader_probe/Cargo.toml` and uses installed Rust crates.

```bash
cargo run --offline --manifest-path memory/evidence/terrain_shader_probe/Cargo.toml --target-dir /tmp/terrain-shader-probe --bin dump-terrain-shader-metadata -- export/lite4.0.apk /tmp/terrain-shader-evidence
cargo run --offline --manifest-path memory/evidence/terrain_shader_probe/Cargo.toml --target-dir /tmp/terrain-shader-probe --bin extract-terrain-glsles -- 4 /tmp/terrain-shader-evidence/terrain-4-subprograms.bin /tmp/terrain-shader-evidence
cargo run --offline --manifest-path memory/evidence/terrain_shader_probe/Cargo.toml --target-dir /tmp/terrain-shader-probe --bin extract-terrain-glsles -- 3 /tmp/terrain-shader-evidence/terrain-3-subprograms.bin /tmp/terrain-shader-evidence
sha256sum --check memory/evidence/terrain-glsles/SHA256SUMS
```

Both Rust probes compiled and ran without Python. The strict fresh extraction's
two decompressed blobs were byte-compared against the prior audited blobs;
both matched. Hashes cover saved metadata, compressed/decompressed blobs,
manifests and all source payloads. These checks prove static extraction and
program math, not pixel equality to a running Unity renderer.

## Remaining boundaries

The terrain scene package, source mip upload and shared rendering/collision
path are implemented and validated. Complete 1:1 game fidelity is still open.
Full prefab hierarchies, their exact instance placement, characters, UI and
gameplay need further reconstruction. The source supermarket alone has
hundreds of child objects; a representative single mesh is insufficient.

Original renderer LOD/keywords, lighting, fog, shadows, tonemapping and exact
GPU texture samples remain unobserved. Bevy's fragment bitangent calculation
and the missing-tangent fallback are explicit normal-mapping boundaries.
The 95 LambertLit and 17 Yo/Add surfaces retain their source identity but
their original shaders are not fully reproduced. Rapier uses the selected
source mesh triangles and flags; parity with original PhysX cooking and
simulation is unverified. GPU captures validate this Rust implementation,
while a matching source runtime capture is still needed for visual parity.

Keep the verified shader math, source mips and instance identities when
continuing. Do not reactivate the old fourth-layer toggle, splat
normalization, guessed placement substitutions or synthetic water geometry.
