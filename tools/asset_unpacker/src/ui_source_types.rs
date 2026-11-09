//! Versioned, source-bound UI data. Runtime equivalence is a separate verification step.

use std::collections::BTreeMap;

use midnight_map_data::{FileIdentity, SourceApk, SourceObject, TextureAsset};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceReference {
    pub owner: SourceObject,
    pub file_id: i64,
    /// Always a string, including zero and small built-in IDs.
    pub path_id: String,
    pub resolution: ReferenceState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ReferenceState {
    Null,
    Resolved { target: SourceObject, class_id: i32 },
    Unresolved { reason: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocatedReference {
    pub field_path: String,
    pub reference: SourceReference,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectRecord {
    pub source: SourceObject,
    pub class_id: i32,
    pub unity_version: String,
    pub serialized_byte_count: u64,
    pub serialized: FileIdentity,
    pub source_fields: FileIdentity,
    pub references: Vec<LocatedReference>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CabEvidence {
    pub cab: String,
    pub unity_version: String,
    pub type_tree_enabled: bool,
    pub object_count: usize,
    pub externals: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleEvidence {
    pub member: String,
    pub sha256: String,
    pub byte_count: u64,
    pub dependencies: Vec<String>,
    pub cabs: Vec<CabEvidence>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RectTransformData {
    pub anchor_min: [f32; 2],
    pub anchor_max: [f32; 2],
    pub anchored_position: [f32; 2],
    pub size_delta: [f32; 2],
    pub pivot: [f32; 2],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformData {
    pub source: SourceObject,
    pub local_position: [f32; 3],
    /// Original quaternion, without normalization or Euler conversion.
    pub local_rotation: [f32; 4],
    pub local_scale: [f32; 3],
    pub parent: SourceReference,
    pub ordered_children: Vec<SourceReference>,
    pub root_order: Option<i64>,
    pub local_euler_angles_hint: Option<[f32; 3]>,
    pub rect: Option<RectTransformData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphicData {
    pub color: [f32; 4],
    pub material: SourceReference,
    pub raycast_target: bool,
    pub raycast_padding: [f32; 4],
    pub maskable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageData {
    pub graphic: GraphicData,
    pub sprite: SourceReference,
    pub image_type: i32,
    pub preserve_aspect: bool,
    pub fill_center: bool,
    pub fill_method: i32,
    pub fill_origin: i32,
    pub fill_clockwise: bool,
    /// Zero is an actual source value, never replaced by a default.
    pub fill_amount: f32,
    pub use_sprite_mesh: bool,
    pub pixels_per_unit_multiplier: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextData {
    pub graphic: GraphicData,
    pub text: String,
    pub font: SourceReference,
    pub font_size: i32,
    pub font_style: i32,
    pub alignment: i32,
    pub align_by_geometry: bool,
    pub line_spacing: f32,
    pub rich_text: bool,
    pub best_fit: bool,
    pub min_size: i32,
    pub max_size: i32,
    pub horizontal_overflow: i32,
    pub vertical_overflow: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawImageData {
    pub graphic: GraphicData,
    pub texture: SourceReference,
    pub uv_rect: [f32; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptIdentity {
    pub source: SourceObject,
    pub class_name: String,
    pub namespace: String,
    pub assembly_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum ComponentProjection {
    Transform,
    Image(ImageData),
    Text(TextData),
    RawImage(RawImageData),
    /// Complete source fields, including camera/sorting/scaling settings.
    Canvas(Value),
    CanvasScaler(Value),
    /// Includes enabled state, masking parameters and custom mask references.
    Mask(Value),
    /// Complete selectable/event fields; this does not infer runtime listeners.
    Selectable(Value),
    /// Full properties and original bytes remain available in the object record.
    RetainedSource,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiComponent {
    /// The original ordered GameObject component PPtr.
    pub reference: SourceReference,
    pub source: SourceObject,
    pub class_id: i32,
    pub enabled: Option<bool>,
    pub script: Option<ScriptIdentity>,
    pub projection: ComponentProjection,
    pub source_fields: FileIdentity,
    pub serialized: FileIdentity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HierarchyNode {
    pub source: SourceObject,
    pub name: String,
    pub active_self: bool,
    pub active_in_prefab_hierarchy: bool,
    pub layer: i32,
    pub tag: Option<i64>,
    pub transform: TransformData,
    pub components: Vec<UiComponent>,
    /// Unity Transform.m_Children order, not object-table or alphabetical order.
    pub children: Vec<HierarchyNode>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PanelStats {
    pub nodes: usize,
    pub rect_transforms: usize,
    pub ordinary_transforms: usize,
    pub components: usize,
    pub image_components: usize,
    pub raw_image_components: usize,
    pub text_components: usize,
    pub canvases: usize,
    pub canvas_scalers: usize,
    pub masks: usize,
    pub selectables: usize,
    pub disabled_components: usize,
    pub inactive_nodes: usize,
    pub zero_fill_images: usize,
    pub non_identity_rotations: usize,
    pub script_types: BTreeMap<String, usize>,
    pub retained_component_types: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiPanel {
    pub schema_version: u32,
    pub source_apk_sha256: String,
    pub name: String,
    pub bundle: String,
    pub prefab_asset_path: String,
    pub prefab_reference: SourceReference,
    pub stats: PanelStats,
    pub root: HierarchyNode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PanelSummary {
    pub name: String,
    pub bundle: String,
    pub layout: Option<FileIdentity>,
    pub hierarchy_complete: bool,
    pub stats: Option<PanelStats>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpriteRenderData {
    pub owner: SourceObject,
    pub atlas: Option<SourceObject>,
    pub render_data_key: Value,
    pub texture: SourceReference,
    pub alpha_texture: SourceReference,
    /// Unity pixel coordinates, with a bottom-left texture origin.
    pub texture_rect: [f32; 4],
    pub texture_rect_offset: [f32; 2],
    pub atlas_rect_offset: [f32; 2],
    pub uv_transform: [f32; 4],
    pub downscale_multiplier: f32,
    pub settings_raw: u32,
    pub packed: bool,
    pub packing_mode: u32,
    pub packing_rotation: u32,
    pub mesh_type: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpritePreview {
    pub file: FileIdentity,
    pub width: u32,
    pub height: u32,
    pub texture_crop_pixels: [u32; 4],
    pub fractional_crop_bounds: bool,
    pub packing_rotation_applied: u32,
    pub separate_alpha_red_channel_applied: bool,
    pub source_mesh_applied: bool,
    pub original_canvas_restored: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiSprite {
    pub source: SourceObject,
    pub name: String,
    pub rect: [f32; 4],
    pub pivot: [f32; 2],
    /// Left, bottom, right, top, in original sprite pixels.
    pub border: [f32; 4],
    pub pixels_per_unit: f32,
    pub is_polygon: bool,
    pub render: SpriteRenderData,
    pub source_fields: FileIdentity,
    pub preview: Option<SpritePreview>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FontAsset {
    pub source: SourceObject,
    pub name: String,
    pub embedded_font: Option<FileIdentity>,
    pub source_fields: FileIdentity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportIssue {
    pub severity: String,
    pub code: String,
    pub source: Option<SourceObject>,
    pub field_path: Option<String>,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiManifest {
    pub schema_version: u32,
    pub source_apk: SourceApk,
    pub dependency_index: FileIdentity,
    pub bundles: Vec<BundleEvidence>,
    pub panels: Vec<PanelSummary>,
    pub sprites: Vec<UiSprite>,
    pub textures: Vec<TextureAsset>,
    pub fonts: Vec<FontAsset>,
    pub source_objects: Vec<ObjectRecord>,
    pub hierarchy_complete: bool,
    pub graphics_complete: bool,
    pub reference_coverage_complete: bool,
    pub runtime_equivalence_verified: bool,
    pub issues: Vec<ImportIssue>,
    pub limitations: Vec<String>,
    pub source_field_encoding: Vec<String>,
}
