# Map evidence

Updated: 2026-11-09. These are static observations from the supplied APK and
repository artifacts, not a witnessed run of the Android game or REA Evidence IDs.

APK: `export/lite4.0.apk`.
SHA-256: `2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
Sampled bundles report Unity `2021.3.23f1c1`.

## Findings that affect the import

1. The sampled A-57, C-10 and military16 scene transforms have position 0,
   scale 1 and quaternion `(-0.7071068,0,0,0.7071068)` (minus 90 degrees about X).
   The existing terrain extraction already applies that mesh-to-scene change.
   Do not apply it a second time merely because the raw mesh uses another up axis.
   The current terrain archive has 241 terrain records and 76 road records and
   no trailing bytes. See `evidence/map-evidence-audit.log`, lines1–6,21–22,
   36–37 and50. This sample does not prove every scene transform is identical.
2. `assets/sceneprefab/building_chaoshi_01.ab` contains 303 GameObjects,
   303 Transforms, 298 MeshRenderers, 298 MeshFilters and 5 Mesh objects.
   Selecting one mesh cannot represent this prefab. Its root has its own
   minus 90-degree X rotation, and its glass child has position
   `(0.021478087,0,0)` relative to the root. The root's saved position is
   `(2339.2349,32.92,666.88544)`; this alone is not proof of a runtime instance.
   See audit log, lines51–59.
3. The bounded APK inventory has 248 `assets/scene/` members, 1129
   `assets/sceneprefab/` members, and no `assets/sceneinfo/` members. The
   existing placements are outputs of a legacy XAPK extraction path. Their
   names, kind buckets, clamped uniform scales and randomized substitutions
   are not evidence of the supplied APK's exact scene placement semantics.
4. Terrain layer count comes from the resolved shader, not the saved `_Layer4`
   float. A-57 points to the4-texture shader despite `_Layer4=0`; C-10 points
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

| Material | Shader layers | Scalar tiling0,1,2,3 | Saved legacy `_Layer4` |
| --- | ---: | --- | ---: |
| A-57 | 4 | 40,60,60,90 | 0 |
| C-10 | 3 | 60,90,50; fourth unused | 1 |
| military16 | 4 | 60,90,50,120 | absent |

Military16 also saves `_Splat3` texture offset X=1.2999999523 and
`_Tiling3=(120,120,0,0)`. These saved values do not enter the extracted
compiled UV calculation. Raw material values are in the audit log, lines10,
25 and40. Avoid interpreting unused saved material properties as live behavior.

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
changes that behavior. The residual is a interpolation toward L0, not an
additive missing weight: for `C=(0.25,0.25,0,0)`, final albedo is
`0.625*L0 + 0.125*L1`, not `0.5*L0 + 0.5*L1`.

Direct references:

- Four layers: `terrain-4-program-08.glsl`, lines260–272 and302–314.
- Three layers: `terrain-3-program-08.glsl`, lines270–280 and303–315.
- LOD250 corroboration: `terrain-4-program-24.glsl`, lines258–277.
- LOD200 corroboration: `terrain-4-program-36.glsl`, lines231–249.
- Normal-map blend uses the same raw control and residual:
  `terrain-4-program-09.glsl`, lines269–291 and320–365.

LOD 300 and 250 set output alpha to `W`; LOD 200 sets it to 1. LOD 300 also
multiplies diffuse albedo by 0.959999979 as part of nonmetallic lighting,
samples the layer texture alpha for smoothness, and includes reflection/light
calculations. A CPU albedo bake should preserve the pre-lighting blend and
record the renderer's separate lighting differences. Applying the lighting
factor in a baked texture and again in a material would change the result.

## Reproduction and verification

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

Canonical import/loading changes are owned by the main task. This report
does not claim they are complete. Full prefab reconstruction, exact placements,
source mip selection, texture color-space sampling, renderer state, colliders,
UI and gameplay still require their own evidence and validation. Keep these
shader findings when consolidating the importer; do not revive the legacy
fourth-layer toggle or splat normalization based on the old manifests.
