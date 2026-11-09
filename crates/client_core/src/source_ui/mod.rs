//! APK panel previews through the client's existing egui backend. Source DTOs
//! and layout are shared with the importer; runtime bindings are explicit input.
//! This is not yet a replacement for the game's dynamic HUD.

mod clip;

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};

use bevy_egui::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, FontId, Mesh, Painter, TextureHandle,
    TextureOptions, epaint::Vertex, pos2,
};
use midnight_map_data::{SourceObject, TextureSampler, read_rgba_mip_archive};
use midnight_ui_data::{
    ComponentProjection, GraphicData, HierarchyNode, ImageData, ReferenceState, SourceReference,
    TextData, UiError, UiPackage, UiPanel,
    layout::{CanvasViewport, ResolvedNode, UiRect, resolve_panel},
    load_ui_package,
    sprite::image_mesh,
};
use serde::Serialize;
use skrifa::MetadataProvider;

use clip::{MaskCoverage, clip_masks};

fn invalid(message: impl Into<String>) -> UiError {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()).into()
}

pub(super) struct TexturePixels {
    size: [u32; 2],
    rgba: Vec<u8>,
    sampler: TextureSampler,
}

impl TexturePixels {
    fn alpha(&self, uv: [f32; 2]) -> f32 {
        fn coordinate(index: i64, dimension: u32, wrap: i32) -> usize {
            let size = i64::from(dimension);
            let index = match wrap {
                0 => index.rem_euclid(size),
                2 => {
                    let index = index.rem_euclid(size * 2);
                    if index >= size {
                        size * 2 - 1 - index
                    } else {
                        index
                    }
                }
                _ => index.clamp(0, size - 1),
            };
            index as usize
        }
        let pixel = |x, y| {
            let x = coordinate(x, self.size[0], self.sampler.wrap[0]);
            let y = coordinate(y, self.size[1], self.sampler.wrap[1]);
            f32::from(self.rgba[(y * self.size[0] as usize + x) * 4 + 3]) / 255.0
        };
        let x = uv[0] * self.size[0] as f32;
        let y = uv[1] * self.size[1] as f32;
        if self.sampler.filter_mode == 0 {
            return pixel(x.floor() as i64, y.floor() as i64);
        }
        let x = x - 0.5;
        let y = y - 0.5;
        let ix = x.floor() as i64;
        let iy = y.floor() as i64;
        let tx = x - x.floor();
        let ty = y - y.floor();
        (pixel(ix, iy) * (1.0 - tx) + pixel(ix + 1, iy) * tx) * (1.0 - ty)
            + (pixel(ix, iy + 1) * (1.0 - tx) + pixel(ix + 1, iy + 1) * tx) * ty
    }
}

struct PreparedTexture {
    pixels: TexturePixels,
    handle: TextureHandle,
}

/// Caller-provided state, keyed by qualified source component/node identities.
/// Empty bindings mean the serialized prefab state, not the APK's battle state.
#[derive(Clone, Debug, Default)]
pub struct SourceUiBindings {
    /// Component -> source Texture2D. Null source RawImages stay explicitly unbound.
    pub raw_images: BTreeMap<SourceObject, SourceObject>,
    pub text: BTreeMap<SourceObject, String>,
    /// Node -> activeSelf override. Parent inactivity still takes precedence.
    pub visibility: BTreeMap<SourceObject, bool>,
    /// RectTransform -> anchoredPosition override in source canvas units.
    pub anchored_positions: BTreeMap<SourceObject, [f32; 2]>,
}

#[derive(Debug, Serialize)]
pub struct GraphicEvidence {
    pub component: SourceObject,
    pub path: String,
    pub kind: String,
    pub status: String,
    pub detail: Option<String>,
    pub triangles: usize,
    pub screen_corners: Option<[[f32; 2]; 4]>,
}

#[derive(Debug, Serialize)]
pub struct PanelRenderReport {
    pub panel: String,
    pub source_apk_sha256: String,
    pub layout_sha256: String,
    pub viewport: CanvasViewport,
    pub reference_pixels_per_unit: f32,
    pub source_nodes: usize,
    pub decoded_atlases: usize,
    pub embedded_fonts: usize,
    pub bound_components: usize,
    /// Sorted entry arrays preserve qualified IDs as JSON objects, not map keys.
    pub bindings: serde_json::Value,
    pub masks: usize,
    pub graphics: Vec<GraphicEvidence>,
    pub limitations: Vec<String>,
}

