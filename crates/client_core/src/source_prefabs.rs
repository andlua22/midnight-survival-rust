//! Bevy presentation of verified prefab packages. Preview transforms are not world placements.

use bevy::{
    asset::Asset,
    camera::visibility::VisibilitySystems,
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    light::{NotShadowCaster, NotShadowReceiver},
    math::Affine2,
    prelude::*,
    render::render_resource::Face,
    transform::TransformSystems,
};
use midnight_map_data::{
    ArchiveError, PrefabManifest, PrefabMaterial, PrefabPackage, SourceComponent, SourceObject,
    TextureAsset,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

fn invalid(message: impl Into<String>) -> ArchiveError {
    ArchiveError::Invalid(message.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefabRenderPolicy {
    /// Reject any unsupported rendering case before adding assets or entities.
    Strict,
    /// Diagnostic preview only: retain metadata and explicitly report omitted renderers.
    SupportedPartsPreview,
}

#[derive(Clone, Debug, Serialize)]
pub struct PrefabUnsupported {
    pub renderer: Option<SourceObject>,
    pub mesh_part: Option<String>,
    pub detail: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct PrefabSupportReport {
    pub source_nodes: usize,
    pub source_renderers: usize,
    pub prepared_parts: usize,
    pub lod_groups: usize,
    pub unsupported: Vec<PrefabUnsupported>,
    pub shading_policy: String,
}

#[derive(Asset, TypePath)]
pub struct SourcePrefabDocument {
    pub manifest: PrefabManifest,
    pub manifest_sha256: String,
}

#[derive(Component)]
pub struct SourcePrefabInstance {
    /// Keeps the complete source metadata alive without copying it per mesh/node.
    pub document: Handle<SourcePrefabDocument>,
    pub preview: bool,
    pub placement_source: Option<SourceObject>,
}

#[derive(Component, Clone, Debug)]
pub struct SourcePrefabNode {
    pub source: SourceObject,
    pub name: String,
    pub node_index: usize,
    pub original_local_transform: [f32; 16],
    pub active_self: bool,
    pub layer: i64,
}

#[derive(Component, Clone, Debug)]
pub struct SourcePrefabPart {
    pub renderer: SourceObject,
    pub material: SourceObject,
    pub mesh_part: String,
    pub source_enabled: bool,
    pub casts_shadows: bool,
    pub receives_shadows: bool,
}

/// An explicit single camera owns source LOD selection for these instances.
#[derive(Component)]
pub struct SourcePrefabLodCamera;

#[derive(Resource, Default, Debug)]
pub struct SourcePrefabLodStatus {
    pub error: Option<String>,
}

#[derive(Component)]
struct SourceLodGroup {
    reference: Vec3,
    size: f32,
    thresholds: Vec<f32>,
    levels: Vec<Vec<Entity>>,
}

#[derive(Clone)]
struct PreparedLod {
    node: usize,
    reference: Vec3,
    size: f32,
    thresholds: Vec<f32>,
    renderers: Vec<Vec<SourceObject>>,
}

struct PreparedPart {
    node: usize,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    mirrored_material: Handle<StandardMaterial>,
    metadata: SourcePrefabPart,
}

pub struct PreparedPrefab {
    pub document: Handle<SourcePrefabDocument>,
    pub report: PrefabSupportReport,
    pub textures: Vec<(Handle<Image>, TextureAsset)>,
    nodes: Vec<SourcePrefabNode>,
    parents: Vec<Option<usize>>,
    local: Vec<Transform>,
    poses: Vec<Mat4>,
    parts: Vec<PreparedPart>,
    lods: Vec<PreparedLod>,
}

pub struct SourcePrefabPlugin;
impl Plugin for SourcePrefabPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SourcePrefabDocument>()
            .init_resource::<SourcePrefabLodStatus>()
            .add_systems(
                PostUpdate,
                update_source_lods
                    .after(TransformSystems::Propagate)
                    .before(VisibilitySystems::VisibilityPropagate)
                    .before(VisibilitySystems::CheckVisibility),
            );
    }
}

/// Source screen-height threshold selection, including the final empty/cull level.
pub fn select_source_lod(thresholds: &[f32], relative_height: f32) -> Option<usize> {
    thresholds
        .iter()
        .position(|threshold| relative_height >= *threshold)
}

#[allow(
    clippy::type_complexity,
    reason = "Bevy camera query combines explicit source-LOD and Camera3d filters"
)]
fn update_source_lods(
    camera: Query<(&GlobalTransform, &Projection), (With<Camera3d>, With<SourcePrefabLodCamera>)>,
    groups: Query<(&GlobalTransform, &SourceLodGroup)>,
    mut parts: Query<(&SourcePrefabPart, &mut Visibility)>,
    mut status: ResMut<SourcePrefabLodStatus>,
) {
    if groups.is_empty() {
        status.error = None;
        return;
    }
    let Ok((camera_transform, Projection::Perspective(projection))) = camera.single() else {
        status.error = Some("source LOD requires exactly one marked perspective camera".into());
        for (_, group) in &groups {
            for entity in group.levels.iter().flatten() {
                if let Ok((_, mut visibility)) = parts.get_mut(*entity) {
                    *visibility = Visibility::Hidden;
                }
            }
        }
        return;
    };
    status.error = None;
    for (transform, group) in &groups {
        let center = transform.transform_point(group.reference);
        let (_, _, translation) = camera_transform.to_scale_rotation_translation();
        let (scale, _, _) = transform.to_scale_rotation_translation();
        let size = group.size * scale.abs().max_element();
        let distance = translation.distance(center).max(0.0001);
        let height = size / (2.0 * distance * (projection.fov * 0.5).tan());
        let selected = select_source_lod(&group.thresholds, height);
        let all = group
            .levels
            .iter()
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>();
        let visible = selected.and_then(|level| group.levels.get(level));
        for entity in all {
            if let Ok((part, mut visibility)) = parts.get_mut(entity) {
                let desired =
                    if part.source_enabled && visible.is_some_and(|list| list.contains(&entity)) {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                if *visibility != desired {
                    *visibility = desired;
                }
            }
        }
    }
}

fn component<'a>(
    package: &'a PrefabPackage,
    source: &SourceObject,
) -> Result<&'a SourceComponent, ArchiveError> {
    package
        .manifest
        .nodes
        .iter()
        .flat_map(|node| &node.components)
        .find(|component| &component.source == source)
        .ok_or_else(|| invalid("source renderer component absent"))
}

