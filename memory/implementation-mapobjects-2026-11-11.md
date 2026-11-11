# MapObject milestone implementation — 2026-11-11

This continuation implements source-prefab loading/collision, bounded placement
readers/discovery and the omitted Z-01 terrain scene on Linux with Bevy 0.20.1.
It builds on [the earlier implementation validation](implementation-2026-11-11.md).
No Python, Android build, system cleanup or source-data deletion was performed.

## Published terrain

Canonical `assets/imported_map/terrain/` now contains 242 source terrain scenes,
400 render instances and 383 enabled solid MeshCollider records. The added
`Z_01_ID_639504_` is taken from `assets/scene/z-01.ab`, including its complete
source hierarchy transform, four-layer material/texture mips and MeshCollider.
Its bounds are X[-3639,-3127], Y[-7.101078,26.990854], Z[-1269,-757].

The new manifest SHA-256 is
`48aa616669b2aa71c8187a6c349db961bc363584dc62dc414de3ea4828f68391`;
geometry SHA-256
`ab54dffcd939dd6bde74e05b115436f94d514c76825aaf5b57b3c5118dda4836`.
All previous records/source metadata were compared unchanged before publishing.
The previous canonical package remains at
`.local/map-import-backups/1794424008828/terrain/`. Repeating the bounded command
verifies the already-published result without replacing it.

See [terrain import evidence](terrain-z01-2026-11-11.md),
[omitted-scene review](evidence/omitted-scenes-review.md) and
[the paired Z-01 GPU views](evidence/environment-z01-bevy/README.md).
The other source-list entry is the main-menu background. All 16 military
terrain bundles were already included. Z-01's identity as the particular
neighboring island still needs a matched original-game landmark view.

## Shared prefab package and Bevy rendering

`midnight_map_data` now owns the existing prefab DTOs and loader. It checks
all retained file hashes, package-contained paths, geometry and mip archives,
source component ownership/order, GameObject name/activity/layer, Transform
TRS/parent/child links, source renderer material order and exact mesh-part IDs.
It composes unsorted hierarchies, retains source identity/evidence and rejects
cycles, corrupt or singular transforms, conflicting records and missing data.
The real supermarket still verifies: 303 nodes, 298 renderers, 46 unique mesh
parts, three distinct collision meshes and 1,314 retained files.

These checks establish a consistent imported package. The loader does not
independently decode all Unity serialized bytes or re-authenticate derived
fields against the APK; authenticity depends on the pinned Rust import
provenance and retained manifest digest.

The Bevy renderer retains hierarchy/local transforms and source flags, loads
source 2D mips/samplers, conjugates texture transforms for top-left UVs, and
selects the source non-fading LOD thresholds through a marked perspective
camera. Selection runs after transform propagation and before inherited
visibility propagation. The full-plugin headless regression exercises camera
movement, child visibility, a hidden parent and invalid camera selection in
the same frame. Missing/multiple marked cameras are explicit errors. Its URP
Lambert/SimpleLit presentation is a declared diffuse Bevy adaptation. One glass
part with a Cubemap remains explicitly unsupported. Strict preparation rejects
it before changing assets; partial diagnostic preparations cannot spawn runtime
placements. The preview stages all source nodes and prepares 297 supported
parts, with 296 visible near parts and two far parts.

The initial prefab captures exposed wall shadow speckling during manual review.
Contact-shadow isolation and finer/thinner ray presets show that the optional
screen-space contact rays caused the dots; reducing thickness did not produce
a stable clean result across camera distances. The shared desktop policy now
omits `ContactShadows` on the camera and disables that light effect, retaining
directional shadow maps. Production and both probes consume the same setting.
The initial snapshot and tuning captures are retained separately from the
accepted captures. Numerical cast-shadow differences alone are insufficient
for appearance acceptance.

## Shared collision

`CollisionWorld::replace_source_prefab` prepares every supported shape before
replacing a stable instance and refreshing the BVH. It assigns the selected
root placement transform, composes local descendants, reflects raw BoxCollider
centers once and uses separately converted source mesh triangles. Active
ancestry and source enabled/trigger flags control solid query geometry. Failed
replacement preserves the previous instance and query index.

The supermarket builds 55 solid colliders and reports one trigger. Independently
built client/server query worlds have matching collider bounds and clearance
results. Rebuilding the canonical island also preserves spawn/clearance results
on Z-01. The shared crate now explicitly enables its Bevy logging dependency so
its isolated test build can compile the existing initialization error path.

