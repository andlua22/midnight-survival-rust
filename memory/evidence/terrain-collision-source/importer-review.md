# Independent terrain collision source review — 2026-11-12

This review inspects the six previously omitted locally active source solid
components and the Rust collision-decoding/transform boundary. It changes no
production code or retained package and runs no Cargo, Python, original game,
GUI or source service. Root owns implementation and executable validation.

## Source inspections

The existing Rust `asset_unpacker` inspected classes 1,4,54,64,65 in the exact
APK members `assets/scene/a-46.ab`, `assets/scene/z-01.ab` and
`assets/scene/b-32.ab`, then class 43 in B-32. `--inspect-bundle` does not
publish packages. Diagnostic JSON was written only to task-owned `/tmp` paths;
the relevant observations below are retained here. JSON field selection used
Node only to read those Rust-generated diagnostics; Unity parsing remains Rust.

Source APK identity inherited from the verified reconstruction ledger:
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.

| Inspection file | SHA-256 |
|---|---|
| /tmp/codex-terrain-collision-a46-review.json | 37aa124b6bb5aa2fa6ebf42e6ccffa72c42757fb507ea48086204e334e706362 |
| /tmp/codex-terrain-collision-z01-review.json | 979045670616dac4317bc9a0608cfcbbb641237a67f2e150cd462ca599169908 |
| /tmp/codex-terrain-collision-b32-review.json | 5eb90a2e7e8ef9dc18fb73914ff26794f40cc38230c15719985c33d077056ef1 |
| /tmp/codex-terrain-collision-b32-mesh-review.json | aed03ae0f2ec6f51689bb087668a206851dda4183e94b85b90355db27be5ad7f |

The inspection executable's SHA-256 at review time was
`02ce3b481b14457ab6ec348dbac06a8a08efc65ca26830d9a94aea32bda4e566`.
It was not rebuilt by this reviewer. Root may subsequently rebuild it for the
new import action; that later executable has separate validation identity.

Reproduce a metadata inspection with:

```sh
tools/asset_unpacker/target/debug/asset_unpacker --apk export/lite4.0.apk --inspect-bundle assets/scene/a-46.ab --inspect-classes 1,4,54,64,65 --report /tmp/codex-terrain-collision-a46-review.json
```

The same flags were used for Z-01 and B-32 with their corresponding paths.
The B-32 mesh inspection used `--inspect-classes 43` and the mesh-review path.

## Six target components

All five Boxes have raw Unity-local center `(0,0,0)`, size `(1,1,1)`, enabled
true, trigger false, and null PhysicMaterial. Every target owner has layer 0
and `m_IsActive=true`; every observed ancestor is also active. No Rigidbody
class 54 appears in any of the three scene serialized files, and the target
owners/ancestors' component lists contain no Rigidbody.

| Scene/CAB | Collider | Owner GameObject | Transform | Parent transform chain |
|---|---:|---:|---:|---|
| A-46 / BuildPlayer-A-46 | Box 34 | 3, Cube | 13 | 19 → 17 → null |
| Z-01 / BuildPlayer-Z-01 | Box 34 | 1, Cube (4) | 14 | 15 → null |
| Z-01 / BuildPlayer-Z-01 | Box 35 | 3, Cube (3) | 16 | 15 → null |
| Z-01 / BuildPlayer-Z-01 | Box 36 | 4, Cube (2) | 17 | 15 → null |
| Z-01 / BuildPlayer-Z-01 | Box 37 | 5, Cube (5) | 18 | 15 → null |
| B-32 / BuildPlayer-B-32 | Mesh 10 | 2, b32 | 4 | null |

Saved target Transform local values, in Unity coordinates (quaternions are
`x,y,z,w`; these values have not been replaced by inferred world poses):

