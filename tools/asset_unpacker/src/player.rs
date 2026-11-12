//! Explicit, staged player import. Source bundles remain independent of terrain.

use std::{collections::{BTreeMap, BTreeSet}, fs::File, path::Path, time::{SystemTime, UNIX_EPOCH}};

use midnight_map_data::{PLAYER_BINDING_CONTRACT, PLAYER_RIG_VERSION, PlayerAnimatorBinding,
    PlayerAnimationMaps, PlayerClipOverride, PlayerDecodedAsset, PlayerRigManifest, PlayerRigRoot, PlayerSourceBundle, PlayerSourceObject,
    SourceComponent, SourceObject, file_sha256, load_player_rig_package, player_named_transform,
    serialized_player_colliders, parse_player_animation_map, player_container_asset};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{fields::{array, field, invalid, property, text}, pass1_index::GlobalIndex,
    prefab::{list_files, raw_component, source_node, store, verify_raw_component}, resolver::{ObjectId, Resolver},
    unity::{Apk, Error, cab_name, read_object, source_apk}};

const ROLE_BUNDLE: &str = "assets/role/role.ab";
const ROOT_PATHS: [&str;3] = ["assets/roles/role/role.prefab", "assets/roles/role/role_new.prefab", "assets/roles/role/rolehand.prefab"];

#[derive(Debug, Serialize)]
pub struct PlayerImportReport {
    pub package: String,
    pub manifest_sha256: String,
    pub source_bundles: usize,
    pub source_objects: usize,
    pub roots: Vec<String>,
    pub nodes: usize,
    pub colliders: usize,
    pub animators: usize,
    pub decoded_avatars: usize,
    pub decoded_clips: usize,
    pub empty_override_clips: usize,
    pub resolved_default_overrides: usize,
    pub verified_files: usize,
    pub already_imported: bool,
    pub limitations: Vec<String>,
}

fn object_id(source: &SourceObject) -> Result<ObjectId, Error> {
    Ok(ObjectId {bundle: source.bundle.clone(), cab: source.cab.clone(), path_id: source.path_id.parse()?})
}

/// Follow only the selected bundle's declared dependencies. All original object
/// bytes, including unsupported types, remain recoverable from retained bundles.
fn dependencies(apk: &Path, output: &Path) -> Result<(GlobalIndex, Vec<PlayerSourceBundle>, Vec<PlayerSourceObject>), Error> {
    let mut archive = Apk::open(apk)?;
    let mut pending = BTreeSet::from([ROLE_BUNDLE.to_owned()]);
    let mut bundles = BTreeMap::new();
    let mut index = GlobalIndex::default();
    let mut inventory = Vec::new();
    while let Some(member) = pending.pop_first() {
        if bundles.contains_key(&member) { continue; }
        if bundles.len() >= 64 { return Err(invalid("player dependency count exceeds bound")); }
        let bytes = archive.member_data(&member)?;
        let identity = store(output, "source/bundles", &bytes)?;
        let byte_count = bytes.len() as u64;
        let bundle = crate::unity::parse_bundle(bytes, Default::default())?;
        let mut declared = BTreeSet::new();
        let mut cabs = Vec::new();
        for (name, asset) in bundle.asset_names.iter().zip(&bundle.assets) {
            let cab = cab_name(name).to_owned();
            if index.cabs.insert(cab.clone(), member.clone()).is_some() {
                return Err(invalid("duplicate CAB across player dependencies"));
            }
            cabs.push(cab.clone());
            for info in &asset.objects {
                inventory.push(PlayerSourceObject {
                    source: ObjectId {bundle: member.clone(), cab: cab.clone(), path_id: info.path_id}.source(),
                    class_id: info.type_id, byte_count: info.byte_size,
                    sha256: format!("{:x}", Sha256::digest(asset.object_bytes(info)?)),
                });
                if info.type_id == 142 {
                    let object = read_object(asset, info)?;
                    for value in array(property(&object, "m_Dependencies")?)? {
                        let path = text(value)?;
                        // Validate relative source names before adding the APK prefix.
                        if path.starts_with('/') || path.split('/').any(|p| p.is_empty() || p == "." || p == "..") {
                            return Err(invalid("unsafe source player dependency"));
                        }
                        declared.insert(path.to_owned());
                        pending.insert(format!("assets/{path}"));
                    }
                }
            }
        }
        cabs.sort();
        bundles.insert(member.clone(), PlayerSourceBundle {member, file: identity, byte_count, cabs, dependencies: declared.into_iter().collect()});
    }
    inventory.sort_by(|a,b| a.source.cmp(&b.source));
    Ok((index, bundles.into_values().collect(), inventory))
}

