//! Source scene matrices, qualified PPtrs, raw textures, and one validated terrain package.

use std::{
    collections::BTreeMap,
    fs::{File, create_dir_all},
    io::{BufWriter, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use glam::{Mat4, Vec3};
use midnight_map_data::{
    FileIdentity, MeshRecord, SceneMesh, SourceCollider, TERRAIN_MAGIC, TERRAIN_MANIFEST_VERSION,
    TerrainManifest, load_terrain_package, write_mesh_archive,
};
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
    create_dir_all(&imported)?;
    let canonical = imported.join("terrain");
    if canonical.exists() && std::fs::canonicalize(candidate)? == std::fs::canonicalize(&canonical)?
    {
        return Err(invalid(
            "the candidate is already the canonical terrain package",
        ));
    }
    let backup = if canonical.exists() {
        let backup = output
            .parent()
            .unwrap_or(Path::new("."))
            .join(".local/map-import-backups")
            .join(stamp.to_string());
        create_dir_all(&backup)?;
        let backup = backup.join("terrain");
        std::fs::rename(&canonical, &backup)?;
        Some(backup)
    } else {
        None
    };
    if let Err(error) = std::fs::rename(candidate, &canonical) {
        if let Some(backup) = &backup {
            std::fs::rename(backup, &canonical).map_err(|restore| invalid(format!(
                "promotion failed: {error}; restoring the prior package failed: {restore}; prior package remains at {}", backup.display())))?;
        }
        return Err(error.into());
    }
    if let Some(backup) = backup {
        eprintln!("Previous terrain package archived at {}", backup.display());
    }
    Ok(())
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
    if members.len() != 241 {
        return Err(invalid(format!(
            "expected 241 terrain bundles in source APK, found {}",
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
    let manifest = TerrainManifest { schema_version: TERRAIN_MANIFEST_VERSION, source_apk: source,
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
