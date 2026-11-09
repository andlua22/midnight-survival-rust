//! Capture the source BattleMinimap with the production egui backend, without a
//! window. Run: cargo run --offline -p client_core --example minimap_probe -- DIR

use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
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
use midnight_ui_data::layout::CanvasViewport;

#[derive(Resource)]
struct Preview {
    renderer: SourcePanelRenderer,
    viewport: CanvasViewport,
    report: Option<PanelRenderReport>,
    error: Option<String>,
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
    match preview
        .renderer
        .paint(&painter, viewport, 100.0, &SourceUiBindings::default())
    {
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
    let directory = std::env::args_os().nth(1).map_or_else(
        || PathBuf::from("memory/evidence/minimap-render"),
        PathBuf::from,
    );
    std::fs::create_dir_all(&directory)?;
    let source = shared::level::asset_root()
        .join("imported_ui/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1");
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
    let renderer = SourcePanelRenderer::load(&source, "BattleMinimap", &context)?;
    app.insert_resource(Preview {
        renderer,
        viewport: CanvasViewport {
            pixels: [1920, 1080],
            scale_factor: 1.0,
        },
        report: None,
        error: None,
    });
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut captures = Vec::new();
    for pixels in [[1920, 1080], [1280, 720]] {
        // Bootstrap evidence: [1920,1080], reference PPU=100, with the native
        // AdjustResolution rule. These remain supplied preview canvas inputs.
        let scale = (pixels[0] as f32 / 1920.0).min(pixels[1] as f32 / 1080.0);
        app.world_mut().resource_mut::<Preview>().viewport = CanvasViewport {
            pixels,
            scale_factor: scale,
        };
        let mut image =
            Image::new_target_texture(pixels[0], pixels[1], TextureFormat::Rgba8UnormSrgb, None);
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
        let filename = format!("prefab-{}x{}.png", pixels[0], pixels[1]);
        image.save(directory.join(&filename))?;
        let rgba = image.to_rgba8();
        let colors = rgba
            .pixels()
            .map(|pixel| [pixel.0[0] >> 4, pixel.0[1] >> 4, pixel.0[2] >> 4])
            .collect::<BTreeSet<_>>();
        if colors.len() < 16 {
            return Err(
                "minimap capture lacks enough color variation to verify textured graphics".into(),
            );
        }
        let crop_width = (440.0 * scale).ceil() as u32;
        let crop_height = (460.0 * scale).ceil() as u32;
        let crop_file = format!("prefab-{}x{}-detail.png", pixels[0], pixels[1]);
        image
            .crop_imm(pixels[0] - crop_width, 0, crop_width, crop_height)
            .save(directory.join(&crop_file))?;
        let preview = app.world().resource::<Preview>();
        let report = preview
            .report
            .as_ref()
            .ok_or("no source panel render report")?;
        let statuses =
            report
                .graphics
                .iter()
                .fold(BTreeMap::<String, usize>::new(), |mut result, graphic| {
                    *result.entry(graphic.status.clone()).or_default() += 1;
                    result
                });
        let report_file = format!("prefab-{}x{}.json", pixels[0], pixels[1]);
        serde_json::to_writer_pretty(std::fs::File::create(directory.join(&report_file))?, report)?;
        captures.push(serde_json::json!({"file":filename,"report":report_file,"detail_file":crop_file,"detail_crop":[pixels[0]-crop_width,0,crop_width,crop_height],"pixels":pixels,"scale_factor":scale,"graphic_statuses":statuses,"distinct_rgb_buckets":colors.len()}));
    }
    let adapter = app.world().resource::<RenderAdapterInfo>();
    let report = serde_json::json!({"source_apk_sha256":app.world().resource::<Preview>().renderer.panel.source_apk_sha256,
        "bevy_version":"0.20.1","bevy_egui_version":"0.43.1","egui_version":"0.36.2",
        "gpu":adapter.name,"driver":adapter.driver_info,"backend":format!("{:?}",adapter.backend),
        "parent_canvas_input":{"reference_resolution":[1920,1080],"reference_pixels_per_unit":100,"scale_rule":"min(width/1920,height/1080)","bootstrap_evidence":"../minimap-canvas-review.md","battle_minimap_runtime_parentage_verified":false},
        "runtime_bindings":{},"captures":captures,
        "limitations":["These are source prefab previews rendered by Bevy/egui, not screenshots of the APK or proof of pixel equality.",
            "Runtime map texture and state are unbound; all serialized template nodes remain active unless their source components are disabled.",
            "Full per-graphic reports identify unsupported materials, custom stencil/fog and line rendering, null sprite references and font/layout limits."]});
    serde_json::to_writer_pretty(
        std::fs::File::create(directory.join("render.json"))?,
        &report,
    )?;
    println!(
        "Captured source BattleMinimap at two explicit canvas sizes: {}",
        directory.display()
    );
    Ok(())
}
