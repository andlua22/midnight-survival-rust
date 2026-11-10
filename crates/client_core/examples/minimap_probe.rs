//! Capture the source BattleMinimap with the production egui backend, without a
//! window. Run: cargo run --offline -p client_core --example minimap_probe -- DIR [PROFILE]
//! A visibility profile adds initialization previews. DIR --native captures the
//! production native HUD adapter with movement, heading, zoom and live markers.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    audio::AudioPlugin,
    camera::{RenderTarget, visibility::RenderLayers},
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    prelude::*,
    render::{
        RenderApp, RenderPlugin,
        pipelined_rendering::PipelinedRenderingPlugin,
        render_asset::RenderAssets,
        render_resource::{
            CachedPipelineState, PipelineCache, PollType, TextureFormat, TextureUsages,
        },
        renderer::{RenderAdapterInfo, RenderDevice},
        texture::GpuImage,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    shader::ShaderCacheError,
    window::ExitCondition,
    winit::WinitPlugin,
};
use bevy_egui::{
    EguiContext, EguiContexts, EguiGlobalSettings, EguiManagedTextures, EguiPlugin,
    EguiPrimaryContextPass, PrimaryEguiContext, egui,
};
use client_core::source_ui::{PanelRenderReport, SourcePanelRenderer, SourceUiBindings};
use client_core::{
    hud::{HudLayout, WidgetId},
    source_minimap::{DEFAULT_ZOOM, MinimapMarker, MinimapMarkerKind, MinimapState, NativeMinimap},
};
use midnight_map_data::SourceObject;
use midnight_ui_data::{UiError, layout::CanvasViewport, load_ui_package};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct InitializationProfile {
    schema_version: u32,
    name: String,
    source_apk: ProfileApk,
    panel: ProfilePanel,
    /// Static evidence only; this object never supplies renderer bindings.
    evidence: Value,
    visibility: Vec<VisibilityOverride>,
    caveats: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProfileApk {
    path: String,
    sha256: String,
    byte_count: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProfilePanel {
    name: String,
    bundle: String,
    cab: String,
    bundle_sha256: String,
    bundle_byte_count: u64,
    layout: ProfileFile,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProfileFile {
    path: String,
    sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(remote = "SourceObject", deny_unknown_fields)]
struct ProfileObject {
    bundle: String,
    cab: String,
    path_id: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct VisibilityOverride {
    #[serde(with = "ProfileObject")]
    node: SourceObject,
    active_self: bool,
    field: String,
    retained_index: usize,
    hierarchy_path: String,
    #[serde(with = "ProfileObject")]
    transform: SourceObject,
    native_method: String,
    false_transfer_va: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dictionary_insertion_va: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    false_argument_va: Option<String>,
}

struct CaptureMode {
    name: &'static str,
    bindings: SourceUiBindings,
    profile: Value,
    caveats: Vec<String>,
    native_state: Option<MinimapState>,
    zoom: f32,
}

fn invalid(message: impl Into<String>) -> UiError {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()).into()
}

fn validate_profile_object(source: &SourceObject, panel: &ProfilePanel) -> Result<(), UiError> {
    let canonical_nonzero_id = source
        .path_id
        .parse::<i64>()
        .is_ok_and(|id| id != 0 && id.to_string() == source.path_id);
    if source.bundle != panel.bundle || source.cab != panel.cab || !canonical_nonzero_id {
        return Err(invalid(format!(
            "initialization profile object has an invalid bundle/CAB/signed PathID: {source:?}"
        )));
    }
    Ok(())
}

fn load_initialization_profile(path: &Path, source: &Path) -> Result<CaptureMode, UiError> {
    let bytes = std::fs::read(path)?;
    let profile: InitializationProfile = serde_json::from_slice(&bytes)?;
    if profile.schema_version != 1
        || profile.name != "battle-minimap-initialization-visibility"
        || profile.visibility.len() != 21
    {
        return Err(invalid(
            "expected version 1 battle-minimap-initialization-visibility profile with 21 overrides",
        ));
    }
    let package = load_ui_package(source)?;
    // load_panel verifies the layout payload's checksum and its manifest identity.
    let panel = package.load_panel("BattleMinimap")?;
    let source_apk = &package.manifest.source_apk;
    let summary = package
        .manifest
        .panels
        .iter()
        .find(|summary| summary.name == panel.name)
        .ok_or_else(|| invalid("BattleMinimap summary is absent"))?;
    let layout = summary
        .layout
        .as_ref()
        .ok_or_else(|| invalid("BattleMinimap layout identity is absent"))?;
    let bundles: Vec<_> = package
        .manifest
        .bundles
        .iter()
        .filter(|bundle| bundle.member == panel.bundle)
        .collect();
    let [bundle] = bundles.as_slice() else {
        return Err(invalid("expected one BattleMinimap bundle identity"));
    };
    if profile.source_apk.path != source_apk.path
        || profile.source_apk.sha256 != source_apk.sha256
        || profile.source_apk.byte_count != source_apk.byte_count
        || profile.panel.name != panel.name
        || profile.panel.bundle != panel.bundle
        || profile.panel.cab != panel.root.source.cab
        || profile.panel.bundle_sha256 != bundle.sha256
        || profile.panel.bundle_byte_count != bundle.byte_count
        || !bundle.cabs.iter().any(|cab| cab.cab == profile.panel.cab)
        || profile.panel.layout.path != layout.path
        || profile.panel.layout.sha256 != layout.sha256
    {
        return Err(invalid(
            "initialization profile APK/panel/layout identity differs from verified UI package",
        ));
    }
    let mut bindings = SourceUiBindings::default();
    let mut transforms = BTreeSet::new();
    let mut retained_indices = BTreeSet::new();
    for entry in &profile.visibility {
        validate_profile_object(&entry.node, &profile.panel)?;
        validate_profile_object(&entry.transform, &profile.panel)?;
        if entry.active_self
            || entry.node == entry.transform
            || entry.retained_index >= 86
            || entry.field.is_empty()
            || !entry.hierarchy_path.starts_with("BattleMinimap/")
        {
            return Err(invalid(format!(
                "initialization profile requires false GO visibility and retained source provenance: {}",
                entry.field
            )));
        }
        if bindings
            .visibility
            .insert(entry.node.clone(), false)
            .is_some()
            || !transforms.insert(entry.transform.clone())
            || !retained_indices.insert(entry.retained_index)
        {
            return Err(invalid(format!(
                "duplicate initialization profile GO/Transform/index: {}",
                entry.field
            )));
        }
    }
    let mut caveats = vec![
        "Initialization visibility is an explicit preview from static ARM64 fallback evidence; successful native Awake/init and IFix execution are unobserved.".into(),
        "The retained GameObjectData has 86 entries; native Awake requests 94. Additional binding entries remain unresolved.".into(),
        "Only the listed GO activeSelf values are overridden. Inactive parents also hide descendants; source records remain retained.".into(),
        "The constructor zoom default 15 is not applied as RawImage UVs or a transform scale. Runtime texture, UV assignment and inherited scale remain unresolved.".into(),
        "Later show/update calls, pooled sound objects and gameplay events can produce different visibility.".into(),
    ];
    caveats.extend(profile.caveats.iter().cloned());
    let evidence = json!({
        "file": {"path": path.display().to_string(), "sha256": format!("{:x}", Sha256::digest(&bytes))},
        "document": profile,
        "validation": {
            "apk_and_bundle_identities": "compared with imported UI manifest; APK/native members are not rehashed by this probe",
            "layout": "payload SHA-256 and panel identity verified by load_ui_package/load_panel",
            "source_node_targets": "validated by SourcePanelRenderer::paint against the selected layout",
            "visibility": "21 unique qualified GO targets, canonical nonzero signed PathIDs, active_self=false"
        }
    });
    Ok(CaptureMode {
        name: "initialization",
        bindings,
        profile: evidence,
        caveats,
        native_state: None,
        zoom: DEFAULT_ZOOM,
    })
}

#[derive(Resource)]
struct Preview {
    renderer: PreviewRenderer,
    bindings: SourceUiBindings,
    native_state: Option<MinimapState>,
    zoom: f32,
    viewport: CanvasViewport,
    report: Option<PanelRenderReport>,
    error: Option<String>,
}

enum PreviewRenderer {
    Source(Box<SourcePanelRenderer>),
    Native(Box<NativeMinimap>),
}

fn native_modes() -> Vec<CaptureMode> {
    let state = MinimapState {
        player_position: [2250.0, 84.0, -700.0],
        yaw: 0.0,
        time_hours: 10.0,
        waypoint: Some([2490.0, -700.0]),
        markers: vec![
            ("north-tree", MinimapMarkerKind::Tree, [2250.0, -900.0]),
            ("east-vehicle", MinimapMarkerKind::Vehicle, [2370.0, -700.0]),
            ("south-enemy", MinimapMarkerKind::Enemy, [2250.0, -500.0]),
            (
                "west-cupboard",
                MinimapMarkerKind::ToolCupboard,
                [2050.0, -700.0],
            ),
            ("gunfire", MinimapMarkerKind::Gunfire, [2230.0, -670.0]),
            (
                "far-footstep",
                MinimapMarkerKind::Footstep,
                [1000.0, -2000.0],
            ),
        ]
        .into_iter()
        .map(|(key, kind, world_xz)| MinimapMarker {
            key: key.into(),
            kind,
            world_xz,
            yaw: 0.0,
            intensity: 0.85,
        })
        .collect(),
    };
    let mut east = state.clone();
    east.yaw = -std::f32::consts::FRAC_PI_2;
    let mut moved = east.clone();
    moved.player_position = [2300.0, 84.0, -750.0];
    let mut night = moved.clone();
    night.time_hours = 23.0;
    [("native-north", state, DEFAULT_ZOOM), ("native-east", east, DEFAULT_ZOOM),
        ("native-moved", moved, DEFAULT_ZOOM), ("native-zoomed-night", night, 30.0)]
        .into_iter().map(|(name, state, zoom)| CaptureMode {
            name, bindings: SourceUiBindings::default(), profile: Value::Null,
            caveats: vec![
                "This exercises the production native HUD adapter with explicit simulation inputs; it does not launch Waydroid or execute Unity scripts.".into(),
                "The retained Map_map texture, source hierarchy/sprite UVs/masks/fonts are used. Minimap texture assignment, crop zoom, marker sizes/categories and lifecycle are explicit native policies.".into(),
                "The shared overview extent remains provisional. These captures verify native rendering, not pixel equality with the APK.".into(),
            ], native_state: Some(state), zoom,
        }).collect()
}

#[derive(Resource, Default)]
struct Capture {
    image: Option<Image>,
}

fn draw(mut contexts: EguiContexts, mut preview: ResMut<Preview>) {
    let Ok(context) = contexts.ctx_mut() else {
        preview.error = Some("preview egui context is absent".into());
        return;
    };
    let viewport = preview.viewport;
    let points = context.pixels_per_point();
    let painter = context
        .layer_painter(egui::LayerId::new(
            egui::Order::Background,
            egui::Id::new("source-battle-minimap"),
        ))
        .with_clip_rect(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(
                viewport.pixels[0] as f32 / points,
                viewport.pixels[1] as f32 / points,
            ),
        ));
    let result = {
        let Preview {
            renderer,
            bindings,
            native_state,
            zoom,
            ..
        } = &mut *preview;
        match renderer {
            PreviewRenderer::Source(renderer) => {
                renderer.paint(&painter, viewport, 100.0, bindings)
            }
            PreviewRenderer::Native(renderer) => {
                let logical = viewport.pixels.map(|value| value as f32 / points);
                let config = HudLayout::default().widgets[&WidgetId::Minimap].clone();
                let placement = config.placement(WidgetId::Minimap, logical);
                let rect = egui::Rect::from_min_size(
                    egui::Pos2::from(placement.position),
                    egui::Vec2::from(placement.size),
                );
                let mut painter = painter.with_clip_rect(rect);
                painter.set_opacity(config.opacity);
                renderer.set_zoom(*zoom);
                native_state
                    .as_ref()
                    .ok_or_else(|| invalid("native capture state is absent"))
                    .and_then(|state| renderer.paint(&painter, rect, placement.render_scale, state))
            }
        }
    };
    match result {
        Ok(report) => preview.report = Some(report),
        Err(error) => preview.error = Some(error.to_string()),
    }
}

fn captured(event: On<ScreenshotCaptured>, mut capture: ResMut<Capture>) {
    capture.image = Some(event.image.clone());
}

fn update(
    app: &mut App,
    deadline: Instant,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if Instant::now() >= deadline {
        return Err("minimap probe exceeded its 90-second render deadline".into());
    }
    app.update();
    app.world()
        .resource::<RenderDevice>()
        .wgpu_device()
        .poll(PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(5)),
        })?;
    if let Some(error) = &app.world().resource::<Preview>().error {
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
            return Err(format!("minimap GPU pipeline failed: {error}").into());
        }
    }
    Ok(())
}

