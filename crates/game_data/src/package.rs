//! Small, hash-verified packages of source configuration, independent of Bevy.
use crate::{
    DataError, GameplayCatalog, read_drawings, read_equipment_slots, read_items, read_recipes,
    read_trees, read_use_items,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub type PackageError = Box<dyn std::error::Error + Send + Sync>;
pub const TABLES: [&str; 6] = [
    "cfg.ItemCfg.oc",
    "cfg.CraftRecipeCfg.oc",
    "cfg.TreeCfg.oc",
    "cfg.EquipSlotCfg.oc",
    "cfg.UseItemCfg.oc",
    "cfg.DrawingCfg.oc",
];
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceFile {
    pub sha256: String,
    pub bytes: u64,
    pub records: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameplayManifest {
    pub schema_version: u32,
    pub source_apk_sha256: String,
    pub tables: BTreeMap<String, SourceFile>,
    pub catalog: SourceFile,
    pub limitations: Vec<String>,
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn file_hash(path: &Path) -> Result<String, PackageError> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut bytes = [0u8; 256 * 1024];
    loop {
        let count = file.read(&mut bytes)?;
        if count == 0 {
            break;
        }
        digest.update(&bytes[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn decode(tables: &BTreeMap<String, Vec<u8>>) -> Result<GameplayCatalog, DataError> {
    let get = |name: &str| {
        tables
            .get(name)
            .map(Vec::as_slice)
            .ok_or_else(|| DataError(format!("missing source table {name}")))
    };
    Ok(GameplayCatalog {
        items: read_items(get(TABLES[0])?)?,
        recipes: read_recipes(get(TABLES[1])?)?,
        trees: read_trees(get(TABLES[2])?)?,
        equipment_slots: read_equipment_slots(get(TABLES[3])?)?,
        use_items: read_use_items(get(TABLES[4])?)?,
        drawings: read_drawings(get(TABLES[5])?)?,
    })
}
fn counts(catalog: &GameplayCatalog) -> [usize; 6] {
    [
        catalog.items.len(),
        catalog.recipes.len(),
        catalog.trees.len(),
        catalog.equipment_slots.len(),
        catalog.use_items.len(),
        catalog.drawings.len(),
    ]
}

/// Reads only the selected small .oc members. Existing output is never replaced.
pub fn import_apk(apk: &Path, output: &Path) -> Result<GameplayManifest, PackageError> {
    if output.exists() {
        return Err(DataError(format!("output already exists: {}", output.display())).into());
    }
    let before = file_hash(apk)?;
    let mut zip = zip::ZipArchive::new(File::open(apk)?)?;
    let selected: BTreeSet<_> = TABLES
        .into_iter()
        .map(|name| format!("assets/cfg/{name}"))
        .collect();
    let mut seen = BTreeSet::new();
    for index in 0..zip.len() {
        let member = zip.by_index_raw(index)?;
        if selected.contains(member.name()) && !seen.insert(member.name().to_owned()) {
            return Err(DataError("duplicate configuration ZIP member".into()).into());
        }
    }
    let mut tables = BTreeMap::new();
    for name in TABLES {
        let mut member = zip.by_name(&format!("assets/cfg/{name}"))?;
        if member.size() > 64 * 1024 * 1024 {
            return Err(DataError("configuration member exceeds size limit".into()).into());
        }
        let mut bytes = Vec::new();
        let declared_size = member.size();
        member
            .by_ref()
            .take(declared_size + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 != declared_size {
            return Err(DataError("configuration member length mismatch".into()).into());
        }
        tables.insert(name.to_owned(), bytes);
    }
    let catalog = decode(&tables)?;
    if file_hash(apk)? != before {
        return Err(DataError("source APK changed during import".into()).into());
    }
    let catalog_bytes = serde_json::to_vec_pretty(&catalog)?;
    let mut limitations = catalog.validate_references();
    limitations.push("Configuration serialization is decoded; runtime callbacks, IFix overrides, crafting time units and item behavior still require native-code corroboration.".into());
    let manifest = GameplayManifest {
        schema_version: 2,
        source_apk_sha256: before,
        tables: TABLES
            .into_iter()
            .zip(counts(&catalog))
            .map(|(name, records)| {
                let bytes = &tables[name];
                (
                    name.to_owned(),
                    SourceFile {
                        sha256: hash(bytes),
                        bytes: bytes.len() as u64,
                        records,
                    },
                )
            })
            .collect(),
        catalog: SourceFile {
            sha256: hash(&catalog_bytes),
            bytes: catalog_bytes.len() as u64,
            records: counts(&catalog).into_iter().sum(),
        },
        limitations,
    };
    let parent = output
        .parent()
        .ok_or_else(|| DataError("output needs a parent directory".into()))?;
    std::fs::create_dir_all(parent)?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let stage = parent.join(format!(".gameplay-import-{stamp}"));
    std::fs::create_dir(&stage)?;
    for (name, bytes) in &tables {
        std::fs::write(stage.join(name), bytes)?;
    }
    std::fs::write(stage.join("catalog.json"), catalog_bytes)?;
    std::fs::write(
        stage.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    load_package(&stage)?;
    std::fs::rename(&stage, output)?;
    Ok(manifest)
}

fn read_verified(root: &Path, name: &str, identity: &SourceFile) -> Result<Vec<u8>, PackageError> {
    if name.is_empty()
        || Path::new(name)
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err(DataError("invalid package path".into()).into());
    }
    if identity.bytes > 64 * 1024 * 1024 {
        return Err(DataError("package file exceeds size limit".into()).into());
    }
    let path = root.join(name);
    let canonical = std::fs::canonicalize(&path)?;
    if !canonical.starts_with(std::fs::canonicalize(root)?) {
        return Err(DataError("package symlink escapes root".into()).into());
    }
    let file = File::open(canonical)?;
    if file.metadata()?.len() != identity.bytes {
        return Err(DataError("package file length mismatch".into()).into());
    }
    let mut bytes = Vec::new();
    file.take(identity.bytes + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != identity.bytes {
        return Err(DataError("package file changed while reading".into()).into());
    }
    if hash(&bytes) != identity.sha256 {
        return Err(DataError(format!("package checksum mismatch: {name}")).into());
    }
    Ok(bytes)
}

pub fn load_package(root: &Path) -> Result<(GameplayManifest, GameplayCatalog), PackageError> {
    let manifest_path: PathBuf = root.join("manifest.json");
    if std::fs::metadata(&manifest_path)?.len() > 1024 * 1024 {
        return Err(DataError("gameplay manifest exceeds limit".into()).into());
    }
    let manifest: GameplayManifest = serde_json::from_reader(File::open(manifest_path)?)?;
    if manifest.schema_version != 2
        || !valid_digest(&manifest.source_apk_sha256)
        || manifest.tables.len() != TABLES.len()
    {
        return Err(DataError("unsupported gameplay manifest".into()).into());
    }
    let mut tables = BTreeMap::new();
    for name in TABLES {
        let identity = manifest
            .tables
            .get(name)
            .ok_or_else(|| DataError(format!("missing table {name}")))?;
        tables.insert(name.to_owned(), read_verified(root, name, identity)?);
    }
    let raw_catalog = decode(&tables)?;
    let catalog: GameplayCatalog =
        serde_json::from_slice(&read_verified(root, "catalog.json", &manifest.catalog)?)?;
    if catalog != raw_catalog {
        return Err(
            DataError("decoded catalog differs from retained source records".into()).into(),
        );
    }
    for (name, count) in TABLES.into_iter().zip(counts(&catalog)) {
        if manifest.tables[name].records != count {
            return Err(DataError("configuration record count mismatch".into()).into());
        }
    }
    if manifest.catalog.records != counts(&catalog).into_iter().sum::<usize>() {
        return Err(DataError("catalog record count mismatch".into()).into());
    }
    Ok((manifest, catalog))
}

fn valid_digest(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "midnight-game-data-test-{}-{stamp}",
                std::process::id()
            ));
            std::fs::create_dir(&root).unwrap();
            Self(root)
        }
        fn apk(&self) -> PathBuf {
            let path = self.0.join("fixture.apk");
            let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
            for name in TABLES {
                zip.start_file(
                    format!("assets/cfg/{name}"),
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
                if name == "cfg.EquipSlotCfg.oc" {
                    for value in [1i32, 2, 3, 4] {
                        zip.write_all(&value.to_le_bytes()).unwrap();
                    }
                }
            }
            zip.finish().unwrap();
            path
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn import_verifies_raw_tables_and_catalog_and_preserves_existing_output() {
        let fixture = Fixture::new();
        let apk = fixture.apk();
        let output = fixture.0.join("package");
        let manifest = import_apk(&apk, &output).unwrap();
        assert_eq!(manifest.source_apk_sha256, file_hash(&apk).unwrap());
        let (_, catalog) = load_package(&output).unwrap();
        assert_eq!(catalog.equipment_slots[&1].slots, [3, 4]);
        assert!(import_apk(&apk, &output).is_err());
        std::fs::write(output.join(TABLES[3]), vec![0u8; 16]).unwrap();
        assert!(load_package(&output).is_err());
    }

    #[test]
    fn a_rehashed_catalog_cannot_disagree_with_retained_octets() {
        let fixture = Fixture::new();
        let output = fixture.0.join("package");
        let mut manifest = import_apk(&fixture.apk(), &output).unwrap();
        let (_, mut catalog) = load_package(&output).unwrap();
        catalog.equipment_slots.get_mut(&1).unwrap().slots = vec![99];
        let bytes = serde_json::to_vec_pretty(&catalog).unwrap();
        manifest.catalog.sha256 = hash(&bytes);
        manifest.catalog.bytes = bytes.len() as u64;
        std::fs::write(output.join("catalog.json"), bytes).unwrap();
        std::fs::write(
            output.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(
            load_package(&output)
                .unwrap_err()
                .to_string()
                .contains("differs from retained source")
        );
    }

    #[test]
    fn catalog_count_and_apk_digest_are_validated() {
        let fixture = Fixture::new();
        let output = fixture.0.join("package");
        let mut manifest = import_apk(&fixture.apk(), &output).unwrap();
        manifest.catalog.records += 1;
        std::fs::write(
            output.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(load_package(&output).is_err());
        manifest.catalog.records -= 1;
        manifest.source_apk_sha256 = "z".repeat(64);
        std::fs::write(
            output.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(load_package(&output).is_err());
    }
}
