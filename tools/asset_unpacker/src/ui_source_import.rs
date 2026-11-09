//! Extract selected prefab hierarchies and their graphics directly from a pinned APK.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet, VecDeque},
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use image::RgbaImage;
use midnight_map_data::{FileIdentity, TextureAsset};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use unity_asset_binary::{bundle::load_bundle_from_memory, object::UnityObject};
use unity_asset_core::UnityValue;

use crate::{
    fields::{array, bytes, field, integer, invalid, number, property, text, vector},
    pass1_index::GlobalIndex,
    resolver::{ObjectId, Resolver},
    unity::{Error, cab_name, read_object, source_apk},
};

use super::{sprite, types::*};

const PANELS: [(&str, &str, &str); 3] = [
    (
        "BattleMinimap",
        "assets/ui/battleminimap.ab",
        "battle_minimap",
    ),
    ("BattlePanel", "assets/ui/battlepanel.ab", "battle_panel"),
    (
        "BattleMapPanel",
        "assets/ui/battlemappanel.ab",
        "battle_map_panel",
    ),
];

struct Discovery {
    index: GlobalIndex,
    classes: BTreeMap<ObjectId, i32>,
    bundles: Vec<BundleEvidence>,
}

/// Only the selected prefabs and their declared dependency closure are indexed.
/// The legacy global index has no APK identity and is never an input to this pass.
fn discover(apk: &Path) -> Result<Discovery, Error> {
    let mut archive = zip::ZipArchive::new(File::open(apk)?)?;
    let members: HashSet<String> = (0..archive.len())
        .map(|index| Ok(archive.by_index_raw(index)?.name().to_owned()))
        .collect::<Result<_, zip::result::ZipError>>()?;
    let mut queue: VecDeque<String> = PANELS
        .iter()
        .map(|(_, member, _)| (*member).into())
        .collect();
    let mut seen = BTreeSet::new();
    let mut index = GlobalIndex::default();
    let mut classes = BTreeMap::new();
    let mut evidence = BTreeMap::new();
    while let Some(member) = queue.pop_front() {
        if !seen.insert(member.clone()) {
            continue;
        }
        if !members.contains(&member) {
            return Err(invalid(format!(
                "APK is missing declared UI bundle {member}"
            )));
        }
        let mut data = Vec::new();
        archive.by_name(&member)?.read_to_end(&mut data)?;
        let hash = format!("{:x}", Sha256::digest(&data));
        let byte_count = data.len() as u64;
        let bundle = load_bundle_from_memory(data)?;
        if bundle.assets.len() != bundle.asset_names.len() || bundle.assets.is_empty() {
            return Err(invalid(format!(
                "{member}: malformed serialized file inventory"
            )));
        }
        let mut dependencies = BTreeSet::new();
        let mut cabs = Vec::new();
        for (asset_name, asset) in bundle.asset_names.iter().zip(&bundle.assets) {
            let cab = cab_name(asset_name).to_owned();
            if let Some(previous) = index.cabs.insert(cab.clone(), member.clone()) {
                if previous != member {
                    return Err(invalid(format!(
                        "CAB {cab} appears in both {previous} and {member}"
                    )));
                }
            }
            for info in &asset.objects {
                let id = ObjectId {
                    bundle: member.clone(),
                    cab: cab.clone(),
                    path_id: info.path_id,
                };
                if classes.insert(id, info.type_id).is_some() {
                    return Err(invalid(format!(
                        "{member}:{cab}: duplicate object ID {}",
                        info.path_id
                    )));
                }
                if info.type_id == 142 {
                    let object = read_object(asset, info)?;
                    for dependency in array(property(&object, "m_Dependencies")?)? {
                        let dependency = text(dependency)?;
                        let dependency = if dependency.starts_with("assets/") {
                            dependency.to_owned()
                        } else {
                            format!("assets/{dependency}")
                        };
                        dependencies.insert(dependency.clone());
                        queue.push_back(dependency);
                    }
                }
            }
            cabs.push(CabEvidence {
                cab,
                unity_version: asset.unity_version.clone(),
                type_tree_enabled: asset.enable_type_tree,
                object_count: asset.objects.len(),
                externals: serde_json::to_value(&asset.externals)?,
            });
        }
        evidence.insert(
            member.clone(),
            BundleEvidence {
                member,
                sha256: hash,
                byte_count,
                dependencies: dependencies.into_iter().collect(),
                cabs,
            },
        );
    }
    Ok(Discovery {
        index,
        classes,
        bundles: evidence.into_values().collect(),
    })
}

/// Unity serializes many UI flags as UInt8, while GameObject uses a boolean.
/// Other integers are rejected instead of being truthy/defaulted.
pub(super) fn flag(value: &UnityValue) -> Result<bool, Error> {
    match value {
        UnityValue::Bool(value) => Ok(*value),
        UnityValue::Integer(0) => Ok(false),
        UnityValue::Integer(1) => Ok(true),
        _ => Err(invalid("expected source boolean or 0/1 flag")),
    }
}

fn int32(value: &UnityValue) -> Result<i32, Error> {
    Ok(i32::try_from(integer(value)?)?)
}

fn image_fill_amount(object: &UnityObject) -> Result<f32, Error> {
    let fill = number(property(object, "m_FillAmount")?)?;
    if !(0.0..=1.0).contains(&fill) {
        return Err(invalid("source Image fill amount is outside 0..1"));
    }
    Ok(fill)
}

