//! Bounded Linux/Vulkan preview of verified source prefab data, without world placement claims.

use bevy::{
    audio::AudioPlugin,
    camera::RenderTarget,
    core_pipeline::{
        prepass::DepthPrepass,
        tonemapping::{DebandDither, Tonemapping},
    },
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    pbr::ContactShadows,
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
    environment::DESKTOP_CONTACT_SHADOWS,
    source_prefabs::{
        PrefabRenderPolicy, PreparedPrefab, SourcePrefabDocument, SourcePrefabLodCamera,
        SourcePrefabLodStatus, SourcePrefabNode, SourcePrefabPart, SourcePrefabPlugin,
        prepare_prefab,
    },
};
use midnight_map_data::{PrefabPackage, load_prefab_package};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Resource)]
struct Stage {
    package: PrefabPackage,
    center: Vec3,
    bounds: [Vec3; 2],
}
#[derive(Resource)]
struct Prepared(PreparedPrefab);
#[derive(Resource, Default)]
struct Capture {
    image: Option<Image>,
    error: Option<String>,
    strict_error: Option<String>,
}

#[allow(
    clippy::too_many_arguments,
    reason = "Bevy injects the package and distinct asset collections into setup"
)]
fn setup(
    mut commands: Commands,
    stage: Res<Stage>,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut documents: ResMut<Assets<SourcePrefabDocument>>,
    mut capture: ResMut<Capture>,
) {
    let before = [meshes.len(), materials.len(), documents.len()];
    let assets = shared::level::asset_root();
    match prepare_prefab(
        &stage.package,
        &assets,
        &server,
        &mut meshes,
        &mut materials,
        &mut documents,
        PrefabRenderPolicy::Strict,
    ) {
        Err(error) if error.to_string().contains("unsupported samplers") => {
            capture.strict_error = Some(error.to_string())
        }
        Err(error) => {
            capture.error = Some(format!("unexpected strict rejection: {error}"));
            return;
        }
        Ok(_) => {
            capture.error = Some(
                "strict supermarket preparation unexpectedly accepted unsupported glass".into(),
            );
            return;
        }
    }
    if before != [meshes.len(), materials.len(), documents.len()] {
        capture.error = Some("strict rejection mutated Bevy asset collections".into());
        return;
    }
    let prepared = match prepare_prefab(
        &stage.package,
        &assets,
        &server,
        &mut meshes,
        &mut materials,
        &mut documents,
        PrefabRenderPolicy::SupportedPartsPreview,
    ) {
        Ok(prepared) => prepared,
        Err(error) => {
            capture.error = Some(error.to_string());
            return;
        }
    };
    if prepared.report.prepared_parts == 0 {
        capture.error = Some("no supported prefab render parts".into());
        return;
    }
    if prepared
        .spawn_placement(
            &mut commands,
            &stage.package.manifest.roots[0],
            &stage.package.manifest.roots[0],
            Mat4::IDENTITY.to_cols_array(),
        )
        .is_ok()
    {
        capture.error = Some("partial preview entered runtime placement path".into());
        return;
    }
    if let Err(error) = prepared.spawn_preview(&mut commands, -stage.center) {
        capture.error = Some(error.to_string());
        return;
    }
    commands.insert_resource(Prepared(prepared));
    let min = stage.bounds[0] - stage.center;
    let max = stage.bounds[1] - stage.center;
    // An explicit diagnostic stage, not imported map geometry or a source placement.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            (max.x - min.x).max(40.0) * 1.3,
            0.5,
            (max.z - min.z).max(40.0) * 1.3,
        ))),
        MeshMaterial3d(materials.add(Color::srgb(0.18, 0.20, 0.23))),
        Transform::from_xyz(0.0, min.y - 0.5, 0.0),
    ));
    commands.insert_resource(DirectionalLightShadowMap { size: 2048 });
    commands.spawn((
        DirectionalLight {
            illuminance: 35_000.0,
            shadow_maps_enabled: true,
            contact_shadows_enabled: DESKTOP_CONTACT_SHADOWS.is_some(),
            ..default()
        },
        CascadeShadowConfigBuilder {
            maximum_distance: 600.0,
            ..default()
        }
        .build(),
        Transform::default().looking_to(Vec3::new(-0.6, -1.0, -0.4), Vec3::Y),
    ));
}

