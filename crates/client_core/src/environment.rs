//! Desktop atmosphere rendering. These defaults are a Bevy implementation,
//! not a claim of pixel parity with the source game's runtime lighting.

use bevy::{
    camera::Hdr,
    core_pipeline::{
        Core3dSystems, FullscreenShader,
        prepass::{DepthPrepass, ViewPrepassTextures},
        schedule::Core3d,
    },
    pbr::ContactShadows,
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        camera::ExtractedCamera,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{
                sampler, texture_2d, texture_3d, texture_depth_2d, texture_depth_2d_multisampled,
                uniform_buffer,
            },
            *,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        texture::{CachedTexture, GpuImage, TextureCache},
        view::{ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
    },
};

/// Shared desktop camera/light policy. Screen-space contact rays produce
/// frame-varying self-shadow dots on verified prefab walls even with thin rays.
/// Directional shadow maps remain enabled independently of this optional pass.
pub const DESKTOP_CONTACT_SHADOWS: Option<ContactShadows> = None;

/// Direction toward the sun. The light, procedural disc and cloud scattering
/// all use this same direction, including its below-horizon part at night.
pub fn sun_direction(hours: f32) -> Vec3 {
    let angle = (hours.rem_euclid(24.0) - 6.0) / 24.0 * std::f32::consts::TAU;
    Vec3::new(angle.cos(), angle.sin(), -0.35).normalize()
}

pub fn daylight_strength(direction: Vec3) -> f32 {
    (direction.y / 0.20).clamp(0.0, 1.0)
}

pub fn cloud_step_count(requested: u32) -> u32 {
    match requested {
        12 | 48 => requested,
        _ => 24,
    }
}

/// Ray interval through a horizontal cloud layer, clipped by opaque geometry.
/// Kept as a CPU reference for the shader's depth/inside-volume behavior.
pub fn cloud_interval(
    origin_y: f32,
    direction_y: f32,
    bottom: f32,
    top: f32,
    opaque_distance: f32,
) -> Option<(f32, f32)> {
    let limit = opaque_distance.min(18_000.0);
    if limit <= 0.0 || bottom >= top {
        return None;
    }
    if direction_y.abs() < 1e-5 {
        return (origin_y >= bottom && origin_y <= top).then_some((0.0, limit));
    }
    let a = (bottom - origin_y) / direction_y;
    let b = (top - origin_y) / direction_y;
    let start = a.min(b).max(0.0);
    let end = a.max(b).min(limit);
    (end > start).then_some((start, end))
}

#[derive(Resource, Clone, ExtractResource)]
#[extract_app(RenderApp)]
pub struct CloudDensity(pub Handle<Image>);

/// Frame-stable render values extracted from the main camera.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
#[extract_app(RenderApp)]
#[require(DepthPrepass, Hdr)]
pub struct CloudSettings {
    /// xyz: normalized direction to sun, w: daylight after weather attenuation.
    pub sun: Vec4,
    /// x/z: world-space wind displacement. Camera position never enters it.
    pub wind: Vec4,
    /// bottom, top, extinction per metre, density multiplier.
    pub layer: Vec4,
    pub sky_zenith: Vec4,
    pub sky_horizon: Vec4,
    pub light_color: Vec4,
    /// View steps, light steps, enabled, reserved.
    pub quality: UVec4,
}

impl Default for CloudSettings {
    fn default() -> Self {
        Self {
            sun: sun_direction(10.0).extend(1.0),
            wind: Vec4::ZERO,
            layer: Vec4::new(1350.0, 1750.0, 0.012, 2.8),
            sky_zenith: Vec4::new(0.12, 0.35, 0.70, 1.0),
            sky_horizon: Vec4::new(0.55, 0.70, 0.85, 1.0),
            light_color: Vec4::new(1.0, 0.96, 0.86, 1.0),
            quality: UVec4::new(24, 6, 1, 0),
        }
    }
}

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<CloudSettings>::default(),
            UniformComponentPlugin::<CloudSettings>::default(),
            ExtractResourcePlugin::<CloudDensity>::default(),
        ));
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_systems(RenderStartup, init_cloud_pipeline)
            .add_systems(
                Render,
                prepare_cloud_targets.in_set(RenderSystems::PrepareViews),
            )
            .add_systems(
                Core3d,
                render_clouds
                    .in_set(Core3dSystems::EarlyPostProcess)
                    .after(Core3dSystems::MainPass)
                    .before(bevy::core_pipeline::tonemapping::tonemapping),
            );
    }
}

