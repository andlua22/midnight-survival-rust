//! Source-bound UI data and native Bevy presentation.
mod native;
use midnight_map_data::SourceObject;
use midnight_ui_data::{
    ComponentProjection, GraphicData, HierarchyNode, ReferenceState, SourceReference, UiError,
    layout::CanvasViewport,
};
pub use native::{PanelFrame, PreparedGraphic, SourcePanelRenderer, SourceUiMaterial};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
fn invalid(message: impl Into<String>) -> UiError {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()).into()
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
    /// RectTransform -> (anchorMin, anchorMax), used by retained Unity sliders.
    pub rect_anchors: BTreeMap<SourceObject, [[f32; 2]; 2]>,
    /// RawImage component -> Unity bottom-left UV rect (x, y, width, height).
    pub uv_rects: BTreeMap<SourceObject, [f32; 4]>,
    pub local_scales: BTreeMap<SourceObject, [f32; 3]>,
    /// Transform -> source quaternion. Screen Y inversion is applied by layout.
    pub local_rotations: BTreeMap<SourceObject, [f32; 4]>,
    /// Image component -> verified source Sprite (including null templates).
    pub sprites: BTreeMap<SourceObject, SourceObject>,
    pub colors: BTreeMap<SourceObject, [f32; 4]>,
    pub fill_amounts: BTreeMap<SourceObject, f32>,
    /// Repeated source templates, retaining parent transforms, order and masks.
    pub instances: Vec<SourceUiInstance>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SourceUiInstance {
    pub key: String,
    /// Qualified GameObject identity, not a component or a bare object name.
    pub template: SourceObject,
    pub anchored_position: [f32; 2],
    pub local_scale: [f32; 3],
    pub local_rotation: [f32; 4],
    /// Optional tint for all graphics in this copy.
    pub color: Option<[f32; 4]>,
    /// Optional Sprite for the template root's single Image component.
    pub sprite: Option<SourceObject>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphicEvidence {
    pub component: SourceObject,
    pub path: String,
    pub kind: String,
    pub status: String,
    pub detail: Option<String>,
    pub triangles: usize,
    pub screen_corners: Option<[[f32; 2]; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
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

pub(crate) fn apply_bindings(node: &mut HierarchyNode, bindings: &SourceUiBindings) {
    if let Some(active) = bindings.visibility.get(&node.source) {
        node.active_self = *active;
    }
    if let Some(position) = bindings.anchored_positions.get(&node.transform.source)
        && let Some(rect) = &mut node.transform.rect
    {
        rect.anchored_position = *position;
    }
    if let Some([min, max]) = bindings.rect_anchors.get(&node.transform.source)
        && let Some(rect) = &mut node.transform.rect
    {
        rect.anchor_min = *min;
        rect.anchor_max = *max;
    }
    if let Some(scale) = bindings.local_scales.get(&node.transform.source) {
        node.transform.local_scale = *scale;
    }
    if let Some(rotation) = bindings.local_rotations.get(&node.transform.source) {
        node.transform.local_rotation = *rotation;
    }
    for component in &mut node.components {
        if let ComponentProjection::Image(image) = &mut component.projection
            && let Some(fill) = bindings.fill_amounts.get(&component.source)
        {
            image.fill_amount = *fill;
        }
        if let ComponentProjection::Image(image) = &mut component.projection
            && let Some(sprite) = bindings.sprites.get(&component.source)
        {
            image.sprite = runtime_reference(image.sprite.owner.clone(), sprite.clone(), 213);
        }
        if let ComponentProjection::RawImage(image) = &mut component.projection
            && let Some(uv) = bindings.uv_rects.get(&component.source)
        {
            image.uv_rect = *uv;
        }
        if let Some(color) = bindings.colors.get(&component.source)
            && let Some(graphic) = graphic_mut(&mut component.projection)
        {
            graphic.color = *color;
        }
    }
    for child in &mut node.children {
        apply_bindings(child, bindings);
    }
}

fn runtime_reference(owner: SourceObject, target: SourceObject, class_id: i32) -> SourceReference {
    SourceReference {
        owner,
        file_id: 0,
        path_id: target.path_id.clone(),
        resolution: ReferenceState::Resolved { target, class_id },
    }
}

fn graphic_mut(projection: &mut ComponentProjection) -> Option<&mut GraphicData> {
    match projection {
        ComponentProjection::Image(image) => Some(&mut image.graphic),
        ComponentProjection::RawImage(image) => Some(&mut image.graphic),
        ComponentProjection::Text(text) => Some(&mut text.graphic),
        _ => None,
    }
}

fn tint_subtree(node: &mut HierarchyNode, tint: Option<[f32; 4]>) {
    let Some(tint) = tint else {
        return;
    };
    for component in &mut node.components {
        if let Some(graphic) = graphic_mut(&mut component.projection) {
            graphic.color = std::array::from_fn(|axis| graphic.color[axis] * tint[axis]);
        }
    }
    for child in &mut node.children {
        tint_subtree(child, Some(tint));
    }
}

pub(crate) fn binding_evidence(bindings: &SourceUiBindings) -> serde_json::Value {
    serde_json::json!({
        "raw_images":bindings.raw_images.iter().map(|(component,texture)| serde_json::json!({"component":component,"texture":texture})).collect::<Vec<_>>(),
        "text":bindings.text.iter().map(|(component,text)| serde_json::json!({"component":component,"text":text})).collect::<Vec<_>>(),
        "visibility":bindings.visibility.iter().map(|(node,active_self)| serde_json::json!({"node":node,"active_self":active_self})).collect::<Vec<_>>(),
        "anchored_positions":bindings.anchored_positions.iter().map(|(transform,position)| serde_json::json!({"transform":transform,"position":position})).collect::<Vec<_>>(),
        "rect_anchors":bindings.rect_anchors.iter().map(|(transform,anchors)| serde_json::json!({"transform":transform,"anchors":anchors})).collect::<Vec<_>>(),
        "uv_rects":bindings.uv_rects.iter().map(|(component,uv_rect)| serde_json::json!({"component":component,"uv_rect":uv_rect})).collect::<Vec<_>>(),
        "local_scales":bindings.local_scales.iter().map(|(transform,scale)| serde_json::json!({"transform":transform,"scale":scale})).collect::<Vec<_>>(),
        "local_rotations":bindings.local_rotations.iter().map(|(transform,rotation)| serde_json::json!({"transform":transform,"rotation":rotation})).collect::<Vec<_>>(),
        "sprites":bindings.sprites.iter().map(|(component,sprite)| serde_json::json!({"component":component,"sprite":sprite})).collect::<Vec<_>>(),
        "colors":bindings.colors.iter().map(|(component,color)| serde_json::json!({"component":component,"color":color})).collect::<Vec<_>>(),
        "instances":bindings.instances,
        "fill_amounts":bindings.fill_amounts.iter().map(|(component,amount)| serde_json::json!({"component":component,"amount":amount})).collect::<Vec<_>>(),
    })
}

fn validate_bindings(root: &HierarchyNode, bindings: &SourceUiBindings) -> Result<(), UiError> {
    #[derive(Default)]
    struct Targets {
        nodes: BTreeSet<SourceObject>,
        transforms: BTreeSet<SourceObject>,
        rectangles: BTreeSet<SourceObject>,
        texts: BTreeSet<SourceObject>,
        raw_images: BTreeSet<SourceObject>,
        images: BTreeSet<SourceObject>,
        graphics: BTreeSet<SourceObject>,
    }
    fn collect(node: &HierarchyNode, targets: &mut Targets) {
        targets.nodes.insert(node.source.clone());
        targets.transforms.insert(node.transform.source.clone());
        if node.transform.rect.is_some() {
            targets.rectangles.insert(node.transform.source.clone());
        }
        for component in &node.components {
            match &component.projection {
                ComponentProjection::Text(_) => {
                    targets.texts.insert(component.source.clone());
                    targets.graphics.insert(component.source.clone());
                }
                ComponentProjection::RawImage(_) => {
                    targets.raw_images.insert(component.source.clone());
                    targets.graphics.insert(component.source.clone());
                }
                ComponentProjection::Image(_) => {
                    targets.images.insert(component.source.clone());
                    targets.graphics.insert(component.source.clone());
                }
                _ => {}
            }
        }
        for child in &node.children {
            collect(child, targets);
        }
    }
    let mut targets = Targets::default();
    collect(root, &mut targets);
    let finite = |values: &[f32]| values.iter().all(|value| value.is_finite());
    let rotation_valid = |values: &[f32; 4]| {
        finite(values) && values.iter().map(|value| value * value).sum::<f32>() > 1e-8
    };
    let color_valid = |values: &[f32; 4]| {
        values
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
    };
    if bindings.fill_amounts.iter().any(|(key, value)| {
        !targets.images.contains(key) || !value.is_finite() || !(0.0..=1.0).contains(value)
    }) || bindings
        .visibility
        .keys()
        .any(|key| !targets.nodes.contains(key))
        || bindings.text.keys().any(|key| !targets.texts.contains(key))
        || bindings
            .raw_images
            .keys()
            .any(|key| !targets.raw_images.contains(key))
        || bindings
            .sprites
            .keys()
            .any(|key| !targets.images.contains(key))
        || bindings
            .uv_rects
            .iter()
            .any(|(key, uv)| !targets.raw_images.contains(key) || !finite(uv))
        || bindings
            .colors
            .iter()
            .any(|(key, color)| !targets.graphics.contains(key) || !color_valid(color))
        || bindings
            .local_scales
            .iter()
            .any(|(key, scale)| !targets.transforms.contains(key) || !finite(scale))
        || bindings
            .local_rotations
            .iter()
            .any(|(key, rotation)| !targets.transforms.contains(key) || !rotation_valid(rotation))
        || bindings
            .anchored_positions
            .iter()
            .any(|(key, position)| !targets.rectangles.contains(key) || !finite(position))
        || bindings.rect_anchors.iter().any(|(key, anchors)| {
            !targets.rectangles.contains(key) || !finite(&anchors[0]) || !finite(&anchors[1])
        })
    {
        return Err(invalid(
            "UI binding has an incorrect source node/component type or invalid numeric value",
        ));
    }
    let mut keys = BTreeSet::new();
    for instance in &bindings.instances {
        let transform = find_node(root, &instance.template).map(|node| &node.transform.source);
        if instance.template == root.source
            || !targets.nodes.contains(&instance.template)
            || transform.is_none_or(|transform| !targets.rectangles.contains(transform))
            || instance.key.is_empty()
            || !keys.insert(&instance.key)
            || !finite(&instance.anchored_position)
            || !finite(&instance.local_scale)
            || !rotation_valid(&instance.local_rotation)
            || instance
                .color
                .as_ref()
                .is_some_and(|color| !color_valid(color))
        {
            return Err(invalid(
                "UI instance has an invalid template, duplicate/empty key or invalid transform/tint",
            ));
        }
    }
    Ok(())
}

pub(crate) fn find_node<'node>(
    root: &'node HierarchyNode,
    source: &SourceObject,
) -> Option<&'node HierarchyNode> {
    if &root.source == source {
        return Some(root);
    }
    root.children
        .iter()
        .find_map(|child| find_node(child, source))
}

fn validate_material(material: &SourceReference) -> Result<(), UiError> {
    if matches!(material.resolution, ReferenceState::Null) {
        Ok(())
    } else {
        Err(invalid(
            "source Graphic has a custom/unresolved material; default Bevy UI shading would change its semantics",
        ))
    }
}
