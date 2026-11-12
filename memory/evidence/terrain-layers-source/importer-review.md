# Terrain layer enrichment: importer and coverage review

2026-11-12. Read-only review of the existing Rust importer, resolver, package
reader and shared content identity. No Cargo, Python, original game, GUI,
source service or APK extraction was run by this reviewer. This note records
code observations and acceptance criteria; it does not report observed APK
collider counts or claim original-runtime parity.

## Existing boundary

`pass2_terrain::is_chunk` selects the audited 242 terrain scene members,
including Z-01 and the 16 Military Base terrain bundles. `collect_scene`
restricts placed instances to serialized files containing RenderSettings
(class 104), deliberately excluding shared-assets prefab definitions.

The collector scans Transform, MeshRenderer, MeshFilter and MeshCollider
tables, then emits geometry from admitted renderers. Its collider map is keyed
by the qualified GameObject, with `insert` replacing a previous MeshCollider
on the same owner. Only local GameObject `m_IsActive` and renderer `m_Enabled`
are checked. Collider-only owners, additional colliders and effective parent
activity therefore cannot be inferred from the exported render-record count.

`Resolver::object` resolves a signed PathID within a selected CAB and APK
bundle. `Resolver::resolve` uses the pointer's actual external-file table and
checks CAB ownership. Reads use strict TypeTree parsing; raw-data fallbacks
are rejected. `GlobalIndex` has no embedded APK identity, so enrichment must
validate actual ObjectInfo class IDs rather than trust index labels.

The resolver's decoded bundle cache retains at most 16 entries / 256 MiB;
caller-held Arcs have separate lifetimes. APK member and decompressor limits
apply before payload allocation. One scene at a time avoids retaining all
242 decoded bundles. Reading GameObject/collider/Transform metadata does not
require mesh, texture or material decoding.

## Source coverage audit

For every selected scene-CAB, enumerate relevant collider object-table entries,
resolve each collider's qualified `m_GameObject`, and require a class-1 owner.
Read a strict integer `m_Layer` in 0..31, source enabled/trigger flags, and
MeshCollider mesh/convex/cooking fields. Retain collider identity separately
from GameObject identity; group multiple colliders without overwriting them.

Compare the qualified collider-ID set to the manifest's deduplicated collider
set. Report represented IDs, omitted IDs, collider-only owners and owners with
multiple colliders. Keep disabled, locally inactive and trigger categories
separate; if effective parent activity is not resolved, label it unresolved.
Other source collider classes require their own coverage/conversion treatment.
New GameObject layer coverage alone does not establish complete collision
coverage or authentic dynamic runtime layers.

## Compatibility and publication

Schema 3 can require known valid GameObject layers on every SceneMesh. Legacy
schema 2 must keep layers explicitly unknown, distinct from source layer 0.
Qualified GameObject layer values must agree across submeshes. MSMAP003
geometry need not change. Schema-2 Z-01 augmentation must keep added layers
unknown, or require a separate explicit enrichment; mixing known fields into
legacy schema 2 would contradict the new reader contract.

Stage outside the canonical package. Copy only package-path-validated retained
geometry/texture dependencies, verify unchanged geometry and texture identities,
and compare previous source metadata with layers removed. Validate each new
layer against the actual source object. Re-hash the APK and canonical manifest
before `publish_verified`; preserve the predecessor and retain failed stages.

Enrichment changes the terrain manifest SHA-256 while geometry can stay
identical. Shared WorldDescriptor equality already checks manifest identity;
client and server must select the same new package. Tests should exercise
legacy unknown versus layer 0, partial/corrupt/conflicting metadata, mask and
trigger behavior, rollback and metadata cleanup, and the real-package source
comparison. Renderer-driven extraction alone cannot justify activating a
complete source posture controller.

## Implementation review

Reviewed the new terrain-layer extractor and CLI, schema admission, legacy
augmentation, source verification and shared terrain-query integration.
No Cargo or real APK enrichment was run by this reviewer.

The new implementation reads class-1 GameObject layers directly from qualified
object identities; verification using an empty GlobalIndex is valid because
that layer path performs no external pointer resolution. Layer validation
preserves legacy unknown values, rejects invalid/partial values, and checks
GameObject/collider layer agreement across submeshes. Shared terrain shapes
and layer/trigger metadata are prepared before the replacement commit.
Source omissions remain reported without asserting full runtime coverage.

