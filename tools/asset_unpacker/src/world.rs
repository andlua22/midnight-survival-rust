//! Bounded placement discovery. Cell topology and prefab transforms stay separate.

use crate::{
    fields::invalid,
    unity::{Apk, BundleLimits, Error, parse_bundle, source_apk},
    world_scene::{ByteSpan, MAX_PAYLOAD_BYTES, decode_scene_info},
};
use midnight_map_data::{SourceApk, file_sha256};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

const MAX_SELECTED_MEMBERS: usize = 8;
const MAX_JSON_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Serialize)]
pub struct MemberIdentity {
    pub member: String,
    pub byte_count: usize,
    pub sha256: String,
}

fn identity(member: &str, bytes: &[u8]) -> MemberIdentity {
    MemberIdentity {
        member: member.into(),
        byte_count: bytes.len(),
        sha256: format!("{:x}", Sha256::digest(bytes)),
    }
}

#[derive(Debug, Serialize)]
pub struct MetadataLiteral {
    pub index: usize,
    pub value: String,
    pub source: ByteSpan,
}

#[derive(Debug, Serialize)]
pub struct MetadataSummary {
    pub source: MemberIdentity,
    pub version: u32,
    pub literal_rows: usize,
    pub selected_literals: Vec<MetadataLiteral>,
    pub runtime_path_concatenation_verified: bool,
}

fn metadata_summary(apk: &mut Apk) -> Result<MetadataSummary, Error> {
    let member = "assets/bin/Data/Managed/Metadata/global-metadata.dat";
    let bytes = apk.member_data(member)?;
    let word = |offset: usize| -> Result<u32, Error> {
        let source = bytes
            .get(offset..offset + 4)
            .ok_or_else(|| invalid("truncated metadata header"))?;
        Ok(u32::from_le_bytes(source.try_into()?))
    };
    let version = word(4)?;
    if word(0)? != 0xFAB1_1BAF || version != 29 {
        return Err(invalid(
            "selected metadata does not use the audited IL2CPP v29 header",
        ));
    }
    let table_offset = usize::try_from(word(8)?)?;
    let table_bytes = usize::try_from(word(12)?)?;
    let data_offset = usize::try_from(word(16)?)?;
    let data_bytes = usize::try_from(word(20)?)?;
    let table_end = table_offset
        .checked_add(table_bytes)
        .ok_or_else(|| invalid("metadata literal table overflow"))?;
    let data_end = data_offset
        .checked_add(data_bytes)
        .ok_or_else(|| invalid("metadata literal data overflow"))?;
    let table = bytes
        .get(table_offset..table_end)
        .ok_or_else(|| invalid("metadata literal table outside payload"))?;
    let data = bytes
        .get(data_offset..data_end)
        .ok_or_else(|| invalid("metadata literal data outside payload"))?;
    if !table_bytes.is_multiple_of(8) {
        return Err(invalid("metadata literal table is misaligned"));
    }
    let requested = [
        "sceneinfo/",
        ".oc",
        "objectInfo.obj",
        "gameconfig/preabs.obj",
        "gameconfig/partpreabs.obj",
        "gameconfig/scenecell.obj",
        "gameconfig/allscenelist.obj",
    ];
    let mut selected_literals = Vec::new();
    for (index, row) in table.as_chunks::<8>().0.iter().enumerate() {
        let count = usize::try_from(u32::from_le_bytes(row[..4].try_into()?))?;
        let offset = usize::try_from(u32::from_le_bytes(row[4..].try_into()?))?;
        let end = offset
            .checked_add(count)
            .ok_or_else(|| invalid("metadata literal span overflow"))?;
        let literal = data
            .get(offset..end)
            .ok_or_else(|| invalid("metadata literal outside data table"))?;
        if let Some(value) = requested.iter().find(|value| value.as_bytes() == literal) {
            selected_literals.push(MetadataLiteral {
                index,
                value: (*value).into(),
                source: ByteSpan {
                    offset: data_offset + offset,
                    byte_count: count,
                    sha256: format!("{:x}", Sha256::digest(literal)),
                },
            });
        }
    }
    Ok(MetadataSummary {
        source: identity(member, &bytes),
        version,
        literal_rows: table_bytes / 8,
        selected_literals,
        runtime_path_concatenation_verified: false,
    })
}

