use byteorder::{ByteOrder, LittleEndian};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use unity_asset_binary::bundle::load_bundle_from_memory;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetEntry {
    pub name: String,
    pub class_id: i32,
    pub bundle: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rect: Option<[f32; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<[f32; 4]>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GlobalIndex {
    pub cabs: HashMap<String, String>,
    pub cab_assets: HashMap<String, AssetEntry>,
    pub assets: HashMap<String, AssetEntry>,
}

fn parse_string_prefix(raw: &[u8]) -> Option<(String, usize)> {
    if raw.len() < 4 {
        return None;
    }
    let nlen = LittleEndian::read_u32(&raw[0..4]) as usize;
    if nlen == 0 || nlen > 256 || 4 + nlen > raw.len() {
        return None;
    }
    let s = String::from_utf8_lossy(&raw[4..4 + nlen]).to_string();
    let pad = (4 - (nlen % 4)) % 4;
    Some((s, 4 + nlen + pad))
}

fn parse_sprite_metadata(raw: &[u8]) -> (Option<[f32; 4]>, Option<[f32; 4]>) {
    if let Some((_, aligned)) = parse_string_prefix(raw) {
        if aligned + 40 <= raw.len() {
            let x = LittleEndian::read_f32(&raw[aligned..aligned + 4]);
            let y = LittleEndian::read_f32(&raw[aligned + 4..aligned + 8]);
            let w = LittleEndian::read_f32(&raw[aligned + 8..aligned + 12]);
            let h = LittleEndian::read_f32(&raw[aligned + 12..aligned + 16]);

            // skip offset (8 bytes)
            let b_start = aligned + 24;
            let bx = LittleEndian::read_f32(&raw[b_start..b_start + 4]);
            let by = LittleEndian::read_f32(&raw[b_start + 4..b_start + 8]);
            let bz = LittleEndian::read_f32(&raw[b_start + 8..b_start + 12]);
            let bw = LittleEndian::read_f32(&raw[b_start + 12..b_start + 16]);
            return (Some([x, y, w, h]), Some([bx, by, bz, bw]));
        }
    }
    (None, None)
}

pub fn run(apk_path: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=======================================================");
    println!("PASS 1: Native Rust Global PathID Index across ALL bundles");
    println!("=======================================================");
    let t0 = Instant::now();

    let apk_file = File::open(apk_path)?;
    let mut zip = zip::ZipArchive::new(apk_file)?;

    let mut bundle_names = Vec::new();
    for i in 0..zip.len() {
        let file = zip.by_index_raw(i)?;
        let name = file.name().to_string();
        if name.ends_with(".ab") {
            bundle_names.push(name);
        }
    }
    bundle_names.sort();
    println!("Total .ab bundles to scan in APK: {}", bundle_names.len());

    let global_cabs = Mutex::new(HashMap::new());
    let global_cab_assets = Mutex::new(HashMap::new());
    let global_assets = Mutex::new(HashMap::new());

    // Process in batches across threads using ZipArchive re-opens
    let batch_size = 120;
    let chunks: Vec<_> = bundle_names.chunks(batch_size).collect();
    println!("Distributing {} batches across Rayon threads...", chunks.len());

    chunks.par_iter().for_each(|batch| {
        let Ok(file) = File::open(apk_path) else { return };
        let Ok(mut thread_zip) = zip::ZipArchive::new(file) else { return };

        let mut local_cabs = HashMap::new();
        let mut local_cab_assets = HashMap::new();
        let mut local_assets = HashMap::new();
        let mut buffer = Vec::with_capacity(512 * 1024);

        for bname in batch.iter() {
            buffer.clear();
            let Ok(mut zf) = thread_zip.by_name(bname.as_str()) else { continue };
            if zf.read_to_end(&mut buffer).is_err() {
                continue;
            }

            let Ok(bundle) = load_bundle_from_memory(buffer.clone()) else { continue };
            for (asset_idx, asset) in bundle.assets.iter().enumerate() {
                let cab_full = bundle.asset_names.get(asset_idx).cloned().unwrap_or_default();
                let cab_name = cab_full.split('/').last().unwrap_or(&cab_full).to_string();
                local_cabs.insert(cab_name.clone(), bname.to_string());
                if !cab_full.is_empty() && cab_full != cab_name {
                    local_cabs.insert(cab_full.clone(), bname.to_string());
                }

                for obj in &asset.objects {
                    // Filter common asset types of interest
                    // 21: Material, 28: Texture2D, 43: Mesh, 213: Sprite, 48: Shader, 128: Font
                    if matches!(obj.type_id, 21 | 28 | 43 | 48 | 128 | 213) {
                        let Ok(raw) = asset.object_bytes(obj) else { continue };
                        let (name, (rect, border)) = if let Some((nm, _)) = parse_string_prefix(raw) {
                            let (r, b) = if obj.type_id == 213 {
                                parse_sprite_metadata(raw)
                            } else {
                                (None, None)
                            };
                            (nm, (r, b))
                        } else {
                            (String::new(), (None, None))
                        };

                        let entry = AssetEntry {
                            name,
                            class_id: obj.type_id,
                            bundle: bname.to_string(),
                            rect,
                            border,
                        };

                        local_cab_assets.insert(format!("{}:{}", cab_name, obj.path_id), entry.clone());
                        local_assets.insert(obj.path_id.to_string(), entry);
                    }
                }
            }
        }

        global_cabs.lock().unwrap().extend(local_cabs);
        global_cab_assets.lock().unwrap().extend(local_cab_assets);
        global_assets.lock().unwrap().extend(local_assets);
    });

    let cabs = global_cabs.into_inner().unwrap();
    let cab_assets = global_cab_assets.into_inner().unwrap();
    let assets = global_assets.into_inner().unwrap();

    let index = GlobalIndex {
        cabs,
        cab_assets,
        assets,
    };

    let imported_map_dir = out_dir.join("imported_map");
    std::fs::create_dir_all(&imported_map_dir)?;
    let index_path = imported_map_dir.join("global_asset_index.json");

    let f = File::create(&index_path)?;
    serde_json::to_writer(&f, &index)?;

    let size_mb = std::fs::metadata(&index_path)?.len() as f64 / (1024.0 * 1024.0);
    println!(
        "Pass 1 Complete in {:.2?}! Indexed {} assets, {} qualified CAB assets, {} CABs -> {} ({:.2} MB)",
        t0.elapsed(),
        index.assets.len(),
        index.cab_assets.len(),
        index.cabs.len(),
        index_path.display(),
        size_mb
    );
    Ok(())
}
