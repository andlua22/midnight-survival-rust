//! Unity packed vectors used by compressed road meshes in the supplied APK.
//! The installed mesh processor exposes these records but does not unpack them.

use glam::{Mat4, Vec3};
use midnight_map_data::MeshRecord;
use unity_asset_binary::object::UnityObject;
use unity_asset_core::UnityValue;

use crate::{fields::{array, bytes, field, integer, invalid, number, property, text}, mesh, unity::Error};

fn unpack_bits(data: &[u8], count: usize, width: usize) -> Result<Vec<u32>, Error> {
    if width > 32 { return Err(invalid("packed mesh bit width exceeds 32")); }
    let bits = count.checked_mul(width).ok_or_else(|| invalid("packed mesh bit count overflow"))?;
    if bits.div_ceil(8) > data.len() { return Err(invalid("truncated packed mesh vector")); }
    let mut output = Vec::with_capacity(count);
    for index in 0..count {
        let mut value = 0u32;
        for bit in 0..width {
            let position = index * width + bit;
            value |= u32::from((data[position / 8] >> (position % 8)) & 1) << bit;
        }
        output.push(value);
    }
    Ok(output)
}

fn packed_integers(vector: &UnityValue) -> Result<Vec<u32>, Error> {
    let count = usize::try_from(integer(field(vector, "m_NumItems")?)?)?;
    let width = usize::try_from(integer(field(vector, "m_BitSize")?)?)?;
    if count > 16_000_000 { return Err(invalid("packed mesh vector exceeds item limit")); }
    unpack_bits(&bytes(field(vector, "m_Data")?)?, count, width)
}

fn packed_floats(vector: &UnityValue) -> Result<Vec<f32>, Error> {
    let integers = packed_integers(vector)?;
    let width = integer(field(vector, "m_BitSize")?)?;
    let start = number(field(vector, "m_Start")?)?;
    let range = number(field(vector, "m_Range")?)?;
    let denominator = ((1u64 << width) - 1) as f32;
    Ok(integers.into_iter().map(|value| {
        if width == 0 { start } else { start + value as f32 * range / denominator }
    }).collect())
}

pub fn decode(object: &UnityObject, world: Mat4) -> Result<Vec<MeshRecord>, Error> {
    let name = text(property(object, "m_Name")?)?;
    let compressed = property(object, "m_CompressedMesh")?;
    let positions = packed_floats(field(compressed, "m_Vertices")?)?;
    if positions.is_empty() || !positions.len().is_multiple_of(3) {
        return Err(invalid(format!("{name}: missing packed vertex triples")));
    }
    let positions: Vec<_> = positions.as_chunks::<3>().0.to_vec();
    let count = positions.len();
    let xy = packed_floats(field(compressed, "m_Normals")?)?;
    let signs = packed_integers(field(compressed, "m_NormalSigns")?)?;
    if xy.len() != count * 2 || signs.len() != count {
        return Err(invalid(format!("{name}: packed normal counts differ")));
    }
    let normals = xy.as_chunks::<2>().0.iter().zip(signs).map(|(&[x, y], sign)| {
        let z = (1.0 - x*x - y*y).max(0.0).sqrt() * if sign == 0 { -1.0 } else { 1.0 };
        Vec3::new(x, y, z).normalize_or_zero().to_array()
    }).collect();
    let tangent_xy = packed_floats(field(compressed, "m_Tangents")?)?;
    let tangent_signs = packed_integers(field(compressed, "m_TangentSigns")?)?;
    let tangents = if tangent_xy.is_empty() && tangent_signs.is_empty() { Vec::new() } else {
        if tangent_xy.len() != count * 2 || tangent_signs.len() != count * 2 {
            return Err(invalid(format!("{name}: packed tangent counts differ")));
        }
        tangent_xy.as_chunks::<2>().0.iter().zip(tangent_signs.as_chunks::<2>().0).map(|(&[x, y], &[z_sign, w_sign])| {
            let z = (1.0 - x*x - y*y).max(0.0).sqrt() * if z_sign == 0 { -1.0 } else { 1.0 };
            let tangent = Vec3::new(x, y, z).normalize_or_zero();
            [tangent.x, tangent.y, tangent.z, if w_sign == 0 { -1.0 } else { 1.0 }]
        }).collect()
    };
    let uv = packed_floats(field(compressed, "m_UV")?)?;
    let uv_info = integer(field(compressed, "m_UVInfo")?)?;
    let dimensions = if uv_info == 0 { 2 } else { (uv_info & 3) as usize + 1 };
    if (uv_info != 0 && uv_info & 4 == 0) || dimensions < 2 || uv.len() < count * dimensions {
        return Err(invalid(format!("{name}: missing packed UV0")));
    }
    let uvs = uv[..count * dimensions].chunks_exact(dimensions).map(|uv| [uv[0], uv[1]]).collect();
    let indices = packed_integers(field(compressed, "m_Triangles")?)?;
    let index_size = match integer(property(object, "m_IndexFormat")?)? {
        0 => 2, 1 => 4, _ => return Err(invalid("unsupported packed mesh index format")),
    };
    let mut parts = Vec::new();
    for part in array(property(object, "m_SubMeshes")?)? {
        if integer(field(part, "topology")?)? != 0 { return Err(invalid("non-triangle packed mesh")); }
        let first_byte = usize::try_from(integer(field(part, "firstByte")?)?)?;
        if !first_byte.is_multiple_of(index_size) { return Err(invalid("misaligned packed mesh index range")); }
        let offset = first_byte / index_size;
        let len = usize::try_from(integer(field(part, "indexCount")?)?)?;
        let end = offset.checked_add(len).ok_or_else(|| invalid("packed mesh index range overflow"))?;
        let part_indices = indices.get(offset..end).ok_or_else(|| invalid("truncated packed mesh indices"))?;
        let base = u32::try_from(integer(field(part, "baseVertex")?)?)?;
        parts.push(part_indices.iter().map(|value| {
            value.checked_add(base).ok_or_else(|| invalid("packed mesh base vertex overflow"))
        }).collect::<Result<Vec<_>, _>>()?);
    }
    mesh::finish(name, world, mesh::SourceVertices { positions, normals, tangents, uvs }, parts)
}

#[cfg(test)]
mod tests {
    use super::unpack_bits;

    #[test]
    fn unpacking_crosses_byte_boundaries_and_rejects_truncation() {
        assert_eq!(unpack_bits(&[0x25, 0x0e], 3, 5).unwrap(), vec![5, 17, 3]);
        assert_eq!(unpack_bits(&[0x78, 0x56, 0x34, 0x12], 1, 32).unwrap(), vec![0x12345678]);
        assert_eq!(unpack_bits(&[], 3, 0).unwrap(), vec![0; 3]);
        assert!(unpack_bits(&[0x25], 3, 5).is_err());
        assert!(unpack_bits(&[], 1, 33).is_err());
    }
}
