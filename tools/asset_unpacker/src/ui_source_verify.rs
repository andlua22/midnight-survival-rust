//! Verify retained UI files and bind original object bytes to the supplied APK.
//! This is static integrity/coverage evidence, not a Unity rendering comparison.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use midnight_map_data::{
    FileIdentity, SourceApk, SourceObject, file_sha256, package_path, read_rgba_mip_archive,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use sha2::{Digest, Sha256};
use unity_asset_binary::{bundle::load_bundle_from_memory, object::ObjectHandle};

use crate::{
    fields::invalid,
    pass1_index::GlobalIndex,
    unity::{Error, cab_name, read_object, source_apk},
};

use super::{
    importer::{PANELS, empty_font_texture},
    types::*,
};

#[derive(Debug, Serialize)]
pub struct VerifiedFont {
    pub source: SourceObject,
    pub name: String,
    pub embedded_outline_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct UiVerification {
    pub schema_version: u32,
    pub source_apk: SourceApk,
    pub manifest: FileIdentity,
    pub source_bundles: usize,
    pub source_objects: usize,
    pub verified_files: usize,
    pub verified_file_bytes: u64,
    pub panels: BTreeMap<String, PanelStats>,
    pub sprites: usize,
    pub decoded_textures: usize,
    pub texture_mip_counts: BTreeMap<u32, usize>,
    pub deferred_textures: Vec<DeferredTexture>,
    pub fonts: Vec<VerifiedFont>,
    pub reference_states: BTreeMap<String, usize>,
    pub issue_counts: BTreeMap<String, BTreeMap<String, usize>>,
    pub hierarchy_complete: bool,
    pub graphics_complete: bool,
    pub reference_coverage_complete: bool,
    pub runtime_equivalence_verified: bool,
    pub limitations: Vec<String>,
}

struct PackageFiles<'a> {
    root: &'a Path,
    checked: BTreeMap<String, (String, u64)>,
}

impl<'a> PackageFiles<'a> {
    fn new(root: &'a Path) -> Self {
        Self {
            root,
            checked: BTreeMap::new(),
        }
    }

    fn verify(&mut self, identity: &FileIdentity) -> Result<u64, Error> {
        let path = package_path(self.root, &identity.path)?;
        if let Some((hash, count)) = self.checked.get(&identity.path) {
            if hash != &identity.sha256 {
                return Err(invalid(format!(
                    "conflicting UI file identities: {}",
                    identity.path
                )));
            }
            return Ok(*count);
        }
        if file_sha256(&path)? != identity.sha256 {
            return Err(invalid(format!(
                "UI file checksum differs: {}",
                identity.path
            )));
        }
        let count = path.metadata()?.len();
        self.checked
            .insert(identity.path.clone(), (identity.sha256.clone(), count));
        Ok(count)
    }

    fn json<T: DeserializeOwned>(&mut self, identity: &FileIdentity) -> Result<T, Error> {
        self.verify(identity)?;
        Ok(serde_json::from_reader(BufReader::new(File::open(
            package_path(self.root, &identity.path)?,
        )?))?)
    }

