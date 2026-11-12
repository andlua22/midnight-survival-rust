//! Independent source collider table and local collision-only geometry.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use glam::{Mat4, Vec3};
use midnight_map_data::{
    FileIdentity, TERRAIN_MANIFEST_VERSION, TerrainCollisionEntry, TerrainCollisionManifest,
    TerrainCollisionMesh, TerrainCollisionShape, TerrainPackage, file_sha256, load_terrain_package,
    package_path, validate_collision_mesh, write_collision_archive,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    bundle_cache::CacheStats,
    fields::{array, boolean, integer, invalid, property, vector},
    mesh,
    package_publish::publish_verified,
    pass1_index::GlobalIndex,
    pass2_terrain::is_chunk,
    resolver::{ObjectId, Resolver},
    terrain_layers::{game_object_layer, retained_files},
    terrain_verification::{self, TerrainVerification},
    unity::{Error, cab_name, source_apk},
};

#[derive(Debug, Serialize)]
pub struct CollisionMeshSummary {
    pub record: String,
    pub vertices: usize,
    pub triangles: usize,
}

#[derive(Debug, Serialize)]
pub struct TerrainCollisionImport {
    pub package: String,
    pub backup: Option<String>,
    pub previous_manifest_sha256: String,
    pub manifest_sha256: String,
    pub render_geometry_sha256: String,
    pub collision_geometry: FileIdentity,
    pub already_imported: bool,
    pub source_bundle_hashes: BTreeMap<String, String>,
    pub source_class_counts: BTreeMap<i32, usize>,
    pub source_entries: usize,
    pub active_solid_entries: usize,
    pub active_trigger_entries: usize,
    pub inactive_or_disabled_entries: usize,
    pub previously_unrepresented_entries: Vec<TerrainCollisionEntry>,
    pub collision_meshes: Vec<CollisionMeshSummary>,
    pub preserved_files: Vec<FileIdentity>,
    pub source_values_verified: bool,
    pub full_runtime_collision_parity_verified: bool,
    pub cache: CacheStats,
    pub verification: TerrainVerification,
}

struct Capture {
    entries: Vec<TerrainCollisionEntry>,
    meshes: Vec<TerrainCollisionMesh>,
    bundle_hashes: BTreeMap<String, String>,
    class_counts: BTreeMap<i32, usize>,
}

fn reflection() -> Mat4 {
    Mat4::from_scale(Vec3::new(1., 1., -1.))
}

fn component_ids(
    resolver: &mut Resolver<'_>,
    owner: &ObjectId,
    object: &unity_asset_binary::object::UnityObject,
) -> Result<Vec<ObjectId>, Error> {
    array(property(object, "m_Component")?)?
        .iter()
        .map(|pointer| {
            let pointer = pointer
                .as_object()
                .and_then(|fields| fields.get("component"))
                .unwrap_or(pointer);
            resolver.required_ref(owner, pointer)
        })
        .collect()
}

fn require_attachment(components: &[ObjectId], id: &ObjectId) -> Result<(), Error> {
    if components
        .iter()
        .filter(|component| *component == id)
        .count()
        != 1
    {
        return Err(invalid(
            "source collider/Transform must occur exactly once in its owner's component list",
        ));
    }
    Ok(())
}

/// Visit every ancestor even after finding inactivity, preserving error and
/// dynamic-body admission rather than assuming a locally active static body.
fn hierarchy_activity(resolver: &mut Resolver<'_>, transform: &ObjectId) -> Result<bool, Error> {
    let mut current = Some(transform.clone());
    let mut seen = BTreeSet::new();
    let mut active = true;
    while let Some(id) = current {
        if seen.len() >= 128 || !seen.insert(id.clone()) {
            return Err(invalid("collision transform hierarchy cycle/depth limit"));
        }
        let object = resolver.object(&id)?;
        if object.info.type_id != 4 {
            return Err(invalid("source collider ancestor is not a Transform"));
        }
        let owner = resolver.required_ref(&id, property(&object, "m_GameObject")?)?;
        let game_object = resolver.object(&owner)?;
        game_object_layer(&game_object)?; // Also checks class-1 identity.
        active &= boolean(property(&game_object, "m_IsActive")?)?;
        let components = component_ids(resolver, &owner, &game_object)?;
        require_attachment(&components, &id)?;
        for component in components {
            if resolver.class_id(&component)? == 54 {
                return Err(invalid(
                    "source terrain collider has a Rigidbody ancestor; dynamic conversion is unresolved",
                ));
            }
        }
        current = resolver.resolve(&id, property(&object, "m_Father")?)?;
    }
    Ok(active)
}