pub(super) fn rect(value: &UnityValue) -> Result<[f32; 4], Error> {
    vector(value, ["x", "y", "width", "height"])
}

pub(super) fn reference_id(reference: &SourceReference) -> Result<ObjectId, Error> {
    match &reference.resolution {
        ReferenceState::Resolved { target, .. } => Ok(ObjectId {
            bundle: target.bundle.clone(),
            cab: target.cab.clone(),
            path_id: target.path_id.parse()?,
        }),
        ReferenceState::Null => Err(invalid("unexpected null source reference")),
        ReferenceState::Unresolved { reason } => Err(invalid(reason.clone())),
    }
}

fn enter_hierarchy(
    id: &ObjectId,
    ancestors: &mut Vec<ObjectId>,
    visited: &mut BTreeSet<ObjectId>,
) -> Result<(), Error> {
    if ancestors.len() >= 128 || ancestors.contains(id) {
        return Err(invalid("UI hierarchy cycle/depth limit"));
    }
    if !visited.insert(id.clone()) {
        return Err(invalid(
            "UI GameObject occurs more than once in the prefab hierarchy",
        ));
    }
    ancestors.push(id.clone());
    Ok(())
}

struct TransformScalars {
    local_position: [f32; 3],
    local_rotation: [f32; 4],
    local_scale: [f32; 3],
    root_order: Option<i64>,
    local_euler_angles_hint: Option<[f32; 3]>,
    rect: Option<RectTransformData>,
}

fn transform_scalars(object: &UnityObject) -> Result<TransformScalars, Error> {
    let local_rotation = vector(property(object, "m_LocalRotation")?, ["x", "y", "z", "w"])?;
    if local_rotation
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        < 1e-8
    {
        return Err(invalid("UI source transform has a zero quaternion"));
    }
    let properties = object.class.properties();
    Ok(TransformScalars {
        local_position: vector(property(object, "m_LocalPosition")?, ["x", "y", "z"])?,
        local_rotation,
        local_scale: vector(property(object, "m_LocalScale")?, ["x", "y", "z"])?,
        root_order: properties.get("m_RootOrder").map(integer).transpose()?,
        local_euler_angles_hint: properties
            .get("m_LocalEulerAnglesHint")
            .map(|value| vector(value, ["x", "y", "z"]))
            .transpose()?,
        rect: if object.info.type_id == 224 {
            Some(RectTransformData {
                anchor_min: vector(property(object, "m_AnchorMin")?, ["x", "y"])?,
                anchor_max: vector(property(object, "m_AnchorMax")?, ["x", "y"])?,
                anchored_position: vector(property(object, "m_AnchoredPosition")?, ["x", "y"])?,
                size_delta: vector(property(object, "m_SizeDelta")?, ["x", "y"])?,
                pivot: vector(property(object, "m_Pivot")?, ["x", "y"])?,
            })
        } else {
            None
        },
    })
}

pub(super) struct Importer<'a> {
    pub(super) resolver: Resolver<'a>,
    pub(super) output: PathBuf,
    classes: &'a BTreeMap<ObjectId, i32>,
    records: BTreeMap<ObjectId, ObjectRecord>,
    resources: BTreeSet<ObjectId>,
    processed_resources: BTreeSet<ObjectId>,
    textures: BTreeMap<ObjectId, TextureAsset>,
    texture_images: BTreeMap<ObjectId, Arc<RgbaImage>>,
    sprites: BTreeMap<ObjectId, UiSprite>,
    fonts: BTreeMap<ObjectId, FontAsset>,
    scripts: BTreeMap<ObjectId, ScriptIdentity>,
    pub(super) issues: Vec<ImportIssue>,
}

impl<'a> Importer<'a> {
    fn new(apk: &Path, discovery: &'a Discovery, output: PathBuf) -> Result<Self, Error> {
        Ok(Self {
            resolver: Resolver::new(apk, &discovery.index)?,
            output,
            classes: &discovery.classes,
            records: BTreeMap::new(),
            resources: BTreeSet::new(),
            processed_resources: BTreeSet::new(),
            textures: BTreeMap::new(),
            texture_images: BTreeMap::new(),
            sprites: BTreeMap::new(),
            fonts: BTreeMap::new(),
            scripts: BTreeMap::new(),
            issues: Vec::new(),
        })
    }

    pub(super) fn issue(
        &mut self,
        severity: &str,
        code: &str,
        id: Option<&ObjectId>,
        field_path: Option<&str>,
        detail: String,
    ) {
        self.issues.push(ImportIssue {
            severity: severity.into(),
            code: code.into(),
            source: id.map(ObjectId::source),
            field_path: field_path.map(str::to_owned),
            detail,
        });
    }

    pub(super) fn reference(
        &mut self,
        owner: &ObjectId,
        pointer: &UnityValue,
    ) -> Result<SourceReference, Error> {
        let file_id = integer(field(pointer, "m_FileID")?)?;
        let path_id = integer(field(pointer, "m_PathID")?)?;
        let resolution = if path_id == 0 {
            ReferenceState::Null
        } else {
            match self.resolver.resolve(owner, pointer) {
                Ok(Some(target)) => match self.classes.get(&target) {
                    Some(&class_id) => ReferenceState::Resolved {
                        target: target.source(),
                        class_id,
                    },
                    None => ReferenceState::Unresolved {
                        reason: format!(
                            "target {}:{}:{} is absent from the observed dependency object tables",
                            target.bundle, target.cab, target.path_id
                        ),
                    },
                },
                Ok(None) => return Err(invalid("nonzero PathID unexpectedly resolved as null")),
                Err(error) => ReferenceState::Unresolved {
                    reason: error.to_string(),
                },
            }
        };
        Ok(SourceReference {
            owner: owner.source(),
            file_id,
            path_id: path_id.to_string(),
            resolution,
        })
    }

