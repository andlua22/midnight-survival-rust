//! Headless Vulkan diagnostic of verified player hierarchy and explicit shared
//! pose samples. Run: player_probe PACKAGE_DIR NEW_OUTPUT_DIR
//! Optional: --sample ROOT_NAME CLIP_PATH_ID TIME_SECONDS

use std::{
    collections::BTreeSet,
    path::PathBuf,
    time::{Duration, Instant},
};

use bevy::{
    audio::AudioPlugin,
    camera::RenderTarget,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
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
    time::TimeUpdateStrategy,
    window::ExitCondition,
    winit::WinitPlugin,
};
use client_core::source_player::{
    apply_source_player_diagnostic_pose, spawn_source_player_diagnostic,
    verify_source_player_diagnostic_pose,
};
use midnight_map_data::load_player_rig_package;
use serde_json::json;
use sha2::{Digest, Sha256};
use shared::source_player_pose::{SourceRigClipSample, evaluate_source_rig_pose};

#[derive(Resource, Default)]
struct Capture(Option<Image>);

fn captured(event: On<ScreenshotCaptured>, mut capture: ResMut<Capture>) {
    capture.0 = Some(event.image.clone());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 2 && !(args.len() == 6 && args[2] == "--sample") {
        return Err("usage: player_probe PACKAGE_DIR NEW_OUTPUT_DIR [--sample ROOT_NAME CLIP_PATH_ID TIME_SECONDS]".into());
    }
    let package = load_player_rig_package(&PathBuf::from(&args[0]))?;
    let output = PathBuf::from(&args[1]);
    let sample = if args.len() == 6 {
        Some((
            args[3]
                .to_str()
                .ok_or("root name must be UTF-8")?
                .to_owned(),
            args[4].to_str().ok_or("clip ID must be UTF-8")?.to_owned(),
            args[5]
                .to_str()
                .ok_or("time must be UTF-8")?
                .parse::<f32>()?,
        ))
    } else {
        None
    };
    if let Some((root, _, _)) = &sample
        && package.root_named(root).is_none()
    {
        return Err("sample root is not present in the verified package".into());
    }
    // Refuse to replace a prior capture or report, including failed attempts.
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::create_dir(&output)?;

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: shared::level::asset_root().to_string_lossy().into_owned(),
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
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
    .insert_resource(ClearColor(Color::srgb(0.027, 0.042, 0.069)))
    .init_resource::<Capture>();
    app.finish();
    app.cleanup();

    let mut diagnostics = Vec::new();
    let mut poses = Vec::new();
    let mut selected_sample = None;
    let roots = package.manifest.roots.len();
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for (ordinal, root) in package.manifest.roots.iter().enumerate() {
        let index = package
            .node_index(&root.game_object)
            .ok_or("source root missing")?;
        let node = &package.manifest.nodes[index];
        // This is explicit diagnostic stage spacing, not an original world placement.
        let offset = Vec3::new((ordinal as f32 - (roots - 1) as f32 * 0.5) * 2.5, 0., 0.);
        let placement = (Mat4::from_translation(offset)
            * Mat4::from_cols_array(&node.bevy_local_transform))
        .to_cols_array();
        let rest = evaluate_source_rig_pose(&package, &root.game_object, placement, None)?;
        let mut diagnostic = spawn_source_player_diagnostic(app.world_mut(), &package, &rest)?;
        let pose = if let Some((root_name, clip_id, time)) = &sample
            && root_name == &node.name
        {
            let candidates = package
                .manifest
                .animators
                .iter()
                .filter_map(|animator| {
                    let node = package.node_index(&animator.game_object)?;
                    if rest.nodes[node].is_none() {
                        return None;
                    }
                    let clip = animator
                        .clips
                        .iter()
                        .flatten()
                        .find(|clip| &clip.path_id == clip_id)?;
                    Some((animator, clip))
                })
                .collect::<Vec<_>>();
            if candidates.len() != 1 {
                return Err(
                    "sample must identify one clip on one source Animator under the selected root"
                        .into(),
                );
            }
            let (animator, clip) = candidates[0];
            let decoded = package
                .clips
                .get(clip)
                .ok_or("selected source clip remains deferred")?;
            let pose = evaluate_source_rig_pose(
                &package,
                &root.game_object,
                placement,
                Some(SourceRigClipSample {
                    animator: &animator.source,
                    clip,
                    time_seconds: *time,
                }),
            )?;
            apply_source_player_diagnostic_pose(app.world_mut(), &mut diagnostic, &pose)?;
            selected_sample = Some(
                json!({"root":root.game_object,"animator":animator.source,"clip":clip,
                "clip_name":decoded.name,"time_seconds":time,"unapplied_animator_channels":pose.unapplied_animator_curves.len()}),
            );
            pose
        } else {
            rest
        };
        min = min.min(Vec3::from_array(diagnostic.report.bounds[0]));
        max = max.max(Vec3::from_array(diagnostic.report.bounds[1]));
        for node in pose.nodes.iter().flatten() {
            let point = Mat4::from_cols_array(&node.world_transform)
                .w_axis
                .truncate();
            min = min.min(point);
            max = max.max(point);
        }
        poses.push(pose);
        diagnostics.push(diagnostic);
    }
    if diagnostics.is_empty() {
        return Err("player package contains no roots".into());
    }
    let center = (min + max) * 0.5;
    let size = max - min;
    let vertical_span = size.y.max(size.x / (16. / 9.)).max(2.) * 1.45;
    let camera_position = center + Vec3::new(0., 0.55, 10.);
    let camera_transform = Transform::from_translation(camera_position).looking_at(center, Vec3::Y);
    let camera = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            Msaa::Sample4,
            Tonemapping::None,
            DebandDither::Disabled,
            Projection::Orthographic(OrthographicProjection {
                scale: vertical_span,
                scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                    viewport_height: 1.,
                },
                far: 100.,
                ..OrthographicProjection::default_3d()
            }),
            camera_transform,
        ))
        .id();

    let heading = if selected_sample.is_some() {
        "SOURCE PLAYER / RAW CLIP DIAGNOSTIC"
    } else {
        "SOURCE PLAYER / SERIALIZED REST DIAGNOSTIC"
    };
    let title = app
        .world_mut()
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(30.),
                top: px(20.),
                right: px(30.),
                flex_direction: FlexDirection::Column,
                row_gap: px(6.),
                ..default()
            },
            UiTargetCamera(camera),
        ))
        .id();
    app.world_mut().spawn((
        Text::new(heading),
        TextFont {
            font_size: FontSize::Px(28.),
            ..default()
        },
        TextColor(Color::srgb(0.82, 0.92, 0.98)),
        ChildOf(title),
    ));
    app.world_mut().spawn((
        Text::new("Last Island of Survival LITE 4.0  |  Bevy 0.20.1  |  No runtime skin attached"),
        TextFont {
            font_size: FontSize::Px(17.),
            ..default()
        },
        TextColor(Color::srgb(0.51, 0.65, 0.74)),
        ChildOf(title),
    ));
    let footer = app
        .world_mut()
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(30.),
                bottom: px(20.),
                right: px(30.),
                flex_direction: FlexDirection::Column,
                row_gap: px(5.),
                ..default()
            },
            UiTargetCamera(camera),
        ))
        .id();
    app.world_mut().spawn((Text::new("CYAN  hierarchy    AMBER  RootBone    PINK  HeadTop    BLUE / ORANGE  saved solid / trigger collider"),
        TextFont { font_size: FontSize::Px(15.), ..default() }, TextColor(Color::srgb(0.65,0.79,0.85)), ChildOf(footer)));
    for diagnostic in &diagnostics {
        let r = &diagnostic.report;
        let layers = r
            .saved_layers
            .iter()
            .map(|(layer, count)| format!("{layer}:{count}"))
            .collect::<Vec<_>>()
            .join(", ");
        let bindings = r
            .bindings
            .iter()
            .map(|b| {
                format!(
                    "{}={}",
                    b.field,
                    b.transform
                        .as_ref()
                        .map_or("unresolved".into(), |s| s.path_id.clone())
                )
            })
            .collect::<Vec<_>>()
            .join("  ");
        app.world_mut().spawn((
            Text::new(format!(
                "{}  |  {} nodes  |  saved layers [{layers}]  |  {bindings}",
                r.root_name, r.nodes
            )),
            TextFont {
                font_size: FontSize::Px(15.),
                ..default()
            },
            TextColor(Color::srgb(0.58, 0.67, 0.75)),
            ChildOf(footer),
        ));
    }
    let state_label = match &sample {
        Some((root,clip,time)) => format!("Explicit raw track sample: root {root}, clip {clip}, t={time:.4}s. Root motion and FSM transitions remain unapplied."),
        None => "Saved rest pose. Capsule scaling, live posture, skin attachment and original runtime timing remain unverified.".into(),
    };
    app.world_mut().spawn((
        Text::new(state_label),
        TextFont {
            font_size: FontSize::Px(14.),
            ..default()
        },
        TextColor(Color::srgb(0.70, 0.59, 0.42)),
        ChildOf(footer),
    ));

    let deadline = Instant::now() + Duration::from_secs(180);
    let step = |app: &mut App| -> Result<(), Box<dyn std::error::Error>> {
        if Instant::now() > deadline {
            return Err("player probe exceeded its 180-second render deadline".into());
        }
        app.update();
        app.world()
            .resource::<RenderDevice>()
            .wgpu_device()
            .poll(PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(5)),
            })?;
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
                return Err(format!("player diagnostic GPU pipeline failed: {error}").into());
            }
        }
        Ok(())
    };
    let mut captures = Vec::new();
    for (width, height) in [(1920, 1080), (1280, 720)] {
        let mut image =
            Image::new_target_texture(width, height, TextureFormat::Rgba8UnormSrgb, None);
        image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
        let target = app.world_mut().resource_mut::<Assets<Image>>().add(image);
        app.world_mut()
            .entity_mut(camera)
            .insert(RenderTarget::Image(target.clone().into()));
        let mut stable = 0;
        while stable < 5 {
            step(&mut app)?;
            stable = if app
                .sub_app(RenderApp)
                .world()
                .resource::<PipelineCache>()
                .waiting_pipelines()
                .next()
                .is_none()
            {
                stable + 1
            } else {
                0
            };
        }
        let verification = diagnostics
            .iter()
            .zip(&poses)
            .map(|(diagnostic, pose)| {
                verify_source_player_diagnostic_pose(app.world(), diagnostic, pose)
            })
            .collect::<Result<Vec<_>, _>>()?;
        // Check the actual source edges as well as the resulting matrices.
        for diagnostic in &diagnostics {
            for (index, entity) in diagnostic.nodes.iter().enumerate() {
                let Some(entity) = entity else { continue };
                let expected_parent = package.manifest.nodes[index]
                    .parent
                    .as_ref()
                    .and_then(|p| package.node_index(p))
                    .and_then(|i| diagnostic.nodes[i]);
                let actual_parent = app.world().get::<ChildOf>(*entity).map(ChildOf::parent);
                if expected_parent != actual_parent {
                    return Err("rendered player source hierarchy edge mismatch".into());
                }
            }
        }
        app.world_mut()
            .spawn(Screenshot::image(target.clone()))
            .observe(captured);
        while app.world().resource::<Capture>().0.is_none() {
            step(&mut app)?;
        }
        let image = app
            .world_mut()
            .resource_mut::<Capture>()
            .0
            .take()
            .ok_or("player screenshot absent")?;
        let rgba = image.try_into_dynamic()?.to_rgba8();
        if rgba.width() != width || rgba.height() != height {
            return Err("player capture size mismatch".into());
        }
        let buckets = rgba
            .pixels()
            .map(|p| [p[0] >> 4, p[1] >> 4, p[2] >> 4])
            .collect::<BTreeSet<_>>();
        if buckets.len() < 12 {
            return Err("player diagnostic produced a blank/near-blank image".into());
        }
        let name = format!("player-{width}x{height}.png");
        rgba.save(output.join(&name))?;
        captures.push(json!({"file":name,"width":width,"height":height,
            "rgba_sha256":format!("{:x}",Sha256::digest(rgba.as_raw())),"color_buckets":buckets.len(),
            "pose_verification":verification}));
    }
    let adapter = app.world().resource::<RenderAdapterInfo>();
    let report = json!({"schema_version":1,"mode":"verified-player-rig-diagnostic",
        "source_package":package.root,"manifest_sha256":package.manifest_sha256,
        "source_apk":package.manifest.source_apk,"sample":selected_sample,
        "clock":"fixed explicit pose time; Bevy ManualDuration(0)",
        "camera":{"translation":camera_transform.translation.to_array(),"rotation":camera_transform.rotation.to_array(),
            "projection":"orthographic","vertical_span":vertical_span},
        "renderer":{"adapter":adapter.name,"backend":format!("{:?}",adapter.backend)},
        "rigs":diagnostics.iter().map(|d|&d.report).collect::<Vec<_>>(),"captures":captures,
        "limitations":["Marker geometry is diagnostic. Runtime body skins are not supplied by role/role.ab.",
            "No authoritative pose or clearance origin is read from a rendered frame.",
            "These captures establish package/Bevy transform agreement, not original runtime visual or movement parity."]});
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("render.json"))?;
    serde_json::to_writer_pretty(file, &report)?;
    println!(
        "Verified {} source roots and {} nodes at 1920x1080 and 1280x720; reports: {}",
        diagnostics.len(),
        diagnostics.iter().map(|d| d.report.nodes).sum::<usize>(),
        output.display()
    );
    Ok(())
}
