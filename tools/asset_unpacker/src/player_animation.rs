//! Bounded decoding of the generic clip/Avatar layout retained in the player bundles.
//! Unsupported clips must retain their complete SourceComponent and original bytes.

use midnight_map_data::{SourceObject, player_animation::*};
use unity_asset_binary::object::UnityObject;
use unity_asset_core::UnityValue;

use crate::{
    fields::{array, boolean, field, integer, invalid, number, property, text, vector},
    unity::Error,
};

const MAX_SCALAR_VALUES: usize = 8 * 1024 * 1024;
const MAX_BINDINGS: usize = 65_536;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ControllerDefaultState {
    pub state_machine_index: usize,
    pub state_index: usize,
    pub layer_names: Vec<String>,
    pub state_name: String,
    pub full_path: String,
    pub clip_indices: Vec<usize>,
}

/// Source controller entry states and their reachable blend-tree clip indices.
/// This identifies an import dependency closure, not a runtime FSM transition.
pub fn controller_default_states(
    object: &UnityObject,
) -> Result<Vec<ControllerDefaultState>, Error> {
    use std::collections::{BTreeMap, BTreeSet};

    let mut names = BTreeMap::new();
    for pair in array(property(object, "m_TOS")?)? {
        let pair = array(pair)?;
        if pair.len() != 2
            || names
                .insert(u32_value(&pair[0])?, text(&pair[1])?)
                .is_some()
        {
            return Err(invalid("invalid/duplicate Animator controller name hash"));
        }
    }
    let resolve_name = |id| -> Result<String, Error> {
        names
            .get(&id)
            .map(|name| (*name).to_string())
            .ok_or_else(|| invalid(format!("missing Animator controller name hash {id}")))
    };
    let controller = property(object, "m_Controller")?;
    let machines = array(field(controller, "m_StateMachineArray")?)?;
    let clip_pointers = array(property(object, "m_AnimationClips")?)?;
    let mut layers = BTreeMap::<usize, Vec<String>>::new();
    for layer in array(field(controller, "m_LayerArray")?)? {
        let data = field(layer, "data")?;
        let machine = usize::try_from(u32_value(field(data, "m_StateMachineIndex")?)?)?;
        if machine >= machines.len() {
            return Err(invalid("Animator layer state-machine index out of range"));
        }
        layers
            .entry(machine)
            .or_default()
            .push(resolve_name(u32_value(field(data, "m_Binding")?)?)?);
    }
    let mut result = Vec::new();
    for (state_machine_index, machine) in machines.iter().enumerate() {
        let machine = field(machine, "data")?;
        let state_index = usize::try_from(u32_value(field(machine, "m_DefaultState")?)?)?;
        let states = array(field(machine, "m_StateConstantArray")?)?;
        let state = states
            .get(state_index)
            .ok_or_else(|| invalid("Animator default state index out of range"))?;
        let state = field(state, "data")?;
        let trees = array(field(state, "m_BlendTreeConstantArray")?)?;
        let mut clip_indices = BTreeSet::new();
        for tree_index in array(field(state, "m_BlendTreeConstantIndexArray")?)? {
            let tree_index = i32_value(tree_index)?;
            if tree_index == -1 {
                continue;
            }
            let tree_index = usize::try_from(tree_index)
                .map_err(|_| invalid("invalid Animator blend-tree index"))?;
            let tree = trees
                .get(tree_index)
                .ok_or_else(|| invalid("Animator blend-tree index out of range"))?;
            let nodes = array(nested(tree, &["data", "m_NodeArray"])?)?;
            if nodes.is_empty() || nodes.len() > MAX_BINDINGS {
                return Err(invalid("empty/oversized Animator blend tree"));
            }
            let mut visited = BTreeSet::new();
            let mut active = BTreeSet::new();
            let mut pending = vec![(0usize, false)];
            while let Some((index, exiting)) = pending.pop() {
                if exiting {
                    active.remove(&index);
                    visited.insert(index);
                    continue;
                }
                if active.contains(&index) {
                    return Err(invalid("cyclic Animator blend tree"));
                }
                if visited.contains(&index) {
                    continue;
                }
                let node = nodes
                    .get(index)
                    .ok_or_else(|| invalid("Animator blend child index out of range"))?;
                let node = field(node, "data")?;
                active.insert(index);
                pending.push((index, true));
                let children = array(field(node, "m_ChildIndices")?)?;
                let clip = u32_value(field(node, "m_ClipID")?)?;
                if clip != u32::MAX {
                    let clip = usize::try_from(clip)?;
                    let pointer = clip_pointers
                        .get(clip)
                        .ok_or_else(|| invalid("Animator clip index out of range"))?;
                    if integer(field(pointer, "m_PathID")?)? == 0 {
                        return Err(invalid("required default Animator clip is null"));
                    }
                    clip_indices.insert(clip);
                }
                for child in children.iter().rev() {
                    pending.push((usize::try_from(u32_value(child)?)?, false));
                }
            }
        }
        result.push(ControllerDefaultState {
            state_machine_index,
            state_index,
            layer_names: layers.remove(&state_machine_index).unwrap_or_default(),
            state_name: resolve_name(u32_value(field(state, "m_NameID")?)?)?,
            full_path: resolve_name(u32_value(field(state, "m_FullPathID")?)?)?,
            clip_indices: clip_indices.into_iter().collect(),
        });
    }
    Ok(result)
}

