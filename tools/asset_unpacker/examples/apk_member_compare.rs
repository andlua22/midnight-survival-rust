//! Compare every APK file payload with streaming SHA-256, independent of ZIP
//! compression, order and signing blocks. No extraction or target execution.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{Read, Seek},
    path::{Path, PathBuf},
};

use clap::Parser;
use midnight_map_data::SourceApk;
use serde::Serialize;
use sha2::{Digest, Sha256};
use zip::ZipArchive;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Members = BTreeMap<String, MemberIdentity>;

#[derive(Parser)]
#[command(about = "Compare all uncompressed APK file payloads with SHA-256")]
struct Args {
    /// The supplied source APK.
    source: PathBuf,
    /// The installed or candidate APK to compare.
    candidate: PathBuf,
    /// Save identities, differences and coverage as JSON.
    #[arg(long)]
    report: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MemberIdentity {
    byte_count: u64,
    sha256: String,
}

#[derive(Serialize)]
struct ArchiveSummary {
    apk: SourceApk,
    files: usize,
    directory_entries: usize,
    uncompressed_bytes: u64,
    /// SHA-256 of compact serde_json encoding of the ordered payload map.
    member_manifest_sha256: String,
}

#[derive(Debug, Serialize)]
struct Difference {
    member: String,
    source: Option<MemberIdentity>,
    candidate: Option<MemberIdentity>,
}

fn invalid(message: impl Into<String>) -> Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()).into()
}

fn member_inventory(reader: impl Read + Seek) -> Result<(Members, usize, u64), Error> {
    const MAX_FILES: usize = 100_000;
    const MAX_MEMBER_BYTES: u64 = 512 * 1024 * 1024;
    const MAX_TOTAL_BYTES: u64 = 8 * 1024 * 1024 * 1024;
    let mut archive = ZipArchive::new(reader)?;
    if archive.len() > MAX_FILES {
        return Err(invalid("APK inventory exceeds 100,000 entries"));
    }
    let mut members = BTreeMap::new();
    let mut directories = 0;
    let mut total = 0u64;
    for index in 0..archive.len() {
        let member = archive.by_index(index)?;
        if member.is_dir() {
            directories += 1;
            continue;
        }
        let name = std::str::from_utf8(member.name_raw())?.to_owned();
        let expected_size = member.size();
        if members.contains_key(&name) || expected_size > MAX_MEMBER_BYTES {
            return Err(invalid(format!(
                "duplicate or oversized APK member: {name}"
            )));
        }
        total = total
            .checked_add(expected_size)
            .filter(|size| *size <= MAX_TOTAL_BYTES)
            .ok_or_else(|| invalid("APK payload inventory exceeds eight GiB"))?;
        // Digest already implements Write. Reuse that implementation rather
        // than allocating a member buffer or writing another hashing loop.
        let mut digest = Sha256::new();
        let copied = std::io::copy(&mut member.take(MAX_MEMBER_BYTES + 1), &mut digest)?;
        if copied != expected_size {
            return Err(invalid(format!("APK member length differs: {name}")));
        }
        members.insert(
            name,
            MemberIdentity {
                byte_count: copied,
                sha256: format!("{:x}", digest.finalize()),
            },
        );
    }
    Ok((members, directories, total))
}

fn read_archive(path: &Path) -> Result<(ArchiveSummary, Members), Error> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let byte_count = std::io::copy(&mut file, &mut digest)?;
    let sha256 = format!("{:x}", digest.finalize());
    file.rewind()?;
    let (members, directories, total) = member_inventory(&mut file)?;
    file.rewind()?;
    let mut after = Sha256::new();
    if std::io::copy(&mut file, &mut after)? != byte_count
        || format!("{:x}", after.finalize()) != sha256
    {
        return Err(invalid("APK identity changed during payload comparison"));
    }
    let summary = ArchiveSummary {
        apk: SourceApk {
            path: path.display().to_string(),
            sha256,
            byte_count,
        },
        files: members.len(),
        directory_entries: directories,
        uncompressed_bytes: total,
        member_manifest_sha256: format!("{:x}", Sha256::digest(serde_json::to_vec(&members)?)),
    };
    Ok((summary, members))
}

