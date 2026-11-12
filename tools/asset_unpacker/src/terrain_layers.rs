//! Source GameObject layer enrichment without decoding meshes or textures.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use midnight_map_data::{
    FileIdentity, SceneMesh, SourceCollider, SourceObject, TerrainManifest, TerrainPackage,
    file_sha256, load_terrain_package, package_path,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use unity_asset_core::UnityValue;

use crate::{
    bundle_cache::CacheStats,
    fields::{boolean, integer, invalid, property},
    package_publish::publish_verified,
    pass1_index::GlobalIndex,
    pass2_terrain::is_chunk,
    resolver::{ObjectId, Resolver},
    terrain_verification::{self, TerrainVerification},
    unity::{Error, cab_name, source_apk},
};

#[derive(Debug, Serialize)]
pub struct GameObjectLayer {
    pub game_object: SourceObject,
    pub layer: u8,
    pub render_records: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ColliderInventoryEntry {
    pub source: SourceObject,
    pub game_object: SourceObject,
    pub class_id: i32,
    pub layer: u8,
    pub enabled: bool,
    /// Only the owner's saved flag, not effective hierarchy/runtime activity.
    pub locally_active: bool,
    pub is_trigger: bool,
    pub retained_in_package: bool,
}

#[derive(Debug, Serialize)]
pub struct TerrainLayerAudit {
    pub game_objects: Vec<GameObjectLayer>,
    pub layer_counts: BTreeMap<u8, usize>,
    pub source_bundle_hashes: BTreeMap<String, String>,
    pub scene_serialized_files: usize,
    pub collider_class_counts: BTreeMap<i32, usize>,
    pub colliders: Vec<ColliderInventoryEntry>,
    pub omitted_colliders: Vec<SourceObject>,
    pub collider_owners_without_retained_render_records: Vec<SourceObject>,
    pub multiple_collider_owners: BTreeMap<String, Vec<SourceObject>>,
    pub full_runtime_collision_coverage_verified: bool,
    pub cache: CacheStats,
}

#[derive(Debug, Serialize)]
pub struct TerrainLayerEnrichment {
    pub package: String,
    pub backup: Option<String>,
    pub previous_manifest_sha256: String,
    pub manifest_sha256: String,
    pub geometry_sha256: String,
    pub already_enriched: bool,
    pub retained_files: Vec<FileIdentity>,
    pub source_audit: TerrainLayerAudit,
    pub verification: TerrainVerification,
}

fn object_id(source: &SourceObject) -> Result<ObjectId, Error> {
    let path_id = source.path_id.parse::<i64>()?;
    if path_id == 0 || path_id.to_string() != source.path_id {
        return Err(invalid(
            "source GameObject requires a canonical signed PathID",
        ));
    }
    Ok(ObjectId {
        bundle: source.bundle.clone(),
        cab: source.cab.clone(),
        path_id,
    })
}

pub(crate) fn layer_value(value: &UnityValue) -> Result<u8, Error> {
    u8::try_from(integer(value)?)
        .ok()
        .filter(|&value| value < 32)
        .ok_or_else(|| invalid("source GameObject m_Layer must be in 0..32"))
}

pub(crate) fn game_object_layer(
    object: &unity_asset_binary::object::UnityObject,
) -> Result<u8, Error> {
    if object.info.type_id != 1 {
        return Err(invalid("source layer owner is not a class-1 GameObject"));
    }
    layer_value(property(object, "m_Layer")?)
}

fn resolve_layers(
    manifest: &TerrainManifest,
    resolver: &mut Resolver<'_>,
) -> Result<BTreeMap<SourceObject, GameObjectLayer>, Error> {
    let mut layers = BTreeMap::<SourceObject, GameObjectLayer>::new();
    for (name, mesh) in &manifest.meshes {
        if let Some(entry) = layers.get_mut(&mesh.game_object) {
            entry.render_records.push(name.clone());
            continue;
        }
        let object = resolver.object(&object_id(&mesh.game_object)?)?;
        let layer = game_object_layer(&object)?;
        layers.insert(
            mesh.game_object.clone(),
            GameObjectLayer {
                game_object: mesh.game_object.clone(),
                layer,
                render_records: vec![name.clone()],
            },
        );
    }
    Ok(layers)
}

fn enriched_manifest(
    previous: &TerrainManifest,
    layers: &BTreeMap<SourceObject, GameObjectLayer>,
) -> Result<TerrainManifest, Error> {
    previous.validate_layers()?;
    let mut manifest = previous.clone();
    for mesh in manifest.meshes.values_mut() {
        let layer = layers
            .get(&mesh.game_object)
            .ok_or_else(|| invalid("source layer coverage is incomplete"))?
            .layer;
        if mesh.layer.is_some_and(|old| old != layer) {
            return Err(invalid("retained layer differs from the source GameObject"));
        }
        mesh.layer = Some(layer);
    }
    // Layer enrichment is schema three; a later collision table is preserved.
    manifest.schema_version = previous.schema_version.max(3);
    manifest.validate_layers()?;
    Ok(manifest)
}

/// Verify every schema-3 layer against the actual qualified class-1 object.
/// A legacy package remains explicitly unknown; no value is synthesized.
pub fn verify_source_layers(package: &TerrainPackage, apk: &Path) -> Result<bool, Error> {
    package.manifest.validate_layers()?;
    if package.manifest.schema_version < 3 {
        return Ok(false);
    }
    let index = GlobalIndex::default();
    let mut resolver = Resolver::new(apk, &index)?;
    let layers = resolve_layers(&package.manifest, &mut resolver)?;
    enriched_manifest(&package.manifest, &layers)?;
    Ok(true)
}

fn verify_retained_collider(
    mesh: &SceneMesh,
    retained: &SourceCollider,
    actual: &SourceCollider,
    owner: &SourceObject,
    layer: u8,
) -> Result<(), Error> {
    if &mesh.game_object != owner
        || mesh.layer.is_some_and(|value| value != layer)
        || retained.source != actual.source
        || retained.mesh != actual.mesh
        || retained.enabled != actual.enabled
        || retained.is_trigger != actual.is_trigger
        || retained.convex != actual.convex
        || retained.cooking_options != actual.cooking_options
    {
        return Err(invalid("retained collider metadata differs from source"));
    }
    Ok(())
}

fn source_audit(
    package: &TerrainPackage,
    resolver: &mut Resolver<'_>,
    members: &BTreeSet<String>,
    layers: BTreeMap<SourceObject, GameObjectLayer>,
) -> Result<TerrainLayerAudit, Error> {
    let mut represented = BTreeMap::<SourceObject, Vec<(&SceneMesh, &SourceCollider)>>::new();
    for mesh in package.manifest.meshes.values() {
        if let Some(collider) = &mesh.collider {
            represented
                .entry(collider.source.clone())
                .or_default()
                .push((mesh, collider));
        }
    }
    let mut found = BTreeSet::new();
    let mut source_bundle_hashes = BTreeMap::new();
    let mut collider_class_counts = BTreeMap::new();
    let mut colliders = Vec::new();
    let mut owners = BTreeMap::<SourceObject, Vec<SourceObject>>::new();
    let mut scene_serialized_files = 0;
    for member in members {
        source_bundle_hashes.insert(
            member.clone(),
            format!("{:x}", Sha256::digest(resolver.apk.member_data(member)?)),
        );
        let bundle = resolver.bundle(member)?;
        for (asset_index, asset) in bundle.assets.iter().enumerate() {
            if !asset.objects.iter().any(|info| info.type_id == 104) {
                continue;
            }
            scene_serialized_files += 1;
            for info in asset
                .objects
                .iter()
                .filter(|info| matches!(info.type_id, 64 | 65 | 135 | 136 | 143))
            {
                let id = ObjectId {
                    bundle: member.clone(),
                    cab: cab_name(&bundle.asset_names[asset_index]).into(),
                    path_id: info.path_id,
                };
                let object = resolver.object(&id)?;
                let owner = resolver.required_ref(&id, property(&object, "m_GameObject")?)?;
                let game_object = resolver.object(&owner)?;
                let layer = game_object_layer(&game_object)?;
                let locally_active = boolean(property(&game_object, "m_IsActive")?)?;
                let enabled = boolean(property(&object, "m_Enabled")?)?;
                let is_trigger = boolean(property(&object, "m_IsTrigger")?)?;
                let source = id.source();
                if !found.insert(source.clone()) {
                    return Err(invalid("duplicate qualified source collider identity"));
                }
                if let Some(occurrences) = represented.get(&source) {
                    if info.type_id != 64 {
                        return Err(invalid("retained collider metadata differs from source"));
                    }
                    let actual = SourceCollider {
                        source: source.clone(),
                        mesh: resolver
                            .required_ref(&id, property(&object, "m_Mesh")?)?
                            .source(),
                        enabled,
                        is_trigger,
                        convex: boolean(property(&object, "m_Convex")?)?,
                        cooking_options: integer(property(&object, "m_CookingOptions")?)?,
                    };
                    for (mesh, collider) in occurrences {
                        verify_retained_collider(mesh, collider, &actual, &owner.source(), layer)?;
                    }
                }
                owners
                    .entry(owner.source())
                    .or_default()
                    .push(source.clone());
                *collider_class_counts.entry(info.type_id).or_default() += 1;
                colliders.push(ColliderInventoryEntry {
                    retained_in_package: represented.contains_key(&source),
                    source,
                    game_object: owner.source(),
                    class_id: info.type_id,
                    layer,
                    enabled,
                    locally_active,
                    is_trigger,
                });
            }
        }
    }
    if represented.keys().any(|source| !found.contains(source)) {
        return Err(invalid(
            "a retained collider is absent from the source scene inventory",
        ));
    }
    let omitted_colliders = colliders
        .iter()
        .filter(|c| !c.retained_in_package)
        .map(|c| c.source.clone())
        .collect();
    let collider_owners_without_retained_render_records = owners
        .keys()
        .filter(|owner| !layers.contains_key(*owner))
        .cloned()
        .collect();
    let multiple_collider_owners = owners
        .into_iter()
        .filter(|(_, colliders)| colliders.len() > 1)
        .map(|(owner, colliders)| {
            (
                format!("{}:{}:{}", owner.bundle, owner.cab, owner.path_id),
                colliders,
            )
        })
        .collect();
    let mut layer_counts = BTreeMap::new();
    for entry in layers.values() {
        *layer_counts.entry(entry.layer).or_default() += 1;
    }
    Ok(TerrainLayerAudit {
        game_objects: layers.into_values().collect(),
        layer_counts,
        source_bundle_hashes,
        scene_serialized_files,
        collider_class_counts,
        colliders,
        omitted_colliders,
        collider_owners_without_retained_render_records,
        multiple_collider_owners,
        full_runtime_collision_coverage_verified: false,
        cache: resolver.cache_stats(),
    })
}

pub(crate) fn retained_files(manifest: &TerrainManifest) -> Result<Vec<FileIdentity>, Error> {
    let mut files = BTreeMap::new();
    let mut add = |identity: &FileIdentity| -> Result<(), Error> {
        if files
            .insert(identity.path.clone(), identity.sha256.clone())
            .is_some_and(|previous| previous != identity.sha256)
        {
            return Err(invalid("conflicting retained terrain file identities"));
        }
        Ok(())
    };
    add(&manifest.geometry)?;
    if let Some(collision) = &manifest.collision {
        add(&collision.geometry)?;
    }
    for texture in manifest
        .meshes
        .values()
        .flat_map(|mesh| mesh.material.textures.values())
        .filter_map(|binding| binding.texture.as_ref())
    {
        add(&texture.file)?;
        add(&texture.encoded)?;
        if let Some(gpu) = &texture.gpu {
            add(gpu)?;
        }
    }
    Ok(files
        .into_iter()
        .map(|(path, sha256)| FileIdentity { path, sha256 })
        .collect())
}

/// Enrich the canonical package only after source/payload checks, retaining its predecessor.
pub fn enrich(apk: &Path, output: &Path) -> Result<TerrainLayerEnrichment, Error> {
    let canonical = output.join("imported_map/terrain");
    let previous_manifest_sha256 = file_sha256(&canonical.join("manifest.json"))?;
    let previous = load_terrain_package(&canonical)?;
    if file_sha256(&canonical.join("manifest.json"))? != previous_manifest_sha256 {
        return Err(invalid("canonical terrain changed during loading"));
    }
    let before = source_apk(apk)?;
    let index: GlobalIndex = serde_json::from_reader(File::open(
        output.join("imported_map/global_asset_index.json"),
    )?)?;
    let mut resolver = Resolver::new(apk, &index)?;
    let members = resolver
        .apk
        .members()?
        .into_iter()
        .filter(|member| is_chunk(member))
        .collect::<BTreeSet<_>>();
    terrain_verification::verify_loaded(&previous, &before, &members)?;
    let layers = resolve_layers(&previous.manifest, &mut resolver)?;
    let mut manifest = enriched_manifest(&previous.manifest, &layers)?;
    let source_audit = source_audit(&previous, &mut resolver, &members, layers)?;
    let retained_files = retained_files(&manifest)?;
    let already_enriched = previous.manifest.schema_version >= 3;
    let mut backup = None;
    let mut verification;
    if already_enriched {
        verification = terrain_verification::verify_loaded(&previous, &before, &members)?;
    } else {
        manifest.limitations.push("Saved source GameObject layers are verified. Collider inventory is limited to static scene serialized files; parent activity, runtime-generated and streamed collision remain separate acceptance work.".into());
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let staged = output
            .join("imported_map")
            .join(format!(".terrain-layers-{stamp}"));
        std::fs::create_dir(&staged)?;
        let canonical_real = std::fs::canonicalize(&canonical)?;
        for identity in &retained_files {
            let source = package_path(&canonical, &identity.path)?;
            if !std::fs::canonicalize(&source)?.starts_with(&canonical_real) {
                return Err(invalid("retained terrain dependency escapes its package"));
            }
            let target = package_path(&staged, &identity.path)?;
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&source, &target)?;
            if file_sha256(&target)? != identity.sha256 {
                return Err(invalid(
                    "copied terrain dependency differs from retained bytes",
                ));
            }
        }
        let file = File::create(staged.join("manifest.json"))?;
        serde_json::to_writer_pretty(&file, &manifest)?;
        file.sync_all()?;
        let candidate = load_terrain_package(&staged)?;
        if candidate.records != previous.records {
            return Err(invalid("terrain layer enrichment changed mesh geometry"));
        }
        verification = terrain_verification::verify_loaded(&candidate, &before, &members)?;
        let after = source_apk(apk)?;
        if after.sha256 != before.sha256
            || after.byte_count != before.byte_count
            || file_sha256(&canonical.join("manifest.json"))? != previous_manifest_sha256
        {
            return Err(invalid(
                "source APK or canonical package changed during enrichment",
            ));
        }
        let retained = output
            .parent()
            .unwrap_or(Path::new("."))
            .join(".local/map-import-backups")
            .join(stamp.to_string())
            .join("terrain");
        publish_verified(&staged, &canonical, &retained)?;
        backup = Some(retained.display().to_string());
    }
    // The no-op path also rejects a concurrent source/package mutation.
    let after = source_apk(apk)?;
    if after.sha256 != before.sha256 || after.byte_count != before.byte_count {
        return Err(invalid("source APK changed during layer verification"));
    }
    if already_enriched
        && file_sha256(&canonical.join("manifest.json"))? != previous_manifest_sha256
    {
        return Err(invalid(
            "canonical terrain changed during layer verification",
        ));
    }
    verification.source_layer_values_verified = true;
    Ok(TerrainLayerEnrichment {
        package: canonical.display().to_string(),
        backup,
        previous_manifest_sha256,
        manifest_sha256: file_sha256(&canonical.join("manifest.json"))?,
        geometry_sha256: manifest.geometry.sha256.clone(),
        already_enriched,
        retained_files,
        source_audit,
        verification,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> TerrainManifest {
        let source = serde_json::json!({"bundle":"scene.ab","cab":"scene","path_id":"-17"});
        let mesh = serde_json::json!({
            "source":source,"game_object":source,
            "world_transform":[1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.],
            "bounds":[0.,0.,0.,1.,0.,1.],"collider":null,
            "material":{"source":source,"name":"retained material","shader":source,
                "shader_name":"source shader","textures":{},"floats":{"source_parameter":0.5},
                "colors":{},"shader_defaults":{},"texture_defaults":{},"valid_keywords":[],"render_queue":-1}
        });
        serde_json::from_value(serde_json::json!({
            "schema_version":2,"source_apk":{"path":"fixture.apk","sha256":"a".repeat(64),"byte_count":1},
            "coordinate_frame":"bevy_right_handed_y_up","uv_origin":"top_left",
            "geometry":{"path":"meshes.bin","sha256":"b".repeat(64)},
            "meshes":{"first":mesh,"first@submesh-1":mesh},"limitations":["retained limitation"]
        })).unwrap()
    }

    #[test]
    fn enrichment_preserves_all_source_metadata_and_applies_one_owner_layer_to_submeshes() {
        let previous = manifest();
        let owner = previous.meshes["first"].game_object.clone();
        let layers = BTreeMap::from([(
            owner.clone(),
            GameObjectLayer {
                game_object: owner,
                layer: 0,
                render_records: vec!["first".into(), "first@submesh-1".into()],
            },
        )]);
        let enriched = enriched_manifest(&previous, &layers).unwrap();
        let mut expected = serde_json::to_value(&previous).unwrap();
        expected["schema_version"] = serde_json::json!(3);
        for mesh in expected["meshes"].as_object_mut().unwrap().values_mut() {
            mesh["layer"] = serde_json::json!(0);
        }
        assert_eq!(serde_json::to_value(enriched).unwrap(), expected);
        assert!(previous.meshes.values().all(|mesh| mesh.layer.is_none()));
    }

    #[test]
    fn incomplete_source_layers_or_changed_verified_layer_reject_without_mutating_input() {
        let previous = manifest();
        let original = serde_json::to_value(&previous).unwrap();
        assert!(enriched_manifest(&previous, &BTreeMap::new()).is_err());
        assert_eq!(serde_json::to_value(&previous).unwrap(), original);
        let owner = previous.meshes["first"].game_object.clone();
        let mut layers = BTreeMap::from([(
            owner.clone(),
            GameObjectLayer {
                game_object: owner.clone(),
                layer: 19,
                render_records: vec![],
            },
        )]);
        let admitted = enriched_manifest(&previous, &layers).unwrap();
        let original = serde_json::to_value(&admitted).unwrap();
        layers.get_mut(&owner).unwrap().layer = 30;
        assert!(enriched_manifest(&admitted, &layers).is_err());
        assert_eq!(serde_json::to_value(admitted).unwrap(), original);
    }

    #[test]
    fn source_layer_parser_rejects_missing_range_and_non_integer_values() {
        for value in [0, 19, 30, 31] {
            let unity: UnityValue = serde_json::from_value(serde_json::json!(value)).unwrap();
            assert_eq!(layer_value(&unity).unwrap(), value as u8);
        }
        for value in [
            serde_json::json!(-1),
            serde_json::json!(32),
            serde_json::json!(256),
            serde_json::json!(1.5),
            serde_json::json!("0"),
            serde_json::Value::Null,
        ] {
            let unity: UnityValue = serde_json::from_value(value).unwrap();
            assert!(layer_value(&unity).is_err());
        }
    }

    #[test]
    fn source_collider_comparison_checks_every_submesh_occurrence() {
        let manifest = manifest();
        let mut first = manifest.meshes["first"].clone();
        let actual = SourceCollider {
            source: SourceObject {
                path_id: "-33".into(),
                ..first.source.clone()
            },
            mesh: first.source.clone(),
            enabled: true,
            is_trigger: false,
            convex: false,
            cooking_options: 30,
        };
        first.collider = Some(actual.clone());
        let mut second = first.clone();
        // The later submesh is valid, but must not conceal an earlier bad flag.
        first.collider.as_mut().unwrap().enabled = false;
        let occurrences = [&first, &second];
        assert!(occurrences.iter().any(|mesh| {
            verify_retained_collider(
                mesh,
                mesh.collider.as_ref().unwrap(),
                &actual,
                &second.game_object,
                19,
            )
            .is_err()
        }));
        first.collider = Some(actual.clone());
        assert!(verify_retained_collider(&first, &actual, &actual, &first.game_object, 19).is_ok());
        second.game_object.path_id = "-18".into();
        assert!(
            verify_retained_collider(&second, &actual, &actual, &first.game_object, 19).is_err()
        );
    }

    #[test]
    fn qualified_object_ids_preserve_signed_values_and_reject_ambiguous_text() {
        let mut source = SourceObject {
            bundle: "scene.ab".into(),
            cab: "scene".into(),
            path_id: i64::MIN.to_string(),
        };
        assert_eq!(object_id(&source).unwrap().source(), source);
        for invalid_id in ["0", "01", "+1", "9223372036854775808"] {
            source.path_id = invalid_id.into();
            assert!(object_id(&source).is_err());
        }
    }
}
