//! Presentation of the validated APK terrain package; no extraction heuristics.

use bevy::{
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    prelude::*,
    render::render_resource::{AsBindGroup, Face, ShaderType},
    shader::ShaderRef,
};
use midnight_map_data::{
    ArchiveError, SourceMaterial, TerrainPackage, TerrainParameters, TextureAsset,
};

#[derive(Resource)]
pub(crate) struct ImportedTerrainActive;

#[derive(Debug, Clone, ShaderType)]
pub struct TerrainUniform {
    pub tiling: Vec4,
    pub smoothness: Vec4,
    pub layer_defaults: [Vec4; 4],
    pub normal_defaults: [Vec4; 4],
    pub control_default: Vec4,
    pub layer_present: UVec4,
    pub normal_present: UVec4,
    pub layer_count: u32,
    pub control_present: u32,
    pub normal_enabled: u32,
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct TerrainSplatMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub control: Option<Handle<Image>>,
    #[texture(2)]
    #[sampler(3)]
    pub layer0: Option<Handle<Image>>,
    #[texture(4)]
    #[sampler(5)]
    pub layer1: Option<Handle<Image>>,
    #[texture(6)]
    #[sampler(7)]
    pub layer2: Option<Handle<Image>>,
    #[texture(8)]
    #[sampler(9)]
    pub layer3: Option<Handle<Image>>,
    #[uniform(10)]
    pub uniforms: TerrainUniform,
    #[texture(11)]
    #[sampler(12)]
    pub normal0: Option<Handle<Image>>,
    #[texture(13)]
    #[sampler(14)]
    pub normal1: Option<Handle<Image>>,
    #[texture(15)]
    #[sampler(16)]
    pub normal2: Option<Handle<Image>>,
    #[texture(17)]
    #[sampler(18)]
    pub normal3: Option<Handle<Image>>,
}

impl Material for TerrainSplatMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain_splat.wesl".into()
    }
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Face::Back);
        Ok(())
    }
}

fn invalid(message: impl Into<String>) -> ArchiveError {
    ArchiveError::Invalid(message.into())
}

fn address_mode(source: i32) -> Result<ImageAddressMode, ArchiveError> {
    match source {
        0 => Ok(ImageAddressMode::Repeat),
        1 => Ok(ImageAddressMode::ClampToEdge),
        2 => Ok(ImageAddressMode::MirrorRepeat),
        _ => Err(invalid(format!(
            "unsupported source texture wrap mode {source}"
        ))),
    }
}

fn load_texture(
    server: &AssetServer,
    texture: &TextureAsset,
) -> Result<Handle<Image>, ArchiveError> {
    let source = &texture.sampler;
    let gpu = texture.gpu.as_ref().ok_or_else(|| {
        invalid(format!(
            "{}: source mip archive is missing; rerun the Rust terrain import",
            texture.name
        ))
    })?;
    if source.aniso > 1 || source.mip_bias != 0.0 {
        return Err(invalid(
            "this source texture needs anisotropy or mip bias support",
        ));
    }
    let (filter, mipmap_filter) = match source.filter_mode {
        0 => (ImageFilterMode::Nearest, ImageFilterMode::Nearest),
        1 => (ImageFilterMode::Linear, ImageFilterMode::Nearest),
        2 => (ImageFilterMode::Linear, ImageFilterMode::Linear),
        _ => return Err(invalid("unsupported source texture filter mode")),
    };
    let sampler = ImageSamplerDescriptor {
        address_mode_u: address_mode(source.wrap[0])?,
        address_mode_v: address_mode(source.wrap[1])?,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter,
        ..default()
    };
    let is_srgb = match texture.unity_color_space {
        0 => false,
        1 => true,
        _ => return Err(invalid("unsupported source texture color space")),
    };
    Ok(server
        .load_builder()
        .with_settings(move |settings: &mut ImageLoaderSettings| {
            settings.is_srgb = is_srgb;
            settings.sampler = ImageSampler::Descriptor(sampler.clone());
        })
        .load(format!("imported_map/terrain/{}", gpu.path)))
}

fn source_texture<'a>(material: &'a SourceMaterial, name: &str) -> Option<&'a TextureAsset> {
    material
        .textures
        .get(name)
        .and_then(|binding| binding.texture.as_ref())
}