pub struct SourcePanelRenderer {
    pub panel: UiPanel,
    package: UiPackage,
    textures: BTreeMap<SourceObject, PreparedTexture>,
    fonts: BTreeMap<SourceObject, FontFamily>,
    font_characters: BTreeMap<SourceObject, BTreeSet<char>>,
    layout_sha256: String,
}

impl SourcePanelRenderer {
    /// Prepare verified full-atlas pixels and embedded source fonts once, before
    /// the context's first frame. No legacy name-based PNG lookup is involved.
    pub fn load(root: &Path, name: &str, context: &egui::Context) -> Result<Self, UiError> {
        let package = load_ui_package(root)?;
        let panel = package.load_panel(name)?;
        let layout_sha256 = package
            .manifest
            .panels
            .iter()
            .find(|summary| summary.name == name)
            .and_then(|summary| summary.layout.as_ref())
            .ok_or_else(|| invalid("panel summary is absent"))?
            .sha256
            .clone();
        let mut textures = BTreeSet::new();
        let mut font_text: BTreeMap<SourceObject, String> = BTreeMap::new();
        fn collect(
            node: &HierarchyNode,
            package: &UiPackage,
            textures: &mut BTreeSet<SourceObject>,
            fonts: &mut BTreeMap<SourceObject, String>,
        ) {
            for component in &node.components {
                match &component.projection {
                    ComponentProjection::Image(image) => {
                        if let Some(sprite) = image
                            .sprite
                            .target()
                            .and_then(|source| package.sprite(source))
                            && let Some(source) = sprite.render.texture.target()
                        {
                            textures.insert(source.clone());
                        }
                    }
                    ComponentProjection::RawImage(image) => {
                        if let Some(source) = image.texture.target() {
                            textures.insert(source.clone());
                        }
                    }
                    ComponentProjection::Text(text) => {
                        if let Some(source) = text.font.target() {
                            fonts
                                .entry(source.clone())
                                .or_default()
                                .push_str(&text.text);
                        }
                    }
                    _ => {}
                }
            }
            for child in &node.children {
                collect(child, package, textures, fonts);
            }
        }
        collect(&panel.root, &package, &mut textures, &mut font_text);
        let mut result = Self {
            panel,
            package,
            textures: BTreeMap::new(),
            fonts: BTreeMap::new(),
            font_characters: BTreeMap::new(),
            layout_sha256,
        };
        for source in textures {
            result.prepare_texture(&source, context)?;
        }
        let mut definitions = FontDefinitions::default();
        for (source, _) in font_text {
            let font = result
                .package
                .font(&source)
                .ok_or_else(|| invalid("panel Font is absent from package"))?;
            let Some(identity) = &font.embedded_font else {
                continue;
            };
            let bytes = result.package.read_verified(identity)?;
            let face = skrifa::FontRef::from_index(&bytes, 0)
                .map_err(|error| invalid(format!("{}: invalid source font: {error}", font.name)))?;
            let characters = face
                .charmap()
                .mappings()
                .filter_map(|(codepoint, _)| char::from_u32(codepoint))
                .collect();
            result.font_characters.insert(source.clone(), characters);
            let key = format!("apk:{}", identity.sha256);
            let family = FontFamily::Name(key.clone().into());
            definitions
                .font_data
                .insert(key.clone(), Arc::new(FontData::from_owned(bytes)));
            definitions.families.insert(family.clone(), vec![key]);
            result.fonts.insert(source, family);
        }
        context.set_fonts(definitions);
        Ok(result)
    }

