//! Source scene matrices, qualified PPtrs, raw textures, and one validated terrain package.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{File, create_dir_all},
    io::{BufWriter, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use glam::{Mat4, Vec3};
use midnight_map_data::{
    FileIdentity, MeshRecord, SceneMesh, SourceCollider, TERRAIN_MAGIC,
    TerrainManifest, file_sha256, load_terrain_package, package_path, write_mesh_archive,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    fields::{array, boolean, integer, invalid, property, text},
    mesh,
    pass1_index::GlobalIndex,
    resolver::{ObjectId, Resolver},
    terrain_verification::{self, TerrainVerification},
    unity::{Error, cab_name, source_apk},
};

pub(crate) fn is_chunk(member: &str) -> bool {
    let Some(name) = member
        .strip_prefix("assets/scene/")
        .and_then(|name| name.strip_suffix(".ab"))
    else {
        return false;
    };
    if name == "z-01" {
        return true;
    }
    if let Some(number) = name.strip_prefix("junshijidi_terrain_") {
        return number
            .parse::<u8>()
            .is_ok_and(|number| (1..=16).contains(&number));
    }
    let Some((quadrant, number)) = name.split_once('-') else {
        return false;
    };
    matches!(quadrant, "a" | "b" | "c" | "d")
        && number
            .parse::<u8>()
            .is_ok_and(|number| (1..=57).contains(&number))
}

fn hash(path: &Path) -> Result<String, Error> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}

fn collect_scene(
    resolver: &mut Resolver<'_>,
    member: &str,
    output: &Path,
    records: &mut BTreeMap<String, MeshRecord>,
    scene_meshes: &mut BTreeMap<String, SceneMesh>,
) -> Result<(), Error> {
    let bundle = resolver.bundle(member)?;
    let mut transforms = BTreeMap::new();
    let mut filters = BTreeMap::new();
    let mut renderers = Vec::new();
    let mut colliders = BTreeMap::new();
    for (asset_index, asset) in bundle.assets.iter().enumerate() {
        // Shared-assets files can contain prefab definitions with their own
        // renderers. Only the scene's serialized file owns placed instances.
        // Every scene file in this APK has its RenderSettings object (class 104).
        if !asset.objects.iter().any(|object| object.type_id == 104) {
            continue;
        }
        let cab = cab_name(&bundle.asset_names[asset_index]).to_owned();
        for info in &asset.objects {
            if !matches!(info.type_id, 4 | 23 | 33 | 64) {
                continue;
            }
            let id = ObjectId {
                bundle: member.to_owned(),
                cab: cab.clone(),
                path_id: info.path_id,
            };
            let object = resolver.object(&id)?;
            let game_object = resolver.required_ref(&id, property(&object, "m_GameObject")?)?;
            match info.type_id {
                4 => {
                    transforms.insert(game_object, id);
                }
                33 => {
                    filters.insert(game_object, (id, object));
                }
                23 => renderers.push((game_object, id, object)),
                64 => {
                    let mesh = resolver.required_ref(&id, property(&object, "m_Mesh")?)?;
                    colliders.insert(
                        game_object,
                        SourceCollider {
                            source: id.source(),
                            mesh: mesh.source(),
                            enabled: boolean(property(&object, "m_Enabled")?)?,
                            is_trigger: boolean(property(&object, "m_IsTrigger")?)?,
                            convex: boolean(property(&object, "m_Convex")?)?,
                            cooking_options: integer(property(&object, "m_CookingOptions")?)?,
                        },
                    );
                }
                _ => unreachable!(),
            }
        }
    }
    for (game_object, renderer_id, renderer) in renderers {
        if property(&renderer, "m_Enabled")?.as_bool() == Some(false) {
            continue;
        }
        let source_object = resolver.object(&game_object)?;
        let layer = crate::terrain_layers::game_object_layer(&source_object)?;
        if property(&source_object, "m_IsActive")?.as_bool() == Some(false) {
            continue;
        }
        let Some((filter_id, filter)) = filters.get(&game_object) else {
            continue;
        };
        let transform_id = transforms
            .get(&game_object)
            .ok_or_else(|| invalid("scene MeshFilter has no Transform"))?;
        let mesh_id = resolver.required_ref(filter_id, property(filter, "m_Mesh")?)?;
        let source_mesh = resolver.object(&mesh_id)?;
        let mesh_name = text(property(&source_mesh, "m_Name")?)?;
        let world = Mat4::from_scale(Vec3::new(1.0, 1.0, -1.0))
            * resolver.transform(transform_id, &mut Vec::new())?;
        let sub_meshes = mesh::decode(&source_mesh, world)?;
        let materials = array(property(&renderer, "m_Materials")?)?;
        if materials.len() < sub_meshes.len() {
            return Err(invalid(format!("{mesh_name}: missing submesh material")));
        }
        for (part, mut record) in sub_meshes.into_iter().enumerate() {
            let material_id = resolver.required_ref(&renderer_id, &materials[part])?;
            let material = resolver.material(&material_id, output)?;
            let suffix = if part == 0 {
                String::new()
            } else {
                format!("@submesh-{part}")
            };
            let mut name = format!("{mesh_name}{suffix}");
            if records.contains_key(&name) {
                name = format!("{name}@{}-{}", game_object.cab, game_object.path_id);
            }
            record.name = name.clone();
            record.texture = material
                .textures
                .get("_MainTex")
                .or_else(|| material.textures.get("_BaseMap"))
                .and_then(|binding| binding.texture.as_ref())
                .map(|texture| format!("imported_map/terrain/{}", texture.file.path))
                .unwrap_or_default();
            scene_meshes.insert(
                name.clone(),
                SceneMesh {
                    source: mesh_id.source(),
                    game_object: game_object.source(),
                    layer: Some(layer),
                    world_transform: world.to_cols_array(),
                    bounds: mesh::bounds(&record),
                    material,
                    collider: colliders.get(&game_object).cloned(),
                },
            );
            records.insert(name, record);
        }
    }
    Ok(())
}

