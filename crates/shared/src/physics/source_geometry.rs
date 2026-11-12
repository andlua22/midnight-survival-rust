//! Shared source collider conversions; callers retain admission and ownership.

use super::*;
use midnight_map_data::{ArchiveError, prefab_transform_point, validate_prefab_affine};

fn invalid(message: impl Into<String>) -> ArchiveError {
    ArchiveError::Invalid(message.into())
}

/// Saved Box fields remain Unity-local; the supplied matrix maps Bevy-local
/// points into Bevy world space. Baking corners preserves the complete affine
/// hierarchy, including rotation, nonuniform scale and shear.
pub(super) fn source_box_builder(
    mut center: [f32; 3],
    size: [f32; 3],
    world_transform: &[f32; 16],
) -> Result<ColliderBuilder, ArchiveError> {
    validate_prefab_affine(world_transform)?;
    if center.iter().any(|v| !v.is_finite()) || size.iter().any(|v| !v.is_finite() || *v <= 0.) {
        return Err(invalid(
            "source box has non-finite fields or nonpositive size",
        ));
    }
    center[2] = -center[2];
    let mut points = Vec::with_capacity(8);
    for x in [-0.5, 0.5] {
        for y in [-0.5, 0.5] {
            for z in [-0.5, 0.5] {
                let point = [
                    center[0] + x * size[0],
                    center[1] + y * size[1],
                    center[2] + z * size[2],
                ];
                points.push(Vec3::from_array(prefab_transform_point(
                    world_transform,
                    point,
                )?));
            }
        }
    }
    ColliderBuilder::convex_hull(&points).ok_or_else(|| invalid("source box convex hull failed"))
}

/// Mesh points are already Bevy-local. A mirrored hierarchy reverses winding
/// once more after the importer's Unity-local reflection.
pub(super) fn source_trimesh_builder(
    positions: &[[f32; 3]],
    indices: &[u32],
    world_transform: &[f32; 16],
) -> Result<ColliderBuilder, ArchiveError> {
    validate_prefab_affine(world_transform)?;
    if indices.is_empty()
        || !indices.len().is_multiple_of(3)
        || indices.iter().any(|&i| i as usize >= positions.len())
    {
        return Err(invalid("invalid source collider triangle indices"));
    }
    let vertices = positions
        .iter()
        .map(|&p| prefab_transform_point(world_transform, p).map(Vec3::from_array))
        .collect::<Result<Vec<_>, _>>()?;
    let mut triangles = indices.as_chunks::<3>().0.to_vec();
    let m = world_transform.map(f64::from);
    let determinant = m[0] * (m[5] * m[10] - m[9] * m[6]) - m[4] * (m[1] * m[10] - m[9] * m[2])
        + m[8] * (m[1] * m[6] - m[5] * m[2]);
    if determinant < 0. {
        for triangle in &mut triangles {
            triangle.swap(1, 2);
        }
    }
    ColliderBuilder::trimesh(vertices, triangles)
        .map_err(|error| invalid(format!("source triangle mesh: {error}")))
}
