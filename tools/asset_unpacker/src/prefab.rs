//! Complete source prefab hierarchy and component evidence; no guessed world placement.

use crate::{
    bundle_cache::CacheStats,
    fields::{array, boolean, field, integer, invalid, property, text, vector},
    mesh,
    pass1_index::GlobalIndex,
    resolver::{ObjectId, Resolver},
    unity::{Error, cab_name, source_apk},
};
use glam::{Mat4, Quat, Vec3};
use midnight_map_data::{
    FileIdentity, MeshRecord, PrefabCollisionMesh, PrefabManifest, PrefabMaterial, PrefabNode,
    PrefabRender, SourceComponent, SourceObject, TERRAIN_MAGIC, file_sha256, load_prefab_package,
    write_mesh_archive,
};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use unity_asset_core::UnityValue;

#[derive(Debug, Serialize)]
pub struct PrefabReport {
    pub package: String,
    pub bundle: String,
    pub nodes: usize,
    pub renderers: usize,
    pub mesh_parts: usize,
    pub collision_meshes: usize,
    pub unsupported_samplers: usize,
    pub verified_files: usize,
    pub bundle_cache: CacheStats,
    pub limitations: Vec<String>,
}

fn reflection() -> Mat4 {
    Mat4::from_scale(Vec3::new(1.0, 1.0, -1.0))
}

pub(crate) fn convert_local(
    position: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
) -> Result<[f32; 16], Error> {
    if position
        .iter()
        .chain(&rotation)
        .chain(&scale)
        .any(|value| !value.is_finite())
    {
        return Err(invalid("non-finite prefab transform"));
    }
    let rotation = Quat::from_array(rotation);
    if rotation.length_squared() < 1e-8 {
        return Err(invalid("zero prefab quaternion"));
    }
    Ok((reflection()
        * Mat4::from_scale_rotation_translation(
            Vec3::from_array(scale),
            rotation.normalize(),
            Vec3::from_array(position),
        )
        * reflection())
    .to_cols_array())
}

pub(crate) fn store(output: &Path, category: &str, bytes: &[u8]) -> Result<FileIdentity, Error> {
    let hash = format!("{:x}", Sha256::digest(bytes));
    let relative = format!("{category}/{hash}.bin");
    let path = output.join(&relative);
    std::fs::create_dir_all(path.parent().ok_or_else(|| invalid("blob has no parent"))?)?;
    if path.exists() {
        if file_sha256(&path)? != hash {
            return Err(invalid("existing source blob checksum differs"));
        }
    } else {
        std::fs::write(path, bytes)?;
    }
    Ok(FileIdentity {
        path: relative,
        sha256: hash,
    })
}

/// Preserve all fields, large IDs and blobs; unresolved references remain explicit.
fn source_fields(
    resolver: &mut Resolver<'_>,
    owner: &ObjectId,
    value: &UnityValue,
    output: &Path,
    depth: usize,
    verify_only: bool,
) -> Result<Value, Error> {
    if depth > 128 {
        return Err(invalid("prefab field depth exceeds limit"));
    }
    Ok(match value {
        UnityValue::Integer(value) if value.unsigned_abs() > (1u64 << 53) => {
            json!({"unity_i64": value.to_string()})
        }
        UnityValue::Float(value) if !value.is_finite() => {
            json!({"unity_non_finite_f64": {"parsed_bits_hex": format!("{:016x}", value.to_bits())}})
        }
        UnityValue::Bytes(bytes) => {
            json!({"unity_bytes": source_payload(output, "source/blobs", bytes, verify_only)?, "byte_count": bytes.len()})
        }
        UnityValue::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| source_fields(resolver, owner, item, output, depth + 1, verify_only))
                .collect::<Result<_, _>>()?,
        ),
        UnityValue::Object(fields)
            if fields.contains_key("m_FileID") && fields.contains_key("m_PathID") =>
        {
            let file_id = integer(field(value, "m_FileID")?)?;
            let path_id = integer(field(value, "m_PathID")?)?;
            match resolver.resolve(owner, value) {
                Ok(Some(target)) => {
                    json!({"owner": owner.source(), "file_id": file_id, "path_id": path_id.to_string(), "state": "resolved", "target": target.source()})
                }
                Ok(None) => {
                    json!({"owner": owner.source(), "file_id": file_id, "path_id": path_id.to_string(), "state": "null"})
                }
                Err(error) => {
                    json!({"owner": owner.source(), "file_id": file_id, "path_id": path_id.to_string(), "state": "unresolved", "detail": error.to_string()})
                }
            }
        }
        UnityValue::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(name, value)| {
                    Ok((
                        name.clone(),
                        source_fields(resolver, owner, value, output, depth + 1, verify_only)?,
                    ))
                })
                .collect::<Result<_, Error>>()?,
        ),
        _ => serde_json::to_value(value)?,
    })
}

