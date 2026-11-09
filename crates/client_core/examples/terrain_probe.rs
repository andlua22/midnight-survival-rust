//! Render the production canonical terrain without a window and retain GPU evidence.
//! Run from the workspace: cargo run --offline -p client_core --example terrain_probe -- OUTPUT_DIR

use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    time::{Duration, Instant},
};

use bevy::{
    audio::AudioPlugin,
    camera::{RenderTarget, ScalingMode},
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    prelude::*,
    render::{
        RenderApp, RenderPlugin,
        pipelined_rendering::PipelinedRenderingPlugin,
        render_asset::RenderAssets,
        render_resource::{
            CachedPipelineState, PipelineCache, PipelineDescriptor, PollType, TextureFormat,
            TextureUsages,
        },
        renderer::{RenderAdapterInfo, RenderDevice},
        texture::GpuImage,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    shader::ShaderCacheError,
    window::ExitCondition,
    winit::WinitPlugin,
};
use client_core::SourceMipImagePlugin;
use client_core::application::{TerrainSectorChunk, TerrainSplatMaterial, spawn_imported_terrain};
use midnight_map_data::{TerrainPackage, load_terrain_package};
use serde::Serialize;

#[derive(Resource)]
struct Package(TerrainPackage);

#[derive(Resource, Default)]
struct Capture {
    image: Option<Image>,
    error: Option<String>,
}

#[derive(Serialize)]
struct ViewEvidence {
    file: String,
    camera_position: [f32; 3],
    look_at: [f32; 3],
    distinct_rgb_buckets: usize,
}

fn spawn_scene(
    mut commands: Commands,
    package: Res<Package>,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut splats: ResMut<Assets<TerrainSplatMaterial>>,
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
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(1.0, 2.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn captured(event: On<ScreenshotCaptured>, mut capture: ResMut<Capture>) {
    capture.image = Some(event.image.clone());
}

fn update(app: &mut App, deadline: Instant) -> Result<(), Box<dyn std::error::Error>> {
    if Instant::now() >= deadline {
        return Err("terrain render probe exceeded its 90-second deadline".into());
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
        if let CachedPipelineState::Err(error) = &pipeline.state {
            if !matches!(
                error,
                ShaderCacheError::ShaderNotLoaded(_)
                    | ShaderCacheError::ShaderImportNotYetAvailable
            ) {
                return Err(format!("GPU pipeline failed: {error}").into());
            }
        }
    }
    Ok(())
}

fn source_ready(app: &App, shader: &Handle<Shader>) -> bool {
    let world = app.world();
    let server = world.resource::<AssetServer>();
    let images = app
        .sub_app(RenderApp)
        .world()
        .resource::<RenderAssets<GpuImage>>();
    let textures_ready = |image: &Handle<Image>| {
        server.is_loaded_with_dependencies(image.id()) && images.get(image.id()).is_some()
    };
    if !server.is_loaded_with_dependencies(shader.id()) {
        return false;
    }
    for (_, material) in world.resource::<Assets<TerrainSplatMaterial>>().iter() {
        for image in [
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
        {
            if !textures_ready(image) {
                return false;
            }
        }
    }
    for (_, material) in world.resource::<Assets<StandardMaterial>>().iter() {
        if material
            .base_color_texture
            .as_ref()
            .is_some_and(|image| !textures_ready(image))
        {
            return false;
        }
    }
    let cache = app.sub_app(RenderApp).world().resource::<PipelineCache>();
    cache.waiting_pipelines().next().is_none() && cache.pipelines().any(|pipeline| {
        matches!(&pipeline.descriptor, PipelineDescriptor::RenderPipelineDescriptor(descriptor)
            if descriptor.fragment.as_ref().is_some_and(|fragment| fragment.shader.id() == shader.id()))
            && matches!(pipeline.state, CachedPipelineState::Ok(_))
    })
}

fn source_mip_evidence(app: &App) -> Result<BTreeMap<u32, usize>, Box<dyn std::error::Error>> {
    let world = app.world();
    let package = &world.resource::<Package>().0;
    let expected = package
        .manifest
        .meshes
        .values()
        .flat_map(|mesh| mesh.material.textures.values())
        .filter_map(|binding| binding.texture.as_ref())
        .filter_map(|texture| texture.gpu.as_ref().map(|gpu| (gpu.path.as_str(), texture)))
        .collect::<BTreeMap<_, _>>();
    let server = world.resource::<AssetServer>();
    let gpu_images = app
        .sub_app(RenderApp)
        .world()
        .resource::<RenderAssets<GpuImage>>();
    let mut counts = BTreeMap::new();
    for (id, image) in world.resource::<Assets<Image>>().iter() {
        let Some(path) = server.get_path(id) else {
            continue;
        };
        let Some(relative) = path
            .path()
            .to_str()
            .and_then(|path| path.strip_prefix("imported_map/terrain/"))
        else {
            continue;
        };
        let source = expected
            .get(relative)
            .ok_or("loaded terrain texture has no source manifest entry")?;
        let gpu = gpu_images
            .get(id)
            .ok_or("source texture was not uploaded to the GPU")?;
        let descriptor = &image.texture_descriptor;
        let format = if source.unity_color_space == 1 {
            TextureFormat::Rgba8UnormSrgb
        } else {
            TextureFormat::Rgba8Unorm
        };
        if (
            descriptor.size.width,
            descriptor.size.height,
            descriptor.mip_level_count,
            descriptor.format,
        ) != (source.width, source.height, source.mip_count, format)
            || gpu.texture.mip_level_count() != source.mip_count
        {
            return Err(format!(
                "{}: source mip metadata differs from the uploaded image",
                source.name
            )
            .into());
        }
        *counts.entry(source.mip_count).or_default() += 1;
    }
    if counts.is_empty() {
        return Err("no source mip textures were uploaded".into());
    }
    Ok(counts)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::args_os().nth(1).map_or_else(
        || PathBuf::from("memory/evidence/terrain-render"),
        PathBuf::from,
    );
    std::fs::create_dir_all(&directory)?;
    let assets = shared::level::asset_root();
    let package = load_terrain_package(&assets.join("imported_map/terrain"))?;
    let source_hash = package.manifest.source_apk.sha256.clone();
    let geometry_hash = package.manifest.geometry.sha256.clone();
    let record_count = package.records.len();
    let bounds = package.manifest.meshes.values().fold(
        [
            f32::INFINITY,
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ],
        |mut bounds, mesh| {
            for axis in 0..3 {
                bounds[axis] = bounds[axis].min(mesh.bounds[axis]);
                bounds[axis + 3] = bounds[axis + 3].max(mesh.bounds[axis + 3]);
            }
            bounds
        },
    );
    let mut collision = shared::physics::CollisionWorld::new(shared::level::MapId::Polygon);
    collision.insert_canonical_terrain(&package)?;
    let spawn = collision
        .surface_spawn_at([0.0, 12.0], None)
        .ok_or("initial player location has no free source terrain surface")?;
    let center = Vec3::new(
        (bounds[0] + bounds[3]) * 0.5,
        0.0,
        (bounds[2] + bounds[5]) * 0.5,
    );
    let size = UVec2::new(1280, 1024);
    let height =
        (bounds[5] - bounds[2]).max((bounds[3] - bounds[0]) * size.y as f32 / size.x as f32) * 1.04;
    let overhead = center + Vec3::Y * 8000.0;

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
                ..default()
            })
            .disable::<WinitPlugin>()
            .disable::<PipelinedRenderingPlugin>()
            .disable::<AudioPlugin>(),
    )
    .add_plugins((
        SourceMipImagePlugin,
        MaterialPlugin::<TerrainSplatMaterial>::default(),
    ))
    .insert_resource(Package(package))
    .init_resource::<Capture>()
    .insert_resource(GlobalAmbientLight {
        brightness: 80.0,
        ..default()
    })
    .insert_resource(ClearColor(Color::srgb(0.01, 0.01, 0.02)))
    .add_systems(Startup, spawn_scene);
    app.finish();
    app.cleanup();
    let mut target_image =
        Image::new_target_texture(size.x, size.y, TextureFormat::Rgba8UnormSrgb, None);
    target_image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let target = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(target_image);
    let shader = app
        .world()
        .resource::<AssetServer>()
        .load::<Shader>("shaders/terrain_splat.wesl");
    let camera = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            RenderTarget::Image(target.clone().into()),
            Msaa::Off,
            Tonemapping::AgX,
            DebandDither::Disabled,
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: height,
                },
                far: 20_000.0,
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_translation(overhead).looking_at(center, Vec3::NEG_Z),
        ))
        .id();

    let deadline = Instant::now() + Duration::from_secs(90);
    let mut views = Vec::new();
    for view in 0..2 {
        let (filename, position, look_at) = if view == 0 {
            ("overview.png", overhead, center)
        } else {
            let look_at = Vec3::from_array(spawn);
            let position = look_at + Vec3::new(45.0, 35.0, 65.0);
            app.world_mut().entity_mut(camera).insert((
                Projection::Perspective(PerspectiveProjection {
                    far: 12_000.0,
                    fov: 65.0_f32.to_radians(),
                    ..default()
                }),
                Transform::from_translation(position).looking_at(look_at, Vec3::Y),
            ));
            ("spawn.png", position, look_at)
        };
        let mut ready_frames = 0;
        while ready_frames < 3 {
            update(&mut app, deadline)?;
            ready_frames = if source_ready(&app, &shader) {
                ready_frames + 1
            } else {
                0
            };
        }
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
            .ok_or("screenshot observer returned no image")?;
        let rgba = image.try_into_dynamic()?.to_rgba8();
        let colors = rgba
            .pixels()
            .map(|pixel| [pixel.0[0] >> 4, pixel.0[1] >> 4, pixel.0[2] >> 4])
            .collect::<BTreeSet<_>>();
        if colors.len() < 16 {
            return Err(format!("{filename}: rendered image has too little color variation to verify textured terrain").into());
        }
        rgba.save(directory.join(filename))?;
        views.push(ViewEvidence {
            file: filename.into(),
            camera_position: position.to_array(),
            look_at: look_at.to_array(),
            distinct_rgb_buckets: colors.len(),
        });
    }
    let entity_count = app
        .world_mut()
        .query::<&TerrainSectorChunk>()
        .iter(app.world())
        .count();
    if entity_count != record_count {
        return Err("renderer did not spawn every canonical source mesh".into());
    }
    let adapter = app.world().resource::<RenderAdapterInfo>();
    let source_mip_textures = source_mip_evidence(&app)?;
    let report = serde_json::json!({
        "source_apk_sha256": source_hash, "geometry_sha256": geometry_hash, "source_mesh_entities": entity_count,
        "collision_records": app.world().resource::<Package>().0.collision_records().count(), "player_spawn": spawn,
        "gpu": adapter.name, "driver": adapter.driver_info, "backend": format!("{:?}", adapter.backend),
        "bevy_version": "0.20.1", "target_size": size.to_array(), "texture_format": "Rgba8UnormSrgb",
        "directional_illuminance": 10_000.0, "ambient_brightness": 80.0, "shadows": false, "tonemapping": "AgX",
        "source_mip_textures_by_level_count": source_mip_textures,
        "manifest_schema": midnight_map_data::TERRAIN_MANIFEST_VERSION,
        "views": views,
        "limitations": ["These are renderer validation images, not screenshots of the APK.",
            "Lighting is a fixed Bevy probe setup. Original lighting, mip sampling and non-terrain shaders remain unverified.",
            "This probe covers the canonical terrain scene package; legacy water, model placements and the game HUD are excluded."]
    });
    serde_json::to_writer_pretty(
        std::fs::File::create(directory.join("render.json"))?,
        &report,
    )?;
    println!(
        "Rendered {entity_count} source mesh instances with the production terrain shader. Evidence: {}",
        directory.display()
    );
    Ok(())
}
