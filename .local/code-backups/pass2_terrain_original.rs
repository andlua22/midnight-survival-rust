use byteorder::{ByteOrder, LittleEndian};
use image::{GenericImageView, Rgba, RgbaImage};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use unity_asset_binary::bundle::load_bundle_from_memory;

use crate::pass1_index::GlobalIndex;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChunkBiomeDef {
    pub biome: String,
    pub l0: String,
    pub l1: String,
    pub l2: String,
    pub l3: String,
    pub normal: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChunkManifestRecord {
    pub biome: String,
    pub layers: Vec<String>,
    pub normal: String,
    pub layer4_active: bool,
    pub bounds: [f32; 4],
}

fn load_chunk_bounds(bin_path: &Path) -> HashMap<String, [f32; 4]> {
    let mut bounds = HashMap::new();
    let Ok(data) = std::fs::read(bin_path) else { return bounds };
    if data.len() < 12 || &data[0..8] != b"MSMAP001" {
        return bounds;
    }
    let count = LittleEndian::read_u32(&data[8..12]) as usize;
    let mut offset = 12;
    for _ in 0..count {
        if offset + 2 > data.len() {
            break;
        }
        let nlen = LittleEndian::read_u16(&data[offset..offset + 2]) as usize;
        offset += 2;
        let name = String::from_utf8_lossy(&data[offset..offset + nlen]).to_string();
        offset += nlen;
        let tlen = LittleEndian::read_u16(&data[offset..offset + 2]) as usize;
        offset += 2 + tlen;
        let _cx = LittleEndian::read_f32(&data[offset..offset + 4]);
        let _cy = LittleEndian::read_f32(&data[offset + 4..offset + 8]);
        let _cz = LittleEndian::read_f32(&data[offset + 8..offset + 12]);
        let vc = LittleEndian::read_u32(&data[offset + 12..offset + 16]) as usize;
        let ic = LittleEndian::read_u32(&data[offset + 16..offset + 20]) as usize;
        offset += 20;

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;

        for _ in 0..vc {
            if offset + 12 > data.len() {
                break;
            }
            let x = LittleEndian::read_f32(&data[offset..offset + 4]);
            let _y = LittleEndian::read_f32(&data[offset + 4..offset + 8]);
            let z = LittleEndian::read_f32(&data[offset + 8..offset + 12]);
            offset += 12;
            let bevy_z = -z;
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_z = min_z.min(bevy_z);
            max_z = max_z.max(bevy_z);
        }
        offset += vc * 20 + ic * 4;
        if !name.to_ascii_lowercase().starts_with("road") && min_x < max_x {
            bounds.insert(name.to_ascii_uppercase(), [min_x, max_x, min_z, max_z]);
        }
    }
    bounds
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

pub fn run(apk_path: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=======================================================");
    println!("PASS 2: Native Rust Terrain Chunk Biomes & Normalization");
    println!("=======================================================");
    let t0 = Instant::now();

    let imported_map_dir = out_dir.join("imported_map");
    let index_path = imported_map_dir.join("global_asset_index.json");
    if !index_path.exists() {
        println!("Index missing at {}, running Pass 1 first...", index_path.display());
        crate::pass1_index::run(apk_path, out_dir)?;
    }

    let global_index: GlobalIndex = serde_json::from_reader(File::open(&index_path)?)?;
    let cabs = &global_index.cabs;

    let terrain_bin_path = imported_map_dir.join("island_terrain.bin");
    let bounds_map = load_chunk_bounds(&terrain_bin_path);
    println!("Loaded bounds for {} terrain chunks from island_terrain.bin.", bounds_map.len());

    let apk_file = File::open(apk_path)?;
    let mut zip = zip::ZipArchive::new(apk_file)?;

    let mut scene_files = Vec::new();
    for i in 0..zip.len() {
        let file = zip.by_index_raw(i)?;
        let name = file.name().to_string();
        if name.starts_with("assets/scene/") && name.ends_with(".ab") {
            let base = name.split('/').last().unwrap().to_ascii_uppercase();
            let is_sector = (base.starts_with('A') || base.starts_with('B') || base.starts_with('C') || base.starts_with('D'))
                && base.contains('-');
            let is_mil = base.starts_with("JUNSHIJIDI_TERRAIN_");
            if is_sector || is_mil {
                scene_files.push(name);
            }
        }
    }
    scene_files.sort();
    println!("Found {} terrain chunk bundles in APK.", scene_files.len());

    let splats_dir = imported_map_dir.join("splatmaps");
    std::fs::create_dir_all(&splats_dir)?;

    let manifest_records = Mutex::new(HashMap::new());
    let biomes_records = Mutex::new(HashMap::new());

    // Process chunk bundles in parallel
    scene_files.par_iter().for_each(|sf| {
        let Ok(file) = File::open(apk_path) else { return };
        let Ok(mut thread_zip) = zip::ZipArchive::new(file) else { return };

        let mut buffer = Vec::new();
        let Ok(mut zf) = thread_zip.by_name(sf) else { return };
        if zf.read_to_end(&mut buffer).is_err() {
            return;
        }

        let cname = sf.split('/').last().unwrap().trim_end_matches(".ab").to_ascii_uppercase();
        let Ok(bundle) = load_bundle_from_memory(buffer) else { return };

        let mut layer4_active = true;
        let mut resolved_splats: HashMap<String, String> = HashMap::new();

        // Search for Material in bundle assets
        for asset in &bundle.assets {
            let exts = &asset.externals;
            for obj in &asset.objects {
                if obj.type_id == 21 { // Material
                    let Ok(raw) = asset.object_bytes(obj) else { continue };

                    // Read _Layer4
                    if let Some(idx) = find_subslice(raw, b"_Layer4\0") {
                        if idx + 12 <= raw.len() {
                            let val = LittleEndian::read_f32(&raw[idx + 8..idx + 12]);
                            if val == 0.0 {
                                layer4_active = false;
                            }
                        }
                    }

                    // Read _Splat0.._Splat3
                    for prop in &["_Splat0", "_Splat1", "_Splat2", "_Splat3"] {
                        let prop_bytes = format!("{}\0", prop).into_bytes();
                        if let Some(idx) = find_subslice(raw, &prop_bytes) {
                            if idx + 20 <= raw.len() {
                                let fid = LittleEndian::read_i32(&raw[idx + 8..idx + 12]);
                                if fid > 0 && (fid as usize - 1) < exts.len() {
                                    let ext_path = &exts[fid as usize - 1].path;
                                    let cab = ext_path.split('/').last().unwrap();
                                    if let Some(bname) = cabs.get(cab) {
                                        let tex_file = bname.split('/').last().unwrap().replace(".ab", ".png");
                                        resolved_splats.insert(prop.to_string(), tex_file);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !resolved_splats.contains_key("_Splat3") {
            layer4_active = false;
        }

        // Clean & Normalize Splatmap
        let splat_file_path = splats_dir.join(format!("{}.png", cname));
        if splat_file_path.exists() {
            if let Ok(dyn_img) = image::open(&splat_file_path) {
                let (w, h) = dyn_img.dimensions();
                let mut out_img = RgbaImage::new(w, h);
                let rgba = dyn_img.to_rgba8();

                for y in 0..h {
                    for x in 0..w {
                        let p = rgba.get_pixel(x, y);
                        let r = p[0] as f32;
                        let g = p[1] as f32;
                        let b = p[2] as f32;
                        let mut a = if layer4_active {
                            p[3] as f32
                        } else {
                            0.0f32
                        };

                        if layer4_active && (a >= 250.0 || (r + g + b) >= 220.0) {
                            a = (255.0 - (r + g + b)).max(0.0);
                        }

                        let tot = (r + g + b + a).max(0.001);
                        let norm_r = ((r / tot) * 255.0).clamp(0.0, 255.0) as u8;
                        let norm_g = ((g / tot) * 255.0).clamp(0.0, 255.0) as u8;
                        let norm_b = ((b / tot) * 255.0).clamp(0.0, 255.0) as u8;
                        let norm_a = ((a / tot) * 255.0).clamp(0.0, 255.0) as u8;

                        out_img.put_pixel(x, y, Rgba([norm_r, norm_g, norm_b, norm_a]));
                    }
                }
                let _ = out_img.save(&splat_file_path);
            }
        }

        let bounds = bounds_map.get(&cname).copied().unwrap_or([-200.0, 200.0, -200.0, 200.0]);
        let min_x = bounds[0];

        let s0 = resolved_splats.get("_Splat0").cloned().unwrap_or_default().to_ascii_lowercase();

        let is_snow = s0.contains("xue") || s0.contains("bing");
        let is_desert = !is_snow && (s0.contains("sand") || s0.contains("gobi") || s0.contains("shamo"));
        let is_mil = !is_snow && !is_desert && (cname.starts_with("JUNSHIJIDI") || min_x < -2800.0);

        let biome_kind = if is_snow {
            "Snow"
        } else if is_desert {
            "Desert"
        } else if is_mil {
            "Military"
        } else {
            "Plains"
        };
        let normal_map = if is_snow {
            "terrain_xuedi_01_N.png"
        } else if is_desert {
            "terrain_sandA2_N.png"
        } else {
            "caodi1215_N.png"
        };

        let l0 = resolved_splats.get("_Splat0").cloned()
            .or_else(|| resolved_splats.get("_Splat1").cloned())
            .unwrap_or_else(|| "terrain_kucao_01.png".to_string());
        let l1 = resolved_splats.get("_Splat1").cloned()
            .unwrap_or_else(|| "terrain_nitu_02.png".to_string());
        let l2 = resolved_splats.get("_Splat2").cloned()
            .unwrap_or_else(|| "terrain_yanshi_01.png".to_string());
        let l3 = if layer4_active {
            resolved_splats.get("_Splat3").cloned().unwrap_or_else(|| l0.clone())
        } else {
            l0.clone()
        };

        let final_layers = vec![l0.clone(), l1.clone(), l2.clone(), l3.clone()];

        manifest_records.lock().unwrap().insert(
            cname.clone(),
            ChunkManifestRecord {
                biome: biome_kind.to_string(),
                layers: final_layers,
                normal: normal_map.to_string(),
                layer4_active,
                bounds,
            },
        );

        biomes_records.lock().unwrap().insert(
            cname,
            ChunkBiomeDef {
                biome: biome_kind.to_string(),
                l0,
                l1,
                l2,
                l3,
                normal: normal_map.to_string(),
            },
        );
    });

    let manifest = manifest_records.into_inner().unwrap();
    let biomes = biomes_records.into_inner().unwrap();

    let manifest_path = imported_map_dir.join("terrain_chunks_manifest.json");
    let biomes_path = imported_map_dir.join("chunk_biomes.json");

    let f1 = File::create(&manifest_path)?;
    serde_json::to_writer_pretty(&f1, &manifest)?;

    let f2 = File::create(&biomes_path)?;
    serde_json::to_writer_pretty(&f2, &biomes)?;

    let snow_c = biomes.values().filter(|v| v.biome == "Snow").count();
    let desert_c = biomes.values().filter(|v| v.biome == "Desert").count();
    let mil_c = biomes.values().filter(|v| v.biome == "Military").count();
    let plains_c = biomes.values().filter(|v| v.biome == "Plains").count();

    println!(
        "Pass 2 Complete in {:.2?}! Processed {} chunks ({} Snow, {} Desert, {} Military, {} Plains)",
        t0.elapsed(),
        biomes.len(),
        snow_c,
        desert_c,
        mil_c,
        plains_c
    );
    println!("Saved chunk manifest -> {}", manifest_path.display());
    println!("Saved chunk biomes    -> {}", biomes_path.display());
    Ok(())
}

