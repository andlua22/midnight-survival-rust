# Source terrain collision layers — 2026-11-12

Terrain layer extraction, compatible package admission and shared query metadata
are implemented. The canonical package now has manifest schema 3 with all
400 GameObject layers compared against the actual APK objects. MSMAP003
geometry, textures, world positions and the existing 383 solid collision
records are preserved. The previous schema-2 package remains at
`.local/map-import-backups/1794435897062652886/terrain`.

## Implemented boundary

[SceneMesh](../crates/map_data/src/lib.rs) now retains a numeric Unity
GameObject `layer: Option<u8>`. [Layer admission](../crates/map_data/src/terrain_layers.rs)
accepts schema 2 only with unknown layers and schema 3 only with complete
0..31 values. Missing, malformed, out-of-range or conflicting qualified
GameObject/collider layers reject. Legacy unknown is never replaced with zero.
The shared package loader calls the same admission used by in-memory physics.

[The Rust enrichment action](../tools/asset_unpacker/src/terrain_layers.rs)
resolves deduplicated qualified GameObject references, requires actual class-1
objects and strictly reads `m_Layer`. It audits collider tables in all 242
terrain scene bundles and compares every retained collider occurrence with
source owner, mesh and flags. Staging copies 970 verified retained files:
the geometry archive and 323 sets of PNG, encoded texture and decoded mip
payloads. It decodes no new mesh/texture data. Source/canonical identity checks
and staged validation precede publication; its predecessor is archived.
Repeating the action verifies the source again and leaves the package unchanged.

Fresh pass-2 imports emit schema-3 layers. The separate Z-01 augmentation
preserves unknown layers when operating on schema 2. Ordinary terrain
verification now compares schema-3 values directly with the source APK;
legacy verification explicitly reports unknown layers. The diagnostic terrain
probe reports the actual loaded manifest version.

[Shared terrain insertion](../crates/shared/src/physics/source_terrain.rs)
stages shapes, solid layers and skipped-trigger metadata before replacement.
Replacement removes old terrain metadata while preserving players and prefabs.
Known solid layers enter the existing standing-cast mask; legacy solids remain
unresolved. Enabled skipped triggers retain optional layers and are deduplicated
by qualified component identity. An unknown legacy trigger gives
`UnknownTriggerLayers` when global trigger queries are enabled; a selected
known trigger still gives `UnrepresentedTriggers` until its geometry is supplied.

## Source results and remaining coverage

The 400 retained GameObjects split into **113 on layer 0 and 287 on layer 18**.
Layer 18 is outside the native standing mask (layers 0,19,30); its exclusion
is deliberate source behavior. All retained 383 solid components are
MeshColliders, and the independent shared worlds classify all 383 handles.

The broader scene inventory in the five audited collider classes contains
**404 colliders**: 387 Mesh and 17 Box.
The renderer-driven import omitted these 21 source components:

| Omitted saved-source category | Count | Layer |
|---|---:|---:|
| Enabled Box triggers with locally active owners | 12 | 2 |
| Enabled solid Boxes with locally active owners | 5 | 0 |
| Enabled solid MeshCollider with locally active owner | 1 | 0 |
| Enabled solid MeshColliders with locally inactive owners | 3 | 0 |

Twenty owners have no retained render record. B-32 GameObject 2 has two
MeshColliders; the prior collector kept component 11 and omitted component 10.
Local activity is only the saved GameObject flag; effective parent activity,
runtime generation/streaming and activation are not established by this audit.
Six omitted solids have active local flags and participate in the standing
mask, so full original collision coverage remains unresolved. No active source
crouch/controller binding is enabled by metadata enrichment.

The exact omitted identities and source bundle hashes are in
[enrichment.json](evidence/terrain-layers-source/enrichment.json).
[Review notes](evidence/terrain-layers-source/importer-review.md) retain the
source inventory limits and implementation review.

## Validation

- 208 workspace tests passed, with 11 ignored; 46 extractor tests passed.
- The separately selected real-terrain test passed: 383 solid handles with
  layer metadata and matching collision bounds in independent shared worlds.
- Fresh APK/package verification passed. Repeat enrichment preserved manifest,
  geometry, retained-file identities and source results, and created no backup.
  The preserved real schema-2 package also passes verification with all 400
  layers still explicitly unknown.
- Strict extractor and shared-library Clippy passed. Map all-target Clippy
  passed with the inherited `collapsible_if` lint excluded; its three existing
  warnings and initial strict failure are retained in the evidence.
- Scoped rustfmt and whitespace checks passed. All-target workspace check is
  recorded in the final [validation ledger](evidence/terrain-layers-source/README.md).

Original APK SHA-256 remains
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
Geometry SHA-256 remains
`ab54dffcd939dd6bde74e05b115436f94d514c76825aaf5b57b3c5118dda4836`.
The new manifest SHA-256 is
`2cf1a5100b16516ad9ebaf19d6fab467bd6fac7e93e13b9e30f4e2fe6a0a98cf`;
the existing client/server content-identity check therefore selects the new
package consistently. Earlier GUI/GPU evidence retains its original scope.
This pass needed no original-game execution, Android build, Python, cleanup
or system changes. About 127 GiB remains free.

## Next implementation work

Add a qualified collider table independent of render records. Preserve source
transforms, activity/flags and layers; import collision-only Box/mesh geometry
without creating visible render entities. Start with A-46 Box 34, Z-01 Boxes
34–37 and B-32 MeshCollider 10. Retain the three inactive records and layer-2
trigger records with explicit unresolved activation/behavior. Then recover
the actual player rig, RootBone/HeadTop poses, player layers, global trigger
policy, PaUp ray dependencies and FSM arbitration before gameplay bindings.