fn retain_asset(resolver: &mut Resolver<'_>, assets: &mut BTreeMap<ObjectId, SourceComponent>, id: &ObjectId, output: &Path) -> Result<(), Error> {
    if !assets.contains_key(id) { assets.insert(id.clone(), raw_component(resolver,id,output)?); }
    Ok(())
}

fn build(apk: &Path, output: &Path, additional_clips: &[String]) -> Result<PlayerRigManifest, Error> {
    let before = source_apk(apk)?;
    if before.sha256 != midnight_map_data::PLAYER_SOURCE_APK { return Err(invalid("native player bindings require the selected LITE 4.0 APK")); }
    let (index, bundles, inventory) = dependencies(apk, output)?;
    let mut resolver = Resolver::new(apk, &index)?;
    let mut assets = BTreeMap::new();
    let mut selected = BTreeMap::new();
    for info in inventory.iter().filter(|info| info.class_id == 142) {
        let id = object_id(&info.source)?;
        let object = resolver.object(&id)?;
        if id.bundle == ROLE_BUNDLE {
            for entry in array(property(&object,"m_Container")?)? {
                let pair = array(entry)?;
                if pair.len() != 2 { return Err(invalid("malformed player container")); }
                let path = text(&pair[0])?;
                if ROOT_PATHS.contains(&path) {
                    let root = resolver.required_ref(&id,field(&pair[1],"asset")?)?;
                    if selected.insert(path.to_owned(),root).is_some() { return Err(invalid("duplicate selected player root")); }
                }
            }
        }
        retain_asset(&mut resolver,&mut assets,&id,output)?;
    }
    if selected.len() != ROOT_PATHS.len() { return Err(invalid("required player prefab root missing")); }
    let name_map = player_container_asset(assets.values(),"Assets/Roles/RoleAnimNameMap.json")?;
    let state_map = player_container_asset(assets.values(),"Assets/Roles/RoleAnimState2ClipMap.json")?;
    let mut read_map = |id:&SourceObject| -> Result<BTreeMap<String,String>,Error> {
        let id = object_id(id)?;
        let object = resolver.object(&id)?;
        if object.info.type_id != 49 { return Err(invalid("animation map is not a TextAsset")); }
        let values = parse_player_animation_map(text(property(&object,"m_Script")?)?)?;
        retain_asset(&mut resolver,&mut assets,&id,output)?;
        Ok(values)
    };
    let animation_maps = PlayerAnimationMaps {
        names:read_map(&name_map)?,states:read_map(&state_map)?,name_map,state_map,
    };
    let mut pending = selected.values().cloned().collect::<Vec<_>>();
    let mut nodes = BTreeMap::new();
    while let Some(id) = pending.pop() {
        if nodes.contains_key(&id) || nodes.len() >= 100_000 { return Err(invalid("duplicate/deep player hierarchy")); }
        let node = source_node(&mut resolver,&id,output)?;
        for child in &node.children { pending.push(object_id(child)?); }
        nodes.insert(id,node);
    }
    let nodes = nodes.into_values().collect::<Vec<_>>();
    let roots = selected.into_iter().map(|(asset_path,id)| {
        let game_object = id.source();
        if nodes.iter().find(|n| n.source == game_object).is_none_or(|n| n.parent.is_some()) {
            return Err(invalid("selected player asset is not a root"));
        }
        Ok(PlayerRigRoot {
            asset_path,
            root_bone: player_named_transform(&nodes,&game_object,"Bip001")?,
            head_top: player_named_transform(&nodes,&game_object,"Head")?,
            player_collider: player_named_transform(&nodes,&game_object,"Collider")?,
            game_object,
        })
    }).collect::<Result<Vec<_>,Error>>()?;
    let colliders = serialized_player_colliders(&nodes)?;
    let mut animators = Vec::new();
    let mut avatars = BTreeSet::new();
    let mut wanted_clips = BTreeSet::new();
    let mut controller_clips = BTreeSet::new();
    for component in nodes.iter().flat_map(|n| &n.components).filter(|c| c.class_id == 95) {
        let id = object_id(&component.source)?;
        let object = resolver.object(&id)?;
        let avatar = resolver.required_ref(&id,property(&object,"m_Avatar")?)?;
        let controller = resolver.required_ref(&id,property(&object,"m_Controller")?)?;
        retain_asset(&mut resolver,&mut assets,&avatar,output)?;
        retain_asset(&mut resolver,&mut assets,&controller,output)?;
        avatars.insert(avatar.clone());
        let controller_object = resolver.object(&controller)?;
        let clips = array(property(&controller_object,"m_AnimationClips")?)?.iter()
            .map(|pointer| resolver.resolve(&controller,pointer))
            .collect::<Result<Vec<_>,_>>()?;
        let default_clips = crate::player_animation::controller_default_clips(&controller_object)?;
        for &index in &default_clips {
            wanted_clips.insert(clips.get(index).and_then(Clone::clone)
                .ok_or_else(|| invalid("default player animation is missing"))?);
        }
        controller_clips.extend(clips.iter().flatten().cloned());
        animators.push(PlayerAnimatorBinding {
            source: id.source(), game_object: component.reference("m_GameObject")?,
            avatar: avatar.source(), controller: controller.source(),
            clips: clips.into_iter().map(|id| id.map(|id| id.source())).collect(),
            default_clip_indices: default_clips.into_iter().map(u32::try_from).collect::<Result<Vec<_>,_>>()?,
        });
    }
    for selector in additional_clips {
        let (member,path_id) = selector.rsplit_once('#').ok_or_else(|| invalid("player clip expects exact APK member#signed-PathID"))?;
        let path_id: i64 = path_id.parse()?;
        let matches = controller_clips.iter().filter(|id| id.bundle == member && id.path_id == path_id).collect::<Vec<_>>();
        if matches.len() != 1 { return Err(invalid("selected clip is absent, ambiguous or outside player controllers")); }
        wanted_clips.insert(matches[0].clone());
    }
    let mut decoded_avatars = Vec::new();
    for id in avatars {
        let object = resolver.object(&id)?;
        let decoded = crate::player_animation::decode_avatar(&object,id.source())?;
        decoded_avatars.push(PlayerDecodedAsset {source:id.source(),file:store(output,"decoded/avatars",&serde_json::to_vec(&decoded)?)?});
    }
    let mut decoded_clips = Vec::new();
    let mut empty_override_clips = Vec::new();
    let mut clip_overrides = Vec::new();
    let mut processed_clips = BTreeSet::new();
    while let Some(id) = wanted_clips.pop_first() {
        if !processed_clips.insert(id.clone()) { continue; }
        let object = resolver.object(&id)?;
        retain_asset(&mut resolver,&mut assets,&id,output)?;
        if crate::player_animation::is_empty_clip_placeholder(&object)? {
            let name = text(property(&object,"m_Name")?)?;
            let asset_path = animation_maps.names.get(name).ok_or_else(||invalid("empty default clip missing from original name map"))?.clone();
            let clip = player_container_asset(assets.values(),&asset_path)?;
            wanted_clips.insert(object_id(&clip)?);
            clip_overrides.push(PlayerClipOverride {placeholder:id.source(),asset_path,clip});
            empty_override_clips.push(id.source());
            continue;
        }
        let decoded = crate::player_animation::decode_clip(&object,id.source())?;
        decoded_clips.push(PlayerDecodedAsset {source:id.source(),file:store(output,"decoded/clips",&serde_json::to_vec(&decoded)?)?});
    }
    decoded_clips.sort_by(|a,b|a.source.cmp(&b.source));
    empty_override_clips.sort();
    clip_overrides.sort_by(|a,b|a.placeholder.cmp(&b.placeholder));
    let after = source_apk(apk)?;
    if before.sha256 != after.sha256 || before.byte_count != after.byte_count { return Err(invalid("APK changed during player import")); }
    let mut retained_files = BTreeMap::new();
    list_files(output,output,&mut retained_files)?;
    Ok(PlayerRigManifest {
        schema_version: PLAYER_RIG_VERSION, source_apk:before,
        coordinate_frame: "bevy_right_handed_y_up".into(), binding_contract:PLAYER_BINDING_CONTRACT.into(),
        bundles,inventory,roots,nodes,assets:assets.into_values().collect(),animators,colliders,
        decoded_avatars,decoded_clips,empty_override_clips,animation_maps,clip_overrides,retained_files,
        limitations:vec![
            "Serialized root transforms, collider flags and numeric GameObject layers are retained separately from runtime posture overrides and activation.".into(),
            "RootBone=Bip001, HeadTop=Head and PlayerCollider=Collider follow unpatched native lookup strings. Ambiguous lookup names reject admission. Runtime dictionary mutations remain unobserved.".into(),
            "The selected role prefabs contain no serialized SkinnedMeshRenderer. Native InitPrototyle adds it; runtime dressing/model choices and skin assembly require additional source evidence.".into(),
            "Default-controller and explicitly selected clips are decoded; other clip objects and unsupported records remain in byte-identical source bundles. Animator arbitration, opaque motion channels, animation events and source frame/fixed timing are not activated.".into(),
            "This package establishes source extraction integrity and diagnostic rest/track poses, not original-runtime rendering, collision or movement parity.".into(),
        ],
    })
}