pub fn controller_default_clips(object: &UnityObject) -> Result<Vec<usize>, Error> {
    Ok(controller_default_states(object)?
        .into_iter()
        .flat_map(|state| state.clip_indices)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect())
}

fn nested<'a>(mut value: &'a UnityValue, names: &[&str]) -> Result<&'a UnityValue, Error> {
    for name in names {
        value = field(value, name)?;
    }
    Ok(value)
}

fn u32_value(value: &UnityValue) -> Result<u32, Error> {
    u32::try_from(integer(value)?).map_err(|_| invalid("source animation value outside u32"))
}

fn i32_value(value: &UnityValue) -> Result<i32, Error> {
    i32::try_from(integer(value)?).map_err(|_| invalid("source animation value outside i32"))
}

fn flag(value: &UnityValue) -> Result<bool, Error> {
    if let Some(value) = value.as_bool() {
        return Ok(value);
    }
    match integer(value)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(invalid("source animation flag outside 0/1")),
    }
}

fn numbers(value: &UnityValue) -> Result<Vec<f32>, Error> {
    let values = array(value)?;
    if values.len() > MAX_SCALAR_VALUES {
        return Err(invalid("source animation scalar budget exceeded"));
    }
    values.iter().map(number).collect()
}

fn motion_transform(value: &UnityValue) -> Result<SourceMotionTransform, Error> {
    Ok(SourceMotionTransform {
        translation: vector(field(value, "t")?, ["x", "y", "z"])?,
        rotation_xyzw: vector(field(value, "q")?, ["x", "y", "z", "w"])?,
        scale: vector(field(value, "s")?, ["x", "y", "z"])?,
    })
}

/// The TypeTree stores streamed clip bytes as little-endian u32 words.
pub fn decode_streamed_words(words: &[u32], curve_count: u32) -> Result<SourceStreamedClip, Error> {
    if words.len() > MAX_SCALAR_VALUES {
        return Err(invalid("source streamed animation word budget exceeded"));
    }
    let mut offset = 0usize;
    let mut frames = Vec::new();
    while offset < words.len() {
        let header = words
            .get(offset..offset + 2)
            .ok_or_else(|| invalid("truncated source streamed animation frame"))?;
        let count = usize::try_from(header[1])?;
        offset += 2;
        let end = count
            .checked_mul(5)
            .and_then(|count| offset.checked_add(count))
            .filter(|end| *end <= words.len())
            .ok_or_else(|| invalid("truncated source streamed animation keys"))?;
        let mut keys = Vec::with_capacity(count);
        for words in words[offset..end].as_chunks::<5>().0 {
            if words[0] >= curve_count {
                return Err(invalid(
                    "source streamed animation curve index out of range",
                ));
            }
            keys.push(SourceStreamedKey {
                curve_index: words[0],
                coefficient_bits: words[1..].try_into()?,
            });
        }
        frames.push(SourceStreamedFrame {
            time_bits: header[0],
            keys,
        });
        offset = end;
    }
    Ok(SourceStreamedClip {
        curve_count,
        frames,
    })
}