pub(crate) fn raw_component(
    resolver: &mut Resolver<'_>,
    id: &ObjectId,
    output: &Path,
) -> Result<SourceComponent, Error> {
    raw_component_mode(resolver, id, output, false)
}

fn source_payload(output: &Path, category: &str, bytes: &[u8], verify_only: bool) -> Result<FileIdentity, Error> {
    if !verify_only { return store(output, category, bytes); }
    let sha256 = format!("{:x}", Sha256::digest(bytes));
    let path = format!("{category}/{sha256}.bin");
    if file_sha256(&output.join(&path))? != sha256 {
        return Err(invalid("retained source payload differs"));
    }
    Ok(FileIdentity {path,sha256})
}

fn raw_component_mode(
    resolver: &mut Resolver<'_>, id: &ObjectId, output: &Path, verify_only: bool,
) -> Result<SourceComponent, Error> {
    let object = resolver.object(id)?;
    let bundle = resolver.bundle(&id.bundle)?;
    let asset = bundle
        .asset_names
        .iter()
        .position(|name| cab_name(name) == id.cab)
        .map(|index| &bundle.assets[index])
        .ok_or_else(|| invalid("component CAB absent"))?;
    let info = asset
        .objects
        .iter()
        .find(|info| info.path_id == id.path_id)
        .ok_or_else(|| invalid("component object absent"))?;
    let serialized = source_payload(output, "source/objects", asset.object_bytes(info)?, verify_only)?;
    let fields = source_fields(
        resolver,
        id,
        &UnityValue::Object(object.class.properties().clone()),
        output,
        0,
        verify_only,
    )?;
    Ok(SourceComponent {
        source: id.source(),
        class_id: object.info.type_id,
        serialized,
        fields,
    })
}

pub(crate) fn verify_raw_component(
    resolver: &mut Resolver<'_>, component: &SourceComponent, output: &Path,
) -> Result<(), Error> {
    let id = ObjectId {bundle:component.source.bundle.clone(),cab:component.source.cab.clone(),path_id:component.source.path_id.parse()?};
    let actual = raw_component_mode(resolver,&id,output,true)?;
    if actual.source != component.source || actual.class_id != component.class_id
        || actual.fields != component.fields || actual.serialized.path != component.serialized.path
        || actual.serialized.sha256 != component.serialized.sha256
    { return Err(invalid(format!("retained player fields differ from source {}:{}",id.cab,id.path_id))); }
    Ok(())
}

