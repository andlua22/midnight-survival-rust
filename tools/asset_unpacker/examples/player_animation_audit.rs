//! Decode selected source clips/Avatars directly from bounded APK bundle reads.
#![allow(dead_code)]

#[path = "../src/fields.rs"]
mod fields;
#[path = "../src/player_animation.rs"]
mod player_animation;
#[path = "../src/unity.rs"]
mod unity;

use midnight_map_data::SourceObject;
use serde_json::json;

fn controller_state(
    state: &unity_asset_core::UnityValue,
    names: &std::collections::BTreeMap<i64, String>,
    clips: &[unity_asset_core::UnityValue],
) -> Result<serde_json::Value, unity::Error> {
    let mut clip_ids = std::collections::BTreeSet::new();
    let mut parameters = std::collections::BTreeSet::new();
    let mut blends = std::collections::BTreeSet::new();
    for tree in fields::array(fields::field(state, "m_BlendTreeConstantArray")?)? {
        for node in fields::array(fields::field(fields::field(tree, "data")?, "m_NodeArray")?)? {
            let node = fields::field(node, "data")?;
            let clip = fields::integer(fields::field(node, "m_ClipID")?)?;
            if clip != i64::from(u32::MAX) {
                clip_ids.insert(clip);
            }
            blends.insert(fields::integer(fields::field(node, "m_BlendType")?)?);
            for parameter in ["m_BlendEventID", "m_BlendEventYID"] {
                let value = fields::integer(fields::field(node, parameter)?)?;
                if value != i64::from(u32::MAX) {
                    parameters.insert(value);
                }
            }
        }
    }
    let clips = clip_ids.into_iter().map(|index| {
        let clip = clips.get(usize::try_from(index)?).ok_or("controller clip ID out of bounds")?;
        Ok(json!({"index":index,"file_id":fields::integer(fields::field(clip,"m_FileID")?)?,"path_id":fields::integer(fields::field(clip,"m_PathID")?)?.to_string()}))
    }).collect::<Result<Vec<_>, unity::Error>>()?;
    Ok(json!({
        "path": names.get(&fields::integer(fields::field(state,"m_PathID")?)?),
        "full_path": names.get(&fields::integer(fields::field(state,"m_FullPathID")?)?),
        "speed":fields::number(fields::field(state,"m_Speed")?)?,
        "loop":fields::boolean(fields::field(state,"m_Loop")?)?,
        "ik_on_feet":fields::boolean(fields::field(state,"m_IKOnFeet")?)?,
        "blend_types":blends,"parameters":parameters.iter().map(|id|json!({"hash":id,"name":names.get(id)})).collect::<Vec<_>>(),
        "clip_references":clips
    }))
}

