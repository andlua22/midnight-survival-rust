//! Engine-independent map records shared by the APK importer, renderer and physics.

use std::{
    collections::BTreeMap,
    fmt,
    io::{Cursor, Read, Write},
    path::{Component, Path},
};

use serde::{Deserialize, Serialize};

mod package;
mod terrain;
mod texture;

pub use package::{TerrainPackage, file_sha256, load_terrain_package, mesh_bounds};
pub use terrain::{TerrainParameters, terrain_albedo, terrain_tangent_normal};
pub use texture::{
    RGBA_MIP_MAGIC, RgbaMipChain, read_rgba_mip_archive, rgba_mip_byte_len, write_rgba_mip_archive,
};

/// Version three preserves source tangent frames as well as Bevy world
/// positions/normals and top-left PNG UVs. Older archives are retained as evidence.
pub const TERRAIN_MAGIC: &[u8; 8] = b"MSMAP003";
pub const TERRAIN_MANIFEST_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct MeshRecord {
    pub name: String,
    pub texture: String,
    pub center: [f32; 3],
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// Empty when the source mesh has no tangent channel; never regenerated for canonical terrain.
    pub tangents: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

#[derive(Debug)]
pub enum ArchiveError {
    Io(std::io::Error),
    Invalid(String),
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Invalid(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for ArchiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Invalid(_) => None,
        }
    }
}

impl From<std::io::Error> for ArchiveError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

fn invalid(message: impl Into<String>) -> ArchiveError {
    ArchiveError::Invalid(message.into())
}

fn read_array<const N: usize>(reader: &mut impl Read) -> Result<[u8; N], ArchiveError> {
    let mut bytes = [0; N];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn read_string(reader: &mut impl Read) -> Result<String, ArchiveError> {
    let len = usize::from(u16::from_le_bytes(read_array(reader)?));
    let mut bytes = vec![0; len];
    reader.read_exact(&mut bytes)?;
    String::from_utf8(bytes).map_err(|_| invalid("mesh name/texture is not UTF-8"))
}

fn read_vector<const N: usize>(reader: &mut impl Read) -> Result<[f32; N], ArchiveError> {
    let mut vector = [0.0; N];
    for component in &mut vector {
        *component = f32::from_le_bytes(read_array(reader)?);
    }
    Ok(vector)
}

pub fn validate_mesh(record: &MeshRecord) -> Result<(), ArchiveError> {
    let count = record.positions.len();
    if record.name.is_empty() || count == 0 {
        return Err(invalid("a mesh requires a name and vertices"));
    }
    if record.normals.len() != count
        || record.uvs.len() != count
        || (!record.tangents.is_empty() && record.tangents.len() != count)
    {
        return Err(invalid(format!(
            "{}: vertex attribute counts differ",
            record.name
        )));
    }
    if record.indices.is_empty()
        || !record.indices.len().is_multiple_of(3)
        || record.indices.iter().any(|&index| index as usize >= count)
    {
        return Err(invalid(format!(
            "{}: invalid triangle indices",
            record.name
        )));
    }
    if record
        .center
        .iter()
        .chain(record.positions.iter().flatten())
        .chain(record.normals.iter().flatten())
        .chain(record.tangents.iter().flatten())
        .chain(record.uvs.iter().flatten())
        .any(|number| !number.is_finite())
    {
        return Err(invalid(format!("{}: non-finite vertex data", record.name)));
    }
    Ok(())
}

/// Reject malformed or truncated archives as a whole; never return a plausible prefix.
pub fn read_mesh_archive(bytes: &[u8], magic: &[u8; 8]) -> Result<Vec<MeshRecord>, ArchiveError> {
    let mut reader = Cursor::new(bytes);
    if &read_array::<8>(&mut reader)? != magic {
        return Err(invalid("unexpected mesh archive version"));
    }
    let count = u32::from_le_bytes(read_array(&mut reader)?) as usize;
    if count > bytes.len().saturating_sub(12) / 56 {
        return Err(invalid("mesh record count exceeds archive size"));
    }
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        let name = read_string(&mut reader)?;
        let texture = read_string(&mut reader)?;
        let center = read_vector(&mut reader)?;
        let vertex_count = u32::from_le_bytes(read_array(&mut reader)?) as usize;
        let index_count = u32::from_le_bytes(read_array(&mut reader)?) as usize;
        let flags = u32::from_le_bytes(read_array(&mut reader)?);
        if flags & !1 != 0 {
            return Err(invalid("unknown mesh attribute flags"));
        }
        let size = vertex_count
            .checked_mul(if flags & 1 != 0 { 48 } else { 32 })
            .and_then(|vertex_bytes| index_count.checked_mul(4)?.checked_add(vertex_bytes))
            .ok_or_else(|| invalid("mesh counts overflow"))?;
        if size > bytes.len().saturating_sub(reader.position() as usize) {
            return Err(invalid(format!("{name}: truncated vertex/index data")));
        }
        let positions = (0..vertex_count)
            .map(|_| read_vector(&mut reader))
            .collect::<Result<_, _>>()?;
        let normals = (0..vertex_count)
            .map(|_| read_vector(&mut reader))
            .collect::<Result<_, _>>()?;
        let uvs = (0..vertex_count)
            .map(|_| read_vector(&mut reader))
            .collect::<Result<_, _>>()?;
        let tangents = (0..if flags & 1 != 0 { vertex_count } else { 0 })
            .map(|_| read_vector(&mut reader))
            .collect::<Result<_, _>>()?;
        let indices = (0..index_count)
            .map(|_| Ok(u32::from_le_bytes(read_array(&mut reader)?)))
            .collect::<Result<_, ArchiveError>>()?;
        let record = MeshRecord {
            name,
            texture,
            center,
            positions,
            normals,
            tangents,
            uvs,
            indices,
        };
        validate_mesh(&record)?;
        records.push(record);
    }
    if reader.position() as usize != bytes.len() {
        return Err(invalid("mesh archive has trailing data"));
    }
    Ok(records)
}

