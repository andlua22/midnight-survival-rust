//! Source sprite geometry and stencil coverage supplied to Bevy UI materials.
//! Only geometry/coverage is rasterized here; source atlas sampling stays on GPU.

use super::{
    GraphicEvidence, PanelRenderReport, SourceUiBindings, apply_bindings, binding_evidence,
    invalid, runtime_reference, tint_subtree, validate_bindings, validate_material,
};
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{
        AsBindGroup, Extent3d, TextureDataOrder, TextureDimension, TextureFormat,
    },
    shader::ShaderRef,
};
use midnight_map_data::{SourceObject, TextureSampler, read_rgba_mip_archive};
use midnight_ui_data::{
    ComponentProjection, HierarchyNode, ImageData, ReferenceState, TextData, UiError, UiPackage,
    UiPanel,
    layout::{CanvasViewport, ResolvedNode, resolve_panel, resolve_subtree},
    load_ui_package,
    sprite::{SpriteMesh, SpriteVertex, image_mesh},
};
use std::{collections::BTreeMap, path::Path};

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct SourceUiMaterial {
    #[uniform(0)]
    pub tint: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub atlas: Handle<Image>,
    #[texture(3, sample_type = "float", filterable = false)]
    pub coordinates: Handle<Image>,
}
impl UiMaterial for SourceUiMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/source_ui.wesl".into()
    }
}