#[derive(Debug, Serialize)]
pub struct TopologySummary {
    pub source: MemberIdentity,
    pub rows: usize,
    pub nonempty_cells: usize,
    pub referenced_scene_names: Vec<String>,
    pub sample_row_fields: Vec<String>,
    pub authority: &'static str,
}

fn topology(apk: &mut Apk, member: &str, cells: bool) -> Result<TopologySummary, Error> {
    let bytes = apk.member_data(member)?;
    if bytes.len() > MAX_JSON_BYTES {
        return Err(invalid("topology JSON exceeds 8 MiB limit"));
    }
    let json: Value = serde_json::from_slice(&bytes)?;
    let rows = json
        .get("sceneList")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("topology sceneList is absent"))?;
    let mut names = BTreeSet::new();
    let mut nonempty_cells = 0;
    let mut fields = BTreeSet::new();
    for row in rows {
        let values = row
            .as_object()
            .ok_or_else(|| invalid("topology row is not an object"))?;
        fields.extend(values.keys().cloned());
        if cells {
            for key in ["xPos", "zPos"] {
                if !values
                    .get(key)
                    .and_then(Value::as_f64)
                    .is_some_and(f64::is_finite)
                {
                    return Err(invalid("topology cell coordinate is absent/non-finite"));
                }
            }
            let scenes = values
                .get("smallSceneList")
                .and_then(Value::as_array)
                .ok_or_else(|| invalid("smallSceneList is absent"))?;
            nonempty_cells += usize::from(!scenes.is_empty());
            for scene in scenes {
                let name = scene
                    .as_str()
                    .or_else(|| scene.get("scenename").and_then(Value::as_str))
                    .ok_or_else(|| invalid("topology scene name is absent"))?;
                names.insert(name.to_owned());
            }
        } else {
            let name = values
                .get("scenename")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid("scene name is absent"))?;
            names.insert(name.to_owned());
        }
    }
    Ok(TopologySummary {
        source: identity(member, &bytes),
        rows: rows.len(),
        nonempty_cells,
        referenced_scene_names: names.into_iter().collect(),
        sample_row_fields: fields.into_iter().collect(),
        authority: "cell_to_scene_inventory_only_not_object_placements",
    })
}

#[derive(Debug, Serialize)]
pub struct BundleSummary {
    pub source: MemberIdentity,
    pub serialized_files: usize,
    pub class_counts: BTreeMap<i32, usize>,
}

#[derive(Debug, Serialize)]
pub struct PayloadSummary {
    pub source: MemberIdentity,
    pub decoded_layout: bool,
    pub error: Option<String>,
    pub counts: BTreeMap<String, usize>,
    pub first_record_spans: Vec<Value>,
    pub runtime_role_verified: bool,
}