fn flag(component: &SourceComponent, field: &str) -> Result<bool, ArchiveError> {
    match component.fields.get(field) {
        Some(serde_json::Value::Bool(value)) => Ok(*value),
        Some(value) if value.as_i64() == Some(0) => Ok(false),
        Some(value) if value.as_i64() == Some(1) => Ok(true),
        _ => Err(invalid(format!(
            "{}: missing/unsupported flag {field}",
            component.source.path_id
        ))),
    }
}

fn transform(matrix: [f32; 16]) -> Result<Transform, ArchiveError> {
    midnight_map_data::validate_prefab_affine(&matrix)?;
    let matrix = Mat4::from_cols_array(&matrix);
    let transform = Transform::from_matrix(matrix);
    if !transform.to_matrix().abs_diff_eq(matrix, 0.0002) {
        return Err(invalid(
            "local prefab shear cannot be represented by Bevy Transform",
        ));
    }
    Ok(transform)
}

fn source_uv_transform(scale: [f32; 2], offset: [f32; 2]) -> Affine2 {
    // Mesh UVs and decoded images use top-left V; conjugate Unity's bottom-left ST.
    Affine2::from_scale_angle_translation(
        Vec2::from_array(scale),
        0.0,
        Vec2::new(offset[0], 1.0 - scale[1] - offset[1]),
    )
}

fn source_lod_reference(point: [f32; 3]) -> Vec3 {
    Vec3::new(point[0], point[1], -point[2])
}