| Transform | Position | Quaternion | Scale |
|---|---|---|---|
| A-46 13 | (-923.4199829101562,30.558000564575195,811.0900268554688) | (0,0.36887702345848083,0,0.9294782280921936) | (8.885788917541504,1,2.3162999153137207) |
| A-46 19 | (842.9000244140625,1.8290328979492188,890.0902099609375) | (0,-0.7071068286895752,0,0.7071068286895752) | (1,1,1) |
| A-46 17 | (-890.0899658203125,-1.8290328979492188,842.9000244140625) | (0,0.7071068286895752,0,0.7071068286895752) | (1,1,1) |
| Z-01 14 | (-3265,18.8799991607666,945) | (0,0,0,1) | (10,0.5,10) |
| Z-01 16 | (-3275,18.579999923706055,954.989990234375) | (0.016135845333337784,0.00019267620518803596,0.011938476003706455,0.999798595905304) | (10,0.5,10) |
| Z-01 17 | (-3265,18.739999771118164,955) | (0.013209734112024307,-0.00010714125528465956,-0.008109813556075096,0.9998798370361328) | (10,0.5,10) |
| Z-01 18 | (-3274.989990234375,18.739999771118164,945) | (-0.002129090018570423,-0.0003375151427462697,0.015730924904346466,0.9998739361763) | (10,0.5,10) |
| Z-01 15 | (0,0,0) | (0,0,0,1) | (1,1,1) |
| B-32 4 | (0,0,0) | (-0.7071068286895752,0,0,0.7071068286895752) | (1,1,1) |

A-46's parent rotations/translations approximately cancel, but both source
parents must still be multiplied; do not collapse them to a guessed identity.
The four Z-01 slabs must retain their distinct small rotations and height
values rather than becoming one axis-aligned region.

## B-32 has duplicate component geometry

B-32 MeshCollider 10 and 11 are distinct ordered components on the same
GameObject. They have identical saved collision fields: both enabled, solid,
non-convex, cooking options 30, null PhysicMaterial, and `m_Mesh` PPtr
`{m_FileID:1,m_PathID:5}`. The external is
`archive:/BuildPlayer-B-32/BuildPlayer-B-32.sharedAssets`; the actual mesh is
class 43, CAB `BuildPlayer-B-32.sharedAssets`, signed PathID 5, name `road_b32`.

That mesh uses compressed data: `m_VertexCount=0`, empty external stream,
one triangle submesh, 464 vertices and 2,046 indices (682 triangles),
baseVertex 0. Packed vertices use bitSize 20 and packed triangles bitSize 9.
Packed normals and UVs are present; tangents are absent.

Preserve both source collider identities and handles while sharing the one
decoded mesh. This restores six source component instances overall, with
five new Box geometries and one duplicate Mesh collider instance; it does
not establish an additional unique B-32 surface. Per-mesh deduplication must
never deduplicate component instances.

## Decoder and coordinate review

`mesh::decode_collision(object, matrix)` returns local/world positions and a
single combined triangle-index vector. It uses all supplied triangle submeshes,
applies baseVertex, validates complete indices and flips winding when the
supplied matrix has a negative determinant. The raw vertex-stream path does
not require shading normals or UVs. The packed path delegates to
`packed_mesh::decode` and `mesh::finish`; each returned part shares the full
position vector, so taking one position vector and concatenating the part
indices is valid. The packed path still requires its source normal/UV fields;
missing packed attributes must fail explicitly rather than be invented.

Use `Resolver::mesh_object`, not just `object`, to supply an external vertex
stream when the mesh needs one. `Resolver::transform` normalizes supplied
quaternions and recursively multiplies parent TRS, with depth/cycle checks.
It does not itself compute effective activity or Rigidbody ancestry. Those
require separate qualified parent/component traversal.

Let `R=diag(1,1,-1,1)` and `U` be the source Unity local-to-world matrix.
The proposed contract is coherent:

- Collision mesh positions decode with `R` into Bevy-local coordinates.
- The table stores `R * U * R` as Bevy-local to Bevy-world.
- Applying that table matrix to a decoded point yields `R * U * p`, reflecting
  Unity world Z exactly once.
- Mesh decoding with `R` already changes triangle winding once. A later runtime
  winding change is needed only for the table matrix's negative determinant,
  corresponding to additional source negative scale.
- Box center remains raw Unity-local; runtime reflects center Z once, generates
  eight corners with positive raw sizes and applies the table matrix. Existing
  prefab corner baking is suitable and preserves rotation/nonuniform scale.

Reviewed code identities at the source-inspection stage:

| File | SHA-256 |
|---|---|
| tools/asset_unpacker/src/mesh.rs | 5774fb71dd796d43f44fa4d3f974537e3371c5baa72d8e41452c7892cd7c7ee4 |
| tools/asset_unpacker/src/resolver.rs | 529bd2202442e76ad14def28fdd3294c1317c69a86d576bfa3fb6dd627ad5819 |
| tools/asset_unpacker/src/packed_mesh.rs | 04a649dd03a4f3b4a061884ce095d0c047e2bd31075c7794d2c14793e54a77bd |

## Remaining limits and implementation review

Saved absence of Rigidbody and saved hierarchy activity admit these six
static instances only within the recovered scene data. Runtime patching,
streaming, activation, PhysX cooking/contact/tolerance and the original global
trigger policy remain separate acceptance work. The twelve saved layer-2
triggers and three locally inactive source MeshColliders must be retained
without inferred gameplay effects or activation.

## Production integration review

The read-only review covered `terrain_collisions.rs`, `Resolver::class_id`,
the layer-version fixes, pass-2 import/augmentation, the map-data collision
schema/archive/loader, shared terrain insertion, shared geometry helpers and
the prefab helper refactor. No Cargo command or production/package mutation
was performed by this reviewer.

The independent payload deliberately uses `TerrainCollisionMesh` and
`MSCOL001`: positions and indices only, with no invented renderer normals,
UVs or tangents. Schema four requires its collision table; schemas two/three
cannot claim one. Metadata admission rejects duplicate qualified collider
identities, conflicting owner layers and mesh-record aliases, disagreement
with retained MeshCollider metadata, invalid source identities, nonfinite or
singular affine transforms, and invalid primitive fields. Payload admission
requires exactly the referenced mesh-record set, finite vertices, complete
in-range triangle indices, unique names, bounded counts, complete data and no
trailing bytes. These checks establish internal consistency rather than source
inventory completeness; full APK recapture establishes the latter for the
supported Mesh/Box scene inventory.

Capture keys decoded geometry by qualified mesh identity and collider entries
by qualified component identity, retaining B-32 components 10 and 11 separately.
It traverses parent activity and attached Rigidbody identity before conversion.
The existing local-to-world reflection contract and one additional winding
change for negative table determinant are consistent. Shared insertion prepares
every admitted active solid shape before changing terrain handles, trigger
layers or the source-component handle map. Disabled/inactive entries remain in
the package and triggers retain query metadata without invented gameplay.
Prefab conversion keeps the previous eight-corner geometry and mesh winding
behavior, with stricter finite input validation and an f64 determinant.

The importer verifies source layers before capture, copies/hash-checks every
retained dependency into an independent stage, verifies unchanged visible
records and complete captured metadata/local geometry, rechecks APK/manifest
identity, and publishes through the existing predecessor-archive/rename rollback
helper. Its repeat path verifies the capture without creating another stage or
backup. Layer enrichment preserves schema four instead of downgrading it;
fresh renderer-only pass two remains schema three; augmentation rejects a
changed schema-four scene set before staging unless coverage is already whole.

The standalone source collision verifier intentionally creates an empty global
index. It can resolve local and same-bundle shared-assets pointers, including
B-32, and rejects unresolved cross-bundle pointers. Successful root-owned full
verification against the actual APK is needed to establish that this restriction
admits this complete source capture; this review does not claim it ran that
verification.