    fn property_reference(
        &mut self,
        owner: &ObjectId,
        object: &UnityObject,
        name: &str,
    ) -> Result<SourceReference, Error> {
        self.reference(owner, property(object, name)?)
    }

    pub(super) fn store_bytes(
        &self,
        directory: &str,
        extension: &str,
        data: &[u8],
    ) -> Result<FileIdentity, Error> {
        let hash = format!("{:x}", Sha256::digest(data));
        let relative = format!("{directory}/{hash}.{extension}");
        let path = self.output.join(&relative);
        std::fs::create_dir_all(
            path.parent()
                .ok_or_else(|| invalid("source output has no parent"))?,
        )?;
        std::fs::write(path, data)?;
        Ok(FileIdentity {
            path: relative,
            sha256: hash,
        })
    }

    fn source_value(
        &mut self,
        owner: &ObjectId,
        value: &UnityValue,
        location: &str,
        references: &mut Vec<LocatedReference>,
    ) -> Result<Value, Error> {
        match value {
            UnityValue::Null => Ok(Value::Null),
            UnityValue::Bool(value) => Ok(json!(value)),
            UnityValue::Integer(value) if value.unsigned_abs() > 9_007_199_254_740_991 => {
                Ok(json!({"unity_i64": value.to_string()}))
            }
            UnityValue::Integer(value) => Ok(json!(value)),
            UnityValue::Float(value) => {
                if !value.is_finite() {
                    return Err(invalid(format!("non-finite source field {location}")));
                }
                Ok(json!(value))
            }
            UnityValue::String(value) => Ok(json!(value)),
            UnityValue::Bytes(value) => Ok(json!({
                "unity_bytes": self.store_bytes("source_blobs", "bin", value)?,
                "byte_count": value.len(),
            })),
            UnityValue::Array(values) => values
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    self.source_value(owner, value, &format!("{location}/{index}"), references)
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Array),
            UnityValue::Object(fields)
                if fields.contains_key("m_FileID") && fields.contains_key("m_PathID") =>
            {
                if fields.len() != 2 {
                    return Err(invalid(format!(
                        "PPtr at {location} has unexpected extra fields"
                    )));
                }
                let reference = self.reference(owner, value)?;
                references.push(LocatedReference {
                    field_path: location.to_owned(),
                    reference: reference.clone(),
                });
                Ok(serde_json::to_value(reference)?)
            }
            UnityValue::Object(fields) => fields
                .iter()
                .map(|(name, value)| {
                    let escaped = name.replace('~', "~0").replace('/', "~1");
                    Ok((
                        name.clone(),
                        self.source_value(
                            owner,
                            value,
                            &format!("{location}/{escaped}"),
                            references,
                        )?,
                    ))
                })
                .collect::<Result<serde_json::Map<String, Value>, Error>>()
                .map(Value::Object),
        }
    }

    pub(super) fn record(
        &mut self,
        id: &ObjectId,
        object: &UnityObject,
        enqueue_resources: bool,
    ) -> Result<ObjectRecord, Error> {
        if let Some(record) = self.records.get(id) {
            return Ok(record.clone());
        }
        let serialized = self.store_bytes("source_objects", "bin", object.raw_data())?;
        let mut references = Vec::new();
        let fields = self.source_value(
            id,
            &UnityValue::Object(object.class.properties().clone()),
            "",
            &mut references,
        )?;
        let source_fields = self.store_bytes(
            "source_objects",
            "json",
            &serde_json::to_vec_pretty(&fields)?,
        )?;
        let bundle = self.resolver.bundle(&id.bundle)?;
        let asset_index = bundle
            .asset_names
            .iter()
            .position(|name| cab_name(name) == id.cab)
            .ok_or_else(|| invalid("source object CAB disappeared"))?;
        for located in &references {
            match &located.reference.resolution {
                ReferenceState::Resolved { class_id, .. }
                    if enqueue_resources
                        && matches!(
                            class_id,
                            21 | 28 | 43 | 48 | 74 | 91 | 115 | 128 | 213 | 687078895
                        ) =>
                {
                    self.resources.insert(reference_id(&located.reference)?);
                }
                ReferenceState::Unresolved { reason } => {
                    let severity = if reason.contains("unity_builtin_extra")
                        || reason.contains("unity default resources")
                    {
                        "warning"
                    } else {
                        "error"
                    };
                    self.issue(
                        severity,
                        "unresolved_reference",
                        Some(id),
                        Some(&located.field_path),
                        reason.clone(),
                    );
                }
                _ => {}
            }
        }
        let record = ObjectRecord {
            source: id.source(),
            class_id: object.info.type_id,
            unity_version: bundle.assets[asset_index].unity_version.clone(),
            serialized_byte_count: object.raw_data().len() as u64,
            serialized,
            source_fields,
            references,
        };
        self.records.insert(id.clone(), record.clone());
        Ok(record)
    }

    pub(super) fn source_fields_for(&self, id: &ObjectId) -> Result<FileIdentity, Error> {
        self.records
            .get(id)
            .map(|record| record.source_fields.clone())
            .ok_or_else(|| invalid("source object was not retained"))
    }

    fn script(
        &mut self,
        owner: &ObjectId,
        object: &UnityObject,
    ) -> Result<Option<ScriptIdentity>, Error> {
        if object.info.type_id != 114 {
            return Ok(None);
        }
        let reference = self.property_reference(owner, object, "m_Script")?;
        if matches!(reference.resolution, ReferenceState::Null) {
            return Err(invalid("MonoBehaviour has no source MonoScript"));
        }
        let id = reference_id(&reference)?;
        if self.classes.get(&id) != Some(&115) {
            return Err(invalid(
                "MonoBehaviour script does not reference MonoScript",
            ));
        }
        if let Some(script) = self.scripts.get(&id) {
            return Ok(Some(script.clone()));
        }
        let object = self.resolver.object(&id)?;
        self.record(&id, &object, false)?;
        let script = ScriptIdentity {
            source: id.source(),
            class_name: text(property(&object, "m_ClassName")?)?.to_owned(),
            namespace: text(property(&object, "m_Namespace")?)?.to_owned(),
            assembly_name: text(property(&object, "m_AssemblyName")?)?.to_owned(),
        };
        self.scripts.insert(id, script.clone());
        Ok(Some(script))
    }

    fn graphic(&mut self, id: &ObjectId, object: &UnityObject) -> Result<GraphicData, Error> {
        Ok(GraphicData {
            color: vector(property(object, "m_Color")?, ["r", "g", "b", "a"])?,
            material: self.property_reference(id, object, "m_Material")?,
            raycast_target: flag(property(object, "m_RaycastTarget")?)?,
            raycast_padding: vector(property(object, "m_RaycastPadding")?, ["x", "y", "z", "w"])?,
            maskable: flag(property(object, "m_Maskable")?)?,
        })
    }

    fn projection(
        &mut self,
        id: &ObjectId,
        object: &UnityObject,
        script: Option<&ScriptIdentity>,
        record: &ObjectRecord,
    ) -> Result<ComponentProjection, Error> {
        let properties = object.class.properties();
        let class_name = script
            .map(|script| script.class_name.as_str())
            .unwrap_or("");
        if matches!(object.info.type_id, 4 | 224) {
            return Ok(ComponentProjection::Transform);
        }
        if properties.contains_key("m_Sprite")
            && properties.contains_key("m_Type")
            && properties.contains_key("m_FillAmount")
        {
            let fill_amount = image_fill_amount(object)?;
            return Ok(ComponentProjection::Image(ImageData {
                graphic: self.graphic(id, object)?,
                sprite: self.property_reference(id, object, "m_Sprite")?,
                image_type: int32(property(object, "m_Type")?)?,
                preserve_aspect: flag(property(object, "m_PreserveAspect")?)?,
                fill_center: flag(property(object, "m_FillCenter")?)?,
                fill_method: int32(property(object, "m_FillMethod")?)?,
                fill_origin: int32(property(object, "m_FillOrigin")?)?,
                fill_clockwise: flag(property(object, "m_FillClockwise")?)?,
                fill_amount,
                use_sprite_mesh: flag(property(object, "m_UseSpriteMesh")?)?,
                pixels_per_unit_multiplier: number(property(object, "m_PixelsPerUnitMultiplier")?)?,
            }));
        }
        if properties.contains_key("m_FontData") && properties.contains_key("m_Text") {
            let font = property(object, "m_FontData")?;
            return Ok(ComponentProjection::Text(TextData {
                graphic: self.graphic(id, object)?,
                text: text(property(object, "m_Text")?)?.to_owned(),
                font: self.reference(id, field(font, "m_Font")?)?,
                font_size: int32(field(font, "m_FontSize")?)?,
                font_style: int32(field(font, "m_FontStyle")?)?,
                alignment: int32(field(font, "m_Alignment")?)?,
                align_by_geometry: flag(field(font, "m_AlignByGeometry")?)?,
                line_spacing: number(field(font, "m_LineSpacing")?)?,
                rich_text: flag(field(font, "m_RichText")?)?,
                best_fit: flag(field(font, "m_BestFit")?)?,
                min_size: int32(field(font, "m_MinSize")?)?,
                max_size: int32(field(font, "m_MaxSize")?)?,
                horizontal_overflow: int32(field(font, "m_HorizontalOverflow")?)?,
                vertical_overflow: int32(field(font, "m_VerticalOverflow")?)?,
            }));
        }
        if class_name == "RawImage" {
            return Ok(ComponentProjection::RawImage(RawImageData {
                graphic: self.graphic(id, object)?,
                texture: self.property_reference(id, object, "m_Texture")?,
                uv_rect: rect(property(object, "m_UVRect")?)?,
            }));
        }
        let retained_fields = || -> Result<Value, Error> {
            Ok(serde_json::from_reader(BufReader::new(File::open(
                self.output.join(&record.source_fields.path),
            )?))?)
        };
        if object.info.type_id == 223 {
            return Ok(ComponentProjection::Canvas(retained_fields()?));
        }
        if class_name == "CanvasScaler" {
            return Ok(ComponentProjection::CanvasScaler(retained_fields()?));
        }
        if matches!(class_name, "Mask" | "RectMask2D" | "StencilMask") {
            return Ok(ComponentProjection::Mask(retained_fields()?));
        }
        if matches!(
            class_name,
            "Button"
                | "EmptyButton"
                | "Slider"
                | "Toggle"
                | "ToggleGroup"
                | "ScrollRect"
                | "InputField"
                | "RadioButton"
        ) {
            return Ok(ComponentProjection::Selectable(retained_fields()?));
        }
        Ok(ComponentProjection::RetainedSource)
    }

    fn node(
        &mut self,
        id: &ObjectId,
        expected_parent: Option<&ObjectId>,
        parent_active: bool,
        ancestors: &mut Vec<ObjectId>,
        visited: &mut BTreeSet<ObjectId>,
        stats: &mut PanelStats,
    ) -> Result<HierarchyNode, Error> {
        enter_hierarchy(id, ancestors, visited)?;
        let object = self.resolver.object(id)?;
        if object.info.type_id != 1 {
            return Err(invalid("prefab hierarchy does not reference GameObject"));
        }
        self.record(id, &object, true)?;
        let active_self = flag(property(&object, "m_IsActive")?)?;
        let mut components = Vec::new();
        let mut transform = None;
        for item in array(property(&object, "m_Component")?)? {
            let reference = self.reference(id, field(item, "component")?)?;
            let component_id = reference_id(&reference)?;
            let component_object = self.resolver.object(&component_id)?;
            let owner =
                self.property_reference(&component_id, &component_object, "m_GameObject")?;
            if reference_id(&owner)? != *id {
                return Err(invalid(
                    "component's GameObject PPtr disagrees with ordered component list",
                ));
            }
            let record = self.record(&component_id, &component_object, true)?;
            let script = self.script(&component_id, &component_object)?;
            let enabled = component_object
                .class
                .properties()
                .get("m_Enabled")
                .map(flag)
                .transpose()?;
            let projection =
                self.projection(&component_id, &component_object, script.as_ref(), &record)?;
            if matches!(component_object.info.type_id, 4 | 224) {
                if transform.is_some() {
                    return Err(invalid("GameObject has more than one source transform"));
                }
                let scalars = transform_scalars(&component_object)?;
                let parent =
                    self.property_reference(&component_id, &component_object, "m_Father")?;
                let actual_parent = if matches!(parent.resolution, ReferenceState::Null) {
                    None
                } else {
                    Some(reference_id(&parent)?)
                };
                if actual_parent.as_ref() != expected_parent {
                    return Err(invalid(
                        "Transform.m_Father disagrees with hierarchy parent",
                    ));
                }
                let ordered_children = array(property(&component_object, "m_Children")?)?
                    .iter()
                    .map(|pointer| self.reference(&component_id, pointer))
                    .collect::<Result<Vec<_>, _>>()?;
                transform = Some(TransformData {
                    source: component_id.source(),
                    local_position: scalars.local_position,
                    local_rotation: scalars.local_rotation,
                    local_scale: scalars.local_scale,
                    parent,
                    ordered_children,
                    root_order: scalars.root_order,
                    local_euler_angles_hint: scalars.local_euler_angles_hint,
                    rect: scalars.rect,
                });
            }
            stats.components += 1;
            if enabled == Some(false) {
                stats.disabled_components += 1;
            }
            if let Some(script) = &script {
                *stats
                    .script_types
                    .entry(script.class_name.clone())
                    .or_default() += 1;
            }
            match &projection {
                ComponentProjection::Image(image) => {
                    stats.image_components += 1;
                    if image.fill_amount == 0.0 {
                        stats.zero_fill_images += 1;
                    }
                }
                ComponentProjection::Text(_) => stats.text_components += 1,
                ComponentProjection::RawImage(_) => stats.raw_image_components += 1,
                ComponentProjection::Canvas(_) => stats.canvases += 1,
                ComponentProjection::CanvasScaler(_) => stats.canvas_scalers += 1,
                ComponentProjection::Mask(_) => stats.masks += 1,
                ComponentProjection::Selectable(_) => stats.selectables += 1,
                ComponentProjection::RetainedSource => {
                    let label = script
                        .as_ref()
                        .map(|script| script.class_name.clone())
                        .unwrap_or_else(|| {
                            format!("native_class_{}", component_object.info.type_id)
                        });
                    *stats.retained_component_types.entry(label).or_default() += 1;
                }
                _ => {}
            }
            components.push(UiComponent {
                reference,
                source: component_id.source(),
                class_id: component_object.info.type_id,
                enabled,
                script,
                projection,
                source_fields: record.source_fields,
                serialized: record.serialized,
            });
        }
        let transform = transform.ok_or_else(|| invalid("GameObject has no source transform"))?;
        let transform_id = ObjectId {
            bundle: transform.source.bundle.clone(),
            cab: transform.source.cab.clone(),
            path_id: transform.source.path_id.parse()?,
        };
        let mut children = Vec::new();
        for reference in &transform.ordered_children {
            let child_transform_id = reference_id(reference)?;
            let child_transform = self.resolver.object(&child_transform_id)?;
            if !matches!(child_transform.info.type_id, 4 | 224) {
                return Err(invalid(
                    "Transform child PPtr does not reference a transform",
                ));
            }
            let child_id = reference_id(&self.property_reference(
                &child_transform_id,
                &child_transform,
                "m_GameObject",
            )?)?;
            children.push(self.node(
                &child_id,
                Some(&transform_id),
                parent_active && active_self,
                ancestors,
                visited,
                stats,
            )?);
        }
        stats.nodes += 1;
        if !active_self {
            stats.inactive_nodes += 1;
        }
        if transform.rect.is_some() {
            stats.rect_transforms += 1;
        } else {
            stats.ordinary_transforms += 1;
        }
        let [x, y, z, w] = transform.local_rotation;
        if x.abs() > 1e-6 || y.abs() > 1e-6 || z.abs() > 1e-6 || (w.abs() - 1.0).abs() > 1e-6 {
            stats.non_identity_rotations += 1;
        }
        ancestors.pop();
        Ok(HierarchyNode {
            source: id.source(),
            name: text(property(&object, "m_Name")?)?.to_owned(),
            active_self,
            active_in_prefab_hierarchy: parent_active && active_self,
            layer: int32(property(&object, "m_Layer")?)?,
            tag: object
                .class
                .properties()
                .get("m_Tag")
                .map(integer)
                .transpose()?,
            transform,
            components,
            children,
        })
    }

    fn panel(&mut self, name: &str, member: &str, apk_hash: &str) -> Result<UiPanel, Error> {
        let bundle = self.resolver.bundle(member)?;
        let mut candidates = Vec::new();
        for (asset_name, asset) in bundle.asset_names.iter().zip(&bundle.assets) {
            for info in asset.objects.iter().filter(|info| info.type_id == 142) {
                let container_id = ObjectId {
                    bundle: member.to_owned(),
                    cab: cab_name(asset_name).to_owned(),
                    path_id: info.path_id,
                };
                let container = read_object(asset, info)?;
                self.record(&container_id, &container, false)?;
                for entry in array(property(&container, "m_Container")?)? {
                    let pair = array(entry)?;
                    if pair.len() != 2 {
                        return Err(invalid("malformed AssetBundle container pair"));
                    }
                    let asset_path = text(&pair[0])?;
                    let expected = format!("{}.prefab", name.to_lowercase());
                    if asset_path.rsplit('/').next() != Some(expected.as_str()) {
                        continue;
                    }
                    let reference = self.reference(&container_id, field(&pair[1], "asset")?)?;
                    candidates.push((asset_path.to_owned(), reference));
                }
            }
        }
        if candidates.len() != 1 {
            return Err(invalid(format!(
                "{member}: expected one source prefab container entry for {name}, found {}",
                candidates.len()
            )));
        }
        let (prefab_asset_path, prefab_reference) = candidates
            .pop()
            .ok_or_else(|| invalid("missing source prefab"))?;
        let root_id = reference_id(&prefab_reference)?;
        let mut visited = BTreeSet::new();
        let mut stats = PanelStats::default();
        let root = self.node(
            &root_id,
            None,
            true,
            &mut Vec::new(),
            &mut visited,
            &mut stats,
        )?;
        if root.name != name {
            return Err(invalid(format!(
                "source prefab root is {}, expected {name}",
                root.name
            )));
        }
        let expected_nodes = self
            .classes
            .iter()
            .filter(|(id, class)| id.bundle == member && **class == 1)
            .count();
        if visited.len() != expected_nodes {
            return Err(invalid(format!(
                "{name}: imported {} hierarchy GameObjects, source bundle contains {expected_nodes}",
                visited.len()
            )));
        }
        Ok(UiPanel {
            schema_version: SCHEMA_VERSION,
            source_apk_sha256: apk_hash.to_owned(),
            name: name.into(),
            bundle: member.into(),
            prefab_asset_path,
            prefab_reference,
            stats,
            root,
        })
    }

    pub(super) fn texture_image(&mut self, id: &ObjectId) -> Result<Arc<RgbaImage>, Error> {
        if let Some(image) = self.texture_images.get(id) {
            return Ok(image.clone());
        }
        if self.classes.get(id) != Some(&28) {
            return Err(invalid("UI graphic texture reference is not Texture2D"));
        }
        let object = self.resolver.object(id)?;
        self.record(id, &object, false)?;
        let texture = self.resolver.texture(id, &self.output)?;
        let image = Arc::new(image::open(self.output.join(&texture.file.path))?.into_rgba8());
        if image.dimensions() != (texture.width, texture.height) {
            return Err(invalid(
                "decoded UI texture dimensions disagree with retained texture metadata",
            ));
        }
        self.textures.insert(id.clone(), texture);
        self.texture_images.insert(id.clone(), image.clone());
        Ok(image)
    }

    fn font(&mut self, id: &ObjectId, object: &UnityObject) -> Result<(), Error> {
        let font_bytes = bytes(property(object, "m_FontData")?)?;
        let embedded_font = if font_bytes.is_empty() {
            self.issue("warning", "font_has_no_embedded_outline", Some(id), Some("/m_FontData"), "Font has no embedded outline bytes; retained source glyph/material fields must be interpreted before runtime use".into());
            None
        } else {
            let extension = if font_bytes.starts_with(b"OTTO") {
                "otf"
            } else if font_bytes.starts_with(b"ttcf") {
                "ttc"
            } else if font_bytes.starts_with(&[0, 1, 0, 0]) {
                "ttf"
            } else {
                "font.bin"
            };
            Some(self.store_bytes("fonts", extension, &font_bytes)?)
        };
        self.fonts.insert(
            id.clone(),
            FontAsset {
                source: id.source(),
                name: text(property(object, "m_Name")?)?.to_owned(),
                embedded_font,
                source_fields: self.source_fields_for(id)?,
            },
        );
        Ok(())
    }

    fn drain_resources(&mut self) {
        while let Some(id) = self.resources.pop_first() {
            if !self.processed_resources.insert(id.clone()) {
                continue;
            }
            let result = (|| -> Result<(), Error> {
                let class = *self
                    .classes
                    .get(&id)
                    .ok_or_else(|| invalid("resource is absent from source object inventory"))?;
                let object = self.resolver.object(&id)?;
                // Atlas packed-sprite tables and shader default dependency lists are
                // retained but do not broaden the selected prefab's graphics closure.
                self.record(&id, &object, !matches!(class, 28 | 48 | 213 | 687078895))?;
                match class {
                    28 => {
                        self.texture_image(&id)?;
                    }
                    128 => self.font(&id, &object)?,
                    213 => {
                        let sprite = sprite::extract(self, &id, &object)?;
                        self.sprites.insert(id.clone(), sprite);
                    }
                    _ => {}
                }
                Ok(())
            })();
            if let Err(error) = result {
                self.issue(
                    "error",
                    "resource_extraction_failed",
                    Some(&id),
                    None,
                    error.to_string(),
                );
            }
        }
    }
}

