//! Source SceneInfo contract recovered from export/gs.battle.scmsg readers.
//! Parsing a matching byte layout alone does not authenticate its runtime role.

use crate::{fields::invalid, unity::Error};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub const MAX_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;
const MAX_LIST_ITEMS: usize = 1_000_000;
const MAX_STRING_BYTES: usize = 4096;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ByteSpan {
    pub offset: usize,
    pub byte_count: usize,
    pub sha256: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Spanned<T> {
    pub source: ByteSpan,
    pub value: T,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct SourceTransform {
    pub position: [f32; 3],
    pub scale: [f32; 3],
    /// Source Angles values; no Euler order or coordinate conversion is inferred.
    pub angles: [f32; 3],
}

#[derive(Debug, Serialize, PartialEq)]
pub struct GameObjectInfo {
    pub transform: SourceTransform,
    /// Preserve the authored signed short; negative table references are rejected.
    pub prefab_index: i16,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct GrassInfo {
    pub object: GameObjectInfo,
    pub brightness: i16,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct TreeObjectInfo {
    pub object: GameObjectInfo,
    pub tree_type: u8,
    pub id: i32,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct StealthGrassInfo {
    pub object: GameObjectInfo,
    pub id: i32,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct GameObjectWithLightmap {
    pub object: GameObjectInfo,
    pub lightmap_index: u8,
    pub lightmap_scale_offset: [f32; 4],
    pub uv_size: i32,
    /// Exact ByteVec2 values, including both source x/z bytes.
    pub uv2_data: Vec<[u8; 2]>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct CellSceneInfo {
    pub terrain_x: f32,
    pub terrain_y: f32,
    pub prefabs: Vec<Spanned<String>>,
    pub grass: Vec<Spanned<GrassInfo>>,
    pub trees: Vec<Spanned<TreeObjectInfo>>,
    pub stealth_grass: Vec<Spanned<StealthGrassInfo>>,
    pub things_no_lightmap: Vec<Spanned<GameObjectInfo>>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct OcclusionGroup {
    pub transform: SourceTransform,
    pub box_size: [f32; 3],
    pub box_center: [f32; 3],
    pub waikes: Vec<Spanned<GameObjectWithLightmap>>,
    pub kuangjias: Vec<Spanned<GameObjectWithLightmap>>,
    pub wujians: Vec<Spanned<GameObjectWithLightmap>>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct SceneInfo {
    pub terrain_lightmap_offset: u8,
    pub prefabs: Vec<Spanned<String>>,
    pub cells: Vec<Spanned<CellSceneInfo>>,
    pub big_size_objects: Vec<Spanned<GameObjectInfo>>,
    pub occlusion_groups: Vec<Spanned<OcclusionGroup>>,
}

pub struct DecodedScene {
    /// Complete original payload remains available; spans never contain preview bytes.
    pub raw_bytes: Vec<u8>,
    pub scene: Spanned<SceneInfo>,
}

impl DecodedScene {
    pub fn bytes_for(&self, span: &ByteSpan) -> Result<&[u8], Error> {
        let end = span
            .offset
            .checked_add(span.byte_count)
            .ok_or_else(|| invalid("source span overflow"))?;
        self.raw_bytes
            .get(span.offset..end)
            .ok_or_else(|| invalid("source span outside payload"))
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Reader<'_> {
    fn error(&self, detail: &str) -> Error {
        invalid(format!("SceneInfo byte {}: {detail}", self.offset))
    }

    fn take<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or_else(|| self.error("offset overflow"))?;
        let source = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| self.error("truncated record"))?;
        let mut result = [0; N];
        result.copy_from_slice(source);
        self.offset = end;
        Ok(result)
    }

    fn byte(&mut self) -> Result<u8, Error> {
        Ok(self.take::<1>()?[0])
    }
    fn short(&mut self) -> Result<i16, Error> {
        Ok(i16::from_le_bytes(self.take()?))
    }
    fn int(&mut self) -> Result<i32, Error> {
        Ok(i32::from_le_bytes(self.take()?))
    }

    fn float(&mut self) -> Result<f32, Error> {
        let value = f32::from_le_bytes(self.take()?);
        if !value.is_finite() {
            return Err(self.error("non-finite float"));
        }
        Ok(value)
    }

    fn vector<const N: usize>(&mut self) -> Result<[f32; N], Error> {
        let mut values = [0.0; N];
        for value in &mut values {
            *value = self.float()?;
        }
        Ok(values)
    }

    fn count(&mut self, minimum_item_bytes: usize, limit: usize) -> Result<usize, Error> {
        let count = usize::try_from(self.int()?).map_err(|_| self.error("negative count"))?;
        if count > limit || count > (self.bytes.len() - self.offset) / minimum_item_bytes {
            return Err(self.error("count exceeds remaining payload or limit"));
        }
        Ok(count)
    }

    fn spanned<T>(
        &mut self,
        read: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<Spanned<T>, Error> {
        let start = self.offset;
        let value = read(self)?;
        Ok(Spanned {
            source: ByteSpan {
                offset: start,
                byte_count: self.offset - start,
                sha256: format!("{:x}", Sha256::digest(&self.bytes[start..self.offset])),
            },
            value,
        })
    }

    fn list<T>(
        &mut self,
        minimum_item_bytes: usize,
        mut read: impl FnMut(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<Spanned<T>>, Error> {
        let count = self.count(minimum_item_bytes, MAX_LIST_ITEMS)?;
        (0..count).map(|_| self.spanned(&mut read)).collect()
    }

    fn string(&mut self) -> Result<String, Error> {
        let count = self.count(1, MAX_STRING_BYTES)?;
        let end = self.offset + count;
        let value = std::str::from_utf8(&self.bytes[self.offset..end])
            .map_err(|_| self.error("invalid UTF-8 prefab name"))?
            .to_owned();
        self.offset = end;
        Ok(value)
    }

    fn transform(&mut self) -> Result<SourceTransform, Error> {
        Ok(SourceTransform {
            position: self.vector()?,
            scale: self.vector()?,
            angles: self.vector()?,
        })
    }

    fn object(&mut self, prefab_count: usize) -> Result<GameObjectInfo, Error> {
        let transform = self.transform()?;
        let prefab_index = self.short()?;
        if prefab_index < 0 || prefab_index as usize >= prefab_count {
            return Err(self.error("prefab index outside its source table"));
        }
        Ok(GameObjectInfo {
            transform,
            prefab_index,
        })
    }

    fn lightmapped_object(&mut self, prefab_count: usize) -> Result<GameObjectWithLightmap, Error> {
        let object = self.object(prefab_count)?;
        let lightmap_index = self.byte()?;
        let lightmap_scale_offset = self.vector()?;
        let uv_size = self.int()?;
        let count = self.count(2, MAX_LIST_ITEMS)?;
        let uv2_data = (0..count).map(|_| self.take()).collect::<Result<_, _>>()?;
        Ok(GameObjectWithLightmap {
            object,
            lightmap_index,
            lightmap_scale_offset,
            uv_size,
            uv2_data,
        })
    }

    fn cell(&mut self) -> Result<CellSceneInfo, Error> {
        let terrain_x = self.float()?;
        let terrain_y = self.float()?;
        let prefabs = self.list(4, Self::string)?;
        let prefab_count = prefabs.len();
        let grass = self.list(40, |r| {
            Ok(GrassInfo {
                object: r.object(prefab_count)?,
                brightness: r.short()?,
            })
        })?;
        let trees = self.list(43, |r| {
            Ok(TreeObjectInfo {
                object: r.object(prefab_count)?,
                tree_type: r.byte()?,
                id: r.int()?,
            })
        })?;
        let stealth_grass = self.list(42, |r| {
            Ok(StealthGrassInfo {
                object: r.object(prefab_count)?,
                id: r.int()?,
            })
        })?;
        let things_no_lightmap = self.list(38, |r| r.object(prefab_count))?;
        Ok(CellSceneInfo {
            terrain_x,
            terrain_y,
            prefabs,
            grass,
            trees,
            stealth_grass,
            things_no_lightmap,
        })
    }

    fn occlusion(&mut self, prefab_count: usize) -> Result<OcclusionGroup, Error> {
        let transform = self.transform()?;
        let box_size = self.vector()?;
        let box_center = self.vector()?;
        let waikes = self.list(63, |r| r.lightmapped_object(prefab_count))?;
        let kuangjias = self.list(63, |r| r.lightmapped_object(prefab_count))?;
        let wujians = self.list(63, |r| r.lightmapped_object(prefab_count))?;
        Ok(OcclusionGroup {
            transform,
            box_size,
            box_center,
            waikes,
            kuangjias,
            wujians,
        })
    }

    fn scene(&mut self) -> Result<SceneInfo, Error> {
        let terrain_lightmap_offset = self.byte()?;
        let prefabs = self.list(4, Self::string)?;
        let cells = self.list(28, Self::cell)?;
        let big_size_objects = self.list(38, |r| r.object(prefabs.len()))?;
        let occlusion_groups = self.list(72, |r| r.occlusion(prefabs.len()))?;
        Ok(SceneInfo {
            terrain_lightmap_offset,
            prefabs,
            cells,
            big_size_objects,
            occlusion_groups,
        })
    }
}

pub fn decode_scene_info(raw_bytes: Vec<u8>) -> Result<DecodedScene, Error> {
    if raw_bytes.len() > MAX_PAYLOAD_BYTES {
        return Err(invalid("SceneInfo payload exceeds 16 MiB limit"));
    }
    let mut reader = Reader {
        bytes: &raw_bytes,
        offset: 0,
    };
    let scene = reader.spanned(Reader::scene)?;
    if reader.offset != raw_bytes.len() {
        return Err(reader.error("trailing bytes or unsupported layout"));
    }
    Ok(DecodedScene { raw_bytes, scene })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn append_int(bytes: &mut Vec<u8>, value: i32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn append_name(bytes: &mut Vec<u8>, name: &str) {
        append_int(bytes, name.len() as i32);
        bytes.extend_from_slice(name.as_bytes());
    }
    fn append_transform(bytes: &mut Vec<u8>) {
        for value in [12.5_f32, -3.0, 400.0, 2.0, 3.0, -4.0, 10.0, 20.0, 30.0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }

    fn fixture() -> Vec<u8> {
        let mut bytes = vec![7];
        append_int(&mut bytes, 1);
        append_name(&mut bytes, "global_prefab");
        append_int(&mut bytes, 1);
        bytes.extend_from_slice(&100.0_f32.to_le_bytes());
        bytes.extend_from_slice(&200.0_f32.to_le_bytes());
        append_int(&mut bytes, 1);
        append_name(&mut bytes, "cell_tree");
        append_int(&mut bytes, 0); // grass
        append_int(&mut bytes, 1); // trees
        append_transform(&mut bytes);
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.push(3);
        append_int(&mut bytes, -12345);
        append_int(&mut bytes, 0); // stealth
        append_int(&mut bytes, 0); // things
        append_int(&mut bytes, 1); // big-size objects
        append_transform(&mut bytes);
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        append_int(&mut bytes, 0); // occlusion
        bytes
    }

    #[test]
    fn preserves_distinct_prefab_tables_nonuniform_scale_ids_and_exact_spans() {
        let bytes = fixture();
        let decoded = decode_scene_info(bytes.clone()).unwrap();
        assert_eq!(decoded.raw_bytes, bytes);
        assert_eq!(decoded.bytes_for(&decoded.scene.source).unwrap(), bytes);
        let scene = &decoded.scene.value;
        assert_eq!(scene.terrain_lightmap_offset, 7);
        assert_eq!(scene.prefabs[0].value, "global_prefab");
        let cell = &scene.cells[0].value;
        assert_eq!(cell.prefabs[0].value, "cell_tree");
        let tree = &cell.trees[0];
        assert_eq!(tree.value.id, -12345);
        assert_eq!(tree.value.tree_type, 3);
        assert_eq!(tree.value.object.transform.scale, [2.0, 3.0, -4.0]);
        assert_eq!(tree.value.object.transform.angles, [10.0, 20.0, 30.0]);
        let raw = decoded.bytes_for(&tree.source).unwrap();
        assert_eq!(raw.len(), 43);
        assert_eq!(tree.source.sha256, format!("{:x}", Sha256::digest(raw)));
        assert_eq!(&raw[39..43], &(-12345_i32).to_le_bytes());
    }

    #[test]
    fn rejects_truncation_trailing_bytes_nonfinite_values_and_invalid_source_indices() {
        let bytes = fixture();
        for count in 0..bytes.len() {
            assert!(
                decode_scene_info(bytes[..count].to_vec()).is_err(),
                "prefix {count}"
            );
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(decode_scene_info(trailing).is_err());
        let decoded = decode_scene_info(bytes.clone()).unwrap();
        let offset = decoded.scene.value.cells[0].value.trees[0].source.offset;
        let mut invalid_float = bytes.clone();
        invalid_float[offset..offset + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(decode_scene_info(invalid_float).is_err());
        let mut index = bytes;
        index[offset + 36..offset + 38].copy_from_slice(&65535_u16.to_le_bytes());
        assert!(decode_scene_info(index).is_err());
    }

    #[test]
    fn validates_counts_before_allocating() {
        for count in [-1_i32, i32::MAX] {
            let mut bytes = vec![0];
            append_int(&mut bytes, count);
            assert!(decode_scene_info(bytes).is_err());
        }
        assert!(decode_scene_info(vec![0; MAX_PAYLOAD_BYTES + 1]).is_err());
    }

    #[test]
    fn retains_grass_stealth_objects_occlusion_and_packed_lightmap_uv_bytes() {
        let mut bytes = vec![0];
        append_int(&mut bytes, 1);
        append_name(&mut bytes, "global");
        append_int(&mut bytes, 1);
        bytes.extend_from_slice(&0.0_f32.to_le_bytes());
        bytes.extend_from_slice(&0.0_f32.to_le_bytes());
        append_int(&mut bytes, 1);
        append_name(&mut bytes, "cell");
        append_int(&mut bytes, 1); // grass
        append_transform(&mut bytes);
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.extend_from_slice(&(-321_i16).to_le_bytes());
        append_int(&mut bytes, 0); // trees
        append_int(&mut bytes, 1); // stealth
        append_transform(&mut bytes);
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        append_int(&mut bytes, 987654);
        append_int(&mut bytes, 1); // things without lightmaps
        append_transform(&mut bytes);
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        append_int(&mut bytes, 0); // big-size objects
        append_int(&mut bytes, 1); // occlusion
        append_transform(&mut bytes);
        for value in [1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        append_int(&mut bytes, 1); // waikes
        append_transform(&mut bytes);
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.push(255);
        for value in [0.5_f32, 0.25, -0.5, 0.0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        append_int(&mut bytes, 256); // uvSize
        append_int(&mut bytes, 2); // ByteVec2 count
        bytes.extend_from_slice(&[0, 255, 128, 64]);
        append_int(&mut bytes, 0); // kuangjias
        append_int(&mut bytes, 0); // wujians
        let decoded = decode_scene_info(bytes).unwrap();
        let cell = &decoded.scene.value.cells[0].value;
        assert_eq!(cell.grass[0].value.brightness, -321);
        assert_eq!(cell.grass[0].source.byte_count, 40);
        assert_eq!(cell.stealth_grass[0].value.id, 987654);
        assert_eq!(cell.stealth_grass[0].source.byte_count, 42);
        assert_eq!(cell.things_no_lightmap[0].source.byte_count, 38);
        let group = &decoded.scene.value.occlusion_groups[0].value;
        assert_eq!(group.box_size, [1.0, 2.0, 3.0]);
        assert_eq!(group.box_center, [4.0, 5.0, 6.0]);
        let object = &group.waikes[0];
        assert_eq!(object.value.lightmap_index, 255);
        assert_eq!(object.value.lightmap_scale_offset, [0.5, 0.25, -0.5, 0.0]);
        assert_eq!(object.value.uv_size, 256);
        assert_eq!(object.value.uv2_data, [[0, 255], [128, 64]]);
        assert_eq!(object.source.byte_count, 67);
        assert_eq!(
            &decoded.bytes_for(&object.source).unwrap()[63..],
            &[0, 255, 128, 64]
        );
    }
}