pub fn decode_clip(
    object: &UnityObject,
    source: SourceObject,
) -> Result<SourceAnimationClip, Error> {
    if boolean(property(object, "m_Legacy")?)? || boolean(property(object, "m_Compressed")?)? {
        return Err(invalid(
            "legacy/packed-rotation clip requires a separate decoder",
        ));
    }
    for name in [
        "m_RotationCurves",
        "m_CompressedRotationCurves",
        "m_EulerCurves",
        "m_PositionCurves",
        "m_ScaleCurves",
        "m_FloatCurves",
        "m_PPtrCurves",
    ] {
        if !array(property(object, name)?)?.is_empty() {
            return Err(invalid(format!(
                "source {name} requires a separate decoder"
            )));
        }
    }
    let binding_constant = property(object, "m_ClipBindingConstant")?;
    if !array(field(binding_constant, "pptrCurveMapping")?)?.is_empty() {
        return Err(invalid("source object animation curves are unsupported"));
    }
    let binding_values = array(field(binding_constant, "genericBindings")?)?;
    if binding_values.len() > MAX_BINDINGS {
        return Err(invalid("source animation binding budget exceeded"));
    }
    let bindings = binding_values
        .iter()
        .map(|value| {
            let script = field(value, "script")?;
            Ok(SourceAnimationBinding {
                path_hash: u32_value(field(value, "path")?)?,
                attribute: u32_value(field(value, "attribute")?)?,
                type_id: i32_value(field(value, "typeID")?)?,
                custom_type: u8::try_from(integer(field(value, "customType")?)?)?,
                is_pptr_curve: flag(field(value, "isPPtrCurve")?)?,
                script_file_id: i32_value(field(script, "m_FileID")?)?,
                script_path_id: integer(field(script, "m_PathID")?)?.to_string(),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let muscle = property(object, "m_MuscleClip")?;
    let data = nested(muscle, &["m_Clip", "data"])?;
    let streamed = field(data, "m_StreamedClip")?;
    let words = array(field(streamed, "data")?)?;
    if words.len() > MAX_SCALAR_VALUES {
        return Err(invalid("source streamed animation word budget exceeded"));
    }
    let words = words.iter().map(u32_value).collect::<Result<Vec<_>, _>>()?;
    let dense = field(data, "m_DenseClip")?;
    let events = array(property(object, "m_Events")?)?
        .iter()
        .map(|event| {
            let object = field(event, "objectReferenceParameter")?;
            Ok(SourceAnimationEvent {
                time: number(field(event, "time")?)?,
                function_name: text(field(event, "functionName")?)?.into(),
                string_parameter: text(field(event, "data")?)?.into(),
                float_parameter: number(field(event, "floatParameter")?)?,
                int_parameter: i32_value(field(event, "intParameter")?)?,
                message_options: i32_value(field(event, "messageOptions")?)?,
                object_file_id: i32_value(field(object, "m_FileID")?)?,
                object_path_id: integer(field(object, "m_PathID")?)?.to_string(),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let clip = SourceAnimationClip {
        source,
        name: text(property(object, "m_Name")?)?.into(),
        sample_rate: number(property(object, "m_SampleRate")?)?,
        wrap_mode: i32_value(property(object, "m_WrapMode")?)?,
        use_high_quality_curve: boolean(property(object, "m_UseHighQualityCurve")?)?,
        has_generic_root_transform: boolean(property(object, "m_HasGenericRootTransform")?)?,
        has_motion_float_curves: boolean(property(object, "m_HasMotionFloatCurves")?)?,
        bindings,
        streamed: decode_streamed_words(&words, u32_value(field(streamed, "curveCount")?)?)?,
        dense: SourceDenseClip {
            frame_count: u32_value(field(dense, "m_FrameCount")?)?,
            curve_count: u32_value(field(dense, "m_CurveCount")?)?,
            sample_rate: number(field(dense, "m_SampleRate")?)?,
            begin_time: number(field(dense, "m_BeginTime")?)?,
            samples: numbers(field(dense, "m_SampleArray")?)?,
        },
        constant: numbers(nested(data, &["m_ConstantClip", "data"])?)?,
        motion: SourceClipMotion {
            start_time: number(field(muscle, "m_StartTime")?)?,
            stop_time: number(field(muscle, "m_StopTime")?)?,
            start: motion_transform(field(muscle, "m_StartX")?)?,
            stop: motion_transform(field(muscle, "m_StopX")?)?,
            average_speed: vector(field(muscle, "m_AverageSpeed")?, ["x", "y", "z"])?,
            average_angular_speed: number(field(muscle, "m_AverageAngularSpeed")?)?,
            orientation_offset_y: number(field(muscle, "m_OrientationOffsetY")?)?,
            level: number(field(muscle, "m_Level")?)?,
            cycle_offset: number(field(muscle, "m_CycleOffset")?)?,
            loop_time: boolean(field(muscle, "m_LoopTime")?)?,
            loop_blend: boolean(field(muscle, "m_LoopBlend")?)?,
            loop_blend_orientation: boolean(field(muscle, "m_LoopBlendOrientation")?)?,
            loop_blend_position_y: boolean(field(muscle, "m_LoopBlendPositionY")?)?,
            loop_blend_position_xz: boolean(field(muscle, "m_LoopBlendPositionXZ")?)?,
            keep_original_orientation: boolean(field(muscle, "m_KeepOriginalOrientation")?)?,
            keep_original_position_y: boolean(field(muscle, "m_KeepOriginalPositionY")?)?,
            keep_original_position_xz: boolean(field(muscle, "m_KeepOriginalPositionXZ")?)?,
            height_from_feet: boolean(field(muscle, "m_HeightFromFeet")?)?,
            start_at_origin: boolean(field(muscle, "m_StartAtOrigin")?)?,
            mirror: boolean(field(muscle, "m_Mirror")?)?,
        },
        events,
    };
    validate_source_animation_clip(&clip)?;
    // This scalar count also has an independently serialized counterpart.
    if array(field(muscle, "m_ValueArrayDelta")?)?.len()
        != clip.streamed.curve_count as usize
            + clip.dense.curve_count as usize
            + clip.constant.len()
    {
        return Err(invalid(
            "source animation value-delta/channel count mismatch",
        ));
    }
    Ok(clip)
}

/// Recognize the complete zero-channel layout used by the source dummy
/// controller. This preserves a placeholder classification, never a rest pose.
pub fn is_empty_clip_placeholder(object: &UnityObject) -> Result<bool, Error> {
    if boolean(property(object, "m_Legacy")?)? || boolean(property(object, "m_Compressed")?)? {
        return Ok(false);
    }
    for name in [
        "m_RotationCurves",
        "m_CompressedRotationCurves",
        "m_EulerCurves",
        "m_PositionCurves",
        "m_ScaleCurves",
        "m_FloatCurves",
        "m_PPtrCurves",
        "m_Events",
    ] {
        if !array(property(object, name)?)?.is_empty() {
            return Ok(false);
        }
    }
    let binding = property(object, "m_ClipBindingConstant")?;
    if !array(field(binding, "genericBindings")?)?.is_empty()
        || !array(field(binding, "pptrCurveMapping")?)?.is_empty()
    {
        return Ok(false);
    }
    let muscle = property(object, "m_MuscleClip")?;
    let data = nested(muscle, &["m_Clip", "data"])?;
    let streamed = field(data, "m_StreamedClip")?;
    let dense = field(data, "m_DenseClip")?;
    let sample_rate = number(property(object, "m_SampleRate")?)?;
    let start = number(field(muscle, "m_StartTime")?)?;
    let stop = number(field(muscle, "m_StopTime")?)?;
    Ok(sample_rate > 0.0
        && stop >= start
        && u32_value(field(streamed, "curveCount")?)? == 0
        && array(field(streamed, "data")?)?.is_empty()
        && u32_value(field(dense, "m_FrameCount")?)? == 0
        && u32_value(field(dense, "m_CurveCount")?)? == 0
        && number(field(dense, "m_SampleRate")?)? == 0.0
        && number(field(dense, "m_BeginTime")?)? == 0.0
        && array(field(dense, "m_SampleArray")?)?.is_empty()
        && array(nested(data, &["m_ConstantClip", "data"])?)?.is_empty()
        && array(field(muscle, "m_ValueArrayDelta")?)?.is_empty()
        && array(field(muscle, "m_ValueArrayReferencePose")?)?.is_empty())
}

pub fn decode_avatar(object: &UnityObject, source: SourceObject) -> Result<SourceAvatar, Error> {
    let data = property(object, "m_Avatar")?;
    let description = property(object, "m_HumanDescription")?;
    let skeleton = nested(data, &["m_AvatarSkeleton", "data"])?;
    let nodes = array(field(skeleton, "m_Node")?)?;
    let ids = array(field(skeleton, "m_ID")?)?;
    let names = array(field(data, "m_SkeletonNameIDArray")?)?;
    let skeleton_poses = array(nested(data, &["m_AvatarSkeletonPose", "data", "m_X"])?)?;
    let default_poses = array(nested(data, &["m_DefaultPose", "data", "m_X"])?)?;
    if nodes.len() > MAX_BINDINGS
        || [
            ids.len(),
            names.len(),
            skeleton_poses.len(),
            default_poses.len(),
        ]
        .iter()
        .any(|count| *count != nodes.len())
    {
        return Err(invalid("source Avatar skeleton/pose count mismatch"));
    }
    let joints = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            Ok(SourceAvatarJoint {
                parent_index: i32_value(field(node, "m_ParentId")?)?,
                axes_index: i32_value(field(node, "m_AxesId")?)?,
                path_hash: u32_value(&ids[index])?,
                name_hash: u32_value(&names[index])?,
                skeleton_pose: motion_transform(&skeleton_poses[index])?,
                default_pose: motion_transform(&default_poses[index])?,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let paths = array(property(object, "m_TOS")?)?
        .iter()
        .map(|pair| {
            let pair = array(pair)?;
            if pair.len() != 2 {
                return Err(invalid(
                    "source Avatar path dictionary pair has wrong length",
                ));
            }
            Ok(SourceAvatarPath {
                hash: u32_value(&pair[0])?,
                path: text(&pair[1])?.into(),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let avatar = SourceAvatar {
        source,
        name: text(property(object, "m_Name")?)?.into(),
        paths,
        joints,
        root_motion_bone_index: i32_value(field(data, "m_RootMotionBoneIndex")?)?,
        root_motion_bone: motion_transform(field(data, "m_RootMotionBoneX")?)?,
        root_motion_bone_name: text(field(description, "m_RootMotionBoneName")?)?.into(),
        human_bone_count: array(field(description, "m_Human")?)?.len(),
        has_translation_dof: boolean(field(description, "m_HasTranslationDoF")?)?,
        global_scale: number(field(description, "m_GlobalScale")?)?,
    };
    validate_source_avatar(&avatar)?;
    Ok(avatar)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(class_id: i32, properties: serde_json::Value) -> UnityObject {
        let mut class = unity_asset_core::UnityClass::new(class_id, "fixture".into(), "1".into());
        let value: UnityValue = serde_json::from_value(properties).unwrap();
        for (key, value) in value.as_object().unwrap() {
            class.set(key.clone(), value.clone());
        }
        UnityObject::from_info_and_class(
            unity_asset_binary::asset::ObjectInfo::new(1, 0, 0, class_id, -1),
            class,
        )
    }

    fn controller() -> serde_json::Value {
        serde_json::json!({
            "m_TOS":[[10,"Base Layer"],[11,"Stand"],[12,"Base Layer.Stand"]],
            "m_AnimationClips":[{"m_FileID":2,"m_PathID":123},{"m_FileID":0,"m_PathID":0}],
            "m_Controller":{
                "m_LayerArray":[{"data":{"m_StateMachineIndex":0,"m_Binding":10}}],
                "m_StateMachineArray":[{"data":{"m_DefaultState":0,"m_StateConstantArray":[{"data":{
                    "m_NameID":11,"m_FullPathID":12,"m_BlendTreeConstantIndexArray":[0],
                    "m_BlendTreeConstantArray":[{"data":{"m_NodeArray":[
                        {"data":{"m_ChildIndices":[1],"m_ClipID":4294967295u32}},
                        {"data":{"m_ChildIndices":[],"m_ClipID":0}},
                        {"data":{"m_ChildIndices":[],"m_ClipID":1}}
                    ]}}]
                }}]}}]
            }
        })
    }

    #[test]
    fn default_dependencies_follow_reachable_children_and_reject_cycles_and_required_nulls() {
        let base = controller();
        assert_eq!(
            controller_default_clips(&object(91, base.clone())).unwrap(),
            vec![0]
        );
        let mut null = base.clone();
        null["m_AnimationClips"][0]["m_PathID"] = serde_json::json!(0);
        assert!(
            controller_default_clips(&object(91, null))
                .unwrap_err()
                .to_string()
                .contains("null")
        );
        let mut cyclic = base.clone();
        cyclic["m_Controller"]["m_StateMachineArray"][0]["data"]["m_StateConstantArray"][0]["data"]
            ["m_BlendTreeConstantArray"][0]["data"]["m_NodeArray"][0]["data"]["m_ChildIndices"] =
            serde_json::json!([0]);
        assert!(
            controller_default_clips(&object(91, cyclic))
                .unwrap_err()
                .to_string()
                .contains("cyclic")
        );
        let mut inactive = base;
        inactive["m_Controller"]["m_StateMachineArray"][0]["data"]["m_StateConstantArray"][0]["data"]
            ["m_BlendTreeConstantIndexArray"] = serde_json::json!([-1]);
        assert!(
            controller_default_clips(&object(91, inactive))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn placeholder_requires_complete_empty_channels_without_discarding_supplied_data() {
        let mut value = serde_json::json!({"m_Legacy":false,"m_Compressed":false,"m_SampleRate":60.0,
            "m_ClipBindingConstant":{"genericBindings":[],"pptrCurveMapping":[]},
            "m_MuscleClip":{"m_StartTime":0.0,"m_StopTime":1.0,"m_ValueArrayDelta":[],"m_ValueArrayReferencePose":[],
                "m_Clip":{"data":{"m_StreamedClip":{"curveCount":0,"data":[]},
                    "m_DenseClip":{"m_FrameCount":0,"m_CurveCount":0,"m_SampleRate":0.0,"m_BeginTime":0.0,"m_SampleArray":[]},
                    "m_ConstantClip":{"data":[]}}}}});
        for key in [
            "m_RotationCurves",
            "m_CompressedRotationCurves",
            "m_EulerCurves",
            "m_PositionCurves",
            "m_ScaleCurves",
            "m_FloatCurves",
            "m_PPtrCurves",
            "m_Events",
        ] {
            value[key] = serde_json::json!([]);
        }
        assert!(is_empty_clip_placeholder(&object(74, value.clone())).unwrap());
        value["m_MuscleClip"]["m_Clip"]["data"]["m_ConstantClip"]["data"] =
            serde_json::json!([0.5]);
        assert!(!is_empty_clip_placeholder(&object(74, value)).unwrap());
    }

    #[test]
    fn streamed_decoder_preserves_exact_words_and_rejects_truncation_before_allocation() {
        let words = [
            (-f32::MAX).to_bits(),
            1,
            0,
            0,
            0,
            0,
            1.25f32.to_bits(),
            f32::INFINITY.to_bits(),
            0,
        ];
        let decoded = decode_streamed_words(&words, 1).unwrap();
        assert_eq!(decoded.frames.len(), 2);
        assert_eq!(decoded.frames[0].time_bits, words[0]);
        assert_eq!(
            decoded.frames[0].keys[0].coefficient_bits,
            [0, 0, 0, 1.25f32.to_bits()]
        );
        assert_eq!(decoded.frames[1].time_bits, f32::INFINITY.to_bits());
        assert!(decode_streamed_words(&words[..6], 1).is_err());
        assert!(decode_streamed_words(&[0, u32::MAX], 1).is_err());
        assert!(decode_streamed_words(&words, 0).is_err());
        assert!(decode_streamed_words(&[0], 1).is_err());
    }
}
