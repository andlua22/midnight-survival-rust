//! Bevy diagnostics of verified source rig poses. Marker meshes are diagnostic
//! geometry; serialized player colliders are not effective runtime posture.
//! Authoritative origins always come from shared::source_player_pose.

use std::collections::BTreeMap;

use bevy::{
    asset::RenderAssetUsages,
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::PrimitiveTopology,
    prelude::*,
};
use midnight_map_data::{
    ArchiveError, PlayerColliderShape, PlayerRigPackage, SerializedPlayerCollider, SourceObject,
    validate_prefab_affine,
};
use serde::Serialize;
use shared::source_player_pose::SourceRigPose;

fn invalid(message: impl Into<String>) -> ArchiveError {
    ArchiveError::Invalid(message.into())
}

#[derive(Component, Clone, Debug)]
pub struct SourcePlayerRigNode {
    pub source: SourceObject,
    pub transform: SourceObject,
    pub node_index: usize,
    pub saved_layer: i64,
    pub saved_active_self: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct SourcePlayerColliderDiagnostic {
    pub serialized: SerializedPlayerCollider,
    pub active_in_serialized_hierarchy: bool,
    pub wire_vertices: usize,
    pub unresolved: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SourcePlayerBindingDiagnostic {
    pub field: String,
    pub transform: Option<SourceObject>,
    pub sampled_bevy_position: Option<[f32; 3]>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SourcePlayerDiagnosticReport {
    pub manifest_sha256: String,
    pub root: SourceObject,
    pub root_name: String,
    pub nodes: usize,
    pub skinned_renderers: usize,
    pub saved_layers: BTreeMap<i64, usize>,
    pub bindings: Vec<SourcePlayerBindingDiagnostic>,
    pub colliders: Vec<SourcePlayerColliderDiagnostic>,
    pub bounds: [[f32; 3]; 2],
    pub unapplied_animator_channels: usize,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SourcePlayerPoseVerification {
    pub nodes: usize,
    pub maximum_matrix_error: f32,
    pub checked_query_origins: usize,
}

pub struct SourcePlayerDiagnostic {
    pub report: SourcePlayerDiagnosticReport,
    pub nodes: Vec<Option<Entity>>,
    links: Vec<(usize, Entity)>,
    query_markers: Vec<(Entity, usize)>,
}

fn local_transform(matrix: [f32; 16]) -> Result<Transform, ArchiveError> {
    validate_prefab_affine(&matrix)?;
    let matrix = Mat4::from_cols_array(&matrix);
    let transform = Transform::from_matrix(matrix);
    if !transform.to_matrix().abs_diff_eq(matrix, 0.0001) {
        return Err(invalid(
            "source rig local shear cannot be represented by Bevy Transform",
        ));
    }
    Ok(transform)
}

fn link_transform(endpoint: Vec3) -> (Transform, Visibility) {
    let length = endpoint.length();
    if length <= 0.00001 {
        return (Transform::default(), Visibility::Hidden);
    }
    (
        Transform::from_translation(endpoint * 0.5)
            .with_rotation(Quat::from_rotation_arc(Vec3::Y, endpoint / length))
            .with_scale(Vec3::new(0.006, length, 0.006)),
        Visibility::Inherited,
    )
}

fn line(vertices: &mut Vec<[f32; 3]>, a: Vec3, b: Vec3) {
    // Shape input is still Unity-local. The parent matrices are already Bevy-local.
    vertices.push([a.x, a.y, -a.z]);
    vertices.push([b.x, b.y, -b.z]);
}

fn circle(vertices: &mut Vec<[f32; 3]>, center: Vec3, a: Vec3, b: Vec3, radius: f32) {
    for step in 0..48 {
        let angle = step as f32 * std::f32::consts::TAU / 48.;
        let next = (step + 1) as f32 * std::f32::consts::TAU / 48.;
        line(
            vertices,
            center + radius * (a * angle.cos() + b * angle.sin()),
            center + radius * (a * next.cos() + b * next.sin()),
        );
    }
}

/// Affine visualization of serialized local geometry. No Unity lossy-scale or
/// PhysX capsule clamp is inferred. Unsupported geometry remains in the report.
fn collider_wire(shape: &PlayerColliderShape) -> Result<Vec<[f32; 3]>, String> {
    let mut vertices = Vec::new();
    match shape {
        PlayerColliderShape::Box { center, size } => {
            let center = Vec3::from_array(*center);
            let half = Vec3::from_array(*size) * 0.5;
            for axis in 0..3 {
                for a in [-1., 1.] {
                    for b in [-1., 1.] {
                        let mut start = center;
                        start[axis] -= half[axis];
                        start[(axis + 1) % 3] += a * half[(axis + 1) % 3];
                        start[(axis + 2) % 3] += b * half[(axis + 2) % 3];
                        let mut end = start;
                        end[axis] += 2. * half[axis];
                        line(&mut vertices, start, end);
                    }
                }
            }
        }
        PlayerColliderShape::Sphere { center, radius } => {
            let center = Vec3::from_array(*center);
            for (a, b) in [(Vec3::X, Vec3::Y), (Vec3::Y, Vec3::Z), (Vec3::Z, Vec3::X)] {
                circle(&mut vertices, center, a, b, *radius);
            }
        }
        PlayerColliderShape::Capsule {
            center,
            radius,
            height,
            direction,
        } => {
            if *direction > 2 {
                return Err("invalid serialized capsule axis".into());
            }
            if *height < 2. * radius {
                return Err(
                    "saved capsule height is below its diameter; runtime clamp unresolved".into(),
                );
            }
            let center = Vec3::from_array(*center);
            let axis = [Vec3::X, Vec3::Y, Vec3::Z][usize::from(*direction)];
            let a = [Vec3::Y, Vec3::Z, Vec3::X][usize::from(*direction)];
            let b = axis.cross(a);
            let half_segment = (height - 2. * radius) * 0.5;
            for sign in [-1., 1.] {
                let cap = center + axis * (sign * half_segment);
                circle(&mut vertices, cap, a, b, *radius);
                for radial in [a, b, -a, -b] {
                    for step in 0..12 {
                        let angle = step as f32 * std::f32::consts::FRAC_PI_2 / 12.;
                        let next = (step + 1) as f32 * std::f32::consts::FRAC_PI_2 / 12.;
                        line(
                            &mut vertices,
                            cap + *radius * (radial * angle.cos() + axis * (sign * angle.sin())),
                            cap + *radius * (radial * next.cos() + axis * (sign * next.sin())),
                        );
                    }
                }
            }
            for radial in [a, b, -a, -b] {
                line(
                    &mut vertices,
                    center + *radius * radial - half_segment * axis,
                    center + *radius * radial + half_segment * axis,
                );
            }
        }
        PlayerColliderShape::Mesh { .. } => {
            return Err(
                "source mesh-collider payload retained but not decoded for this diagnostic".into(),
            );
        }
        PlayerColliderShape::UnresolvedMesh { detail, .. } => {
            return Err(format!("source mesh-collider reference unresolved: {detail}"));
        }
    }
    Ok(vertices)
}

/// Spawn the authentic source hierarchy, with explicit diagnostic markers.
/// Every source node is shown, including dimmed inactive nodes. Marker geometry
/// never substitutes for a source SkinnedMeshRenderer or a runtime collider.
pub fn spawn_source_player_diagnostic(
    world: &mut World,
    package: &PlayerRigPackage,
    pose: &SourceRigPose,
) -> Result<SourcePlayerDiagnostic, ArchiveError> {
    let count = package.manifest.nodes.len();
    if pose.nodes.len() != count || pose.local_transforms.len() != count {
        return Err(invalid(
            "source pose does not match player package node count",
        ));
    }
    let root = package
        .manifest
        .roots
        .iter()
        .find(|root| root.game_object == pose.root)
        .ok_or_else(|| invalid("source pose root is not a package root"))?;
    let root_index = package
        .node_index(&pose.root)
        .ok_or_else(|| invalid("source root missing"))?;
    let mut local = Vec::with_capacity(count);
    let mut saved_layers = BTreeMap::new();
    let mut bounds = [Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)];
    let mut skin_count = 0;
    for (index, source) in package.manifest.nodes.iter().enumerate() {
        match (&pose.nodes[index], pose.local_transforms[index]) {
            (Some(node), Some(matrix)) => {
                local.push(Some(local_transform(matrix)?));
                validate_prefab_affine(&node.world_transform)?;
                let point = Mat4::from_cols_array(&node.world_transform)
                    .w_axis
                    .truncate();
                bounds[0] = bounds[0].min(point);
                bounds[1] = bounds[1].max(point);
                *saved_layers.entry(source.layer).or_insert(0) += 1;
                skin_count += source
                    .components
                    .iter()
                    .filter(|c| c.class_id == 137)
                    .count();
                if let Some(parent) = &source.parent {
                    let parent = package
                        .node_index(parent)
                        .ok_or_else(|| invalid("rig parent missing"))?;
                    if pose.nodes[parent].is_none() {
                        return Err(invalid("selected source rig has an unselected parent"));
                    }
                }
            }
            (None, None) => local.push(None),
            _ => return Err(invalid("source pose local/world selection disagrees")),
        }
    }
    if local[root_index].is_none() {
        return Err(invalid("source pose omits its root"));
    }
    let mut collider_reports = Vec::new();
    let mut wires = Vec::new();
    for collider in &package.manifest.colliders {
        let index = package
            .node_index(&collider.game_object)
            .ok_or_else(|| invalid("collider node missing"))?;
        let Some(node) = &pose.nodes[index] else {
            continue;
        };
        let (vertices, unresolved) = match collider_wire(&collider.shape) {
            Ok(vertices) => (vertices, None),
            Err(error) => (Vec::new(), Some(error)),
        };
        let matrix = Mat4::from_cols_array(&node.world_transform);
        for point in &vertices {
            let point = matrix.transform_point3(Vec3::from_array(*point));
            bounds[0] = bounds[0].min(point);
            bounds[1] = bounds[1].max(point);
        }
        collider_reports.push(SourcePlayerColliderDiagnostic {
            serialized: collider.clone(),
            active_in_serialized_hierarchy: node.active_in_hierarchy,
            wire_vertices: vertices.len(),
            unresolved,
        });
        if !vertices.is_empty() {
            wires.push((index, collider, vertices));
        }
    }
    let bindings = [
        ("RootBone", root.root_bone.as_ref(), pose.root_bone),
        ("HeadTop", root.head_top.as_ref(), pose.head_top),
    ];
    let mut bound_nodes = Vec::new();
    for (field, source, position) in bindings {
        if let Some(source) = source {
            let index = package
                .transform_index(source)
                .ok_or_else(|| invalid("bound rig Transform missing"))?;
            if local[index].is_none() || position.is_none() {
                return Err(invalid(format!(
                    "selected rig binding {field} is absent from shared pose"
                )));
            }
            bound_nodes.push((field, index));
        } else if position.is_some() {
            return Err(invalid("unbound query origin was supplied by source pose"));
        }
    }
    if !world.contains_resource::<Assets<Mesh>>()
        || !world.contains_resource::<Assets<StandardMaterial>>()
    {
        return Err(invalid(
            "player diagnostic requires Bevy mesh and material assets",
        ));
    }
    // All source/pose admission completes before Bevy asset/entity mutation.
    let joint_mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(Cuboid::new(0.015, 0.015, 0.015));
    let link_mesh = world.resource_mut::<Assets<Mesh>>().add(Cuboid::default());
    let origin_mesh = world.resource_mut::<Assets<Mesh>>().add(Sphere::new(0.045));
    let colors = [
        Color::srgb(0.18, 0.82, 0.88),
        Color::srgb(0.22, 0.32, 0.38),
        Color::srgb(1.0, 0.67, 0.13),
        Color::srgb(1.0, 0.25, 0.51),
        Color::srgb(0.42, 0.58, 0.98),
        Color::srgb(0.94, 0.38, 0.1),
    ];
    let materials = colors.map(|base_color| {
        world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color,
                unlit: true,
                cull_mode: None,
                ..default()
            })
    });
    let mut entities = vec![None; count];
    for (index, transform) in local.iter().enumerate() {
        let Some(transform) = transform else { continue };
        let node = &package.manifest.nodes[index];
        let entity = world
            .spawn((
                Name::new(node.name.clone()),
                *transform,
                Visibility::Inherited,
                SourcePlayerRigNode {
                    source: node.source.clone(),
                    transform: node.transform.clone(),
                    node_index: index,
                    saved_layer: node.layer,
                    saved_active_self: node.active_self,
                },
            ))
            .id();
        let material = if pose.nodes[index]
            .as_ref()
            .is_some_and(|n| n.active_in_hierarchy)
        {
            0
        } else {
            1
        };
        world.spawn((
            Name::new("Diagnostic joint marker"),
            Mesh3d(joint_mesh.clone()),
            MeshMaterial3d(materials[material].clone()),
            Transform::default(),
            ChildOf(entity),
            NotShadowCaster,
            NotShadowReceiver,
        ));
        entities[index] = Some(entity);
    }
    let mut links = Vec::new();
    for (index, entity) in entities.iter().enumerate() {
        let Some(entity) = entity else { continue };
        let source = &package.manifest.nodes[index];
        if let Some(parent) = &source.parent {
            let parent_index = package
                .node_index(parent)
                .ok_or_else(|| invalid("rig parent changed"))?;
            let parent = entities[parent_index].ok_or_else(|| invalid("rig parent not spawned"))?;
            world.entity_mut(*entity).insert(ChildOf(parent));
            let transform = local[index].ok_or_else(|| invalid("rig local changed"))?;
            let (transform, visibility) = link_transform(transform.translation);
            let link = world
                .spawn((
                    Name::new("Diagnostic hierarchy link"),
                    Mesh3d(link_mesh.clone()),
                    MeshMaterial3d(materials[0].clone()),
                    transform,
                    visibility,
                    ChildOf(parent),
                    NotShadowCaster,
                    NotShadowReceiver,
                ))
                .id();
            links.push((index, link));
        }
    }
    let mut query_markers = Vec::new();
    for (field, index) in bound_nodes {
        let material = if field == "RootBone" { 2 } else { 3 };
        let parent = entities[index].ok_or_else(|| invalid("bound rig entity missing"))?;
        let marker = world
            .spawn((
                Name::new(format!("Diagnostic {field} query origin")),
                Mesh3d(origin_mesh.clone()),
                MeshMaterial3d(materials[material].clone()),
                Transform::default(),
                ChildOf(parent),
                NotShadowCaster,
                NotShadowReceiver,
            ))
            .id();
        query_markers.push((marker, index));
    }
    for (index, collider, vertices) in wires {
        let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
        let normals = vec![[0., 1., 0.]; vertices.len()];
        mesh.try_insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .map_err(|e| invalid(e.to_string()))?;
        mesh.try_insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .map_err(|e| invalid(e.to_string()))?;
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let material = if collider.is_trigger {
            5
        } else if collider.enabled {
            4
        } else {
            1
        };
        world.spawn((
            Name::new(format!(
                "Serialized collider {} layer {}",
                collider.source.path_id, collider.layer
            )),
            Mesh3d(mesh),
            MeshMaterial3d(materials[material].clone()),
            Transform::default(),
            ChildOf(entities[index].ok_or_else(|| invalid("collider node not spawned"))?),
            NotShadowCaster,
            NotShadowReceiver,
        ));
    }
    Ok(SourcePlayerDiagnostic {
        report: SourcePlayerDiagnosticReport {
            manifest_sha256: package.manifest_sha256.clone(), root: pose.root.clone(),
            root_name: package.manifest.nodes[root_index].name.clone(), nodes: entities.iter().flatten().count(),
            skinned_renderers: skin_count, saved_layers,
            bindings: bindings.into_iter().map(|(field, transform, position)| SourcePlayerBindingDiagnostic {
                field: field.into(), transform: transform.cloned(), sampled_bevy_position: position,
            }).collect(), colliders: collider_reports, bounds: bounds.map(|point| point.to_array()),
            unapplied_animator_channels: pose.unapplied_animator_curves.len(),
            limitations: vec![
                "Diagnostic bones and query markers; no source body skin is fabricated.".into(),
                "All saved hierarchy nodes shown; saved activity and layers are metadata.".into(),
                "Collider wires show affine serialized local geometry, not effective runtime PhysX scaling or posture.".into(),
                "Root motion, Avatar retargeting, FSM blending and original runtime timing remain unverified.".into(),
            ],
        }, nodes: entities, links, query_markers,
    })
}

/// Copy only an explicitly evaluated shared pose to presentation. Bevy's
/// propagated transforms never feed the source clearance or movement queries.
pub fn apply_source_player_diagnostic_pose(
    world: &mut World,
    diagnostic: &mut SourcePlayerDiagnostic,
    pose: &SourceRigPose,
) -> Result<(), ArchiveError> {
    if pose.root != diagnostic.report.root
        || pose.local_transforms.len() != diagnostic.nodes.len()
        || pose.nodes.len() != diagnostic.nodes.len()
    {
        return Err(invalid("replacement source pose uses a different rig"));
    }
    let mut updates = Vec::new();
    for (index, entity) in diagnostic.nodes.iter().enumerate() {
        match (entity, pose.local_transforms[index], &pose.nodes[index]) {
            (Some(entity), Some(matrix), Some(_)) => {
                if world.get::<Transform>(*entity).is_none() {
                    return Err(invalid("diagnostic node was removed"));
                }
                updates.push((*entity, local_transform(matrix)?));
            }
            (None, None, None) => {}
            _ => return Err(invalid("replacement pose changes the selected hierarchy")),
        }
    }
    for (entity, transform) in updates {
        world.entity_mut(entity).insert(transform);
    }
    for (index, entity) in &diagnostic.links {
        let matrix = pose.local_transforms[*index].ok_or_else(|| invalid("link local absent"))?;
        let (transform, visibility) =
            link_transform(Mat4::from_cols_array(&matrix).w_axis.truncate());
        world.entity_mut(*entity).insert((transform, visibility));
    }
    for binding in &mut diagnostic.report.bindings {
        binding.sampled_bevy_position = if binding.field == "RootBone" {
            pose.root_bone
        } else {
            pose.head_top
        };
    }
    diagnostic.report.unapplied_animator_channels = pose.unapplied_animator_curves.len();
    Ok(())
}

/// Verify presentation against the independent shared evaluator after Bevy's
/// TransformSystems::Propagate. This function only reports, never authors pose.
pub fn verify_source_player_diagnostic_pose(
    world: &World,
    diagnostic: &SourcePlayerDiagnostic,
    pose: &SourceRigPose,
) -> Result<SourcePlayerPoseVerification, ArchiveError> {
    if pose.root != diagnostic.report.root || pose.nodes.len() != diagnostic.nodes.len() {
        return Err(invalid("verification source pose uses a different rig"));
    }
    let mut result = SourcePlayerPoseVerification {
        nodes: 0,
        maximum_matrix_error: 0.,
        checked_query_origins: 0,
    };
    for (index, entity) in diagnostic.nodes.iter().enumerate() {
        match (entity, &pose.nodes[index]) {
            (Some(entity), Some(expected)) => {
                let actual = world
                    .get::<GlobalTransform>(*entity)
                    .ok_or_else(|| invalid("rig GlobalTransform absent"))?
                    .to_matrix();
                for (actual, expected) in actual
                    .to_cols_array()
                    .into_iter()
                    .zip(expected.world_transform)
                {
                    let error = (actual - expected).abs();
                    result.maximum_matrix_error = result.maximum_matrix_error.max(error);
                    if !error.is_finite() || error > 0.0002 + expected.abs() * 0.000002 {
                        return Err(invalid(format!(
                            "render/shared player pose mismatch at source node {index}"
                        )));
                    }
                }
                result.nodes += 1;
            }
            (None, None) => {}
            _ => return Err(invalid("render/shared player hierarchy selection mismatch")),
        }
    }
    for (entity, index) in &diagnostic.query_markers {
        let expected = pose.nodes[*index]
            .as_ref()
            .ok_or_else(|| invalid("query source pose absent"))?;
        let expected = Mat4::from_cols_array(&expected.world_transform)
            .w_axis
            .truncate();
        let actual = world
            .get::<GlobalTransform>(*entity)
            .ok_or_else(|| invalid("query marker GlobalTransform absent"))?
            .translation();
        if !actual.abs_diff_eq(expected, 0.0002) {
            return Err(invalid("render/shared query-origin mismatch"));
        }
        result.checked_query_origins += 1;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_local_trs_preserves_negative_and_nonuniform_parents_through_bevy_propagation() {
        let parent_matrix = Mat4::from_scale_rotation_translation(
            Vec3::new(-2., 3., 0.5),
            Quat::from_rotation_y(0.61),
            Vec3::new(7., -2., 3.),
        );
        let child_matrix = Mat4::from_scale_rotation_translation(
            Vec3::new(0.7, -1.1, 1.8),
            Quat::from_rotation_z(0.37),
            Vec3::new(0.4, 1.7, -0.3),
        );
        let expected = parent_matrix * child_matrix;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin));
        let parent = app
            .world_mut()
            .spawn(local_transform(parent_matrix.to_cols_array()).unwrap())
            .id();
        let child = app
            .world_mut()
            .spawn((
                local_transform(child_matrix.to_cols_array()).unwrap(),
                ChildOf(parent),
            ))
            .id();
        app.update();
        let actual = app
            .world()
            .get::<GlobalTransform>(child)
            .unwrap()
            .to_matrix();
        assert!(actual.abs_diff_eq(expected, 0.00001));
        // Parent composition may shear; decomposing and re-authoring the world
        // transform would lose that information. Locals remain individually TRS.
        assert!(
            !Transform::from_matrix(expected)
                .to_matrix()
                .abs_diff_eq(expected, 0.001)
        );
    }

    #[test]
    fn serialized_collider_geometry_converts_local_z_once_and_refuses_an_unproved_clamp() {
        let wire = collider_wire(&PlayerColliderShape::Box {
            center: [2., 3., 7.],
            size: [2., 4., 6.],
        })
        .unwrap();
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for point in wire {
            min = min.min(Vec3::from_array(point));
            max = max.max(Vec3::from_array(point));
        }
        assert_eq!(min, Vec3::new(1., 1., -10.));
        assert_eq!(max, Vec3::new(3., 5., -4.));
        assert!(
            collider_wire(&PlayerColliderShape::Capsule {
                center: [0.; 3],
                radius: 1.,
                height: 1.,
                direction: 1
            })
            .is_err()
        );
    }
}