fn main() -> Result<(), unity::Error> {
    let mut args = std::env::args().skip(1);
    let apk = args.next().ok_or("expected APK path")?;
    let member = args.next().ok_or("expected exact APK bundle member")?;
    let selectors = args.collect::<Vec<_>>();
    let mut archive = unity::Apk::open(std::path::Path::new(&apk))?;
    let bundle = archive.bundle(&member)?;
    for (index, asset) in bundle.assets.iter().enumerate() {
        for info in &asset.objects {
            if ![74, 90, 91].contains(&info.type_id) {
                continue;
            }
            let object = unity::read_object(asset, info)?;
            let name = fields::text(fields::property(&object, "m_Name")?)?;
            if !selectors.is_empty()
                && !selectors
                    .iter()
                    .any(|selector| selector == name || selector == &info.path_id.to_string())
            {
                continue;
            }
            let source = SourceObject {
                bundle: member.clone(),
                cab: unity::cab_name(&bundle.asset_names[index]).into(),
                path_id: info.path_id.to_string(),
            };
            let result = if info.type_id == 91 {
                let default_dependencies = player_animation::controller_default_states(&object)?;
                let fields = object.class.properties();
                let tos = fields::array(fields.get("m_TOS").ok_or("missing controller TOS")?)?;
                let names = tos
                    .iter()
                    .map(|pair| {
                        let pair = fields::array(pair)?;
                        Ok((
                            fields::integer(&pair[0])?,
                            fields::text(&pair[1])?.to_string(),
                        ))
                    })
                    .collect::<Result<std::collections::BTreeMap<_, _>, unity::Error>>()?;
                let controller = fields
                    .get("m_Controller")
                    .ok_or("missing controller data")?;
                let machines = fields::array(fields::field(controller, "m_StateMachineArray")?)?;
                let clips = fields::array(
                    fields
                        .get("m_AnimationClips")
                        .ok_or("missing controller clips")?,
                )?;
                let mut defaults = Vec::new();
                let mut states = Vec::new();
                for (machine_index, machine) in machines.iter().enumerate() {
                    let machine = fields::field(machine, "data")?;
                    let default = fields::integer(fields::field(machine, "m_DefaultState")?)?;
                    let values = fields::array(fields::field(machine, "m_StateConstantArray")?)?;
                    for (state_index, state) in values.iter().enumerate() {
                        let state = fields::field(state, "data")?;
                        let name_id = fields::integer(fields::field(state, "m_NameID")?)?;
                        let name = names.get(&name_id).cloned().unwrap_or_default();
                        if state_index as i64 == default {
                            defaults.push(json!({"machine":machine_index,"index":state_index,"name":name,"state":controller_state(state,&names,clips)?}));
                        }
                        if (machine_index == 0 && state_index < 14)
                            || name.contains("kongshou")
                            || [
                                "zhan_stand01",
                                "zhan_run_front01",
                                "dun_stand01",
                                "pa_stand01",
                            ]
                            .contains(&name.as_str())
                        {
                            states.push(json!({"machine":machine_index,"index":state_index,"name":name,"state":controller_state(state,&names,clips)?}));
                        }
                    }
                }
                Ok(
                    json!({"source":source,"name":name,"defaults":defaults,"default_dependencies":default_dependencies,"matching_states":states,"clip_count":clips.len(),"null_clip_count":clips.iter().filter(|clip|fields::integer(fields::field(clip,"m_PathID").unwrap()).unwrap()==0).count()}),
                )
            } else if info.type_id == 74 {
                player_animation::decode_clip(&object, source.clone()).map(|clip| json!({
                    "source":source,"name":name,"bindings":clip.bindings.len(),
                    "streamed_curves":clip.streamed.curve_count,"streamed_frames":clip.streamed.frames.len(),
                    "dense_curves":clip.dense.curve_count,"dense_frames":clip.dense.frame_count,
                    "constant_curves":clip.constant.len(),"events":clip.events.len(),
                    "start_time":clip.motion.start_time,"stop_time":clip.motion.stop_time,
                    "binding_paths":clip.bindings.iter().map(|binding|binding.path_hash).collect::<std::collections::BTreeSet<_>>(),
                    "first_times":clip.streamed.frames.iter().take(5).map(|frame|f32::from_bits(frame.time_bits)).collect::<Vec<_>>()
                }))
            } else {
                player_animation::decode_avatar(&object, source.clone()).map(|avatar| json!({
                    "source":source,"name":name,"joints":avatar.joints.len(),"paths":avatar.paths.len(),
                    "human_bones":avatar.human_bone_count,"root_motion_bone":avatar.root_motion_bone_name,
                    "path_entries":avatar.paths
                }))
            };
            match result {
                Ok(result) => println!("{result}"),
                Err(error) if info.type_id == 74 => {
                    let bindings = fields::array(fields::field(
                        fields::property(&object, "m_ClipBindingConstant")?,
                        "genericBindings",
                    )?)?;
                    let other = bindings
                        .iter()
                        .filter(|binding| {
                            fields::integer(fields::field(binding, "typeID").unwrap()).unwrap() != 4
                        })
                        .collect::<Vec<_>>();
                    println!(
                        "{}",
                        json!({"source":source,"name":name,"error":error.to_string(),"non_transform_bindings":other,
                            "sample_rate":fields::property(&object,"m_SampleRate")?,
                            "empty_placeholder":player_animation::is_empty_clip_placeholder(&object)?})
                    );
                }
                Err(error) => println!(
                    "{}",
                    json!({"source":source,"name":name,"error":error.to_string()})
                ),
            }
        }
    }
    Ok(())
}