fn promote_verified(candidate: &Path, output: &Path, stamp: u128) -> Result<(), Error> {
    let imported = output.join("imported_map");
    let canonical = imported.join("terrain");
    let backup = output
        .parent()
        .unwrap_or(Path::new("."))
        .join(".local/map-import-backups")
        .join(stamp.to_string())
        .join("terrain");
    crate::package_publish::publish_verified(candidate, &canonical, &backup)
}

/// Resume a completed staged import without decoding all source bundles again.
/// Verify every payload and source APK identity before changing the active package.
pub fn promote(apk: &Path, candidate: &Path, output: &Path) -> Result<TerrainVerification, Error> {
    let verification = terrain_verification::verify(candidate, apk)?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    promote_verified(candidate, output, stamp)?;
    Ok(verification)
}

fn import(apk: &Path, output: &Path) -> Result<(), Error> {
    let imported = output.join("imported_map");
    let index_path = imported.join("global_asset_index.json");
    if !index_path.exists() {
        crate::pass1_index::run(apk, output).map_err(|error| invalid(error.to_string()))?;
    }
    let index: GlobalIndex = serde_json::from_reader(File::open(index_path)?)?;
    let source = source_apk(apk)?;
    let mut resolver = Resolver::new(apk, &index)?;
    let mut members = resolver
        .apk
        .members()?
        .into_iter()
        .filter(|member| is_chunk(member))
        .collect::<Vec<_>>();
    members.sort();
    if members.len() != 242 {
        return Err(invalid(format!(
            "expected 242 terrain bundles in source APK, found {}",
            members.len()
        )));
    }
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let staged = imported.join(format!(".terrain-import-{stamp}"));
    create_dir_all(&staged)?;
    let mut records = BTreeMap::new();
    let mut scene_meshes = BTreeMap::new();
    for (index, member) in members.iter().enumerate() {
        collect_scene(
            &mut resolver,
            member,
            &staged,
            &mut records,
            &mut scene_meshes,
        )
        .map_err(|error| {
            invalid(format!(
                "{member}: {error}; candidate preserved at {}",
                staged.display()
            ))
        })?;
        if (index + 1).is_multiple_of(24) || index + 1 == members.len() {
            eprintln!(
                "Imported {}/{} terrain scenes ({} render mesh parts)",
                index + 1,
                members.len(),
                records.len()
            );
        }
    }
    let records = records.into_values().collect::<Vec<_>>();
    let mut writer = BufWriter::new(File::create(staged.join("meshes.bin"))?);
    write_mesh_archive(&mut writer, TERRAIN_MAGIC, &records)?;
    writer.flush()?;
    writer.get_ref().sync_all()?;
    let manifest = TerrainManifest { schema_version: 3, source_apk: source, collision:None,
        coordinate_frame: "bevy_right_handed_y_up".into(), uv_origin: "top_left".into(),
        geometry: FileIdentity { path: "meshes.bin".into(), sha256: hash(&staged.join("meshes.bin"))? }, meshes: scene_meshes,
        limitations: vec![
            "Package contains shipped terrain scene instances and source material/texture evidence; full gameplay is not reconstructed.".into(),
            "Scene-info placement files are absent from this APK. Legacy model/placement archives have separate, unverified provenance.".into(),
            "Some source meshes have _NORMALMAP materials but no tangents. Source absence is retained; Bevy uses geometric normals for those records. Original runtime shader LOD and generated attributes remain unknown.".into(),
            "Exact original lighting and rendering still require visual comparison.".into(),
        ] };
    serde_json::to_writer_pretty(File::create(staged.join("manifest.json"))?, &manifest)?;
    let package = load_terrain_package(&staged)?;
    let verification = terrain_verification::verify_loaded(
        &package,
        &package.manifest.source_apk,
        &members.into_iter().collect(),
    )?;
    promote_verified(&staged, output, stamp)?;
    eprintln!(
        "Verified canonical terrain package: {} ({} source instances, {} render parts)",
        imported.join("terrain").display(),
        verification.source_instances,
        records.len()
    );
    Ok(())
}