/// Read one complete node; callers choose the root set and traversal boundary.
pub(crate) fn source_node(
    resolver: &mut Resolver<'_>,
    id: &ObjectId,
    output: &Path,
) -> Result<PrefabNode, Error> {
    let object = resolver.object(id)?;
    if object.info.type_id != 1 {
        return Err(invalid("hierarchy node is not a GameObject"));
    }
    let mut components = vec![raw_component(resolver, id, output)?];
    let mut transform_id = None;
    for pointer in array(property(&object, "m_Component")?)? {
        let pointer = pointer.as_object().and_then(|f| f.get("component")).unwrap_or(pointer);
        let component_id = resolver.required_ref(id, pointer)?;
        let component = raw_component(resolver, &component_id, output)?;
        if matches!(component.class_id, 4 | 224)
            && transform_id.replace(component_id).is_some()
        {
            return Err(invalid("multiple node Transforms"));
        }
        components.push(component);
    }
    let transform_id = transform_id.ok_or_else(|| invalid("GameObject has no Transform"))?;
    let transform = resolver.object(&transform_id)?;
    let parent = resolver.resolve(&transform_id, property(&transform, "m_Father")?)?
        .map(|id| transform_game_object(resolver, &id)).transpose()?;
    let children = array(property(&transform, "m_Children")?)?.iter().map(|pointer| {
        let id = resolver.required_ref(&transform_id, pointer)?;
        transform_game_object(resolver, &id).map(|id| id.source())
    }).collect::<Result<Vec<_>, Error>>()?;
    Ok(PrefabNode {
        source: id.source(),
        name: text(property(&object, "m_Name")?)?.into(),
        active_self: boolean(property(&object, "m_IsActive")?)?,
        layer: integer(property(&object, "m_Layer")?)?,
        transform: transform_id.source(),
        parent: parent.map(|id| id.source()),
        children,
        bevy_local_transform: convert_local(
            vector(property(&transform, "m_LocalPosition")?, ["x", "y", "z"])?,
            vector(property(&transform, "m_LocalRotation")?, ["x", "y", "z", "w"])?,
            vector(property(&transform, "m_LocalScale")?, ["x", "y", "z"])?,
        )?,
        components,
    })
}

fn transform_game_object(resolver: &mut Resolver<'_>, id: &ObjectId) -> Result<ObjectId, Error> {
    let transform = resolver.object(id)?;
    if !matches!(transform.info.type_id, 4 | 224) {
        return Err(invalid("hierarchy reference is not a Transform"));
    }
    resolver.required_ref(id, property(&transform, "m_GameObject")?)
}

pub(crate) fn list_files(
    root: &Path,
    directory: &Path,
    result: &mut BTreeMap<String, String>,
) -> Result<(), Error> {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            list_files(root, &path, result)?;
        } else {
            let relative = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            if relative != "manifest.json" {
                result.insert(relative, file_sha256(&path)?);
            }
        }
    }
    Ok(())
}

fn validate_hierarchy(nodes: &[PrefabNode]) -> Result<Vec<SourceObject>, Error> {
    let ids = nodes
        .iter()
        .map(|node| (&node.source, node))
        .collect::<BTreeMap<_, _>>();
    if ids.len() != nodes.len() {
        return Err(invalid("duplicate prefab GameObject"));
    }
    let mut roots = Vec::new();
    for node in nodes {
        if let Some(parent) = &node.parent {
            let parent = ids
                .get(parent)
                .ok_or_else(|| invalid("prefab parent outside extracted hierarchy"))?;
            if !parent.children.contains(&node.source) {
                return Err(invalid("prefab parent/child link differs"));
            }
        } else {
            roots.push(node.source.clone());
        }
        let unique = node.children.iter().collect::<BTreeSet<_>>();
        if unique.len() != node.children.len() {
            return Err(invalid("duplicate prefab child"));
        }
        for child in &node.children {
            if ids
                .get(child)
                .is_none_or(|child| child.parent.as_ref() != Some(&node.source))
            {
                return Err(invalid("prefab child/parent link differs"));
            }
        }
        let mut seen = BTreeSet::new();
        let mut current = Some(&node.source);
        while let Some(id) = current {
            if !seen.insert(id) || seen.len() > 128 {
                return Err(invalid("cyclic/deep prefab hierarchy"));
            }
            current = ids.get(id).and_then(|node| node.parent.as_ref());
        }
    }
    if roots.is_empty() && !nodes.is_empty() {
        return Err(invalid("prefab has no root"));
    }
    Ok(roots)
}