fn capture(resolver: &mut Resolver<'_>, members: &BTreeSet<String>) -> Result<Capture, Error> {
    let mut entries = BTreeMap::new();
    let mut meshes = BTreeMap::<ObjectId, TerrainCollisionMesh>::new();
    let mut bundle_hashes = BTreeMap::new();
    let mut class_counts = BTreeMap::new();
    for member in members {
        bundle_hashes.insert(
            member.clone(),
            format!("{:x}", Sha256::digest(resolver.apk.member_data(member)?)),
        );
        let bundle = resolver.bundle(member)?;
        let mut transforms = BTreeMap::new();
        let mut colliders = Vec::new();
        for (asset_index, asset) in bundle.assets.iter().enumerate() {
            if !asset.objects.iter().any(|info| info.type_id == 104) {
                continue;
            }
            let cab = cab_name(&bundle.asset_names[asset_index]);
            for info in &asset.objects {
                let id = ObjectId {
                    bundle: member.clone(),
                    cab: cab.into(),
                    path_id: info.path_id,
                };
                match info.type_id {
                    4 => {
                        let object = resolver.object(&id)?;
                        let owner =
                            resolver.required_ref(&id, property(&object, "m_GameObject")?)?;
                        if transforms.insert(owner, id).is_some() {
                            return Err(invalid("source collider owner has multiple Transforms"));
                        }
                    }
                    64 | 65 => colliders.push((id, info.type_id)),
                    135 | 136 | 143 => {
                        return Err(invalid(
                            "source terrain collider class requires its own verified shape conversion",
                        ));
                    }
                    _ => {}
                }
            }
        }
        for (id, class_id) in colliders {
            let object = resolver.object(&id)?;
            let owner = resolver.required_ref(&id, property(&object, "m_GameObject")?)?;
            let game_object = resolver.object(&owner)?;
            let layer = game_object_layer(&game_object)?;
            let transform = transforms
                .get(&owner)
                .ok_or_else(|| invalid("source collider has no scene Transform"))?;
            let components = component_ids(resolver, &owner, &game_object)?;
            require_attachment(&components, &id)?;
            require_attachment(&components, transform)?;
            let active_self = boolean(property(&game_object, "m_IsActive")?)?;
            let active_in_hierarchy = hierarchy_activity(resolver, transform)?;
            let world_transform =
                (reflection() * resolver.transform(transform, &mut Vec::new())? * reflection())
                    .to_cols_array();
            let shape = match class_id {
                65 => TerrainCollisionShape::Box {
                    center: vector(property(&object, "m_Center")?, ["x", "y", "z"])?,
                    size: vector(property(&object, "m_Size")?, ["x", "y", "z"])?,
                },
                64 => {
                    let mesh_id = resolver.required_ref(&id, property(&object, "m_Mesh")?)?;
                    if !meshes.contains_key(&mesh_id) {
                        let source_mesh = resolver.mesh_object(&mesh_id)?;
                        if source_mesh.info.type_id != 43 {
                            return Err(invalid("source MeshCollider reference is not a Mesh"));
                        }
                        let (positions, indices) =
                            mesh::decode_collision(&source_mesh, reflection())?;
                        let record =
                            format!("{}:{}:{}", mesh_id.bundle, mesh_id.cab, mesh_id.path_id);
                        let decoded = TerrainCollisionMesh {
                            name: record,
                            positions,
                            indices,
                        };
                        validate_collision_mesh(&decoded)?;
                        meshes.insert(mesh_id.clone(), decoded);
                    }
                    TerrainCollisionShape::Mesh {
                        mesh: mesh_id.source(),
                        record: meshes[&mesh_id].name.clone(),
                        convex: boolean(property(&object, "m_Convex")?)?,
                        cooking_options: integer(property(&object, "m_CookingOptions")?)?,
                    }
                }
                _ => return Err(invalid("unsupported source terrain collider")),
            };
            let source = id.source();
            if entries
                .insert(
                    source.clone(),
                    TerrainCollisionEntry {
                        source,
                        game_object: owner.source(),
                        transform: transform.source(),
                        layer,
                        active_self,
                        active_in_hierarchy,
                        enabled: boolean(property(&object, "m_Enabled")?)?,
                        is_trigger: boolean(property(&object, "m_IsTrigger")?)?,
                        world_transform,
                        shape,
                    },
                )
                .is_some()
            {
                return Err(invalid("duplicate qualified source collider identity"));
            }
            *class_counts.entry(class_id).or_default() += 1;
        }
    }
    Ok(Capture {
        entries: entries.into_values().collect(),
        meshes: meshes.into_values().collect(),
        bundle_hashes,
        class_counts,
    })
}