fn payload_summary(apk: &mut Apk, member: &str) -> Result<PayloadSummary, Error> {
    let bytes = apk.member_data(member)?;
    let source = identity(member, &bytes);
    let decoded = match decode_scene_info(bytes) {
        Ok(decoded) => decoded,
        Err(error) => {
            return Ok(PayloadSummary {
                source,
                decoded_layout: false,
                error: Some(error.to_string()),
                counts: BTreeMap::new(),
                first_record_spans: Vec::new(),
                runtime_role_verified: false,
            });
        }
    };
    // Ensure the complete retained payload agrees with its root span. Individual
    // records retain exact spans in the decoded DTO even though this report is bounded.
    if decoded.bytes_for(&decoded.scene.source)?.len() != source.byte_count {
        return Err(invalid("decoded source span differs from complete payload"));
    }
    let scene = &decoded.scene.value;
    let mut counts = BTreeMap::from([
        ("prefabs".into(), scene.prefabs.len()),
        ("cells".into(), scene.cells.len()),
        ("big_size_objects".into(), scene.big_size_objects.len()),
        ("occlusion_groups".into(), scene.occlusion_groups.len()),
        ("grass".into(), 0),
        ("trees".into(), 0),
        ("stealth_grass".into(), 0),
        ("things_no_lightmap".into(), 0),
        ("lightmapped_objects".into(), 0),
    ]);
    let mut first_record_spans = Vec::new();
    for cell in &scene.cells {
        for (key, count) in [
            ("grass", cell.value.grass.len()),
            ("trees", cell.value.trees.len()),
            ("stealth_grass", cell.value.stealth_grass.len()),
            ("things_no_lightmap", cell.value.things_no_lightmap.len()),
        ] {
            *counts.entry(key.into()).or_default() += count;
        }
        for tree in &cell.value.trees {
            if first_record_spans.len() < 16 {
                first_record_spans.push(json!({
                    "kind": "TreeObjectInfo", "source": tree.source,
                    "position": tree.value.object.transform.position,
                    "scale": tree.value.object.transform.scale,
                    "angles": tree.value.object.transform.angles,
                    "prefab_index": tree.value.object.prefab_index,
                    "prefab_name": cell.value.prefabs[tree.value.object.prefab_index as usize].value,
                    "tree_type": tree.value.tree_type, "id": tree.value.id,
                }));
            }
        }
    }
    for group in &scene.occlusion_groups {
        *counts.entry("lightmapped_objects".into()).or_default() +=
            group.value.waikes.len() + group.value.kuangjias.len() + group.value.wujians.len();
    }
    Ok(PayloadSummary {
        source,
        decoded_layout: true,
        error: None,
        counts,
        first_record_spans,
        runtime_role_verified: false,
    })
}

#[derive(Debug, Serialize)]
pub struct WorldDiscovery {
    pub schema_version: u32,
    pub source_apk: SourceApk,
    pub apk_member_count: usize,
    pub direct_sceneinfo_members: Vec<String>,
    pub file_list: MemberIdentity,
    pub file_list_sceneinfo_paths: Vec<String>,
    pub metadata_literals: MetadataSummary,
    pub topology: Vec<TopologySummary>,
    pub inspected_bundles: Vec<BundleSummary>,
    pub selected_payloads: Vec<PayloadSummary>,
    pub source_reader_files: BTreeMap<String, String>,
    pub payload_byte_limit: usize,
    pub authenticated_placements: usize,
    pub world_package_published: bool,
    pub limitations: Vec<String>,
}

