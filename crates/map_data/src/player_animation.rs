//! Lossless generic-animation channels from the source TypeTree.
//!
//! These records use Unity coordinates. They do not choose an Animator state,
//! retarget a humanoid Avatar, apply root motion, or dispatch animation events.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{ArchiveError, SourceObject, invalid};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceAnimationBinding {
    pub path_hash: u32,
    pub attribute: u32,
    pub type_id: i32,
    pub custom_type: u8,
    pub is_pptr_curve: bool,
    pub script_file_id: i32,
    /// Signed Unity PathIDs remain decimal strings in interchange formats.
    pub script_path_id: String,
}

impl SourceAnimationBinding {
    /// Serialized generic Transform channels; other binding types remain unsupported.
    pub fn transform_dimension(&self) -> Option<usize> {
        if self.type_id != 4
            || self.custom_type != 0
            || self.is_pptr_curve
            || self.script_file_id != 0
            || self.script_path_id != "0"
        {
            return None;
        }
        match self.attribute {
            1 | 3 | 4 => Some(3),
            2 => Some(4),
            _ => None,
        }
    }

    /// Width of retained scalar data. Animator attributes remain opaque source
    /// channels; their presence does not authorize a root-motion implementation.
    pub fn scalar_dimension(&self) -> Option<usize> {
        self.transform_dimension().or_else(|| {
            (self.type_id == 95
                && self.custom_type == 8
                && self.path_hash == 0
                && (7..=13).contains(&self.attribute)
                && !self.is_pptr_curve
                && self.script_file_id == 0
                && self.script_path_id == "0")
                .then_some(1)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceStreamedKey {
    pub curve_index: u32,
    /// Source cubic coefficients in descending powers, including their exact bits.
    pub coefficient_bits: [u32; 4],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceStreamedFrame {
    /// Unity sentinel frame times can be infinite; JSON floats cannot retain them.
    pub time_bits: u32,
    pub keys: Vec<SourceStreamedKey>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceStreamedClip {
    pub curve_count: u32,
    pub frames: Vec<SourceStreamedFrame>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceDenseClip {
    pub frame_count: u32,
    pub curve_count: u32,
    pub sample_rate: f32,
    pub begin_time: f32,
    /// Frame-major, in the source's scalar channel order.
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceMotionTransform {
    pub translation: [f32; 3],
    pub rotation_xyzw: [f32; 4],
    pub scale: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceClipMotion {
    pub start_time: f32,
    pub stop_time: f32,
    pub start: SourceMotionTransform,
    pub stop: SourceMotionTransform,
    pub average_speed: [f32; 3],
    pub average_angular_speed: f32,
    pub orientation_offset_y: f32,
    pub level: f32,
    pub cycle_offset: f32,
    pub loop_time: bool,
    pub loop_blend: bool,
    pub loop_blend_orientation: bool,
    pub loop_blend_position_y: bool,
    pub loop_blend_position_xz: bool,
    pub keep_original_orientation: bool,
    pub keep_original_position_y: bool,
    pub keep_original_position_xz: bool,
    pub height_from_feet: bool,
    pub start_at_origin: bool,
    pub mirror: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceAnimationEvent {
    pub time: f32,
    pub function_name: String,
    pub string_parameter: String,
    pub float_parameter: f32,
    pub int_parameter: i32,
    pub message_options: i32,
    pub object_file_id: i32,
    pub object_path_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceAnimationClip {
    pub source: SourceObject,
    pub name: String,
    pub sample_rate: f32,
    pub wrap_mode: i32,
    pub use_high_quality_curve: bool,
    pub has_generic_root_transform: bool,
    pub has_motion_float_curves: bool,
    pub bindings: Vec<SourceAnimationBinding>,
    pub streamed: SourceStreamedClip,
    pub dense: SourceDenseClip,
    pub constant: Vec<f32>,
    pub motion: SourceClipMotion,
    pub events: Vec<SourceAnimationEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceAvatarPath {
    pub hash: u32,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceAvatarJoint {
    pub parent_index: i32,
    pub axes_index: i32,
    pub path_hash: u32,
    pub name_hash: u32,
    pub skeleton_pose: SourceMotionTransform,
    pub default_pose: SourceMotionTransform,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceAvatar {
    pub source: SourceObject,
    pub name: String,
    pub paths: Vec<SourceAvatarPath>,
    pub joints: Vec<SourceAvatarJoint>,
    pub root_motion_bone_index: i32,
    pub root_motion_bone: SourceMotionTransform,
    pub root_motion_bone_name: String,
    pub human_bone_count: usize,
    pub has_translation_dof: bool,
    pub global_scale: f32,
}

fn validate_transform(value: &SourceMotionTransform) -> Result<(), ArchiveError> {
    if value
        .translation
        .iter()
        .chain(&value.rotation_xyzw)
        .chain(&value.scale)
        .any(|v| !v.is_finite())
        || value.rotation_xyzw.iter().map(|v| v * v).sum::<f32>() < 1e-12
    {
        return Err(invalid("invalid source animation transform"));
    }
    // Zero and negative scales are source data, not grounds to replace the pose.
    Ok(())
}

pub fn validate_source_avatar(avatar: &SourceAvatar) -> Result<(), ArchiveError> {
    crate::prefab::validate_source(&avatar.source)?;
    if avatar.name.is_empty() || !avatar.global_scale.is_finite() || avatar.global_scale <= 0.0 {
        return Err(invalid("invalid source Avatar name/scale"));
    }
    let mut paths = BTreeMap::new();
    for entry in &avatar.paths {
        if paths.insert(entry.hash, &entry.path).is_some() {
            return Err(invalid("duplicate source Avatar path hash"));
        }
    }
    let mut hashes = BTreeSet::new();
    for (index, joint) in avatar.joints.iter().enumerate() {
        if !hashes.insert(joint.path_hash) || !paths.contains_key(&joint.path_hash) {
            return Err(invalid("missing/duplicate source Avatar joint path"));
        }
        validate_transform(&joint.default_pose)?;
        validate_transform(&joint.skeleton_pose)?;
        let mut next = joint.parent_index;
        let mut seen = BTreeSet::from([index]);
        while next != -1 {
            let parent = usize::try_from(next)
                .ok()
                .filter(|parent| *parent < avatar.joints.len())
                .ok_or_else(|| invalid("invalid source Avatar parent index"))?;
            if !seen.insert(parent) {
                return Err(invalid("cyclic source Avatar hierarchy"));
            }
            next = avatar.joints[parent].parent_index;
        }
    }
    if avatar.root_motion_bone_index != -1
        && usize::try_from(avatar.root_motion_bone_index)
            .ok()
            .is_none_or(|index| index >= avatar.joints.len())
    {
        return Err(invalid("invalid source Avatar root motion bone index"));
    }
    validate_transform(&avatar.root_motion_bone)
}

pub fn validate_source_animation_clip(clip: &SourceAnimationClip) -> Result<(), ArchiveError> {
    crate::prefab::validate_source(&clip.source)?;
    if clip.name.is_empty()
        || !clip.sample_rate.is_finite()
        || clip.sample_rate <= 0.0
        || !clip.dense.sample_rate.is_finite()
        || clip.dense.sample_rate <= 0.0
        || !clip.dense.begin_time.is_finite()
        || !clip.motion.start_time.is_finite()
        || !clip.motion.stop_time.is_finite()
        || clip.motion.stop_time < clip.motion.start_time
    {
        return Err(invalid("invalid source animation timing"));
    }
    let mut dimensions = 0usize;
    let mut bindings = BTreeSet::new();
    for binding in &clip.bindings {
        let dimension = binding
            .scalar_dimension()
            .ok_or_else(|| invalid("unsupported non-generic source animation binding"))?;
        if !bindings.insert((
            binding.path_hash,
            binding.type_id,
            binding.custom_type,
            binding.attribute,
        )) {
            return Err(invalid("duplicate source animation binding"));
        }
        dimensions = dimensions
            .checked_add(dimension)
            .ok_or_else(|| invalid("source animation dimension overflow"))?;
    }
    let scalar_count = (clip.streamed.curve_count as usize)
        .checked_add(clip.dense.curve_count as usize)
        .and_then(|count| count.checked_add(clip.constant.len()))
        .ok_or_else(|| invalid("source animation channel count overflow"))?;
    if dimensions != scalar_count {
        return Err(invalid("source animation binding/channel count mismatch"));
    }
    let sample_count = (clip.dense.frame_count as usize)
        .checked_mul(clip.dense.curve_count as usize)
        .ok_or_else(|| invalid("source dense animation count overflow"))?;
    if sample_count != clip.dense.samples.len()
        || (clip.dense.curve_count != 0 && clip.dense.frame_count == 0)
        || clip
            .dense
            .samples
            .iter()
            .chain(&clip.constant)
            .any(|value| !value.is_finite())
    {
        return Err(invalid("invalid source dense/constant animation data"));
    }
    let mut previous_time = f32::NEG_INFINITY;
    let mut supplied = BTreeSet::new();
    for (index, frame) in clip.streamed.frames.iter().enumerate() {
        let time = f32::from_bits(frame.time_bits);
        if time.is_nan()
            || time < previous_time
            || (time == f32::NEG_INFINITY && index != 0)
            || (time == f32::INFINITY && index + 1 != clip.streamed.frames.len())
        {
            return Err(invalid("invalid source streamed animation frame order"));
        }
        previous_time = time;
        let mut indices = BTreeSet::new();
        for key in &frame.keys {
            if key.curve_index >= clip.streamed.curve_count
                || !indices.insert(key.curve_index)
                || key
                    .coefficient_bits
                    .iter()
                    .any(|bits| f32::from_bits(*bits).is_nan())
            {
                return Err(invalid("invalid source streamed animation key"));
            }
            if time <= clip.motion.start_time {
                supplied.insert(key.curve_index);
            }
        }
    }
    if supplied.len() != clip.streamed.curve_count as usize {
        return Err(invalid(
            "source streamed animation lacks initial channel values",
        ));
    }
    validate_transform(&clip.motion.start)?;
    validate_transform(&clip.motion.stop)?;
    if clip.motion.average_speed.iter().any(|v| !v.is_finite())
        || [
            clip.motion.average_angular_speed,
            clip.motion.orientation_offset_y,
            clip.motion.level,
            clip.motion.cycle_offset,
        ]
        .iter()
        .any(|v| !v.is_finite())
    {
        return Err(invalid("invalid source root motion metadata"));
    }
    for event in &clip.events {
        if !event.time.is_finite()
            || !event.float_parameter.is_finite()
            || event.object_path_id.parse::<i64>().is_err()
        {
            return Err(invalid("invalid source animation event"));
        }
    }
    Ok(())
}

impl SourceAvatar {
    pub fn path(&self, hash: u32) -> Option<&str> {
        self.paths
            .iter()
            .find(|entry| entry.hash == hash)
            .map(|entry| entry.path.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> SourceObject {
        SourceObject {
            bundle: "assets/role/role.ab".into(),
            cab: "CAB-test".into(),
            path_id: "1".into(),
        }
    }

    fn transform() -> SourceMotionTransform {
        SourceMotionTransform {
            translation: [0.0; 3],
            rotation_xyzw: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
        }
    }

    fn clip() -> SourceAnimationClip {
        SourceAnimationClip {
            source: source(),
            name: "fixture".into(),
            sample_rate: 30.0,
            wrap_mode: 0,
            use_high_quality_curve: false,
            has_generic_root_transform: false,
            has_motion_float_curves: false,
            bindings: vec![SourceAnimationBinding {
                path_hash: 0,
                attribute: 1,
                type_id: 4,
                custom_type: 0,
                is_pptr_curve: false,
                script_file_id: 0,
                script_path_id: "0".into(),
            }],
            streamed: SourceStreamedClip {
                curve_count: 1,
                frames: vec![SourceStreamedFrame {
                    time_bits: (-f32::MAX).to_bits(),
                    keys: vec![SourceStreamedKey {
                        curve_index: 0,
                        coefficient_bits: [0, 0, 0, 2.0f32.to_bits()],
                    }],
                }],
            },
            dense: SourceDenseClip {
                frame_count: 2,
                curve_count: 1,
                sample_rate: 30.0,
                begin_time: 0.0,
                samples: vec![3.0, 4.0],
            },
            constant: vec![5.0],
            motion: SourceClipMotion {
                start_time: 0.0,
                stop_time: 1.0,
                start: transform(),
                stop: transform(),
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

    #[test]
    fn clip_rejects_missing_channels_duplicate_bindings_and_truncated_samples() {
        let base = clip();
        validate_source_animation_clip(&base).unwrap();
        let mut changed = base.clone();
        changed.constant.clear();
        assert!(validate_source_animation_clip(&changed).is_err());
        let mut changed = base.clone();
        changed.bindings.push(changed.bindings[0].clone());
        assert!(validate_source_animation_clip(&changed).is_err());
        let mut changed = base.clone();
        changed.dense.samples.pop();
        assert!(validate_source_animation_clip(&changed).is_err());
        let mut changed = base;
        changed.streamed.frames[0].time_bits = 0.5f32.to_bits();
        assert!(validate_source_animation_clip(&changed).is_err());
    }

    #[test]
    fn streamed_sentinel_bits_round_trip_and_invalid_curve_indices_are_rejected() {
        let mut clip = clip();
        clip.streamed.frames.push(SourceStreamedFrame {
            time_bits: f32::INFINITY.to_bits(),
            keys: vec![],
        });
        let encoded = serde_json::to_vec(&clip).unwrap();
        let decoded: SourceAnimationClip = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, clip);
        validate_source_animation_clip(&decoded).unwrap();
        clip.streamed.frames[0].keys[0].curve_index = 1;
        assert!(validate_source_animation_clip(&clip).is_err());
    }

    #[test]
    fn avatar_hash_mapping_and_parent_graph_are_validated_without_scale_normalization() {
        let mut pose = transform();
        pose.scale = [-1.0, 2.0, 0.5];
        let mut avatar = SourceAvatar {
            source: source(),
            name: "fixture".into(),
            paths: vec![SourceAvatarPath {
                hash: 0,
                path: "".into(),
            }],
            joints: vec![SourceAvatarJoint {
                parent_index: -1,
                axes_index: -1,
                path_hash: 0,
                name_hash: 0,
                skeleton_pose: pose.clone(),
                default_pose: pose,
            }],
            root_motion_bone_index: 0,
            root_motion_bone: transform(),
            root_motion_bone_name: "".into(),
            human_bone_count: 0,
            has_translation_dof: false,
            global_scale: 1.0,
        };
        validate_source_avatar(&avatar).unwrap();
        assert_eq!(avatar.path(0), Some(""));
        avatar.joints[0].parent_index = 0;
        assert!(validate_source_avatar(&avatar).is_err());
    }
}