fn differences(source: &Members, candidate: &Members) -> Vec<Difference> {
    source
        .keys()
        .chain(candidate.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|name| source.get(*name) != candidate.get(*name))
        .map(|name| Difference {
            member: name.clone(),
            source: source.get(name).cloned(),
            candidate: candidate.get(name).cloned(),
        })
        .collect()
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    if args.report.exists() {
        let report = args.report.canonicalize()?;
        if report == args.source.canonicalize()? || report == args.candidate.canonicalize()? {
            return Err(invalid("comparison report must not overwrite an input APK"));
        }
    }
    let (source, source_members) = read_archive(&args.source)?;
    let (candidate, candidate_members) = read_archive(&args.candidate)?;
    let differences = differences(&source_members, &candidate_members);
    let matched = source_members
        .iter()
        .filter(|(name, identity)| candidate_members.get(*name) == Some(*identity))
        .count();
    let critical_members: Vec<_> = [
        "AndroidManifest.xml",
        "classes.dex",
        "lib/arm64-v8a/libil2cpp.so",
        "lib/arm64-v8a/libunity.so",
        "assets/bin/Data/Managed/Metadata/global-metadata.dat",
        "assets/bin/Data/level0",
        "assets/bin/Data/level1",
        "assets/cfg/cfg.MapCfg.oc",
        "assets/ui/battleminimap.ab",
        "assets/ui/battlemappanel.ab",
        "assets/ui/battlepanel.ab",
    ]
    .into_iter()
    .map(|name| {
        let source = source_members.get(name);
        let candidate = candidate_members.get(name);
        serde_json::json!({"member":name,"source":source,"candidate":candidate,
            "matched":source.is_some() && source == candidate})
    })
    .collect();
    let report = serde_json::json!({
        "evidence_type":"rust_streaming_apk_payload_comparison",
        "source":source,"candidate":candidate,
        "matched_files":matched,"identical_file_payloads":differences.is_empty(),
        "differences":differences,"critical_members":critical_members,
        "limitations":[
            "Every regular ZIP member is decoded, length-checked and hashed. Directory entries, ZIP order/compression metadata and APK signing blocks are outside payload equality.",
            "Both whole-file SHA-256 identities were checked before and after member inventory.",
            "Payload equality does not establish loaded ABI, IFix/downloaded asset state, successful initialization, font fallback, or live UI behavior.",
            "This is an installed Rust-tool observation, not REA runtime Evidence or an APK screenshot."
        ]
    });
    if let Some(parent) = args
        .report
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    serde_json::to_writer_pretty(File::create(&args.report)?, &report)?;
    println!(
        "APK payload comparison: {matched} matched files, {} differences; report {}",
        differences.len(),
        args.report.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    fn fixture(files: &[(&str, &[u8])], compression: CompressionMethod) -> Vec<u8> {
        let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in files {
            archive
                .start_file(
                    *name,
                    SimpleFileOptions::default().compression_method(compression),
                )
                .unwrap();
            archive.write_all(bytes).unwrap();
        }
        archive.finish().unwrap().into_inner()
    }

    #[test]
    fn compression_and_member_order_do_not_change_payload_identity() {
        let first = fixture(
            &[("a", b"same"), ("b", b"other")],
            CompressionMethod::Stored,
        );
        let second = fixture(
            &[("b", b"other"), ("a", b"same")],
            CompressionMethod::Deflated,
        );
        assert_ne!(first, second);
        let (first, _, _) = member_inventory(Cursor::new(first)).unwrap();
        let (second, _, _) = member_inventory(Cursor::new(second)).unwrap();
        assert!(differences(&first, &second).is_empty());
    }

    #[test]
    fn equal_length_changes_missing_files_and_added_files_are_reported() {
        let first = fixture(
            &[("same", b"ok"), ("changed", b"abc"), ("gone", b"x")],
            CompressionMethod::Stored,
        );
        let second = fixture(
            &[("same", b"ok"), ("changed", b"abd"), ("added", b"x")],
            CompressionMethod::Stored,
        );
        let (first, _, _) = member_inventory(Cursor::new(first)).unwrap();
        let (second, _, _) = member_inventory(Cursor::new(second)).unwrap();
        let differences = differences(&first, &second);
        assert_eq!(
            differences
                .iter()
                .map(|row| row.member.as_str())
                .collect::<Vec<_>>(),
            ["added", "changed", "gone"]
        );
        assert!(differences[0].source.is_none());
        assert!(differences[2].candidate.is_none());
        assert_eq!(
            differences[1].source.as_ref().unwrap().byte_count,
            differences[1].candidate.as_ref().unwrap().byte_count
        );
    }

    #[test]
    fn damaged_member_crc_fails_instead_of_producing_equality_evidence() {
        let mut bytes = fixture(&[("member", b"content")], CompressionMethod::Stored);
        let start = ZipArchive::new(Cursor::new(&bytes))
            .unwrap()
            .by_name("member")
            .unwrap()
            .data_start() as usize;
        bytes[start] ^= 1;
        assert!(member_inventory(Cursor::new(bytes)).is_err());
    }
}
