# Terrain normal and tangent review

Reviewed 2026-11-09 against the retained GLES3 programs and C-27 mesh evidence.
This is a bounded static review, not an Android runtime comparison. No production
code was edited, no shader extraction was repeated, and no Python was used.

## Confirmed behavior

The encoded-layer-0 residual is correct. In
`terrain-glsles/terrain-4-program-09.glsl:275`, `u_xlat16_5` retains the raw RGB
sample of `_Splat0_Normal`; its decoded value is held separately. Lines 280–290
form the weighted decoded normal and interpolate it toward the retained encoded
sample. The three-layer program does the same at
`terrain-glsles/terrain-3-program-09.glsl:281` and lines 286–295. Both include
the control alpha in the residual; only the four-layer variant adds an
alpha-weighted layer contribution.

Therefore the current CPU function in `crates/map_data/src/terrain.rs:59` and
WESL calculation at `assets/shaders/terrain_splat.wesl:83` match the source:

```text
R = clamp(1 - sum(control RGBA), 0, 1)
D = sum(control[layer] * (2*encoded[layer] - 1))
tangentNormal = D + R*(encoded[0] - D)
```

Do not decode the residual target, normalize the control weights, or discard
three-layer control alpha. The CPU result is in the source tangent frame;
WESL's later Y inversion is a separate coordinate conversion.

The UV and tangent sign conversions are consistent for the sampled orthogonal
scene transforms. `tools/asset_unpacker/src/mesh.rs:135` transforms source
normals; lines 143–148 transform tangent XYZ, set
`w_imported = -sign(det(world))*w_source`, and store `(u,1-v)`.
`assets/shaders/terrain_splat.wesl:46` then samples layers at
`(u*S, 1-v*S)`, after recovering source V from the stored UV. The importer
flips PNG rows in `tools/asset_unpacker/src/resolver.rs:143`; its ASTC path
returns decoded rows without an additional flip. This coordinate calculation
also handles fractional tiling without a constant vertical phase error.

The V change reverses the imported bitangent relative to the mirrored source
bitangent. Negating the completed tangent normal's Y at WESL line 90 cancels
that reversal. Applying the Y inversion to encoded texture values before the
residual calculation would change the encoded residual and would not be
equivalent. No global tangent-sign correction is indicated by this review.

The runtime preserves the source streams in
`crates/client_core/src/application.rs:378`: supplied normals and tangents
are inserted directly; tangents are not generated there.

## C-27 source normals

`c-27-mesh.json` contains two different meshes. The terrain is
`BuildPlayer-C-27.sharedAssets`, PathID 8, name `C-27`, with 1067 vertices.
Its normal, tangent and UV channels are float32 at offsets 12, 24 and 40 in
a 48-byte stream (`c-27-mesh.json:728`). The other mesh, PathID 7,
`Hupo_C27`, has a different half-float layout and no tangents; do not apply its
channel description to the terrain mesh.

An isolated Rust probe at `/tmp/c27-normal-tbn-probe.rs` read only C-27 mesh
PathID 8, checked the channel layout and matched its complete 51,216-byte
stream to the retained SHA-256:
`1bf8db34676750d4b65bb7261c5850d5d7782666572c479cf995191f0e7c71f8`.

Results:

- Source normal lengths range from 0.9999999001 to 1.0000000992; 1054 distinct
  source normals and 1053 distinct tangent XYZ values are present.
- Every source tangent W is -1.
- First source normal is `(0.089086406,0.021399003,0.995793939)`; the mirrored
  minus-90-degree X scene transform yields approximately
  `(0.089086406,0.995793939,0.021399003)` in the imported frame.
- Imported normal Y ranges from 0.9070245362 to 1.0 under that transform.
- All 1814 triangle faces agree in orientation with their averaged source
  normals; the minimum normalized dot product is 0.9917405358.

These facts support preserving the authored C-27 normals. They do not show an
up-axis, global sign, or inverted-normal error in the current conversion.

The current package manifest associates C-27 with the four-layer shader and
`_NORMALMAP`, with `terrain_sandA2_N` bound to normal layer 0 and bump defaults
for the remaining normal samplers. Those material values are a static source
association, not proof of the Android renderer's selected shader LOD.

## Remaining actionable fidelity difference

The APK constructs the bitangent **per vertex** and interpolates it separately.
See `terrain-glsles/terrain-4-program-09.glsl:107`: the cross product and
source determinant/tangent-W signs produce `vs_TEXCOORD4`. The fragment
stage uses the interpolated T, B and N directly at lines 266–303, then
normalizes the final mapped normal.

The current WESL calls `calculate_tbn_mikktspace` at line 91. Bevy 0.20.1's
`bevy_pbr/src/render/pbr_functions.wesl:148` calculates B in the fragment
shader from the interpolated N and T. In general:

```text
interpolate(cross(N_vertex,T_vertex)) !=
cross(interpolate(N_vertex),interpolate(T_vertex))
```

The C-27 probe evaluated source triangle centroids after per-vertex N/T
normalization, as the compiled source does. The two bitangent calculations
differ by more than 1e-7 at 1812 of 1814 centroids; the largest vector
difference is 0.0106478287. For a diagnostic tangent-space normal
`(0.3,-0.4,0.866025404)`, the largest final angular difference is 0.189194607
degrees, at triangle 633. This is a static counterexample using an explicit
diagnostic vector, not an observed source texture sample or rendered pixel.

For exact normal mapping, a source-matching vertex output should interpolate
the converted source bitangent separately, and the fragment should use that
basis. The current method has a small, measured C-27 difference; a change to
the global handedness signs would not resolve it.

## Missing source tangents

The main task reported the initial fresh-import rejection:
`Terrain_jichang: source normal-map material requires source tangents`.
This review did not independently decode that mesh. The source keyword alone
does not establish an unconditional tangent requirement: the retained LOD 200
programs 36–39 have no tangent input or normal-map variant. For example,
`terrain-glsles/terrain-4-program-36.glsl:56` declares position, normal and UV
inputs only. Shader metadata at
`terrain-glsles/terrain-4-shader-metadata.json:4641` maps that group to LOD 200.

The latest code has addressed the unsupported package rejection:
`crates/map_data/src/package.rs:122` preserves source absence;
`crates/client_core/src/terrain.rs:153` enables normal mapping only when
source tangents exist, and line 233 reports the geometric-normal fallback.
`tools/asset_unpacker/src/pass2_terrain.rs:276` records the same limitation.
This explicit fallback avoids inventing tangents. It should remain identified
as a reconstruction choice until the original runtime LOD and any generated or
default vertex attributes have been witnessed. Missing tangents must not be
silently presented as proof that the source itself used geometric normals.

## Other review boundaries

- The compiled GLSL transforms vertex normals with the forward ObjectToWorld
  matrix (`terrain-4-program-09.glsl:87`), whereas the importer uses inverse
  transpose. These agree in direction for the orthogonal C-27 transform;
  nonuniform source scaling would require another source-specific comparison.
- Source mediump behavior, compressed texture sampling, mip selection,
  renderer keyword/LOD state, shadows and Bevy lighting were not validated
  by this normal review. The current full PNG sampling preserves encoded
  normal RGB but does not prove matching source GPU samples.
- Existing assertions validate several source formulas and the coordinate
  conversion. They do not witness the APK renderer or eliminate the separate
  bitangent-interpolation difference above.