fn lods(package: &PrefabPackage) -> Result<Vec<PreparedLod>, ArchiveError> {
    let mut result = Vec::new();
    let mut owners = BTreeMap::new();
    for (node, source_node) in package.manifest.nodes.iter().enumerate() {
        for component in source_node
            .components
            .iter()
            .filter(|component| component.class_id == 205)
        {
            if !component.boolean("m_Enabled")? {
                continue;
            }
            if component.integer("m_FadeMode")? != 0 || component.boolean("m_AnimateCrossFading")? {
                return Err(invalid("source LOD crossfading is unsupported"));
            }
            let size = component
                .fields
                .get("m_Size")
                .and_then(|value| value.as_f64())
                .ok_or_else(|| invalid("missing source LOD size"))? as f32;
            if !size.is_finite() || size <= 0.0 {
                return Err(invalid("invalid source LOD size"));
            }
            let mut thresholds = Vec::new();
            let mut renderers = Vec::new();
            for level in component
                .fields
                .get("m_LODs")
                .and_then(|value| value.as_array())
                .ok_or_else(|| invalid("missing source LOD levels"))?
            {
                let threshold = level
                    .get("screenRelativeHeight")
                    .and_then(|value| value.as_f64())
                    .ok_or_else(|| invalid("missing source LOD threshold"))?
                    as f32;
                if !threshold.is_finite()
                    || !(0.0..=1.0).contains(&threshold)
                    || thresholds
                        .last()
                        .is_some_and(|previous| *previous <= threshold)
                {
                    return Err(invalid(
                        "source LOD thresholds are not finite and descending",
                    ));
                }
                thresholds.push(threshold);
                let mut refs = Vec::new();
                for entry in level
                    .get("renderers")
                    .and_then(|value| value.as_array())
                    .ok_or_else(|| invalid("missing source LOD renderers"))?
                {
                    let pointer = entry
                        .get("renderer")
                        .ok_or_else(|| invalid("missing source LOD renderer pointer"))?;
                    if pointer.get("state").and_then(|value| value.as_str()) != Some("resolved") {
                        return Err(invalid("unresolved source LOD renderer"));
                    }
                    let source: SourceObject = serde_json::from_value(
                        pointer
                            .get("target")
                            .cloned()
                            .ok_or_else(|| invalid("missing source LOD renderer target"))?,
                    )
                    .map_err(|error| invalid(error.to_string()))?;
                    if !package
                        .manifest
                        .renderers
                        .iter()
                        .any(|renderer| renderer.source == source)
                    {
                        return Err(invalid("source LOD references an unextracted renderer"));
                    }
                    if owners
                        .insert(source.clone(), node)
                        .is_some_and(|owner| owner != node)
                    {
                        return Err(invalid("renderer shared by multiple source LOD groups"));
                    }
                    refs.push(source);
                }
                renderers.push(refs);
            }
            result.push(PreparedLod {
                node,
                reference: source_lod_reference(component.vector3("m_LocalReferencePoint")?),
                size,
                thresholds,
                renderers,
            });
        }
    }
    Ok(result)
}

fn source_material(material: &PrefabMaterial) -> Result<StandardMaterial, ArchiveError> {
    let source = &material.material;
    if !material.unsupported_textures.is_empty() {
        return Err(invalid(format!(
            "{}: unsupported samplers {:?}",
            source.name,
            material.unsupported_textures.keys().collect::<Vec<_>>()
        )));
    }
    if !matches!(
        source.shader_name.as_str(),
        "_URP/LambertLit" | "_URP/SimpleLit"
    ) {
        return Err(invalid(format!(
            "{}: unsupported shader {}",
            source.name, source.shader_name
        )));
    }
    if !source.valid_keywords.is_empty() {
        return Err(invalid(format!(
            "{}: unsupported shader keywords {:?}",
            source.name, source.valid_keywords
        )));
    }
    for (name, binding) in &source.textures {
        if name != "_BaseMap" && binding.texture.is_some() {
            return Err(invalid(format!(
                "{}: unsupported texture binding {name}",
                source.name
            )));
        }
    }
    for property in [
        "_EnableShadowOffset",
        "_EmissionEnabled",
        "_EnablePattern",
        "_PanOrPulsateEmission",
        "_AlphaClip",
        "_Surface",
    ] {
        if source.float(property).is_some_and(|value| value != 0.0) {
            return Err(invalid(format!(
                "{}: unsupported active property {property}",
                source.name
            )));
        }
    }
    if source.float("_ZWrite") != Some(1.0)
        || source.float("_SrcBlend").is_some_and(|value| value != 1.0)
        || source.float("_DstBlend").is_some_and(|value| value != 0.0)
    {
        return Err(invalid(format!(
            "{}: unsupported blend/depth mode",
            source.name
        )));
    }
    let binding = source
        .textures
        .get("_BaseMap")
        .ok_or_else(|| invalid("missing source base-map binding"))?;
    if binding.texture.is_none() {
        return Err(invalid(
            "source base-map default texture is not yet supported",
        ));
    }
    let color = source
        .colors
        .get("_Color")
        .copied()
        .or_else(|| source.shader_defaults.get("_Color").copied())
        .ok_or_else(|| invalid("missing declared source color"))?;
    let brightness = source.float("_Brightness").unwrap_or(1.0);
    if !brightness.is_finite() || brightness < 0.0 {
        return Err(invalid("invalid source brightness"));
    }
    let cull_mode = match source.float("_Cull") {
        Some(0.0) => None,
        Some(1.0) => Some(Face::Front),
        Some(2.0) => Some(Face::Back),
        _ => return Err(invalid("unsupported source cull mode")),
    };
    // A declared Bevy adaptation: source maps/colors/UVs are preserved, while
    // original URP Lambert/Blinn–Phong and baked lighting remain unported.
    Ok(StandardMaterial {
        base_color: Color::srgba(
            color[0] * brightness,
            color[1] * brightness,
            color[2] * brightness,
            color[3],
        ),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        cull_mode,
        double_sided: cull_mode.is_none(),
        uv_transform: source_uv_transform(binding.scale, binding.offset),
        ..default()
    })
}

