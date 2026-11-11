//! Headless Vulkan captures of production terrain, shadows and volumetric clouds.
//! cargo run --offline -p client_core --example environment_probe -- OUTPUT_DIR

use bevy::{
    audio::AudioPlugin,
    camera::RenderTarget,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
    render::{
        RenderApp, RenderPlugin,
        pipelined_rendering::PipelinedRenderingPlugin,
        render_resource::{
            CachedPipelineState, PipelineCache, PollType, TextureFormat, TextureUsages,
        },
        renderer::{RenderAdapterInfo, RenderDevice},
        settings::{Backends, RenderCreation, WgpuSettings},
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    shader::ShaderCacheError,
    window::ExitCondition,
    winit::WinitPlugin,
};
use client_core::{
    SourceMipImagePlugin,
    application::{TerrainSplatMaterial, generate_cloud_3d_image, spawn_imported_terrain},
    environment::{
        CloudDensity, CloudSettings, DESKTOP_CONTACT_SHADOWS, EnvironmentPlugin, daylight_strength,
        sun_direction,
    },
};
use midnight_map_data::{TerrainPackage, load_terrain_package};
use std::{
    collections::BTreeSet,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Resource)]
struct Package(TerrainPackage);
#[derive(Resource, Default)]
struct Capture {
    image: Option<Image>,
    error: Option<String>,
}
#[derive(Component)]
struct Sun;
#[derive(Component)]
struct ReferenceCaster;

#[allow(
    clippy::too_many_arguments,
    reason = "Bevy injects the terrain package and asset collections as separate system parameters"
)]
fn setup(
    mut commands: Commands,
    package: Res<Package>,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut splats: ResMut<Assets<TerrainSplatMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut capture: ResMut<Capture>,
) {
    if let Err(error) = spawn_imported_terrain(
        &mut commands,
        &server,
        &package.0,
        &mut meshes,
        &mut materials,
        &mut splats,
    ) {
        capture.error = Some(error.to_string());
    }
    commands.insert_resource(CloudDensity(images.add(generate_cloud_3d_image())));
    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });
    commands.spawn((
        Sun,
        DirectionalLight {
            illuminance: 45_000.0,
            shadow_maps_enabled: true,
            contact_shadows_enabled: DESKTOP_CONTACT_SHADOWS.is_some(),
            shadow_depth_bias: 0.02,
            shadow_normal_bias: DirectionalLight::DEFAULT_SHADOW_NORMAL_BIAS,
            ..default()
        },
        CascadeShadowConfigBuilder {
            maximum_distance: 400.0,
            first_cascade_far_bound: 12.0,
            ..default()
        }
        .build(),
        Transform::default().looking_to(-sun_direction(12.0), Vec3::Y),
    ));
}

fn captured(event: On<ScreenshotCaptured>, mut capture: ResMut<Capture>) {
    capture.image = Some(event.image.clone());
}

fn update(app: &mut App, deadline: Instant) -> Result<(), Box<dyn std::error::Error>> {
    if Instant::now() > deadline {
        return Err("environment probe exceeded its 180-second deadline".into());
    }
    app.update();
    app.world()
        .resource::<RenderDevice>()
        .wgpu_device()
        .poll(PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(5)),
        })?;
    if let Some(error) = &app.world().resource::<Capture>().error {
        return Err(error.clone().into());
    }
    let cache = app.sub_app(RenderApp).world().resource::<PipelineCache>();
    for pipeline in cache.pipelines() {
        if let CachedPipelineState::Err(error) = &pipeline.state
            && !matches!(
                error,
                ShaderCacheError::ShaderNotLoaded(_)
                    | ShaderCacheError::ShaderImportNotYetAvailable
            )
        {
            return Err(format!("environment GPU pipeline failed: {error}").into());
        }
    }
    Ok(())
}

