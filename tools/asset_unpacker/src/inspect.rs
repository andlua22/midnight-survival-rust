use std::{collections::BTreeMap, fs::File, path::Path};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use unity_asset_core::UnityValue;

use crate::unity::{Apk, cab_name, read_object};

/// Bounded previews are diagnostic only. Extraction always reads complete objects.
fn preview(value: &UnityValue) -> Value {
    match value {
        UnityValue::Bytes(bytes) => json!({
            "byte_count": bytes.len(),
            "sha256": format!("{:x}", Sha256::digest(bytes)),
            "preview": &bytes[..bytes.len().min(16)],
        }),
        UnityValue::Array(items) if items.len() > 16 => json!({
            "item_count": items.len(),
            "preview": items.iter().take(16).map(preview).collect::<Vec<_>>(),
            "truncated": true,
        }),
        UnityValue::Array(items) => Value::Array(items.iter().map(preview).collect()),
        UnityValue::Object(properties) => Value::Object(
            properties.iter().map(|(name, value)| (name.clone(), preview(value))).collect(),
        ),
        _ => serde_json::to_value(value).expect("Unity scalar values serialize to JSON"),
    }
}

pub fn run(
    apk: &Path,
    member: &str,
    classes: &[i32],
    report: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = Apk::open(apk).map_err(|error| error as Box<dyn std::error::Error>)?;
    let bundle = archive.bundle(member).map_err(|error| error as Box<dyn std::error::Error>)?;
    let mut assets = Vec::new();
    for (index, asset) in bundle.assets.iter().enumerate() {
        let mut counts = BTreeMap::<i32, usize>::new();
        let mut objects = Vec::new();
        for info in &asset.objects {
            *counts.entry(info.type_id).or_default() += 1;
            if !classes.is_empty() && !classes.contains(&info.type_id) {
                continue;
            }
            let object = read_object(asset, info)
                .map_err(|error| error as Box<dyn std::error::Error>)?;
            objects.push(json!({
                "class_id": info.type_id,
                // Keep signed IDs losslessly, including in clients that parse JSON numbers as f64.
                "path_id": info.path_id.to_string(),
                "byte_count": info.byte_size,
                "properties": preview(&UnityValue::Object(object.class.properties().clone())),
            }));
        }
        assets.push(json!({
            "cab": cab_name(&bundle.asset_names[index]),
            "unity_version": asset.unity_version,
            "type_tree_enabled": asset.enable_type_tree,
            "class_counts": counts,
            "externals": asset.externals,
            "objects": objects,
        }));
    }
    let result = json!({"apk": apk, "bundle": member, "assets": assets});
    if let Some(path) = report {
        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        serde_json::to_writer_pretty(File::create(path)?, &result)?;
        eprintln!("Inspected {member} -> {}", path.display());
    } else {
        serde_json::to_writer_pretty(std::io::stdout().lock(), &result)?;
    }
    Ok(())
}
