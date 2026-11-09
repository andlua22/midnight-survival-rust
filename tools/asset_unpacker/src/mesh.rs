//! Decode real Unity vertex streams; the dependency's MeshProcessor extraction methods are stubs.

use std::collections::BTreeMap;

use glam::{Mat4, Vec3};
use midnight_map_data::MeshRecord;
use unity_asset_binary::object::UnityObject;
use unity_asset_core::UnityValue;

use crate::{fields::{array, bytes, field, integer, invalid, property, text}, unity::Error};

pub use midnight_map_data::mesh_bounds as bounds;

#[derive(Debug)]
struct Channel { stream: usize, offset: usize, format: i64, dimension: usize }

pub(crate) struct SourceVertices {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
}

fn format_size(format: i64) -> Result<usize, Error> {
    match format {
        0 | 10 | 11 => Ok(4),
        1 | 4 | 5 | 8 | 9 => Ok(2),
        2 | 3 | 6 | 7 => Ok(1),
        _ => Err(invalid(format!("unsupported Unity vertex format {format}"))),
    }
}

fn scalar(data: &[u8], offset: usize, format: i64) -> Result<f32, Error> {
    let size = format_size(format)?;
    let raw = data.get(offset..offset + size).ok_or_else(|| invalid("truncated vertex stream"))?;
    let number = match format {
        0 => f32::from_le_bytes(raw.try_into()?),
        1 => half::f16::from_bits(u16::from_le_bytes(raw.try_into()?)).to_f32(),
        2 => f32::from(raw[0]) / 255.0,
        3 => (f32::from(raw[0] as i8) / 127.0).max(-1.0),
        4 => f32::from(u16::from_le_bytes(raw.try_into()?)) / 65535.0,
        5 => (f32::from(i16::from_le_bytes(raw.try_into()?)) / 32767.0).max(-1.0),
        6 => f32::from(raw[0]),
        7 => f32::from(raw[0] as i8),
        8 => f32::from(u16::from_le_bytes(raw.try_into()?)),
        9 => f32::from(i16::from_le_bytes(raw.try_into()?)),
        10 => u32::from_le_bytes(raw.try_into()?) as f32,
        11 => i32::from_le_bytes(raw.try_into()?) as f32,
        _ => unreachable!(),
    };
    if !number.is_finite() { return Err(invalid("non-finite mesh attribute")); }
    Ok(number)
}

fn unsigned(value: &UnityValue) -> Result<usize, Error> {
    usize::try_from(integer(value)?).map_err(|_| invalid("negative mesh count/offset"))
}

pub fn decode(object: &UnityObject, world: Mat4) -> Result<Vec<MeshRecord>, Error> {
    let name = text(property(object, "m_Name")?)?;
    let vertex_data = property(object, "m_VertexData")?;
    let count = unsigned(field(vertex_data, "m_VertexCount")?)?;
    if count == 0 { return crate::packed_mesh::decode(object, world); }
    let channels = array(field(vertex_data, "m_Channels")?)?.iter().map(|value| Ok(Channel {
        stream: unsigned(field(value, "stream")?)?, offset: unsigned(field(value, "offset")?)?,
        format: integer(field(value, "format")?)?, dimension: unsigned(field(value, "dimension")?)? & 15,
    })).collect::<Result<Vec<_>, Error>>()?;
    let data = bytes(field(vertex_data, "m_DataSize")?)?;
    let mut strides = BTreeMap::<usize, usize>::new();
    for channel in &channels {
        if channel.dimension > 0 {
            let end = channel.offset + channel.dimension * format_size(channel.format)?;
            strides.entry(channel.stream).and_modify(|stride| *stride = (*stride).max(end)).or_insert(end);
        }
    }
    let mut starts = BTreeMap::new();
    let mut start = 0usize;
    for (&stream, &stride) in &strides {
        starts.insert(stream, start);
        start = start.checked_add(count.checked_mul(stride).ok_or_else(|| invalid("vertex count overflow"))?)
            .ok_or_else(|| invalid("vertex stream overflow"))?.next_multiple_of(16);
    }
    let attribute = |index: usize, vertex: usize, components: usize| -> Result<Vec<f32>, Error> {
        let channel = channels.get(index).ok_or_else(|| invalid(format!("{name}: missing vertex channel {index}")))?;
        if channel.dimension < components { return Err(invalid(format!("{name}: incomplete vertex channel {index}"))); }
        let offset = starts[&channel.stream] + strides[&channel.stream] * vertex + channel.offset;
        (0..components).map(|component| scalar(&data, offset + component * format_size(channel.format)?, channel.format)).collect()
    };
    let mut positions = Vec::with_capacity(count);
    let mut normals = Vec::with_capacity(count);
    let tangent_dimension = channels.get(2).map_or(0, |channel| channel.dimension);
    if tangent_dimension != 0 && tangent_dimension != 4 { return Err(invalid(format!("{name}: incomplete tangent channel"))); }
    let mut tangents = Vec::new();
    let mut uvs = Vec::with_capacity(count);
    for vertex in 0..count {
        let source = attribute(0, vertex, 3)?;
        positions.push([source[0], source[1], source[2]]);
        let source = attribute(1, vertex, 3)?;
        normals.push([source[0], source[1], source[2]]);
        if tangent_dimension == 4 {
            let tangent = attribute(2, vertex, 4)?;
            tangents.push([tangent[0], tangent[1], tangent[2], tangent[3]]);
        }
        let uv = attribute(4, vertex, 2)?;
        uvs.push([uv[0], uv[1]]);
    }
    let index_format = integer(property(object, "m_IndexFormat")?)?;
    let index_size = match index_format { 0 => 2, 1 => 4, _ => return Err(invalid("unsupported index format")) };
    let index_buffer = bytes(property(object, "m_IndexBuffer")?)?;
    let mut parts = Vec::new();
    for sub_mesh in array(property(object, "m_SubMeshes")?)? {
        if integer(field(sub_mesh, "topology")?)? != 0 { return Err(invalid(format!("{name}: non-triangle mesh"))); }
        let offset = unsigned(field(sub_mesh, "firstByte")?)?;
        let count = unsigned(field(sub_mesh, "indexCount")?)?;
        let base = u32::try_from(integer(field(sub_mesh, "baseVertex")?)?)?;
        let len = count.checked_mul(index_size).ok_or_else(|| invalid("index count overflow"))?;
        let data = index_buffer.get(offset..offset + len).ok_or_else(|| invalid("truncated index buffer"))?;
        let indices = data.chunks_exact(index_size).map(|bytes| {
            let value = if index_size == 2 { u32::from(u16::from_le_bytes(bytes.try_into().expect("two-byte chunk"))) }
                else { u32::from_le_bytes(bytes.try_into().expect("four-byte chunk")) };
            value.checked_add(base).ok_or_else(|| invalid("index/baseVertex overflow"))
        }).collect::<Result<Vec<_>, _>>()?;
        parts.push(indices);
    }
    finish(name, world, SourceVertices { positions, normals, tangents, uvs }, parts)
}