fn report(package: &Path, already_imported: bool) -> Result<PlayerImportReport,Error> {
    let loaded = load_player_rig_package(package)?;
    Ok(PlayerImportReport {
        package:package.display().to_string(), manifest_sha256:loaded.manifest_sha256,
        source_bundles:loaded.manifest.bundles.len(),source_objects:loaded.manifest.inventory.len(),
        roots:loaded.manifest.roots.iter().map(|r|r.asset_path.clone()).collect(),
        nodes:loaded.manifest.nodes.len(),colliders:loaded.manifest.colliders.len(),animators:loaded.manifest.animators.len(),
        decoded_avatars:loaded.avatars.len(),decoded_clips:loaded.clips.len(),verified_files:loaded.manifest.retained_files.len(),
        empty_override_clips:loaded.manifest.empty_override_clips.len(),
        resolved_default_overrides:loaded.manifest.clip_overrides.len(),
        already_imported,limitations:loaded.manifest.limitations,
    })
}

pub fn import(apk: &Path, output: &Path, additional_clips: &[String]) -> Result<PlayerImportReport,Error> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let parent = output.join("imported_player");
    std::fs::create_dir_all(&parent)?;
    let candidate = parent.join(format!(".player-import-{stamp}"));
    std::fs::create_dir(&candidate)?;
    let manifest = build(apk,&candidate,additional_clips)?;
    serde_json::to_writer_pretty(File::create(candidate.join("manifest.json"))?,&manifest)?;
    let verified = load_player_rig_package(&candidate)?;
    verify(apk,&candidate)?;
    let canonical = parent.join(format!("{}-v{}-{}", manifest.source_apk.sha256, PLAYER_RIG_VERSION, &verified.manifest_sha256[..16]));
    if canonical.try_exists()? {
        let previous = load_player_rig_package(&canonical)?;
        if previous.manifest_sha256 != verified.manifest_sha256 { return Err(invalid("player revision path already holds different content")); }
        // Retain the independently produced candidate as repeatability evidence.
        let repeat = output.join("../.local/player-import-repeats").join(stamp.to_string());
        std::fs::create_dir_all(repeat.parent().ok_or_else(||invalid("repeat path has no parent"))?)?;
        std::fs::rename(&candidate,repeat)?;
        return report(&canonical,true);
    }
    let backup = output.join("../.local/player-import-backups").join(stamp.to_string());
    crate::package_publish::publish_verified(&candidate,&canonical,&backup)?;
    report(&canonical,false)
}

