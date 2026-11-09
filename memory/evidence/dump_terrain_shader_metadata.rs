//! Exact metadata and compressed-program evidence for the two terrain shaders.
//! Uses the already installed unity-asset-binary, unity-asset-core, zip and
//! serde_json crates. Only assets/shader/shaders.ab and the two known PathIDs
//! are read; no shader or application code is executed.

use std::{fs, fs::File, io::Read, path::Path};
use unity_asset_binary::{
    bundle::load_bundle_from_memory,
    compression::{CompressionType, decompress},
    typetree::{TypeTreeParseMode, TypeTreeParseOptions},
};
use unity_asset_core::UnityValue;

type Error = Box<dyn std::error::Error + Send + Sync>;

fn main() -> Result<(), Error> {
    let arguments: Vec<_> = std::env::args().collect();
    if arguments.len() != 3 {
        return Err("usage: dump-terrain-shader-metadata <apk> <output-directory>".into());
    }
    let output = Path::new(&arguments[2]);
    fs::create_dir_all(output)?;
    let member = "assets/shader/shaders.ab";
    let mut archive = zip::ZipArchive::new(File::open(&arguments[1])?)?;
    let mut bytes = Vec::new();
    archive.by_name(member)?.read_to_end(&mut bytes)?;
    let bundle = load_bundle_from_memory(bytes)?;
    let mut matches = 0;
    for (asset_index, asset) in bundle.assets.iter().enumerate() {
        for info in &asset.objects {
            let layers = match (info.type_id, info.path_id) {
                (48, -1302531423149963743) => "4",
                (48, 8944411127715196945) => "3",
                _ => continue,
            };
            let handle = asset
                .find_object_handle(info.path_id)
                .ok_or("missing shader object")?;
            let object = handle.read_with_options(TypeTreeParseOptions {
                mode: TypeTreeParseMode::Strict,
            })?;
            let properties = object.class.properties();
            let mut metadata = serde_json::json!({
                "apk": arguments[1], "bundle": member, "cab": bundle.asset_names[asset_index],
                "unity_version": asset.unity_version, "path_id": info.path_id.to_string(), "byte_count": info.byte_size,
            });
            for field in [
                "m_Name",
                "m_ParsedForm",
                "platforms",
                "offsets",
                "compressedLengths",
                "decompressedLengths",
                "stageCounts",
                "m_ShaderIsBaked",
            ] {
                let value = properties
                    .get(field)
                    .ok_or_else(|| format!("missing {field}"))?;
                metadata[field] = serde_json::to_value(value)?;
            }
            serde_json::to_writer_pretty(
                File::create(output.join(format!("terrain-{layers}-shader-metadata.json")))?,
                &metadata,
            )?;
            let compressed = match properties
                .get("compressedBlob")
                .ok_or("missing compressedBlob")?
            {
                UnityValue::Bytes(bytes) => bytes.to_owned(),
                UnityValue::Array(values) => values
                    .iter()
                    .map(|value| {
                        value
                            .as_i64()
                            .and_then(|integer| u8::try_from(integer).ok())
                            .ok_or("invalid compressed byte")
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                _ => return Err("unsupported compressedBlob type".into()),
            };
            let expected = properties
                .get("decompressedLengths")
                .and_then(UnityValue::as_array)
                .and_then(|values| values.first())
                .and_then(UnityValue::as_array)
                .and_then(|values| values.first())
                .and_then(UnityValue::as_i64)
                .and_then(|integer| usize::try_from(integer).ok())
                .ok_or("invalid decompressed length")?;
            // This target was verified to have one platform-9 block at offset0.
            // Preserve both compressed bytes and decompressed output for hashes.
            fs::write(
                output.join(format!("terrain-{layers}-compressed-programs.lz4")),
                &compressed,
            )?;
            let program = decompress(&compressed, CompressionType::Lz4, expected)?;
            if program.len() != expected {
                return Err("decompressed size mismatch".into());
            }
            fs::write(
                output.join(format!("terrain-{layers}-subprograms.bin")),
                &program,
            )?;
            println!(
                "terrain-{layers}: strict metadata, {} compressed bytes, {} decompressed bytes",
                compressed.len(),
                program.len()
            );
            matches += 1;
        }
    }
    if matches != 2 {
        return Err(format!("expected two shaders, found {matches}").into());
    }
    Ok(())
}