    fn prepare_texture(
        &mut self,
        source: &SourceObject,
        context: &egui::Context,
    ) -> Result<(), UiError> {
        if self.textures.contains_key(source) {
            return Ok(());
        }
        let texture = self
            .package
            .texture(source)
            .ok_or_else(|| invalid("panel Texture2D is absent from package"))?;
        let identity = texture
            .gpu
            .as_ref()
            .ok_or_else(|| invalid("panel Texture2D has no decoded source mip archive"))?;
        let bytes = self.package.read_verified(identity)?;
        let chain = read_rgba_mip_archive(&bytes)?;
        if [chain.width, chain.height, chain.mip_count]
            != [texture.width, texture.height, texture.mip_count]
        {
            return Err(invalid("panel Texture2D source mip metadata differs"));
        }
        if !matches!(texture.sampler.filter_mode, 0..=2)
            || texture.sampler.wrap[0] != texture.sampler.wrap[1]
            || !matches!(texture.sampler.wrap[0], 0..=2)
            || !matches!(texture.unity_color_space, 0 | 1)
        {
            return Err(invalid(
                "egui panel preview cannot represent source sampler/color space",
            ));
        }
        let size = [chain.width, chain.height];
        let count = size[0] as usize * size[1] as usize * 4;
        let rgba = chain.pixels[..count].to_vec();
        let pixels = if texture.unity_color_space == 1 {
            egui::ColorImage::from_rgba_unmultiplied([size[0] as usize, size[1] as usize], &rgba)
        } else {
            egui::ColorImage::new(
                [size[0] as usize, size[1] as usize],
                rgba.as_chunks::<4>()
                    .0
                    .iter()
                    .map(|pixel| {
                        Color32::from(egui::Rgba::from_rgba_unmultiplied(
                            f32::from(pixel[0]) / 255.0,
                            f32::from(pixel[1]) / 255.0,
                            f32::from(pixel[2]) / 255.0,
                            f32::from(pixel[3]) / 255.0,
                        ))
                    })
                    .collect(),
            )
        };
        let filter = if texture.sampler.filter_mode == 0 {
            egui::TextureFilter::Nearest
        } else {
            egui::TextureFilter::Linear
        };
        let wrap_mode = match texture.sampler.wrap[0] {
            0 => egui::TextureWrapMode::Repeat,
            2 => egui::TextureWrapMode::MirroredRepeat,
            _ => egui::TextureWrapMode::ClampToEdge,
        };
        let handle = context.load_texture(
            format!("apk:{}", identity.sha256),
            pixels,
            TextureOptions {
                magnification: filter,
                minification: filter,
                wrap_mode,
                ..Default::default()
            },
        );
        self.textures.insert(
            source.clone(),
            PreparedTexture {
                pixels: TexturePixels {
                    size,
                    rgba,
                    sampler: texture.sampler.clone(),
                },
                handle,
            },
        );
        Ok(())
    }

