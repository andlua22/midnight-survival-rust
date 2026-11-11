use midnight_map_data::SourceApk;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs::File, io::Read, path::Path};

use unity_asset_binary::{
    asset::{ObjectInfo, SerializedFile},
    bundle::{AssetBundle, BundleLoadOptions, BundleParser},
    object::{ObjectHandle, UnityObject},
    typetree::{TypeTreeParseMode, TypeTreeParseOptions},
};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Limits apply before ZIP allocation and inside the Unity decompressor.
#[derive(Clone, Copy, Debug)]
pub struct BundleLimits {
    pub member_bytes: u64,
    pub decompressed_bytes: usize,
}

impl Default for BundleLimits {
    fn default() -> Self {
        Self {
            member_bytes: 256 * 1024 * 1024,
            decompressed_bytes: 512 * 1024 * 1024,
        }
    }
}

pub fn parse_bundle(data: Vec<u8>, limits: BundleLimits) -> Result<AssetBundle, Error> {
    if data.len() as u64 > limits.member_bytes {
        return Err(crate::fields::invalid("bundle member exceeds byte limit"));
    }
    Ok(BundleParser::from_bytes_with_options(
        data,
        BundleLoadOptions {
            max_memory: Some(limits.decompressed_bytes),
            max_unityfs_block_cache_memory: Some(limits.decompressed_bytes),
            max_compressed_block_size: Some(limits.member_bytes.min(usize::MAX as u64) as usize),
            max_compressed_blocks_info_size: Some(8 * 1024 * 1024),
            max_blocks_info_size: Some(8 * 1024 * 1024),
            max_legacy_directory_compressed_size: Some(8 * 1024 * 1024),
            max_blocks: 65_536,
            max_nodes: 65_536,
            ..Default::default()
        },
    )?)
}

/// China builds append `cN` to the usual Unity version. The texture converter
/// needs the base engine layout; retain the full build string in source evidence.
pub fn texture_version(
    source: &str,
) -> Result<unity_asset_binary::unity_version::UnityVersion, Error> {
    let base = source
        .rsplit_once('c')
        .filter(|(_, suffix)| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
        })
        .map_or(source, |(base, _)| base);
    Ok(unity_asset_binary::unity_version::UnityVersion::parse_version(base)?)
}

pub struct Apk {
    archive: zip::ZipArchive<File>,
    members: BTreeMap<String, usize>,
    limits: BundleLimits,
}

impl Apk {
    pub fn open(path: &Path) -> Result<Self, Error> {
        Self::with_limits(path, BundleLimits::default())
    }

    pub fn with_limits(path: &Path, limits: BundleLimits) -> Result<Self, Error> {
        if limits.member_bytes == 0 || limits.decompressed_bytes == 0 {
            return Err(crate::fields::invalid("bundle limits must be positive"));
        }
        let mut archive = zip::ZipArchive::new(File::open(path)?)?;
        let mut members = BTreeMap::new();
        for index in 0..archive.len() {
            let name = archive.by_index_raw(index)?.name().to_owned();
            if members.insert(name.clone(), index).is_some() {
                return Err(crate::fields::invalid(format!(
                    "duplicate APK member {name}"
                )));
            }
        }
        Ok(Self {
            archive,
            members,
            limits,
        })
    }

    pub fn members(&mut self) -> Result<Vec<String>, Error> {
        Ok(self.members.keys().cloned().collect())
    }

    pub fn bundle(&mut self, member: &str) -> Result<AssetBundle, Error> {
        let data = self.member_data(member)?;
        parse_bundle(data, self.limits).map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("{member}: {error}"),
            )
            .into()
        })
    }

    pub fn member_data(&mut self, member: &str) -> Result<Vec<u8>, Error> {
        let index = self
            .members
            .get(member)
            .ok_or_else(|| crate::fields::invalid(format!("missing APK member {member}")))?;
        let mut file = self.archive.by_index(*index)?;
        if file.size() > self.limits.member_bytes {
            return Err(crate::fields::invalid(format!(
                "{member}: ZIP member size {} exceeds limit {}",
                file.size(),
                self.limits.member_bytes
            )));
        }
        let expected = file.size();
        let mut data = Vec::with_capacity(usize::try_from(expected)?);
        (&mut file)
            .take(self.limits.member_bytes.saturating_add(1))
            .read_to_end(&mut data)?;
        if data.len() as u64 != expected || data.len() as u64 > self.limits.member_bytes {
            return Err(crate::fields::invalid(format!(
                "{member}: ZIP payload length differs from declaration/limit"
            )));
        }
        Ok(data)
    }
}

pub fn read_object(asset: &SerializedFile, info: &ObjectInfo) -> Result<UnityObject, Error> {
    let object = ObjectHandle::new(asset, info).read_with_options(TypeTreeParseOptions {
        mode: TypeTreeParseMode::Strict,
    })?;
    if object.class.properties().contains_key("_raw_data_len") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "class {} path ID {} has no usable type tree",
                info.type_id, info.path_id
            ),
        )
        .into());
    }
    Ok(object)
}

pub fn cab_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

pub fn source_apk(path: &Path) -> Result<SourceApk, Error> {
    let mut file = File::open(path)?;
    let byte_count = file.metadata()?.len();
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 256 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(SourceApk {
        path: path.display().to_string(),
        sha256: format!("{:x}", digest.finalize()),
        byte_count,
    })
}

#[cfg(test)]
mod tests {
    use super::texture_version;

    #[test]
    fn china_texture_layout_matches_the_base_release() {
        assert_eq!(
            texture_version("2021.3.23f1c1").unwrap(),
            texture_version("2021.3.23f1").unwrap()
        );
        assert!(texture_version("2021.3.23f1cgarbage").is_err());
        assert!(texture_version("not-a-version").is_err());
    }

    #[test]
    fn zip_member_limit_is_checked_before_payload_parsing() {
        use super::{Apk, BundleLimits};
        use std::{
            fs::File,
            io::Write,
            time::{SystemTime, UNIX_EPOCH},
        };
        let path = std::env::temp_dir().join(format!(
            "midnight-zip-limit-{}-{}.apk",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
        zip.start_file("oversized.ab", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&[0; 4096]).unwrap();
        zip.finish().unwrap();
        let mut apk = Apk::with_limits(
            &path,
            BundleLimits {
                member_bytes: 32,
                decompressed_bytes: 64,
            },
        )
        .unwrap();
        assert!(
            apk.bundle("oversized.ab")
                .unwrap_err()
                .to_string()
                .contains("ZIP member size 4096 exceeds limit 32")
        );
        assert!(apk.member_data("missing.ab").is_err());
        std::fs::remove_file(path).unwrap();
    }
}