struct TexturePixels {
    size: [u32; 2],
    rgba: Vec<u8>,
    sampler: TextureSampler,
}
impl TexturePixels {
    fn alpha(&self, uv: [f32; 2]) -> f32 {
        let coordinate = |i: i64, d: u32, wrap: i32| -> usize {
            let d = i64::from(d);
            (match wrap {
                0 => i.rem_euclid(d),
                2 => {
                    let k = i.rem_euclid(d * 2);
                    if k >= d { d * 2 - 1 - k } else { k }
                }
                _ => i.clamp(0, d - 1),
            }) as usize
        };
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
    handle: Handle<Image>,
}

pub enum PreparedGraphic {
    Image {
        key: String,
        source: SourceObject,
        path: String,
        origin: Vec2,
        size: UVec2,
        atlas: Handle<Image>,
        coordinates: Vec<u8>,
        tint: Vec4,
    },
    Text {
        key: String,
        source: SourceObject,
        path: String,
        origin: Vec2,
        size: Vec2,
        text: String,
        font: Handle<Font>,
        font_size: f32,
        color: Color,
        alignment: i32,
        rotation: f32,
        clips: Option<Vec<Rect>>,
    },
}
pub struct PanelFrame {
    pub report: PanelRenderReport,
    pub graphics: Vec<PreparedGraphic>,
    pub layouts: Vec<ResolvedNode>,
}

pub struct SourcePanelRenderer {
    pub panel: UiPanel,
    package: UiPackage,
    textures: BTreeMap<SourceObject, PreparedTexture>,
    fonts: BTreeMap<SourceObject, Handle<Font>>,
    layout_sha256: String,
}
impl SourcePanelRenderer {
    pub fn package(&self) -> &UiPackage {
        &self.package
    }
    pub fn embedded_font(&self) -> Option<Handle<Font>> {
        self.fonts.values().next().cloned()
    }
    pub fn load(
        root: &Path,
        name: &str,
        images: &mut Assets<Image>,
        fonts: &mut Assets<Font>,
    ) -> Result<Self, UiError> {
        let package = load_ui_package(root)?;
        let panel = package.load_panel(name)?;
        let layout_sha256 = package
            .manifest
            .panels
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| p.layout.as_ref())
            .ok_or_else(|| invalid("source panel summary absent"))?
            .sha256
            .clone();
        let mut renderer = Self {
            panel,
            package,
            textures: BTreeMap::new(),
            fonts: BTreeMap::new(),
            layout_sha256,
        };
        let mut textures = Vec::new();
        let mut font_sources = Vec::new();
        fn collect(
            n: &HierarchyNode,
            package: &UiPackage,
            textures: &mut Vec<SourceObject>,
            fonts: &mut Vec<SourceObject>,
        ) {
            for c in &n.components {
                match &c.projection {
                    ComponentProjection::Image(i) => {
                        if let Some(t) = i
                            .sprite
                            .target()
                            .and_then(|s| package.sprite(s))
                            .and_then(|s| s.render.texture.target())
                        {
                            textures.push(t.clone())
                        }
                    }
                    ComponentProjection::RawImage(i) => {
                        if let Some(t) = i.texture.target() {
                            textures.push(t.clone())
                        }
                    }
                    ComponentProjection::Text(t) => {
                        if let Some(f) = t.font.target() {
                            fonts.push(f.clone())
                        }
                    }
                    _ => {}
                }
            }
            for c in &n.children {
                collect(c, package, textures, fonts)
            }
        }
        collect(
            &renderer.panel.root,
            &renderer.package,
            &mut textures,
            &mut font_sources,
        );
        textures.sort();
        textures.dedup();
        for source in textures {
            renderer.prepare_texture(&source, images)?;
        }
        font_sources.sort();
        font_sources.dedup();
        for source in font_sources {
            if let Some(identity) = renderer
                .package
                .font(&source)
                .and_then(|f| f.embedded_font.as_ref())
            {
                let bytes = renderer.package.read_verified(identity)?;
                skrifa::FontRef::from_index(&bytes, 0)
                    .map_err(|e| invalid(format!("invalid retained font: {e}")))?;
                renderer
                    .fonts
                    .insert(source, fonts.add(Font::from_bytes(bytes)));
            }
        }
        Ok(renderer)
    }
    fn prepare_texture(
        &mut self,
        source: &SourceObject,
        images: &mut Assets<Image>,
    ) -> Result<(), UiError> {
        if self.textures.contains_key(source) {
            return Ok(());
        }
        let texture = self
            .package
            .texture(source)
            .ok_or_else(|| invalid("source UI texture absent"))?;
        let identity = texture
            .gpu
            .as_ref()
            .ok_or_else(|| invalid("source UI texture has no mip payload"))?;
        let bytes = self.package.read_verified(identity)?;
        let chain = read_rgba_mip_archive(&bytes)?;
        if [chain.width, chain.height, chain.mip_count]
            != [texture.width, texture.height, texture.mip_count]
        {
            return Err(invalid("source UI mip metadata differs"));
        }
        if !matches!(texture.unity_color_space, 0 | 1)
            || !matches!(texture.sampler.filter_mode, 0..=2)
            || texture.sampler.wrap.iter().any(|w| !matches!(w, 0..=2))
        {
            return Err(invalid("unsupported source UI sampler/color space"));
        }
        let mut image = Image::new_uninit(
            Extent3d {
                width: chain.width,
                height: chain.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            if texture.unity_color_space == 1 {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            },
            RenderAssetUsages::default(),
        );
        image.texture_descriptor.mip_level_count = chain.mip_count;
        image.data = Some(chain.pixels.to_vec());
        image.data_order = TextureDataOrder::MipMajor;
        let wrap = |v| match v {
            0 => ImageAddressMode::Repeat,
            2 => ImageAddressMode::MirrorRepeat,
            _ => ImageAddressMode::ClampToEdge,
        };
        let filter = if texture.sampler.filter_mode == 0 {
            ImageFilterMode::Nearest
        } else {
            ImageFilterMode::Linear
        };
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: wrap(texture.sampler.wrap[0]),
            address_mode_v: wrap(texture.sampler.wrap[1]),
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: if texture.sampler.filter_mode == 2 {
                ImageFilterMode::Linear
            } else {
                ImageFilterMode::Nearest
            },
            ..default()
        });
        let count = chain.width as usize * chain.height as usize * 4;
        self.textures.insert(
            source.clone(),
            PreparedTexture {
                pixels: TexturePixels {
                    size: [chain.width, chain.height],
                    rgba: chain.pixels[..count].to_vec(),
                    sampler: texture.sampler.clone(),
                },
                handle: images.add(image),
            },
        );
        Ok(())
    }
    pub fn prepare_bindings(
        &mut self,
        bindings: &SourceUiBindings,
        images: &mut Assets<Image>,
    ) -> Result<(), UiError> {
        validate_bindings(&self.panel.root, bindings)?;
        for source in bindings.raw_images.values() {
            self.prepare_texture(source, images)?;
        }
        for source in bindings
            .sprites
            .values()
            .chain(bindings.instances.iter().filter_map(|i| i.sprite.as_ref()))
        {
            let texture = self
                .package
                .sprite(source)
                .and_then(|s| s.render.texture.target())
                .ok_or_else(|| invalid("runtime UI sprite absent"))?
                .clone();
            self.prepare_texture(&texture, images)?;
        }
        Ok(())
    }
    pub fn frame(
        &mut self,
        viewport: CanvasViewport,
        reference_ppu: f32,
        bindings: &SourceUiBindings,
        images: &mut Assets<Image>,
    ) -> Result<PanelFrame, UiError> {
        if !reference_ppu.is_finite() || reference_ppu <= 0.0 {
            return Err(invalid(
                "reference pixels per unit must be finite and positive",
            ));
        }
        if u64::from(viewport.pixels[0]) * u64::from(viewport.pixels[1]) > 16_777_216 {
            return Err(invalid(
                "UI viewport exceeds 16-million-pixel coverage budget",
            ));
        }
        self.prepare_bindings(bindings, images)?;
        let mut panel = self.panel.clone();
        apply_bindings(&mut panel.root, bindings);
        let layouts = resolve_panel(&panel, viewport)?;
        let mut report=PanelRenderReport{panel:panel.name.clone(),source_apk_sha256:panel.source_apk_sha256.clone(),layout_sha256:self.layout_sha256.clone(),viewport,reference_pixels_per_unit:reference_ppu,source_nodes:layouts.len(),decoded_atlases:self.textures.len(),embedded_fonts:self.fonts.len(),bound_components:bindings.raw_images.len()+bindings.text.len()+bindings.visibility.len()+bindings.anchored_positions.len()+bindings.uv_rects.len()+bindings.local_scales.len()+bindings.local_rotations.len()+bindings.sprites.len()+bindings.colors.len()+bindings.instances.len(),bindings:binding_evidence(bindings),masks:0,graphics:Vec::new(),limitations:vec!["Bevy UI samples retained source atlas mip chains. Unity shader keywords, tonemapping, font shaping and original runtime bindings are unobserved.".into(),"Texture stencil coverage is binary at physical pixel centers with the retained alpha threshold 0.001. Native text uses disjoint clip rectangles with a 4096-rectangle budget; custom materials/lines and override sorting remain unsupported.".into(),"Source font payloads use Bevy Text; Unity best-fit, rich-text, style and TextGenerator metrics require separate reconstruction.".into()]};
        let mut graphics = Vec::new();
        report.bound_components += bindings.fill_amounts.len() + bindings.rect_anchors.len();
        self.walk(
            &panel.root,
            &layouts,
            &mut 0,
            viewport,
            reference_ppu,
            bindings,
            &[],
            None,
            None,
            &mut report,
            &mut graphics,
        )?;
        Ok(PanelFrame {
            report,
            graphics,
            layouts,
        })
    }
    fn sprite_mesh(
        &self,
        image: &ImageData,
        layout: &ResolvedNode,
        ppu: f32,
    ) -> Result<(SpriteMesh, &PreparedTexture), UiError> {
        validate_material(&image.graphic.material)?;
        let sprite = image
            .sprite
            .target()
            .and_then(|s| self.package.sprite(s))
            .ok_or_else(|| invalid("source sprite null/unresolved"))?;
        if !matches!(sprite.render.alpha_texture.resolution, ReferenceState::Null) {
            return Err(invalid("separate-alpha sprite unsupported"));
        }
        let texture = sprite
            .render
            .texture
            .target()
            .and_then(|s| self.textures.get(s))
            .ok_or_else(|| invalid("source atlas unprepared"))?;
        let rect = layout
            .local_rect
            .ok_or_else(|| invalid("source Image has no RectTransform"))?;
        let mut mesh = image_mesh(image, sprite, rect, texture.pixels.size, ppu)?;
        for vertex in &mut mesh.vertices {
            vertex.position = layout.to_screen(vertex.position)
        }
        Ok((mesh, texture))
    }
    #[allow(clippy::too_many_arguments)]
    fn walk(
        &self,
        node: &HierarchyNode,
        layouts: &[ResolvedNode],
        index: &mut usize,
        viewport: CanvasViewport,
        ppu: f32,
        bindings: &SourceUiBindings,
        masks: &[&Coverage],
        blocked: Option<&str>,
        instance: Option<&str>,
        report: &mut PanelRenderReport,
        graphics: &mut Vec<PreparedGraphic>,
    ) -> Result<(), UiError> {
        let layout = &layouts[*index];
        *index += 1;
        let mut blocked = blocked.map(str::to_owned);
        let mut local_masks = Vec::new();
        let mut hidden = None;
        for c in node.components.iter().filter(|c| c.enabled != Some(false)) {
            match &c.projection {
                ComponentProjection::Mask(fields) => {
                    let class = c.script.as_ref().map_or("", |s| s.class_name.as_str());
                    if class == "RectMask2D" {
                        if layout.active
                            && let Some(corners) = layout.screen_corners
                        {
                            local_masks.push(Coverage::rectangle(corners, viewport.pixels));
                            report.masks += 1;
                        }
                    } else if class == "Mask" && layout.active {
                        let image = node.components.iter().find_map(|c| {
                            if let ComponentProjection::Image(i) = &c.projection {
                                Some((c, i))
                            } else {
                                None
                            }
                        });
                        if let Some((graphic, image)) = image {
                            match self.sprite_mesh(image, layout, ppu) {
                                Ok((mesh, texture)) => {
                                    local_masks.push(Coverage::texture(
                                        &mesh,
                                        &texture.pixels,
                                        viewport.pixels,
                                        image.graphic.color[3],
                                    ));
                                    report.masks += 1;
                                    if !fields
                                        .get("m_ShowMaskGraphic")
                                        .and_then(|v| {
                                            v.as_bool().or_else(|| v.as_i64().map(|n| n != 0))
                                        })
                                        .unwrap_or(false)
                                    {
                                        hidden = Some(&graphic.source)
                                    }
                                }
                                Err(e) => blocked = Some(e.to_string()),
                            }
                        } else {
                            blocked = Some("source mask graphic absent".into())
                        }
                    } else if class != "Mask" {
                        blocked = Some(format!(
                            "custom {class} requires source shader interpretation"
                        ))
                    }
                }
                ComponentProjection::Canvas(fields)
                    if fields.get("m_OverrideSorting").and_then(|v| v.as_bool()) == Some(true) =>
                {
                    blocked = Some("override-sorting Canvas unsupported".into())
                }
                _ => {}
            }
        }
        for c in &node.components {
            let kind = match &c.projection {
                ComponentProjection::Image(_) => "image",
                ComponentProjection::RawImage(_) => "raw_image",
                ComponentProjection::Text(_) => "text",
                ComponentProjection::RetainedSource
                    if c.script.as_ref().is_some_and(|s| {
                        matches!(s.class_name.as_str(), "DrawLine" | "UILineRenderer")
                    }) =>
                {
                    "custom_graphic"
                }
                _ => continue,
            };
            let mut evidence = GraphicEvidence {
                component: c.source.clone(),
                path: layout.path.clone(),
                kind: kind.into(),
                status: String::new(),
                detail: None,
                triangles: 0,
                screen_corners: layout.screen_corners,
                instance: instance.map(str::to_owned),
            };
            if !layout.active {
                evidence.status = "inactive".into()
            } else if c.enabled == Some(false) {
                evidence.status = "disabled".into()
            } else if let Some(reason) = &blocked {
                evidence.status = "unsupported".into();
                evidence.detail = Some(reason.clone())
            } else if hidden == Some(&c.source) {
                evidence.status = "mask_only".into()
            } else {
                let key = format!(
                    "{}:{}:{}",
                    c.source.bundle,
                    c.source.path_id,
                    instance.unwrap_or("")
                );
                let prepared: Result<Option<PreparedGraphic>, UiError> = match &c.projection {
                    ComponentProjection::Image(image) => self
                        .sprite_mesh(image, layout, ppu)
                        .and_then(|(mesh, texture)| {
                            evidence.triangles = mesh.indices.len() / 3;
                            prepare_image(
                                key,
                                c.source.clone(),
                                layout.path.clone(),
                                &mesh,
                                texture,
                                image.graphic.color,
                                if image.graphic.maskable { masks } else { &[] },
                                viewport.pixels,
                            )
                        }),
                    ComponentProjection::RawImage(image) => {
                        validate_material(&image.graphic.material).and_then(|()| {
                            let texture = bindings
                                .raw_images
                                .get(&c.source)
                                .or_else(|| image.texture.target())
                                .and_then(|s| self.textures.get(s))
                                .ok_or_else(|| invalid("RawImage null/unbound"))?;
                            let rect = layout
                                .local_rect
                                .ok_or_else(|| invalid("RawImage RectTransform absent"))?;
                            let [u, v, w, h] = image.uv_rect;
                            let uvs = [
                                [u, 1.0 - v],
                                [u + w, 1.0 - v],
                                [u + w, 1.0 - v - h],
                                [u, 1.0 - v - h],
                            ];
                            let mesh = SpriteMesh {
                                vertices: rect
                                    .corners()
                                    .into_iter()
                                    .zip(uvs)
                                    .map(|(p, uv)| SpriteVertex {
                                        position: layout.to_screen(p),
                                        uv,
                                    })
                                    .collect(),
                                indices: vec![0, 1, 2, 2, 3, 0],
                            };
                            evidence.triangles = 2;
                            prepare_image(
                                key,
                                c.source.clone(),
                                layout.path.clone(),
                                &mesh,
                                texture,
                                image.graphic.color,
                                if image.graphic.maskable { masks } else { &[] },
                                viewport.pixels,
                            )
                        })
                    }
                    ComponentProjection::Text(text) => self.prepare_text(
                        key,
                        c.source.clone(),
                        text,
                        bindings.text.get(&c.source),
                        layout,
                        masks,
                    ),
                    _ => Err(invalid("custom source line renderer unsupported")),
                };
                match prepared {
                    Ok(Some(graphic)) => {
                        evidence.status = "submitted".into();
                        if let PreparedGraphic::Text {
                            clips: Some(clips), ..
                        } = &graphic
                        {
                            evidence.detail = Some(format!(
                                "retained binary mask coverage uses {} disjoint native text clip rectangles",
                                clips.len()
                            ));
                        }
                        graphics.push(graphic)
                    }
                    Ok(None) => evidence.status = "empty".into(),
                    Err(e) => {
                        evidence.status = "unsupported".into();
                        evidence.detail = Some(e.to_string())
                    }
                }
            }
            report.graphics.push(evidence);
        }
        let mut descendants = masks.to_vec();
        descendants.extend(local_masks.iter());
        for child in &node.children {
            self.walk(
                child,
                layouts,
                index,
                viewport,
                ppu,
                bindings,
                &descendants,
                blocked.as_deref(),
                instance,
                report,
                graphics,
            )?
        }
        if instance.is_none() {
            for ins in bindings
                .instances
                .iter()
                .filter(|i| i.template == node.source)
            {
                let parent = layout
                    .parent
                    .and_then(|i| layouts.get(i))
                    .ok_or_else(|| invalid("instance parent absent"))?;
                let mut copy = node.clone();
                copy.active_self = true;
                copy.transform
                    .rect
                    .as_mut()
                    .ok_or_else(|| invalid("instance rect absent"))?
                    .anchored_position = ins.anchored_position;
                copy.transform.local_scale = ins.local_scale;
                copy.transform.local_rotation = ins.local_rotation;
                tint_subtree(&mut copy, ins.color);
                if let Some(sprite) = &ins.sprite {
                    for c in &mut copy.components {
                        if let ComponentProjection::Image(i) = &mut c.projection {
                            i.sprite =
                                runtime_reference(i.sprite.owner.clone(), sprite.clone(), 213)
                        }
                    }
                }
                let sub = resolve_subtree(&copy, parent)?;
                self.walk(
                    &copy,
                    &sub,
                    &mut 0,
                    viewport,
                    ppu,
                    bindings,
                    masks,
                    blocked.as_deref(),
                    Some(&ins.key),
                    report,
                    graphics,
                )?;
            }
        }
        Ok(())
    }
    fn prepare_text(
        &self,
        key: String,
        source: SourceObject,
        text: &TextData,
        binding: Option<&String>,
        layout: &ResolvedNode,
        masks: &[&Coverage],
    ) -> Result<Option<PreparedGraphic>, UiError> {
        validate_material(&text.graphic.material)?;
        let content = binding.unwrap_or(&text.text);
        if content.is_empty() || text.font_size <= 0 {
            return Ok(None);
        }
        let corners = layout
            .screen_corners
            .ok_or_else(|| invalid("source Text RectTransform absent"))?;
        let origin = Vec2::from_array(corners[3]);
        let x = Vec2::from_array(corners[2]) - origin;
        let y = Vec2::from_array(corners[0]) - origin;
        let local = layout
            .local_rect
            .ok_or_else(|| invalid("source Text local rect absent"))?;
        if x.length_squared() == 0.0 || y.length_squared() == 0.0 || local.size[1] == 0.0 {
            return Ok(None);
        }
        let scale = y.length() / local.size[1].abs();
        let font = text
            .font
            .target()
            .and_then(|s| self.fonts.get(s))
            .ok_or_else(|| invalid("source text has no retained embedded font"))?;
        let clips = if text.graphic.maskable {
            text_clip_rectangles(corners, masks)?
        } else {
            None
        };
        if clips.as_ref().is_some_and(Vec::is_empty) {
            return Ok(None);
        }
        Ok(Some(PreparedGraphic::Text {
            key,
            source,
            path: layout.path.clone(),
            origin,
            size: Vec2::new(x.length(), y.length()),
            text: content.clone(),
            font: font.clone(),
            font_size: text.font_size as f32 * scale,
            color: Color::srgba(
                text.graphic.color[0],
                text.graphic.color[1],
                text.graphic.color[2],
                text.graphic.color[3],
            ),
            alignment: text.alignment,
            rotation: x.y.atan2(x.x),
            clips,
        }))
    }
}