fn ready(app: &App) -> bool {
    let world = app.world();
    let server = world.resource::<AssetServer>();
    for (_, material) in world.resource::<Assets<TerrainSplatMaterial>>().iter() {
        if [
            &material.control,
            &material.layer0,
            &material.layer1,
            &material.layer2,
            &material.layer3,
            &material.normal0,
            &material.normal1,
            &material.normal2,
            &material.normal3,
        ]
        .into_iter()
        .flatten()
        .any(|image| !server.is_loaded_with_dependencies(image.id()))
        {
            return false;
        }
    }
    for (_, material) in world.resource::<Assets<StandardMaterial>>().iter() {
        if material
            .base_color_texture
            .as_ref()
            .is_some_and(|image| !server.is_loaded_with_dependencies(image.id()))
        {
            return false;
        }
    }
    let cache = app.sub_app(RenderApp).world().resource::<PipelineCache>();
    cache.waiting_pipelines().next().is_none() && cache.pipelines().count() >= 4
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/midnight-environment-probe"));
    std::fs::create_dir_all(&directory)?;
    let assets = shared::level::asset_root();
    let package = load_terrain_package(&assets.join("imported_map/terrain"))?;
    let source_hash = package.manifest.source_apk.sha256.clone();
    let source_meshes = package.records.len();
    let z01 = package
        .records
        .iter()
        .find(|record| {
            package.manifest.meshes[&record.name].source.bundle == "assets/scene/z-01.ab"
        })
        .ok_or("verified Z-01 terrain scene is required for this probe")?;
    let z01_name = z01.name.clone();
    let z01_bounds = midnight_map_data::mesh_bounds(z01);
    let z01_center = Vec3::new(
        (z01_bounds[0] + z01_bounds[3]) * 0.5,
        (z01_bounds[1] + z01_bounds[4]) * 0.5,
        (z01_bounds[2] + z01_bounds[5]) * 0.5,
    );
    let mut collision = shared::physics::CollisionWorld::new(shared::level::MapId::Polygon);
    collision.insert_canonical_terrain(&package)?;
    let spawn = collision
        .surface_spawn_at([0.0, 12.0], None)
        .ok_or("no source terrain spawn")?;
    let position = Vec3::from_array(spawn) + Vec3::new(0.0, 4.0, 0.0);
    let look_at = position + Vec3::new(0.0, 0.0, -100.0);
    let shadow_ground = Vec3::from_array(
        collision
            .surface_spawn_at([0.0, -16.0], None)
            .ok_or("no source terrain shadow reference point")?,
    );
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: assets.to_string_lossy().into_owned(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .set(RenderPlugin {
                synchronous_pipeline_compilation: true,
                render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                    backends: Some(Backends::VULKAN),
                    ..default()
                })),
                ..default()
            })
            .disable::<WinitPlugin>()
            .disable::<PipelinedRenderingPlugin>()
            .disable::<AudioPlugin>(),
    )
    .add_plugins((
        SourceMipImagePlugin,
        MaterialPlugin::<TerrainSplatMaterial>::default(),
        EnvironmentPlugin,
    ))
    .insert_resource(Package(package))
    .init_resource::<Capture>()
    .insert_resource(GlobalAmbientLight {
        brightness: 180.0,
        ..default()
    })
    .add_systems(Startup, setup);
    app.finish();
    app.cleanup();
    let mut image = Image::new_target_texture(1280, 720, TextureFormat::Rgba8UnormSrgb, None);
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let target = app.world_mut().resource_mut::<Assets<Image>>().add(image);
    let camera = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            RenderTarget::Image(target.clone().into()),
            CloudSettings::default(),
            Msaa::Off,
            Tonemapping::TonyMcMapface,
            DebandDither::Disabled,
            Projection::Perspective(PerspectiveProjection {
                far: 3500.0,
                fov: 80.0_f32.to_radians(),
                ..default()
            }),
            Transform::from_translation(position).looking_at(look_at, Vec3::Y),
        ))
        .id();
    if let Some(contact) = DESKTOP_CONTACT_SHADOWS {
        app.world_mut().entity_mut(camera).insert(contact);
    }
    let caster_mesh = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Cuboid::new(8.0, 8.0, 8.0));
    let caster_material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(Color::srgb(0.4, 0.4, 0.4));
    let caster = app
        .world_mut()
        .spawn((
            ReferenceCaster,
            Mesh3d(caster_mesh),
            MeshMaterial3d(caster_material),
            Transform::from_translation(shadow_ground + Vec3::Y * 4.5),
            Visibility::Hidden,
        ))
        .id();
    let deadline = Instant::now() + Duration::from_secs(180);
    let mut views = Vec::new();
    let mut noon_pixels = None;
    let mut noon_luma = 0.0;
    let mut shadow_pixels = None;
    let mut z01_pixels = None;
    for (name, hours, steps, msaa, view_kind) in [
        ("noon-24.png", 12.0, 24, Msaa::Off, "terrain"),
        ("noon-disabled.png", 12.0, 24, Msaa::Off, "terrain"),
        ("noon-48-msaa.png", 12.0, 48, Msaa::Sample4, "terrain"),
        ("night-12-msaa.png", 0.0, 12, Msaa::Sample4, "terrain"),
        ("sunrise-clear.png", 8.0, 24, Msaa::Off, "sun"),
        ("inside-cloud-48.png", 12.0, 48, Msaa::Off, "inside"),
        ("shadow-on.png", 9.0, 24, Msaa::Off, "shadow"),
        ("shadow-off.png", 9.0, 24, Msaa::Off, "shadow"),
        ("z01-overview.png", 12.0, 24, Msaa::Sample4, "z01"),
        ("z01-hidden.png", 12.0, 24, Msaa::Sample4, "z01"),
    ] {
        let direction = sun_direction(hours);
        let strength = daylight_strength(direction);
        let defaults = CloudSettings::default();
        let mut settings = CloudSettings {
            sun: direction.extend(strength),
            quality: UVec4::new(
                steps,
                defaults.quality.y,
                u32::from(
                    name != "noon-disabled.png"
                        && view_kind != "sun"
                        && view_kind != "shadow"
                        && view_kind != "z01",
                ),
                defaults.quality.w,
            ),
            ..defaults
        };
        if hours == 0.0 {
            settings.sky_zenith = Vec4::new(0.001, 0.003, 0.008, 1.0);
            settings.sky_horizon = Vec4::new(0.007, 0.011, 0.020, 1.0);
        }
        let transform = match view_kind {
            "sun" => Transform::from_translation(position).looking_to(direction, Vec3::Y),
            "inside" => Transform::from_xyz(position.x, 1500.0, position.z)
                .looking_to(Vec3::new(0.0, 0.1, -1.0), Vec3::Y),
            "shadow" => Transform::from_translation(shadow_ground + Vec3::new(18.0, 20.0, 25.0))
                .looking_at(shadow_ground, Vec3::Y),
            "z01" => Transform::from_translation(z01_center + Vec3::new(100.0, 350.0, -440.0))
                .looking_at(z01_center, Vec3::Y),
            _ => Transform::from_translation(position).looking_at(look_at, Vec3::Y),
        };
        app.world_mut()
            .entity_mut(camera)
            .insert((settings, msaa, transform));
        for (source_name, mut visibility) in app
            .world_mut()
            .query::<(&Name, &mut Visibility)>()
            .iter_mut(app.world_mut())
        {
            if source_name.as_str() == z01_name {
                *visibility = if name == "z01-hidden.png" {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                };
            }
        }
        app.world_mut()
            .entity_mut(caster)
            .insert(if view_kind == "shadow" {
                Visibility::Visible
            } else {
                Visibility::Hidden
            });
        app.world_mut()
            .resource_mut::<GlobalAmbientLight>()
            .brightness = if hours == 0.0 { 8.0 } else { 180.0 };
        for (mut light, mut transform) in app
            .world_mut()
            .query_filtered::<(&mut DirectionalLight, &mut Transform), With<Sun>>()
            .iter_mut(app.world_mut())
        {
            light.illuminance = strength * 45_000.0;
            light.shadow_maps_enabled = name != "shadow-off.png";
            light.contact_shadows_enabled =
                name != "shadow-off.png" && DESKTOP_CONTACT_SHADOWS.is_some();
            *transform = Transform::default().looking_to(-direction, Vec3::Y);
        }
        let mut stable_frames = 0;
        while stable_frames < 5 {
            update(&mut app, deadline)?;
            stable_frames = if ready(&app) { stable_frames + 1 } else { 0 };
        }
        // This measures the whole synchronous capture harness, including CPU
        // extraction and GPU completion. It is not an in-game FPS benchmark.
        let mut frame_ms = Vec::with_capacity(20);
        for _ in 0..20 {
            let start = Instant::now();
            update(&mut app, deadline)?;
            frame_ms.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        let mean_frame_ms = frame_ms.iter().sum::<f64>() / frame_ms.len() as f64;
        app.world_mut()
            .spawn(Screenshot::image(target.clone()))
            .observe(captured);
        while app.world().resource::<Capture>().image.is_none() {
            update(&mut app, deadline)?;
        }
        let image = app
            .world_mut()
            .resource_mut::<Capture>()
            .image
            .take()
            .ok_or("missing screenshot")?;
        let rgba = image.try_into_dynamic()?.to_rgba8();
        let mean_luma = rgba
            .pixels()
            .map(|pixel| {
                0.2126 * f64::from(pixel.0[0])
                    + 0.7152 * f64::from(pixel.0[1])
                    + 0.0722 * f64::from(pixel.0[2])
            })
            .sum::<f64>()
            / f64::from(rgba.width() * rgba.height());
        let buckets = rgba
            .pixels()
            .map(|pixel| [pixel.0[0] >> 4, pixel.0[1] >> 4, pixel.0[2] >> 4])
            .collect::<BTreeSet<_>>();
        if buckets.len() < 4 {
            return Err(format!("{name}: capture has too little color variation").into());
        }
        if name == "noon-24.png" {
            noon_pixels = Some(rgba.as_raw().clone());
            noon_luma = mean_luma;
        }
        if name == "noon-disabled.png" {
            let enabled = noon_pixels
                .as_ref()
                .ok_or("missing enabled cloud capture")?;
            let changed = enabled
                .as_chunks::<4>()
                .0
                .iter()
                .zip(rgba.as_raw().as_chunks::<4>().0)
                .take(1280 * 360)
                .filter(|(a, b)| a[..3] != b[..3])
                .count();
            if changed < 100 {
                return Err(format!(
                    "cloud pass did not measurably affect the sky: {changed} changed pixels"
                )
                .into());
            }
            let foreground_changed = enabled
                .as_chunks::<4>()
                .0
                .iter()
                .zip(rgba.as_raw().as_chunks::<4>().0)
                .skip(1280 * 480)
                .filter(|(a, b)| a[..3].iter().zip(&b[..3]).any(|(a, b)| a.abs_diff(*b) > 1))
                .count();
            if foreground_changed > 50 {
                return Err(format!(
                    "clouds leaked onto opaque foreground: {foreground_changed} changed pixels"
                )
                .into());
            }
            views.push(serde_json::json!({"cloud_toggle_changed_sky_pixels": changed, "cloud_toggle_changed_foreground_pixels": foreground_changed}));
        }
        if name == "sunrise-clear.png" {
            let center = rgba.get_pixel(640, 360).0;
            let adjacent = rgba.get_pixel(700, 360).0;
            if center[..3].iter().map(|v| u32::from(*v)).sum::<u32>()
                <= adjacent[..3].iter().map(|v| u32::from(*v)).sum::<u32>() + 30
            {
                return Err("sun disc is not brighter than adjacent sky".into());
            }
            views.push(serde_json::json!({"sun_disc_center_rgb": &center[..3], "adjacent_sky_rgb": &adjacent[..3]}));
        }
        if name == "shadow-on.png" {
            shadow_pixels = Some(rgba.as_raw().clone());
        }
        if name == "shadow-off.png" {
            let enabled = shadow_pixels
                .as_ref()
                .ok_or("missing shadow-enabled capture")?;
            let darkened = enabled
                .as_chunks::<4>()
                .0
                .iter()
                .zip(rgba.as_raw().as_chunks::<4>().0)
                .skip(1280 * 360)
                .filter(|(a, b)| {
                    a[..3].iter().map(|v| u32::from(*v)).sum::<u32>() + 30
                        < b[..3].iter().map(|v| u32::from(*v)).sum::<u32>()
                })
                .count();
            if darkened < 100 {
                return Err(format!("source terrain did not receive the reference caster shadow: {darkened} darkened pixels").into());
            }
            views.push(serde_json::json!({"shadow_toggle_darkened_ground_pixels": darkened}));
        }
        if name == "z01-overview.png" {
            z01_pixels = Some(rgba.as_raw().clone());
        }
        if name == "z01-hidden.png" {
            let visible = z01_pixels.as_ref().ok_or("missing Z-01 overview")?;
            let changed = visible
                .as_chunks::<4>()
                .0
                .iter()
                .zip(rgba.as_raw().as_chunks::<4>().0)
                .filter(|(a, b)| a[..3].iter().zip(&b[..3]).any(|(a, b)| a.abs_diff(*b) > 2))
                .count();
            if changed < 5000 {
                return Err(format!(
                    "Z-01 terrain is not measurably visible: {changed} changed pixels"
                )
                .into());
            }
            views.push(serde_json::json!({"z01_toggle_changed_pixels":changed}));
        }
        rgba.save(directory.join(name))?;
        if hours == 0.0 && mean_luma >= noon_luma {
            return Err("night capture is not darker than the noon capture".into());
        }
        views.push(serde_json::json!({"file": name, "view_kind": view_kind, "time_hours": hours, "steps": steps, "msaa": msaa.samples(), "rgb_buckets": buckets.len(), "mean_luma": mean_luma, "synchronized_frame_ms": frame_ms, "mean_synchronized_frame_ms": mean_frame_ms}));
    }
    let adapter = app.world().resource::<RenderAdapterInfo>();
    let report = serde_json::json!({"bevy_version": "0.20.1", "gpu": adapter.name, "backend": format!("{:?}", adapter.backend),
        "source_apk_sha256": source_hash, "source_meshes": source_meshes, "camera_position": position.to_array(),
        "z01_source_mesh": z01_name, "z01_bounds": z01_bounds,
        "target_size": [1280,720], "cloud_size": [640,360], "density_size": [64,64,64], "depth_rejection": true,
        "shadow_maps": true, "contact_shadows": DESKTOP_CONTACT_SHADOWS.is_some(), "shadow_normal_bias": DirectionalLight::DEFAULT_SHADOW_NORMAL_BIAS, "views": views,
        "timing_scope": "20 warmed frames per view; app.update plus synchronous GPU drain on the dev build. Includes CPU work; not an in-game FPS benchmark or isolated GPU-pass timing.",
        "limitations": ["Bevy renderer validation; no original-runtime pixel parity is claimed.", "Cloud shadows on terrain and original DayNightSystem animation curves remain future work."]});
    serde_json::to_writer_pretty(
        std::fs::File::create(directory.join("render.json"))?,
        &report,
    )?;
    println!("Environment captures: {}", directory.display());
    Ok(())
}