Review requested two correctness fixes before publication:

- Compare every retained occurrence of a collider ID against its source
  component. Deduplicating to one last record before comparison can hide
  different owner/flag/mesh/cooking metadata on an earlier submesh record.
- Capture and check the canonical manifest hash around initial package load,
  before the APK hash/scan. Capturing it only after loading can pair an old
  loaded package with a newer canonical identity and miss a replacement race.

The coverage field initially called `collider_only_owners` represented owners
without retained render records, which also includes filtered renderers.
Requested an exact label or a separate source-renderer owner audit.
Parent will record the implemented corrections and executable validation.

The corrected sections were re-reviewed before real enrichment: all retained
collider occurrences are now compared with actual owner/component fields;
the initial canonical hash brackets package loading and is retained through
publication checks; the coverage field is now named
`collider_owners_without_retained_render_records`. The added submesh test
includes an earlier invalid enabled flag and a mismatched owner. No remaining
production-review blocker was found within this metadata-enrichment scope.
Executable checks and actual APK/package results remain the parent's validation.

## Recorded source inventory result

The parent's [enrichment report](enrichment.json), SHA-256
`8aea48d8af6833278e2364ff6229bc555e9eeba7bd1361e7b91d98554aa29be2`,
was read as structured JSON without printing its retained-file inventory.
It records `export/lite4.0.apk`, SHA-256
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`,
242 source bundle hashes and 242 scene serialized files. Schema 3 contains
400 known-layer render records /
400 qualified GameObjects: 113 on layer 0 and 287 on layer 18.

The source inventory contains **404 colliders: 387 Mesh and 17 Box**.
Its **21 omissions** classify as follows; every omitted collider is enabled
in the saved source fields:

| Omitted source type | Layer | Owner locally active | Trigger | Count |
|---|---:|---|---|---:|
| Box | 2 | true | true | 12 |
| Box | 0 | true | false | 5 |
| Mesh | 0 | true | false | 1 |
| Mesh | 0 | false | false | 3 |

The five locally active solid Boxes are A-46 collider 34 and Z-01 colliders
34, 35, 36 and 37, qualified by their scene CABs. The locally active omitted
MeshCollider is B-32 collider 10, owner GameObject 2; that owner also has
retained MeshCollider 11. This directly demonstrates why one collider per
GameObject cannot represent the complete source component set. The three
locally inactive MeshColliders are B-15 collider 14, B-30 collider 14 and
B-37 collider 15, each on GameObject 2 of its qualified scene CAB.

Layer 18 and the twelve omitted layer-2 triggers are outside the exact standing
mask (0, 19, 30). The six locally active omitted solids are on included layer 0.
Their absence therefore matters before gameplay can claim complete standing
clearance. Parent activity, dynamic activation and streamed/runtime collision
remain unresolved; the report correctly keeps full runtime coverage false.

## Concrete next collision representation

Add a dedicated scene-collider table/package keyed by qualified collider
component identity, independent of render records and GameObject cardinality.
Retain owner/Transform hierarchy, layer, enabled/trigger/local-active fields
and original collider parameters. Resolve effective saved hierarchy activity
before selecting static shapes; preserve the three inactive records separately
without inventing runtime activation.

For the five solid Boxes, retain raw center/size and source transforms and
reuse the established shared prefab Box conversion. For B-32 collider 10,
resolve its actual `m_Mesh`/convex/cooking parameters, decode the complete
collision mesh once under its qualified identity, and retain a hash-verified
collision-only buffer. Do not assume it matches collider 11 or use only a
render submesh. Keep both component identities on owner 2.

Instantiate these records through one atomic shared collision batch. A
collision-only record needs a collider and metadata, with no Bevy Mesh3d,
material or visible render entity. Preserve the current render archive and
texture hashes. Retain layer-2 trigger parameters for later trigger behavior,
and validate the new package/counts/source fields before publication and
client/server content-identity changes. This is the next source collision
boundary; physical runtime parity and active player binding remain separate.