/// Disjoint native clip rectangles preserve the binary source stencil without
/// altering Bevy's text layout. Equal horizontal spans merge across adjacent rows.
fn text_clip_rectangles(
    corners: [[f32; 2]; 4],
    masks: &[&Coverage],
) -> Result<Option<Vec<Rect>>, UiError> {
    let Some(first) = masks.first() else {
        return Ok(None);
    };
    let mesh = SpriteMesh {
        vertices: corners
            .into_iter()
            .map(|position| SpriteVertex {
                position,
                uv: [0.0; 2],
            })
            .collect(),
        indices: vec![0, 1, 2, 2, 3, 0],
    };
    let [x0, y0, x1, y1] = bounds(&mesh, first.size);
    if u64::from(x1 - x0) * u64::from(y1 - y0) > 4_194_304 {
        return Err(invalid("source text coverage exceeds budget"));
    }
    let mut clips: Vec<Rect> = Vec::new();
    let mut previous: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    let mut count = 0_u64;
    for y in y0..y1 {
        let mut current = BTreeMap::new();
        let mut x = x0;
        while x < x1 {
            if !masks.iter().all(|mask| mask.contains(x, y)) {
                x += 1;
                continue;
            }
            let start = x;
            while x < x1 && masks.iter().all(|mask| mask.contains(x, y)) {
                x += 1;
            }
            count += u64::from(x - start);
            let span = (start, x);
            let index = if let Some(&index) = previous.get(&span) {
                clips[index].max.y = (y + 1) as f32;
                index
            } else {
                if clips.len() == 4096 {
                    return Err(invalid(
                        "source text mask exceeds 4096 native clip rectangles",
                    ));
                }
                clips.push(Rect::from_corners(
                    Vec2::new(start as f32, y as f32),
                    Vec2::new(x as f32, (y + 1) as f32),
                ));
                clips.len() - 1
            };
            current.insert(span, index);
        }
        previous = current;
    }
    if count == u64::from(x1 - x0) * u64::from(y1 - y0) && count != 0 {
        return Ok(None);
    }
    Ok(Some(clips))
}