This is geometry-derived Rapier query collision. PhysX cooking/cleaning/welding,
nondefault layer/material rules, sensor callbacks, dynamic bodies and source
runtime activation/state are not reproduced. Current supermarket boxes have
no observed shear; affine convex-box behavior under shear is not established
as Unity BoxCollider parity. No prefab coordinates are automatically inserted
into the playable world or network contract.

## Placement discovery

The strict plain SceneInfo/CellSceneInfo decoder preserves exact payload/record
byte spans and hashes, source position/scale/angles, signed prefab indices,
independent string tables, typed grass/tree/stealth/object lists, brightness,
IDs, lightmap settings and UV bytes. It rejects invalid counts, references,
non-finite values, truncation and trailing bytes. Synthetic tests prove reader
behavior; no actual placement body has been authenticated.

Discovery examined 9,430 APK member names, FileList paths, selected source
metadata literals, topology and bounded bundles. No sceneinfo/objectInfo
carrier was found there or in the task-owned project/review caches. The selected
`scenemaxmin.oc` fails this layout at byte 9. The legacy `MSPLC002` placement
archive retains separate, unverified provenance and was not activated.
The exact installed-game Waydroid cache directories are protected: traversal
was denied and noninteractive sudo required a password. An authorized host
ADB check started its incidental daemon but found no attached devices; the
known local endpoint timed out after 12 seconds. The graphical session is
stopped. No game/container/service was started or permissions changed. The
cache contents remain unknown; its absence is not established.
Exact remote path binding, IFix runtime overrides and source Euler composition
remain open. See [placement reader/discovery](world-placements-2026-11-11.md),
[source audit](mapobjects-source-2026-11-11.md) and
[cache search](evidence/placement-cache-search.md).

## Rust validation

| Check | Observed result |
|---|---|
| Offline locked workspace all-target check after the final shadow/LOD changes | Passed |
| Offline locked workspace tests after the final shadow/LOD changes | 151 passed, ten intentionally ignored |
| Full-plugin same-frame prefab visibility regression | Passed, including child/parent/camera transitions |
| Client library strict Clippy with `--no-deps -- -D warnings` | Passed |
| Workspace all-target Clippy | Completed with the same five inherited warnings; no new warning in this continuation |
| Scoped formatting of prefab loader/collision/renderer, environment and both capture probes | Passed |
| Retained supermarket shared loader | Passed: 303 nodes, 298 renderers, 46 mesh parts, three collision meshes, 1,314 retained files |
| Retained supermarket shared collision | Passed: 55 solid colliders, one trigger; independent query worlds agree |
| Retained canonical-world rebuild | Passed: 400 render instances, 383 solid colliders, matching spawn/clearance including Z-01 |
| Independent extractor tests and strict all-target Clippy | 41 tests passed; strict Clippy passed |

The retained-asset and extractor checks above were completed earlier in this
continuation; subsequent changes only affected Bevy presentation, its LOD
regression and documentation. Ordinary workspace tests omit the retained-asset
cases, real-UDP acceptance and sustained capacity/timing scenarios. Earlier
desktop/network/UI checks remain recorded in the predecessor validation.
The five inherited Clippy warnings are in `map_data/src/package.rs`,
`shared/src/physics.rs` and `client_core/examples/terrain_probe.rs`.

## Accepted prefab GPU result

The corrected desktop preset passed all seven Linux/Vulkan prefab capture
views on the RTX 4060 Laptop GPU. The probe checks 303 composed node poses,
12 source mip textures, strict rejection before asset mutation and rejection
of partial preparations on the runtime placement path. It shows 296 near
parts and two far-LOD parts. Both repeated wall views contain zero changed
pixels above the eight-level RGB-channel threshold. Disabling only directional
shadow maps lightens 10,819 lower-frame pixels, so the correction preserves
cast shadows.

Manual review of the accepted near and close views confirms that the black
wall dots are removed and cast shadows remain on the air conditioner, crates
and floor slabs. See [accepted prefab captures](evidence/prefab-shadow-fixed-bevy/README.md)
and [their exact report](evidence/prefab-shadow-fixed-bevy/render.json).
The [initial rejected result](evidence/prefab-bevy/README.md) is preserved.

The final desktop executable was rebuilt after the correction; its predecessor
is retained at `.local/runtime-backups/pre-contact-shadow-fix/client_runner`.
The repository filesystem still has approximately 128 GiB available. No
cleanup of source exports, user files, system directories or Cargo caches
was needed or performed in this continuation.

Current commands and the implementation ledger are in [README](../README.md)
and [the plan](../PLAN/source_reconstruction.md). Android remains deferred.
