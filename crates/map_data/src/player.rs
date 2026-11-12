//! Verified player-prefab evidence and decoded assets, separate from terrain.
//!
//! Saved capsule settings and rest transforms are not effective runtime posture.
//! Native InitBodyParts supplies the lookup names below; the serialized hierarchy
//! supplies their exact Transform identities. No render-frame pose is consulted.

use std::{collections::{BTreeMap, BTreeSet}, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ArchiveError, FileIdentity, PrefabNode, PrefabNodePose, SourceApk, SourceComponent,
    SourceObject, invalid, player_animation::{SourceAnimationClip, SourceAvatar,
        validate_source_animation_clip, validate_source_avatar},
    prefab::{contained_path, read_bounded, resolved_reference, retained_identity,
        source_hierarchy, valid_hash, validate_component, validate_source, verify_retained_files}};

pub const PLAYER_RIG_VERSION: u32 = 1;
pub const PLAYER_BINDING_CONTRACT: &str = "lite4.0-unpatched-init-body-parts-v1";
pub const PLAYER_SOURCE_APK: &str = "2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSourceBundle {
    pub member: String,
    pub file: FileIdentity,
    pub byte_count: u64,
    pub cabs: Vec<String>,
    pub dependencies: Vec<String>,
}

/// Every source object remains covered by an exact retained bundle, including
/// objects not yet decoded. PathIDs are always qualified by bundle and CAB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSourceObject {
    pub source: SourceObject,
    pub class_id: i32,
    pub byte_count: u32,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRigRoot {
    pub asset_path: String,
    pub game_object: SourceObject,
    /// Established by InitBodyParts/GetOriChildObjByName, never inferred from a
    /// skeleton naming convention. Missing bindings remain explicit.
    pub root_bone: Option<SourceObject>,
    pub head_top: Option<SourceObject>,
    pub player_collider: Option<SourceObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAnimatorBinding {
    pub source: SourceObject,
    pub game_object: SourceObject,
    pub avatar: SourceObject,
    pub controller: SourceObject,
    /// Includes deferred clips. This is serialized controller order, not a
    /// recovered FSM transition graph or a selection of active locomotion states.
    pub clips: Vec<Option<SourceObject>>,
    pub default_clip_indices: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDecodedAsset {
    pub source: SourceObject,
    pub file: FileIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAnimationMaps {
    pub name_map: SourceObject,
    pub state_map: SourceObject,
    pub names: BTreeMap<String, String>,
    pub states: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerClipOverride {
    pub placeholder: SourceObject,
    /// Original spelling from RoleAnimNameMap; the container preserves its
    /// canonical lower-case key independently.
    pub asset_path: String,
    pub clip: SourceObject,
}

pub fn parse_player_animation_map(text: &str) -> Result<BTreeMap<String,String>,ArchiveError> {
    struct UniqueStrings(BTreeMap<String,String>);
    impl<'de> Deserialize<'de> for UniqueStrings {
        fn deserialize<D:serde::Deserializer<'de>>(deserializer:D)->Result<Self,D::Error> {
            struct MapVisitor;
            impl<'de> serde::de::Visitor<'de> for MapVisitor {
                type Value = UniqueStrings;
                fn expecting(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result { f.write_str("a unique string map") }
                fn visit_map<M:serde::de::MapAccess<'de>>(self,mut map:M)->Result<Self::Value,M::Error> {
                    let mut result = BTreeMap::new();
                    while let Some((key,value)) = map.next_entry::<String,String>()? {
                        if result.len() >= 100_000 || result.insert(key,value).is_some() {
                            return Err(serde::de::Error::custom("duplicate/oversized animation map"));
                        }
                    }
                    Ok(UniqueStrings(result))
                }
            }
            deserializer.deserialize_map(MapVisitor)
        }
    }
    if text.len() > 4 * 1024 * 1024 { return Err(invalid("animation map exceeds byte bound")); }
    serde_json::from_str::<UniqueStrings>(text).map(|map|map.0)
        .map_err(|error|invalid(format!("animation map: {error}")))
}

pub fn player_container_asset<'a>(
    assets: impl IntoIterator<Item=&'a SourceComponent>, path: &str,
) -> Result<SourceObject,ArchiveError> {
    let mut found = None;
    for asset in assets {
        if asset.class_id != 142 { continue; }
        let entries = asset.fields.get("m_Container").and_then(serde_json::Value::as_array)
            .ok_or_else(||invalid("source bundle container missing"))?;
        for value in entries {
            let pair = value.as_array().ok_or_else(||invalid("invalid source container entry"))?;
            if pair.len() != 2 { return Err(invalid("invalid source container pair")); }
            if pair[0].as_str().is_some_and(|name|name.eq_ignore_ascii_case(path)) {
                let target = resolved_reference(pair[1].get("asset").ok_or_else(||invalid("source container asset missing"))?)?;
                if found.replace(target).is_some() { return Err(invalid("ambiguous source container asset")); }
            }
        }
    }
    found.ok_or_else(||invalid(format!("required container asset missing: {path}")))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlayerColliderShape {
    Capsule { center: [f32; 3], radius: f32, height: f32, direction: u8 },
    Sphere { center: [f32; 3], radius: f32 },
    Box { center: [f32; 3], size: [f32; 3] },
    /// Source geometry remains in the retained bundle until explicitly decoded.
    Mesh { mesh: SourceObject, convex: bool },
    UnresolvedMesh { file_id: i64, path_id: String, detail: String, convex: bool },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializedPlayerCollider {
    pub source: SourceObject,
    pub game_object: SourceObject,
    pub layer: u8,
    pub enabled: bool,
    pub is_trigger: bool,
    /// Shape values remain Unity local values. The hierarchy matrices are
    /// already converted; consumers reflect the local center exactly once.
    pub shape: PlayerColliderShape,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRigManifest {
    pub schema_version: u32,
    pub source_apk: SourceApk,
    pub coordinate_frame: String,
    pub binding_contract: String,
    pub bundles: Vec<PlayerSourceBundle>,
    pub inventory: Vec<PlayerSourceObject>,
    pub roots: Vec<PlayerRigRoot>,
    pub nodes: Vec<PrefabNode>,
    /// Full controller, Avatar, selected clip, bundle-container and script
    /// records. Unsupported fields and blobs retain their original bytes.
    pub assets: Vec<SourceComponent>,
    pub animators: Vec<PlayerAnimatorBinding>,
    pub colliders: Vec<SerializedPlayerCollider>,
    pub decoded_avatars: Vec<PlayerDecodedAsset>,
    pub decoded_clips: Vec<PlayerDecodedAsset>,
    /// Structurally empty source clips used by a dummy controller. They are
    /// retained, never replaced with a same-name clip or a fabricated pose.
    pub empty_override_clips: Vec<SourceObject>,
    pub animation_maps: PlayerAnimationMaps,
    pub clip_overrides: Vec<PlayerClipOverride>,
    pub retained_files: BTreeMap<String, String>,
    pub limitations: Vec<String>,
}

#[derive(Debug)]
pub struct PlayerRigPackage {
    pub root: PathBuf,
    pub manifest: PlayerRigManifest,
    pub manifest_sha256: String,
    /// Aligned with manifest.nodes. These are serialized rest poses only.
    pub rest_poses: Vec<PrefabNodePose>,
    pub avatars: BTreeMap<SourceObject, SourceAvatar>,
    pub clips: BTreeMap<SourceObject, SourceAnimationClip>,
}

impl PlayerRigPackage {
    pub fn node_index(&self, source: &SourceObject) -> Option<usize> {
        self.manifest.nodes.iter().position(|node| &node.source == source)
    }

    pub fn transform_index(&self, source: &SourceObject) -> Option<usize> {
        self.manifest.nodes.iter().position(|node| &node.transform == source)
    }

    pub fn root_named(&self, name: &str) -> Option<&PlayerRigRoot> {
        self.manifest.roots.iter().find(|root| {
            self.node_index(&root.game_object).is_some_and(|i| self.manifest.nodes[i].name == name)
        })
    }

    pub fn asset_path(&self, identity: &FileIdentity) -> Result<PathBuf, ArchiveError> {
        retained_identity(&self.manifest.retained_files, identity)?;
        contained_path(&self.root, &identity.path)
    }
}

/// Resolve an established native lookup name within one complete source root.
/// Ambiguous matches fail; no arbitrary first bone is admitted.
pub fn player_named_transform(
    nodes: &[PrefabNode], root: &SourceObject, name: &str,
) -> Result<Option<SourceObject>, ArchiveError> {
    let indices = nodes.iter().enumerate().map(|(i, node)| (&node.source, i))
        .collect::<BTreeMap<_, _>>();
    let mut pending = vec![*indices.get(root).ok_or_else(|| invalid("missing player root"))?];
    let mut visited = BTreeSet::new();
    let mut result = None;
    while let Some(index) = pending.pop() {
        if !visited.insert(index) || visited.len() > 100_000 {
            return Err(invalid("cyclic/duplicate player child"));
        }
        let node = &nodes[index];
        if node.name == name && result.replace(node.transform.clone()).is_some() {
            return Err(invalid(format!("ambiguous player binding {name}")));
        }
        for child in &node.children {
            pending.push(*indices.get(child).ok_or_else(|| invalid("missing player child"))?);
        }
    }
    Ok(result)
}

fn number(component: &SourceComponent, name: &str) -> Result<f32, ArchiveError> {
    let value = component.fields.get(name).and_then(serde_json::Value::as_f64)
        .ok_or_else(|| invalid(format!("missing collider {name}")))? as f32;
    if !value.is_finite() || value < 0. {
        return Err(invalid(format!("invalid collider {name}")));
    }
    Ok(value)
}

/// Derive saved settings directly from the verified source components. No
/// standing/crouch dimensions, scale clamp or active-state override is applied.
pub fn serialized_player_colliders(nodes: &[PrefabNode]) -> Result<Vec<SerializedPlayerCollider>, ArchiveError> {
    let mut result = Vec::new();
    for node in nodes {
        for component in &node.components {
            let shape = match component.class_id {
                136 => {
                    let direction = u8::try_from(component.integer("m_Direction")?)
                        .map_err(|_| invalid("invalid capsule direction"))?;
                    if direction > 2 { return Err(invalid("invalid capsule direction")); }
                    PlayerColliderShape::Capsule {
                        center: component.vector3("m_Center")?, radius: number(component, "m_Radius")?,
                        height: number(component, "m_Height")?, direction,
                    }
                }
                135 => PlayerColliderShape::Sphere {
                    center: component.vector3("m_Center")?, radius: number(component, "m_Radius")?,
                },
                65 => {
                    let size = component.vector3("m_Size")?;
                    if size.iter().any(|v| *v < 0.) { return Err(invalid("negative box size")); }
                    PlayerColliderShape::Box { center: component.vector3("m_Center")?, size }
                }
                64 => {
                    let convex = component.boolean("m_Convex")?;
                    let reference = component.fields.get("m_Mesh").ok_or_else(||invalid("missing collider mesh reference"))?;
                    if reference.get("state").and_then(serde_json::Value::as_str) == Some("unresolved") {
                        let file_id = reference.get("file_id").and_then(serde_json::Value::as_i64).ok_or_else(||invalid("unresolved mesh file ID missing"))?;
                        let path_id = reference.get("path_id").and_then(serde_json::Value::as_str).ok_or_else(||invalid("unresolved mesh path ID missing"))?.to_owned();
                        if path_id.parse::<i64>().ok().is_none_or(|id|id == 0) { return Err(invalid("invalid unresolved mesh path ID")); }
                        PlayerColliderShape::UnresolvedMesh {
                            file_id,path_id,convex,
                            detail:reference.get("detail").and_then(serde_json::Value::as_str).ok_or_else(||invalid("unresolved mesh detail missing"))?.to_owned(),
                        }
                    } else { PlayerColliderShape::Mesh {mesh:component.reference("m_Mesh")?,convex} }
                }
                _ => continue,
            };
            result.push(SerializedPlayerCollider {
                source: component.source.clone(), game_object: node.source.clone(),
                layer: u8::try_from(node.layer).ok().filter(|layer| *layer < 32)
                    .ok_or_else(|| invalid("invalid player source layer"))?,
                enabled: component.boolean("m_Enabled")?, is_trigger: component.boolean("m_IsTrigger")?, shape,
            });
        }
    }
    result.sort_by(|a,b| a.source.cmp(&b.source));
    Ok(result)
}

fn read_asset<T: serde::de::DeserializeOwned>(root: &Path, files: &BTreeMap<String,String>, asset: &PlayerDecodedAsset)
    -> Result<T, ArchiveError>
{
    retained_identity(files, &asset.file)?;
    let bytes = read_bounded(&contained_path(root, &asset.file.path)?, 64 * 1024 * 1024)?;
    if format!("{:x}", Sha256::digest(&bytes)) != asset.file.sha256 {
        return Err(invalid("decoded player asset changed while loading"));
    }
    serde_json::from_slice(&bytes).map_err(|error| invalid(format!("decoded player asset: {error}")))
}

pub fn load_player_rig_package(package: &Path) -> Result<PlayerRigPackage, ArchiveError> {
    let root = package.canonicalize()?;
    let manifest_path = contained_path(&root, "manifest.json")?;
    let bytes = read_bounded(&manifest_path, 64 * 1024 * 1024)?;
    let manifest: PlayerRigManifest = serde_json::from_slice(&bytes)
        .map_err(|error| invalid(format!("player manifest: {error}")))?;
    if manifest.schema_version != PLAYER_RIG_VERSION
        || manifest.coordinate_frame != "bevy_right_handed_y_up"
        || manifest.binding_contract != PLAYER_BINDING_CONTRACT
        || manifest.source_apk.sha256 != PLAYER_SOURCE_APK
        || manifest.source_apk.byte_count == 0
        || manifest.nodes.len() > 100_000 || manifest.retained_files.len() > 100_000
        || manifest.inventory.len() > 100_000 || manifest.bundles.is_empty()
        || manifest.bundles.len() > 64
    { return Err(invalid("unsupported player conventions/version/limits")); }
    verify_retained_files(&root, &manifest.retained_files)?;
    let declared_roots = manifest.roots.iter().map(|r| r.game_object.clone()).collect::<Vec<_>>();
    let rest_poses = source_hierarchy(&manifest.nodes, &declared_roots, &manifest.retained_files)?;
    let mut bundles = BTreeMap::new();
    for bundle in &manifest.bundles {
        if bundles.insert(&bundle.member, bundle).is_some() || bundle.member.is_empty()
            || bundle.cabs.is_empty() || bundle.cabs.iter().collect::<BTreeSet<_>>().len() != bundle.cabs.len()
        { return Err(invalid("duplicate/invalid player bundle")); }
        retained_identity(&manifest.retained_files, &bundle.file)?;
        if contained_path(&root, &bundle.file.path)?.metadata()?.len() != bundle.byte_count {
            return Err(invalid("player bundle byte count differs"));
        }
    }
    for bundle in &manifest.bundles {
        if bundle.dependencies.iter().any(|dependency| !bundles.contains_key(&format!("assets/{dependency}"))) {
            return Err(invalid("missing required player bundle dependency"));
        }
    }
    let mut inventory = BTreeMap::new();
    for object in &manifest.inventory {
        validate_source(&object.source)?;
        if inventory.insert(&object.source, object).is_some() || !valid_hash(&object.sha256)
            || !bundles.get(&object.source.bundle).is_some_and(|bundle| bundle.cabs.contains(&object.source.cab))
        { return Err(invalid("invalid player source inventory")); }
    }
    let mut components = BTreeMap::new();
    for component in manifest.nodes.iter().flat_map(|node| &node.components).chain(&manifest.assets) {
        validate_component(&manifest.retained_files, component)?;
        let source = inventory.get(&component.source).ok_or_else(|| invalid("component missing from source inventory"))?;
        if source.class_id != component.class_id || source.sha256 != component.serialized.sha256
            || components.insert(&component.source, component).is_some()
            || contained_path(&root, &component.serialized.path)?.metadata()?.len() != u64::from(source.byte_count)
        { return Err(invalid("player component identity differs from source")); }
    }
    for selected in &manifest.roots {
        if player_container_asset(&manifest.assets,&selected.asset_path)? != selected.game_object {
            return Err(invalid("selected player root differs from bundle container"));
        }
        for (name, expected) in [("Bip001", &selected.root_bone), ("Head", &selected.head_top), ("Collider", &selected.player_collider)] {
            if &player_named_transform(&manifest.nodes, &selected.game_object, name)? != expected {
                return Err(invalid(format!("player {name} binding differs from source lookup")));
            }
        }
    }
    if serialized_player_colliders(&manifest.nodes)? != manifest.colliders {
        return Err(invalid("saved player collider geometry/layers/flags differ"));
    }
    let expected_animators = components.values().filter(|c| c.class_id == 95).map(|c| &c.source).collect::<BTreeSet<_>>();
    let mut animators = BTreeSet::new();
    let mut referenced_clips = BTreeSet::new();
    let mut referenced_avatars = BTreeSet::new();
    let mut default_clips = BTreeSet::new();
    for animator in &manifest.animators {
        let evidence = components.get(&animator.source).ok_or_else(|| invalid("missing Animator"))?;
        if !animators.insert(&animator.source) || evidence.class_id != 95
            || evidence.reference("m_GameObject")? != animator.game_object
            || evidence.reference("m_Avatar")? != animator.avatar
            || evidence.reference("m_Controller")? != animator.controller
        { return Err(invalid("Animator binding differs from source")); }
        let avatar = components.get(&animator.avatar).ok_or_else(|| invalid("missing required Avatar"))?;
        let controller = components.get(&animator.controller).ok_or_else(|| invalid("missing required controller"))?;
        if avatar.class_id != 90 || controller.class_id != 91 { return Err(invalid("invalid Animator dependencies")); }
        let clips = controller.fields.get("m_AnimationClips").and_then(serde_json::Value::as_array)
            .ok_or_else(|| invalid("controller clip references missing"))?.iter()
            .map(|value| {
                if value.get("state").and_then(serde_json::Value::as_str) == Some("null") {
                    Ok(None)
                } else { resolved_reference(value).map(Some) }
            }).collect::<Result<Vec<_>,ArchiveError>>()?;
        if clips != animator.clips { return Err(invalid("controller clip order differs")); }
        if animator.default_clip_indices.iter().collect::<BTreeSet<_>>().len() != animator.default_clip_indices.len() {
            return Err(invalid("duplicate default animation index"));
        }
        for &index in &animator.default_clip_indices {
            default_clips.insert(animator.clips.get(index as usize).and_then(Option::as_ref)
                .ok_or_else(||invalid("required default clip is null/missing"))?);
        }
        for clip in animator.clips.iter().flatten() {
            if !inventory.get(clip).is_some_and(|record| record.class_id == 74) {
                return Err(invalid("required controller clip absent from retained bundles"));
            }
            referenced_clips.insert(clip);
        }
        referenced_avatars.insert(&animator.avatar);
    }
    if animators != expected_animators { return Err(invalid("incomplete Animator inventory")); }
    let maps = &manifest.animation_maps;
    for (path,id,values) in [
        ("Assets/Roles/RoleAnimNameMap.json",&maps.name_map,&maps.names),
        ("Assets/Roles/RoleAnimState2ClipMap.json",&maps.state_map,&maps.states),
    ] {
        let component = components.get(id).ok_or_else(||invalid("required animation map missing"))?;
        if component.class_id != 49 || &player_container_asset(&manifest.assets,path)? != id {
            return Err(invalid("animation map identity differs"));
        }
        let script = component.fields.get("m_Script").and_then(serde_json::Value::as_str)
            .ok_or_else(||invalid("animation map TextAsset is not text"))?;
        if &parse_player_animation_map(script)? != values { return Err(invalid("animation map differs from source TextAsset")); }
    }
    let mut override_sources = BTreeSet::new();
    for entry in &manifest.clip_overrides {
        let placeholder = components.get(&entry.placeholder).ok_or_else(||invalid("override source missing"))?;
        let name = placeholder.fields.get("m_Name").and_then(serde_json::Value::as_str).ok_or_else(||invalid("placeholder name missing"))?;
        if !override_sources.insert(&entry.placeholder) || !manifest.empty_override_clips.contains(&entry.placeholder)
            || maps.names.get(name) != Some(&entry.asset_path)
            || player_container_asset(&manifest.assets,&entry.asset_path)? != entry.clip
            || !inventory.get(&entry.clip).is_some_and(|item|item.class_id == 74)
        { return Err(invalid("animation override differs from source map/container")); }
        referenced_clips.insert(&entry.clip);
    }
    let mut avatars = BTreeMap::new();
    for asset in &manifest.decoded_avatars {
        let avatar: SourceAvatar = read_asset(&root, &manifest.retained_files, asset)?;
        validate_source_avatar(&avatar)?;
        if !referenced_avatars.contains(&asset.source) || avatar.source != asset.source
            || avatars.insert(asset.source.clone(), avatar).is_some()
        { return Err(invalid("invalid/duplicate decoded Avatar")); }
    }
    if avatars.keys().collect::<BTreeSet<_>>() != referenced_avatars { return Err(invalid("missing decoded Avatar")); }
    let mut clips = BTreeMap::new();
    for asset in &manifest.decoded_clips {
        let clip: SourceAnimationClip = read_asset(&root, &manifest.retained_files, asset)?;
        validate_source_animation_clip(&clip)?;
        if !referenced_clips.contains(&asset.source) || clip.source != asset.source
            || !components.get(&asset.source).is_some_and(|c| c.class_id == 74 && c.fields.get("m_Name").and_then(serde_json::Value::as_str) == Some(&clip.name))
            || clips.insert(asset.source.clone(), clip).is_some()
        { return Err(invalid("invalid/duplicate decoded clip")); }
    }
    let mut placeholders = BTreeSet::new();
    for source in &manifest.empty_override_clips {
        let evidence = components.get(source).ok_or_else(||invalid("missing empty clip source"))?;
        if !referenced_clips.contains(source) || !placeholders.insert(source)
            || clips.contains_key(source) || evidence.class_id != 74
            || !evidence.fields.get("m_ClipBindingConstant").and_then(|v|v.get("genericBindings"))
                .and_then(serde_json::Value::as_array).is_some_and(Vec::is_empty)
        { return Err(invalid("invalid/duplicate empty override clip")); }
    }
    if default_clips.iter().any(|source| !clips.contains_key(*source) && !placeholders.contains(*source)) {
        return Err(invalid("required default animation dependency not decoded or retained as an empty override"));
    }
    if override_sources != placeholders || manifest.clip_overrides.iter().any(|entry|!clips.contains_key(&entry.clip)) {
        return Err(invalid("missing required decoded override animation"));
    }
    if read_bounded(&manifest_path, 64 * 1024 * 1024)? != bytes { return Err(invalid("player manifest changed while loading")); }
    Ok(PlayerRigPackage {root, manifest, manifest_sha256: format!("{:x}",Sha256::digest(&bytes)), rest_poses, avatars, clips})
}