#[derive(Component)]
struct CloudTarget {
    texture: CachedTexture,
}

fn prepare_cloud_targets(
    mut commands: Commands,
    views: Query<(Entity, &ExtractedCamera), With<CloudSettings>>,
    mut cache: ResMut<TextureCache>,
    device: Res<RenderDevice>,
) {
    for (entity, camera) in &views {
        let Some(full_size) = camera.physical_target_size else {
            continue;
        };
        let size = UVec2::new(
            full_size.x.div_ceil(2).max(1),
            full_size.y.div_ceil(2).max(1),
        );
        // TextureCache leases last one frame. Reacquire every frame so another
        // view cannot reuse an attachment we still intend to write.
        let texture = cache.get(
            &device,
            TextureDescriptor {
                label: Some("half_resolution_clouds"),
                size: Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        );
        commands.entity(entity).insert(CloudTarget { texture });
    }
}

#[derive(Resource)]
struct CloudPipeline {
    layouts: [BindGroupLayoutDescriptor; 2],
    raymarch: [CachedRenderPipelineId; 2],
    composite: [CachedRenderPipelineId; 2],
    sampler: Sampler,
    screen_sampler: Sampler,
}

fn init_cloud_pipeline(
    mut commands: Commands,
    device: Res<RenderDevice>,
    assets: Res<AssetServer>,
    fullscreen: Res<FullscreenShader>,
    cache: Res<PipelineCache>,
) {
    let shader = assets.load("shaders/volumetric_clouds.wesl");
    let layouts = [false, true].map(|multisampled| {
        BindGroupLayoutDescriptor::new(
            "volumetric_cloud_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<CloudSettings>(true),
                    if multisampled {
                        texture_depth_2d_multisampled()
                    } else {
                        texture_depth_2d()
                    },
                    texture_3d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        )
    });
    let queue = |index: usize, entry: &'static str| {
        cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some(format!("cloud_{entry}_{index}").into()),
            layout: vec![layouts[index].clone()],
            vertex: fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: shader.clone(),
                shader_defs: if index == 1 {
                    vec!["MULTISAMPLED".into()]
                } else {
                    vec![]
                },
                entry_point: Some(entry.into()),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        })
    };
    let raymarch = [queue(0, "raymarch"), queue(1, "raymarch")];
    let composite = [queue(0, "composite"), queue(1, "composite")];
    let sampler = device.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..default()
    });
    let screen_sampler = device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..default()
    });
    commands.insert_resource(CloudPipeline {
        layouts,
        raymarch,
        composite,
        sampler,
        screen_sampler,
    });
}

