//! Native HUD bindings for the retained BattleMinimap. The texture assignment,
//! crop/zoom policy and simulation marker categories are explicit native choices;
//! the source hierarchy, sprite UVs, masks and font payloads remain unchanged.

use std::{collections::BTreeMap, path::Path};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use midnight_map_data::SourceObject;
use midnight_ui_data::{ComponentProjection, HierarchyNode, UiError, layout::CanvasViewport};
use serde::Serialize;
use shared::map_projection;

use crate::source_ui::{
    PanelRenderReport, SourcePanelRenderer, SourceUiBindings, SourceUiInstance,
};

pub const SOURCE_UI_PACKAGE: &str =
    "imported_ui/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1";
pub const SOURCE_APK_SHA256: &str =
    "2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df";
pub const PANEL_SIZE: [f32; 2] = [360.0, 363.0];
pub const MAP_DIAMETER: f32 = 260.0;
pub const DEFAULT_ZOOM: f32 = 15.0;
const HEADER_GUTTER: f32 = 32.5;
const BUNDLE: &str = "assets/ui/battleminimap.ab";
const CAB: &str = "CAB-88df926bf45d4de159fb33b326042714";
const ATLAS_BUNDLE: &str = "assets/uiatlas/battlepanel.ab";
const ATLAS_CAB: &str = "CAB-2c321042086139a67290fadcb8d82a1a";

const ROOT: &str = "1749944185578701977";
const MAP_IMAGE: &str = "-7917619469819177287";
const MAP_RAW_IMAGE: &str = "910110631840687877";
const COORDINATES: &str = "6160426282398158155";
const HEIGHT: &str = "6409308081353037390";
const GAME_TIME: &str = "-6601116259395585505";
const PLAYER_ARROW: &str = "8066549267026405476";
const PLAYER_CONE: &str = "8229701778458991505";
const TREE: &str = "-519494033856982854";
const GENERIC: &str = "7759753996820907906";
const VEHICLE: &str = "9090670034721949667";
const WAYPOINT: &str = "-7306333143872860004";
const CARGO: &str = "4952203664210595836";
const FOOTSTEP: &str = "-6352242480817724359";
const GUNFIRE: &str = "1292197259836537604";
const VEHICLE_SOUND: &str = "6438195031107934756";
const AIRDROP: &str = "7901497134089479617";

fn source(id: &str) -> SourceObject {
    SourceObject {
        bundle: BUNDLE.into(),
        cab: CAB.into(),
        path_id: id.into(),
    }
}

fn atlas_sprite(id: &str) -> SourceObject {
    SourceObject {
        bundle: ATLAS_BUNDLE.into(),
        cab: ATLAS_CAB.into(),
        path_id: id.into(),
    }
}

pub fn map_texture() -> SourceObject {
    SourceObject {
        bundle: "assets/texture/map_map.ab".into(),
        cab: "CAB-6f665c88cf2187ff08df209ad5c96ca6".into(),
        path_id: "-945003921129902140".into(),
    }
}

fn invalid(message: impl Into<String>) -> UiError {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()).into()
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct MinimapProjection {
    pub center_xz: [f32; 2],
    /// Shared with the native overview; this extent remains provisional.
    pub half_extent: f32,
    pub zoom: f32,
}

impl MinimapProjection {
    pub fn world_range(self) -> f32 {
        self.half_extent / self.zoom
    }

    /// Unity RawImage UVs start at bottom-left. Decoded atlases and shared
    /// overview projection start at top-left, so invert V exactly once here.
    pub fn uv_rect(self) -> [f32; 4] {
        let [u, v] = map_projection::world_to_overview_uv(self.center_xz, self.half_extent);
        let span = 1.0 / self.zoom;
        [u - span * 0.5, 1.0 - v - span * 0.5, span, span]
    }

    /// Source RectTransform XY uses Y-up; canonical world north is Bevy -Z.
    pub fn world_to_local(self, point: [f32; 2]) -> [f32; 2] {
        let [x, y] = map_projection::world_to_radar_offset(
            point,
            self.center_xz,
            self.world_range(),
            MAP_DIAMETER * 0.5,
        );
        [x, -y]
    }