One metadata-admission gap was raised with root before publication: finding a
class-64/65 object and following its `m_GameObject` does not by itself prove that
the owner's `m_Component` list attaches it. Require the collider and selected
Transform exactly once in that list, and every traversed ancestor Transform
exactly once in its own owner's list. This excludes orphan or duplicate
attachments from a purported source inventory. Node field selection from the
existing Rust diagnostics independently confirmed that all 16 Mesh/Box
colliders in A-46, Z-01 and B-32 satisfy these checks, including all six target
components and every traversed ancestor; the remaining bundles need the
importer's full check.

Root subsequently added `component_ids` and `require_attachment`. The follow-up
review confirmed fully qualified resolution, exact-once collider/selected
Transform checks before capture, and exact-once ancestor Transform checks before
Rigidbody admission. The new fixture covers missing, duplicate and same-PathID
different-CAB matches. This resolves the metadata-admission finding. No remaining
code-review blocker was found; root owns compilation, the full APK import and
independent source verification, and this reviewer did not execute their tests.

The review does not establish PhysX cooking/contact/tolerance, runtime scene
activation or original-game trigger policy, and does not claim original-runtime
parity.

## Actual import outcome

Reviewed root-generated `import.json` and `source-verification.json` plus their
logs, the published manifest and archived predecessor. Independent JSON checks
recounted **404 unique source component identities: 387 MeshColliders and 17
BoxColliders, with 389 active solids, 12 active layer-2 triggers and 3 inactive
entries**. Capture covers all 242 audited terrain scene bundles and references
364 decoded collision meshes. The 21 previously unrepresented entries are
exactly six active solids, twelve triggers and three inactive MeshColliders
(B-15/14, B-30/14 and B-37/15); no activity was inferred for the latter.

The six solids are A-46 Box34, Z-01 Boxes34–37 and B-32 Mesh10. Published B-32
Mesh10 and Mesh11 have distinct qualified component identities; all other
entry fields agree and both reference sharedAssets Mesh5, with one local record
containing 464 vertices and 682 triangles. The restored B-32 component remains
duplicate geometry, not a newly discovered unique surface.

Canonical schema four was compared with archived schema three at
`.local/map-import-backups/1794438051734897066/terrain`. All 400 renderer records,
source APK identity and render-geometry identity are structurally unchanged:
288 terrain and 112 other instances, 567,906 vertices and 1,005,341 triangles.
The report's 970 unique preserved payload identities exactly equal all prior
manifest dependencies. Independent hashing confirms the published manifest,
render/collision payloads and predecessor manifest/render payload agree with
their reported identities.

| Identity | SHA-256 |
|---|---|
| Import report | 8c61369943ed55f28e27086d43b9cf84bdba2a415c8be917732255c41b68b7c3 |
| Published manifest | 88e8da4f66c27a75a74cf418725a3f3b8b2a683f4b57817a11314979e10aea18 |
| Predecessor manifest | 2cf1a5100b16516ad9ebaf19d6fab467bd6fac7e93e13b9e30f4e2fe6a0a98cf |
| Unchanged render geometry | ab54dffcd939dd6bde74e05b115436f94d514c76825aaf5b57b3c5118dda4836 |
| Independent collision geometry | 50b0246734ab71ccc20ef43292805e33d0c5ce2471dc237d265e08b7c076edbf |

Root's fresh `--verify` completed with source layer/collision value checks true
and the same component/instance counts against the pinned APK. Thus the
standalone verifier's same-bundle resolution admits this actual complete
capture. No actual-import review blocker remains. This reviewer ran no Cargo
commands. Original runtime activation, triggers, PhysX cooking/contact/tolerance,
other collider classes, scene-info MapObject placements and visual rendering
parity remain separate work; `full_runtime_collision_parity_verified` remains
false.
