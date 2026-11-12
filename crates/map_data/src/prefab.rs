//! Verified source prefab data. A prefab root transform is never a world placement.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    ArchiveError, FileIdentity, MeshRecord, SourceApk, SourceMaterial, SourceObject, TERRAIN_MAGIC,
    invalid, package_path, read_mesh_archive, read_rgba_mip_archive,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceComponent {
    pub source: SourceObject,
    pub class_id: i32,
    pub serialized: FileIdentity,
    pub fields: Value,
}

impl SourceComponent {
    pub fn boolean(&self, name: &str) -> Result<bool, ArchiveError> {
        self.fields
            .get(name)
            .and_then(Value::as_bool)
            .ok_or_else(|| invalid(format!("{}: missing boolean {name}", self.source.path_id)))
    }

    pub fn integer(&self, name: &str) -> Result<i64, ArchiveError> {
        self.fields
            .get(name)
            .and_then(Value::as_i64)
            .ok_or_else(|| invalid(format!("{}: missing integer {name}", self.source.path_id)))
    }

    pub fn vector3(&self, name: &str) -> Result<[f32; 3], ArchiveError> {
        let fields = self
            .fields
            .get(name)
            .ok_or_else(|| invalid(format!("missing vector {name}")))?;
        let mut result = [0.; 3];
        for (index, key) in ["x", "y", "z"].iter().enumerate() {
            result[index] = fields
                .get(*key)
                .and_then(Value::as_f64)
                .ok_or_else(|| invalid(format!("missing {name}.{key}")))?
                as f32;
        }
        if result.iter().any(|v| !v.is_finite()) {
            return Err(invalid(format!("non-finite {name}")));
        }
        Ok(result)
    }

    pub fn reference(&self, name: &str) -> Result<SourceObject, ArchiveError> {
        resolved_reference(
            self.fields
                .get(name)
                .ok_or_else(|| invalid(format!("missing reference {name}")))?,
        )
    }
}