    fn blobs(&mut self, value: &Value) -> Result<(), Error> {
        match value {
            Value::Object(fields) => {
                if let Some(blob) = fields.get("unity_bytes") {
                    let identity: FileIdentity = serde_json::from_value(blob.clone())?;
                    let count = fields
                        .get("byte_count")
                        .and_then(Value::as_u64)
                        .ok_or_else(|| invalid("source byte blob has no valid byte count"))?;
                    if self.verify(&identity)? != count {
                        return Err(invalid(format!(
                            "UI source byte count differs: {}",
                            identity.path
                        )));
                    }
                }
                for value in fields.values() {
                    self.blobs(value)?;
                }
            }
            Value::Array(values) => {
                for value in values {
                    self.blobs(value)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn same_file(actual: &FileIdentity, expected: &FileIdentity) -> Result<(), Error> {
    if actual.path != expected.path || actual.sha256 != expected.sha256 {
        return Err(invalid(
            "UI projection file identity differs from its source object record",
        ));
    }
    Ok(())
}

fn source_record<'a>(
    records: &'a BTreeMap<&SourceObject, &ObjectRecord>,
    source: &SourceObject,
    class: i32,
) -> Result<&'a ObjectRecord, Error> {
    let record = records.get(source).ok_or_else(|| {
        invalid(format!(
            "UI source object is not retained: {}:{}:{}",
            source.bundle, source.cab, source.path_id
        ))
    })?;
    if record.class_id != class {
        return Err(invalid(
            "UI projection class differs from its source object record",
        ));
    }
    Ok(record)
}

fn target(reference: &SourceReference) -> Option<&SourceObject> {
    match &reference.resolution {
        ReferenceState::Resolved { target, .. } => Some(target),
        _ => None,
    }
}

fn hierarchy(
    node: &HierarchyNode,
    parent: Option<&SourceObject>,
    parent_active: bool,
    records: &BTreeMap<&SourceObject, &ObjectRecord>,
    visited: &mut BTreeSet<SourceObject>,
) -> Result<(usize, usize), Error> {
    if !visited.insert(node.source.clone()) {
        return Err(invalid("duplicate/cyclic UI hierarchy GameObject"));
    }
    source_record(records, &node.source, 1)?;
    let transform_class = if node.transform.rect.is_some() {
        224
    } else {
        4
    };
    source_record(records, &node.transform.source, transform_class)?;
    if target(&node.transform.parent) != parent
        || (parent.is_none() && !matches!(node.transform.parent.resolution, ReferenceState::Null))
        || node.active_in_prefab_hierarchy != (parent_active && node.active_self)
        || node.transform.ordered_children.len() != node.children.len()
    {
        return Err(invalid(
            "UI hierarchy parent, activation or child count differs",
        ));
    }
    let mut transforms = 0;
    for component in &node.components {
        let record = source_record(records, &component.source, component.class_id)?;
        same_file(&component.source_fields, &record.source_fields)?;
        same_file(&component.serialized, &record.serialized)?;
        if component.reference.owner != node.source
            || target(&component.reference) != Some(&component.source)
        {
            return Err(invalid(
                "ordered UI component PPtr differs from its owner/target",
            ));
        }
        if matches!(component.class_id, 4 | 224) {
            transforms += 1;
            if component.source != node.transform.source
                || !matches!(component.projection, ComponentProjection::Transform)
            {
                return Err(invalid(
                    "UI transform projection differs from the ordered component",
                ));
            }
        }
    }
    if transforms != 1 {
        return Err(invalid(
            "UI GameObject must have exactly one source transform",
        ));
    }
    let mut counts = (1, node.components.len());
    for (reference, child) in node.transform.ordered_children.iter().zip(&node.children) {
        if reference.owner != node.transform.source
            || target(reference) != Some(&child.transform.source)
        {
            return Err(invalid(
                "UI child order differs from source transform PPtrs",
            ));
        }
        let child_counts = hierarchy(
            child,
            Some(&node.transform.source),
            node.active_in_prefab_hierarchy,
            records,
            visited,
        )?;
        counts.0 += child_counts.0;
        counts.1 += child_counts.1;
    }
    Ok(counts)
}

fn verify_sources(manifest: &UiManifest, index: &GlobalIndex, apk: &Path) -> Result<(), Error> {
    let mut archive = zip::ZipArchive::new(File::open(apk)?)?;
    let mut expected_cabs = BTreeMap::new();
    let members: BTreeSet<_> = manifest
        .bundles
        .iter()
        .map(|bundle| bundle.member.as_str())
        .collect();
    if members.len() != manifest.bundles.len() {
        return Err(invalid("duplicate UI source bundle"));
    }
    let mut by_bundle = BTreeMap::<&str, Vec<&ObjectRecord>>::new();
    for record in &manifest.source_objects {
        if !members.contains(record.source.bundle.as_str()) {
            return Err(invalid(
                "UI source object bundle is outside the dependency closure",
            ));
        }
        by_bundle
            .entry(&record.source.bundle)
            .or_default()
            .push(record);
    }
    let deferred: BTreeSet<_> = manifest
        .deferred_textures
        .iter()
        .map(|texture| &texture.source)
        .collect();
    for evidence in &manifest.bundles {
        for dependency in &evidence.dependencies {
            if !members.contains(dependency.as_str()) {
                return Err(invalid(
                    "UI declared dependency is absent from the retained closure",
                ));
            }
        }
        let mut data = Vec::new();
        archive.by_name(&evidence.member)?.read_to_end(&mut data)?;
        if data.len() as u64 != evidence.byte_count
            || format!("{:x}", Sha256::digest(&data)) != evidence.sha256
        {
            return Err(invalid(format!(
                "UI source bundle identity differs from APK: {}",
                evidence.member
            )));
        }
        let bundle = load_bundle_from_memory(data)?;
        let mut objects = BTreeMap::new();
        if evidence.cabs.len() != bundle.assets.len() {
            return Err(invalid("UI CAB count differs from APK bundle"));
        }
        for (name, asset) in bundle.asset_names.iter().zip(&bundle.assets) {
            let cab = cab_name(name);
            let cab_evidence = evidence
                .cabs
                .iter()
                .find(|evidence| evidence.cab == cab)
                .ok_or_else(|| invalid("UI source CAB is absent from bundle evidence"))?;
            if cab_evidence.unity_version != asset.unity_version
                || cab_evidence.object_count != asset.objects.len()
                || cab_evidence.type_tree_enabled != asset.enable_type_tree
                || cab_evidence.externals != serde_json::to_value(&asset.externals)?
            {
                return Err(invalid("UI CAB metadata differs from APK bundle"));
            }
            if expected_cabs
                .insert(cab.to_owned(), evidence.member.clone())
                .is_some()
            {
                return Err(invalid("duplicate UI CAB identity"));
            }
            for info in &asset.objects {
                if objects.insert((cab, info.path_id), (asset, info)).is_some() {
                    return Err(invalid("duplicate UI source PathID within CAB"));
                }
            }
        }
        if let Some(records) = by_bundle.get(evidence.member.as_str()) {
            for record in records {
                let path_id: i64 = record.source.path_id.parse()?;
                if path_id.to_string() != record.source.path_id {
                    return Err(invalid("UI source PathID is not canonical signed decimal"));
                }
                let (asset, info) = objects
                    .get(&(record.source.cab.as_str(), path_id))
                    .ok_or_else(|| {
                        invalid("retained UI source object does not exist in APK bundle")
                    })?;
                let bytes = ObjectHandle::new(asset, info).raw_data()?;
                if record.class_id != info.type_id
                    || record.unity_version != asset.unity_version
                    || record.serialized_byte_count != bytes.len() as u64
                    || record.serialized.sha256 != format!("{:x}", Sha256::digest(bytes))
                {
                    return Err(invalid(format!(
                        "retained UI object bytes differ from APK: {}:{}:{}",
                        record.source.bundle, record.source.cab, record.source.path_id
                    )));
                }
                if deferred.contains(&record.source)
                    && !empty_font_texture(&read_object(asset, info)?)?
                {
                    return Err(invalid(
                        "deferred UI font texture is not explicitly empty in the APK",
                    ));
                }
            }
        }
    }
    if expected_cabs
        != index
            .cabs
            .iter()
            .map(|(cab, member)| (cab.clone(), member.clone()))
            .collect()
    {
        return Err(invalid(
            "UI dependency index differs from APK CAB inventory",
        ));
    }
    Ok(())
}

pub fn verify(package: &Path, apk: &Path) -> Result<UiVerification, Error> {
    let source = source_apk(apk)?;
    let identity = FileIdentity {
        path: "source_ui.json".into(),
        sha256: file_sha256(&package.join("source_ui.json"))?,
    };
    let mut files = PackageFiles::new(package);
    let manifest: UiManifest = files.json(&identity)?;
    if manifest.schema_version != SCHEMA_VERSION
        || manifest.source_apk.sha256 != source.sha256
        || manifest.source_apk.byte_count != source.byte_count
    {
        return Err(invalid("UI package schema or source APK identity differs"));
    }
    let index: GlobalIndex = files.json(&manifest.dependency_index)?;
    let records: BTreeMap<_, _> = manifest
        .source_objects
        .iter()
        .map(|record| (&record.source, record))
        .collect();
    if records.len() != manifest.source_objects.len() {
        return Err(invalid("duplicate retained UI source object"));
    }
    for record in &manifest.source_objects {
        if files.verify(&record.serialized)? != record.serialized_byte_count {
            return Err(invalid("retained UI object byte count differs"));
        }
        let fields: Value = files.json(&record.source_fields)?;
        files.blobs(&fields)?;
    }
    verify_sources(&manifest, &index, apk)?;
    if manifest.panels.len() != PANELS.len() || !manifest.hierarchy_complete {
        return Err(invalid("UI panel hierarchy coverage is incomplete"));
    }
    let mut panels = BTreeMap::new();
    for (name, member, _) in PANELS {
        let summary = manifest
            .panels
            .iter()
            .find(|panel| panel.name == name && panel.bundle == member)
            .ok_or_else(|| invalid("expected UI panel is absent from manifest"))?;
        let layout = summary
            .layout
            .as_ref()
            .ok_or_else(|| invalid("UI panel has no retained layout"))?;
        let panel: UiPanel = files.json(layout)?;
        if panel.schema_version != SCHEMA_VERSION
            || panel.source_apk_sha256 != source.sha256
            || panel.name != name
            || panel.bundle != member
            || !summary.hierarchy_complete
            || summary.stats.as_ref() != Some(&panel.stats)
            || target(&panel.prefab_reference) != Some(&panel.root.source)
        {
            return Err(invalid(
                "UI panel identity, prefab pointer or summary differs",
            ));
        }
        let counts = hierarchy(&panel.root, None, true, &records, &mut BTreeSet::new())?;
        let source_nodes = manifest
            .source_objects
            .iter()
            .filter(|record| record.class_id == 1 && record.source.bundle == member)
            .count();
        if counts != (panel.stats.nodes, panel.stats.components) || counts.0 != source_nodes {
            return Err(invalid(
                "UI panel node/component coverage differs from retained source objects",
            ));
        }
        panels.insert(name.into(), panel.stats);
    }
    for sprite in &manifest.sprites {
        let record = source_record(&records, &sprite.source, 213)?;
        same_file(&sprite.source_fields, &record.source_fields)?;
        if let Some(preview) = &sprite.preview {
            files.verify(&preview.file)?;
            if image::image_dimensions(package_path(package, &preview.file.path)?)?
                != (preview.width, preview.height)
            {
                return Err(invalid("UI sprite preview dimensions differ"));
            }
        }
    }
    let mut texture_mip_counts = BTreeMap::new();
    let mut texture_sources = BTreeSet::new();
    for texture in &manifest.textures {
        source_record(&records, &texture.source, 28)?;
        if !texture_sources.insert(&texture.source) {
            return Err(invalid("duplicate decoded UI texture"));
        }
        files.verify(&texture.file)?;
        files.verify(&texture.encoded)?;
        let gpu = texture
            .gpu
            .as_ref()
            .ok_or_else(|| invalid("UI source mip chain is missing"))?;
        files.verify(gpu)?;
        let bytes = std::fs::read(package_path(package, &gpu.path)?)?;
        let chain = read_rgba_mip_archive(&bytes)?;
        let preview = image::open(package_path(package, &texture.file.path)?)?.into_rgba8();
        if (chain.width, chain.height, chain.mip_count)
            != (texture.width, texture.height, texture.mip_count)
            || preview.dimensions() != (texture.width, texture.height)
            || chain.pixels.get(..preview.as_raw().len()) != Some(preview.as_raw().as_slice())
        {
            return Err(invalid(
                "UI decoded source mips/base PNG differ from texture metadata",
            ));
        }
        *texture_mip_counts.entry(texture.mip_count).or_default() += 1;
    }
    for texture in &manifest.deferred_textures {
        let record = source_record(&records, &texture.source, 28)?;
        same_file(&texture.source_fields, &record.source_fields)?;
        same_file(&texture.serialized, &record.serialized)?;
        if !texture_sources.insert(&texture.source) || texture.reason != "empty_source_font_atlas" {
            return Err(invalid("invalid/duplicate deferred UI texture"));
        }
    }
    let mut fonts = Vec::new();
    for font in &manifest.fonts {
        let record = source_record(&records, &font.source, 128)?;
        same_file(&font.source_fields, &record.source_fields)?;
        let embedded_outline_bytes = font
            .embedded_font
            .as_ref()
            .map(|file| files.verify(file))
            .transpose()?
            .unwrap_or(0);
        fonts.push(VerifiedFont {
            source: font.source.clone(),
            name: font.name.clone(),
            embedded_outline_bytes,
        });
    }
    let mut issue_counts = BTreeMap::<String, BTreeMap<String, usize>>::new();
    for issue in &manifest.issues {
        *issue_counts
            .entry(issue.severity.clone())
            .or_default()
            .entry(issue.code.clone())
            .or_default() += 1;
    }
    if issue_counts.contains_key("error")
        || manifest.graphics_complete != manifest.deferred_textures.is_empty()
        || manifest.reference_coverage_complete
            != !manifest
                .issues
                .iter()
                .any(|issue| issue.code == "unresolved_reference")
        || manifest.runtime_equivalence_verified
    {
        return Err(invalid(
            "UI import errors or unsupported completeness/runtime claims",
        ));
    }
    let mut reference_states = BTreeMap::new();
    for reference in manifest
        .source_objects
        .iter()
        .flat_map(|record| &record.references)
    {
        let state = match reference.reference.resolution {
            ReferenceState::Null => "null",
            ReferenceState::Resolved { .. } => "resolved",
            ReferenceState::Unresolved { .. } => "unresolved",
        };
        *reference_states.entry(state.into()).or_default() += 1;
    }
    let after = source_apk(apk)?;
    if after.sha256 != source.sha256 || after.byte_count != source.byte_count {
        return Err(invalid("APK changed during UI verification"));
    }
    Ok(UiVerification {
        schema_version: manifest.schema_version, source_apk: source, manifest: identity,
        source_bundles: manifest.bundles.len(), source_objects: records.len(),
        verified_files: files.checked.len(), verified_file_bytes: files.checked.values().map(|(_, count)| count).sum(),
        panels, sprites: manifest.sprites.len(), decoded_textures: manifest.textures.len(), texture_mip_counts,
        deferred_textures: manifest.deferred_textures, fonts, reference_states, issue_counts,
        hierarchy_complete: manifest.hierarchy_complete, graphics_complete: manifest.graphics_complete,
        reference_coverage_complete: manifest.reference_coverage_complete, runtime_equivalence_verified: false,
        limitations: manifest.limitations.into_iter().chain([
            "Verification checks retained file hashes, raw object bytes against the supplied APK, hierarchy links, and decoded mip metadata/base PNG consistency. It does not independently rerun every field/texture decoder or compare rendered Unity pixels.".into(),
        ]).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_blob_integrity_rejects_wrong_counts_conflicting_hashes_and_tampering() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "midnight-ui-integrity-{}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("blob.bin");
        std::fs::write(&path, [1, 2, 3, 4]).unwrap();
        let identity = FileIdentity {
            path: "blob.bin".into(),
            sha256: file_sha256(&path).unwrap(),
        };
        let mut value = serde_json::json!({"nested": [{"unity_bytes": identity, "byte_count": 4}]});
        let mut files = PackageFiles::new(&root);
        files.blobs(&value).unwrap();
        value["nested"][0]["byte_count"] = serde_json::json!(3);
        assert!(files.blobs(&value).is_err());
        assert!(
            files
                .verify(&FileIdentity {
                    sha256: "0".repeat(64),
                    ..identity.clone()
                })
                .is_err()
        );
        assert!(
            files
                .verify(&FileIdentity {
                    path: "../blob.bin".into(),
                    ..identity.clone()
                })
                .is_err()
        );
        std::fs::write(&path, [4, 3, 2, 1]).unwrap();
        assert!(PackageFiles::new(&root).verify(&identity).is_err());
        std::fs::remove_file(&path).unwrap();
        assert!(PackageFiles::new(&root).verify(&identity).is_err());
        std::fs::remove_dir(root).unwrap();
    }
}