pub fn run(apk_path: &Path, member: &str, output_root: &Path) -> Result<PrefabReport, Error> {
    let source = source_apk(apk_path)?;
    let index_path = output_root.join("imported_map/global_asset_index.json");
    let fallback = Path::new("assets/imported_map/global_asset_index.json");
    let index: GlobalIndex = serde_json::from_reader(File::open(if index_path.exists() {
        &index_path
    } else {
        fallback
    })?)?;
    let mut resolver = Resolver::new(apk_path, &index)?;
    let bundle = resolver.bundle(member)?;
    let stem = Path::new(member)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| invalid("prefab bundle has no filename"))?;
    if !stem
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(invalid("unsafe prefab package name"));
    }
    let bundle_hash = format!("{:x}", Sha256::digest(member.as_bytes()));
    let imported = output_root.join("imported_prefabs").join(&source.sha256);
    std::fs::create_dir_all(&imported)?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let candidate = imported.join(format!(".prefab-import-{stamp}"));
    std::fs::create_dir(&candidate)?;
    let canonical = imported.join(format!("{stem}-{}", &bundle_hash[..12]));
    let mut classes = BTreeMap::new();
    let mut game_objects = Vec::new();
    for (cab, asset) in bundle.asset_names.iter().zip(&bundle.assets) {
        for info in &asset.objects {
            *classes.entry(info.type_id).or_default() += 1;
            let id = ObjectId {
                bundle: member.into(),
                cab: cab_name(cab).into(),
                path_id: info.path_id,
            };
            if info.type_id == 1 {
                game_objects.push(id);
            }
        }
    }
    let mut nodes = Vec::new();
    let mut renderers = Vec::new();
    let mut collision_meshes = BTreeMap::<ObjectId, PrefabCollisionMesh>::new();
    let mut meshes = BTreeMap::<String, MeshRecord>::new();
    for id in game_objects {
        let node = source_node(&mut resolver, &id, &candidate)?;
        let mut component_ids = Vec::new();
        for component in node.components.iter().skip(1) {
            component_ids.push(ObjectId {
                bundle: component.source.bundle.clone(),
                cab: component.source.cab.clone(),
                path_id: component.source.path_id.parse()?,
            });
        }
        let mut filter = None;
        let mut renderer = None;
        for (component, source_component) in component_ids.iter().zip(node.components.iter().skip(1)) {
            match source_component.class_id {
                33 => filter = Some(component.clone()),
                23 => renderer = Some(component.clone()),
                64 => {
                    let object = resolver.object(component)?;
                    let mesh_id = resolver.required_ref(component, property(&object, "m_Mesh")?)?;
                    if !collision_meshes.contains_key(&mesh_id) {
                        let object = resolver.mesh_object(&mesh_id)?;
                        let (positions, indices) = mesh::decode_collision(&object, reflection())?;
                        collision_meshes.insert(
                            mesh_id.clone(),
                            PrefabCollisionMesh {
                                source: mesh_id.source(),
                                positions,
                                indices,
                            },
                        );
                    }
                }
                _ => {}
            }
        }
        if let (Some(filter), Some(renderer)) = (filter, renderer) {
            let filter_object = resolver.object(&filter)?;
            let renderer_object = resolver.object(&renderer)?;
            let mesh_id = resolver.required_ref(&filter, property(&filter_object, "m_Mesh")?)?;
            let source_mesh = resolver.mesh_object(&mesh_id)?;
            let mut parts = Vec::new();
            for (part, mut record) in mesh::decode(&source_mesh, reflection())?
                .into_iter()
                .enumerate()
            {
                let key = format!("{}:{}@{part}", mesh_id.cab, mesh_id.path_id);
                record.name = key.clone();
                meshes.entry(key.clone()).or_insert(record);
                parts.push(key);
            }
            let materials = array(property(&renderer_object, "m_Materials")?)?
                .iter()
                .map(|pointer| {
                    let material = resolver.required_ref(&renderer, pointer)?;
                    let (decoded, unsupported) = resolver.prefab_material(&material, &candidate)?;
                    let mut unsupported_textures = BTreeMap::new();
                    for (name, source) in unsupported {
                        let id = ObjectId {
                            bundle: source.bundle,
                            cab: source.cab,
                            path_id: source.path_id.parse()?,
                        };
                        unsupported_textures
                            .insert(name, raw_component(&mut resolver, &id, &candidate)?);
                    }
                    Ok(PrefabMaterial {
                        material: decoded,
                        evidence: raw_component(&mut resolver, &material, &candidate)?,
                        unsupported_textures,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            if materials.len() < parts.len() {
                return Err(invalid(
                    "prefab renderer has fewer materials than mesh parts",
                ));
            }
            renderers.push(PrefabRender {
                source: renderer.source(),
                game_object: id.source(),
                mesh: mesh_id.source(),
                mesh_parts: parts,
                materials,
            });
        }
        nodes.push(node);
    }
    nodes.sort_by(|a, b| a.source.cmp(&b.source));
    let roots = validate_hierarchy(&nodes)?;
    let records = meshes.into_values().collect::<Vec<_>>();
    let geometry_path = candidate.join("meshes.bin");
    let mut writer = BufWriter::new(File::create(&geometry_path)?);
    write_mesh_archive(&mut writer, TERRAIN_MAGIC, &records)?;
    writer.flush()?;
    writer.get_ref().sync_all()?;
    let mut retained_files = BTreeMap::new();
    list_files(&candidate, &candidate, &mut retained_files)?;
    let limitations = vec!["Root positions are serialized prefab values, not authenticated runtime placements.".into(), "All component fields/bytes are retained, including renderer shadow/lightmap metadata and collider flags; MonoBehaviour execution and skinned animation conversion remain separate work.".into(), "Material and source 2D texture mip assets are decoded; non-2D samplers retain full fields/bytes and explicit unsupported status. Shader execution and original runtime appearance remain unverified.".into(), "Unresolved built-in references remain explicit; decoded mesh/material dependencies are required.".into(), "Bundle-cache statistics bound cache-owned payloads, excluding caller-held handles and parsed object metadata.".into()];
    let manifest = PrefabManifest {
        schema_version: 1,
        source_apk: source,
        bundle: member.into(),
        coordinate_frame: "bevy_right_handed_y_up".into(),
        geometry: FileIdentity {
            path: "meshes.bin".into(),
            sha256: file_sha256(&geometry_path)?,
        },
        roots,
        nodes,
        renderers,
        collision_meshes: collision_meshes.into_values().collect(),
        class_counts: classes,
        retained_files,
        limitations: limitations.clone(),
    };
    serde_json::to_writer_pretty(File::create(candidate.join("manifest.json"))?, &manifest)?;
    let loaded = load_prefab_package(&candidate)?;
    if loaded.records != records {
        return Err(invalid("prefab geometry roundtrip differs"));
    }
    let after = source_apk(apk_path)?;
    if after.sha256 != manifest.source_apk.sha256
        || after.byte_count != manifest.source_apk.byte_count
    {
        return Err(invalid("APK changed during prefab extraction"));
    }
    let backup = output_root
        .join("../.local/prefab-import-backups")
        .join(stamp.to_string());
    crate::package_publish::publish_verified(&candidate, &canonical, &backup)?;
    let unsupported_samplers = manifest
        .renderers
        .iter()
        .flat_map(|renderer| &renderer.materials)
        .map(|material| material.unsupported_textures.len())
        .sum();
    Ok(PrefabReport {
        package: canonical.display().to_string(),
        bundle: member.into(),
        nodes: manifest.nodes.len(),
        renderers: manifest.renderers.len(),
        mesh_parts: records.len(),
        collision_meshes: manifest.collision_meshes.len(),
        unsupported_samplers,
        verified_files: manifest.retained_files.len(),
        bundle_cache: resolver.cache_stats(),
        limitations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prefab_transform_reflects_translation_and_keeps_nonuniform_scale_once() {
        let local = Mat4::from_cols_array(
            &convert_local([1., 2., 3.], [0., 0., 0., 1.], [2., 3., 4.]).unwrap(),
        );
        assert_eq!(local.transform_point3(Vec3::ZERO), Vec3::new(1., 2., -3.));
        assert_eq!(local.transform_vector3(Vec3::ONE), Vec3::new(2., 3., 4.));
        assert!(convert_local([0.; 3], [0.; 4], [1.; 3]).is_err());
    }
}
