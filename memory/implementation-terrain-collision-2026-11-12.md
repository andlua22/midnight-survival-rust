# Independent source terrain collision — 2026-11-12

The canonical terrain package now uses **manifest schema 4** with a collider
table independent of visible meshes. The source-verified table contains
**404 components: 389 active solids, 12 active triggers and three inactive
components**. It restores the six solids omitted by the renderer-driven import
while preserving all 400 render records, their positions and the existing
geometry/texture payloads. The preceding
[layer milestone](implementation-terrain-layers-2026-11-12.md) remains recorded.

## Implemented boundary

[The collision schema and reader](../crates/map_data/src/terrain_collision.rs)
retain qualified collider, GameObject, Transform and mesh identities, source
layers, enabled/trigger flags, saved local and ancestor activity, complete
world transforms and original Box center/size or Mesh cooking fields. Schema 4
requires the independent table and its hash-verified collision-only archive.
Schemas 2 and 3 remain readable through their earlier collision boundary;
schema-2 layers remain unknown. Render geometry still uses `MSMAP003`.
The separate `collision-meshes.bin` uses `MSCOL001` and contains 364 unique
source meshes with positions and triangle indices.

[The Rust import action](../tools/asset_unpacker/src/terrain_collisions.rs)
captures Mesh/Box components in all 242 selected terrain scene bundles,
verifies owner component attachments and every saved Transform ancestor,
and rejects unsupported collider classes or Rigidbody ancestors. Mesh geometry
is decoded once per qualified mesh identity; component instances remain
distinct. Source comparisons cover the complete table and decoded local
geometry. Staging preserves and verifies 970 prior payload files and checks
unchanged render records before publishing. The old package is archived before
replacement. Repeating the action compares the current package against the
source and leaves an already imported package unchanged.

[Shared terrain insertion](../crates/shared/src/physics/source_terrain.rs)
now installs the independent active solid components with their source layers
and qualified handle identities. It prepares all shapes and metadata before
replacing terrain, preserving players and prefabs on success or failure.
Box centers and collision-local meshes receive the Unity/Bevy reflection once;
the complete affine hierarchy places them in world coordinates. Triggers retain
layer metadata and remain physically unrepresented. Inactive components remain
in the package without becoming installed solid handles.

## Restored source components

The source inventory contains 387 MeshColliders and 17 BoxColliders. Its
previously omitted records are five active solid Boxes, one active solid Mesh,
12 layer-2 Box triggers and three inactive layer-0 Meshes. Saved ancestors of
all six restored solids are active; all six use layer 0.

| Scene / source CAB | Component | Owner GameObject | Shape |
|---|---:|---:|---|
| A-46 / BuildPlayer-A-46 | 34 | 3 | Box |
| Z-01 / BuildPlayer-Z-01 | 34 | 1 | Box |
| Z-01 / BuildPlayer-Z-01 | 35 | 3 | Box |
| Z-01 / BuildPlayer-Z-01 | 36 | 4 | Box |
| Z-01 / BuildPlayer-Z-01 | 37 | 5 | Box |
| B-32 / BuildPlayer-B-32 | 10 | 2 | Mesh |

B-32 components **10 and 11** have the same owner and reference the same
`road_b32` mesh, `BuildPlayer-B-32.sharedAssets:5`. They retain two distinct
collider identities and handles while sharing one decoded mesh payload.
Restoring component 10 adds a source instance of an existing surface; it does
not establish an additional unique road surface. The five Boxes retain their
individual saved rotations/scales and ancestor transforms.

Exact values, qualified identities and bundle hashes are retained in
[import.json](evidence/terrain-collision-source/import.json) and
[the independent source review](evidence/terrain-collision-source/importer-review.md).
The report's older `enabled_source_colliders: 383` field describes the retained
renderer-associated records; `independent_active_solids: 389` describes the
new runtime table.

## Publication identities

The immediate schema-3 predecessor is retained at
`.local/map-import-backups/1794438051734897066/terrain`.
The earlier schema-2 backup and previous evidence remain available.

| Payload | SHA-256 |
|---|---|
| Source APK | `2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df` |
| Previous manifest | `2cf1a5100b16516ad9ebaf19d6fab467bd6fac7e93e13b9e30f4e2fe6a0a98cf` |
| Current schema-4 manifest | `88e8da4f66c27a75a74cf418725a3f3b8b2a683f4b57817a11314979e10aea18` |
| Unchanged render geometry | `ab54dffcd939dd6bde74e05b115436f94d514c76825aaf5b57b3c5118dda4836` |
| New collision geometry | `50b0246734ab71ccc20ef43292805e33d0c5ce2471dc237d265e08b7c076edbf` |

The changed manifest identity is covered by the existing client/server content
check; both peers must load matching packages.

## Validation and remaining dependencies

The first source import succeeded with `source_values_verified: true` and all
400 render-record layers verified. Confirmed checks include:

- 50 extractor tests and strict extractor Clippy passed; scoped rustfmt passed
  for 11 selected Rust files.
- Fresh source/package verification passed, and the repeat action was a no-op:
  matching manifest/table/geometry, no replacement and no additional backup.
- Preservation verification checked 1,940 hashes across the 970 prior payloads
  (603,921,092 bytes) and complete equality of all 400 renderer metadata records.
- Retained real schema-3 and schema-2 packages passed source verification.
  Schema 2 retained unknown layers and neither legacy package claimed an
  independent collider table.
- Each separately selected real-package shared test passed: complete independent
  collision (389 solid handles, 404 entries and 12 skipped triggers), terrain
  layer metadata, and matching canonical collision/spawn in independent worlds.
- Workspace tests passed: 218 passed, zero failed and 12 ignored.
- Final map-data tests passed: 36 passed and one ignored.
- All-target workspace check and strict shared-library Clippy passed.
- Map all-target Clippy passed with `-A clippy::collapsible_if` for three
  inherited warnings. The initial strict failure and reviewed allowance are
  retained in the evidence; strict extractor Clippy needed no allowance.

The [validation ledger](evidence/terrain-collision-source/README.md) records
the exact command scope, code/report identities, preserved initial failures
and final executable results. Earlier GUI/GPU evidence retains its original
scope; this import adds no new rendering parity claim.

This milestone establishes saved static Mesh/Box component coverage in the
selected terrain scenes. Runtime activation/streaming, other collider classes,
trigger behavior, PhysX cooking/tolerance and initial-overlap semantics still
need source-runtime verification. Convex terrain meshes and Rigidbody ancestry
are rejected rather than approximated. Full original collision parity remains
explicitly false in the import report.

Continue with the actual player rig and source runtime binding: recover
RootBone/HeadTop references, effective poses/scaling and player collision layers;
resolve global trigger policy and the distinct CheckCanPaUp cast/ray conditions;
then recover FSM switch arbitration before active posture, movement, UI and
audio bindings. Authenticated MapObject/TreeObject/BuildingObject/GrassObject
placements remain unknown. This collision import supplies no guessed placements
and does not enable an inferred source player controller. Linux remains the
test target; Android is deferred. The final read-only disk check found about
126 GiB free. No Python, cleanup or system changes were needed for this milestone.