fn captured(event: On<ScreenshotCaptured>, mut capture: ResMut<Capture>) {
    capture.image = Some(event.image.clone());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/midnight-prefab-probe"));
    let contact_review = match std::env::args_os().nth(2) {
        None => false,
        Some(value) if value == "--review-contact-shadows" => true,
        Some(_) => return Err("expected optional --review-contact-shadows argument".into()),
    };
    std::fs::create_dir_all(&output)?;
    let assets = shared::level::asset_root();
    let path = assets.join("imported_prefabs/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df/building_chaoshi_01-3be4bce3aeca");
    let package = load_prefab_package(&path)?;
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for renderer in &package.manifest.renderers {
        let node = package
            .node_index(&renderer.game_object)
            .ok_or("renderer node absent")?;
        let matrix = Mat4::from_cols_array(&package.poses[node].world_transform);
        for part in &renderer.mesh_parts {
            let mesh = package
                .records
                .iter()
                .find(|mesh| &mesh.name == part)
                .ok_or("mesh absent")?;
            for point in &mesh.positions {
                let point = matrix.transform_point3(Vec3::from_array(*point));
                min = min.min(point);
                max = max.max(point);
            }
        }
    }
    let center = (min + max) * 0.5;
    let manifest_hash = package.manifest_sha256.clone();
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
    .add_plugins((SourceMipImagePlugin, SourcePrefabPlugin))
    .insert_resource(Stage {
        package,
        center,
        bounds: [min, max],
    })
    .init_resource::<Capture>()
    .insert_resource(ClearColor(Color::srgb(0.10, 0.15, 0.21)))
    .insert_resource(GlobalAmbientLight {
        brightness: 200.0,
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
            SourcePrefabLodCamera,
            DepthPrepass,
            RenderTarget::Image(target.clone().into()),
            Msaa::Sample4,
            Tonemapping::TonyMcMapface,
            DebandDither::Disabled,
            Projection::Perspective(PerspectiveProjection {
                fov: 60.0_f32.to_radians(),
                far: 1500.0,
                ..default()
            }),
            Transform::default(),
        ))
        .id();
    let deadline = Instant::now() + Duration::from_secs(180);
    let step = |app: &mut App| -> Result<(), Box<dyn std::error::Error>> {
        if Instant::now() > deadline {
            return Err("prefab probe exceeded 180-second deadline".into());
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
        if let Some(error) = &app.world().resource::<SourcePrefabLodStatus>().error {
            return Err(error.clone().into());
        }
        for pipeline in app
            .sub_app(RenderApp)
            .world()
            .resource::<PipelineCache>()
            .pipelines()
        {
            if let CachedPipelineState::Err(error) = &pipeline.state
                && !matches!(
                    error,
                    ShaderCacheError::ShaderNotLoaded(_)
                        | ShaderCacheError::ShaderImportNotYetAvailable
                )
            {
                return Err(format!("prefab GPU pipeline failed: {error}").into());
            }
        }
        Ok(())
    };
    let mut reports = Vec::new();
    let mut near_parts = 0;
    let mut near_pixels = None;
    let mut diagnostic_pixels = BTreeMap::<String, Vec<u8>>::new();
    let views = if contact_review {
        vec![
            ("near-contact-off.png", 45.0, Vec3::new(1.0, 0.35, 1.0)),
            ("near.png", 45.0, Vec3::new(1.0, 0.35, 1.0)),
            ("near-shadows-off.png", 45.0, Vec3::new(1.0, 0.35, 1.0)),
            ("near-contact-thin.png", 45.0, Vec3::new(1.0, 0.35, 1.0)),
            (
                "near-contact-thin-repeat.png",
                45.0,
                Vec3::new(1.0, 0.35, 1.0),
            ),
            ("near-contact-finer.png", 45.0, Vec3::new(1.0, 0.35, 1.0)),
            (
                "near-contact-finer-repeat.png",
                45.0,
                Vec3::new(1.0, 0.35, 1.0),
            ),
            ("close-contact-off.png", 22.0, Vec3::new(1.0, -0.14, 1.0)),
            (
                "close-contact-default.png",
                22.0,
                Vec3::new(1.0, -0.14, 1.0),
            ),
            ("close-contact-thin.png", 22.0, Vec3::new(1.0, -0.14, 1.0)),
            (
                "close-contact-thin-repeat.png",
                22.0,
                Vec3::new(1.0, -0.14, 1.0),
            ),
            ("close-contact-finer.png", 22.0, Vec3::new(1.0, -0.14, 1.0)),
            (
                "close-contact-finer-repeat.png",
                22.0,
                Vec3::new(1.0, -0.14, 1.0),
            ),
            ("opposite.png", 45.0, Vec3::new(-1.0, 0.25, -1.0)),
            ("far-lod.png", 180.0, Vec3::new(1.0, 0.35, 1.0)),
        ]
    } else {
        vec![
            ("near.png", 45.0, Vec3::new(1.0, 0.35, 1.0)),
            ("near-repeat.png", 45.0, Vec3::new(1.0, 0.35, 1.0)),
            ("near-shadows-off.png", 45.0, Vec3::new(1.0, 0.35, 1.0)),
            ("close.png", 22.0, Vec3::new(1.0, -0.14, 1.0)),
            ("close-repeat.png", 22.0, Vec3::new(1.0, -0.14, 1.0)),
            ("opposite.png", 45.0, Vec3::new(-1.0, 0.25, -1.0)),
            ("far-lod.png", 180.0, Vec3::new(1.0, 0.35, 1.0)),
        ]
    };
    for (name, distance, direction) in views {
        app.world_mut().entity_mut(camera).insert(
            Transform::from_translation(direction.normalize() * distance)
                .looking_at(Vec3::ZERO, Vec3::Y),
        );
        let contact = if !contact_review {
            DESKTOP_CONTACT_SHADOWS
        } else if name.ends_with("contact-off.png") || name == "near-shadows-off.png" {
            None
        } else if name.contains("-thin") {
            Some(ContactShadows {
                linear_steps: 32,
                thickness: 0.01,
                length: 0.1,
            })
        } else if name.contains("-finer") {
            Some(ContactShadows {
                linear_steps: 32,
                thickness: 0.001,
                length: 0.1,
            })
        } else {
            Some(ContactShadows::default())
        };
        if let Some(contact) = contact {
            app.world_mut().entity_mut(camera).insert(contact);
        } else {
            app.world_mut()
                .entity_mut(camera)
                .remove::<ContactShadows>();
        }
        let mut lights = app.world_mut().query::<&mut DirectionalLight>();
        for mut light in lights.iter_mut(app.world_mut()) {
            light.shadow_maps_enabled = name != "near-shadows-off.png";
            light.contact_shadows_enabled = light.shadow_maps_enabled && contact.is_some();
        }
        let mut stable = 0;
        while stable < 5 {
            step(&mut app)?;
            let loaded = app
                .world()
                .get_resource::<Prepared>()
                .is_some_and(|prepared| {
                    prepared.0.textures.iter().all(|(handle, _)| {
                        app.world()
                            .resource::<AssetServer>()
                            .is_loaded_with_dependencies(handle.id())
                    })
                });
            let pipelines = app.sub_app(RenderApp).world().resource::<PipelineCache>();
            stable = if loaded && pipelines.waiting_pipelines().next().is_none() {
                stable + 1
            } else {
                0
            };
        }
        // Compare every composed Bevy node pose with the independently verified package pose.
        let mut query = app
            .world_mut()
            .query::<(&SourcePrefabNode, &GlobalTransform)>();
        let expected = &app.world().resource::<Stage>().package.poses;
        let offset = Mat4::from_translation(-center);
        let mut node_count = 0;
        for (node, global) in query.iter(app.world()) {
            let expected =
                offset * Mat4::from_cols_array(&expected[node.node_index].world_transform);
            if !global.to_matrix().abs_diff_eq(expected, 0.002) {
                return Err(
                    format!("prefab hierarchy pose mismatch: {}", node.source.path_id).into(),
                );
            }
            node_count += 1;
        }
        if node_count != app.world().resource::<Stage>().package.manifest.nodes.len() {
            return Err("prefab hierarchy dropped source nodes".into());
        }
        let mut parts = app.world_mut().query::<(&SourcePrefabPart, &Visibility)>();
        let visible_parts = parts
            .iter(app.world())
            .filter(|(_, visibility)| **visibility != Visibility::Hidden)
            .count();
        if name == "near.png" {
            near_parts = visible_parts;
        }
        if name == "far-lod.png" && (visible_parts >= near_parts || visible_parts == 0) {
            return Err(format!(
                "source LOD did not reduce visible parts: near{near_parts}, far{visible_parts}"
            )
            .into());
        }
        for (handle, source) in &app.world().resource::<Prepared>().0.textures {
            let image = app
                .world()
                .resource::<Assets<Image>>()
                .get(handle)
                .ok_or("source texture unavailable")?;
            if image.texture_descriptor.mip_level_count != source.mip_count
                || image.width() != source.width
                || image.height() != source.height
            {
                return Err("source texture dimensions/mips changed".into());
            }
        }
        app.world_mut()
            .spawn(Screenshot::image(target.clone()))
            .observe(captured);
        while app.world().resource::<Capture>().image.is_none() {
            step(&mut app)?;
        }
        let image = app
            .world_mut()
            .resource_mut::<Capture>()
            .image
            .take()
            .ok_or("missing prefab capture")?;
        let rgba = image.try_into_dynamic()?.to_rgba8();
        let buckets = rgba
            .pixels()
            .map(|pixel| [pixel.0[0] >> 4, pixel.0[1] >> 4, pixel.0[2] >> 4])
            .collect::<BTreeSet<_>>();
        let group = if name.starts_with("close") {
            "close"
        } else {
            "near"
        };
        let reference = if contact_review {
            format!("{group}-contact-off.png")
        } else {
            format!("{group}.png")
        };
        let wall_region = |index: usize| {
            let x = index % 1280;
            let y = index / 1280;
            if group == "close" {
                (625..720).contains(&x) && (320..350).contains(&y)
            } else {
                (637..667).contains(&x) && (415..454).contains(&y)
            }
        };
        let dark_wall = diagnostic_pixels
            .get(&reference)
            .filter(|_| name.starts_with(group))
            .map(|reference| {
                rgba.as_raw()
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .zip(reference.as_chunks::<4>().0)
                    .enumerate()
                    .filter(|(index, _)| wall_region(*index))
                    .filter(|(_, (a, b))| {
                        a[..3].iter().map(|value| u32::from(*value)).sum::<u32>() + 30
                            < b[..3].iter().map(|value| u32::from(*value)).sum::<u32>()
                    })
                    .count()
            });
        if let Some(base) = name.strip_suffix("-repeat.png")
            && let Some(previous) = diagnostic_pixels.get(&format!("{base}.png"))
        {
            let changed = rgba
                .as_raw()
                .as_chunks::<4>()
                .0
                .iter()
                .zip(previous.as_chunks::<4>().0)
                .enumerate()
                .filter(|(index, _)| wall_region(*index))
                .filter(|(_, (a, b))| a[..3].iter().zip(&b[..3]).any(|(a, b)| a.abs_diff(*b) > 8))
                .count();
            if !contact_review && changed != 0 {
                return Err(format!(
                    "accepted desktop shadows changed wall pixels between frames: {name}, {changed}"
                )
                .into());
            }
            reports.push(serde_json::json!({"repeat": name, "wall_changed_pixels": changed}));
        }
        diagnostic_pixels.insert(name.into(), rgba.as_raw().clone());
        if name == "near.png" {
            near_pixels = Some(rgba.as_raw().clone());
        }
        if name == "near-shadows-off.png" {
            let enabled = near_pixels.as_ref().ok_or("missing near shadow capture")?;
            let darkened = enabled
                .as_chunks::<4>()
                .0
                .iter()
                .zip(rgba.as_raw().as_chunks::<4>().0)
                .skip(1280 * 360)
                .filter(|(a, b)| {
                    a[..3].iter().map(|value| u32::from(*value)).sum::<u32>() + 30
                        < b[..3].iter().map(|value| u32::from(*value)).sum::<u32>()
                })
                .count();
            if darkened < 100 {
                return Err(format!("source prefab did not cast a measurable stage-ground shadow: {darkened} darkened pixels").into());
            }
            reports.push(serde_json::json!({"shadow_toggle_darkened_lower_pixels": darkened}));
        }
        if buckets.len() < 8 {
            return Err(format!("{name}: nearly empty prefab capture").into());
        }
        rgba.save(output.join(name))?;
        reports.push(serde_json::json!({"file":name,"camera_distance":distance,"source_nodes":node_count,"visible_parts":visible_parts,"rgb_buckets":buckets.len(),"shadow_maps_enabled":name != "near-shadows-off.png","contact_shadows_enabled":contact.is_some(),"contact_steps":contact.map(|settings| settings.linear_steps),"contact_thickness":contact.map(|settings| settings.thickness),"contact_length":contact.map(|settings| settings.length),"wall_darkened_vs_reference":dark_wall}));
    }
    let adapter = app.world().resource::<RenderAdapterInfo>();
    let prepared = &app.world().resource::<Prepared>().0;
    let report = serde_json::json!({"bevy_version":"0.20.1","gpu":adapter.name,"backend":format!("{:?}",adapter.backend),"manifest_sha256":manifest_hash,
        "contact_shadow_review":contact_review,"desktop_contact_shadows_enabled":DESKTOP_CONTACT_SHADOWS.is_some(),
        "wall_comparison":{"near":[637,415,30,39],"close":[625,320,95,30],"darkening_threshold_rgb_sum":30,"repeat_threshold_any_rgb_channel":8},
        "preview_only":true,"diagnostic_stage_offset":(-center).to_array(),"verified_hierarchy":true,"verified_source_mips":true,
        "strict_rejected_before_assets":true,"partial_placement_rejected":true,"strict_error":app.world().resource::<Capture>().strict_error,
        "support":prepared.report,"textures":prepared.textures.iter().map(|(_,source)|source).collect::<Vec<_>>(),"views":reports,
        "limitations":["Standalone preview; no authenticated world placements are inferred.","Source shader execution and baked/probe lighting are not reproduced. Glass/cubemap parts are explicitly omitted.","Source non-fading LOD thresholds use a single perspective camera and unit native LOD bias; original QualitySettings.lodBias remains unknown."]});
    serde_json::to_writer_pretty(std::fs::File::create(output.join("render.json"))?, &report)?;
    println!("Prefab preview captures: {}", output.display());
    Ok(())
}