pub fn discover(apk_path: &Path, selected: &[String]) -> Result<WorldDiscovery, Error> {
    if selected.len() > MAX_SELECTED_MEMBERS {
        return Err(invalid(
            "world discovery permits at most 8 selected payloads",
        ));
    }
    let source = source_apk(apk_path)?;
    let mut apk = Apk::with_limits(
        apk_path,
        BundleLimits {
            member_bytes: 32 * 1024 * 1024,
            ..Default::default()
        },
    )?;
    let members = apk.members()?;
    let direct_sceneinfo_members = members
        .iter()
        .filter(|member| member.to_ascii_lowercase().contains("sceneinfo"))
        .cloned()
        .collect();
    let file_list_name = "assets/version/FileList.txt";
    let bytes = apk.member_data(file_list_name)?;
    if bytes.len() > MAX_JSON_BYTES {
        return Err(invalid("FileList JSON exceeds 8 MiB limit"));
    }
    let file_list: Value = serde_json::from_slice(&bytes)?;
    let files = file_list
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("version FileList files is absent"))?;
    let mut listed = BTreeSet::new();
    for file in files {
        let path = file
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("version FileList path is absent"))?;
        if path.to_ascii_lowercase().contains("sceneinfo") {
            listed.insert(path.to_owned());
        }
    }
    let topology = vec![
        topology(&mut apk, "assets/gameconfig/allscenelist.obj", false)?,
        topology(&mut apk, "assets/gameconfig/scenecell.obj", true)?,
    ];
    let metadata_literals = metadata_summary(&mut apk)?;
    let mut inspected_bundles = Vec::new();
    for member in ["assets/scene/a-01.ab", "assets/StreamingAssets"] {
        let bytes = apk.member_data(member)?;
        let source = identity(member, &bytes);
        let bundle = parse_bundle(bytes, BundleLimits::default())?;
        let mut class_counts = BTreeMap::new();
        for asset in &bundle.assets {
            for object in &asset.objects {
                *class_counts.entry(object.type_id).or_default() += 1;
            }
        }
        inspected_bundles.push(BundleSummary {
            source,
            serialized_files: bundle.assets.len(),
            class_counts,
        });
    }
    let mut selected_payloads = Vec::new();
    for member in selected {
        selected_payloads.push(payload_summary(&mut apk, member)?);
    }
    let mut source_reader_files = BTreeMap::new();
    for name in [
        "SceneObject.cs",
        "TreeObject.cs",
        "BuildingObject.cs",
        "GrassObject.cs",
        "SmallSceneMgr.cs",
        "gs.battle.scmsg/SceneInfo.cs",
        "gs.battle.scmsg/CellSceneInfo.cs",
        "gs.battle.scmsg/GameObjectInfo.cs",
        "gs.battle.scmsg/TreeObjectInfo.cs",
        "gs.battle.scmsg/GrassInfo.cs",
        "gs.battle.scmsg/StealthGrassMapObjectInfo.cs",
        "gs.battle.scmsg/GameObjectWithLightmap.cs",
        "gs.battle.scmsg/OcclusionGroup.cs",
        "gs.battle.scmsg/Vec3.cs",
        "gs.battle.scmsg/Vec4.cs",
        "gs.battle.scmsg/ByteVec2.cs",
    ] {
        let path = format!("export/{name}");
        source_reader_files.insert(path.clone(), file_sha256(Path::new(&path))?);
    }
    let after = source_apk(apk_path)?;
    if after.sha256 != source.sha256 || after.byte_count != source.byte_count {
        return Err(invalid("APK changed during world discovery"));
    }
    Ok(WorldDiscovery {
        schema_version: 1, source_apk: source, apk_member_count: members.len(), direct_sceneinfo_members,
        file_list: identity(file_list_name, &bytes), file_list_sceneinfo_paths: listed.into_iter().collect(),
        metadata_literals, topology, inspected_bundles, selected_payloads, source_reader_files, payload_byte_limit: MAX_PAYLOAD_BYTES,
        authenticated_placements: 0, world_package_published: false,
        limitations: vec![
            "Search covers all APK ZIP member names and version/FileList path entries, two declared map-1 topology files, and class inventories of A-01 and StreamingAssets. It does not scan every serialized object in every bundle.".into(),
            "Scene-cell xPos/zPos and smallSceneList describe cell-to-scene loading topology; they are not prefab instance positions. Saved prefab-local transforms are also not placement evidence.".into(),
            "The strict plain SceneInfo reader follows recovered C# field/pop sequences and retains original bytes, byte spans, IDs, nonuniform scales, lightmap data and UV bytes. Encrypted layouts, IFix overrides and native consumer equivalence remain unverified.".into(),
            "A successful selected-member layout decode does not authenticate its runtime role. No world placement or transformed matrix is published without source payload/consumer/frame corroboration.".into(),
            "Selected original IL2CPP metadata literals retain their exact indices/byte spans. Their presence does not establish the loader's exact concatenation or a remote URI.".into(),
            "External downloaded/cached scene information, legacy XAPK-derived placements and the installed APK evidence copy are outside this selected source search.".into(),
        ],
    })
}