pub(crate) fn resolved_reference(value: &Value) -> Result<SourceObject, ArchiveError> {
    if value.get("state").and_then(Value::as_str) != Some("resolved") {
        return Err(invalid("required source reference is not resolved"));
    }
    let source: SourceObject = serde_json::from_value(
        value
            .get("target")
            .cloned()
            .ok_or_else(|| invalid("resolved reference has no target"))?,
    )
    .map_err(|error| invalid(format!("source reference: {error}")))?;
    validate_source(&source)?;
    Ok(source)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabNode {
    pub source: SourceObject,
    pub name: String,
    pub active_self: bool,
    pub layer: i64,
    pub transform: SourceObject,
    pub parent: Option<SourceObject>,
    pub children: Vec<SourceObject>,
    /// Column-major Bevy-local matrix. Geometry has already been reflected separately.
    pub bevy_local_transform: [f32; 16],
    pub components: Vec<SourceComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabRender {
    pub source: SourceObject,
    pub game_object: SourceObject,
    pub mesh: SourceObject,
    pub mesh_parts: Vec<String>,
    pub materials: Vec<PrefabMaterial>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabMaterial {
    pub material: SourceMaterial,
    pub evidence: SourceComponent,
    pub unsupported_textures: BTreeMap<String, SourceComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabCollisionMesh {
    pub source: SourceObject,
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabManifest {
    pub schema_version: u32,
    pub source_apk: SourceApk,
    pub bundle: String,
    pub coordinate_frame: String,
    pub geometry: FileIdentity,
    pub nodes: Vec<PrefabNode>,
    pub roots: Vec<SourceObject>,
    pub renderers: Vec<PrefabRender>,
    pub collision_meshes: Vec<PrefabCollisionMesh>,
    pub class_counts: BTreeMap<i32, usize>,
    pub retained_files: BTreeMap<String, String>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PrefabNodePose {
    /// Composed prefab-space matrix, including the serialized root transform.
    pub world_transform: [f32; 16],
    pub active_in_hierarchy: bool,
}

#[derive(Debug)]
pub struct PrefabPackage {
    pub root: PathBuf,
    pub manifest: PrefabManifest,
    pub manifest_sha256: String,
    pub records: Vec<MeshRecord>,
    /// Aligned with manifest.nodes; parent order in the manifest is irrelevant.
    pub poses: Vec<PrefabNodePose>,
}

impl PrefabPackage {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn texture_path(&self, relative: &str) -> Result<PathBuf, ArchiveError> {
        if !self.manifest.retained_files.contains_key(relative) {
            return Err(invalid("asset is not in the verified prefab inventory"));
        }
        contained_path(&self.root, relative)
    }

    pub fn node_index(&self, source: &SourceObject) -> Option<usize> {
        self.manifest
            .nodes
            .iter()
            .position(|node| &node.source == source)
    }

    /// Unity's placement loader assigns the selected root's TRS. Do not multiply
    /// its serialized prefab root transform into a runtime placement a second time.
    /// Only descendants of that root are returned. Input is already Bevy-space.
    pub fn placed_poses(
        &self,
        root: &SourceObject,
        placement: [f32; 16],
    ) -> Result<Vec<Option<PrefabNodePose>>, ArchiveError> {
        validate_prefab_affine(&placement)?;
        let index = self
            .node_index(root)
            .ok_or_else(|| invalid("unknown prefab root"))?;
        if !self.manifest.roots.contains(root) {
            return Err(invalid("placement must select a prefab root"));
        }
        let indices = self
            .manifest
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (&node.source, index))
            .collect::<BTreeMap<_, _>>();
        if indices.len() != self.manifest.nodes.len() {
            return Err(invalid("duplicate prefab node"));
        }
        let mut result = vec![None; self.manifest.nodes.len()];
        result[index] = Some(PrefabNodePose {
            world_transform: placement,
            active_in_hierarchy: self.manifest.nodes[index].active_self,
        });
        let mut pending = vec![index];
        let mut visited = BTreeSet::new();
        while let Some(parent) = pending.pop() {
            if !visited.insert(parent) {
                return Err(invalid("cyclic/duplicate prefab child"));
            }
            for child in &self.manifest.nodes[parent].children {
                let child = *indices.get(child).ok_or_else(|| invalid("missing child"))?;
                if self.manifest.nodes[child].parent.as_ref()
                    != Some(&self.manifest.nodes[parent].source)
                {
                    return Err(invalid("prefab parent/child links differ"));
                }
                let pose = result[parent]
                    .as_ref()
                    .ok_or_else(|| invalid("missing parent pose"))?;
                result[child] = Some(PrefabNodePose {
                    world_transform: prefab_affine_product(
                        &pose.world_transform,
                        &self.manifest.nodes[child].bevy_local_transform,
                    )?,
                    active_in_hierarchy: pose.active_in_hierarchy
                        && self.manifest.nodes[child].active_self,
                });
                pending.push(child);
            }
        }
        Ok(result)
    }
}

pub fn validate_prefab_affine(matrix: &[f32; 16]) -> Result<(), ArchiveError> {
    if matrix.iter().any(|v| !v.is_finite())
        || [matrix[3], matrix[7], matrix[11], matrix[15]] != [0., 0., 0., 1.]
    {
        return Err(invalid("non-finite or non-affine prefab transform"));
    }
    // Singular scales cannot provide valid collision or normal transforms.
    let determinant = f64::from(matrix[0])
        * (f64::from(matrix[5]) * f64::from(matrix[10])
            - f64::from(matrix[9]) * f64::from(matrix[6]))
        - f64::from(matrix[4])
            * (f64::from(matrix[1]) * f64::from(matrix[10])
                - f64::from(matrix[9]) * f64::from(matrix[2]))
        + f64::from(matrix[8])
            * (f64::from(matrix[1]) * f64::from(matrix[6])
                - f64::from(matrix[5]) * f64::from(matrix[2]));
    if determinant == 0. || !determinant.is_finite() {
        return Err(invalid("singular prefab transform"));
    }
    Ok(())
}

pub fn prefab_affine_product(a: &[f32; 16], b: &[f32; 16]) -> Result<[f32; 16], ArchiveError> {
    validate_prefab_affine(a)?;
    validate_prefab_affine(b)?;
    let mut result = [0.; 16];
    for column in 0..4 {
        for row in 0..4 {
            result[column * 4 + row] = (0..4).map(|i| a[i * 4 + row] * b[column * 4 + i]).sum();
        }
    }
    validate_prefab_affine(&result)?;
    Ok(result)
}

pub fn prefab_transform_point(
    matrix: &[f32; 16],
    point: [f32; 3],
) -> Result<[f32; 3], ArchiveError> {
    validate_prefab_affine(matrix)?;
    let result = std::array::from_fn(|row| {
        matrix[row] * point[0]
            + matrix[4 + row] * point[1]
            + matrix[8 + row] * point[2]
            + matrix[12 + row]
    });
    if result.iter().any(|v| !v.is_finite()) {
        return Err(invalid("prefab vertex transform overflow"));
    }
    Ok(result)
}

pub(crate) fn validate_source(source: &SourceObject) -> Result<(), ArchiveError> {
    if source.bundle.is_empty()
        || source.cab.is_empty()
        || source.path_id.parse::<i64>().ok().is_none_or(|id| id == 0)
    {
        return Err(invalid("invalid source object identity"));
    }
    Ok(())
}

pub(crate) fn valid_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub(crate) fn contained_path(root: &Path, relative: &str) -> Result<PathBuf, ArchiveError> {
    let path = package_path(root, relative)?;
    let canonical = path.canonicalize()?;
    if !canonical.starts_with(root) {
        return Err(invalid("prefab symlink escapes package"));
    }
    Ok(canonical)
}

pub(crate) fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, ArchiveError> {
    let file = File::open(path)?;
    if file.metadata()?.len() > limit {
        return Err(invalid("prefab file exceeds byte limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(invalid("prefab file exceeds byte limit"));
    }
    Ok(bytes)
}

pub(crate) fn retained_identity(
    retained_files: &BTreeMap<String, String>,
    identity: &FileIdentity,
) -> Result<(), ArchiveError> {
    if retained_files.get(&identity.path) != Some(&identity.sha256) {
        return Err(invalid(format!(
            "{}: conflicting or missing retained identity",
            identity.path
        )));
    }
    Ok(())
}

fn validate_field_blobs(retained_files: &BTreeMap<String, String>, value: &Value) -> Result<(), ArchiveError> {
    match value {
        Value::Object(fields) => {
            if let Some(blob) = fields.get("unity_bytes") {
                let identity: FileIdentity = serde_json::from_value(blob.clone())
                    .map_err(|error| invalid(format!("source blob identity: {error}")))?;
                retained_identity(retained_files, &identity)?;
            }
            for value in fields.values() {
                validate_field_blobs(retained_files, value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_field_blobs(retained_files, value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn validate_component(
    retained_files: &BTreeMap<String, String>,
    component: &SourceComponent,
) -> Result<(), ArchiveError> {
    validate_source(&component.source)?;
    retained_identity(retained_files, &component.serialized)?;
    validate_field_blobs(retained_files, &component.fields)
}

fn validate_node_evidence(
    node: &PrefabNode,
    nodes: &[PrefabNode],
    ids: &BTreeMap<&SourceObject, usize>,
) -> Result<(), ArchiveError> {
    let game_object = node
        .components
        .iter()
        .find(|c| c.class_id == 1 && c.source == node.source)
        .ok_or_else(|| invalid("missing source GameObject evidence"))?;
    if game_object.fields.get("m_Name").and_then(Value::as_str) != Some(node.name.as_str())
        || game_object.boolean("m_IsActive")? != node.active_self
        || game_object.integer("m_Layer")? != node.layer
    {
        return Err(invalid(
            "node name/activity/layer differs from source GameObject",
        ));
    }
    let declared = game_object
        .fields
        .get("m_Component")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("missing GameObject component order"))?;
    let components = node
        .components
        .iter()
        .filter(|c| c.class_id != 1)
        .collect::<Vec<_>>();
    if declared.len() != components.len() {
        return Err(invalid("source component count differs"));
    }
    for (pointer, component) in declared.iter().zip(components) {
        if resolved_reference(pointer.get("component").unwrap_or(pointer))? != component.source {
            return Err(invalid("source component order differs"));
        }
    }
    let transform = node
        .components
        .iter()
        .find(|c| matches!(c.class_id, 4 | 224) && c.source == node.transform)
        .ok_or_else(|| invalid("missing source Transform evidence"))?;
    let position = transform.vector3("m_LocalPosition")?;
    let scale = transform.vector3("m_LocalScale")?;
    let rotation = transform
        .fields
        .get("m_LocalRotation")
        .ok_or_else(|| invalid("missing source rotation"))?;
    let mut q = [0.; 4];
    for (value, axis) in q.iter_mut().zip(["x", "y", "z", "w"]) {
        *value = rotation
            .get(axis)
            .and_then(Value::as_f64)
            .ok_or_else(|| invalid("missing quaternion axis"))? as f32;
    }
    let length_squared: f32 = q.iter().map(|v| v * v).sum();
    if !length_squared.is_finite() || length_squared < 1e-8 {
        return Err(invalid("invalid source quaternion"));
    }
    let reciprocal = length_squared.sqrt().recip();
    for value in &mut q {
        *value *= reciprocal;
    }
    let [x, y, z, w] = q;
    let [x2, y2, z2] = [x + x, y + y, z + z];
    let [xx, yy, zz, xy, xz, yz, wx, wy, wz] = [
        x * x2,
        y * y2,
        z * z2,
        x * y2,
        x * z2,
        y * z2,
        w * x2,
        w * y2,
        w * z2,
    ];
    let columns = [
        [1. - (yy + zz), xy + wz, xz - wy],
        [xy - wz, 1. - (xx + zz), yz + wx],
        [xz + wy, yz - wx, 1. - (xx + yy)],
    ];
    // F * TRS * F with F=diag(1,1,-1,1), matching the importer.
    // Floating evaluation may differ by a few ulps from SIMD glam operations.
    for (column, axis) in columns.iter().enumerate() {
        for (row, &component) in axis.iter().enumerate() {
            let sign = if (column == 2) != (row == 2) { -1. } else { 1. };
            let expected = component * scale[column] * sign;
            if (node.bevy_local_transform[column * 4 + row] - expected).abs()
                > 0.00002 * expected.abs().max(1.)
            {
                return Err(invalid("local prefab matrix differs from source TRS"));
            }
        }
    }
    if node.bevy_local_transform[12..15] != [position[0], position[1], -position[2]] {
        return Err(invalid("prefab translation differs from source position"));
    }
    if let Some(parent) = &node.parent {
        if transform.reference("m_Father")? != nodes[ids[parent]].transform {
            return Err(invalid("source Transform parent differs"));
        }
    } else if transform
        .fields
        .get("m_Father")
        .and_then(|v| v.get("state"))
        .and_then(Value::as_str)
        != Some("null")
    {
        return Err(invalid("source Transform root has a parent"));
    }
    let children = transform
        .fields
        .get("m_Children")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("missing source Transform children"))?;
    if children.len() != node.children.len() {
        return Err(invalid("source Transform child count differs"));
    }
    for (pointer, child) in children.iter().zip(&node.children) {
        if resolved_reference(pointer)? != nodes[ids[child]].transform {
            return Err(invalid("source Transform child order differs"));
        }
    }
    Ok(())
}

pub(crate) fn source_hierarchy(
    nodes: &[PrefabNode],
    declared_roots: &[SourceObject],
    retained_files: &BTreeMap<String, String>,
) -> Result<Vec<PrefabNodePose>, ArchiveError> {
    let ids = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (&n.source, i))
        .collect::<BTreeMap<_, _>>();
    if ids.len() != nodes.len() || ids.is_empty() {
        return Err(invalid("duplicate/empty prefab hierarchy"));
    }
    let mut transforms = BTreeSet::new();
    let mut roots = BTreeSet::new();
    for node in nodes {
        validate_source(&node.source)?;
        validate_source(&node.transform)?;
        validate_prefab_affine(&node.bevy_local_transform)?;
        if !transforms.insert(&node.transform) || !(0..32).contains(&node.layer) {
            return Err(invalid("duplicate transform or invalid source layer"));
        }
        if let Some(parent) = &node.parent {
            if ids
                .get(parent)
                .is_none_or(|&i| !nodes[i].children.contains(&node.source))
            {
                return Err(invalid("prefab parent/child links differ"));
            }
        } else {
            roots.insert(&node.source);
        }
        if node.children.iter().collect::<BTreeSet<_>>().len() != node.children.len() {
            return Err(invalid("duplicate prefab child"));
        }
        for child in &node.children {
            if ids
                .get(child)
                .is_none_or(|&i| nodes[i].parent.as_ref() != Some(&node.source))
            {
                return Err(invalid("prefab child/parent links differ"));
            }
        }
        let mut components = BTreeSet::new();
        for component in &node.components {
            if !components.insert(&component.source) {
                return Err(invalid("duplicate node component"));
            }
            validate_component(retained_files, component)?;
            if component.class_id != 1 && component.reference("m_GameObject")? != node.source {
                return Err(invalid("component belongs to another GameObject"));
            }
        }
        if !node
            .components
            .iter()
            .any(|c| c.class_id == 1 && c.source == node.source)
            || !node
                .components
                .iter()
                .any(|c| matches!(c.class_id, 4 | 224) && c.source == node.transform)
        {
            return Err(invalid("missing source GameObject/Transform evidence"));
        }
        validate_node_evidence(node, nodes, &ids)?;
    }
    if roots.is_empty()
        || roots.len() != declared_roots.len()
        || roots != declared_roots.iter().collect::<BTreeSet<_>>()
    {
        return Err(invalid("prefab root inventory differs"));
    }
    let mut poses = Vec::with_capacity(nodes.len());
    for index in 0..nodes.len() {
        let mut path = Vec::new();
        let mut visited = BTreeSet::new();
        let mut current = Some(index);
        while let Some(i) = current {
            if !visited.insert(i) || path.len() >= 128 {
                return Err(invalid("cyclic/deep prefab hierarchy"));
            }
            path.push(i);
            current = nodes[i]
                .parent
                .as_ref()
                .and_then(|p| ids.get(p).copied());
        }
        let root = path.pop().ok_or_else(|| invalid("empty prefab path"))?;
        let mut pose = PrefabNodePose {
            world_transform: nodes[root].bevy_local_transform,
            active_in_hierarchy: nodes[root].active_self,
        };
        for i in path.into_iter().rev() {
            pose.world_transform = prefab_affine_product(
                &pose.world_transform,
                &nodes[i].bevy_local_transform,
            )?;
            pose.active_in_hierarchy &= nodes[i].active_self;
        }
        poses.push(pose);
    }
    Ok(poses)
}

/// Common bounded payload verification for source prefab and player packages.
pub(crate) fn verify_retained_files(
    root: &Path,
    retained_files: &BTreeMap<String, String>,
) -> Result<(), ArchiveError> {
    let mut total = 0u64;
    for (relative, hash) in retained_files {
        if relative == "manifest.json" || !valid_hash(hash) {
            return Err(invalid("invalid retained source identity"));
        }
        let bytes = read_bounded(&contained_path(root, relative)?, 256 * 1024 * 1024)?;
        total = total.checked_add(bytes.len() as u64)
            .ok_or_else(|| invalid("source package bytes overflow"))?;
        if total > 1024 * 1024 * 1024 {
            return Err(invalid("source package exceeds byte limit"));
        }
        if format!("{:x}", Sha256::digest(&bytes)) != *hash {
            return Err(invalid(format!("source retained checksum differs: {relative}")));
        }
    }
    Ok(())
}

/// Hash every retained file and validate all executable render/collision references.
/// Unsupported components and samplers remain evidence for explicit runtime handling.
pub fn load_prefab_package(package: &Path) -> Result<PrefabPackage, ArchiveError> {
    let root = package.canonicalize()?;
    let manifest_path = contained_path(&root, "manifest.json")?;
    let bytes = read_bounded(&manifest_path, 64 * 1024 * 1024)?;
    let manifest_sha256 = format!("{:x}", Sha256::digest(&bytes));
    let manifest: PrefabManifest = serde_json::from_slice(&bytes)
        .map_err(|error| invalid(format!("prefab manifest: {error}")))?;
    if manifest.schema_version != 1
        || manifest.coordinate_frame != "bevy_right_handed_y_up"
        || !valid_hash(&manifest.source_apk.sha256)
        || manifest.bundle.is_empty()
        || manifest.retained_files.len() > 100_000
        || manifest.nodes.len() > 100_000
    {
        return Err(invalid("unsupported prefab conventions/version/limits"));
    }
    retained_identity(&manifest.retained_files, &manifest.geometry)?;
    let poses = source_hierarchy(&manifest.nodes, &manifest.roots, &manifest.retained_files)?;
    verify_retained_files(&root, &manifest.retained_files)?;
    let geometry = read_bounded(
        &contained_path(&root, &manifest.geometry.path)?,
        256 * 1024 * 1024,
    )?;
    if format!("{:x}", Sha256::digest(&geometry)) != manifest.geometry.sha256 {
        return Err(invalid("prefab geometry changed while loading"));
    }
    let records = read_mesh_archive(&geometry, TERRAIN_MAGIC)?;
    let parts = records
        .iter()
        .map(|r| r.name.as_str())
        .collect::<BTreeSet<_>>();
    if parts.len() != records.len() {
        return Err(invalid("duplicate prefab geometry part"));
    }
    let mut used = BTreeSet::new();
    let mut renderers = BTreeSet::new();
    let mut mips = BTreeMap::new();
    for renderer in &manifest.renderers {
        validate_source(&renderer.source)?;
        validate_source(&renderer.mesh)?;
        let node = manifest
            .nodes
            .iter()
            .find(|n| n.source == renderer.game_object)
            .ok_or_else(|| invalid("renderer GameObject is missing"))?;
        if !renderers.insert(&renderer.source)
            || renderer.mesh_parts.is_empty()
            || renderer.materials.len() < renderer.mesh_parts.len()
            || !node
                .components
                .iter()
                .any(|c| c.class_id == 23 && c.source == renderer.source)
        {
            return Err(invalid("renderer evidence/part/material inventory differs"));
        }
        let filter = node
            .components
            .iter()
            .find(|c| c.class_id == 33)
            .ok_or_else(|| invalid("renderer has no MeshFilter"))?;
        if filter.reference("m_Mesh")? != renderer.mesh {
            return Err(invalid("renderer mesh differs from filter"));
        }
        let evidence = node
            .components
            .iter()
            .find(|c| c.source == renderer.source)
            .ok_or_else(|| invalid("missing renderer evidence"))?;
        let materials = evidence
            .fields
            .get("m_Materials")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid("missing source renderer material order"))?;
        if materials.len() != renderer.materials.len() {
            return Err(invalid("renderer material count differs from source"));
        }
        for (pointer, material) in materials.iter().zip(&renderer.materials) {
            if resolved_reference(pointer)? != material.material.source {
                return Err(invalid("renderer material order differs from source"));
            }
        }
        for (index, part) in renderer.mesh_parts.iter().enumerate() {
            if *part != format!("{}:{}@{index}", renderer.mesh.cab, renderer.mesh.path_id)
                || !parts.contains(part.as_str())
            {
                return Err(invalid("renderer mesh part is missing"));
            }
            used.insert(part.as_str());
        }
        for material in &renderer.materials {
            let source = &material.material;
            validate_source(&source.source)?;
            if source
                .floats
                .values()
                .chain(source.colors.values().flatten())
                .chain(source.shader_defaults.values().flatten())
                .any(|v| !v.is_finite())
            {
                return Err(invalid("non-finite prefab material parameters"));
            }
            validate_component(&manifest.retained_files, &material.evidence)?;
            if material.evidence.class_id != 21
                || material.evidence.source != material.material.source
            {
                return Err(invalid("material evidence identity differs"));
            }
            for component in material.unsupported_textures.values() {
                validate_component(&manifest.retained_files, component)?;
            }
            for binding in material.material.textures.values() {
                if binding
                    .scale
                    .iter()
                    .chain(&binding.offset)
                    .any(|v| !v.is_finite())
                {
                    return Err(invalid("non-finite texture transform"));
                }
                if let Some(texture) = &binding.texture {
                    validate_source(&texture.source)?;
                    let gpu = texture
                        .gpu
                        .as_ref()
                        .ok_or_else(|| invalid("missing prefab source mip chain"))?;
                    for identity in [&texture.file, &texture.encoded, gpu] {
                        retained_identity(&manifest.retained_files, identity)?;
                    }
                    let metadata = (texture.width, texture.height, texture.mip_count);
                    if mips
                        .insert(gpu.path.clone(), (gpu.sha256.clone(), metadata))
                        .is_some_and(|old| old != (gpu.sha256.clone(), metadata))
                    {
                        return Err(invalid("conflicting prefab mip metadata"));
                    }
                }
            }
        }
    }
    if parts != used {
        return Err(invalid("unreferenced prefab mesh parts"));
    }
    for (relative, (hash, metadata)) in mips {
        let bytes = read_bounded(&contained_path(&root, &relative)?, 256 * 1024 * 1024)?;
        if format!("{:x}", Sha256::digest(&bytes)) != hash {
            return Err(invalid("prefab mip changed while loading"));
        }
        let chain = read_rgba_mip_archive(&bytes)?;
        if (chain.width, chain.height, chain.mip_count) != metadata {
            return Err(invalid("prefab mip metadata differs"));
        }
    }
    let mut collisions = BTreeSet::new();
    for mesh in &manifest.collision_meshes {
        validate_source(&mesh.source)?;
        if !collisions.insert(&mesh.source)
            || mesh.positions.is_empty()
            || mesh.indices.is_empty()
            || !mesh.indices.len().is_multiple_of(3)
            || mesh
                .indices
                .iter()
                .any(|&i| i as usize >= mesh.positions.len())
            || mesh.positions.iter().flatten().any(|v| !v.is_finite())
        {
            return Err(invalid("invalid/duplicate source collision mesh"));
        }
    }
    for node in &manifest.nodes {
        for component in node.components.iter().filter(|c| c.class_id == 64) {
            if !collisions.contains(&component.reference("m_Mesh")?) {
                return Err(invalid("source collider mesh is missing"));
            }
        }
    }
    if read_bounded(&manifest_path, 64 * 1024 * 1024)? != bytes {
        return Err(invalid("prefab manifest changed while loading"));
    }
    Ok(PrefabPackage {
        root,
        manifest,
        manifest_sha256,
        records,
        poses,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const IDENTITY: [f32; 16] = [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ];

    fn source(id: i64) -> SourceObject {
        SourceObject {
            bundle: "assets/source.ab".into(),
            cab: "CAB-source".into(),
            path_id: id.to_string(),
        }
    }

    struct Fixture {
        root: PathBuf,
        manifest: PrefabManifest,
    }

    impl Fixture {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "midnight-prefab-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&root).unwrap();
            let mut geometry = Vec::new();
            crate::write_mesh_archive(&mut geometry, TERRAIN_MAGIC, &[]).unwrap();
            std::fs::write(root.join("meshes.bin"), &geometry).unwrap();
            let geometry = FileIdentity {
                path: "meshes.bin".into(),
                sha256: format!("{:x}", Sha256::digest(&geometry)),
            };
            std::fs::write(root.join("source.bin"), b"source evidence").unwrap();
            let evidence = FileIdentity {
                path: "source.bin".into(),
                sha256: format!("{:x}", Sha256::digest(b"source evidence")),
            };
            let component = |id, class_id, owner| SourceComponent {
                source: source(id),
                class_id,
                serialized: evidence.clone(),
                fields: json!({"m_GameObject": {"state":"resolved","target":source(owner)}}),
            };
            let mut parent = IDENTITY;
            parent[0] = 2.;
            parent[5] = 3.;
            parent[10] = 4.;
            parent[12] = 10.;
            parent[14] = -20.;
            let mut child = IDENTITY;
            child[12] = 1.;
            child[13] = 2.;
            child[14] = -3.;
            let mut nodes = vec![
                PrefabNode {
                    source: source(2),
                    name: "child".into(),
                    active_self: true,
                    layer: 0,
                    transform: source(12),
                    parent: Some(source(1)),
                    children: vec![],
                    bevy_local_transform: child,
                    components: vec![component(2, 1, 2), component(12, 4, 2)],
                },
                PrefabNode {
                    source: source(1),
                    name: "root".into(),
                    active_self: false,
                    layer: 0,
                    transform: source(11),
                    parent: None,
                    children: vec![source(2)],
                    bevy_local_transform: parent,
                    components: vec![component(1, 1, 1), component(11, 4, 1)],
                },
            ];
            let transforms = nodes
                .iter()
                .map(|n| (n.source.clone(), n.transform.clone()))
                .collect::<BTreeMap<_, _>>();
            for node in &mut nodes {
                let declared = node
                    .components
                    .iter()
                    .filter(|c| c.class_id != 1)
                    .map(|c| json!({"component":{"state":"resolved","target":c.source}}))
                    .collect::<Vec<_>>();
                node.components[0].fields["m_Component"] = json!(declared);
                node.components[0].fields["m_Name"] = json!(node.name);
                node.components[0].fields["m_IsActive"] = json!(node.active_self);
                node.components[0].fields["m_Layer"] = json!(node.layer);
                let m = node.bevy_local_transform;
                let fields = &mut node.components[1].fields;
                fields["m_LocalPosition"] = json!({"x":m[12],"y":m[13],"z":-m[14]});
                fields["m_LocalScale"] = json!({"x":m[0],"y":m[5],"z":m[10]});
                fields["m_LocalRotation"] = json!({"x":0.,"y":0.,"z":0.,"w":1.});
                fields["m_Father"] = match &node.parent {
                    Some(parent) => json!({"state":"resolved","target":transforms[parent]}),
                    None => json!({"state":"null"}),
                };
                fields["m_Children"] = json!(
                    node.children
                        .iter()
                        .map(|c| json!({"state":"resolved","target":transforms[c]}))
                        .collect::<Vec<_>>()
                );
            }
            let manifest = PrefabManifest {
                schema_version: 1,
                source_apk: SourceApk {
                    path: "source.apk".into(),
                    sha256: "a".repeat(64),
                    byte_count: 1,
                },
                bundle: "assets/source.ab".into(),
                coordinate_frame: "bevy_right_handed_y_up".into(),
                geometry: geometry.clone(),
                nodes,
                roots: vec![source(1)],
                renderers: vec![],
                collision_meshes: vec![],
                class_counts: BTreeMap::new(),
                retained_files: BTreeMap::from([
                    (geometry.path.clone(), geometry.sha256),
                    (evidence.path, evidence.sha256),
                ]),
                limitations: vec![],
            };
            Self { root, manifest }
        }
        fn save(&self) {
            std::fs::write(
                self.root.join("manifest.json"),
                serde_json::to_vec(&self.manifest).unwrap(),
            )
            .unwrap();
        }
        fn load(&self) -> Result<PrefabPackage, ArchiveError> {
            self.save();
            load_prefab_package(&self.root)
        }
        fn add_render(&mut self) -> MeshRecord {
            let identity = self.manifest.nodes[0].components[0].serialized.clone();
            let material = |id| PrefabMaterial {
                material: SourceMaterial {
                    source: source(id),
                    name: format!("material {id}"),
                    shader: source(60),
                    shader_name: "source shader".into(),
                    textures: BTreeMap::new(),
                    floats: BTreeMap::new(),
                    colors: BTreeMap::new(),
                    shader_defaults: BTreeMap::new(),
                    texture_defaults: BTreeMap::new(),
                    valid_keywords: vec![],
                    render_queue: 2000,
                },
                evidence: SourceComponent {
                    source: source(id),
                    class_id: 21,
                    serialized: identity.clone(),
                    fields: json!({}),
                },
                unsupported_textures: BTreeMap::new(),
            };
            let node = &mut self.manifest.nodes[0];
            node.components.push(SourceComponent {source:source(14),class_id:33,serialized:identity.clone(),
                fields:json!({"m_GameObject":{"state":"resolved","target":source(2)},"m_Mesh":{"state":"resolved","target":source(30)}})});
            node.components.push(SourceComponent {source:source(15),class_id:23,serialized:identity.clone(),
                fields:json!({"m_GameObject":{"state":"resolved","target":source(2)},"m_Materials":[{"state":"resolved","target":source(50)},{"state":"resolved","target":source(51)}]})});
            node.components[0].fields["m_Component"] = json!(
                node.components
                    .iter()
                    .filter(|c| c.class_id != 1)
                    .map(|c| json!({"component":{"state":"resolved","target":c.source}}))
                    .collect::<Vec<_>>()
            );
            let record = MeshRecord {
                name: "CAB-source:30@0".into(),
                texture: String::new(),
                center: [0.; 3],
                positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 0., -1.]],
                normals: vec![[0., 1., 0.]; 3],
                tangents: vec![],
                uvs: vec![[0., 0.]; 3],
                indices: vec![0, 1, 2],
            };
            self.manifest.renderers.push(PrefabRender {
                source: source(15),
                game_object: source(2),
                mesh: source(30),
                mesh_parts: vec![record.name.clone()],
                materials: vec![material(50), material(51)],
            });
            self.write_record(&record);
            record
        }
        fn write_record(&mut self, record: &MeshRecord) {
            let mut bytes = Vec::new();
            crate::write_mesh_archive(&mut bytes, TERRAIN_MAGIC, std::slice::from_ref(record))
                .unwrap();
            std::fs::write(self.root.join("meshes.bin"), &bytes).unwrap();
            self.manifest.geometry.sha256 = format!("{:x}", Sha256::digest(&bytes));
            self.manifest
                .retained_files
                .insert("meshes.bin".into(), self.manifest.geometry.sha256.clone());
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn composes_unsorted_hierarchy_preserving_scale_reflection_and_active_parent() {
        let fixture = Fixture::new();
        let package = fixture.load().unwrap();
        let child = &package.poses[0];
        assert_eq!(
            prefab_transform_point(&child.world_transform, [0.; 3]).unwrap(),
            [12., 6., -32.]
        );
        assert_eq!(
            prefab_transform_point(&child.world_transform, [1.; 3]).unwrap(),
            [14., 9., -28.]
        );
        assert!(!child.active_in_hierarchy);
        assert!(package.root.is_absolute());
        assert!(package.texture_path("source.bin").is_ok());
        assert!(package.texture_path("../other.bin").is_err());
    }

    #[test]
    fn placement_replaces_serialized_root_transform_and_never_reflects_again() {
        let package = Fixture::new().load().unwrap();
        let mut placement = IDENTITY;
        placement[12] = 100.;
        placement[14] = -200.;
        let poses = package.placed_poses(&source(1), placement).unwrap();
        assert_eq!(
            prefab_transform_point(&poses[0].as_ref().unwrap().world_transform, [0.; 3]).unwrap(),
            [101., 2., -203.]
        );
        assert!(package.placed_poses(&source(2), placement).is_err());
        assert!(package.placed_poses(&source(99), placement).is_err());
    }

    #[test]
    fn rejects_cycles_duplicate_links_missing_roots_and_nonaffine_transforms() {
        let mut fixture = Fixture::new();
        let original = fixture.manifest.clone();
        fixture.manifest.nodes[1].parent = Some(source(2));
        fixture.manifest.nodes[0].children.push(source(1));
        fixture.manifest.roots.clear();
        assert!(fixture.load().is_err());
        fixture.manifest = original.clone();
        fixture.manifest.nodes[1].children.push(source(2));
        assert!(fixture.load().is_err());
        fixture.manifest = original.clone();
        fixture.manifest.roots.clear();
        assert!(fixture.load().is_err());
        fixture.manifest = original;
        fixture.manifest.nodes[0].bevy_local_transform[3] = 1.;
        assert!(fixture.load().is_err());
        assert!(prefab_transform_point(&IDENTITY, [f32::NAN, 0., 0.]).is_err());
        assert!(validate_prefab_affine(&[0.; 16]).is_err());
    }

    #[test]
    fn rejects_changed_evidence_geometry_and_escaping_paths() {
        let mut fixture = Fixture::new();
        std::fs::write(fixture.root.join("source.bin"), b"changed").unwrap();
        assert!(fixture.load().unwrap_err().to_string().contains("checksum"));
        std::fs::write(fixture.root.join("source.bin"), b"source evidence").unwrap();
        fixture.manifest.geometry.path = "../meshes.bin".into();
        assert!(fixture.load().is_err());
        fixture.manifest.geometry.path = "meshes.bin".into();
        std::fs::write(fixture.root.join("meshes.bin"), b"truncated").unwrap();
        assert!(fixture.load().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_retained_symlink_escaping_package() {
        let mut fixture = Fixture::new();
        std::os::unix::fs::symlink("/etc/hostname", fixture.root.join("escaped.bin")).unwrap();
        fixture
            .manifest
            .retained_files
            .insert("escaped.bin".into(), "a".repeat(64));
        assert!(fixture.load().unwrap_err().to_string().contains("symlink"));
    }

    #[test]
    fn rejects_missing_collision_mesh_and_cross_owned_component() {
        let mut fixture = Fixture::new();
        let mut component = fixture.manifest.nodes[0].components[1].clone();
        component.source = source(20);
        component.class_id = 64;
        component.fields["m_Mesh"] = json!({"state":"resolved","target":source(30)});
        fixture.manifest.nodes[0].components.push(component);
        fixture.manifest.nodes[0].components[0].fields["m_Component"]
            .as_array_mut()
            .unwrap()
            .push(json!({"component":{"state":"resolved","target":source(20)}}));
        assert!(
            fixture
                .load()
                .unwrap_err()
                .to_string()
                .contains("collider mesh")
        );
        fixture.manifest.nodes[0].components[1].fields["m_GameObject"]["target"] = json!(source(1));
        assert!(
            fixture
                .load()
                .unwrap_err()
                .to_string()
                .contains("another GameObject")
        );
    }

    #[test]
    fn rejects_material_order_and_cross_mesh_part_substitution() {
        let mut fixture = Fixture::new();
        let mut record = fixture.add_render();
        assert!(fixture.load().is_ok());
        fixture.manifest.renderers[0].materials.swap(0, 1);
        assert!(
            fixture
                .load()
                .unwrap_err()
                .to_string()
                .contains("material order")
        );
        fixture.manifest.renderers[0].materials.swap(0, 1);
        record.name = "CAB-source:31@0".into();
        fixture.write_record(&record);
        fixture.manifest.renderers[0].mesh_parts[0] = record.name;
        assert!(
            fixture
                .load()
                .unwrap_err()
                .to_string()
                .contains("mesh part")
        );
    }

    #[test]
    fn rejects_derived_activity_and_transform_differing_from_source_fields() {
        let mut fixture = Fixture::new();
        fixture.manifest.nodes[0].active_self = false;
        assert!(fixture.load().unwrap_err().to_string().contains("activity"));
        fixture.manifest.nodes[0].active_self = true;
        fixture.manifest.nodes[0].bevy_local_transform[0] = 2.;
        assert!(fixture.load().unwrap_err().to_string().contains("TRS"));
    }

    #[test]
    #[ignore = "requires retained supermarket source prefab package"]
    fn verifies_retained_supermarket_prefab() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/imported_prefabs/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df/building_chaoshi_01-3be4bce3aeca");
        let package = load_prefab_package(&root).unwrap();
        assert_eq!(package.manifest.nodes.len(), 303);
        assert_eq!(package.manifest.renderers.len(), 298);
        assert_eq!(package.records.len(), 46);
        assert_eq!(package.manifest.collision_meshes.len(), 3);
        assert_eq!(package.manifest.retained_files.len(), 1314);
        assert_eq!(package.poses.len(), 303);
    }
}
