use bevy::prelude::*;
use shared::{
    ResourceNodeMarker, SimulationClock, SimulationMode, SimulationSurvival, SimulationWorld,
    protocol::{ClientControl, PlayerId, ServerTick},
    survival::{ActionOutcome, RecipeId, SlotIndex},
};

use crate::{
    editor::HudEditor, hud::HudLayout, hud_painter, online::Online, storage::StorageService,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum UiMode {
    Playing,
    Loading,
    Menu,
    Inventory,
    Map,
    Editor,
    Benchmark,
}

#[derive(Resource)]
pub(crate) struct Interface {
    pub mode: UiMode,
    pub editor: HudEditor,
    pub notice: Option<String>,
    pub selected_slot: Option<SlotIndex>,
    pub map_marker: Option<[f32; 2]>,
    pub swing_timer: f32,
    pub recoil_timer: f32,
    pub last_shot_trace: Option<([f32; 3], [f32; 3], f32)>,
    pub show_gm_panel: bool,
    pub gm_fly_mode: bool,
    pub gm_fog_enabled: bool,
    pub gm_hud_enabled: bool,
    pub volumetric_fog_steps: u32,
    pub loading_progress: f32,
    pub loading_timer: f32,
    pub sound_cues: Vec<hud_painter::VisualSoundCue>,
}

impl Interface {
    pub fn new(storage: &dyn StorageService, benchmark: bool) -> Self {
        let (layout, notice) = if benchmark {
            // The capture scene and HUD must be independent of user settings.
            (HudLayout::default(), None)
        } else {
            match storage.load_hud() {
                Ok(loaded) => (loaded.layout, loaded.notice),
                Err(error) => (HudLayout::default(), Some(error.to_string())),
            }
        };
        Self {
            mode: if benchmark {
                UiMode::Benchmark
            } else {
                UiMode::Loading
            },
            editor: HudEditor::new(layout),
            notice,
            selected_slot: None,
            map_marker: None,
            swing_timer: 0.0,
            recoil_timer: 0.0,
            last_shot_trace: None,
            show_gm_panel: false,
            gm_fly_mode: false,
            gm_fog_enabled: true,
            gm_hud_enabled: true,
            volumetric_fog_steps: 24,
            loading_progress: 0.0,
            loading_timer: 0.0,
            sound_cues: Vec::new(),
        }
    }

    pub fn add_sound_cue(
        &mut self,
        kind: hud_painter::SoundCueKind,
        world_pos: [f32; 3],
        intensity: f32,
    ) {
        self.sound_cues.push(hud_painter::VisualSoundCue {
            kind,
            world_pos,
            age_ticks: 0,
            intensity,
        });
        if self.sound_cues.len() > 32 {
            self.sound_cues.remove(0);
        }
    }

    pub fn tick_sound_cues(&mut self) {
        self.sound_cues.retain_mut(|cue| {
            cue.age_ticks += 1;
            cue.intensity *= 0.94;
            cue.age_ticks < 90 && cue.intensity > 0.05
        });
    }

    pub fn begin_editing(&mut self) {
        self.editor.begin();
        self.mode = UiMode::Editor;
    }

    pub fn toggle_inventory(&mut self) {
        match self.mode {
            UiMode::Playing | UiMode::Map => self.mode = UiMode::Inventory,
            UiMode::Inventory => {
                self.selected_slot = None;
                self.mode = UiMode::Playing;
            }
            _ => {}
        }
    }

    pub fn toggle_map(&mut self) {
        match self.mode {
            UiMode::Playing | UiMode::Inventory => {
                self.selected_slot = None;
                self.mode = UiMode::Map;
            }
            UiMode::Map => self.mode = UiMode::Playing,
            _ => {}
        }
    }

    pub fn escape(&mut self) {
        if self.show_gm_panel {
            self.show_gm_panel = false;
            return;
        }
        match self.mode {
            UiMode::Playing => self.mode = UiMode::Menu,
            UiMode::Loading => self.mode = UiMode::Playing,
            UiMode::Menu => self.mode = UiMode::Playing,
            UiMode::Inventory | UiMode::Map => {
                self.selected_slot = None;
                self.mode = UiMode::Playing;
            }
            UiMode::Editor => {
                self.editor.cancel();
                self.mode = UiMode::Menu;
            }
            UiMode::Benchmark => {}
        }
    }
}

pub(crate) fn select_hotbar_slot(
    slot: u8,
    mode: &SimulationMode,
    survival: &mut SimulationSurvival,
    online: &Online,
) {
    if let Some(player) = survival.0.player_mut(PlayerId(0)) {
        player.inventory.select_slot(slot);
    }
    if *mode == SimulationMode::Network {
        online.send_control(ClientControl::SelectSlot(slot));
    }
}

pub(crate) fn cycle_structure_kind(
    mode: &SimulationMode,
    survival: &mut SimulationSurvival,
    online: &Online,
) {
    if let Some(player) = survival.0.player_mut(PlayerId(0)) {
        player.cycle_structure();
    }
    if *mode == SimulationMode::Network {
        online.send_control(ClientControl::CycleStructure);
    }
}

pub(crate) fn rotate_structure_socket(
    mode: &SimulationMode,
    survival: &mut SimulationSurvival,
    online: &Online,
) {
    if let Some(player) = survival.0.player_mut(PlayerId(0)) {
        player.rotate_structure();
    }
    if *mode == SimulationMode::Network {
        online.send_control(ClientControl::RotateStructure);
    }
}

pub(crate) fn toggle_mount_vehicle(
    mode: &SimulationMode,
    survival: &mut SimulationSurvival,
    online: &Online,
    position: [f32; 3],
    yaw: f32,
    pitch: f32,
) {
    if *mode == SimulationMode::Network {
        online.send_control(ClientControl::MountOrDismount { yaw, pitch });
        return;
    }
    if let Err(err) = survival
        .0
        .toggle_mount_vehicle(PlayerId(0), position, yaw, pitch)
        && let Some(player) = survival.0.player_mut(PlayerId(0))
    {
        player.notice = Some(err.into());
    }
}

pub(crate) fn move_inventory_slot(
    from: SlotIndex,
    to: SlotIndex,
    mode: &SimulationMode,
    survival: &mut SimulationSurvival,
    online: &Online,
) {
    if let Some(player) = survival.0.player_mut(PlayerId(0)) {
        player.inventory.move_slot(from, to);
    }
    if *mode == SimulationMode::Network {
        online.send_control(ClientControl::MoveSlot { from, to });
    }
}

pub(crate) fn craft_recipe(
    recipe: RecipeId,
    mode: &SimulationMode,
    survival: &mut SimulationSurvival,
    online: &Online,
) {
    if *mode == SimulationMode::Network {
        online.send_control(ClientControl::Craft(recipe));
        return;
    }
    if let Some(player) = survival.0.player_mut(PlayerId(0)) {
        match player.inventory.craft(recipe) {
            Ok(item) => {
                player.notice = Some(format!("Скрафчено: {}", item.label()));
            }
            Err(err) => {
                player.notice = Some(err.into());
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn trigger_primary_action(
    interface: &mut Interface,
    mode: &SimulationMode,
    clock: &SimulationClock,
    physics: &mut SimulationWorld,
    survival: &mut SimulationSurvival,
    online: &Online,
    position: [f32; 3],
    yaw: f32,
    pitch: f32,
    nodes: &mut Query<&mut ResourceNodeMarker>,
) {
    interface.swing_timer = 0.28;
    if *mode == SimulationMode::Network {
        if let Some(player) = survival.0.player(PlayerId(0))
            && let Some(active) = player.inventory.active_item()
            && let Some((ammo_kind, _, _, max_range)) = active.kind.ranged_spec()
            && player.inventory.count_item(ammo_kind) > 0
        {
            let dir = shared::survival::look_direction(yaw, pitch);
            let start = [position[0], position[1] + 0.56, position[2]];
            let end = [
                start[0] + dir[0] * max_range.min(45.0),
                (start[1] + dir[1] * max_range.min(45.0)).max(0.05),
                start[2] + dir[2] * max_range.min(45.0),
            ];
            interface.last_shot_trace = Some((start, end, 0.22));
            interface.recoil_timer = 0.14;
        }
        online.send_control(ClientControl::PrimaryAction { yaw, pitch });
        return;
    }
    match survival.0.primary_action(
        PlayerId(0),
        ServerTick(clock.tick.max(1)),
        position,
        yaw,
        pitch,
        &[],
    ) {
        Ok(ActionOutcome::Harvested { node_id, .. }) => {
            if let Some(updated) = survival.0.nodes.iter().find(|n| n.id == node_id).copied() {
                for mut marker in nodes.iter_mut() {
                    if marker.0.id == node_id {
                        marker.0 = updated;
                    }
                }
            }
        }
        Ok(ActionOutcome::BuiltStructure(placed)) => {
            physics.0.insert_structure(placed);
        }
        Ok(ActionOutcome::DamagedStructure {
            destroyed: true, ..
        }) => {
            physics.0.sync_structures(&survival.0.structures);
        }
        Ok(ActionOutcome::FiredWeapon { start, end, .. }) => {
            interface.last_shot_trace = Some((start, end, 0.22));
            interface.recoil_timer = 0.14;
        }
        Ok(_) => {}
        Err(err) => {
            if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                player.notice = Some(err.into());
            }
        }
    }
}

#[derive(Resource)]
pub(crate) struct PlatformStorage(pub Box<dyn StorageService>);

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum LookedTargetKind {
    Structure(u32),
    ResourceNode(u16),
    Vehicle(u16),
    Npc(u16),
    Scenic(bevy::ecs::entity::Entity),
    Hotspot(&'static str),
    Terrain([f32; 3]),
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct LookedObjectInfo {
    pub title: String,
    pub detail: String,
    pub distance: f32,
    pub hit_coords: [f32; 3],
    pub interact_prompt: Option<String>,
    pub target_kind: Option<LookedTargetKind>,
}

#[derive(Resource, Default)]
pub(crate) struct Telemetry {
    pub fps: f64,
    pub mean_ms: f64,
    pub p95_ms: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub player_position: [f32; 3],
    pub player_velocity: [f32; 3],
    pub gpu: String,
    pub backend: String,
    pub benchmark_status: String,
    pub looked_object: Option<LookedObjectInfo>,
}

pub fn delete_looked_object(
    looked: &LookedObjectInfo,
    survival: &mut SimulationSurvival,
    physics: &mut SimulationWorld,
    commands: Option<&mut Commands>,
) -> Option<String> {
    match looked.target_kind {
        Some(LookedTargetKind::Structure(id)) => {
            if let Some(pos) = survival.0.structures.iter().position(|s| s.id == id) {
                let st = survival.0.structures.remove(pos);
                physics.0.sync_structures(&survival.0.structures);
                return Some(format!(
                    "🗑️ Споруду видалено: {} [{}]",
                    st.kind.label(),
                    st.tier.label()
                ));
            }
        }
        Some(LookedTargetKind::ResourceNode(id)) => {
            if let Some(node) = survival.0.nodes.iter_mut().find(|n| n.id == id) {
                node.health = 0;
                node.respawn_at = None;
                return Some(format!(
                    "🗑️ Ресурсний об'єкт видалено: {}",
                    node.kind.label()
                ));
            }
        }
        Some(LookedTargetKind::Vehicle(id)) => {
            if let Some(pos) = survival.0.vehicles.iter().position(|v| v.id == id) {
                let v = survival.0.vehicles.remove(pos);
                return Some(format!("🗑️ Транспорт видалено: {}", v.kind.label()));
            }
        }
        Some(LookedTargetKind::Npc(id)) => {
            if let Some(pos) = survival.0.npcs.iter().position(|n| n.id == id) {
                let n = survival.0.npcs.remove(pos);
                return Some(format!("🗑️ NPC видалено: {}", n.kind.label()));
            }
        }
        Some(LookedTargetKind::Scenic(entity)) => {
            if let Some(cmd) = commands {
                cmd.entity(entity).despawn();
                return Some("🗑️ Декоративний об'єкт сцени видалено!".to_string());
            }
        }
        _ => {
            let hit = looked.hit_coords;
            if let Some(pos) = survival.0.structures.iter().position(|s| {
                let dx = s.position[0] - hit[0];
                let dy = s.position[1] - hit[1];
                let dz = s.position[2] - hit[2];
                dx * dx + dy * dy + dz * dz < 3.5 * 3.5
            }) {
                let st = survival.0.structures.remove(pos);
                physics.0.sync_structures(&survival.0.structures);
                return Some(format!("🗑️ Споруду видалено: {}", st.kind.label()));
            }
            if let Some(node) = survival.0.nodes.iter_mut().find(|n| {
                let dx = n.position[0] - hit[0];
                let dy = n.position[1] - hit[1];
                let dz = n.position[2] - hit[2];
                dx * dx + dy * dy + dz * dz < 3.5 * 3.5
            }) {
                node.health = 0;
                node.respawn_at = None;
                return Some(format!(
                    "🗑️ Ресурсний вузол видалено: {}",
                    node.kind.label()
                ));
            }
        }
    }
    None
}