fn verify_capture(package: &TerrainPackage, capture: &Capture) -> Result<(), Error> {
    let collision = package
        .manifest
        .collision
        .as_ref()
        .ok_or_else(|| invalid("independent source collider table is missing"))?;
    if serde_json::to_vec(&collision.colliders)? != serde_json::to_vec(&capture.entries)?
        || package.collision_meshes != capture.meshes
    {
        return Err(invalid(
            "terrain collision metadata or local geometry differs from the source APK",
        ));
    }
    Ok(())
}

/// Compare complete source collider values and decoded local mesh geometry.
pub fn verify_source_collision(package: &TerrainPackage, apk: &Path) -> Result<bool, Error> {
    if package.manifest.collision.is_none() {
        return Ok(false);
    }
    let index = GlobalIndex::default();
    let mut resolver = Resolver::new(apk, &index)?;
    let members = resolver
        .apk
        .members()?
        .into_iter()
        .filter(|member| is_chunk(member))
        .collect();
    let source = capture(&mut resolver, &members)?;
    verify_capture(package, &source)?;
    Ok(true)
}

pub fn import(apk: &Path, output: &Path) -> Result<TerrainCollisionImport, Error> {
    let canonical = output.join("imported_map/terrain");
    let previous_manifest_sha256 = file_sha256(&canonical.join("manifest.json"))?;
    let previous = load_terrain_package(&canonical)?;
    if file_sha256(&canonical.join("manifest.json"))? != previous_manifest_sha256 {
        return Err(invalid(
            "canonical terrain changed during collision loading",
        ));
    }
    if previous.manifest.schema_version < 3 {
        return Err(invalid(
            "source terrain layers must be enriched before collision-table import",
        ));
    }
    let before = source_apk(apk)?;
    let index: GlobalIndex = serde_json::from_reader(File::open(
        output.join("imported_map/global_asset_index.json"),
    )?)?;
    let mut resolver = Resolver::new(apk, &index)?;
    let members = resolver
        .apk
        .members()?
        .into_iter()
        .filter(|member| is_chunk(member))
        .collect::<BTreeSet<_>>();
    terrain_verification::verify_loaded(&previous, &before, &members)?;
    if !crate::terrain_layers::verify_source_layers(&previous, apk)? {
        return Err(invalid("terrain GameObject layers are not source-verified"));
    }
    let captured = capture(&mut resolver, &members)?;
    let represented = previous
        .manifest
        .meshes
        .values()
        .filter_map(|mesh| mesh.collider.as_ref())
        .map(|c| c.source.clone())
        .collect::<BTreeSet<_>>();
    let previously_unrepresented_entries = captured
        .entries
        .iter()
        .filter(|entry| !represented.contains(&entry.source))
        .cloned()
        .collect();
    let active_solid_entries = captured
        .entries
        .iter()
        .filter(|e| e.active_in_hierarchy && e.enabled && !e.is_trigger)
        .count();
    let active_trigger_entries = captured
        .entries
        .iter()
        .filter(|e| e.active_in_hierarchy && e.enabled && e.is_trigger)
        .count();
    let inactive_or_disabled_entries =
        captured.entries.len() - active_solid_entries - active_trigger_entries;
    let preserved_files = retained_files(&previous.manifest)?;
    let already_imported = previous.manifest.collision.is_some();
    let mut backup = None;
    let mut verification;
    let collision_geometry;
    if already_imported {
        verify_capture(&previous, &captured)?;
        collision_geometry = previous
            .manifest
            .collision
            .as_ref()
            .ok_or_else(|| invalid("source collider table missing"))?
            .geometry
            .clone();
        verification = terrain_verification::verify_loaded(&previous, &before, &members)?;
    } else {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let staged = output
            .join("imported_map")
            .join(format!(".terrain-collision-{stamp}"));
        std::fs::create_dir(&staged)?;
        let real = std::fs::canonicalize(&canonical)?;
        for identity in &preserved_files {
            let source = package_path(&canonical, &identity.path)?;
            if !std::fs::canonicalize(&source)?.starts_with(&real) {
                return Err(invalid("retained terrain dependency escapes its package"));
            }
            let target = package_path(&staged, &identity.path)?;
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&source, &target)?;
            if file_sha256(&target)? != identity.sha256 {
                return Err(invalid(
                    "retained terrain payload changed during collision import",
                ));
            }
        }
        let geometry_path = "collision-meshes.bin";
        let mut writer = BufWriter::new(File::create(staged.join(geometry_path))?);
        write_collision_archive(&mut writer, &captured.meshes)?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
        collision_geometry = FileIdentity {
            path: geometry_path.into(),
            sha256: file_sha256(&staged.join(geometry_path))?,
        };
        let mut manifest = previous.manifest.clone();
        manifest.schema_version = TERRAIN_MANIFEST_VERSION;
        manifest.collision = Some(TerrainCollisionManifest {
            geometry: collision_geometry.clone(),
            colliders: captured.entries.clone(),
        });
        manifest.limitations.push("Independent Mesh/Box source collider table includes saved ancestor activity and local collision geometry; trigger behavior, runtime activation, PhysX cooking/tolerance and other collider classes remain separate acceptance work.".into());
        manifest.validate_collision_metadata()?;
        let file = File::create(staged.join("manifest.json"))?;
        serde_json::to_writer_pretty(&file, &manifest)?;
        file.sync_all()?;
        let candidate = load_terrain_package(&staged)?;
        if candidate.records != previous.records {
            return Err(invalid(
                "independent collision import changed visible terrain geometry",
            ));
        }
        verify_capture(&candidate, &captured)?;
        verification = terrain_verification::verify_loaded(&candidate, &before, &members)?;
        let after = source_apk(apk)?;
        if after.sha256 != before.sha256
            || after.byte_count != before.byte_count
            || file_sha256(&canonical.join("manifest.json"))? != previous_manifest_sha256
        {
            return Err(invalid(
                "source APK or canonical package changed during collision import",
            ));
        }
        let retained = output
            .parent()
            .unwrap_or(Path::new("."))
            .join(".local/map-import-backups")
            .join(stamp.to_string())
            .join("terrain");
        publish_verified(&staged, &canonical, &retained)?;
        backup = Some(retained.display().to_string());
    }
    if already_imported {
        let after = source_apk(apk)?;
        if after.sha256 != before.sha256
            || after.byte_count != before.byte_count
            || file_sha256(&canonical.join("manifest.json"))? != previous_manifest_sha256
        {
            return Err(invalid(
                "source APK or canonical package changed during collision verification",
            ));
        }
    }
    verification.source_layer_values_verified = true;
    verification.source_collision_values_verified = true;
    let collision_meshes = captured
        .meshes
        .iter()
        .map(|mesh| CollisionMeshSummary {
            record: mesh.name.clone(),
            vertices: mesh.positions.len(),
            triangles: mesh.indices.len() / 3,
        })
        .collect();
    Ok(TerrainCollisionImport {
        package: canonical.display().to_string(),
        backup,
        previous_manifest_sha256,
        manifest_sha256: file_sha256(&canonical.join("manifest.json"))?,
        render_geometry_sha256: previous.manifest.geometry.sha256.clone(),
        collision_geometry,
        already_imported,
        source_bundle_hashes: captured.bundle_hashes,
        source_class_counts: captured.class_counts,
        source_entries: captured.entries.len(),
        active_solid_entries,
        active_trigger_entries,
        inactive_or_disabled_entries,
        previously_unrepresented_entries,
        collision_meshes,
        preserved_files,
        source_values_verified: true,
        full_runtime_collision_parity_verified: false,
        cache: resolver.cache_stats(),
        verification,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Quat;
    use midnight_map_data::{SourceObject, TerrainManifest};

    fn source(id: i64) -> SourceObject {
        SourceObject {
            bundle: "scene.ab".into(),
            cab: "scene".into(),
            path_id: id.to_string(),
        }
    }

    #[test]
    fn attachment_admission_rejects_orphans_duplicate_entries_and_unqualified_matches() {
        let id = ObjectId {
            bundle: "scene.ab".into(),
            cab: "scene".into(),
            path_id: 10,
        };
        require_attachment(std::slice::from_ref(&id), &id).unwrap();
        assert!(require_attachment(&[], &id).is_err());
        assert!(require_attachment(&[id.clone(), id.clone()], &id).is_err());
        let other = ObjectId {
            cab: "other-scene".into(),
            ..id.clone()
        };
        assert!(require_attachment(&[other], &id).is_err());
    }

    fn fixture() -> (TerrainPackage, Capture) {
        let entries = [10, 11]
            .into_iter()
            .map(|id| TerrainCollisionEntry {
                source: source(id),
                game_object: source(2),
                transform: source(4),
                layer: 0,
                active_self: true,
                active_in_hierarchy: true,
                enabled: true,
                is_trigger: false,
                world_transform: Mat4::IDENTITY.to_cols_array(),
                shape: TerrainCollisionShape::Mesh {
                    mesh: source(5),
                    record: "road".into(),
                    convex: false,
                    cooking_options: 30,
                },
            })
            .collect::<Vec<_>>();
        let meshes = vec![TerrainCollisionMesh {
            name: "road".into(),
            positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 0., -1.]],
            indices: vec![0, 1, 2],
        }];
        let manifest:TerrainManifest = serde_json::from_value(serde_json::json!({
            "schema_version":4,"source_apk":{"path":"fixture.apk","sha256":"a".repeat(64),"byte_count":1},
            "coordinate_frame":"bevy_right_handed_y_up","uv_origin":"top_left",
            "geometry":{"path":"meshes.bin","sha256":"b".repeat(64)},"meshes":{},"limitations":[],
            "collision":{"geometry":{"path":"collision.bin","sha256":"c".repeat(64)},"colliders":entries}
        })).unwrap();
        (
            TerrainPackage {
                root: "fixture".into(),
                manifest,
                records: vec![],
                collision_meshes: meshes.clone(),
            },
            Capture {
                entries,
                meshes,
                bundle_hashes: BTreeMap::new(),
                class_counts: BTreeMap::from([(64, 2)]),
            },
        )
    }

    #[test]
    fn separate_source_components_share_geometry_without_losing_identity() {
        let (mut package, captured) = fixture();
        verify_capture(&package, &captured).unwrap();
        assert_eq!(package.collision_meshes.len(), 1);
        package
            .manifest
            .collision
            .as_mut()
            .unwrap()
            .colliders
            .remove(0);
        assert!(verify_capture(&package, &captured).is_err());
    }

    #[test]
    fn source_verification_rejects_changed_flags_transforms_or_geometry() {
        let (mut package, captured) = fixture();
        package.manifest.collision.as_mut().unwrap().colliders[0].active_in_hierarchy = false;
        assert!(verify_capture(&package, &captured).is_err());
        package.manifest.collision.as_mut().unwrap().colliders = captured.entries.clone();
        package.manifest.collision.as_mut().unwrap().colliders[0].world_transform[12] = 1.;
        assert!(verify_capture(&package, &captured).is_err());
        package.manifest.collision.as_mut().unwrap().colliders = captured.entries.clone();
        package.collision_meshes[0].positions[0][1] = 0.5;
        assert!(verify_capture(&package, &captured).is_err());
    }

    #[test]
    fn local_reflection_and_converted_hierarchy_match_one_world_reflection() {
        let unity = Mat4::from_scale_rotation_translation(
            Vec3::new(3., 0.5, 2.),
            Quat::from_rotation_y(0.7),
            Vec3::new(14., 5., -8.),
        );
        let local = Vec3::new(0.25, 1., 2.);
        let expected = reflection().transform_point3(unity.transform_point3(local));
        let converted_world = reflection() * unity * reflection();
        let actual = converted_world.transform_point3(reflection().transform_point3(local));
        assert!(actual.abs_diff_eq(expected, 1e-5));
    }
}
