use std::{fs::File, io::Read, path::Path};
use midnight_map_data::SourceApk;
use sha2::{Digest, Sha256};

use unity_asset_binary::{
    asset::{ObjectInfo, SerializedFile},
    bundle::{AssetBundle, load_bundle_from_memory},
    object::{ObjectHandle, UnityObject},
    typetree::{TypeTreeParseMode, TypeTreeParseOptions},
};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// China builds append `cN` to the usual Unity version. The texture converter
/// needs the base engine layout; retain the full build string in source evidence.
pub fn texture_version(source: &str) -> Result<unity_asset_binary::unity_version::UnityVersion, Error> {
    let base = source.rsplit_once('c')
        .filter(|(_, suffix)| !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit()))
        .map_or(source, |(base, _)| base);
    Ok(unity_asset_binary::unity_version::UnityVersion::parse_version(base)?)
}

pub struct Apk {
    archive: zip::ZipArchive<File>,
}

impl Apk {
    pub fn open(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            archive: zip::ZipArchive::new(File::open(path)?)?,
        })
    }

    pub fn members(&mut self) -> Result<Vec<String>, Error> {
        (0..self.archive.len())
            .map(|index| Ok(self.archive.by_index_raw(index)?.name().to_owned()))
            .collect()
    }

    pub fn bundle(&mut self, member: &str) -> Result<AssetBundle, Error> {
        let mut data = Vec::new();
        self.archive.by_name(member)?.read_to_end(&mut data)?;
        load_bundle_from_memory(data).map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("{member}: {error}"),
            )
            .into()
        })
    }
}

pub fn read_object(asset: &SerializedFile, info: &ObjectInfo) -> Result<UnityObject, Error> {
    let object = ObjectHandle::new(asset, info).read_with_options(TypeTreeParseOptions {
        mode: TypeTreeParseMode::Strict,
    })?;
    if object.class.properties().contains_key("_raw_data_len") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("class {} path ID {} has no usable type tree", info.type_id, info.path_id),
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
        if count == 0 { break; }
        digest.update(&buffer[..count]);
    }
    Ok(SourceApk { path: path.display().to_string(), sha256: format!("{:x}", digest.finalize()), byte_count })
}

#[cfg(test)]
mod tests {
    use super::texture_version;

    #[test]
    fn china_texture_layout_matches_the_base_release() {
        assert_eq!(texture_version("2021.3.23f1c1").unwrap(), texture_version("2021.3.23f1").unwrap());
        assert!(texture_version("2021.3.23f1cgarbage").is_err());
        assert!(texture_version("not-a-version").is_err());
    }
}