fn ready(app: &App) -> bool {
    let managed = app.world().resource::<EguiManagedTextures>();
    let render = app.sub_app(RenderApp).world();
    let gpu = render.resource::<RenderAssets<GpuImage>>();
    let cache = render.resource::<PipelineCache>();
    !managed.is_empty()
        && managed
            .values()
            .all(|texture| gpu.get(texture.handle.id()).is_some())
        && cache.waiting_pipelines().next().is_none()
        && app
            .world()
            .resource::<Preview>()
            .report
            .as_ref()
            .is_some_and(|report| {
                report
                    .graphics
                    .iter()
                    .any(|graphic| graphic.status == "submitted")
            })
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut arguments = std::env::args_os().skip(1);
    let directory = arguments.next().map_or_else(
        || PathBuf::from("memory/evidence/minimap-render"),
        PathBuf::from,
    );
    let second = arguments.next();
    let native_only = second.as_deref() == Some(std::ffi::OsStr::new("--native"));
    let profile_path = if native_only {
        None
    } else {
        second.map(PathBuf::from)
    };
    if arguments.next().is_some() {
        return Err("usage: minimap_probe [DIR [PROFILE | --native]]".into());
    }
    let source = shared::level::asset_root()
        .join("imported_ui/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1");
    let mut modes = if native_only {
        native_modes()
    } else {
        vec![CaptureMode {
        name: "prefab",
        bindings: SourceUiBindings::default(),
        profile: Value::Null,
        native_state: None,
        zoom: DEFAULT_ZOOM,
        caveats: vec![
            "This mode renders authored prefab visibility with empty bindings; templates and placeholders retain their serialized activeSelf values.".into(),
            "It does not execute native initialization, show/update calls or gameplay state.".into(),
        ],
    }]
    };
    if let Some(path) = profile_path {
        // Reject mismatched profiles before initializing the GPU or writing captures.
        modes.push(load_initialization_profile(&path, &source)?);
    }
    std::fs::create_dir_all(&directory)?;
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
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
    .add_plugins(EguiPlugin::default())
    .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.05)))
    .init_resource::<Capture>()
    .add_systems(EguiPrimaryContextPass, draw);
    app.world_mut()
        .resource_mut::<EguiGlobalSettings>()
        .auto_create_primary_context = false;
    app.finish();
    app.cleanup();
    let camera = app
        .world_mut()
        .spawn((
            PrimaryEguiContext,
            Camera3d::default(),
            RenderLayers::none(),
            Msaa::Off,
            Tonemapping::None,
            DebandDither::Disabled,
        ))
        .id();
    let context = app
        .world_mut()
        .entity_mut(camera)
        .get_mut::<EguiContext>()
        .ok_or("camera has no required egui context")?
        .get_mut()
        .clone();
    let renderer = if native_only {
        PreviewRenderer::Native(Box::new(NativeMinimap::load(&source, &context)?))
    } else {
        PreviewRenderer::Source(Box::new(SourcePanelRenderer::load(
            &source,
            "BattleMinimap",
            &context,
        )?))
    };
    app.insert_resource(Preview {
        renderer,
        bindings: SourceUiBindings::default(),
        native_state: None,
        zoom: DEFAULT_ZOOM,
        viewport: CanvasViewport {
            pixels: [1920, 1080],
            scale_factor: 1.0,
        },
        report: None,
        error: None,
    });
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut captures = Vec::new();
    let mut mode_reports = Vec::new();
    for mode in &modes {
        for pixels in [[1920, 1080], [1280, 720]] {
            // Bootstrap evidence: [1920,1080], reference PPU=100, with the native
            // AdjustResolution rule. These remain supplied preview canvas inputs.
            let scale = (pixels[0] as f32 / 1920.0).min(pixels[1] as f32 / 1080.0);
            {
                let mut preview = app.world_mut().resource_mut::<Preview>();
                preview.viewport = CanvasViewport {
                    pixels,
                    scale_factor: scale,
                };
                preview.bindings = mode.bindings.clone();
                preview.native_state = mode.native_state.clone();
                preview.zoom = mode.zoom;
                preview.report = None;
                preview.error = None;
            }
            app.world_mut().resource_mut::<Capture>().image = None;
            let mut image = Image::new_target_texture(
                pixels[0],
                pixels[1],
                TextureFormat::Rgba8UnormSrgb,
                None,
            );
            image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
            let target = app.world_mut().resource_mut::<Assets<Image>>().add(image);
            app.world_mut()
                .entity_mut(camera)
                .insert(RenderTarget::Image(target.clone().into()));
            let mut ready_frames = 0;
            while ready_frames < 3 {
                update(&mut app, deadline)?;
                ready_frames = if ready(&app) { ready_frames + 1 } else { 0 };
            }
            app.world_mut()
                .spawn(Screenshot::image(target))
                .observe(captured);
            while app.world().resource::<Capture>().image.is_none() {
                update(&mut app, deadline)?;
            }
            let image = app
                .world_mut()
                .resource_mut::<Capture>()
                .image
                .take()
                .ok_or("screenshot observer returned no minimap image")?
                .try_into_dynamic()?;
            let filename = format!("{}-{}x{}.png", mode.name, pixels[0], pixels[1]);
            image.save(directory.join(&filename))?;
            let rgba = image.to_rgba8();
            let colors = rgba
                .pixels()
                .map(|pixel| [pixel.0[0] >> 4, pixel.0[1] >> 4, pixel.0[2] >> 4])
                .collect::<BTreeSet<_>>();
            if colors.len() < 16 {
                return Err(
                    "minimap capture lacks enough color variation to verify textured graphics"
                        .into(),
                );
            }
            let crop_width = (440.0 * scale).ceil() as u32;
            let crop_height = (460.0 * scale).ceil() as u32;
            let crop_file = format!("{}-{}x{}-detail.png", mode.name, pixels[0], pixels[1]);
            image
                .crop_imm(pixels[0] - crop_width, 0, crop_width, crop_height)
                .save(directory.join(&crop_file))?;
            let preview = app.world().resource::<Preview>();
            let report = preview
                .report
                .as_ref()
                .ok_or("no source panel render report")?;
            if native_only
                && (report
                    .graphics
                    .iter()
                    .any(|graphic| graphic.status == "unsupported")
                    || !report.graphics.iter().any(|graphic| {
                        graphic.kind == "raw_image" && graphic.status == "submitted"
                    })
                    || !report.graphics.iter().any(|graphic| {
                        graphic.instance.as_deref() == Some("player-arrow")
                            && graphic.status == "submitted"
                    }))
            {
                return Err("native minimap did not submit its map/player or has unsupported active graphics".into());
            }
            let statuses = report.graphics.iter().fold(
                BTreeMap::<String, usize>::new(),
                |mut result, graphic| {
                    *result.entry(graphic.status.clone()).or_default() += 1;
                    result
                },
            );
            let report_file = format!("{}-{}x{}.json", mode.name, pixels[0], pixels[1]);
            // Keep the production report fields at the root for existing consumers.
            let mut graphic_report = serde_json::to_value(report)?;
            graphic_report["preview_mode"] = json!(mode.name);
            graphic_report["initialization_profile"] = mode.profile.clone();
            graphic_report["preview_caveats"] = json!(mode.caveats);
            serde_json::to_writer_pretty(
                std::fs::File::create(directory.join(&report_file))?,
                &graphic_report,
            )?;
            captures.push(json!({
                "file": filename,
                "report": report_file,
                "detail_file": crop_file,
                "detail_crop": [pixels[0] - crop_width, 0, crop_width, crop_height],
                "pixels": pixels,
                "scale_factor": scale,
                "preview_mode": mode.name,
                "initialization_profile": mode.profile.get("file"),
                "native_state": mode.native_state,
                "native_zoom": if native_only { Some(mode.zoom) } else { None },
                "bindings": report.bindings,
                "preview_caveats": mode.caveats,
                "graphic_statuses": statuses,
                "distinct_rgb_buckets": colors.len()
            }));
        }
        let report = app
            .world()
            .resource::<Preview>()
            .report
            .as_ref()
            .ok_or("no source panel render report for capture mode")?;
        mode_reports.push(json!({
            "name": mode.name,
            "initialization_profile": mode.profile,
            "native_state": mode.native_state,
            "native_zoom": if native_only { Some(mode.zoom) } else { None },
            "bindings": report.bindings,
            "caveats": mode.caveats
        }));
    }
    let adapter = app.world().resource::<RenderAdapterInfo>();
    let source_apk_sha256 = &app
        .world()
        .resource::<Preview>()
        .report
        .as_ref()
        .ok_or("no final render report")?
        .source_apk_sha256;
    let limitations = if native_only {
        vec![
            "Native Bevy/egui HUD captures with explicit simulation inputs; no APK runtime or pixel-equality claim.",
            "Map_map assignment, source image scale 260/2048, player-centered UV zoom and live marker categories/sizes are native policies. Original Unity runtime bindings remain unverified.",
            "The shared overview extent is provisional; map and marker calibration remain consistent with each other.",
            "Source font metrics/color/mip equivalence and custom Unity stencil/fog/particles/line rendering remain unverified.",
        ]
    } else {
        vec![
            "These are source panel previews rendered by Bevy/egui, not screenshots of the APK or proof of pixel equality.",
            "Runtime map texture is unbound. Visibility inputs and their static-evidence caveats are recorded separately for each preview mode.",
            "Constructor zoom 15 does not establish a RawImage UV setter, runtime texture or final image scale; no such binding is applied in these prefab modes.",
            "Full per-graphic reports identify unsupported materials, custom stencil/fog and line rendering, null sprite references and font/layout limits.",
        ]
    };
    let report = serde_json::json!({"source_apk_sha256":source_apk_sha256,
        "bevy_version":"0.20.1","bevy_egui_version":"0.43.1","egui_version":"0.36.2",
        "gpu":adapter.name,"driver":adapter.driver_info,"backend":format!("{:?}",adapter.backend),
        "parent_canvas_input":{"reference_resolution":[1920,1080],"reference_pixels_per_unit":100,"scale_rule":"min(width/1920,height/1080)","bootstrap_evidence":"../minimap-canvas-review.md","battle_minimap_runtime_parentage_verified":false},
        "native_hud_adapter":native_only,"waydroid_required":false,
        "native_map_texture":if native_only {Some(client_core::source_minimap::map_texture())} else {None},
        "native_overview_half_extent":if native_only {Some(shared::level::MAP_OVERVIEW_HALF_EXTENT)} else {None},
        "preview_modes":mode_reports,"captures":captures,"limitations":limitations});
    serde_json::to_writer_pretty(
        std::fs::File::create(directory.join("render.json"))?,
        &report,
    )?;
    println!(
        "Captured {} source BattleMinimap previews at two explicit canvas sizes: {}",
        captures.len(),
        directory.display()
    );
    Ok(())
}