impl TerrainSplatMaterial {
    fn from_source(
        server: &AssetServer,
        source: &SourceMaterial,
        has_source_tangents: bool,
    ) -> Result<Self, ArchiveError> {
        let parameters = TerrainParameters::from_material(source)?;
        let mut layers: [Option<Handle<Image>>; 4] = [None, None, None, None];
        let mut normals: [Option<Handle<Image>>; 4] = [None, None, None, None];
        let mut defaults = [Vec4::ZERO; 4];
        let mut normal_defaults = [Vec4::ZERO; 4];
        let mut present = [0u32; 4];
        let mut normal_present = [0u32; 4];
        let normal_enabled = source.uses_terrain_normal_maps() && has_source_tangents;
        for layer in 0..parameters.layer_count {
            let name = format!("_Splat{layer}");
            if let Some(texture) = source_texture(source, &name) {
                layers[layer] = Some(load_texture(server, texture)?);
                present[layer] = 1;
            } else {
                defaults[layer] =
                    Vec4::from_array(source.default_texture_color(&name).ok_or_else(|| {
                        invalid(format!(
                            "{}: missing declared default for {name}",
                            source.name
                        ))
                    })?);
            }
            if normal_enabled {
                let name = format!("_Splat{layer}_Normal");
                if let Some(texture) = source_texture(source, &name) {
                    normals[layer] = Some(load_texture(server, texture)?);
                    normal_present[layer] = 1;
                } else {
                    normal_defaults[layer] =
                        Vec4::from_array(source.default_texture_color(&name).ok_or_else(|| {
                            invalid(format!(
                                "{}: missing declared normal default for {name}",
                                source.name
                            ))
                        })?);
                }
            }
        }
        let control = source_texture(source, "_Control")
            .map(|texture| load_texture(server, texture))
            .transpose()?;
        let control_default = source.default_texture_color("_Control").unwrap_or([0.0; 4]);
        let control_present = u32::from(control.is_some());
        let [layer0, layer1, layer2, layer3] = layers;
        let [normal0, normal1, normal2, normal3] = normals;
        Ok(Self {
            control,
            layer0,
            layer1,
            layer2,
            layer3,
            normal0,
            normal1,
            normal2,
            normal3,
            uniforms: TerrainUniform {
                tiling: Vec4::from_array(parameters.tiling),
                smoothness: Vec4::from_array(parameters.smoothness),
                layer_defaults: defaults,
                normal_defaults,
                control_default: Vec4::from_array(control_default),
                layer_present: UVec4::from_array(present),
                normal_present: UVec4::from_array(normal_present),
                layer_count: parameters.layer_count as u32,
                control_present,
                normal_enabled: u32::from(normal_enabled),
            },
        })
    }
}

pub fn spawn(
    commands: &mut Commands,
    server: &AssetServer,
    package: &TerrainPackage,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    splats: &mut Assets<TerrainSplatMaterial>,
) -> Result<(), ArchiveError> {
    for record in &package.records {
        let source = &package.manifest.meshes[&record.name].material;
        let mesh = meshes.add(crate::application::build_bevy_mesh(record));
        let chunk = crate::application::TerrainSectorChunk {
            center: Vec3::from_array(record.center),
            is_road: source.terrain_layer_count().is_none(),
        };
        if source.terrain_layer_count().is_some() {
            if source.uses_terrain_normal_maps() && record.tangents.is_empty() {
                warn!(
                    "{}: source mesh has no tangents; rendering geometric normals. Original shader LOD remains unverified.",
                    record.name
                );
            }
            let material = splats.add(TerrainSplatMaterial::from_source(
                server,
                source,
                !record.tangents.is_empty(),
            )?);
            commands.spawn((
                Name::new(record.name.clone()),
                chunk,
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::default(),
            ));
        } else {
            let texture = source_texture(source, "_BaseMap")
                .or_else(|| source_texture(source, "_MainTex"))
                .map(|texture| load_texture(server, texture))
                .transpose()?;
            let color = source
                .colors
                .get("_BaseColor")
                .or_else(|| source.colors.get("_Color"))
                .copied()
                .unwrap_or([1.0; 4]);
            let material = materials.add(StandardMaterial {
                base_color: Color::srgba(color[0], color[1], color[2], color[3]),
                base_color_texture: texture,
                // Road shader edges are translucent. Its lighting remains a Bevy approximation.
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.9,
                depth_bias: 1.0,
                ..default()
            });
            commands.spawn((
                Name::new(record.name.clone()),
                chunk,
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::default(),
            ));
        }
    }
    commands.insert_resource(ImportedTerrainActive);
    Ok(())
}