#[allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "Bevy injects view data, GPU resources and render context as separate system parameters"
)]
fn render_clouds(
    view: ViewQuery<(
        &ViewTarget,
        &ViewUniformOffset,
        &DynamicUniformIndex<CloudSettings>,
        &CloudTarget,
        &ViewPrepassTextures,
        &Msaa,
    )>,
    pipeline: Option<Res<CloudPipeline>>,
    cache: Res<PipelineCache>,
    view_uniforms: Res<ViewUniforms>,
    settings_uniforms: Res<ComponentUniforms<CloudSettings>>,
    density: Option<Res<CloudDensity>>,
    images: Res<RenderAssets<GpuImage>>,
    mut ctx: RenderContext,
) {
    let (target, view_offset, settings_index, clouds, prepass, msaa) = view.into_inner();
    let (Some(pipeline), Some(density)) = (pipeline, density) else {
        return;
    };
    let Some(image) = images.get(&density.0) else {
        return;
    };
    let Some(depth) = prepass.depth_only_view() else {
        return;
    };
    let (Some(view_binding), Some(settings_binding)) = (
        view_uniforms.uniforms.binding(),
        settings_uniforms.uniforms().binding(),
    ) else {
        return;
    };
    let index = usize::from(msaa.samples() > 1);
    let (Some(raymarch), Some(composite)) = (
        cache.get_render_pipeline(pipeline.raymarch[index]),
        cache.get_render_pipeline(pipeline.composite[index]),
    ) else {
        return;
    };
    // Every readiness check precedes the ping-pong swap: an unavailable shader
    // cannot accidentally replace the camera's completed scene with stale data.
    let post = target.post_process_write();
    let layout = cache.get_bind_group_layout(&pipeline.layouts[index]);
    let bind = |cloud_view: &TextureView| {
        ctx.render_device().create_bind_group(
            "cloud_bind_group",
            &layout,
            &BindGroupEntries::sequential((
                view_binding.clone(),
                settings_binding.clone(),
                depth,
                &image.texture_view,
                &pipeline.sampler,
                post.source,
                cloud_view,
                &pipeline.screen_sampler,
            )),
        )
    };
    // The raymarch does not read binding 6. Bind the existing scene there rather
    // than binding the cloud texture while it is a render attachment.
    let march_bind = bind(post.source);
    let composite_bind = bind(&clouds.texture.default_view);
    let offsets = [view_offset.offset, settings_index.index()];
    {
        let mut pass = ctx
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("half_resolution_volumetric_clouds"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &clouds.texture.default_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(default()),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        pass.set_pipeline(raymarch);
        pass.set_bind_group(0, &march_bind, &offsets);
        pass.draw(0..3, 0..1);
    }
    {
        let mut pass = ctx
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("depth_aware_cloud_composite"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: post.destination,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(default()),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        pass.set_pipeline(composite);
        pass.set_bind_group(0, &composite_bind, &offsets);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_passes_below_horizon_and_wraps_midnight() {
        assert!(sun_direction(12.0).y > 0.9);
        assert!(sun_direction(0.0).y < -0.9);
        assert_eq!(daylight_strength(sun_direction(0.0)), 0.0);
        assert!(sun_direction(0.0).abs_diff_eq(sun_direction(24.0), 1e-6));
        assert!(sun_direction(6.0).y.abs() < 1e-6);
    }

    #[test]
    fn foreground_occludes_clouds_and_inside_camera_can_exit() {
        assert_eq!(cloud_interval(0.0, 1.0, 1350.0, 1750.0, 100.0), None);
        assert_eq!(
            cloud_interval(0.0, 1.0, 1350.0, 1750.0, 1600.0),
            Some((1350.0, 1600.0))
        );
        assert_eq!(
            cloud_interval(1500.0, -1.0, 1350.0, 1750.0, 500.0),
            Some((0.0, 150.0))
        );
        assert_eq!(cloud_interval(0.0, 0.0, 1350.0, 1750.0, 500.0), None);
        assert_eq!(
            cloud_interval(1500.0, 0.0, 1350.0, 1750.0, 500.0),
            Some((0.0, 500.0))
        );
    }

    #[test]
    fn quality_settings_bound_shader_work() {
        assert_eq!(
            [12, 24, 48, 0, 10_000].map(cloud_step_count),
            [12, 24, 48, 24, 24]
        );
        assert_eq!(CloudSettings::default().quality.x, 24);
    }

    #[test]
    fn camera_settings_install_depth_and_hdr_requirements() {
        let mut world = World::new();
        let camera = world.spawn(CloudSettings::default()).id();
        assert!(world.get::<DepthPrepass>(camera).is_some());
        assert!(world.get::<Hdr>(camera).is_some());
    }
}
