//! Deterministic source rig diagnostics, independent of rendering and player movement.
//!
//! Generic curves are sampled at an explicit source time. Animator motion channels,
//! state-machine blending, Avatar retargeting and animation events remain unapplied.
//! This evaluator is not yet an admitted original-runtime locomotion controller.

use std::collections::{BTreeMap, BTreeSet};

use bevy::math::{Mat4, Quat, Vec3};
use midnight_map_data::{
    PlayerRigPackage, PrefabNode, SourceObject,
    player_animation::{
        SourceAnimationBinding, SourceAnimationClip, SourceMotionTransform,
        validate_source_animation_clip,
    },
    prefab_affine_product, prefab_transform_point, validate_prefab_affine,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRigPoseError(pub String);

impl std::fmt::Display for SourceRigPoseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for SourceRigPoseError {}

impl From<midnight_map_data::ArchiveError> for SourceRigPoseError {
    fn from(error: midnight_map_data::ArchiveError) -> Self {
        Self(error.to_string())
    }
}

fn invalid(message: impl Into<String>) -> SourceRigPoseError {
    SourceRigPoseError(message.into())
}

#[derive(Debug, Clone, Copy)]
pub struct SourceRigClipSample<'a> {
    pub animator: &'a SourceObject,
    pub clip: &'a SourceObject,
    pub time_seconds: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRigNodePose {
    pub world_transform: [f32; 16],
    pub active_in_hierarchy: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceUnappliedAnimatorCurve {
    pub binding: SourceAnimationBinding,
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRigPose {
    pub root: SourceObject,
    /// Aligned with manifest.nodes. The selected root's local is caller placement.
    pub local_transforms: Vec<Option<[f32; 16]>>,
    /// Aligned with manifest.nodes. Unselected roots/descendants are absent.
    pub nodes: Vec<Option<SourceRigNodePose>>,
    pub root_bone: Option<[f32; 3]>,
    pub head_top: Option<[f32; 3]>,
    /// Retained for diagnosis; no displacement is applied from these values.
    pub unapplied_animator_curves: Vec<SourceUnappliedAnimatorCurve>,
}

/// Sample scalar storage without deciding Transform/root-motion/FSM semantics.
/// Dense samples interpolate linearly; streamed segments use their source cubic.
/// Exact knots, constant data and channel order are retained. Unity interpolation
/// and frame/fixed-update equivalence still require an original-runtime witness.
pub fn sample_source_clip_scalars(
    clip: &SourceAnimationClip,
    time_seconds: f32,
) -> Result<Vec<f32>, SourceRigPoseError> {
    validate_source_animation_clip(clip)?;
    if !time_seconds.is_finite()
        || time_seconds < clip.motion.start_time
        || time_seconds > clip.motion.stop_time
    {
        return Err(invalid(
            "source clip sample time is outside its retained interval",
        ));
    }
    let mut streamed = vec![None; clip.streamed.curve_count as usize];
    for frame in &clip.streamed.frames {
        let time = f32::from_bits(frame.time_bits);
        if time > time_seconds {
            break;
        }
        for key in &frame.keys {
            streamed[key.curve_index as usize] =
                Some((time, key.coefficient_bits.map(f32::from_bits)));
        }
    }
    let mut result =
        Vec::with_capacity(streamed.len() + clip.dense.curve_count as usize + clip.constant.len());
    for curve in streamed {
        let (time, coefficients) =
            curve.ok_or_else(|| invalid("source streamed channel has no sample"))?;
        let [a, b, c, d] = coefficients;
        let value = if a == 0.0 && b == 0.0 && c == 0.0 {
            d
        } else {
            if coefficients.iter().any(|value| !value.is_finite()) || !time.is_finite() {
                return Err(invalid(
                    "unresolved non-finite source streamed interpolation",
                ));
            }
            let delta = time_seconds - time;
            ((a * delta + b) * delta + c) * delta + d
        };
        if !value.is_finite() {
            return Err(invalid("source streamed interpolation overflow"));
        }
        result.push(value);
    }
    let dense = &clip.dense;
    if dense.curve_count > 0 {
        let frame = (time_seconds - dense.begin_time) * dense.sample_rate;
        if !frame.is_finite() || frame < 0.0 || frame > (dense.frame_count - 1) as f32 + 1e-4 {
            return Err(invalid("source time is outside dense sample coverage"));
        }
        let frame = frame.min((dense.frame_count - 1) as f32);
        let first = frame.floor() as usize;
        let second = (first + 1).min(dense.frame_count as usize - 1);
        let fraction = frame - first as f32;
        let count = dense.curve_count as usize;
        for index in 0..count {
            let a = dense.samples[first * count + index];
            let b = dense.samples[second * count + index];
            let value = if fraction == 0.0 {
                a
            } else {
                a + (b - a) * fraction
            };
            if !value.is_finite() {
                return Err(invalid("source dense interpolation overflow"));
            }
            result.push(value);
        }
    }
    result.extend_from_slice(&clip.constant);
    Ok(result)
}

fn source_local(node: &PrefabNode) -> Result<SourceMotionTransform, SourceRigPoseError> {
    let component = node
        .components
        .iter()
        .find(|component| component.source == node.transform && component.class_id == 4)
        .ok_or_else(|| invalid("sampled source node has no retained Transform component"))?;
    let rotation = component
        .fields
        .get("m_LocalRotation")
        .ok_or_else(|| invalid("missing source local rotation"))?;
    let mut rotation_xyzw = [0.0; 4];
    for (index, key) in ["x", "y", "z", "w"].iter().enumerate() {
        rotation_xyzw[index] = rotation
            .get(*key)
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| invalid("missing source local quaternion component"))?
            as f32;
    }
    Ok(SourceMotionTransform {
        translation: component.vector3("m_LocalPosition")?,
        rotation_xyzw,
        scale: component.vector3("m_LocalScale")?,
    })
}

fn bevy_local(value: &SourceMotionTransform) -> Result<[f32; 16], SourceRigPoseError> {
    if value
        .translation
        .iter()
        .chain(&value.rotation_xyzw)
        .chain(&value.scale)
        .any(|v| !v.is_finite())
    {
        return Err(invalid("non-finite sampled source Transform"));
    }
    let rotation = Quat::from_array(value.rotation_xyzw);
    if rotation.length_squared() < 1e-8 {
        return Err(invalid("zero sampled source quaternion"));
    }
    let reflection = Mat4::from_scale(Vec3::new(1.0, 1.0, -1.0));
    let matrix = (reflection
        * Mat4::from_scale_rotation_translation(
            Vec3::from_array(value.scale),
            rotation.normalize(),
            Vec3::from_array(value.translation),
        )
        * reflection)
        .to_cols_array();
    validate_prefab_affine(&matrix)?;
    Ok(matrix)
}

fn node_at_path(
    nodes: &[PrefabNode],
    indices: &BTreeMap<&SourceObject, usize>,
    root: usize,
    path: &str,
) -> Result<usize, SourceRigPoseError> {
    if path.is_empty() {
        return Ok(root);
    }
    let mut current = root;
    for name in path.split('/') {
        if name.is_empty() {
            return Err(invalid("empty source Avatar path segment"));
        }
        let mut matches = nodes[current]
            .children
            .iter()
            .map(|child| {
                indices
                    .get(child)
                    .copied()
                    .ok_or_else(|| invalid("source Avatar path has a missing child"))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|index| nodes[*index].name == name);
        current = matches
            .next()
            .ok_or_else(|| invalid(format!("source Avatar path is absent: {path}")))?;
        if matches.next().is_some() {
            return Err(invalid(format!("source Avatar path is ambiguous: {path}")));
        }
    }
    Ok(current)
}

/// Resolve a requested controller clip or its verified map-driven override.
/// Ownership remains the selected Animator's serialized clip slots.
pub fn source_animator_clip<'a>(
    package: &'a PlayerRigPackage,
    animator_source: &SourceObject,
    requested: &SourceObject,
) -> Result<&'a SourceAnimationClip, SourceRigPoseError> {
    let animator = package
        .manifest
        .animators
        .iter()
        .find(|animator| &animator.source == animator_source)
        .ok_or_else(|| invalid("unknown source Animator for clip resolution"))?;
    let owns = |id: &SourceObject| animator.clips.iter().flatten().any(|clip| clip == id);
    let resolved = if owns(requested) {
        if package.manifest.empty_override_clips.contains(requested) {
            &package
                .manifest
                .clip_overrides
                .iter()
                .find(|entry| &entry.placeholder == requested)
                .ok_or_else(|| invalid("source clip placeholder has no verified override"))?
                .clip
        } else {
            requested
        }
    } else if package
        .manifest
        .clip_overrides
        .iter()
        .any(|entry| &entry.clip == requested && owns(&entry.placeholder))
    {
        requested
    } else {
        return Err(invalid("source clip is not bound to selected Animator"));
    };
    package
        .clips
        .get(resolved)
        .ok_or_else(|| invalid("source clip is not decoded"))
}

/// Evaluate one complete selected root using a supplied placement. No package,
/// source player state, collision world or presentation state is mutated.
pub fn evaluate_source_rig_pose(
    package: &PlayerRigPackage,
    root: &SourceObject,
    placement: [f32; 16],
    sample: Option<SourceRigClipSample<'_>>,
) -> Result<SourceRigPose, SourceRigPoseError> {
    validate_prefab_affine(&placement)?;
    let root_binding = package
        .manifest
        .roots
        .iter()
        .find(|binding| &binding.game_object == root)
        .ok_or_else(|| invalid("unknown selected source player root"))?;
    let nodes = &package.manifest.nodes;
    let indices = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (&node.source, index))
        .collect::<BTreeMap<_, _>>();
    if indices.len() != nodes.len() {
        return Err(invalid("duplicate source player nodes"));
    }
    let root_index = *indices
        .get(root)
        .ok_or_else(|| invalid("selected source player root has no node"))?;
    let mut order = Vec::new();
    let mut selected = BTreeSet::new();
    let mut pending = vec![root_index];
    while let Some(index) = pending.pop() {
        if !selected.insert(index) {
            return Err(invalid("cyclic/duplicate source player child"));
        }
        order.push(index);
        for child in nodes[index].children.iter().rev() {
            let child = *indices
                .get(child)
                .ok_or_else(|| invalid("missing source player child"))?;
            if nodes[child].parent.as_ref() != Some(&nodes[index].source) {
                return Err(invalid("source player parent/child references disagree"));
            }
            pending.push(child);
        }
    }
    let mut local_transforms = vec![None; nodes.len()];
    for &index in &order {
        local_transforms[index] = Some(nodes[index].bevy_local_transform);
    }
    local_transforms[root_index] = Some(placement);
    let mut unapplied_animator_curves = Vec::new();
    if let Some(sample) = sample {
        let animator = package
            .manifest
            .animators
            .iter()
            .find(|animator| &animator.source == sample.animator)
            .ok_or_else(|| invalid("unknown source Animator for clip sample"))?;
        let animator_index = *indices
            .get(&animator.game_object)
            .ok_or_else(|| invalid("source Animator has no node"))?;
        if !selected.contains(&animator_index) {
            return Err(invalid(
                "source clip is not bound to the selected root/Animator",
            ));
        }
        let avatar = package
            .avatars
            .get(&animator.avatar)
            .ok_or_else(|| invalid("source Animator Avatar is not decoded"))?;
        if avatar.human_bone_count != 0 || avatar.has_translation_dof {
            return Err(invalid("source humanoid Avatar retargeting is unresolved"));
        }
        let clip = source_animator_clip(package, sample.animator, sample.clip)?;
        let values = sample_source_clip_scalars(clip, sample.time_seconds)?;
        let mut offset = 0usize;
        let mut poses = BTreeMap::<usize, SourceMotionTransform>::new();
        for binding in &clip.bindings {
            let dimension = binding
                .scalar_dimension()
                .ok_or_else(|| invalid("unsupported source animation channel"))?;
            let end = offset
                .checked_add(dimension)
                .filter(|end| *end <= values.len())
                .ok_or_else(|| invalid("source animation channel range overflow"))?;
            let values = &values[offset..end];
            offset = end;
            if binding.transform_dimension().is_none() {
                unapplied_animator_curves.push(SourceUnappliedAnimatorCurve {
                    binding: binding.clone(),
                    value: values[0],
                });
                continue;
            }
            if binding.attribute == 4 {
                return Err(invalid("source Euler-curve rotation order is unresolved"));
            }
            let path = avatar.path(binding.path_hash).ok_or_else(|| {
                invalid(format!(
                    "source animation path hash {} is absent from Avatar",
                    binding.path_hash
                ))
            })?;
            let index = node_at_path(nodes, &indices, animator_index, path)?;
            if index == root_index {
                return Err(invalid(
                    "animation of the selected placement root requires resolved root motion",
                ));
            }
            if let std::collections::btree_map::Entry::Vacant(entry) = poses.entry(index) {
                entry.insert(source_local(&nodes[index])?);
            }
            let pose = poses
                .get_mut(&index)
                .ok_or_else(|| invalid("source animation pose preparation failed"))?;
            match binding.attribute {
                1 => pose.translation.copy_from_slice(values),
                2 => pose.rotation_xyzw.copy_from_slice(values),
                3 => pose.scale.copy_from_slice(values),
                _ => return Err(invalid("unsupported source Transform animation property")),
            }
        }
        for (index, pose) in poses {
            local_transforms[index] = Some(bevy_local(&pose)?);
        }
    }
    let mut result = vec![None::<SourceRigNodePose>; nodes.len()];
    for &index in &order {
        let local = local_transforms[index]
            .ok_or_else(|| invalid("source player pose has no local matrix"))?;
        validate_prefab_affine(&local)?;
        let (world_transform, active_in_hierarchy) = if index == root_index {
            (local, nodes[index].active_self)
        } else {
            let parent = nodes[index]
                .parent
                .as_ref()
                .and_then(|parent| indices.get(parent))
                .ok_or_else(|| invalid("source player pose has no parent"))?;
            let parent = result[*parent]
                .as_ref()
                .ok_or_else(|| invalid("source player pose parent is unevaluated"))?;
            (
                prefab_affine_product(&parent.world_transform, &local)?,
                parent.active_in_hierarchy && nodes[index].active_self,
            )
        };
        result[index] = Some(SourceRigNodePose {
            world_transform,
            active_in_hierarchy,
        });
    }
    let binding_position =
        |binding: Option<&SourceObject>| -> Result<Option<[f32; 3]>, SourceRigPoseError> {
            binding
                .map(|binding| {
                    let index = package
                        .transform_index(binding)
                        .filter(|index| selected.contains(index))
                        .ok_or_else(|| {
                            invalid("native source query binding is outside selected rig")
                        })?;
                    let pose = result[index]
                        .as_ref()
                        .ok_or_else(|| invalid("native source query binding has no pose"))?;
                    Ok(prefab_transform_point(&pose.world_transform, [0.0; 3])?)
                })
                .transpose()
        };
    let root_bone = binding_position(root_binding.root_bone.as_ref())?;
    let head_top = binding_position(root_binding.head_top.as_ref())?;
    Ok(SourceRigPose {
        root: root.clone(),
        local_transforms,
        nodes: result,
        root_bone,
        head_top,
        unapplied_animator_curves,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use midnight_map_data::{
        FileIdentity, PlayerAnimatorBinding, PlayerRigManifest, PlayerRigRoot, SourceApk,
        SourceComponent, player_animation::*,
    };

    fn id(value: i64) -> SourceObject {
        SourceObject {
            bundle: "assets/role/role.ab".into(),
            cab: "CAB-fixture".into(),
            path_id: value.to_string(),
        }
    }
    fn trs(
        translation: [f32; 3],
        rotation_xyzw: [f32; 4],
        scale: [f32; 3],
    ) -> SourceMotionTransform {
        SourceMotionTransform {
            translation,
            rotation_xyzw,
            scale,
        }
    }
    fn identity() -> SourceMotionTransform {
        trs([0.0; 3], [0.0, 0.0, 0.0, 1.0], [1.0; 3])
    }
    fn clip() -> SourceAnimationClip {
        SourceAnimationClip {
            source: id(90),
            name: "scalar fixture".into(),
            sample_rate: 30.0,
            wrap_mode: 0,
            use_high_quality_curve: false,
            has_generic_root_transform: false,
            has_motion_float_curves: false,
            bindings: vec![SourceAnimationBinding {
                path_hash: 2,
                attribute: 1,
                type_id: 4,
                custom_type: 0,
                is_pptr_curve: false,
                script_file_id: 0,
                script_path_id: "0".into(),
            }],
            streamed: SourceStreamedClip {
                curve_count: 1,
                frames: vec![
                    SourceStreamedFrame {
                        time_bits: (-f32::MAX).to_bits(),
                        keys: vec![SourceStreamedKey {
                            curve_index: 0,
                            coefficient_bits: [0, 0, 0, 1.0f32.to_bits()],
                        }],
                    },
                    SourceStreamedFrame {
                        time_bits: 0.0f32.to_bits(),
                        keys: vec![SourceStreamedKey {
                            curve_index: 0,
                            coefficient_bits: [0, 0, 2.0f32.to_bits(), 1.0f32.to_bits()],
                        }],
                    },
                    SourceStreamedFrame {
                        time_bits: 1.0f32.to_bits(),
                        keys: vec![SourceStreamedKey {
                            curve_index: 0,
                            coefficient_bits: [0, 0, 0, 3.0f32.to_bits()],
                        }],
                    },
                ],
            },
            dense: SourceDenseClip {
                frame_count: 2,
                curve_count: 1,
                sample_rate: 1.0,
                begin_time: 0.0,
                samples: vec![3.0, 5.0],
            },
            constant: vec![7.0],
            motion: SourceClipMotion {
                start_time: 0.0,
                stop_time: 1.0,
                start: identity(),
                stop: identity(),
                average_speed: [0.0; 3],
                average_angular_speed: 0.0,
                orientation_offset_y: 0.0,
                level: 0.0,
                cycle_offset: 0.0,
                loop_time: false,
                loop_blend: false,
                loop_blend_orientation: false,
                loop_blend_position_y: false,
                loop_blend_position_xz: false,
                keep_original_orientation: false,
                keep_original_position_y: false,
                keep_original_position_xz: false,
                height_from_feet: false,
                start_at_origin: false,
                mirror: false,
            },
            events: vec![],
        }
    }
    fn node(
        value: i64,
        name: &str,
        parent: Option<i64>,
        children: &[i64],
        pose: SourceMotionTransform,
    ) -> PrefabNode {
        let [x, y, z] = pose.translation;
        let [qx, qy, qz, qw] = pose.rotation_xyzw;
        let [sx, sy, sz] = pose.scale;
        PrefabNode {
            source: id(value),
            name: name.into(),
            active_self: true,
            layer: 0,
            transform: id(value + 100),
            parent: parent.map(id),
            children: children.iter().copied().map(id).collect(),
            bevy_local_transform: bevy_local(&pose).unwrap(),
            components: vec![SourceComponent {
                source: id(value + 100),
                class_id: 4,
                serialized: FileIdentity {
                    path: "fixture".into(),
                    sha256: "0".repeat(64),
                },
                fields: serde_json::json!({"m_LocalPosition":{"x":x,"y":y,"z":z},"m_LocalRotation":{"x":qx,"y":qy,"z":qz,"w":qw},"m_LocalScale":{"x":sx,"y":sy,"z":sz}}),
            }],
        }
    }
    fn fixture() -> PlayerRigPackage {
        let nodes = vec![
            node(
                1,
                "role",
                None,
                &[2],
                trs([900.0, 800.0, 700.0], [0.0, 0.0, 0.0, 1.0], [1.0; 3]),
            ),
            node(
                2,
                "Bone",
                Some(1),
                &[3],
                trs(
                    [1.0, 2.0, 3.0],
                    Quat::from_rotation_y(std::f32::consts::FRAC_PI_2).to_array(),
                    [-2.0, 3.0, 4.0],
                ),
            ),
            node(
                3,
                "Head",
                Some(2),
                &[],
                trs([0.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0], [-1.0, 0.5, 2.0]),
            ),
        ];
        let avatar = SourceAvatar {
            source: id(80),
            name: "generic fixture".into(),
            paths: vec![
                SourceAvatarPath {
                    hash: 1,
                    path: "Bone".into(),
                },
                SourceAvatarPath {
                    hash: 2,
                    path: "Bone/Head".into(),
                },
            ],
            joints: vec![],
            root_motion_bone_index: -1,
            root_motion_bone: identity(),
            root_motion_bone_name: "".into(),
            human_bone_count: 0,
            has_translation_dof: false,
            global_scale: 1.0,
        };
        PlayerRigPackage {
            root: std::path::PathBuf::new(),
            manifest_sha256: "fixture".into(),
            rest_poses: vec![],
            manifest: PlayerRigManifest {
                schema_version: 1,
                source_apk: SourceApk {
                    path: "fixture.apk".into(),
                    sha256: "0".repeat(64),
                    byte_count: 0,
                },
                coordinate_frame: "fixture".into(),
                binding_contract: midnight_map_data::PLAYER_BINDING_CONTRACT.into(),
                bundles: vec![],
                inventory: vec![],
                roots: vec![PlayerRigRoot {
                    asset_path: "role".into(),
                    game_object: id(1),
                    root_bone: Some(id(102)),
                    head_top: Some(id(103)),
                    player_collider: None,
                }],
                nodes,
                assets: vec![],
                animators: vec![PlayerAnimatorBinding {
                    source: id(70),
                    game_object: id(1),
                    avatar: id(80),
                    controller: id(60),
                    clips: vec![Some(id(90))],
                    default_clip_indices: vec![0],
                }],
                colliders: vec![],
                decoded_avatars: vec![],
                decoded_clips: vec![],
                empty_override_clips: vec![],
                animation_maps: midnight_map_data::PlayerAnimationMaps {
                    name_map: id(200),
                    state_map: id(201),
                    names: BTreeMap::new(),
                    states: BTreeMap::new(),
                },
                clip_overrides: vec![],
                retained_files: BTreeMap::new(),
                limitations: vec![],
            },
            avatars: BTreeMap::from([(id(80), avatar)]),
            clips: BTreeMap::from([(id(90), clip())]),
        }
    }
    fn close(a: [f32; 3], b: [f32; 3]) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < 2e-5, "{a:?} != {b:?}");
        }
    }

    #[test]
    fn scalar_samples_preserve_knots_cubic_dense_and_constant_channel_order() {
        let mut clip = clip();
        assert_eq!(
            sample_source_clip_scalars(&clip, 0.0).unwrap(),
            vec![1.0, 3.0, 7.0]
        );
        assert_eq!(
            sample_source_clip_scalars(&clip, 0.5).unwrap(),
            vec![2.0, 4.0, 7.0]
        );
        assert_eq!(
            sample_source_clip_scalars(&clip, 1.0).unwrap(),
            vec![3.0, 5.0, 7.0]
        );
        clip.streamed.frames[1].keys[0].coefficient_bits = [
            2.0f32.to_bits(),
            3.0f32.to_bits(),
            4.0f32.to_bits(),
            5.0f32.to_bits(),
        ];
        assert_eq!(sample_source_clip_scalars(&clip, 0.5).unwrap()[0], 8.0);
        assert!(sample_source_clip_scalars(&clip, -0.1).is_err());
        assert!(sample_source_clip_scalars(&clip, f32::NAN).is_err());
    }

    #[test]
    fn sampled_parent_rotation_and_signed_nonuniform_scale_reflect_coordinates_once() {
        let package = fixture();
        let root = id(1);
        let animator = id(70);
        let clip = id(90);
        let placement = Mat4::from_translation(Vec3::new(10.0, 20.0, 30.0)).to_cols_array();
        let pose = evaluate_source_rig_pose(
            &package,
            &root,
            placement,
            Some(SourceRigClipSample {
                animator: &animator,
                clip: &clip,
                time_seconds: 0.5,
            }),
        )
        .unwrap();
        close(pose.root_bone.unwrap(), [11.0, 22.0, 27.0]);
        close(pose.head_top.unwrap(), [39.0, 34.0, 23.0]);
        let changed_placement = Mat4::from_scale_rotation_translation(
            Vec3::new(2.0, 1.0, -1.0),
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            Vec3::new(10.0, 20.0, 30.0),
        )
        .to_cols_array();
        let pose = evaluate_source_rig_pose(
            &package,
            &root,
            changed_placement,
            Some(SourceRigClipSample {
                animator: &animator,
                clip: &clip,
                time_seconds: 0.5,
            }),
        )
        .unwrap();
        close(pose.head_top.unwrap(), [-4.0, 78.0, 37.0]);
        assert_eq!(pose.local_transforms[0], Some(changed_placement));
    }

    #[test]
    fn rest_pose_preserves_hierarchy_and_unresolved_samples_do_not_produce_query_origins() {
        let mut package = fixture();
        let root = id(1);
        let animator = id(70);
        let clip = id(90);
        let placement = Mat4::IDENTITY.to_cols_array();
        let rest = evaluate_source_rig_pose(&package, &root, placement, None).unwrap();
        close(rest.root_bone.unwrap(), [1.0, 2.0, -3.0]);
        close(rest.head_top.unwrap(), [5.0, 5.0, -3.0]);
        package.clips.get_mut(&clip).unwrap().bindings[0].path_hash = 999;
        assert!(
            evaluate_source_rig_pose(
                &package,
                &root,
                placement,
                Some(SourceRigClipSample {
                    animator: &animator,
                    clip: &clip,
                    time_seconds: 0.5
                })
            )
            .unwrap_err()
            .0
            .contains("absent from Avatar")
        );
        assert_eq!(
            rest,
            evaluate_source_rig_pose(&package, &root, placement, None).unwrap()
        );
        package.manifest.roots[0].root_bone = None;
        assert_eq!(
            evaluate_source_rig_pose(&package, &root, placement, None)
                .unwrap()
                .root_bone,
            None
        );
    }

    #[test]
    fn opaque_animator_curves_remain_unapplied_and_cannot_move_the_placement_root() {
        let mut package = fixture();
        let root = id(1);
        let animator = id(70);
        let clip_id = id(90);
        let clip = package.clips.get_mut(&clip_id).unwrap();
        clip.bindings.push(SourceAnimationBinding {
            path_hash: 0,
            attribute: 7,
            type_id: 95,
            custom_type: 8,
            is_pptr_curve: false,
            script_file_id: 0,
            script_path_id: "0".into(),
        });
        clip.constant.push(200.0);
        let pose = evaluate_source_rig_pose(
            &package,
            &root,
            Mat4::IDENTITY.to_cols_array(),
            Some(SourceRigClipSample {
                animator: &animator,
                clip: &clip_id,
                time_seconds: 0.5,
            }),
        )
        .unwrap();
        assert_eq!(pose.unapplied_animator_curves.len(), 1);
        assert_eq!(pose.unapplied_animator_curves[0].value, 200.0);
        close(pose.head_top.unwrap(), [29.0, 14.0, -7.0]);
        assert_eq!(
            pose.nodes[0].as_ref().unwrap().world_transform,
            Mat4::IDENTITY.to_cols_array()
        );
    }

    #[test]
    fn placeholder_resolution_requires_verified_mapping_and_animator_slot_ownership() {
        let mut package = fixture();
        package.manifest.animators[0].clips = vec![Some(id(89))];
        package.manifest.empty_override_clips = vec![id(89)];
        assert!(source_animator_clip(&package, &id(70), &id(89)).is_err());
        assert!(source_animator_clip(&package, &id(70), &id(90)).is_err());
        package
            .manifest
            .clip_overrides
            .push(midnight_map_data::PlayerClipOverride {
                placeholder: id(89),
                asset_path: "source/verified.anim".into(),
                clip: id(90),
            });
        assert_eq!(
            source_animator_clip(&package, &id(70), &id(89))
                .unwrap()
                .source,
            id(90)
        );
        assert_eq!(
            source_animator_clip(&package, &id(70), &id(90))
                .unwrap()
                .source,
            id(90)
        );
        assert!(source_animator_clip(&package, &id(71), &id(90)).is_err());
    }

    #[test]
    #[ignore = "requires a verified imported player package in MIDNIGHT_PLAYER_PACKAGE"]
    fn retained_role_default_clips_sample_with_exact_avatar_paths() {
        let path =
            std::env::var_os("MIDNIGHT_PLAYER_PACKAGE").expect("set MIDNIGHT_PLAYER_PACKAGE");
        let package =
            midnight_map_data::load_player_rig_package(std::path::Path::new(&path)).unwrap();
        let mut sampled = 0;
        for root_name in ["role", "role_new"] {
            let root = package
                .root_named(root_name)
                .expect("source role root retained");
            let animator = package
                .manifest
                .animators
                .iter()
                .find(|animator| animator.game_object == root.game_object)
                .expect("source role Animator retained");
            for index in &animator.default_clip_indices {
                let id = animator.clips[*index as usize]
                    .as_ref()
                    .expect("required default clip nonnull");
                let clip = source_animator_clip(&package, &animator.source, id)
                    .expect("role default clip or override decoded");
                for fraction in [0.0, 0.25, 0.75, 1.0] {
                    let time_seconds = clip.motion.start_time
                        + (clip.motion.stop_time - clip.motion.start_time) * fraction;
                    let pose = evaluate_source_rig_pose(
                        &package,
                        &root.game_object,
                        Mat4::IDENTITY.to_cols_array(),
                        Some(SourceRigClipSample {
                            animator: &animator.source,
                            clip: id,
                            time_seconds,
                        }),
                    )
                    .unwrap_or_else(|error| panic!("{} at {time_seconds}: {error}", clip.name));
                    assert!(pose.root_bone.is_some());
                    assert!(pose.head_top.is_some());
                    println!(
                        "{} {} time={time_seconds} root={:?} head={:?} unapplied={}",
                        clip.name,
                        id.path_id,
                        pose.root_bone,
                        pose.head_top,
                        pose.unapplied_animator_curves.len()
                    );
                    sampled += 1;
                }
            }
        }
        assert!(sampled > 0);
    }
}