pub fn run(apk: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    import(apk, output).map_err(|error| error as Box<dyn std::error::Error>)
}

#[derive(Debug, Serialize)]
pub struct TerrainAugmentation {
    pub package: String,
    pub backup: Option<String>,
    pub scene: String,
    pub source_bundle_sha256: String,
    pub map_scene_list_sha256: String,
    pub previous_manifest_sha256: String,
    pub previous_geometry_sha256: String,
    pub added_records: Vec<String>,
    pub added_scene_meshes: BTreeMap<String, SceneMesh>,
    pub already_present: bool,
    pub verification: TerrainVerification,
}

/// Decode one authenticated omitted scene; preserve every existing source record.
pub fn augment(apk_path: &Path, output: &Path, member: &str) -> Result<TerrainAugmentation, Error> {
    if member != "assets/scene/z-01.ab" {
        return Err(invalid(
            "terrain augmentation currently supports the audited assets/scene/z-01.ab only",
        ));
    }
    let imported = output.join("imported_map");
    let canonical = imported.join("terrain");
    let source = source_apk(apk_path)?;
    let previous = load_terrain_package(&canonical)?;
    if previous.manifest.source_apk.sha256 != source.sha256
        || previous.manifest.source_apk.byte_count != source.byte_count
    {
        return Err(invalid(
            "canonical terrain and selected scene come from different APKs",
        ));
    }
    let previous_manifest_sha256 = file_sha256(&canonical.join("manifest.json"))?;
    let previous_geometry_sha256 = previous.manifest.geometry.sha256.clone();
    let index: GlobalIndex =
        serde_json::from_reader(File::open(imported.join("global_asset_index.json"))?)?;
    let mut resolver = Resolver::new(apk_path, &index)?;
    let all_members = resolver
        .apk
        .members()?
        .into_iter()
        .filter(|name| is_chunk(name))
        .collect::<BTreeSet<_>>();
    if all_members.len() != 242 || !all_members.contains(member) {
        return Err(invalid(
            "source terrain membership differs from the audited 242-scene inventory",
        ));
    }
    let scene_list = resolver
        .apk
        .member_data("assets/gameconfig/allscenelist.obj")?;
    let list: serde_json::Value = serde_json::from_slice(&scene_list)?;
    if !list
        .get("sceneList")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| row.get("scenename").and_then(serde_json::Value::as_str) == Some("Z-01"))
        })
    {
        return Err(invalid(
            "Z-01 is absent from the source map-1 scene inventory",
        ));
    }
    let map_scene_list_sha256 = format!("{:x}", Sha256::digest(&scene_list));
    let source_bundle_sha256 = format!("{:x}", Sha256::digest(resolver.apk.member_data(member)?));
    let covered = previous
        .manifest
        .meshes
        .values()
        .filter(|mesh| mesh.material.terrain_layer_count().is_some())
        .map(|mesh| mesh.game_object.bundle.clone())
        .collect::<BTreeSet<_>>();
    if covered == all_members {
        let verification = terrain_verification::verify_loaded(&previous, &source, &all_members)?;
        return Ok(TerrainAugmentation {
            package: canonical.display().to_string(),
            backup: None,
            scene: member.into(),
            source_bundle_sha256,
            map_scene_list_sha256,
            previous_manifest_sha256,
            previous_geometry_sha256,
            added_records: Vec::new(),
            added_scene_meshes: BTreeMap::new(),
            already_present: true,
            verification,
        });
    }
    if previous.manifest.collision.is_some() {
        return Err(invalid("scene augmentation requires a separately rebuilt independent collision table"));
    }
    let expected_previous = all_members
        .iter()
        .filter(|name| name.as_str() != member)
        .cloned()
        .collect::<BTreeSet<_>>();
    if covered != expected_previous {
        return Err(invalid(
            "canonical coverage must differ only by the explicitly selected Z-01 scene",
        ));
    }
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let staged = imported.join(format!(".terrain-augment-{stamp}"));
    std::fs::create_dir(&staged)?;
    let mut files = BTreeSet::new();
    for source_mesh in previous.manifest.meshes.values() {
        for texture in source_mesh
            .material
            .textures
            .values()
            .filter_map(|binding| binding.texture.as_ref())
        {
            files.insert(texture.file.path.clone());
            files.insert(texture.encoded.path.clone());
            if let Some(gpu) = &texture.gpu {
                files.insert(gpu.path.clone());
            }
        }
    }
    let canonical_real = std::fs::canonicalize(&canonical)?;
    for relative in files {
        let source_file = package_path(&canonical, &relative)?;
        if !std::fs::canonicalize(&source_file)?.starts_with(&canonical_real) {
            return Err(invalid("canonical terrain dependency escapes its package"));
        }
        let target = package_path(&staged, &relative)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(source_file, target)?;
    }
    let mut records = previous
        .records
        .iter()
        .cloned()
        .map(|record| (record.name.clone(), record))
        .collect::<BTreeMap<_, _>>();
    let mut manifest = previous.manifest.clone();
    collect_scene(
        &mut resolver,
        member,
        &staged,
        &mut records,
        &mut manifest.meshes,
    )?;
    let added_records = records
        .keys()
        .filter(|name| !previous.manifest.meshes.contains_key(*name))
        .cloned()
        .collect::<Vec<_>>();
    // Legacy augmentation preserves unknown layers; enrichment is a separate action.
    if manifest.schema_version == 2 {
        for name in &added_records {
            if let Some(mesh) = manifest.meshes.get_mut(name) {
                mesh.layer = None;
            }
        }
    }
    if added_records.len() != 1 {
        return Err(invalid(
            "audited Z-01 scene must add exactly one terrain mesh record",
        ));
    }
    let added = manifest
        .meshes
        .get(&added_records[0])
        .ok_or_else(|| invalid("added scene record is absent"))?;
    if added.game_object.bundle != member
        || added.game_object.cab != "BuildPlayer-Z-01"
        || added.game_object.path_id != "13"
        || added.source.cab != "BuildPlayer-Z-01.sharedAssets"
        || added.source.path_id != "7"
        || added.material.shader_name != "_URP/TerrainMeshBase/4 Textures"
        || !added.collider.as_ref().is_some_and(|collider| {
            collider.source.path_id == "33"
                && collider.mesh == added.source
                && collider.enabled
                && !collider.is_trigger
                && !collider.convex
        })
    {
        return Err(invalid(
            "Z-01 renderer/material/source MeshCollider differs from the audited source identities",
        ));
    }
    let added_scene_meshes = BTreeMap::from([(added_records[0].clone(), added.clone())]);
    // Old source records and metadata are immutable during augmentation.
    for record in &previous.records {
        if records.get(&record.name) != Some(record) {
            return Err(invalid("augmentation changed existing terrain geometry"));
        }
    }
    for (name, mesh) in &previous.manifest.meshes {
        if serde_json::to_vec(mesh)? != serde_json::to_vec(&manifest.meshes[name])? {
            return Err(invalid(
                "augmentation changed existing terrain source metadata",
            ));
        }
    }
    let ordered = records.into_values().collect::<Vec<_>>();
    let mut writer = BufWriter::new(File::create(staged.join("meshes.bin"))?);
    write_mesh_archive(&mut writer, TERRAIN_MAGIC, &ordered)?;
    writer.flush()?;
    writer.get_ref().sync_all()?;
    manifest.geometry = FileIdentity {
        path: "meshes.bin".into(),
        sha256: file_sha256(&staged.join("meshes.bin"))?,
    };
    let manifest_file = File::create(staged.join("manifest.json"))?;
    serde_json::to_writer_pretty(&manifest_file, &manifest)?;
    manifest_file.sync_all()?;
    let loaded = load_terrain_package(&staged)?;
    if loaded.records != ordered {
        return Err(invalid("augmented terrain geometry roundtrip differs"));
    }
    let verification = terrain_verification::verify_loaded(&loaded, &source, &all_members)?;
    let after = source_apk(apk_path)?;
    if after.sha256 != source.sha256
        || after.byte_count != source.byte_count
        || file_sha256(&canonical.join("manifest.json"))? != previous_manifest_sha256
    {
        return Err(invalid(
            "source APK or canonical terrain changed during augmentation",
        ));
    }
    let backup = output
        .parent()
        .unwrap_or(Path::new("."))
        .join(".local/map-import-backups")
        .join(stamp.to_string())
        .join("terrain");
    crate::package_publish::publish_verified(&staged, &canonical, &backup)?;
    Ok(TerrainAugmentation {
        package: canonical.display().to_string(),
        backup: Some(backup.display().to_string()),
        scene: member.into(),
        source_bundle_sha256,
        map_scene_list_sha256,
        previous_manifest_sha256,
        previous_geometry_sha256,
        added_records,
        added_scene_meshes,
        already_present: false,
        verification,
    })
}

#[cfg(test)]
mod tests {
    use super::is_chunk;

    #[test]
    fn terrain_inventory_includes_exact_z01_and_excludes_menu_scenes() {
        assert!(is_chunk("assets/scene/z-01.ab"));
        assert!(is_chunk("assets/scene/a-01.ab"));
        assert!(is_chunk("assets/scene/junshijidi_terrain_16.ab"));
        assert!(!is_chunk("assets/scene/z-02.ab"));
        assert!(!is_chunk("assets/scene/zhujiemian_01.ab"));
        assert!(!is_chunk("assets/sceneprefab/z-01.ab"));
    }
}
