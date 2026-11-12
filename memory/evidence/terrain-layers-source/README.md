# Terrain GameObject layers: source and Rust validation

2026-11-12. The canonical terrain now uses manifest schema 3 and retains
verified numeric Unity GameObject layers for all **400 render records / 400
qualified GameObjects** across 242 terrain scenes. Layer counts are **113 on
layer 0** and **287 on layer 18**. These are collision filter identities,
separate from terrain material/splat layers.

Legacy schema-2 packages remain readable with explicitly unknown layers.
Schema 3 rejects missing/invalid layers and conflicting values for a shared
qualified GameObject or collider component. Shared terrain collision retains
solid layers and skipped-trigger metadata atomically through replacement;
unknown legacy metadata does not become layer 0. The active source controller
and original player rig remain separate implementation work.

## Retained packages and payload identity

[enrichment.json](enrichment.json) records the APK identity, all source bundle
hashes, GameObject layers, collider inventory, retained file identities and
publication result. The prior schema-2 package is preserved at
`.local/map-import-backups/1794435897062652886/terrain`.

| Identity | SHA-256 |
|---|---|
| Prior terrain manifest | 48aa616669b2aa71c8187a6c349db961bc363584dc62dc414de3ea4828f68391 |
| Schema-3 terrain manifest | 2cf1a5100b16516ad9ebaf19d6fab467bd6fac7e93e13b9e30f4e2fe6a0a98cf |
| Unchanged MSMAP003 geometry | ab54dffcd939dd6bde74e05b115436f94d514c76825aaf5b57b3c5118dda4836 |

The enrichment resolves strict class-1 `m_Layer` values from existing qualified
GameObject references and scans saved scene collider metadata. It reuses mesh
and texture payloads without decoding those assets again. All **970 retained
files** comprise the mesh archive and three payloads for each of 323 textures
(PNG, encoded source data and decoded mip archive). Their identities are
retained in the report. The source bundle inventory records 242 SHA-256 hashes.
An independent comparison checked all 970 payloads in both the active and
backup packages: **1,940 file identities matched**, covering 603,921,092 payload
bytes per package. Existing mesh source metadata is identical after removing
the new layer fields; the geometry identity remains unchanged.

[enrichment-repeat.json](enrichment-repeat.json) records an already enriched
package with no additional backup. Its manifest, geometry, source inventory
(excluding cache counters) and retained-file identities match the first report.
[verification.json](verification.json) separately reports 400 known layer
records, no unknown layer records, and source layer values verified against
the APK. The current package retains **383 solid source collision records**.
[legacy-verification.json](legacy-verification.json) separately verifies the
preserved schema-2 package with 400 unknown layers, no known layers and
`source_layer_values_verified: false`, preserving the distinction from layer 0.

## Saved collider coverage

Within the five audited classes (64, 65, 135, 136 and 143), the metadata audit
enumerated **404 source colliders**: 387 MeshCollider and 17 BoxCollider.
WheelCollider, TerrainCollider and unknown classes are outside this inventory.
All 404 are enabled in their saved component fields; 401 have locally active
owners and three have locally inactive owners. Effective parent activity
and runtime changes are not inferred from the local `m_IsActive` value.
The represented set contains 383 locally active non-trigger MeshColliders.

The 21 omitted source colliders have these exact saved classifications:

| Source class | Layer | Locally active | Trigger | Count |
|---|---:|---|---|---:|
| BoxCollider | 2 | yes | yes | 12 |
| BoxCollider | 0 | yes | no | 5 |
| MeshCollider | 0 | no | no | 3 |
| MeshCollider | 0 | yes | no | 1 |

Twenty collider owners have no retained render record. The owner
`assets/scene/b-32.ab:BuildPlayer-B-32:2` has two MeshCollider components,
PathIDs 10 and 11; the renderer-driven package retains 11 and omits 10.
The six omitted locally active solid colliders on layer 0 participate in the
standing-clearance mask and require their own geometry/runtime admission. Layer enrichment alone
does not supply that coverage. The report explicitly records
`full_runtime_collision_coverage_verified: false`.

[importer-review.md](importer-review.md) records the publication/metadata review,
including the corrected per-occurrence collider comparison, canonical-load
identity guard and precise owner-without-retained-record label. The source
audit describes static serialized scene data, not original-game execution or
REA provider Evidence IDs.

## Executable validation

- `extractor-tests-reviewed.log`: final **46 passed**, no ignored tests.
  Earlier 43/45-test runs are preserved in their own logs.
- `extractor-clippy.log`: strict all-target extractor Clippy passed.
- `map-shared-tests.log`: **31 map-data and 68 shared tests passed**;
  one map-data and four shared tests were ignored in that run.
- `retained-terrain-tests.log`: separately selected real-package test passed,
  with **383 classified solid handles** and **0 skipped source triggers** in
  the retained package. This does not count the omitted source triggers above.
- `workspace-tests.log`: **208 passed, 0 failed, 11 ignored**.
- `workspace-check.log`: all-target workspace compatibility check passed.
- `shared-clippy.log`: strict shared-library Clippy passed.
- `map-clippy.log`: initial strict map-data Clippy failed on three inherited
  `collapsible_if` warnings. `map-clippy-baseline.log` passed with the explicit
  `-A clippy::collapsible_if` allowance. This is not an unconditional strict pass.
- Fresh schema-3 verification, repeated enrichment and preserved schema-2
  verification all completed successfully. Scoped rustfmt and whitespace
  checks passed.

[validation.json](validation.json) records commands, final code/report/log
identities and completed validation results. No Android build, new GUI/GPU
probe, original-game execution, Python, cleanup or system mutation was used
for this metadata milestone. Prior renderer captures keep their original
executable and validation scope.

[Implementation and continuation](../../implementation-terrain-layers-2026-11-12.md).