struct Coverage {
    size: [u32; 2],
    pixels: Vec<bool>,
}
impl Coverage {
    fn contains(&self, x: u32, y: u32) -> bool {
        x < self.size[0] && y < self.size[1] && self.pixels[(y * self.size[0] + x) as usize]
    }
    fn texture(mesh: &SpriteMesh, texture: &TexturePixels, size: [u32; 2], alpha: f32) -> Self {
        let mut result = Self {
            size,
            pixels: vec![false; size[0] as usize * size[1] as usize],
        };
        visit(mesh, size, |x, y, uv| {
            if texture.alpha(uv) * alpha >= 0.001 {
                result.pixels[(y * size[0] + x) as usize] = true
            }
        });
        result
    }
    fn rectangle(corners: [[f32; 2]; 4], size: [u32; 2]) -> Self {
        let mesh = SpriteMesh {
            vertices: corners
                .into_iter()
                .map(|p| SpriteVertex {
                    position: p,
                    uv: [0.0; 2],
                })
                .collect(),
            indices: vec![0, 1, 2, 2, 3, 0],
        };
        let mut result = Self {
            size,
            pixels: vec![false; size[0] as usize * size[1] as usize],
        };
        visit(&mesh, size, |x, y, _| {
            result.pixels[(y * size[0] + x) as usize] = true
        });
        result
    }
}
fn bounds(mesh: &SpriteMesh, size: [u32; 2]) -> [u32; 4] {
    let mut lo = Vec2::splat(f32::INFINITY);
    let mut hi = Vec2::splat(f32::NEG_INFINITY);
    for v in &mesh.vertices {
        lo = lo.min(Vec2::from_array(v.position));
        hi = hi.max(Vec2::from_array(v.position))
    }
    [
        lo.x.floor().clamp(0.0, size[0] as f32) as u32,
        lo.y.floor().clamp(0.0, size[1] as f32) as u32,
        hi.x.ceil().clamp(0.0, size[0] as f32) as u32,
        hi.y.ceil().clamp(0.0, size[1] as f32) as u32,
    ]
}
fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}
fn visit(mesh: &SpriteMesh, size: [u32; 2], mut pixel: impl FnMut(u32, u32, [f32; 2])) {
    for t in mesh.indices.as_chunks::<3>().0 {
        let [a, b, c] = t.map(|i| mesh.vertices[i as usize]);
        let ap = Vec2::from_array(a.position);
        let bp = Vec2::from_array(b.position);
        let cp = Vec2::from_array(c.position);
        let area = cross(bp - ap, cp - ap);
        if area.abs() < 1e-8 {
            continue;
        }
        let tri = SpriteMesh {
            vertices: vec![a, b, c],
            indices: vec![],
        };
        let [x0, y0, x1, y1] = bounds(&tri, size);
        for y in y0..y1 {
            for x in x0..x1 {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let w = [
                    cross(bp - p, cp - p) / area,
                    cross(cp - p, ap - p) / area,
                    cross(ap - p, bp - p) / area,
                ];
                if w.iter().any(|v| *v < -1e-6) {
                    continue;
                }
                let uv = Vec2::from_array(a.uv) * w[0]
                    + Vec2::from_array(b.uv) * w[1]
                    + Vec2::from_array(c.uv) * w[2];
                pixel(x, y, uv.to_array())
            }
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn prepare_image(
    key: String,
    source: SourceObject,
    path: String,
    mesh: &SpriteMesh,
    texture: &PreparedTexture,
    color: [f32; 4],
    masks: &[&Coverage],
    viewport: [u32; 2],
) -> Result<Option<PreparedGraphic>, UiError> {
    if mesh.indices.is_empty() {
        return Ok(None);
    }
    let [x0, y0, x1, y1] = bounds(mesh, viewport);
    if x1 <= x0 || y1 <= y0 {
        return Ok(None);
    }
    let size = UVec2::new(x1 - x0, y1 - y0);
    let count = size.x as usize * size.y as usize;
    if count > 4_194_304 {
        return Err(invalid("source UI graphic exceeds coverage budget"));
    }
    let mut map = vec![[0.0f32; 4]; count];
    visit(mesh, viewport, |x, y, uv| {
        if masks.iter().all(|m| m.contains(x, y)) {
            map[((y - y0) * size.x + x - x0) as usize] = [uv[0], uv[1], 0.0, 1.0]
        }
    });
    let coordinates = map
        .iter()
        .flat_map(|v| v.iter().flat_map(|f| f.to_le_bytes()))
        .collect();
    let tint = Color::srgba(color[0], color[1], color[2], color[3])
        .to_linear()
        .to_vec4();
    Ok(Some(PreparedGraphic::Image {
        key,
        source,
        path,
        origin: Vec2::new(x0 as f32, y0 as f32),
        size,
        atlas: texture.handle.clone(),
        coordinates,
        tint,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn quad() -> SpriteMesh {
        SpriteMesh {
            vertices: vec![
                SpriteVertex {
                    position: [0.0, 0.0],
                    uv: [0.0, 0.0],
                },
                SpriteVertex {
                    position: [4.0, 0.0],
                    uv: [1.0, 0.0],
                },
                SpriteVertex {
                    position: [4.0, 4.0],
                    uv: [1.0, 1.0],
                },
                SpriteVertex {
                    position: [0.0, 4.0],
                    uv: [0.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2, 2, 3, 0],
        }
    }
    #[test]
    fn coverage_preserves_texture_holes_and_nested_masks() {
        let texture = TexturePixels {
            size: [2, 2],
            rgba: vec![
                255, 255, 255, 255, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 255,
            ],
            sampler: TextureSampler {
                filter_mode: 0,
                wrap: [1; 3],
                aniso: 1,
                mip_bias: 0.0,
            },
        };
        let mask = Coverage::texture(&quad(), &texture, [4, 4], 1.0);
        assert!(mask.contains(0, 0) && mask.contains(3, 3));
        assert!(!mask.contains(3, 0) && !mask.contains(0, 3));
        let nested = Coverage::rectangle([[0.0, 0.0], [2.0, 0.0], [2.0, 4.0], [0.0, 4.0]], [4, 4]);
        assert!(nested.contains(1, 1) && !nested.contains(3, 3));
        assert!(mask.contains(1, 1) && nested.contains(1, 1));
    }
    #[test]
    fn native_text_clips_preserve_holes_and_nested_intersections_without_overlap() {
        let corners = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
        let mut mask = Coverage::rectangle(corners, [4, 4]);
        mask.pixels[5] = false;
        let nested = Coverage::rectangle([[0.0, 0.0], [3.0, 0.0], [3.0, 4.0], [0.0, 4.0]], [4, 4]);
        let clips = text_clip_rectangles(corners, &[&mask, &nested])
            .expect("bounded native mask")
            .expect("partial coverage");
        for y in 0..4 {
            for x in 0..4 {
                let pixel = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let drawn = clips.iter().filter(|clip| clip.contains(pixel)).count();
                assert_eq!(
                    drawn,
                    usize::from(mask.contains(x, y) && nested.contains(x, y))
                );
            }
        }
        let rectangle = text_clip_rectangles(corners, &[&nested])
            .expect("rectangular mask")
            .expect("partial rectangle");
        assert_eq!(
            rectangle,
            vec![Rect::from_corners(Vec2::ZERO, Vec2::new(3.0, 4.0))]
        );
        assert!(
            text_clip_rectangles(corners, &[])
                .expect("no mask")
                .is_none()
        );
    }
    #[test]
    fn native_text_clip_fragmentation_is_bounded() {
        let width = 8194;
        let mask = Coverage {
            size: [width, 1],
            pixels: (0..width).map(|x| x % 2 == 0).collect(),
        };
        let corners = [
            [0.0, 0.0],
            [width as f32, 0.0],
            [width as f32, 1.0],
            [0.0, 1.0],
        ];
        assert!(text_clip_rectangles(corners, &[&mask]).is_err());
    }
    #[test]
    fn source_uv_interpolation_is_preserved() {
        let mut seen = Vec::new();
        visit(&quad(), [4, 4], |x, y, uv| seen.push((x, y, uv)));
        assert!(!seen.is_empty());
        for (x, y, uv) in seen {
            assert!((uv[0] - (x as f32 + 0.5) / 4.0).abs() < 1e-6);
            assert!((uv[1] - (y as f32 + 0.5) / 4.0).abs() < 1e-6);
        }
    }
}