/// Compare every retained source object and bundle against this exact APK.
/// Package loading also validates hierarchy, bindings and decoded references.
pub fn verify(apk: &Path, package: &Path) -> Result<PlayerImportReport,Error> {
    let loaded = load_player_rig_package(package)?;
    let before = source_apk(apk)?;
    if before.sha256 != loaded.manifest.source_apk.sha256 || before.byte_count != loaded.manifest.source_apk.byte_count {
        return Err(invalid("player package belongs to a different APK"));
    }
    let mut archive = Apk::open(apk)?;
    let expected = loaded.manifest.inventory.iter().map(|item| (&item.source,item)).collect::<BTreeMap<_,_>>();
    let mut actual = BTreeSet::new();
    for source in &loaded.manifest.bundles {
        let bytes = archive.member_data(&source.member)?;
        if bytes.len() as u64 != source.byte_count || format!("{:x}",Sha256::digest(&bytes)) != source.file.sha256 {
            return Err(invalid("retained player bundle differs from APK"));
        }
        let bundle = crate::unity::parse_bundle(bytes,Default::default())?;
        for (name,asset) in bundle.asset_names.iter().zip(&bundle.assets) {
            for info in &asset.objects {
                let id = ObjectId {bundle:source.member.clone(),cab:cab_name(name).into(),path_id:info.path_id}.source();
                let expected = expected.get(&id).ok_or_else(||invalid("player bundle object omitted"))?;
                if info.type_id != expected.class_id || info.byte_size != expected.byte_count
                    || format!("{:x}",Sha256::digest(asset.object_bytes(info)?)) != expected.sha256
                { return Err(invalid("player source object differs from APK")); }
                actual.insert(id);
            }
        }
    }
    if actual.len() != expected.len() { return Err(invalid("player inventory contains unrelated objects")); }
    let mut index = GlobalIndex::default();
    for bundle in &loaded.manifest.bundles {
        for cab in &bundle.cabs { index.cabs.insert(cab.clone(),bundle.member.clone()); }
    }
    let mut resolver = Resolver::new(apk,&index)?;
    for component in loaded.manifest.nodes.iter().flat_map(|n|&n.components).chain(&loaded.manifest.assets) {
        verify_raw_component(&mut resolver,component,package)?;
    }
    for animator in &loaded.manifest.animators {
        let source = resolver.object(&object_id(&animator.controller)?)?;
        let defaults = crate::player_animation::controller_default_clips(&source)?.into_iter()
            .map(u32::try_from).collect::<Result<Vec<_>,_>>()?;
        if defaults != animator.default_clip_indices { return Err(invalid("default animation selection differs from source controller")); }
    }
    for (id,decoded) in &loaded.avatars {
        let source = resolver.object(&object_id(id)?)?;
        if crate::player_animation::decode_avatar(&source,id.clone())? != *decoded { return Err(invalid("decoded Avatar differs from source")); }
    }
    for (id,decoded) in &loaded.clips {
        let source = resolver.object(&object_id(id)?)?;
        if crate::player_animation::decode_clip(&source,id.clone())? != *decoded { return Err(invalid("decoded animation differs from source")); }
    }
    for id in &loaded.manifest.empty_override_clips {
        let source = resolver.object(&object_id(id)?)?;
        if !crate::player_animation::is_empty_clip_placeholder(&source)? { return Err(invalid("source clip is not an empty override placeholder")); }
    }
    let after = source_apk(apk)?;
    if before.sha256 != after.sha256 || before.byte_count != after.byte_count { return Err(invalid("APK changed during player verification")); }
    if file_sha256(&package.join("manifest.json"))? != loaded.manifest_sha256 { return Err(invalid("player manifest changed during verification")); }
    report(package,false)
}