fn write_json<T: Serialize>(root: &Path, relative: &str, value: &T) -> Result<FileIdentity, Error> {
    let data = serde_json::to_vec_pretty(value)?;
    let path = root.join(relative);
    std::fs::create_dir_all(
        path.parent()
            .ok_or_else(|| invalid("UI JSON output has no parent"))?,
    )?;
    std::fs::write(&path, &data)?;
    Ok(FileIdentity {
        path: relative.into(),
        sha256: format!("{:x}", Sha256::digest(&data)),
    })
}

pub(super) fn run(apk: &Path, out_dir: &Path) -> Result<(), Error> {
    let start = Instant::now();
    eprintln!("PASS 3: source UI extraction from {}", apk.display());
    let source = source_apk(apk)?;
    let output = out_dir
        .join("imported_ui")
        .join(format!("{}-v{SCHEMA_VERSION}", source.sha256));
    std::fs::create_dir_all(&output)?;
    let discovery = discover(apk)?;
    let dependency_index = write_json(&output, "dependency_index.json", &discovery.index)?;
    let mut importer = Importer::new(apk, &discovery, output.clone())?;
    let mut panels = Vec::new();
    for (name, member, filename) in PANELS {
        match importer.panel(name, member, &source.sha256) {
            Ok(panel) => {
                eprintln!(
                    "  {name}: {} nodes, {} Images, {} Texts, {} disabled components",
                    panel.stats.nodes,
                    panel.stats.image_components,
                    panel.stats.text_components,
                    panel.stats.disabled_components
                );
                let layout = write_json(&output, &format!("panels/{filename}.json"), &panel)?;
                panels.push(PanelSummary {
                    name: name.into(),
                    bundle: member.into(),
                    layout: Some(layout),
                    hierarchy_complete: true,
                    stats: Some(panel.stats),
                });
            }
            Err(error) => {
                importer.issue(
                    "error",
                    "panel_extraction_failed",
                    None,
                    None,
                    format!("{name}: {error}"),
                );
                panels.push(PanelSummary {
                    name: name.into(),
                    bundle: member.into(),
                    layout: None,
                    hierarchy_complete: false,
                    stats: None,
                });
            }
        }
    }
    importer.drain_resources();
    let source_after = source_apk(apk)?;
    if source_after.sha256 != source.sha256 || source_after.byte_count != source.byte_count {
        importer.issue(
            "error",
            "source_apk_changed",
            None,
            None,
            "APK content changed during extraction".into(),
        );
    }
    let hierarchy_complete =
        panels.len() == PANELS.len() && panels.iter().all(|panel| panel.hierarchy_complete);
    let graphics_complete = !importer
        .issues
        .iter()
        .any(|issue| issue.severity == "error");
    let reference_coverage_complete = !importer
        .issues
        .iter()
        .any(|issue| issue.code == "unresolved_reference");
    let error_count = importer
        .issues
        .iter()
        .filter(|issue| issue.severity == "error")
        .count();
    let warning_count = importer
        .issues
        .iter()
        .filter(|issue| issue.severity == "warning")
        .count();
    let manifest = UiManifest {
        schema_version: SCHEMA_VERSION, source_apk: source, dependency_index, bundles: discovery.bundles.clone(), panels,
        sprites: importer.sprites.into_values().collect(), textures: importer.textures.into_values().collect(),
        fonts: importer.fonts.into_values().collect(), source_objects: importer.records.into_values().collect(),
        hierarchy_complete, graphics_complete, reference_coverage_complete, runtime_equivalence_verified: false,
        issues: importer.issues,
        limitations: vec![
            "Static prefab values do not establish runtime visibility, state changes, event listeners, or behavior.".into(),
            "CanvasScaler absent from a selected prefab remains absent; its runtime parent canvas/scaling configuration is unobserved.".into(),
            "Custom/native components without a projection retain complete source fields and original bytes; their behavior is not implemented.".into(),
            "Sprite PNGs are unpacked integer texture-crop previews. Original sprite canvas, fractional trim, and mesh geometry are retained separately and have not been rasterized into the previews.".into(),
            "Unity built-in resource PPtrs may remain explicit unresolved references because their serialized resource file is outside the declared APK bundle closure.".into(),
            "No runtime UI files were replaced or promoted by this extraction pass.".into(),
        ],
        source_field_encoding: vec![
            "PPtrs become SourceReference objects with owner, FileID, signed PathID string, and explicit resolution state.".into(),
            "Integers outside the exact JavaScript range use {unity_i64: decimal_string}; smaller non-PPtr integers retain JSON integer values.".into(),
            "Unity byte arrays use {unity_bytes: FileIdentity, byte_count}; each referenced file contains all original bytes.".into(),
            "Other objects, arrays, strings, finite floats and boolean values are retained in full; source binary object bytes are also retained.".into(),
        ],
    };
    let manifest_file = write_json(&output, "source_ui.json", &manifest)?;
    eprintln!(
        "  {} sprites, {} textures, {} fonts, {} source objects; {error_count} errors, {warning_count} warnings -> {} ({:.2?})",
        manifest.sprites.len(),
        manifest.textures.len(),
        manifest.fonts.len(),
        manifest.source_objects.len(),
        output.join(&manifest_file.path).display(),
        start.elapsed()
    );
    if !hierarchy_complete || error_count > 0 {
        return Err(invalid(format!(
            "UI source import is incomplete; review {}",
            output.join(&manifest_file.path).display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use unity_asset_binary::asset::ObjectInfo;
    use unity_asset_core::UnityClass;

    fn id(cab: &str, path_id: i64) -> ObjectId {
        ObjectId {
            bundle: format!("assets/{cab}.ab"),
            cab: cab.into(),
            path_id,
        }
    }

    fn object(class_id: i32, fields: Value) -> UnityObject {
        let mut class = UnityClass::new(class_id, "Test".into(), "1".into());
        let fields: UnityValue = serde_json::from_value(fields).unwrap();
        for (name, value) in fields.as_object().unwrap() {
            class.set(name.clone(), value.clone());
        }
        UnityObject::from_info_and_class(ObjectInfo::new(1, 0, 0, class_id, -1), class)
    }

    #[test]
    fn false_source_flags_and_zero_fill_remain_zero() {
        assert!(!flag(&UnityValue::Bool(false)).unwrap());
        assert!(!flag(&UnityValue::Integer(0)).unwrap());
        assert!(flag(&UnityValue::Integer(2)).is_err());
        assert!(flag(&UnityValue::Null).is_err());
        let image = object(114, json!({"m_FillAmount": 0.0}));
        assert_eq!(image_fill_amount(&image).unwrap(), 0.0);
        assert!(image_fill_amount(&object(114, json!({}))).is_err());
        assert!(image_fill_amount(&object(114, json!({"m_FillAmount": 1.1}))).is_err());
    }

    #[test]
    fn source_rotation_and_rect_fields_are_preserved_without_defaults() {
        let fields = json!({
            "m_LocalPosition": {"x": 1.0, "y": 2.0, "z": 3.0},
            "m_LocalScale": {"x": 2.0, "y": 3.0, "z": 4.0},
            "m_LocalRotation": {"x": 0.0, "y": 0.0, "z": 0.5, "w": 0.75},
            "m_AnchorMin": {"x": 0.0, "y": 0.25}, "m_AnchorMax": {"x": 1.0, "y": 0.75},
            "m_AnchoredPosition": {"x": -20.0, "y": 30.0}, "m_SizeDelta": {"x": 330.0, "y": 260.0},
            "m_Pivot": {"x": 0.1, "y": 0.9}
        });
        let source = transform_scalars(&object(224, fields.clone())).unwrap();
        assert_eq!(source.local_rotation, [0.0, 0.0, 0.5, 0.75]);
        assert_eq!(source.local_position, [1.0, 2.0, 3.0]);
        assert_eq!(source.rect.unwrap().anchored_position, [-20.0, 30.0]);
        let mut incomplete = fields;
        incomplete.as_object_mut().unwrap().remove("m_Pivot");
        assert!(transform_scalars(&object(224, incomplete)).is_err());
    }

    #[test]
    fn qualified_ids_and_explicit_unresolved_states_survive_json() {
        let first = id("CAB-a", i64::MIN + 1);
        let second = id("CAB-b", i64::MIN + 1);
        let reference = SourceReference {
            owner: first.source(),
            file_id: 7,
            path_id: second.path_id.to_string(),
            resolution: ReferenceState::Resolved {
                target: second.source(),
                class_id: 213,
            },
        };
        let encoded = serde_json::to_string(&reference).unwrap();
        assert!(encoded.contains("\"path_id\":\"-9223372036854775807\""));
        let decoded: SourceReference = serde_json::from_str(&encoded).unwrap();
        assert_eq!(reference_id(&decoded).unwrap(), second);
        assert_ne!(reference_id(&decoded).unwrap(), first);
        let unresolved = SourceReference {
            resolution: ReferenceState::Unresolved {
                reason: "missing CAB".into(),
            },
            ..reference
        };
        assert!(reference_id(&unresolved).is_err());
        assert!(
            serde_json::to_value(&unresolved).unwrap()["resolution"]["reason"] == "missing CAB"
        );
    }

    #[test]
    fn hierarchy_rejects_cycles_and_duplicate_children_with_qualified_identity() {
        let first = id("CAB-a", 42);
        let second = id("CAB-b", 42);
        let mut ancestors = Vec::new();
        let mut visited = BTreeSet::new();
        enter_hierarchy(&first, &mut ancestors, &mut visited).unwrap();
        enter_hierarchy(&second, &mut ancestors, &mut visited).unwrap();
        assert_eq!(ancestors, vec![first.clone(), second.clone()]);
        assert!(enter_hierarchy(&first, &mut ancestors, &mut visited).is_err());
        ancestors.clear();
        assert!(enter_hierarchy(&second, &mut ancestors, &mut visited).is_err());
    }
}