fn texture_sampler(texture: &TextureAsset) -> Result<ImageSamplerDescriptor, ArchiveError> {
    let source = &texture.sampler;
    if source.aniso > 1 || source.mip_bias != 0.0 {
        return Err(invalid("source texture anisotropy/mip bias is unsupported"));
    }
    let wrap = |value| match value {
        0 => Ok(ImageAddressMode::Repeat),
        1 => Ok(ImageAddressMode::ClampToEdge),
        2 => Ok(ImageAddressMode::MirrorRepeat),
        _ => Err(invalid("unsupported source wrap mode")),
    };
    let (filter, mipmap_filter) = match source.filter_mode {
        0 => (ImageFilterMode::Nearest, ImageFilterMode::Nearest),
        1 => (ImageFilterMode::Linear, ImageFilterMode::Nearest),
        2 => (ImageFilterMode::Linear, ImageFilterMode::Linear),
        _ => return Err(invalid("unsupported source filter mode")),
    };
    Ok(ImageSamplerDescriptor {
        address_mode_u: wrap(source.wrap[0])?,
        address_mode_v: wrap(source.wrap[1])?,
        address_mode_w: wrap(source.wrap[2])?,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter,
        ..default()
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "Preparation receives the verified package and distinct Bevy asset collections"
)]
pub fn prepare_prefab(
    package: &PrefabPackage,
    asset_root: &Path,
    server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    documents: &mut Assets<SourcePrefabDocument>,
    policy: PrefabRenderPolicy,
) -> Result<PreparedPrefab, ArchiveError> {
    let asset_root = asset_root.canonicalize()?;
    let local = package
        .manifest
        .nodes
        .iter()
        .map(|node| transform(node.bevy_local_transform))
        .collect::<Result<Vec<_>, _>>()?;
    let lods = lods(package)?;
    let mut report = PrefabSupportReport { source_nodes: package.manifest.nodes.len(), source_renderers: package.manifest.renderers.len(), lod_groups: lods.len(), shading_policy: "Explicit Bevy diffuse preview adaptation for URP LambertLit/SimpleLit; source shader execution and baked/probe lighting are not reproduced.".into(), ..default() };
    if package
        .manifest
        .nodes
        .iter()
        .flat_map(|node| &node.components)
        .any(|component| component.class_id == 137)
    {
        return Err(invalid("source skinned mesh renderers are unsupported"));
    }
    let mut plans = Vec::new();
    for renderer in &package.manifest.renderers {
        let plan: Result<_, ArchiveError> = (|| {
            let node = package
                .node_index(&renderer.game_object)
                .ok_or_else(|| invalid("renderer GameObject absent"))?;
            let source = component(package, &renderer.source)?;
            let enabled = flag(source, "m_Enabled")?;
            let casts_shadows = match source.integer("m_CastShadows")? {
                0 => false,
                1 => true,
                _ => return Err(invalid("two-sided/shadows-only casting is unsupported")),
            };
            let receives_shadows = flag(source, "m_ReceiveShadows")?;
            if renderer.materials.len() != renderer.mesh_parts.len() {
                return Err(invalid(
                    "extra source material slots need unsupported repeated-submesh rendering",
                ));
            }
            for name in ["m_LightmapIndex", "m_LightmapIndexDynamic"] {
                if source.integer(name)? != 65535 {
                    return Err(invalid("source baked lightmap binding is unsupported"));
                }
            }
            let mut parts = Vec::new();
            for (part, name) in renderer.mesh_parts.iter().enumerate() {
                let part_plan: Result<_, ArchiveError> = (|| {
                    let material = &renderer.materials[part];
                    let presentation = source_material(material)?;
                    let texture = material.material.textures["_BaseMap"]
                        .texture
                        .as_ref()
                        .ok_or_else(|| invalid("source base map absent"))?;
                    let sampler = texture_sampler(texture)?;
                    let is_srgb = match texture.unity_color_space {
                        0 => false,
                        1 => true,
                        _ => return Err(invalid("unsupported source color space")),
                    };
                    let gpu = texture
                        .gpu
                        .as_ref()
                        .ok_or_else(|| invalid("source mip archive absent"))?;
                    let path = package.texture_path(&gpu.path)?;
                    let relative = path
                        .strip_prefix(&asset_root)
                        .map_err(|_| {
                            invalid("prefab textures must be under the configured Bevy asset root")
                        })?
                        .to_string_lossy()
                        .replace('\\', "/");
                    let record = package
                        .records
                        .iter()
                        .find(|record| &record.name == name)
                        .ok_or_else(|| invalid("source mesh part absent"))?;
                    Ok((
                        name.clone(),
                        record,
                        material,
                        presentation,
                        texture,
                        sampler,
                        is_srgb,
                        relative,
                    ))
                })();
                match part_plan {
                    Ok(part) => parts.push(part),
                    Err(error) => report.unsupported.push(PrefabUnsupported {
                        renderer: Some(renderer.source.clone()),
                        mesh_part: Some(name.clone()),
                        detail: error.to_string(),
                    }),
                }
            }
            Ok((node, enabled, casts_shadows, receives_shadows, parts))
        })();
        match plan {
            Ok(plan) => plans.push((renderer, plan)),
            Err(error) => report.unsupported.push(PrefabUnsupported {
                renderer: Some(renderer.source.clone()),
                mesh_part: None,
                detail: error.to_string(),
            }),
        }
    }
    if policy == PrefabRenderPolicy::Strict && !report.unsupported.is_empty() {
        return Err(invalid(format!(
            "prefab rendering rejected: {}",
            serde_json::to_string(&report.unsupported)
                .map_err(|error| invalid(error.to_string()))?
        )));
    }
    let mut mesh_handles = BTreeMap::new();
    let mut texture_handles = BTreeMap::new();
    let mut material_handles = BTreeMap::new();
    let mut textures = Vec::new();
    let mut parts = Vec::new();
    for (renderer, (node, enabled, casts_shadows, receives_shadows, plan)) in plans {
        for (name, record, source, mut presentation, texture, sampler, is_srgb, path) in plan {
            let mesh = mesh_handles
                .entry(name.clone())
                .or_insert_with(|| meshes.add(crate::application::build_bevy_mesh(record)))
                .clone();
            let image = texture_handles
                .entry(texture.source.clone())
                .or_insert_with(|| {
                    let image = server
                        .load_builder()
                        .with_settings(move |settings: &mut ImageLoaderSettings| {
                            settings.is_srgb = is_srgb;
                            settings.sampler = ImageSampler::Descriptor(sampler.clone());
                        })
                        .load(path);
                    textures.push((image.clone(), texture.clone()));
                    image
                })
                .clone();
            presentation.base_color_texture = Some(image);
            let handles = material_handles
                .entry(source.material.source.clone())
                .or_insert_with(|| {
                    let normal = materials.add(presentation.clone());
                    presentation.cull_mode = presentation.cull_mode.map(|face| match face {
                        Face::Front => Face::Back,
                        Face::Back => Face::Front,
                    });
                    (normal, materials.add(presentation))
                });
            let receives_shadows =
                receives_shadows && source.material.float("_ReceiveShadows") != Some(0.0);
            parts.push(PreparedPart {
                node,
                mesh,
                material: handles.0.clone(),
                mirrored_material: handles.1.clone(),
                metadata: SourcePrefabPart {
                    renderer: renderer.source.clone(),
                    material: source.material.source.clone(),
                    mesh_part: name,
                    source_enabled: enabled,
                    casts_shadows,
                    receives_shadows,
                },
            });
        }
    }
    report.prepared_parts = parts.len();
    let parents = package
        .manifest
        .nodes
        .iter()
        .map(|node| {
            node.parent
                .as_ref()
                .map(|source| {
                    package
                        .node_index(source)
                        .ok_or_else(|| invalid("parent absent"))
                })
                .transpose()
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PreparedPrefab {
        document: documents.add(SourcePrefabDocument {
            manifest: package.manifest.clone(),
            manifest_sha256: package.manifest_sha256.clone(),
        }),
        report,
        textures,
        parents,
        local,
        nodes: package
            .manifest
            .nodes
            .iter()
            .enumerate()
            .map(|(node_index, node)| SourcePrefabNode {
                source: node.source.clone(),
                name: node.name.clone(),
                node_index,
                original_local_transform: node.bevy_local_transform,
                active_self: node.active_self,
                layer: node.layer,
            })
            .collect(),
        poses: package
            .poses
            .iter()
            .map(|pose| Mat4::from_cols_array(&pose.world_transform))
            .collect(),
        parts,
        lods,
    })
}

impl PreparedPrefab {
    /// Standalone diagnostic translation only; original local hierarchy is preserved.
    pub fn spawn_preview(
        &self,
        commands: &mut Commands,
        offset: Vec3,
    ) -> Result<Entity, ArchiveError> {
        if !offset.is_finite() {
            return Err(invalid("non-finite preview offset"));
        }
        self.spawn(commands, Transform::from_translation(offset), None, None)
    }

    /// Caller must supply a verified placement identity and final Bevy root TRS.
    /// Partial diagnostic preparations cannot enter the runtime placement path.
    pub fn spawn_placement(
        &self,
        commands: &mut Commands,
        root: &SourceObject,
        placement: &SourceObject,
        matrix: [f32; 16],
    ) -> Result<Entity, ArchiveError> {
        if !self.report.unsupported.is_empty() {
            return Err(invalid(
                "partial prefab preview cannot be used as a runtime placement",
            ));
        }
        if placement
            .path_id
            .parse::<i64>()
            .ok()
            .is_none_or(|id| id == 0)
            || placement.bundle.is_empty()
            || placement.cab.is_empty()
        {
            return Err(invalid("invalid placement source identity"));
        }
        let root = self
            .nodes
            .iter()
            .position(|node| &node.source == root)
            .ok_or_else(|| invalid("unknown placement prefab root"))?;
        if self.parents[root].is_some() {
            return Err(invalid("placement must select a prefab root"));
        }
        self.spawn(
            commands,
            Transform::default(),
            Some((root, transform(matrix)?)),
            Some(placement.clone()),
        )
    }

    fn spawn(
        &self,
        commands: &mut Commands,
        container_transform: Transform,
        replacement: Option<(usize, Transform)>,
        placement: Option<SourceObject>,
    ) -> Result<Entity, ArchiveError> {
        let selected = |index: usize| {
            let Some((root, _)) = replacement else {
                return true;
            };
            let mut current = Some(index);
            while let Some(index) = current {
                if index == root {
                    return true;
                }
                current = self.parents[index];
            }
            false
        };
        let container = commands
            .spawn((
                Name::new("Verified source prefab"),
                container_transform,
                Visibility::Inherited,
                SourcePrefabInstance {
                    document: self.document.clone(),
                    preview: replacement.is_none(),
                    placement_source: placement,
                },
            ))
            .id();
        let mut nodes = BTreeMap::new();
        for (index, node) in self
            .nodes
            .iter()
            .enumerate()
            .filter(|(index, _)| selected(*index))
        {
            let local = replacement
                .filter(|(root, _)| *root == index)
                .map_or(self.local[index], |(_, transform)| transform);
            let entity = commands
                .spawn((
                    Name::new(node.name.clone()),
                    node.clone(),
                    local,
                    if node.active_self {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                ))
                .id();
            nodes.insert(index, entity);
        }
        for (&index, &entity) in &nodes {
            let parent = self.parents[index]
                .and_then(|parent| nodes.get(&parent))
                .copied()
                .unwrap_or(container);
            commands.entity(parent).add_child(entity);
        }
        let mut renderer_entities = BTreeMap::<SourceObject, Vec<Entity>>::new();
        for part in self.parts.iter().filter(|part| selected(part.node)) {
            let world = if let Some((root, replacement)) = replacement {
                replacement.to_matrix() * self.poses[root].inverse() * self.poses[part.node]
            } else {
                self.poses[part.node]
            };
            let material = if world.determinant() < 0.0 {
                &part.mirrored_material
            } else {
                &part.material
            };
            let mut entity = commands.spawn((
                Name::new(part.metadata.mesh_part.clone()),
                part.metadata.clone(),
                Mesh3d(part.mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::default(),
                if part.metadata.source_enabled {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
            ));
            if !part.metadata.casts_shadows {
                entity.insert(NotShadowCaster);
            }
            if !part.metadata.receives_shadows {
                entity.insert(NotShadowReceiver);
            }
            let entity = entity.id();
            commands.entity(nodes[&part.node]).add_child(entity);
            renderer_entities
                .entry(part.metadata.renderer.clone())
                .or_default()
                .push(entity);
        }
        for group in self.lods.iter().filter(|group| selected(group.node)) {
            let levels = group
                .renderers
                .iter()
                .map(|level| {
                    level
                        .iter()
                        .flat_map(|renderer| {
                            renderer_entities
                                .get(renderer)
                                .into_iter()
                                .flatten()
                                .copied()
                        })
                        .collect()
                })
                .collect();
            commands.entity(nodes[&group.node]).insert(SourceLodGroup {
                reference: group.reference,
                size: group.size,
                thresholds: group.thresholds.clone(),
                levels,
            });
        }
        Ok(container)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lod_visibility_propagates_to_children_in_the_camera_update_frame() {
        use bevy::{
            camera::visibility::VisibilityPlugin, mesh::skinning::SkinnedMeshInverseBindposes,
        };

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            TransformPlugin,
            VisibilityPlugin,
            SourcePrefabPlugin,
        ))
        .init_asset::<Mesh>()
        .init_asset::<SkinnedMeshInverseBindposes>();
        let source = SourceObject {
            bundle: "fixture.ab".into(),
            cab: "fixture".into(),
            path_id: "1".into(),
        };
        let part = app
            .world_mut()
            .spawn((
                SourcePrefabPart {
                    renderer: source.clone(),
                    material: source,
                    mesh_part: "fixture".into(),
                    source_enabled: true,
                    casts_shadows: true,
                    receives_shadows: true,
                },
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        let child = app
            .world_mut()
            .spawn((Transform::default(), Visibility::Inherited, ChildOf(part)))
            .id();
        let group = app
            .world_mut()
            .spawn((
                Transform::default(),
                Visibility::Inherited,
                SourceLodGroup {
                    reference: Vec3::ZERO,
                    size: 10.0,
                    thresholds: vec![0.5],
                    levels: vec![vec![part]],
                },
            ))
            .id();
        app.world_mut().entity_mut(group).add_child(part);
        let camera = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                SourcePrefabLodCamera,
                Transform::from_xyz(0.0, 0.0, 5.0),
                Projection::Perspective(PerspectiveProjection::default()),
            ))
            .id();

        for distance in [5.0, 100.0, 5.0, 100.0, 5.0] {
            app.world_mut()
                .get_mut::<Transform>(camera)
                .unwrap()
                .translation
                .z = distance;
            app.update();
            for entity in [part, child] {
                assert_eq!(
                    app.world()
                        .get::<InheritedVisibility>(entity)
                        .unwrap()
                        .get(),
                    distance == 5.0,
                    "camera movement must update inherited visibility in this frame"
                );
            }
        }

        *app.world_mut().get_mut::<Visibility>(group).unwrap() = Visibility::Hidden;
        app.update();
        assert!(!app.world().get::<InheritedVisibility>(child).unwrap().get());
        *app.world_mut().get_mut::<Visibility>(group).unwrap() = Visibility::Inherited;
        app.world_mut()
            .entity_mut(camera)
            .remove::<SourcePrefabLodCamera>();
        app.update();
        assert!(
            app.world()
                .resource::<SourcePrefabLodStatus>()
                .error
                .is_some()
        );
        assert!(!app.world().get::<InheritedVisibility>(child).unwrap().get());
    }

    #[test]
    fn lod_requires_one_camera_and_hides_parts_on_invalid_selection() {
        let mut app = App::new();
        app.init_resource::<SourcePrefabLodStatus>()
            .add_systems(Update, update_source_lods);
        let source = SourceObject {
            bundle: "fixture.ab".into(),
            cab: "fixture".into(),
            path_id: "1".into(),
        };
        let part = app
            .world_mut()
            .spawn((
                SourcePrefabPart {
                    renderer: source.clone(),
                    material: source,
                    mesh_part: "fixture".into(),
                    source_enabled: true,
                    casts_shadows: true,
                    receives_shadows: true,
                },
                Visibility::Inherited,
            ))
            .id();
        app.world_mut().spawn((
            GlobalTransform::IDENTITY,
            SourceLodGroup {
                reference: Vec3::ZERO,
                size: 10.0,
                thresholds: vec![0.5],
                levels: vec![vec![part]],
            },
        ));
        app.update();
        assert!(
            app.world()
                .resource::<SourcePrefabLodStatus>()
                .error
                .is_some()
        );
        assert_eq!(
            app.world().get::<Visibility>(part),
            Some(&Visibility::Hidden)
        );
        app.world_mut().spawn((
            Camera3d::default(),
            SourcePrefabLodCamera,
            GlobalTransform::IDENTITY,
            Projection::Perspective(PerspectiveProjection::default()),
        ));
        app.update();
        assert!(
            app.world()
                .resource::<SourcePrefabLodStatus>()
                .error
                .is_none()
        );
        assert_eq!(
            app.world().get::<Visibility>(part),
            Some(&Visibility::Inherited)
        );
        app.world_mut().spawn((
            Camera3d::default(),
            SourcePrefabLodCamera,
            GlobalTransform::IDENTITY,
            Projection::Perspective(PerspectiveProjection::default()),
        ));
        app.update();
        assert!(
            app.world()
                .resource::<SourcePrefabLodStatus>()
                .error
                .is_some()
        );
        assert_eq!(
            app.world().get::<Visibility>(part),
            Some(&Visibility::Hidden)
        );
    }
    #[test]
    fn lod_thresholds_select_source_levels_and_cull() {
        let thresholds = [0.68, 0.48, 0.008, 0.0];
        assert_eq!(
            [0.8, 0.6, 0.1, 0.001].map(|height| select_source_lod(&thresholds, height)),
            [Some(0), Some(1), Some(2), Some(3)]
        );
    }
    #[test]
    fn source_reference_and_texture_st_convert_coordinate_frames_once() {
        assert_eq!(
            source_lod_reference([1.0, 2.0, 3.0]),
            Vec3::new(1.0, 2.0, -3.0)
        );
        let unity_uv = Vec2::new(0.2, 0.7);
        let scale = [2.0, 3.0];
        let offset = [0.25, 0.125];
        let bevy_uv = Vec2::new(unity_uv.x, 1.0 - unity_uv.y);
        let expected = Vec2::new(
            unity_uv.x * scale[0] + offset[0],
            1.0 - unity_uv.y * scale[1] - offset[1],
        );
        assert!(
            source_uv_transform(scale, offset)
                .transform_point2(bevy_uv)
                .abs_diff_eq(expected, 0.00001)
        );
    }
    #[test]
    fn source_transform_rejects_shear_instead_of_discarding_it() {
        let mut matrix = Mat4::IDENTITY.to_cols_array();
        matrix[4] = 0.25;
        assert!(transform(matrix).is_err());
        let matrix = Mat4::from_scale_rotation_translation(
            Vec3::new(-2.0, 1.0, 3.0),
            Quat::from_rotation_x(0.7),
            Vec3::new(1.0, 2.0, 3.0),
        );
        assert!(
            transform(matrix.to_cols_array())
                .unwrap()
                .to_matrix()
                .abs_diff_eq(matrix, 0.0002)
        );
    }
}