    pub fn paint(
        &mut self,
        painter: &Painter,
        viewport: CanvasViewport,
        reference_pixels_per_unit: f32,
        bindings: &SourceUiBindings,
    ) -> Result<PanelRenderReport, UiError> {
        if u64::from(viewport.pixels[0]) * u64::from(viewport.pixels[1]) > 16_777_216 {
            return Err(invalid(
                "source panel preview exceeds its 16-million-pixel mask budget",
            ));
        }
        if !reference_pixels_per_unit.is_finite() || reference_pixels_per_unit <= 0.0 {
            return Err(invalid("canvas reference pixels-per-unit must be positive"));
        }
        validate_bindings(&self.panel.root, bindings)?;
        for source in bindings.raw_images.values() {
            self.prepare_texture(source, painter.ctx())?;
        }
        let mut panel = self.panel.clone();
        apply_bindings(&mut panel.root, bindings);
        let layouts = resolve_panel(&panel, viewport)?;
        let mut report = PanelRenderReport { panel: panel.name.clone(), source_apk_sha256: panel.source_apk_sha256.clone(), layout_sha256: self.layout_sha256.clone(), viewport,
            reference_pixels_per_unit, source_nodes: layouts.len(), decoded_atlases: self.textures.len(), embedded_fonts: self.fonts.len(),
            bound_components: bindings.raw_images.len() + bindings.text.len() + bindings.visibility.len() + bindings.anchored_positions.len(), bindings: binding_evidence(bindings), masks: 0, graphics: Vec::new(),
            limitations: vec![
                "The caller supplies the parent canvas. BattleMinimap runtime parentage/scaling is not established by this preview.".into(),
                "Empty bindings display serialized prefab state, including templates/placeholders; script-driven visibility, events, animation and gameplay state are not executed.".into(),
                "egui renders source full-atlas base pixels with its gamma-space tint and premultiplied blending; Unity shader/color and source mip-filter equivalence are unverified.".into(),
                "Standard texture masks use a binary alpha threshold of 0.001 at physical pixel centers. Custom stencil/fog materials and line renderers are explicitly omitted.".into(),
                "Source font outlines are used, with egui shaping, alignment, best-fit and line-spacing previews; Unity TextGenerator metrics, overflow and rasterization equivalence are unverified.".into(),
            ] };
        let mut index = 0;
        let mut meshes = Vec::new();
        self.paint_node(
            &panel.root,
            &layouts,
            &mut index,
            painter,
            viewport,
            reference_pixels_per_unit,
            bindings,
            &[],
            None,
            &mut report,
            &mut meshes,
        )?;
        // Glyph allocations can grow the font atlas during layout. Normalize all
        // text UVs only after the last allocation, including on the first frame.
        let atlas = painter.ctx().fonts_mut(|fonts| fonts.font_image_size());
        let pixels_per_point = painter.ctx().pixels_per_point();
        for (mut mesh, is_text) in meshes {
            for vertex in &mut mesh.vertices {
                vertex.pos = pos2(
                    vertex.pos.x / pixels_per_point,
                    vertex.pos.y / pixels_per_point,
                );
                if is_text {
                    vertex.uv = pos2(vertex.uv.x / atlas[0] as f32, vertex.uv.y / atlas[1] as f32);
                }
            }
            painter.add(egui::Shape::mesh(mesh));
        }
        Ok(report)
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_node(
        &self,
        node: &HierarchyNode,
        layouts: &[ResolvedNode],
        index: &mut usize,
        painter: &Painter,
        viewport: CanvasViewport,
        reference_ppu: f32,
        bindings: &SourceUiBindings,
        ancestors: &[&MaskCoverage],
        blocked: Option<&str>,
        report: &mut PanelRenderReport,
        meshes: &mut Vec<(Mesh, bool)>,
    ) -> Result<(), UiError> {
        let layout = &layouts[*index];
        *index += 1;
        let mut blocked = blocked.map(str::to_owned);
        let mut local_masks = Vec::new();
        let mut hidden_graphic = None;
        for component in node
            .components
            .iter()
            .filter(|component| component.enabled != Some(false))
        {
            if let ComponentProjection::Mask(fields) = &component.projection {
                let class = component
                    .script
                    .as_ref()
                    .map(|script| script.class_name.as_str())
                    .unwrap_or("");
                if class != "Mask" {
                    blocked = Some(format!(
                        "custom {class} requires source shader/runtime interpretation"
                    ));
                } else if layout.active {
                    let graphic =
                        node.components
                            .iter()
                            .find_map(|candidate| match &candidate.projection {
                                ComponentProjection::Image(image)
                                    if candidate.enabled != Some(false) =>
                                {
                                    Some((candidate, image))
                                }
                                _ => None,
                            });
                    match graphic.and_then(|(component, image)| {
                        self.sprite_mesh(image, layout, reference_ppu)
                            .ok()
                            .map(|(mesh, texture)| (component, image, mesh, texture))
                    }) {
                        Some((graphic, image, mesh, texture)) => {
                            local_masks.push(MaskCoverage::from_mesh(
                                &mesh,
                                &texture.pixels,
                                viewport.pixels,
                                image.graphic.color[3],
                            ));
                            report.masks += 1;
                            let show = fields
                                .get("m_ShowMaskGraphic")
                                .and_then(|value| value.as_bool())
                                .unwrap_or_else(|| {
                                    fields
                                        .get("m_ShowMaskGraphic")
                                        .and_then(|value| value.as_i64())
                                        .unwrap_or(0)
                                        != 0
                                });
                            if !show {
                                hidden_graphic = Some(&graphic.source);
                            }
                        }
                        None => {
                            blocked = Some("standard Mask graphic could not be represented".into())
                        }
                    }
                }
            }
            if let ComponentProjection::Canvas(fields) = &component.projection
                && fields
                    .get("m_OverrideSorting")
                    .and_then(|value| value.as_bool())
                    == Some(true)
            {
                blocked = Some(
                    "override-sorting Canvas requires a separate source draw-order pass".into(),
                );
            }
        }
        for component in &node.components {
            let (kind, graphic) = match &component.projection {
                ComponentProjection::Image(image) => ("image", Some(&image.graphic)),
                ComponentProjection::RawImage(image) => ("raw_image", Some(&image.graphic)),
                ComponentProjection::Text(text) => ("text", Some(&text.graphic)),
                ComponentProjection::RetainedSource
                    if component.script.as_ref().is_some_and(|script| {
                        matches!(script.class_name.as_str(), "DrawLine" | "UILineRenderer")
                    }) =>
                {
                    ("custom_graphic", None)
                }
                _ => continue,
            };
            let mut evidence = GraphicEvidence {
                component: component.source.clone(),
                path: layout.path.clone(),
                kind: kind.into(),
                status: String::new(),
                detail: None,
                triangles: 0,
                screen_corners: layout.screen_corners,
            };
            if !layout.active {
                evidence.status = "inactive".into();
            } else if component.enabled == Some(false) {
                evidence.status = "disabled".into();
            } else if let Some(reason) = &blocked {
                evidence.status = "unsupported".into();
                evidence.detail = Some(reason.clone());
            } else if hidden_graphic == Some(&component.source) {
                evidence.status = "mask_only".into();
            } else {
                let mesh = match &component.projection {
                    ComponentProjection::Image(image) => self
                        .sprite_mesh(image, layout, reference_ppu)
                        .map(|(mesh, _)| mesh),
                    ComponentProjection::RawImage(image) => {
                        let source = bindings
                            .raw_images
                            .get(&component.source)
                            .or_else(|| image.texture.target());
                        validate_material(&image.graphic.material).and_then(|()| match source.and_then(|source| self.textures.get(source)) {
                            Some(texture) => layout.local_rect.ok_or_else(|| invalid("RawImage has no RectTransform")).map(|rect| raw_mesh(rect, image.uv_rect, layout, texture.handle.id(), color(&image.graphic))),
                            None => Err(invalid("source RawImage texture is null/unbound or absent from prepared package")),
                        })
                    }
                    ComponentProjection::Text(text) => self.text_mesh(
                        text,
                        bindings.text.get(&component.source).map(String::as_str),
                        layout,
                        painter.ctx(),
                    ),
                    _ => Err(invalid("source custom line renderer is not implemented")),
                };
                match mesh {
                    Ok(mut mesh) => {
                        if graphic.is_some_and(|graphic| graphic.maskable) {
                            mesh = clip_masks(mesh, ancestors, viewport.pixels);
                        }
                        evidence.triangles = mesh.indices.len() / 3;
                        evidence.status = if mesh.indices.is_empty() {
                            "empty"
                        } else {
                            "submitted"
                        }
                        .into();
                        meshes.push((mesh, kind == "text"));
                    }
                    Err(error) => {
                        evidence.status = "unsupported".into();
                        evidence.detail = Some(error.to_string());
                    }
                }
            }
            report.graphics.push(evidence);
        }
        let mut descendants = ancestors.to_vec();
        descendants.extend(local_masks.iter());
        for child in &node.children {
            self.paint_node(
                child,
                layouts,
                index,
                painter,
                viewport,
                reference_ppu,
                bindings,
                &descendants,
                blocked.as_deref(),
                report,
                meshes,
            )?;
        }
        Ok(())
    }

    fn sprite_mesh(
        &self,
        image: &ImageData,
        layout: &ResolvedNode,
        reference_ppu: f32,
    ) -> Result<(Mesh, &PreparedTexture), UiError> {
        validate_material(&image.graphic.material)?;
        let sprite = image
            .sprite
            .target()
            .and_then(|source| self.package.sprite(source))
            .ok_or_else(|| invalid("source Image sprite is null/unresolved"))?;
        if !matches!(sprite.render.alpha_texture.resolution, ReferenceState::Null) {
            return Err(invalid("separate-alpha source sprite is not yet supported"));
        }
        let texture = sprite
            .render
            .texture
            .target()
            .and_then(|source| self.textures.get(source))
            .ok_or_else(|| invalid("source Sprite texture was not prepared"))?;
        let rect = layout
            .local_rect
            .ok_or_else(|| invalid("source Image has no RectTransform"))?;
        let source = image_mesh(image, sprite, rect, texture.pixels.size, reference_ppu)?;
        let mut mesh = Mesh::with_texture(texture.handle.id());
        mesh.vertices = source
            .vertices
            .into_iter()
            .map(|vertex| Vertex {
                pos: pos2_from(layout.to_screen(vertex.position)),
                uv: pos2_from(vertex.uv),
                color: color(&image.graphic),
            })
            .collect();
        mesh.indices = source.indices;
        Ok((mesh, texture))
    }

    fn text_mesh(
        &self,
        text: &TextData,
        bound_text: Option<&str>,
        layout: &ResolvedNode,
        context: &egui::Context,
    ) -> Result<Mesh, UiError> {
        validate_material(&text.graphic.material)?;
        let source = text
            .font
            .target()
            .ok_or_else(|| invalid("source Text font is unresolved"))?;
        let family = self
            .fonts
            .get(source)
            .ok_or_else(|| invalid("source Text font has no embedded outline"))?;
        let string = bound_text.unwrap_or(&text.text);
        if text.font_style != 0 || text.align_by_geometry || string.contains('<') && text.rich_text
        {
            return Err(invalid(
                "source styled/rich/geometry-aligned text needs additional interpretation",
            ));
        }
        let missing: BTreeSet<_> = string
            .chars()
            .filter(|ch| {
                !ch.is_whitespace()
                    && self
                        .font_characters
                        .get(source)
                        .is_none_or(|characters| !characters.contains(ch))
            })
            .collect();
        if !missing.is_empty() {
            return Err(invalid(format!(
                "source font lacks required mappings: {}",
                missing
                    .iter()
                    .map(|ch| format!("{ch} (U+{:04X})", *ch as u32))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        if !(0..=8).contains(&text.alignment)
            || !(0..=1).contains(&text.horizontal_overflow)
            || !(0..=1).contains(&text.vertical_overflow)
            || !text.line_spacing.is_finite()
            || text.line_spacing <= 0.0
        {
            return Err(invalid("unsupported source Text layout settings"));
        }
        let rect = layout
            .local_rect
            .ok_or_else(|| invalid("source Text has no RectTransform"))?;
        if rect.size.iter().any(|size| *size <= 0.0) {
            return Ok(Mesh::default());
        }
        let min = if text.best_fit {
            text.min_size
        } else {
            text.font_size
        };
        let max = if text.best_fit {
            text.max_size
        } else {
            text.font_size
        };
        if min <= 0 || max < min || max > 512 {
            return Err(invalid("invalid source Text font-size range"));
        }
        let wrap = if text.horizontal_overflow == 0 {
            rect.size[0]
        } else {
            f32::INFINITY
        };
        let halign = match text.alignment % 3 {
            0 => egui::Align::LEFT,
            1 => egui::Align::Center,
            _ => egui::Align::RIGHT,
        };
        let mut selected = None;
        for size in (min..=max).rev() {
            let mut job = egui::text::LayoutJob::simple(
                string.to_owned(),
                FontId::new(size as f32, family.clone()),
                color(&text.graphic),
                wrap,
            );
            job.halign = halign;
            // Use source line spacing with the selected font's own row metric.
            let row_height = context
                .fonts_mut(|fonts| fonts.row_height(&FontId::new(size as f32, family.clone())));
            job.sections[0].format.line_height = Some(row_height * text.line_spacing);
            let galley = context.fonts_mut(|fonts| fonts.layout_job(job));
            let fits = galley.rect.width() <= rect.size[0] + 0.001
                && galley.rect.height() <= rect.size[1] + 0.001;
            selected = Some(galley);
            if !text.best_fit || fits {
                break;
            }
        }
        let mut galley =
            selected.ok_or_else(|| invalid("source Text layout produced no galley"))?;
        if text.vertical_overflow == 0 {
            // Truncate limits generated rows. It does not install a rectangle
            // stencil over glyphs; source ancestor Masks provide pixel clipping.
            // This row budget uses egui metrics and remains a preview approximation.
            let mut job = (*galley.job).clone();
            let line_height = job.sections[0]
                .format
                .line_height
                .ok_or_else(|| invalid("source Text preview line metric is absent"))?;
            job.wrap.max_rows = (rect.size[1] / line_height).floor().max(1.0) as usize;
            job.wrap.overflow_character = None;
            galley = context.fonts_mut(|fonts| fonts.layout_job(job));
        }
        let x = rect.min[0]
            + rect.size[0]
                * match text.alignment % 3 {
                    0 => 0.0,
                    1 => 0.5,
                    _ => 1.0,
                };
        let y = rect.max()[1]
            - (rect.size[1] - galley.rect.height()) * (text.alignment / 3) as f32 / 2.0;
        let mut mesh = Mesh::default();
        for row in &galley.rows {
            let mut glyphs = row.visuals.mesh.clone();
            for vertex in &mut glyphs.vertices {
                vertex.pos = pos2_from(
                    layout.to_screen([x + row.pos.x + vertex.pos.x, y - row.pos.y - vertex.pos.y]),
                );
            }
            mesh.append(glyphs);
        }
        Ok(mesh)
    }
}

fn apply_bindings(node: &mut HierarchyNode, bindings: &SourceUiBindings) {
    if let Some(active) = bindings.visibility.get(&node.source) {
        node.active_self = *active;
    }
    if let Some(position) = bindings.anchored_positions.get(&node.transform.source)
        && let Some(rect) = &mut node.transform.rect
    {
        rect.anchored_position = *position;
    }
    for child in &mut node.children {
        apply_bindings(child, bindings);
    }
}

fn binding_evidence(bindings: &SourceUiBindings) -> serde_json::Value {
    serde_json::json!({
        "raw_images":bindings.raw_images.iter().map(|(component,texture)| serde_json::json!({"component":component,"texture":texture})).collect::<Vec<_>>(),
        "text":bindings.text.iter().map(|(component,text)| serde_json::json!({"component":component,"text":text})).collect::<Vec<_>>(),
        "visibility":bindings.visibility.iter().map(|(node,active_self)| serde_json::json!({"node":node,"active_self":active_self})).collect::<Vec<_>>(),
        "anchored_positions":bindings.anchored_positions.iter().map(|(transform,position)| serde_json::json!({"transform":transform,"position":position})).collect::<Vec<_>>()
    })
}

fn validate_bindings(root: &HierarchyNode, bindings: &SourceUiBindings) -> Result<(), UiError> {
    let mut nodes = BTreeSet::new();
    let mut rectangles = BTreeSet::new();
    let mut texts = BTreeSet::new();
    let mut raw_images = BTreeSet::new();
    fn collect(
        node: &HierarchyNode,
        nodes: &mut BTreeSet<SourceObject>,
        rectangles: &mut BTreeSet<SourceObject>,
        texts: &mut BTreeSet<SourceObject>,
        raw_images: &mut BTreeSet<SourceObject>,
    ) {
        nodes.insert(node.source.clone());
        if node.transform.rect.is_some() {
            rectangles.insert(node.transform.source.clone());
        }
        for component in &node.components {
            match &component.projection {
                ComponentProjection::Text(_) => {
                    texts.insert(component.source.clone());
                }
                ComponentProjection::RawImage(_) => {
                    raw_images.insert(component.source.clone());
                }
                _ => {}
            }
        }
        for child in &node.children {
            collect(child, nodes, rectangles, texts, raw_images);
        }
    }
    collect(
        root,
        &mut nodes,
        &mut rectangles,
        &mut texts,
        &mut raw_images,
    );
    if bindings.visibility.keys().any(|key| !nodes.contains(key))
        || bindings.text.keys().any(|key| !texts.contains(key))
        || bindings
            .raw_images
            .keys()
            .any(|key| !raw_images.contains(key))
        || bindings.anchored_positions.iter().any(|(key, position)| {
            !rectangles.contains(key) || position.iter().any(|value| !value.is_finite())
        })
    {
        return Err(invalid(
            "UI binding does not identify a matching source node/component or has a non-finite position",
        ));
    }
    Ok(())
}

fn validate_material(material: &SourceReference) -> Result<(), UiError> {
    if matches!(material.resolution, ReferenceState::Null) {
        Ok(())
    } else {
        Err(invalid(
            "source Graphic has a custom/unresolved material; default egui shading would change its semantics",
        ))
    }
}

fn pos2_from(point: [f32; 2]) -> egui::Pos2 {
    pos2(point[0], point[1])
}

fn color(graphic: &GraphicData) -> Color32 {
    let [r, g, b, a] = graphic
        .color
        .map(|value| (value.clamp(0.0, 1.0) * 255.0).round() as u8);
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn raw_mesh(
    rect: UiRect,
    uv: [f32; 4],
    layout: &ResolvedNode,
    texture: egui::TextureId,
    color: Color32,
) -> Mesh {
    let [u, v, width, height] = uv;
    let uvs = [
        [u, 1.0 - v],
        [u + width, 1.0 - v],
        [u + width, 1.0 - v - height],
        [u, 1.0 - v - height],
    ];
    let mut mesh = Mesh::with_texture(texture);
    mesh.vertices = rect
        .corners()
        .into_iter()
        .zip(uvs)
        .map(|(point, uv)| Vertex {
            pos: pos2_from(layout.to_screen(point)),
            uv: pos2_from(uv),
            color,
        })
        .collect();
    mesh.indices = vec![0, 1, 2, 2, 3, 0];
    mesh
}