pub(crate) fn finish(name: &str, world: Mat4, source: SourceVertices, parts: Vec<Vec<u32>>) -> Result<Vec<MeshRecord>, Error> {
    let SourceVertices { positions, normals, tangents, uvs } = source;
    if !world.is_finite() || world.determinant().abs() < 1e-8 { return Err(invalid(format!("{name}: invalid source transform"))); }
    if positions.len() != normals.len() || positions.len() != uvs.len()
        || (!tangents.is_empty() && tangents.len() != positions.len()) {
        return Err(invalid(format!("{name}: source vertex attribute counts differ")));
    }
    let normal_transform = world.inverse().transpose();
    let positions: Vec<_> = positions.into_iter().map(|position| world.transform_point3(Vec3::from_array(position)).to_array()).collect();
    let normals = normals.into_iter().map(|normal| {
        normal_transform.transform_vector3(Vec3::from_array(normal)).try_normalize()
            .map(|normal| normal.to_array()).ok_or_else(|| invalid(format!("{name}: zero source normal")))
    }).collect::<Result<Vec<_>, _>>()?;
    // The world reflection changes tangent handedness; flipping UV0.y changes
    // it once more. Normal texture Y is correspondingly inverted in WESL.
    let tangent_sign = -world.determinant().signum();
    let tangents = tangents.into_iter().map(|[x, y, z, w]| {
        let tangent = world.transform_vector3(Vec3::new(x, y, z)).normalize_or_zero();
        [tangent.x, tangent.y, tangent.z, w * tangent_sign]
    }).collect::<Vec<_>>();
    let uvs = uvs.into_iter().map(|[u, v]| [u, 1.0 - v]).collect::<Vec<_>>();
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for position in &positions { min = min.min(Vec3::from_array(*position)); max = max.max(Vec3::from_array(*position)); }
    let mut records = Vec::new();
    for mut indices in parts {
        if world.determinant() < 0.0 {
            for triangle in indices.as_chunks_mut::<3>().0 { triangle.swap(1, 2); }
        }
        let record = MeshRecord { name: name.to_string(), texture: String::new(), center: ((min + max) * 0.5).to_array(),
            positions: positions.clone(), normals: normals.clone(), tangents: tangents.clone(), uvs: uvs.clone(), indices };
        midnight_map_data::validate_mesh(&record)?;
        records.push(record);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Quat;

    #[test]
    fn scene_rotation_world_mirror_and_uv_conversion_are_applied_once() {
        let world = Mat4::from_translation(Vec3::new(10.0, 20.0, 30.0))
            * Mat4::from_scale(Vec3::new(1.0, 1.0, -1.0))
            * Mat4::from_quat(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
        let records = finish("asymmetric source", world, SourceVertices {
            positions: vec![[1.0, 2.0, 3.0], [2.0, 2.0, 3.0], [1.0, 3.0, 3.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3], tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.2, 0.1], [0.7, 0.1], [0.2, 0.9]] },
            vec![vec![0, 1, 2]]).unwrap();
        let record = &records[0];
        assert!(Vec3::from_array(record.positions[0]).abs_diff_eq(Vec3::new(11.0, 23.0, 32.0), 1e-5));
        assert!(Vec3::from_array(record.normals[0]).abs_diff_eq(Vec3::Y, 1e-5));
        assert_eq!(record.indices, vec![0, 2, 1]);
        assert_eq!(record.uvs[0], [0.2, 0.9]);
        assert_eq!(record.tangents[0], [1.0, 0.0, 0.0, 1.0]);
        let converted_bitangent = Vec3::from_array(record.normals[0]).cross(Vec3::X) * record.tangents[0][3];
        let source_bitangent = world.transform_vector3(Vec3::Y).normalize();
        assert!((-converted_bitangent).abs_diff_eq(source_bitangent, 1e-5));
        let [a, b, c] = record.indices.as_chunks::<3>().0[0].map(|index| Vec3::from_array(record.positions[index as usize]));
        assert!((b-a).cross(c-a).dot(Vec3::from_array(record.normals[0])) > 0.99);
    }
}
