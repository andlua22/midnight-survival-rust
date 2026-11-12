//! Independent source collider metadata and collision-only local mesh payloads.

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{
    ArchiveError, FileIdentity, SourceObject, TERRAIN_MANIFEST_VERSION, TerrainManifest,
    TerrainPackage, invalid, package_path, read_array, read_string, read_vector,
    validate_prefab_affine, write_string,
};

/// Positions and triangle indices only; no invented renderer attributes.
pub const COLLISION_MAGIC: &[u8; 8] = b"MSCOL001";

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainCollisionMesh {
    pub name: String,
    /// Bevy-local coordinates. Entry.world_transform places this mesh in world.
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TerrainCollisionShape {
    Box {
        /// Original Unity-local center; reflect Z once before applying the matrix.
        center: [f32; 3],
        /// Original positive Unity-local full extents.
        size: [f32; 3],
    },
    Mesh {
        mesh: SourceObject,
        record: String,
        convex: bool,
        cooking_options: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainCollisionEntry {
    pub source: SourceObject,
    pub game_object: SourceObject,
    pub transform: SourceObject,
    pub layer: u8,
    pub active_self: bool,
    /// Saved effective hierarchy activity, separately recovered from parent flags.
    pub active_in_hierarchy: bool,
    pub enabled: bool,
    pub is_trigger: bool,
    /// Bevy-local to Bevy-world, column-major affine matrix.
    pub world_transform: [f32; 16],
    pub shape: TerrainCollisionShape,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainCollisionManifest {
    pub geometry: FileIdentity,
    pub colliders: Vec<TerrainCollisionEntry>,
}

fn validate_source_id(source: &SourceObject) -> Result<(), ArchiveError> {
    let id = source
        .path_id
        .parse::<i64>()
        .map_err(|_| invalid("collision source PathID is not a signed integer"))?;
    if source.bundle.is_empty()
        || source.cab.is_empty()
        || id == 0
        || id.to_string() != source.path_id
    {
        return Err(invalid(
            "collision requires a qualified canonical source identity",
        ));
    }
    Ok(())
}

impl TerrainManifest {
    /// Validate source metadata without decoding payloads. This admission does
    /// not prove complete source inventory, PhysX cooking or runtime activity.
    pub fn validate_collision_metadata(&self) -> Result<(), ArchiveError> {
        self.validate_layers()?;
        match (self.schema_version, &self.collision) {
            (2 | 3, None) => return Ok(()),
            (2 | 3, Some(_)) => {
                return Err(invalid(
                    "legacy terrain schema cannot claim independent collision",
                ));
            }
            (TERRAIN_MANIFEST_VERSION, None) => {
                return Err(invalid(
                    "schema-four independent collision table is missing",
                ));
            }
            (_, Some(_)) => {}
            _ => return Err(invalid("unsupported terrain collision schema")),
        }
        let Some(collision) = &self.collision else {
            return Err(invalid("independent collision metadata is missing"));
        };
        package_path(Path::new("."), &collision.geometry.path)?;
        if collision.geometry.path == self.geometry.path {
            return Err(invalid(
                "collision geometry must be independent of renderer geometry",
            ));
        }
        if collision.geometry.sha256.len() != 64
            || !collision
                .geometry
                .sha256
                .bytes()
                .all(|value| value.is_ascii_hexdigit())
        {
            return Err(invalid("invalid terrain collision geometry identity"));
        }
        let mut entries = BTreeMap::new();
        let mut owner_layers = BTreeMap::new();
        let mut records = BTreeMap::new();
        let mut mesh_records = BTreeMap::new();
        for entry in &collision.colliders {
            for source in [&entry.source, &entry.game_object, &entry.transform] {
                validate_source_id(source)?;
            }
            if entries.insert(&entry.source, entry).is_some() {
                return Err(invalid("duplicate qualified terrain collider identity"));
            }
            if entry.layer >= 32 {
                return Err(invalid("terrain collider layer must be in 0..31"));
            }
            if let Some(previous) = owner_layers.insert(&entry.game_object, entry.layer)
                && previous != entry.layer
            {
                return Err(invalid(
                    "terrain collider GameObject has conflicting layers",
                ));
            }
            if !entry.active_self && entry.active_in_hierarchy {
                return Err(invalid(
                    "inactive source collider owner cannot be active in hierarchy",
                ));
            }
            validate_prefab_affine(&entry.world_transform)?;
            match &entry.shape {
                TerrainCollisionShape::Box { center, size } => {
                    if center.iter().chain(size).any(|value| !value.is_finite())
                        || size.iter().any(|value| *value <= 0.)
                    {
                        return Err(invalid("terrain BoxCollider has invalid center/size"));
                    }
                }
                TerrainCollisionShape::Mesh { mesh, record, .. } => {
                    validate_source_id(mesh)?;
                    if record.is_empty() {
                        return Err(invalid("terrain MeshCollider record is empty"));
                    }
                    if let Some(previous) = records.insert(record, mesh)
                        && previous != mesh
                    {
                        return Err(invalid(
                            "terrain collision record aliases different source meshes",
                        ));
                    }
                    if let Some(previous) = mesh_records.insert(mesh, record)
                        && previous != record
                    {
                        return Err(invalid(
                            "terrain source mesh has conflicting collision records",
                        ));
                    }
                }
            }
        }
        for mesh in self.meshes.values() {
            if let Some(layer) = owner_layers.get(&mesh.game_object)
                && mesh.layer != Some(*layer)
            {
                return Err(invalid("renderer and collider GameObject layers disagree"));
            }
            let Some(retained) = &mesh.collider else {
                continue;
            };
            let entry = entries
                .get(&retained.source)
                .ok_or_else(|| invalid("independent table omits a retained source collider"))?;
            let TerrainCollisionShape::Mesh {
                mesh: collision_mesh,
                convex,
                cooking_options,
                ..
            } = &entry.shape
            else {
                return Err(invalid(
                    "retained MeshCollider has a different independent shape",
                ));
            };
            if entry.game_object != mesh.game_object
                || Some(entry.layer) != mesh.layer
                || entry.enabled != retained.enabled
                || entry.is_trigger != retained.is_trigger
                || collision_mesh != &retained.mesh
                || *convex != retained.convex
                || *cooking_options != retained.cooking_options
            {
                return Err(invalid(
                    "independent collision differs from retained source metadata",
                ));
            }
        }
        Ok(())
    }
}

pub fn validate_collision_mesh(mesh: &TerrainCollisionMesh) -> Result<(), ArchiveError> {
    if mesh.name.is_empty() || mesh.positions.is_empty() {
        return Err(invalid("a collision mesh requires a name and vertices"));
    }
    if mesh
        .positions
        .iter()
        .flatten()
        .any(|value| !value.is_finite())
    {
        return Err(invalid("non-finite terrain collision vertex data"));
    }
    if mesh.indices.is_empty()
        || !mesh.indices.len().is_multiple_of(3)
        || mesh
            .indices
            .iter()
            .any(|&index| index as usize >= mesh.positions.len())
    {
        return Err(invalid("invalid terrain collision triangle indices"));
    }
    Ok(())
}

/// Reject truncated/count-overflow archives as a whole, including trailing data.
pub fn read_collision_archive(bytes: &[u8]) -> Result<Vec<TerrainCollisionMesh>, ArchiveError> {
    let mut reader = Cursor::new(bytes);
    if &read_array::<8>(&mut reader)? != COLLISION_MAGIC {
        return Err(invalid("unexpected collision archive version"));
    }
    let count = u32::from_le_bytes(read_array(&mut reader)?) as usize;
    // A valid record has a nonempty name, one vertex and at least one triangle.
    if count > bytes.len().saturating_sub(12) / 35 {
        return Err(invalid("collision record count exceeds archive size"));
    }
    let mut meshes = Vec::with_capacity(count);
    let mut names = BTreeSet::new();
    for _ in 0..count {
        let name = read_string(&mut reader)?;
        let vertex_count = u32::from_le_bytes(read_array(&mut reader)?) as usize;
        let index_count = u32::from_le_bytes(read_array(&mut reader)?) as usize;
        let size = vertex_count
            .checked_mul(12)
            .and_then(|vertex_bytes| index_count.checked_mul(4)?.checked_add(vertex_bytes))
            .ok_or_else(|| invalid("collision mesh counts overflow"))?;
        if size > bytes.len().saturating_sub(reader.position() as usize) {
            return Err(invalid("truncated collision vertex/index data"));
        }
        let positions = (0..vertex_count)
            .map(|_| read_vector(&mut reader))
            .collect::<Result<_, _>>()?;
        let indices = (0..index_count)
            .map(|_| Ok(u32::from_le_bytes(read_array(&mut reader)?)))
            .collect::<Result<_, ArchiveError>>()?;
        let mesh = TerrainCollisionMesh {
            name,
            positions,
            indices,
        };
        validate_collision_mesh(&mesh)?;
        if !names.insert(mesh.name.clone()) {
            return Err(invalid("duplicate terrain collision mesh record"));
        }
        meshes.push(mesh);
    }
    if reader.position() as usize != bytes.len() {
        return Err(invalid("collision archive has trailing data"));
    }
    Ok(meshes)
}

/// Validate the whole set before writing any bytes; callers stage publication.
pub fn write_collision_archive(
    writer: &mut impl Write,
    meshes: &[TerrainCollisionMesh],
) -> Result<(), ArchiveError> {
    let mut names = BTreeSet::new();
    for mesh in meshes {
        validate_collision_mesh(mesh)?;
        if !names.insert(&mesh.name) {
            return Err(invalid("duplicate terrain collision mesh record"));
        }
        u16::try_from(mesh.name.len())
            .map_err(|_| invalid("collision name exceeds format limit"))?;
        u32::try_from(mesh.positions.len()).map_err(|_| invalid("too many collision vertices"))?;
        u32::try_from(mesh.indices.len()).map_err(|_| invalid("too many collision indices"))?;
    }
    let count = u32::try_from(meshes.len()).map_err(|_| invalid("too many collision records"))?;
    writer.write_all(COLLISION_MAGIC)?;
    writer.write_all(&count.to_le_bytes())?;
    for mesh in meshes {
        write_string(writer, &mesh.name)?;
        writer.write_all(&(mesh.positions.len() as u32).to_le_bytes())?;
        writer.write_all(&(mesh.indices.len() as u32).to_le_bytes())?;
        for component in mesh.positions.iter().flatten() {
            writer.write_all(&component.to_le_bytes())?;
        }
        for index in &mesh.indices {
            writer.write_all(&index.to_le_bytes())?;
        }
    }
    Ok(())
}

impl TerrainPackage {
    pub fn collision_mesh(&self, name: &str) -> Option<&TerrainCollisionMesh> {
        self.collision_meshes.iter().find(|mesh| mesh.name == name)
    }

    /// Also used by shared physics for in-memory packages, before any mutation.
    pub fn validate_collision_geometry(&self) -> Result<(), ArchiveError> {
        self.manifest.validate_collision_metadata()?;
        let Some(collision) = &self.manifest.collision else {
            if !self.collision_meshes.is_empty() {
                return Err(invalid(
                    "legacy terrain cannot contain independent collision meshes",
                ));
            }
            return Ok(());
        };
        let mut names = BTreeSet::new();
        for mesh in &self.collision_meshes {
            validate_collision_mesh(mesh)?;
            if !names.insert(mesh.name.as_str()) {
                return Err(invalid("duplicate terrain collision mesh record"));
            }
        }
        let referenced = collision
            .colliders
            .iter()
            .filter_map(|entry| match &entry.shape {
                TerrainCollisionShape::Mesh { record, .. } => Some(record.as_str()),
                TerrainCollisionShape::Box { .. } => None,
            })
            .collect::<BTreeSet<_>>();
        if names != referenced {
            return Err(invalid(
                "terrain collision mesh records and references differ",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> TerrainCollisionMesh {
        TerrainCollisionMesh {
            name: "qualified local collision".into(),
            positions: vec![[1., 2., -3.], [4., 2., -3.], [1., 2., -7.]],
            indices: vec![0, 1, 2],
        }
    }

    #[test]
    fn collision_archive_preserves_local_geometry_without_renderer_attributes() {
        let original = triangle();
        let mut bytes = Vec::new();
        write_collision_archive(&mut bytes, std::slice::from_ref(&original)).unwrap();
        assert_eq!(&bytes[..8], b"MSCOL001");
        assert_eq!(read_collision_archive(&bytes).unwrap(), [original]);
        let mut empty = Vec::new();
        write_collision_archive(&mut empty, &[]).unwrap();
        assert!(read_collision_archive(&empty).unwrap().is_empty());
    }

    #[test]
    fn collision_archive_rejects_truncation_overflow_duplicates_and_invalid_geometry() {
        let original = triangle();
        let mut bytes = Vec::new();
        write_collision_archive(&mut bytes, std::slice::from_ref(&original)).unwrap();
        for end in 0..bytes.len() {
            assert!(read_collision_archive(&bytes[..end]).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(read_collision_archive(&trailing).is_err());
        let mut counts = bytes.clone();
        counts[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(read_collision_archive(&counts).is_err());
        let mut duplicate_payload = bytes.clone();
        duplicate_payload[8..12].copy_from_slice(&2u32.to_le_bytes());
        duplicate_payload.extend_from_slice(&bytes[12..]);
        assert!(read_collision_archive(&duplicate_payload).is_err());
        let mut duplicates = Vec::new();
        assert!(
            write_collision_archive(&mut duplicates, &[original.clone(), original.clone()])
                .is_err()
        );
        assert!(duplicates.is_empty());
        for corrupt in 0..2 {
            let mut mesh = original.clone();
            if corrupt == 0 {
                mesh.indices[1] = 99;
            } else {
                mesh.positions[0][1] = f32::NAN;
            }
            assert!(write_collision_archive(&mut Vec::new(), &[mesh]).is_err());
        }
    }
}