    pub fn marker_position(self, point: [f32; 2], icon_span: f32, clamp: bool) -> Option<[f32; 2]> {
        if point.iter().any(|value| !value.is_finite()) {
            return None;
        }
        let position = Vec2::from_array(self.world_to_local(point));
        let edge = (MAP_DIAMETER - icon_span) * 0.5 - 2.0;
        let distance = position.length();
        if distance <= edge {
            Some(position.to_array())
        } else if clamp {
            Some((position / distance * edge).to_array())
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MinimapMarkerKind {
    Tree,
    Rock,
    Structure,
    ToolCupboard,
    Enemy,
    Vehicle,
    Cargo,
    Footstep,
    Gunfire,
    VehicleSound,
    Airdrop,
}

#[derive(Clone, Debug, Serialize)]
pub struct MinimapMarker {
    pub key: String,
    pub kind: MinimapMarkerKind,
    pub world_xz: [f32; 2],
    pub yaw: f32,
    pub intensity: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct MinimapState {
    pub player_position: [f32; 3],
    pub yaw: f32,
    pub time_hours: f32,
    pub waypoint: Option<[f32; 2]>,
    pub markers: Vec<MinimapMarker>,
}

impl Default for MinimapState {
    fn default() -> Self {
        Self {
            player_position: [0.0, 2.0, 12.0],
            yaw: 0.0,
            time_hours: 10.0,
            waypoint: None,
            markers: Vec::new(),
        }
    }
}

struct NodeInfo {
    path: String,
    transform: SourceObject,
    ancestors: Vec<SourceObject>,
    image: Option<SourceObject>,
    size: [f32; 2],
    rotation: Quat,
}

pub struct NativeMinimap {
    renderer: SourcePanelRenderer,
    nodes: BTreeMap<SourceObject, NodeInfo>,
    base: SourceUiBindings,
    zoom: f32,
}

impl NativeMinimap {
    pub fn load(root: &Path, context: &egui::Context) -> Result<Self, UiError> {
        let mut renderer = SourcePanelRenderer::load(root, "BattleMinimap", context)?;
        if renderer.panel.source_apk_sha256 != SOURCE_APK_SHA256
            || renderer.panel.root.source != source(ROOT)
        {
            return Err(invalid(
                "native minimap requires the pinned lite4.0 source panel",
            ));
        }
        let texture = renderer
            .package()
            .texture(&map_texture())
            .ok_or_else(|| invalid("retained Map_map texture is absent"))?;
        if texture.name != "Map_map"
            || [texture.width, texture.height] != [2048, 2048]
            || texture.sampler.wrap[..2] != [1, 1]
        {
            return Err(invalid(
                "Map_map texture identity/dimensions/clamp sampler differ",
            ));
        }
        let mut nodes = BTreeMap::new();
        fn collect(
            node: &HierarchyNode,
            prefix: &str,
            ancestors: &mut Vec<SourceObject>,
            nodes: &mut BTreeMap<SourceObject, NodeInfo>,
        ) {
            let path = if prefix.is_empty() {
                node.name.clone()
            } else {
                format!("{prefix}/{}", node.name)
            };
            nodes.insert(
                node.source.clone(),
                NodeInfo {
                    path: path.clone(),
                    transform: node.transform.source.clone(),
                    ancestors: ancestors.clone(),
                    image: node.components.iter().find_map(|component| {
                        matches!(component.projection, ComponentProjection::Image(_))
                            .then(|| component.source.clone())
                    }),
                    size: node
                        .transform
                        .rect
                        .as_ref()
                        .map_or([0.0; 2], |rect| rect.size_delta),
                    rotation: Quat::from_array(node.transform.local_rotation),
                },
            );
            ancestors.push(node.source.clone());
            for child in &node.children {
                collect(child, &path, ancestors, nodes);
            }
            ancestors.pop();
        }
        collect(&renderer.panel.root, "", &mut Vec::new(), &mut nodes);
        // Resolve qualified IDs and check full paths once; repeated bare names
        // in this prefab cannot redirect a runtime binding.
        for (id, suffix) in [
            (MAP_IMAGE, "GameObject/ditu/lefttop/m_lefttop"),
            (
                PLAYER_ARROW,
                "GameObject/ditu/lefttop/m_mapShowIcon/m_team_members/member1/m_pos",
            ),
            (
                PLAYER_CONE,
                "GameObject/ditu/lefttop/m_mapShowIcon/m_team_members/member1/m_pos_light",
            ),
            (
                TREE,
                "GameObject/ditu/lefttop/m_mapShowIcon/m_resource_tags/m_tree_tags_root/m_tree_tag",
            ),
            (
                GENERIC,
                "GameObject/ditu/lefttop/m_mapShowIcon/m_resource_tags/m_main_tags_root/m_main_tag",
            ),
            (
                VEHICLE,
                "GameObject/ditu/lefttop/m_mapShowIcon/m_self_vehicle_parent/m_vehicle_icon",
            ),
            (
                WAYPOINT,
                "GameObject/ditu/lefttop/m_mapShowIcon/m_resource_tags/m_badge_pos_root/m_badge_img/Image",
            ),
            (
                CARGO,
                "GameObject/ditu/lefttop/m_mapShowIcon/m_resource_tags/m_freighter_pos_root/m_freighter_tag",
            ),
            (
                FOOTSTEP,
                "GameObject/ditu/lefttop/m_mapShowIcon/m_mapSound_parent/m_footSoundInMap/Image",
            ),
            (
                GUNFIRE,
                "GameObject/ditu/lefttop/m_mapShowIcon/m_mapSound_parent/m_gunSoundInMap/image",
            ),
            (
                VEHICLE_SOUND,
                "GameObject/ditu/lefttop/m_mapShowIcon/m_mapSound_parent/m_vehicleSoundInMap/Image",
            ),
            (
                AIRDROP,
                "GameObject/ditu/lefttop/m_mapShowIcon/m_resource_tags/m_airdrop_pos_root/m_airDrop_tag/Image",
            ),
        ] {
            let info = nodes
                .get(&source(id))
                .ok_or_else(|| invalid(format!("native minimap binding {id} is absent")))?;
            if info.path != format!("BattleMinimap/{suffix}") {
                return Err(invalid("native minimap binding source path differs"));
            }
        }
        for (id, name) in [
            ("-8404998493670798139", "1BattlePanel_Map_lv"),
            ("7901072812560143050", "1BattlePanel_Map_huibai"),
            ("3924826172084765333", "1BattlePanel_Map_huang"),
            ("7733724338977180109", "1BattlePanel_Map_hong"),
            ("4445139762678479602", "1BattlePanel_map_lingdigui"),
        ] {
            if renderer
                .package()
                .sprite(&atlas_sprite(id))
                .is_none_or(|sprite| sprite.name != name)
            {
                return Err(invalid("native marker Sprite identity differs"));
            }
        }
        let mut base = SourceUiBindings {
            visibility: nodes.keys().map(|source| (source.clone(), false)).collect(),
            ..Default::default()
        };
        // Native lifecycle: hide authored templates and unimplemented effects.
        // Activate only the required source nodes and their real ancestors.
        for id in [
            MAP_IMAGE,
            "-1917073483142831674",
            "-7520044512448050654",
            "-8977948790690202524",
            "-821781803138336836",
            "8365677517520596295",
            "9066736818001988192",
            "-3991086821951747717",
            "-7822706158898638032",
            "3799445044750627459",
        ] {
            activate(&nodes, &mut base, id, true)?;
        }
        for id in [
            PLAYER_ARROW,
            PLAYER_CONE,
            TREE,
            GENERIC,
            VEHICLE,
            WAYPOINT,
            CARGO,
            FOOTSTEP,
            GUNFIRE,
            VEHICLE_SOUND,
            AIRDROP,
        ] {
            activate(&nodes, &mut base, id, false)?;
        }
        base.raw_images.insert(source(MAP_RAW_IMAGE), map_texture());
        base.local_scales.insert(
            nodes[&source(MAP_IMAGE)].transform.clone(),
            [MAP_DIAMETER / 2048.0, MAP_DIAMETER / 2048.0, 1.0],
        );
        base.anchored_positions
            .insert(nodes[&source(MAP_IMAGE)].transform.clone(), [0.0; 2]);
        base.sprites.insert(
            nodes[&source(TREE)]
                .image
                .clone()
                .ok_or_else(|| invalid("tree template Image is absent"))?,
            atlas_sprite("-8404998493670798139"),
        );
        base.sprites.insert(
            nodes[&source(GENERIC)]
                .image
                .clone()
                .ok_or_else(|| invalid("generic template Image is absent"))?,
            atlas_sprite("7901072812560143050"),
        );
        renderer.prepare_bindings(&base, context)?;
        Ok(Self {
            renderer,
            nodes,
            base,
            zoom: DEFAULT_ZOOM,
        })
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        if zoom.is_finite() {
            self.zoom = zoom.clamp(3.0, 60.0);
        }
    }

    pub fn projection(&self, state: &MinimapState) -> MinimapProjection {
        MinimapProjection {
            center_xz: [state.player_position[0], state.player_position[2]],
            half_extent: shared::level::MAP_OVERVIEW_HALF_EXTENT,
            zoom: self.zoom,
        }
    }

    pub fn bindings(
        &self,
        rect: egui::Rect,
        viewport: CanvasViewport,
        state: &MinimapState,
        pixels_per_point: f32,
    ) -> Result<SourceUiBindings, UiError> {
        if state
            .player_position
            .iter()
            .chain([&state.yaw, &state.time_hours])
            .any(|value| !value.is_finite())
            || !pixels_per_point.is_finite()
            || pixels_per_point <= 0.0
            || !viewport.scale_factor.is_finite()
            || viewport.scale_factor <= 0.0
        {
            return Err(invalid(
                "native minimap received invalid player/canvas state",
            ));
        }
        let mut bindings = self.base.clone();
        let projection = self.projection(state);
        bindings
            .uv_rects
            .insert(source(MAP_RAW_IMAGE), projection.uv_rect());
        // Root anchors/pivot remain source top-right. The HUD editor controls its
        // containing envelope, including the 32.5-unit source header overhang.
        bindings.anchored_positions.insert(
            self.nodes[&source(ROOT)].transform.clone(),
            [
                (rect.right() * pixels_per_point - viewport.pixels[0] as f32)
                    / viewport.scale_factor,
                -rect.top() * pixels_per_point / viewport.scale_factor - HEADER_GUTTER,
            ],
        );
        bindings.text.insert(
            source(COORDINATES),
            format!(
                "({:.0},{:.0})",
                state.player_position[0], -state.player_position[2]
            ),
        );
        bindings
            .text
            .insert(source(HEIGHT), format!("{:.0}", state.player_position[1]));
        bindings.text.insert(
            source(GAME_TIME),
            shared::survival::format_time_of_day(state.time_hours),
        );
        let hours = state.time_hours.rem_euclid(24.0);
        let phase = if !(6.0..20.0).contains(&hours) {
            0
        } else if hours < 8.0 {
            3
        } else if hours >= 18.0 {
            1
        } else {
            2
        };
        for (index, id) in [
            "-821781803138336836",
            "8365677517520596295",
            "9066736818001988192",
            "-3991086821951747717",
        ]
        .into_iter()
        .enumerate()
        {
            bindings.visibility.insert(source(id), phase == index);
        }
        bindings
            .visibility
            .insert(source("-7822706158898638032"), phase == 0);
        bindings
            .visibility
            .insert(source("3799445044750627459"), phase != 0);
        // The cone widens upward from its player-facing tip; its view direction
        // is +Y, like the arrow. Layout's Y inversion maps this to north/-Z.
        self.instance(
            &mut bindings,
            PLAYER_CONE,
            "player-cone".into(),
            [0.0; 2],
            45.0,
            state.yaw,
            1.0,
            None,
        )?;
        self.instance(
            &mut bindings,
            PLAYER_ARROW,
            "player-arrow".into(),
            [0.0; 2],
            30.0,
            state.yaw,
            1.0,
            None,
        )?;
        if let Some(point) = state.waypoint
            && let Some(position) = projection.marker_position(point, 22.0, true)
        {
            self.instance(
                &mut bindings,
                WAYPOINT,
                "waypoint".into(),
                position,
                22.0,
                0.0,
                1.0,
                None,
            )?;
        }
        for marker in &state.markers {
            if !marker.yaw.is_finite() || !marker.intensity.is_finite() || marker.intensity <= 0.0 {
                continue;
            }
            let (template, span, clamp, sprite) = match marker.kind {
                MinimapMarkerKind::Tree => (TREE, 9.0, false, None),
                MinimapMarkerKind::Rock => (GENERIC, 9.0, false, None),
                MinimapMarkerKind::Structure => (
                    GENERIC,
                    13.0,
                    false,
                    Some(atlas_sprite("3924826172084765333")),
                ),
                MinimapMarkerKind::ToolCupboard => (
                    GENERIC,
                    18.0,
                    false,
                    Some(atlas_sprite("4445139762678479602")),
                ),
                MinimapMarkerKind::Enemy => (
                    GENERIC,
                    13.0,
                    false,
                    Some(atlas_sprite("7733724338977180109")),
                ),
                MinimapMarkerKind::Vehicle => (VEHICLE, 22.0, false, None),
                MinimapMarkerKind::Cargo => (CARGO, 22.0, false, None),
                MinimapMarkerKind::Footstep => (FOOTSTEP, 32.0, true, None),
                MinimapMarkerKind::Gunfire => (GUNFIRE, 32.0, true, None),
                MinimapMarkerKind::VehicleSound => (VEHICLE_SOUND, 26.0, true, None),
                MinimapMarkerKind::Airdrop => (AIRDROP, 22.0, true, None),
            };
            if let Some(position) = projection.marker_position(marker.world_xz, span, clamp) {
                self.instance(
                    &mut bindings,
                    template,
                    format!("marker:{}", marker.key),
                    position,
                    span,
                    marker.yaw,
                    marker.intensity.clamp(0.0, 1.0),
                    sprite,
                )?;
            }
        }
        Ok(bindings)
    }

    #[allow(clippy::too_many_arguments)]
    fn instance(
        &self,
        bindings: &mut SourceUiBindings,
        template: &str,
        key: String,
        position: [f32; 2],
        span: f32,
        angle: f32,
        alpha: f32,
        sprite: Option<SourceObject>,
    ) -> Result<(), UiError> {
        let source = source(template);
        let info = self
            .nodes
            .get(&source)
            .ok_or_else(|| invalid("native minimap template is absent"))?;
        let scale = span / info.size[0].max(info.size[1]);
        if !scale.is_finite() || scale <= 0.0 || info.image.is_none() {
            return Err(invalid(
                "native minimap template has no Image/valid source rect",
            ));
        }
        bindings.instances.push(SourceUiInstance {
            key,
            template: source,
            anchored_position: position,
            local_scale: [scale, scale, 1.0],
            local_rotation: (Quat::from_rotation_z(angle) * info.rotation).to_array(),
            color: Some([1.0, 1.0, 1.0, alpha]),
            sprite,
        });
        Ok(())
    }

    pub fn paint(
        &mut self,
        painter: &egui::Painter,
        rect: egui::Rect,
        source_scale: f32,
        state: &MinimapState,
    ) -> Result<PanelRenderReport, UiError> {
        let pixels_per_point = painter.ctx().pixels_per_point();
        let screen = painter.ctx().input(|input| input.viewport_rect());
        let viewport = CanvasViewport {
            pixels: [
                (screen.right() * pixels_per_point).round().max(1.0) as u32,
                (screen.bottom() * pixels_per_point).round().max(1.0) as u32,
            ],
            scale_factor: source_scale * pixels_per_point,
        };
        let bindings = self.bindings(rect, viewport, state, pixels_per_point)?;
        let mut report = self.renderer.paint(painter, viewport, 100.0, &bindings)?;
        report.limitations.retain(|detail| {
            !detail.starts_with("The caller supplies") && !detail.starts_with("Empty bindings")
        });
        report.limitations.push("Native HUD policy binds the retained BattleMapPanel Map_map texture to BattleMinimap, uses the shared provisional overview extent, player-centered UV zoom (default 15), and native marker/lifecycle bindings. Original minimap texture assignment and live Unity UV/zoom/scale behavior remain unverified.".into());
        Ok(report)
    }
}

fn activate(
    nodes: &BTreeMap<SourceObject, NodeInfo>,
    bindings: &mut SourceUiBindings,
    id: &str,
    active: bool,
) -> Result<(), UiError> {
    let key = source(id);
    let info = nodes
        .get(&key)
        .ok_or_else(|| invalid(format!("native minimap source node {id} is absent")))?;
    for ancestor in &info.ancestors {
        bindings.visibility.insert(ancestor.clone(), true);
    }
    bindings.visibility.insert(key, active);
    Ok(())
}

#[derive(Resource, Default)]
pub(crate) struct SourceMinimapHud {
    pub minimap: Option<NativeMinimap>,
    pub error: Option<String>,
}

pub(crate) fn prepare_native_minimap(
    mut contexts: EguiContexts,
    mut hud: ResMut<SourceMinimapHud>,
) {
    if hud.minimap.is_some() || hud.error.is_some() {
        return;
    }
    let Ok(context) = contexts.ctx_mut() else {
        return;
    };
    match NativeMinimap::load(
        &shared::level::asset_root().join(SOURCE_UI_PACKAGE),
        context,
    ) {
        Ok(minimap) => hud.minimap = Some(minimap),
        Err(error) => {
            warn!("Native source minimap could not load: {error}");
            hud.error = Some(error.to_string());
        }
    }
}

pub(crate) fn state_from_simulation(ctx: &crate::hud_painter::MinimapContext<'_>) -> MinimapState {
    use crate::hud_painter::SoundCueKind;
    use shared::survival::{ResourceKind, StructureKind, cargo_ship_transform};
    let mut state = MinimapState {
        player_position: ctx.player_position,
        yaw: ctx.yaw,
        time_hours: shared::survival::time_of_day_hours(ctx.tick),
        waypoint: ctx.marker,
        markers: Vec::with_capacity(
            ctx.nodes.len()
                + ctx.structures.len()
                + ctx.npcs.len()
                + ctx.vehicles.len()
                + ctx.sound_cues.len()
                + 1,
        ),
    };
    for node in ctx.nodes {
        state.markers.push(MinimapMarker {
            key: format!("resource:{}", node.id),
            kind: match node.kind {
                ResourceKind::Tree => MinimapMarkerKind::Tree,
                ResourceKind::Rock => MinimapMarkerKind::Rock,
            },
            world_xz: [node.position[0], node.position[2]],
            yaw: 0.0,
            intensity: if node.is_available() { 1.0 } else { 0.25 },
        });
    }
    for structure in ctx
        .structures
        .iter()
        .filter(|structure| structure.health > 0)
    {
        state.markers.push(MinimapMarker {
            key: format!("structure:{}", structure.id),
            kind: if structure.kind == StructureKind::ToolCupboard {
                MinimapMarkerKind::ToolCupboard
            } else {
                MinimapMarkerKind::Structure
            },
            world_xz: [structure.position[0], structure.position[2]],
            yaw: 0.0,
            intensity: 1.0,
        });
    }
    for npc in ctx.npcs.iter().filter(|npc| npc.is_alive()) {
        state.markers.push(MinimapMarker {
            key: format!("npc:{}", npc.id),
            kind: MinimapMarkerKind::Enemy,
            world_xz: [npc.position[0], npc.position[2]],
            yaw: npc.yaw,
            intensity: 1.0,
        });
    }
    for vehicle in ctx.vehicles.iter().filter(|vehicle| vehicle.health > 0) {
        state.markers.push(MinimapMarker {
            key: format!("vehicle:{}", vehicle.id),
            kind: MinimapMarkerKind::Vehicle,
            world_xz: [vehicle.position[0], vehicle.position[2]],
            yaw: vehicle.yaw,
            intensity: 1.0,
        });
    }
    let (cargo, yaw) = cargo_ship_transform(ctx.tick);
    state.markers.push(MinimapMarker {
        key: "cargo".into(),
        kind: MinimapMarkerKind::Cargo,
        world_xz: [cargo[0], cargo[2]],
        yaw,
        intensity: 1.0,
    });
    for (index, cue) in ctx.sound_cues.iter().enumerate() {
        state.markers.push(MinimapMarker {
            key: format!("sound:{index}"),
            kind: match cue.kind {
                SoundCueKind::Footstep => MinimapMarkerKind::Footstep,
                SoundCueKind::Gunfire => MinimapMarkerKind::Gunfire,
                SoundCueKind::Vehicle => MinimapMarkerKind::VehicleSound,
                SoundCueKind::Airdrop => MinimapMarkerKind::Airdrop,
            },
            world_xz: [cue.world_pos[0], cue.world_pos[2]],
            yaw: -map_projection::world_bearing([
                cue.world_pos[0] - ctx.player_position[0],
                cue.world_pos[2] - ctx.player_position[2],
            ]),
            intensity: cue.intensity,
        });
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::{HudLayout, WidgetId};

    fn assert_pair(actual: [f32; 2], expected: [f32; 2], tolerance: f32) {
        for axis in 0..2 {
            assert!(
                (actual[axis] - expected[axis]).abs() <= tolerance,
                "{actual:?} != {expected:?}"
            );
        }
    }

    #[test]
    fn crop_and_markers_agree_after_movement_zoom_and_outside_overview_edges() {
        let half_extent = shared::level::MAP_OVERVIEW_HALF_EXTENT;
        for center_xz in [
            [2250.0, -700.0],
            [2290.0, -760.0],
            [half_extent + 80.0, -half_extent - 50.0],
        ] {
            for zoom in [3.0, 15.0, 30.0, 60.0] {
                let projection = MinimapProjection {
                    center_xz,
                    half_extent,
                    zoom,
                };
                let point = [center_xz[0] + 45.0, center_xz[1] - 35.0];
                let [u, v, width, height] = projection.uv_rect();
                let uv = map_projection::world_to_overview_uv(point, half_extent);
                // Project the texture feature through the actual RawImage crop.
                let background = [
                    (uv[0] - u) / width * MAP_DIAMETER - MAP_DIAMETER * 0.5,
                    MAP_DIAMETER * 0.5 - (uv[1] - (1.0 - v - height)) / height * MAP_DIAMETER,
                ];
                assert_pair(background, projection.world_to_local(point), 0.004);
                assert_pair(projection.world_to_local(center_xz), [0.0; 2], 1e-6);
            }
        }
    }

    #[test]
    fn native_source_coordinates_put_north_up_and_east_right() {
        let projection = MinimapProjection {
            center_xz: [2000.0, -700.0],
            half_extent: 5360.0,
            zoom: 15.0,
        };
        let range = projection.world_range();
        assert_pair(
            projection.world_to_local([2000.0, -700.0 - range]),
            [0.0, 130.0],
            0.001,
        );
        assert_pair(
            projection.world_to_local([2000.0 + range, -700.0]),
            [130.0, 0.0],
            0.001,
        );
        assert_pair(
            projection.world_to_local([2000.0, -700.0 + range]),
            [0.0, -130.0],
            0.001,
        );
    }

    #[test]
    fn far_waypoints_clamp_without_moving_the_map_crop_or_creating_nan() {
        let projection = MinimapProjection {
            center_xz: [2250.0, -700.0],
            half_extent: 5360.0,
            zoom: 15.0,
        };
        let before = projection.uv_rect();
        assert_pair(
            projection
                .marker_position([2250.0, -700.0], 22.0, true)
                .expect("center marker"),
            [0.0; 2],
            1e-6,
        );
        assert!(
            projection
                .marker_position([8000.0, -9000.0], 22.0, false)
                .is_none()
        );
        let far = Vec2::from_array(
            projection
                .marker_position([8000.0, -9000.0], 22.0, true)
                .expect("perimeter marker"),
        );
        assert!((far.length() - 117.0).abs() < 0.001);
        assert!(far.x > 0.0 && far.y > 0.0);
        assert_eq!(projection.uv_rect(), before);
        assert!(
            projection
                .marker_position([f32::NAN, 0.0], 22.0, true)
                .is_none()
        );
    }

    #[test]
    fn simulation_binding_uses_tick_time_and_retires_dead_entities() {
        use crate::hud_painter::{MinimapContext, SoundCueKind, VisualSoundCue};
        use shared::{
            protocol::ServerTick,
            survival::{ResourceKind, ResourceNode, VehicleEntity, VehicleKind},
        };
        let nodes = [ResourceNode {
            id: 7,
            kind: ResourceKind::Tree,
            position: [20.0, 0.0, -30.0],
            health: 0,
            max_health: 100,
            respawn_at: None,
        }];
        let vehicles = [VehicleEntity {
            id: 3,
            kind: VehicleKind::Bicycle,
            position: [30.0, 0.0, -40.0],
            yaw: 0.0,
            health: 0,
            max_health: 100,
            fuel: 0,
            driver: None,
        }];
        let sounds = [VisualSoundCue {
            kind: SoundCueKind::Gunfire,
            world_pos: [0.0, 0.0, -100.0],
            age_ticks: 1,
            intensity: 0.4,
        }];
        let ctx = MinimapContext {
            nodes: &nodes,
            vehicles: &vehicles,
            sound_cues: &sounds,
            tick: ServerTick(1234),
            ..Default::default()
        };
        let state = state_from_simulation(&ctx);
        assert_eq!(
            state.time_hours,
            shared::survival::time_of_day_hours(ctx.tick)
        );
        assert!(
            !state
                .markers
                .iter()
                .any(|marker| marker.kind == MinimapMarkerKind::Vehicle)
        );
        let tree = state
            .markers
            .iter()
            .find(|marker| marker.kind == MinimapMarkerKind::Tree)
            .expect("depleted tree");
        assert_eq!(tree.intensity, 0.25);
        let sound = state
            .markers
            .iter()
            .find(|marker| marker.kind == MinimapMarkerKind::Gunfire)
            .expect("gunfire binding");
        assert_eq!(sound.intensity, 0.4);
        assert_eq!(sound.world_xz, [0.0, -100.0]);
    }

    fn native() -> (NativeMinimap, egui::Context) {
        let context = egui::Context::default();
        let native = NativeMinimap::load(
            &shared::level::asset_root().join(SOURCE_UI_PACKAGE),
            &context,
        )
        .expect("verified lite4.0 UI package");
        (native, context)
    }

    fn render(
        native: &mut NativeMinimap,
        context: &egui::Context,
        pixels: [u32; 2],
        dpi: f32,
        state: &MinimapState,
    ) -> PanelRenderReport {
        context.set_zoom_factor(1.0);
        let logical = pixels.map(|value| value as f32 / dpi);
        let placement =
            HudLayout::default().widgets[&WidgetId::Minimap].placement(WidgetId::Minimap, logical);
        let rect = egui::Rect::from_min_size(
            egui::Pos2::from(placement.position),
            egui::Vec2::from(placement.size),
        );
        let mut report = None;
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::from(logical),
            )),
            ..Default::default()
        };
        input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(dpi);
        context
            .run_ui(input, |ui| {
                report = Some(
                    native
                        .paint(ui.painter(), rect, placement.render_scale, state)
                        .expect("native HUD paint"),
                );
            })
            .drop_without_applying_deltas();
        let report = report.expect("render report");
        assert_eq!(report.viewport.pixels, pixels);
        report
    }

    fn graphic_center(report: &PanelRenderReport, instance: &str) -> Vec2 {
        let corners = report
            .graphics
            .iter()
            .find(|graphic| graphic.instance.as_deref() == Some(instance))
            .expect("runtime instance")
            .screen_corners
            .expect("source rect");
        (Vec2::from_array(corners[0]) + Vec2::from_array(corners[2])) * 0.5
    }

    #[test]
    #[ignore = "requires the extracted lite4.0 UI package"]
    fn source_hud_geometry_tracks_cardinal_headings_movement_and_dpi() {
        let (mut native, context) = native();
        let mut state = MinimapState {
            player_position: [2250.0, 80.0, -700.0],
            waypoint: Some([2500.0, -700.0]),
            markers: vec![MinimapMarker {
                key: "landmark".into(),
                kind: MinimapMarkerKind::Tree,
                world_xz: [2310.0, -780.0],
                yaw: 0.0,
                intensity: 1.0,
            }],
            ..Default::default()
        };
        for (pixels, dpi) in [([1920, 1080], 1.0), ([1280, 720], 1.0), ([3840, 2160], 2.0)] {
            for yaw in [
                0.0,
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::PI,
                -std::f32::consts::FRAC_PI_2,
            ] {
                state.yaw = yaw;
                let report = render(&mut native, &context, pixels, dpi, &state);
                assert!(
                    !report
                        .graphics
                        .iter()
                        .any(|graphic| graphic.status == "unsupported")
                );
                let player = graphic_center(&report, "player-arrow");
                let map = report
                    .graphics
                    .iter()
                    .find(|graphic| graphic.component == source(MAP_RAW_IMAGE))
                    .expect("bound RawImage");
                assert_eq!(map.status, "submitted");
                let map_corners = map.screen_corners.expect("map rect");
                let map_center =
                    (Vec2::from_array(map_corners[0]) + Vec2::from_array(map_corners[2])) * 0.5;
                assert_pair(player.to_array(), map_center.to_array(), 0.01);
                // The source arrow tip and the broad end of the view-cone art
                // face +Y. Both must face the player's movement direction.
                for instance in ["player-arrow", "player-cone"] {
                    let corners = report
                        .graphics
                        .iter()
                        .find(|graphic| graphic.instance.as_deref() == Some(instance))
                        .expect("player marker")
                        .screen_corners
                        .expect("player marker rect");
                    let center = graphic_center(&report, instance);
                    assert_pair(center.to_array(), player.to_array(), 0.01);
                    let heading = ((Vec2::from_array(corners[2]) + Vec2::from_array(corners[3]))
                        * 0.5
                        - center)
                        .normalize();
                    assert_pair(heading.to_array(), [-yaw.sin(), -yaw.cos()], 0.0001);
                }
                let marker = graphic_center(&report, "marker:landmark");
                let offset = native.projection(&state).world_to_local([2310.0, -780.0]);
                assert_pair(
                    (marker - player).to_array(),
                    [
                        offset[0] * report.viewport.scale_factor,
                        -offset[1] * report.viewport.scale_factor,
                    ],
                    0.01,
                );
            }
        }
        let before = render(&mut native, &context, [1920, 1080], 1.0, &state);
        state.player_position[0] += 40.0;
        let after = render(&mut native, &context, [1920, 1080], 1.0, &state);
        assert!(
            graphic_center(&after, "marker:landmark").x
                < graphic_center(&before, "marker:landmark").x
        );
        assert_ne!(before.bindings["uv_rects"], after.bindings["uv_rects"]);
    }

    #[test]
    #[ignore = "requires the extracted lite4.0 UI package"]
    fn source_instances_respect_parent_visibility_and_invalid_bindings_fail() {
        let (mut native, context) = native();
        let viewport = CanvasViewport {
            pixels: [1920, 1080],
            scale_factor: 1.0,
        };
        let rect =
            egui::Rect::from_min_size(egui::pos2(1540.0, 20.0), egui::Vec2::from(PANEL_SIZE));
        let state = MinimapState::default();
        let bindings = native
            .bindings(rect, viewport, &state, 1.0)
            .expect("bindings");
        let mut wrong = bindings.clone();
        let mut wrong_cab = source(MAP_RAW_IMAGE);
        wrong_cab.cab = "CAB-wrong".into();
        wrong.uv_rects.insert(wrong_cab, [0.0, 0.0, 1.0, 1.0]);
        assert!(native.renderer.prepare_bindings(&wrong, &context).is_err());
        wrong = bindings.clone();
        wrong
            .local_rotations
            .insert(source("-3798366094520507772"), [0.0; 4]);
        assert!(native.renderer.prepare_bindings(&wrong, &context).is_err());
        wrong = bindings.clone();
        wrong.instances.push(wrong.instances[0].clone());
        assert!(native.renderer.prepare_bindings(&wrong, &context).is_err());
        let mut hidden = bindings;
        hidden
            .visibility
            .insert(source("-4394325854588574010"), false);
        context
            .run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1920.0, 1080.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    let report = native
                        .renderer
                        .paint(ui.painter(), viewport, 100.0, &hidden)
                        .expect("hidden parent render");
                    let instances: Vec<_> = report
                        .graphics
                        .iter()
                        .filter(|graphic| graphic.instance.is_some())
                        .collect();
                    assert_eq!(instances.len(), 2);
                    assert!(instances.iter().all(|graphic| graphic.status == "inactive"));
                },
            )
            .drop_without_applying_deltas();
    }
}