fn write_string(writer: &mut impl Write, value: &str) -> Result<(), ArchiveError> {
    let len =
        u16::try_from(value.len()).map_err(|_| invalid("mesh string exceeds format limit"))?;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(value.as_bytes())?;
    Ok(())
}

pub fn write_mesh_archive(
    writer: &mut impl Write,
    magic: &[u8; 8],
    records: &[MeshRecord],
) -> Result<(), ArchiveError> {
    for record in records {
        validate_mesh(record)?;
    }
    let count = u32::try_from(records.len()).map_err(|_| invalid("too many mesh records"))?;
    writer.write_all(magic)?;
    writer.write_all(&count.to_le_bytes())?;
    for record in records {
        write_string(writer, &record.name)?;
        write_string(writer, &record.texture)?;
        for component in record.center {
            writer.write_all(&component.to_le_bytes())?;
        }
        let vertex_count =
            u32::try_from(record.positions.len()).map_err(|_| invalid("too many vertices"))?;
        let index_count =
            u32::try_from(record.indices.len()).map_err(|_| invalid("too many indices"))?;
        writer.write_all(&vertex_count.to_le_bytes())?;
        writer.write_all(&index_count.to_le_bytes())?;
        writer.write_all(&u32::from(!record.tangents.is_empty()).to_le_bytes())?;
        for component in record
            .positions
            .iter()
            .flatten()
            .chain(record.normals.iter().flatten())
            .chain(record.uvs.iter().flatten())
            .chain(record.tangents.iter().flatten())
        {
            writer.write_all(&component.to_le_bytes())?;
        }
        for index in &record.indices {
            writer.write_all(&index.to_le_bytes())?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceObject {
    pub bundle: String,
    pub cab: String,
    /// Signed Unity PathIDs must remain lossless in JSON readers.
    pub path_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceApk {
    pub path: String,
    pub sha256: String,
    pub byte_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIdentity {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureAsset {
    pub file: FileIdentity,
    /// Complete decoded source mip chain; PNG is a base-level preview.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu: Option<FileIdentity>,
    pub source: SourceObject,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub unity_format: i32,
    pub unity_color_space: i32,
    pub unity_version: String,
    pub mip_count: u32,
    pub sampler: TextureSampler,
    /// Complete encoded texture payload, including its source mip chain.
    pub encoded: FileIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureSampler {
    pub filter_mode: i32,
    pub wrap: [i32; 3],
    pub aniso: i32,
    pub mip_bias: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureBinding {
    pub texture: Option<TextureAsset>,
    pub scale: [f32; 2],
    pub offset: [f32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMaterial {
    pub source: SourceObject,
    pub name: String,
    pub shader: SourceObject,
    pub shader_name: String,
    pub textures: BTreeMap<String, TextureBinding>,
    pub floats: BTreeMap<String, f32>,
    pub colors: BTreeMap<String, [f32; 4]>,
    pub shader_defaults: BTreeMap<String, [f32; 4]>,
    pub texture_defaults: BTreeMap<String, String>,
    pub valid_keywords: Vec<String>,
    pub render_queue: i32,
}

impl SourceMaterial {
    pub fn terrain_layer_count(&self) -> Option<usize> {
        match self.shader_name.as_str() {
            "_URP/TerrainMeshBase/3 Textures" => Some(3),
            "_URP/TerrainMeshBase/4 Textures" => Some(4),
            _ => None,
        }
    }

    pub fn float(&self, name: &str) -> Option<f32> {
        self.floats
            .get(name)
            .copied()
            .or_else(|| self.shader_defaults.get(name).map(|value| value[0]))
    }

    pub fn uses_terrain_normal_maps(&self) -> bool {
        self.terrain_layer_count().is_some()
            && self
                .valid_keywords
                .iter()
                .any(|keyword| keyword == "_NORMALMAP")
    }

    pub fn terrain_sampler_names(&self) -> Vec<String> {
        let Some(layers) = self.terrain_layer_count() else {
            return Vec::new();
        };
        let mut names = vec!["_Control".into()];
        for layer in 0..layers {
            names.push(format!("_Splat{layer}"));
            if self.uses_terrain_normal_maps() {
                names.push(format!("_Splat{layer}_Normal"));
            }
        }
        names
    }

    /// ShaderLab's declared built-in colours for an explicit null sampler.
    pub fn default_texture_color(&self, name: &str) -> Option<[f32; 4]> {
        match self.texture_defaults.get(name)?.as_str() {
            "white" => Some([1.0; 4]),
            "black" => Some([0.0; 4]),
            "grey" | "gray" => Some([0.5; 4]),
            "red" => Some([1.0, 0.0, 0.0, 0.0]),
            "bump" => Some([0.5, 0.5, 1.0, 0.5]),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCollider {
    pub source: SourceObject,
    pub mesh: SourceObject,
    pub enabled: bool,
    pub is_trigger: bool,
    pub convex: bool,
    pub cooking_options: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneMesh {
    pub source: SourceObject,
    pub game_object: SourceObject,
    pub world_transform: [f32; 16],
    pub bounds: [f32; 6],
    pub material: SourceMaterial,
    pub collider: Option<SourceCollider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainManifest {
    pub schema_version: u32,
    pub source_apk: SourceApk,
    pub coordinate_frame: String,
    pub uv_origin: String,
    pub geometry: FileIdentity,
    pub meshes: BTreeMap<String, SceneMesh>,
    pub limitations: Vec<String>,
}

/// Paths in a generated package may only resolve inside that package.
pub fn package_path(root: &Path, relative: &str) -> Result<std::path::PathBuf, ArchiveError> {
    let path = Path::new(relative);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(invalid(format!("invalid map asset path: {relative}")));
    }
    Ok(root.join(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> MeshRecord {
        MeshRecord {
            name: "asymmetric terrain".into(),
            texture: String::new(),
            center: [2.0, 4.0, -3.0],
            positions: vec![[0.0, 4.0, 0.0], [4.0, 4.0, 0.0], [0.0, 4.0, -6.0]],
            normals: vec![[0.0, 1.0, 0.0]; 3],
            tangents: vec![[1.0, 0.0, 0.0, -1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
        }
    }

    #[test]
    fn preserves_geometry_and_attributes_losslessly() {
        let records = vec![triangle()];
        let mut bytes = Vec::new();
        write_mesh_archive(&mut bytes, TERRAIN_MAGIC, &records).unwrap();
        assert_eq!(read_mesh_archive(&bytes, TERRAIN_MAGIC).unwrap(), records);
        assert!(read_mesh_archive(&bytes, b"MSMAP001").is_err());
    }

    #[test]
    fn preserves_source_tangent_absence_and_rejects_partial_channels() {
        let mut record = triangle();
        record.tangents.clear();
        let mut bytes = Vec::new();
        write_mesh_archive(&mut bytes, TERRAIN_MAGIC, &[record.clone()]).unwrap();
        assert_eq!(
            read_mesh_archive(&bytes, TERRAIN_MAGIC).unwrap(),
            [record.clone()]
        );
        record.tangents.push([1.0, 0.0, 0.0, 1.0]);
        assert!(write_mesh_archive(&mut Vec::new(), TERRAIN_MAGIC, &[record]).is_err());
    }

    #[test]
    fn refuses_unknown_vertex_attribute_flags() {
        let record = triangle();
        let flags_offset = 12 + 2 + record.name.len() + 2 + record.texture.len() + 12 + 8;
        let mut bytes = Vec::new();
        write_mesh_archive(&mut bytes, TERRAIN_MAGIC, &[record]).unwrap();
        bytes[flags_offset..flags_offset + 4].copy_from_slice(&2u32.to_le_bytes());
        assert!(
            read_mesh_archive(&bytes, TERRAIN_MAGIC)
                .unwrap_err()
                .to_string()
                .contains("attribute flags")
        );
    }

    #[test]
    fn refuses_every_truncated_prefix_and_trailing_bytes() {
        let mut bytes = Vec::new();
        write_mesh_archive(&mut bytes, TERRAIN_MAGIC, &[triangle()]).unwrap();
        for end in 0..bytes.len() {
            assert!(read_mesh_archive(&bytes[..end], TERRAIN_MAGIC).is_err());
        }
        bytes.push(0);
        assert!(read_mesh_archive(&bytes, TERRAIN_MAGIC).is_err());
    }

    #[test]
    fn refuses_out_of_range_indices_and_non_finite_vertices() {
        let mut record = triangle();
        record.indices[2] = 3;
        assert!(validate_mesh(&record).is_err());
        record.indices[2] = 2;
        record.positions[0][1] = f32::NAN;
        assert!(validate_mesh(&record).is_err());
    }

    #[test]
    fn package_paths_cannot_escape_the_map_directory() {
        let root = Path::new("assets/imported_map/terrain");
        for path in ["../old.bin", "/tmp/map.bin", "textures/../map.png", ""] {
            assert!(package_path(root, path).is_err());
        }
        assert_eq!(
            package_path(root, "textures/control.png").unwrap(),
            root.join("textures/control.png")
        );
    }
}
