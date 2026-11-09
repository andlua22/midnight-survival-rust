use std::{
    collections::BTreeMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use midnight_map_data::{FileIdentity, SourceObject, TextureAsset, package_path};
use sha2::{Digest, Sha256};

use crate::{
    FontAsset, ObjectRecord, UI_SCHEMA_VERSION, UiError, UiManifest, UiPanel, UiSprite, invalid,
};

#[derive(Debug)]
pub struct UiPackage {
    pub root: PathBuf,
    pub manifest: UiManifest,
    sprites: BTreeMap<SourceObject, usize>,
    textures: BTreeMap<SourceObject, usize>,
    fonts: BTreeMap<SourceObject, usize>,
    records: BTreeMap<SourceObject, usize>,
}

fn index(
    sources: impl Iterator<Item = SourceObject>,
    label: &str,
) -> Result<BTreeMap<SourceObject, usize>, UiError> {
    let mut result = BTreeMap::new();
    for (position, source) in sources.enumerate() {
        if result.insert(source, position).is_some() {
            return Err(invalid(format!("duplicate source {label}")));
        }
    }
    Ok(result)
}

pub fn load_ui_package(root: &Path) -> Result<UiPackage, UiError> {
    let manifest: UiManifest =
        serde_json::from_reader(BufReader::new(File::open(root.join("source_ui.json"))?))?;
    if manifest.schema_version != UI_SCHEMA_VERSION
        || manifest.source_apk.sha256.len() != 64
        || !manifest
            .source_apk
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || manifest.source_apk.byte_count == 0
        || !manifest.hierarchy_complete
        || manifest
            .issues
            .iter()
            .any(|issue| issue.severity == "error")
    {
        return Err(invalid("unsupported/incomplete UI source package"));
    }
    let package = UiPackage {
        sprites: index(
            manifest.sprites.iter().map(|item| item.source.clone()),
            "sprite",
        )?,
        textures: index(
            manifest.textures.iter().map(|item| item.source.clone()),
            "texture",
        )?,
        fonts: index(
            manifest.fonts.iter().map(|item| item.source.clone()),
            "font",
        )?,
        records: index(
            manifest
                .source_objects
                .iter()
                .map(|item| item.source.clone()),
            "object",
        )?,
        root: root.to_path_buf(),
        manifest,
    };
    Ok(package)
}

impl UiPackage {
    /// Check a payload when first consumed. The APK-bound full audit remains in the importer.
    pub fn read_verified(&self, identity: &FileIdentity) -> Result<Vec<u8>, UiError> {
        let path = package_path(&self.root, &identity.path)?;
        let bytes = std::fs::read(&path)?;
        if format!("{:x}", Sha256::digest(&bytes)) != identity.sha256 {
            return Err(invalid(format!(
                "UI payload checksum differs: {}",
                identity.path
            )));
        }
        Ok(bytes)
    }

    pub fn load_panel(&self, name: &str) -> Result<UiPanel, UiError> {
        let matches: Vec<_> = self
            .manifest
            .panels
            .iter()
            .filter(|panel| panel.name == name)
            .collect();
        let [summary] = matches.as_slice() else {
            return Err(invalid(format!("expected one UI panel named {name}")));
        };
        let file = summary
            .layout
            .as_ref()
            .ok_or_else(|| invalid("UI panel layout is absent"))?;
        let panel: UiPanel = serde_json::from_slice(&self.read_verified(file)?)?;
        if panel.schema_version != UI_SCHEMA_VERSION
            || panel.source_apk_sha256 != self.manifest.source_apk.sha256
            || panel.name != name
            || panel.bundle != summary.bundle
            || panel.root.name != name
            || panel.prefab_reference.target() != Some(&panel.root.source)
            || !summary.hierarchy_complete
            || summary.stats.as_ref() != Some(&panel.stats)
        {
            return Err(invalid("UI panel identity/summary differs from package"));
        }
        Ok(panel)
    }

    pub fn sprite(&self, source: &SourceObject) -> Option<&UiSprite> {
        self.sprites
            .get(source)
            .map(|&index| &self.manifest.sprites[index])
    }
    pub fn texture(&self, source: &SourceObject) -> Option<&TextureAsset> {
        self.textures
            .get(source)
            .map(|&index| &self.manifest.textures[index])
    }
    pub fn font(&self, source: &SourceObject) -> Option<&FontAsset> {
        self.fonts
            .get(source)
            .map(|&index| &self.manifest.fonts[index])
    }
    pub fn record(&self, source: &SourceObject) -> Option<&ObjectRecord> {
        self.records
            .get(source)
            .map(|&index| &self.manifest.source_objects[index])
    }

    pub fn source_fields(&self, source: &SourceObject) -> Result<serde_json::Value, UiError> {
        let record = self
            .record(source)
            .ok_or_else(|| invalid("source UI object fields are absent"))?;
        Ok(serde_json::from_slice(
            &self.read_verified(&record.source_fields)?,
        )?)
    }
}
