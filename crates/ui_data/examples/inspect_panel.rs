//! Bounded UI inspection; does not print the full source manifest.

use midnight_ui_data::{
    ComponentProjection,
    layout::{CanvasViewport, resolve_panel},
    load_ui_package,
};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut args = std::env::args().skip(1);
    let root = args.next().ok_or("provide the UI package path")?;
    let name = args.next().unwrap_or_else(|| "BattleMinimap".into());
    let mode = args.next();
    let summary_only = mode.as_deref() == Some("--summary");
    let package = load_ui_package(Path::new(&root))?;
    if mode.as_deref() == Some("--sprites") {
        let filter = args.next().unwrap_or_default();
        for sprite in &package.manifest.sprites {
            if sprite.name.contains(&filter) {
                serde_json::to_writer(
                    std::io::stdout().lock(),
                    &serde_json::json!({"name":sprite.name,"source":sprite.source}),
                )?;
                println!();
            }
        }
        return Ok(());
    }
    let panel = package.load_panel(&name)?;
    if mode.as_deref() == Some("--bindings") {
        fn bindings(
            node: &midnight_ui_data::HierarchyNode,
            package: &midnight_ui_data::UiPackage,
            prefix: &str,
        ) -> Result<(), serde_json::Error> {
            let path = if prefix.is_empty() {
                node.name.clone()
            } else {
                format!("{prefix}/{}", node.name)
            };
            let components: Vec<_> = node.components.iter().filter_map(|component| {
                let fields = match &component.projection {
                    ComponentProjection::Image(image) => serde_json::json!({"sprite": image.sprite.target().and_then(|source| package.sprite(source)).map(|sprite| (&sprite.name, &sprite.source)), "material":image.graphic.material.target(), "color":image.graphic.color}),
                    ComponentProjection::RawImage(image) => serde_json::json!({"texture":image.texture.target(),"uv":image.uv_rect,"material":image.graphic.material.target()}),
                    ComponentProjection::Text(text) => serde_json::json!({"text":text.text,"size":text.font_size,"material":text.graphic.material.target()}),
                    ComponentProjection::Mask(fields) => fields.clone(),
                    _ => return None,
                };
                Some(serde_json::json!({"id":component.source.path_id,"class":component.script.as_ref().map(|script| &script.class_name),"enabled":component.enabled,"fields":fields}))
            }).collect();
            serde_json::to_writer(
                std::io::stdout().lock(),
                &serde_json::json!({"path":path,"go":node.source.path_id,"transform":node.transform.source.path_id,"active":node.active_self,"rect":node.transform.rect,"scale":node.transform.local_scale,"rotation":node.transform.local_rotation,"components":components}),
            )?;
            println!();
            for child in &node.children {
                bindings(child, package, &path)?;
            }
            Ok(())
        }
        bindings(&panel.root, &package, "")?;
        return Ok(());
    }
    let layout = resolve_panel(
        &panel,
        CanvasViewport {
            pixels: [1280, 720],
            scale_factor: 1.0,
        },
    )?;
    fn nodes(
        node: &midnight_ui_data::HierarchyNode,
        package: &midnight_ui_data::UiPackage,
        result: &mut Vec<serde_json::Value>,
    ) {
        for component in &node.components {
            let value = match &component.projection {
                ComponentProjection::Image(image) => {
                    serde_json::json!({"kind":"image", "fields":image, "sprite":image.sprite.target().and_then(|source| package.sprite(source)).map(|sprite| serde_json::json!({"source":sprite.source,"name":sprite.name,"rect":sprite.rect,"border":sprite.border,"render":sprite.render}))})
                }
                ComponentProjection::Text(text) => {
                    serde_json::json!({"kind":"text", "fields":text, "font":text.font.target().and_then(|source| package.font(source))})
                }
                ComponentProjection::RawImage(image) => {
                    serde_json::json!({"kind":"raw_image", "texture":image.texture.target(), "uv_rect":image.uv_rect})
                }
                ComponentProjection::Mask(mask) => {
                    serde_json::json!({"kind":"mask", "fields":mask})
                }
                ComponentProjection::Canvas(canvas) => {
                    serde_json::json!({"kind":"canvas", "fields":canvas})
                }
                _ => continue,
            };
            result.push(serde_json::json!({"node":node.source,"name":node.name,"component":component.source,"script":component.script,"enabled":component.enabled,"projection":value}));
        }
        for child in &node.children {
            nodes(child, package, result);
        }
    }
    let mut graphics = Vec::new();
    nodes(&panel.root, &package, &mut graphics);
    if summary_only {
        graphics.retain(|entry| {
            matches!(
                entry["projection"]["kind"].as_str(),
                Some("text" | "mask" | "canvas" | "raw_image")
            ) || entry["name"] == "ditu"
        });
    }
    let result = serde_json::json!({"source_apk_sha256":panel.source_apk_sha256, "name":panel.name, "stats":panel.stats,
        "preview_canvas":{"pixels":[1280,720],"scale_factor":1.0,"apk_reference_resolution_verified":false}, "layout":if summary_only { layout.into_iter().filter(|node| node.parent.is_none() || node.path.ends_with("/ditu") || node.path.ends_with("/m_lefttop")).collect::<Vec<_>>() } else { layout },"graphics":graphics});
    serde_json::to_writer_pretty(std::io::stdout().lock(), &result)?;
    Ok(())
}
