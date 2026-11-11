//! Bounded, deterministic CAB indexing. Failed scans never replace a usable index.

use std::{
    collections::BTreeMap,
    fs::File,
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use byteorder::{ByteOrder, LittleEndian};
use midnight_map_data::SourceApk;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    fields::invalid,
    unity::{Apk, BundleLimits, Error, cab_name, source_apk},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    pub cabs: BTreeMap<String, String>,
    pub cab_assets: BTreeMap<String, AssetEntry>,
    /// Only unique bare PathIDs are retained. Resolution must use CAB identities.
    pub assets: BTreeMap<String, AssetEntry>,
    #[serde(skip)]
    origins: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct ScanOptions {
    pub workers: usize,
    pub limits: BundleLimits,
    pub members: Vec<String>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            workers: 2,
            limits: BundleLimits::default(),
            members: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ScanFailure {
    pub bundle: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct IndexReport {
    pub schema_version: u32,
    pub source_apk: SourceApk,
    pub workers: usize,
    pub max_member_bytes: u64,
    pub max_decompressed_bytes: usize,
    pub total_source_bundles: usize,
    pub selected_bundles: usize,
    pub scanned_bundles: usize,
    pub class_counts: BTreeMap<i32, usize>,
    pub failures: Vec<ScanFailure>,
    pub ambiguous_path_ids: BTreeMap<String, Vec<String>>,
    pub complete: bool,
    pub limitations: Vec<String>,
}

fn parse_string_prefix(raw: &[u8]) -> Option<(String, usize)> {
    let length = usize::try_from(LittleEndian::read_u32(raw.get(..4)?)).ok()?;
    if length == 0 || length > 256 {
        return None;
    }
    let name = std::str::from_utf8(raw.get(4..4 + length)?)
        .ok()?
        .to_owned();
    Some((name, 4 + length + (4 - length % 4) % 4))
}

fn parse_sprite_metadata(raw: &[u8]) -> (Option<[f32; 4]>, Option<[f32; 4]>) {
    let Some((_, offset)) = parse_string_prefix(raw) else {
        return (None, None);
    };
    let Some(bytes) = raw.get(offset..offset + 40) else {
        return (None, None);
    };
    let rect: [f32; 4] =
        std::array::from_fn(|index| LittleEndian::read_f32(&bytes[index * 4..index * 4 + 4]));
    let border: [f32; 4] =
        std::array::from_fn(|index| LittleEndian::read_f32(&bytes[24 + index * 4..28 + index * 4]));
    if rect.iter().chain(&border).any(|value| !value.is_finite()) {
        return (None, None);
    }
    (Some(rect), Some(border))
}

struct ScannedBundle {
    member: String,
    cabs: Vec<String>,
    entries: Vec<(String, String, AssetEntry)>,
    classes: BTreeMap<i32, usize>,
}

fn scan_bundle(apk: &mut Apk, member: &str) -> Result<ScannedBundle, Error> {
    let bundle = apk.bundle(member)?;
    let mut result = ScannedBundle {
        member: member.into(),
        cabs: Vec::new(),
        entries: Vec::new(),
        classes: BTreeMap::new(),
    };
    for (asset_index, asset) in bundle.assets.iter().enumerate() {
        let full = bundle
            .asset_names
            .get(asset_index)
            .ok_or_else(|| invalid("missing serialized file name"))?;
        let cab = cab_name(full);
        if cab.is_empty() {
            return Err(invalid("empty serialized file identity"));
        }
        result.cabs.push(cab.into());
        if full != cab {
            result.cabs.push(full.clone());
        }
        for object in &asset.objects {
            *result.classes.entry(object.type_id).or_default() += 1;
            if !matches!(object.type_id, 21 | 28 | 43 | 48 | 74 | 90 | 91 | 128 | 213) {
                continue;
            }
            let raw = asset.object_bytes(object)?;
            let name = parse_string_prefix(raw).map_or_else(String::new, |(name, _)| name);
            let (rect, border) = if object.type_id == 213 {
                parse_sprite_metadata(raw)
            } else {
                (None, None)
            };
            result.entries.push((
                format!("{cab}:{}", object.path_id),
                object.path_id.to_string(),
                AssetEntry {
                    name,
                    class_id: object.type_id,
                    bundle: member.into(),
                    rect,
                    border,
                },
            ));
        }
    }
    Ok(result)
}

fn merge(index: &mut GlobalIndex, report: &mut IndexReport, scanned: ScannedBundle) {
    for (class, count) in scanned.classes {
        *report.class_counts.entry(class).or_default() += count;
    }
    for cab in scanned.cabs {
        if let Some(previous) = index.cabs.get(&cab)
            && previous != &scanned.member
        {
            report.failures.push(ScanFailure {
                bundle: scanned.member.clone(),
                detail: format!(
                    "CAB {cab} also belongs to {previous}; external resolution is ambiguous"
                ),
            });
            continue;
        }
        index.cabs.insert(cab, scanned.member.clone());
    }
    for (qualified, path_id, entry) in scanned.entries {
        if let Some(previous) = index.cab_assets.get(&qualified) {
            if previous != &entry {
                report.failures.push(ScanFailure {
                    bundle: scanned.member.clone(),
                    detail: format!("conflicting qualified object {qualified}"),
                });
            }
            continue;
        }
        if let Some(owners) = report.ambiguous_path_ids.get_mut(&path_id) {
            if !owners.contains(&qualified) {
                owners.push(qualified.clone());
            }
        } else if let Some(previous) = index.origins.remove(&path_id) {
            index.assets.remove(&path_id);
            report
                .ambiguous_path_ids
                .insert(path_id.clone(), vec![previous, qualified.clone()]);
        } else {
            index.origins.insert(path_id.clone(), qualified.clone());
            index.assets.insert(path_id.clone(), entry.clone());
        }
        index.cab_assets.insert(qualified, entry);
    }
    report.scanned_bundles += 1;
}

fn publish_json(path: &Path, value: &impl Serialize) -> Result<(), Error> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temporary = path.with_extension(format!("json.{stamp}.tmp"));
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    serde_json::to_writer(&mut file, value)?;
    file.sync_all()?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

pub fn run(apk: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_selected(apk, output, &ScanOptions::default(), None)
        .map(|_| ())
        .map_err(|error| error as Box<dyn std::error::Error>)
}

pub fn run_selected(
    apk_path: &Path,
    output: &Path,
    options: &ScanOptions,
    report_path: Option<&Path>,
) -> Result<IndexReport, Error> {
    if !(1..=8).contains(&options.workers) {
        return Err(invalid("index workers must be between 1 and 8"));
    }
    let started = Instant::now();
    let mut apk = Apk::with_limits(apk_path, options.limits)?;
    let mut members = apk
        .members()?
        .into_iter()
        .filter(|name| name.ends_with(".ab"))
        .collect::<Vec<_>>();
    let total_source_bundles = members.len();
    if total_source_bundles == 0 {
        return Err(invalid("APK contains no .ab bundles; index not replaced"));
    }
    if !options.members.is_empty() {
        for name in &options.members {
            if !members.contains(name) {
                return Err(invalid(format!("selected bundle is absent: {name}")));
            }
        }
        members.retain(|name| options.members.contains(name));
    }
    let mut report = IndexReport {
        schema_version: 1, source_apk: source_apk(apk_path)?, workers: options.workers,
        max_member_bytes: options.limits.member_bytes, max_decompressed_bytes: options.limits.decompressed_bytes,
        total_source_bundles, selected_bundles: members.len(), scanned_bundles: 0,
        class_counts: BTreeMap::new(), failures: Vec::new(), ambiguous_path_ids: BTreeMap::new(), complete: false,
        limitations: vec!["Names and sprite rectangles in the global index are bounded metadata hints; strict object parsing supplies authoritative fields.".into(), "Bare PathIDs are omitted when ambiguous; resolve cross-file references using the CAB-qualified index.".into()],
    };
    let mut index = GlobalIndex::default();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(options.workers)
        .build()?;
    // Bound both parsed bundles and pending index batches. Each worker opens once per chunk.
    for group in members.chunks(options.workers * 120) {
        let batches = pool.install(|| {
            group
                .par_chunks(120)
                .map(|batch| {
                    let mut reader = Apk::with_limits(apk_path, options.limits);
                    batch
                        .iter()
                        .map(|member| match &mut reader {
                            Ok(reader) => {
                                scan_bundle(reader, member).map_err(|error| ScanFailure {
                                    bundle: member.clone(),
                                    detail: error.to_string(),
                                })
                            }
                            Err(error) => Err(ScanFailure {
                                bundle: member.clone(),
                                detail: error.to_string(),
                            }),
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        });
        for result in batches.into_iter().flatten() {
            match result {
                Ok(scanned) => merge(&mut index, &mut report, scanned),
                Err(failure) => report.failures.push(failure),
            }
        }
        eprintln!(
            "Indexed {}/{} selected bundles",
            report.scanned_bundles,
            members.len()
        );
    }
    let after = source_apk(apk_path)?;
    if after.sha256 != report.source_apk.sha256 || after.byte_count != report.source_apk.byte_count
    {
        return Err(invalid("APK changed while indexing"));
    }
    report.complete =
        report.failures.is_empty() && report.scanned_bundles == report.selected_bundles;
    let imported = output.join("imported_map");
    std::fs::create_dir_all(&imported)?;
    let default_report = imported.join(if options.members.is_empty() {
        "global_asset_index.coverage.json"
    } else {
        "selected_asset_index.coverage.json"
    });
    let report_path = report_path.unwrap_or(&default_report);
    if let Some(parent) = report_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    publish_json(report_path, &report)?;
    if !report.complete {
        return Err(invalid(format!(
            "index incomplete: {} failures; previous index retained; report {}",
            report.failures.len(),
            report_path.display()
        )));
    }
    let index_path = imported.join(if options.members.is_empty() {
        "global_asset_index.json"
    } else {
        "selected_asset_index.json"
    });
    publish_json(&index_path, &index)?;
    eprintln!(
        "Indexed {} qualified assets in {:.2?} -> {}",
        index.cab_assets.len(),
        started.elapsed(),
        index_path.display()
    );
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> IndexReport {
        IndexReport {
            schema_version: 1,
            source_apk: SourceApk {
                path: String::new(),
                sha256: String::new(),
                byte_count: 0,
            },
            workers: 1,
            max_member_bytes: 1,
            max_decompressed_bytes: 1,
            total_source_bundles: 2,
            selected_bundles: 2,
            scanned_bundles: 0,
            class_counts: BTreeMap::new(),
            failures: vec![],
            ambiguous_path_ids: BTreeMap::new(),
            complete: false,
            limitations: vec![],
        }
    }

    #[test]
    fn bare_path_collisions_are_omitted_and_qualified_entries_survive() {
        let mut index = GlobalIndex::default();
        let mut coverage = report();
        for cab in ["CAB-a", "CAB-b", "CAB-c"] {
            merge(
                &mut index,
                &mut coverage,
                ScannedBundle {
                    member: format!("{cab}.ab"),
                    cabs: vec![cab.into()],
                    entries: vec![(
                        format!("{cab}:42"),
                        "42".into(),
                        AssetEntry {
                            name: cab.into(),
                            class_id: 43,
                            bundle: format!("{cab}.ab"),
                            rect: None,
                            border: None,
                        },
                    )],
                    classes: BTreeMap::new(),
                },
            );
        }
        assert!(!index.assets.contains_key("42"));
        assert_eq!(index.cab_assets.len(), 3);
        assert_eq!(coverage.ambiguous_path_ids["42"].len(), 3);
        assert!(coverage.failures.is_empty());
    }

    #[test]
    fn duplicate_cab_ownership_cannot_silently_replace_resolution() {
        let mut index = GlobalIndex::default();
        let mut coverage = report();
        for member in ["a.ab", "b.ab"] {
            merge(
                &mut index,
                &mut coverage,
                ScannedBundle {
                    member: member.into(),
                    cabs: vec!["CAB-shared".into()],
                    entries: vec![],
                    classes: BTreeMap::new(),
                },
            );
        }
        assert_eq!(index.cabs["CAB-shared"], "a.ab");
        assert_eq!(coverage.failures.len(), 1);
    }

    #[test]
    fn failed_bundle_scan_reports_coverage_and_preserves_previous_index() {
        use std::io::Write;
        let root = std::env::temp_dir().join(format!(
            "midnight-index-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("imported_map")).unwrap();
        let apk = root.join("source.apk");
        let mut archive = zip::ZipWriter::new(File::create(&apk).unwrap());
        archive
            .start_file(
                "assets/scene/bad.ab",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        archive.write_all(b"invalid Unity bundle").unwrap();
        archive.finish().unwrap();
        std::fs::write(
            root.join("imported_map/global_asset_index.json"),
            b"previous index",
        )
        .unwrap();
        assert!(run_selected(&apk, &root, &ScanOptions::default(), None).is_err());
        assert_eq!(
            std::fs::read(root.join("imported_map/global_asset_index.json")).unwrap(),
            b"previous index"
        );
        let report: serde_json::Value = serde_json::from_reader(
            File::open(root.join("imported_map/global_asset_index.coverage.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(report["complete"], false);
        assert_eq!(report["failures"][0]["bundle"], "assets/scene/bad.ab");
        assert_eq!(report["scanned_bundles"], 0);
        std::fs::remove_dir_all(root).unwrap();
    }
}
