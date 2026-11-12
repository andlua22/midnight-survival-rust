# Independent terrain source collision: import and validation

2026-11-12. The canonical terrain package now uses **schema 4** with an
independent saved Mesh/Box collider table. Its **404 qualified component
entries** produce **389 active solid components**, with **12 source triggers**
retained as query metadata and **3 inactive components** retained without
activation. The visible terrain remains **400 render records** across
242 selected scene bundles.

The table restores all 21 previously unrepresented collider records. Six
are active layer-0 solids: A-46 BoxCollider 34, Z-01 BoxColliders 34–37,
and B-32 MeshCollider 10. The latter shares geometry with B-32 MeshCollider
11 but retains its own component identity and runtime handle. These are six
restored component instances, including five Box geometries and one duplicate
Mesh instance.

## Package and retained payloads

[import.json](import.json) records the source APK, all 242 source bundle
hashes, captured component counts, 364 distinct collision mesh records,
970 preserved payload identities and the successful publication. The prior
schema-3 package is retained at
`.local/map-import-backups/1794438051734897066/terrain`; the earlier schema-2
backup remains separate.

| Identity | SHA-256 |
|---|---|
| Preserved schema-3 manifest | 2cf1a5100b16516ad9ebaf19d6fab467bd6fac7e93e13b9e30f4e2fe6a0a98cf |
| Published schema-4 manifest | 88e8da4f66c27a75a74cf418725a3f3b8b2a683f4b57817a11314979e10aea18 |
| Preserved MSMAP003 renderer geometry | ab54dffcd939dd6bde74e05b115436f94d514c76825aaf5b57b3c5118dda4836 |
| New MSCOL001 collision geometry | 50b0246734ab71ccc20ef43292805e33d0c5ce2471dc237d265e08b7c076edbf |

The collision archive contains positions and triangle indices only:
**559,815 vertices / 994,873 triangles** across 364 qualified mesh records.
Renderer normals, UVs and tangents are not required by this independent
archive. Original renderer geometry and texture payloads remain retained.
[preservation-check.json](preservation-check.json) records an independent
comparison of all **1,940 current/backup file hashes** for the 970 preserved
payloads (**603,921,092 bytes** per package); all matched. It also confirms
unchanged visible records, renderer geometry and source APK identity.

[repeat-import.json](repeat-import.json) verifies the already imported package
without another backup. The manifest and source capture are unchanged; the
repeat preservation inventory includes the collision archive, totaling 971
payloads. Fresh preserved schema-3 and schema-2 checks remain readable with
zero independent collision-table entries. Schema 3 retains 400 known source
layers; schema 2 retains 400 explicitly unknown layers. Neither legacy package
claims independent source collision verification.

The supported saved scene inventory contains **387 MeshColliders and
17 BoxColliders**. Owner/component attachments, parent transforms, saved
ancestor activity and Rigidbody ancestry are checked before conversion.
Captured mesh geometry is shared by qualified mesh identity while collider
entries retain qualified component identity. Box centers remain raw Unity
local coordinates; the runtime reflects Z once and applies the complete
affine transform.

Shared insertion prepares all admitted active solids before replacing terrain
handles, source layers, skipped-trigger metadata or component-to-handle
identities. Invalid or unsupported shapes roll back without changing players
or imported prefab state. Triggers have no inferred gameplay effects.

## Source review and executable evidence

[importer-review.md](importer-review.md) retains the source inspection of the
six restored components, transform/decoder review and production integration
review. The reviewer identified an attachment-admission gap; root added
fully qualified exact-once collider/Transform attachment checks, including
every traversed ancestor. The follow-up review found no remaining code
blocker. This is static source/package review, separate from runtime parity.

- `extractor-tests-reviewed.log`: **50 passed**, no ignored tests; earlier
  46/49-test runs remain preserved.
- `extractor-clippy-reviewed.log`: strict all-target extractor Clippy passed.
- `map-shared-tests.log`: **36 map-data / 73 shared passed**, with one map-data
  and five shared tests ignored in this run.
- `map-tests-reviewed.log`: final map-only rerun after the two test-only
  clone sites were replaced passed **36 tests**, with one ignored.
- `retained-collision-tests.log`: the separate real-package test passed,
  checking 389 solid handles, 404 entries and 12 skipped source triggers in
  independent shared worlds, including distinct B-32 component handles.
- `retained-layer-tests.log` and `canonical-world-tests.log`: separately
  selected real-package layer and canonical-world reconstruction tests passed.
- `workspace-tests.log`: **218 passed, 0 failed, 12 ignored**.
- `workspace-check.log`: all-target workspace compatibility check passed.
- `shared-clippy.log`: strict shared-library Clippy passed.
- `map-clippy-strict.log`: the initial strict all-target map-data run failed
  on three inherited `collapsible_if` diagnostics and two new test-only clone
  diagnostics. Root fixed the two clone sites. The first allowed-baseline
  failure remains preserved; `map-clippy-baseline-reviewed.log` passed with
  the explicit `-A clippy::collapsible_if` allowance. This is not an
  unconditional strict package pass.
- `rustfmt-check-reviewed.log`: final scoped formatting passed for 11
  implementation files; the earlier formatting log remains preserved.
- `diff-check.log`: whitespace check passed.
- `import.log` and `import.json`: successful schema-4 import and predecessor
  preservation.
- `source-verification.log` and `source-verification.json`: a fresh APK
  verification reports source collision/layer values verified and confirms
  the 404/389/12/3 component inventory.
- `repeat-import.log`, both legacy verification logs and
  `preservation-check.log`: repeat, compatibility and payload preservation
  checks completed successfully.

The actual-import review independently recounted the table and confirmed
unchanged renderer records and package identities. Fresh source verification
also admits the complete captured table through the standalone verifier's
same-bundle resolver. No actual-import review blocker remains.

[validation.json](validation.json) records 49 final code/report/log identities,
completed validation and reproduction commands where invocation text is not
retained. All identities matched at finalization. The disk audit reported
126 GiB free; no cleanup was required.

The captured table covers the supported Mesh/Box inventory of the selected
shipped scenes. Runtime scene activation, streamed/generated colliders,
other collider classes, source trigger behavior, IFix patches and PhysX
cooking/contact/tolerance remain separate acceptance work. Full original
runtime collision parity is explicitly **false** in the import report.
The source player rig, posture/FSM binding and world MapObject placements
remain separate implementation dependencies. No Android build, original-game
execution, Python, cleanup or new GUI/GPU probe is claimed here.

[Implementation and continuation](../../implementation-terrain-collision-2026-11-12.md).

