use bevy::{
    audio::{AudioPlayer, PlaybackSettings},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiTextureHandle, egui};
use shared::{
    CharacterMotion, LocalPlayer, ResourceNodeMarker, SimulationClock, SimulationMode,
    SimulationPose, SimulationSurvival, SimulationWorld,
    protocol::{ClientControl, PlayerId, ServerTick},
    survival::{
        ActionOutcome, BACKPACK_SLOTS, HOTBAR_SLOTS, ItemKind, PlayerSurvival, RecipeId, SlotIndex,
        format_time_of_day, preview_socket_placement, socket_edge_label,
    },
};

use crate::{
    application::{AudioAssets, CameraPerspective, MapTextures, UiSpriteAssets},
    editor::HudEditor,
    hud::{HudLayout, HudPreset, WidgetAnchor, WidgetId},
    hud_painter::{self, MinimapContext, UiTexturesIds},
    online::Online,
    storage::StorageService,
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
    pub gm_active_tab: u8,
    pub gm_target_coords: [f32; 3],
    pub gm_selected_item: shared::survival::ItemKind,
    pub gm_spawn_amount: u16,
    pub gm_fly_mode: bool,
    pub gm_fog_enabled: bool,
    pub gm_hud_enabled: bool,
    pub volumetric_fog_steps: u32,
    pub loading_progress: f32,
    pub loading_tip_idx: usize,
    pub loading_timer: f32,
    pub sound_cues: Vec<hud_painter::VisualSoundCue>,
    theme_initialized: bool,
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
            gm_active_tab: 0,
            gm_target_coords: [0.0, 10.0, 0.0],
            gm_selected_item: shared::survival::ItemKind::TeleportGun,
            gm_spawn_amount: 1,
            gm_fly_mode: false,
            gm_fog_enabled: true,
            gm_hud_enabled: true,
            volumetric_fog_steps: 24,
            loading_progress: 0.0,
            loading_tip_idx: 0,
            loading_timer: 0.0,
            sound_cues: Vec::new(),
            theme_initialized: false,
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

#[allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::collapsible_if
)]
pub(crate) fn draw_interface(
    mut contexts: EguiContexts,
    mut interface: ResMut<Interface>,
    storage: Res<PlatformStorage>,
    telemetry: Res<Telemetry>,
    mode: Res<SimulationMode>,
    mut clock: ResMut<SimulationClock>,
    mut physics: ResMut<SimulationWorld>,
    mut survival: ResMut<SimulationSurvival>,
    mut online: ResMut<Online>,
    mut local_player: Query<
        (&mut SimulationPose, &mut CharacterMotion, &mut Transform),
        With<LocalPlayer>,
    >,
    mut nodes: Query<&mut ResourceNodeMarker>,
    mut exit: MessageWriter<AppExit>,
    mut commands: Commands,
    (audio_assets, map_textures, ui_sprite_assets, perspective, mut windows): (
        Option<Res<AudioAssets>>,
        Option<Res<MapTextures>>,
        Option<Res<UiSpriteAssets>>,
        Option<ResMut<CameraPerspective>>,
        Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    ),
) {
    let (
        map_tex_id,
        marker_tex_id,
        cargo_tex_id,
        vehicle_tex_id,
        loading_bg_id,
        bar_bg_id,
        bar_fill_id,
    ) = if let Some(texs) = &map_textures {
        (
            Some(contexts.add_image(EguiTextureHandle::Strong(texs.overview.clone()))),
            Some(contexts.add_image(EguiTextureHandle::Strong(texs.marker.clone()))),
            Some(contexts.add_image(EguiTextureHandle::Strong(texs.cargo_ship.clone()))),
            Some(contexts.add_image(EguiTextureHandle::Strong(texs.vehicle.clone()))),
            Some(contexts.add_image(EguiTextureHandle::Strong(texs.loading_bg.clone()))),
            Some(contexts.add_image(EguiTextureHandle::Strong(texs.loading_bar_bg.clone()))),
            Some(contexts.add_image(EguiTextureHandle::Strong(texs.loading_bar_fill.clone()))),
        )
    } else {
        (None, None, None, None, None, None, None)
    };

    let ui_tex_ids = if let Some(ui_sprites) = &ui_sprite_assets {
        UiTexturesIds {
            minimap_frame: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.minimap_frame.clone()))),
            minimap_bg: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.minimap_bg.clone()))),
            player_arrow: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.player_arrow.clone()))),
            player_cone: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.player_cone.clone()))),
            day_night_bg: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.day_night_bg.clone()))),
            sun_icon: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.sun_icon.clone()))),
            moon_icon: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.moon_icon.clone()))),
            coord_bg: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.coord_bg.clone()))),
            bar_bg: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.bar_bg.clone()))),
            bar_fill_hp: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.bar_fill_hp.clone()))),
            bar_fill_energy: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.bar_fill_energy.clone()))),
            hotbar_slot: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.hotbar_slot.clone()))),
            hotbar_select: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.hotbar_select.clone()))),
            btn_fire_bg: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.btn_fire_bg.clone()))),
            btn_fire_icon: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.btn_fire_icon.clone()))),
            btn_aim_bg: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.btn_aim_bg.clone()))),
            btn_aim_icon: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.btn_aim_icon.clone()))),
            btn_jump: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.btn_jump.clone()))),
            btn_crouch: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.btn_crouch.clone()))),
            btn_reload: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.btn_reload.clone()))),
            joystick_bg: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.joystick_bg.clone()))),
            map_close: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.map_close.clone()))),
            map_zoom_in: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.map_zoom_in.clone()))),
            map_zoom_out: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.map_zoom_out.clone()))),
            sound_foot: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.sound_foot.clone()))),
            sound_car: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.sound_car.clone()))),
            sound_gun: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.sound_gun.clone()))),
            airdrop_icon: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.airdrop_icon.clone()))),
            cargo_icon: Some(contexts.add_image(EguiTextureHandle::Strong(ui_sprites.cargo_icon.clone()))),
        }
    } else {
        UiTexturesIds::default()
    };

    interface.tick_sound_cues();

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if !interface.theme_initialized {
        apply_theme(ctx);
        interface.theme_initialized = true;
    }

    if interface.mode == UiMode::Loading {
        draw_loading(ctx, &mut interface, loading_bg_id, bar_bg_id, bar_fill_id);
        return;
    }

    let player_survival = survival.0.player(PlayerId(0)).cloned().unwrap_or_default();

    if interface.gm_hud_enabled {
        draw_widgets(
            ctx,
            &mut interface,
            &telemetry,
            &player_survival,
            &mode,
            &clock,
            &mut physics,
            &mut survival,
            &online,
            &local_player,
            &mut nodes,
            map_tex_id,
            marker_tex_id,
            ui_tex_ids,
        );
        if matches!(interface.mode, UiMode::Playing | UiMode::Benchmark) {
            hud_painter::crosshair(ctx);
        }

        if interface.mode == UiMode::Playing {
            let is_teleport_gun = player_survival
                .inventory
                .active_item()
                .is_some_and(|s| s.kind == shared::survival::ItemKind::TeleportGun);
            if let Some(ref looked) = telemetry.looked_object {
                draw_looked_object_hud(ctx, looked, is_teleport_gun);
            }
            draw_playing_overlays(
                ctx,
                &telemetry,
                &clock,
                &survival,
                &online,
                &player_survival,
            );
        }

        egui::Area::new(egui::Id::new("frame-telemetry"))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 12.0))
            .interactable(false)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{:.0} FPS  ·  {:.2} мс  ·  p95 {:.2} мс",
                        telemetry.fps, telemetry.mean_ms, telemetry.p95_ms
                    ))
                    .small()
                    .color(egui::Color32::from_rgb(187, 207, 212)),
                );
                if online.connected {
                    ui.small(format!(
                        "Мережа: {}/{} · {:.0} мс",
                        online.population,
                        online.capacity,
                        online.rtt.as_secs_f64() * 1000.0
                    ));
                }
            });
    }

    match interface.mode {
        UiMode::Menu => {
            let scr = ctx.content_rect();
            egui::Window::new("MIDNIGHT / SURVIVAL")
                .id(egui::Id::new("main-menu"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .default_width(380.0)
                .max_width((scr.width() - 32.0).max(280.0))
                .max_height((scr.height() - 32.0).max(240.0))
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.heading("Острів виживання (Фаза 5: Повний реліз)");
                    ui.label("Біоми, Будівництво, Радтаун (РТ), Транспорт (F), Боси та Ядерний удар.");
                    ui.add_space(10.0);
                    ui.separator();
                    ui.label("Гра в мережі");
                    if online.connected {
                        ui.label(format!(
                            "Гравців: {}/{} · Пінг: {:.0} мс",
                            online.population,
                            online.capacity,
                            online.rtt.as_secs_f64() * 1000.0
                        ));
                        if ui.button("Від'єднатися").clicked() {
                            online.disconnect();
                        }
                    } else if online.connecting {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Підключення…");
                        });
                        if ui.button("Скасувати підключення").clicked() {
                            online.disconnect();
                        }
                    } else {
                        ui.add(
                            egui::TextEdit::singleline(&mut online.address)
                                .hint_text("Адреса сервера: порт"),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut online.code)
                                .password(true)
                                .hint_text("Код доступу MS1-…"),
                        );
                        if ui.button("Підключитися").clicked() {
                            online.connect();
                        }
                    }
                    ui.small(&online.status);
                    ui.separator();
                    if ui
                        .add_sized([ui.available_width(), 40.0], egui::Button::new("Грати"))
                        .clicked()
                    {
                        interface.mode = UiMode::Loading;
                        interface.loading_progress = 0.0;
                    }
                    if ui
                        .add_sized(
                            [ui.available_width(), 36.0],
                            egui::Button::new("🎬 Екран завантаження (BattleLoadingPanel)"),
                        )
                        .clicked()
                    {
                        interface.mode = UiMode::Loading;
                        interface.loading_progress = 0.0;
                    }
                    let is_fullscreen = windows
                        .single()
                        .ok()
                        .is_some_and(|w| !matches!(w.mode, bevy::window::WindowMode::Windowed));
                    let fs_text = if is_fullscreen {
                        "🪟 Віконний режим / Windowed (F11)"
                    } else {
                        "🖥️ Повноекранний режим / Fullscreen (F11)"
                    };
                    if ui
                        .add_sized([ui.available_width(), 36.0], egui::Button::new(fs_text))
                        .clicked()
                        && let Ok(mut win) = windows.single_mut()
                    {
                        win.mode = if is_fullscreen {
                            bevy::window::WindowMode::Windowed
                        } else {
                            bevy::window::WindowMode::BorderlessFullscreen(
                                bevy::window::MonitorSelection::Current,
                            )
                        };
                    }
                    if ui
                        .add_sized(
                            [ui.available_width(), 36.0],
                            egui::Button::new("Інвентар та Крафт (Tab)"),
                        )
                        .clicked()
                    {
                        interface.mode = UiMode::Inventory;
                    }
                    if ui
                        .add_sized(
                            [ui.available_width(), 36.0],
                            egui::Button::new("Тактична карта острова (M)"),
                        )
                        .clicked()
                    {
                        interface.mode = UiMode::Map;
                    }
                    if ui
                        .add_sized(
                            [ui.available_width(), 36.0],
                            egui::Button::new("Редактор HUD (F2)"),
                        )
                        .clicked()
                    {
                        interface.begin_editing();
                    }
                    if ui
                        .add_sized(
                            [ui.available_width(), 36.0],
                            egui::Button::new("⚡ GM Admin Panel (F1 / ~)"),
                        )
                        .clicked()
                    {
                        interface.show_gm_panel = true;
                        interface.mode = UiMode::Playing;
                    }
                    if ui
                        .add_sized([ui.available_width(), 32.0], egui::Button::new("Вийти"))
                        .clicked()
                    {
                        exit.write(AppExit::Success);
                    }
                    ui.separator();
                    ui.label(
                        "F11 / Alt+Enter — повноекранний режим  ·  WASD — рух  ·  Миша — огляд\nЛКМ — атака / дія / видалення (пушка)  ·  F — транспорт  ·  Esc — меню\n1–6 / Колесо — слот хотбару  ·  Tab / I — інвентар  ·  M — карта острова  ·  F2 — HUD",
                    );
                    if let Some(notice) = &interface.notice {
                        ui.separator();
                        ui.colored_label(egui::Color32::from_rgb(237, 190, 110), notice);
                    }
                });
        }
        UiMode::Inventory => draw_inventory(
            ctx,
            &mut interface,
            &mode,
            &clock,
            &mut physics,
            &mut survival,
            &online,
            &local_player,
            &mut nodes,
        ),
        UiMode::Map => draw_map(
            ctx,
            &mut interface,
            &telemetry,
            &clock,
            &survival,
            &player_survival,
            map_tex_id,
            marker_tex_id,
            cargo_tex_id,
            vehicle_tex_id,
            ui_tex_ids,
            audio_assets.as_deref(),
            &mut commands,
            perspective,
        ),
        UiMode::Editor => draw_editor(ctx, &mut interface, storage.0.as_ref()),
        UiMode::Benchmark => {
            egui::Window::new("Замір продуктивності")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 42.0))
                .show(ctx, |ui| {
                    ui.label(&telemetry.benchmark_status);
                    ui.small(format!("{} · {}", telemetry.gpu, telemetry.backend));
                    ui.small("Камера зафіксована. Залиште вікно активним. Esc — скасувати.");
                });
        }
        UiMode::Loading | UiMode::Playing => {}
    }

    if interface.show_gm_panel {
        draw_gm_panel(
            ctx,
            &mut interface,
            &mut clock,
            &mut physics,
            &mut survival,
            &mut local_player,
            &telemetry,
            &mut windows,
            &mut commands,
        );
    }
}

fn draw_playing_overlays(
    ctx: &egui::Context,
    telemetry: &Telemetry,
    clock: &SimulationClock,
    survival: &SimulationSurvival,
    online: &Online,
    player_survival: &PlayerSurvival,
) {
    let env = survival
        .0
        .environment_at(ServerTick(clock.tick), telemetry.player_position);
    let active_kind = player_survival.inventory.active_item().map(|s| s.kind);
    let reach = active_kind.map(|k| k.reach_meters()).unwrap_or(2.6);

    if active_kind == Some(ItemKind::BuildingPlan) {
        let kind = player_survival.selected_structure;
        let (pos, rot) = preview_socket_placement(
            telemetry.player_position,
            telemetry.yaw,
            telemetry.pitch,
            kind,
            player_survival.build_rotation,
            &survival.0.structures,
        );
        let allowed = survival
            .0
            .has_building_privilege(online.local_player_id(), pos);
        let prompt = if allowed {
            format!(
                "Будівництво: {} ({} Деревина) [Сокет: {}] — ЛКМ: збудувати  ·  Q / ПКМ: поворот 90°  ·  R: форма",
                kind.label(),
                kind.build_cost_wood(),
                socket_edge_label(rot)
            )
        } else {
            format!(
                "Будівництво заблоковано чужою Шафою (TC)  ·  Q / ПКМ: поворот  ·  R: форма ({})",
                kind.label()
            )
        };
        let color = if allowed {
            egui::Color32::from_rgb(156, 231, 212)
        } else {
            egui::Color32::from_rgb(240, 133, 127)
        };
        egui::Area::new(egui::Id::new("building-plan-prompt"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 28.0))
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_unmultiplied(14, 23, 29, 215))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(prompt).color(color).size(13.5));
                    });
            });
    } else if active_kind == Some(ItemKind::Hammer)
        && let Some(structure) = survival.0.targeted_structure(
            telemetry.player_position,
            telemetry.yaw,
            telemetry.pitch,
            reach,
        )
    {
        let action_hint = if structure.health < structure.max_health {
            "ЛКМ: ремонтувати (10 Деревина)".to_string()
        } else if let Some((next_tier, mat, cost)) = structure.tier.next_upgrade() {
            format!(
                "ЛКМ: покращити до {} ({}: {})",
                next_tier.label(),
                mat.label(),
                cost
            )
        } else {
            "Максимальний рівень міцності".to_string()
        };
        egui::Area::new(egui::Id::new("hammer-target-prompt"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 28.0))
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_unmultiplied(14, 23, 29, 215))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} [{}] ({}/{}) — {}",
                                structure.kind.label(),
                                structure.tier.label(),
                                structure.health,
                                structure.max_health,
                                action_hint
                            ))
                            .color(egui::Color32::from_rgb(156, 231, 212))
                            .size(13.5),
                        );
                    });
            });
    } else if let Some(hotspot) = survival.0.targeted_monument_hotspot(
        telemetry.player_position,
        telemetry.yaw,
        telemetry.pitch,
        3.6,
    ) {
        let pwr = if survival.0.monument.is_powered() {
            "Живлення РТ: УВІМК"
        } else {
            "Живлення РТ: ВИМК"
        };
        let door = if survival.0.monument.is_door_open() {
            "Бункер: ВІДЧИНЕНО"
        } else {
            "Бункер: ЗАЧИНЕНО"
        };
        egui::Area::new(egui::Id::new("monument-hotspot-prompt"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 28.0))
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_unmultiplied(14, 23, 29, 225))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "[ЛКМ / E] {}  ·  {pwr}  ·  {door}",
                                hotspot.label()
                            ))
                            .color(egui::Color32::from_rgb(206, 248, 92))
                            .size(13.5),
                        );
                    });
            });
    } else if let Some(npc) = survival.0.targeted_npc(
        telemetry.player_position,
        telemetry.yaw,
        telemetry.pitch,
        28.0,
    ) {
        egui::Area::new(egui::Id::new("target-npc-prompt"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 28.0))
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_unmultiplied(28, 16, 18, 225))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "🎯 {} ({}/{} ХП) — ЛКМ: вогонь / атака",
                                npc.kind.label(),
                                npc.health,
                                npc.max_health
                            ))
                            .color(egui::Color32::from_rgb(252, 132, 118))
                            .size(13.5),
                        );
                    });
            });
    } else if player_survival.mounted_vehicle.is_none()
        && let Some(vehicle) = survival.0.targeted_vehicle(
            telemetry.player_position,
            telemetry.yaw,
            telemetry.pitch,
            3.8,
        )
    {
        let fuel_txt = if vehicle.kind.uses_fuel() {
            format!("Паливо: {} од.", vehicle.fuel)
        } else {
            "Без палива".to_string()
        };
        egui::Area::new(egui::Id::new("target-vehicle-prompt"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 28.0))
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_unmultiplied(14, 26, 34, 225))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "[F] Сісти за кермо: {} ({}/{} ХП · {fuel_txt})",
                                vehicle.kind.label(),
                                vehicle.health,
                                vehicle.max_health
                            ))
                            .color(egui::Color32::from_rgb(112, 232, 252))
                            .size(13.5),
                        );
                    });
            });
    } else if let Some(node) = survival.0.targeted_node(
        telemetry.player_position,
        telemetry.yaw,
        telemetry.pitch,
        reach,
    ) {
        egui::Area::new(egui::Id::new("target-node-prompt"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 28.0))
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_unmultiplied(14, 23, 29, 215))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} ({}/{}) — ЛКМ: добувати {}",
                                node.kind.label(),
                                node.health,
                                node.max_health,
                                node.kind.item().label()
                            ))
                            .color(egui::Color32::from_rgb(156, 231, 212))
                            .size(13.5),
                        );
                    });
            });
    }

    if survival.0.nuclear.is_active() {
        let secs = (survival.0.nuclear.ticks_remaining as f32 / 60.0).ceil();
        let (nuke_text, nuke_bg, nuke_fg) = match survival.0.nuclear.phase {
            shared::survival::NuclearPhase::SirenWarning => (
                format!(
                    "☢ СИРЕНА ЯДЕРНОЇ ТРИВОГИ! Детонація через {secs:.0}с — покиньте червону зону на карті (M)! ☢"
                ),
                egui::Color32::from_rgba_unmultiplied(142, 18, 18, 235),
                egui::Color32::from_rgb(255, 224, 188),
            ),
            shared::survival::NuclearPhase::DetonationShockwave => (
                format!(
                    "☢ ЯДЕРНИЙ ВИБУХ ТА УДАРНА ХВИЛЯ ({secs:.0}с)! Гриб у небі над островом! ☢"
                ),
                egui::Color32::from_rgba_unmultiplied(186, 68, 14, 240),
                egui::Color32::from_rgb(255, 246, 206),
            ),
            shared::survival::NuclearPhase::Fallout => (
                format!(
                    "☢ РАДІОАКТИВНИЙ ПОПІЛ ({secs:.0}с): у кратері ядерного вибуху відкрито Елітний контейнер! ☢"
                ),
                egui::Color32::from_rgba_unmultiplied(64, 78, 16, 230),
                egui::Color32::from_rgb(222, 255, 112),
            ),
            shared::survival::NuclearPhase::Inactive => (
                String::new(),
                egui::Color32::TRANSPARENT,
                egui::Color32::WHITE,
            ),
        };
        egui::Area::new(egui::Id::new("nuclear-event-banner"))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 48.0))
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).fill(nuke_bg).show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(nuke_text)
                            .strong()
                            .size(14.5)
                            .color(nuke_fg),
                    );
                });
            });
    }

    if let Some(vid) = player_survival.mounted_vehicle
        && let Some(v) = survival.0.vehicles.iter().find(|v| v.id == vid)
    {
        let fuel_info = if v.kind.uses_fuel() {
            format!("Паливо: {} од.", v.fuel)
        } else {
            "Педальний привід".to_string()
        };
        egui::Area::new(egui::Id::new("vehicle-dashboard-banner"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -114.0))
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_unmultiplied(14, 28, 36, 225))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "🚙 Кермування: {}  ·  Міцність: {}/{} ХП  ·  {fuel_info}  ·  [F] Вийти з транспорту",
                                v.kind.label(),
                                v.health,
                                v.max_health
                            ))
                            .color(egui::Color32::from_rgb(112, 232, 252))
                            .size(13.0),
                        );
                    });
            });
    }

    let active_label = player_survival
        .inventory
        .active_item()
        .map(|stack| {
            if let Some((ammo_kind, _, _, _)) = stack.kind.ranged_spec() {
                let ammo_count = player_survival.inventory.count_item(ammo_kind);
                format!(
                    "{} [Набої: {} ×{}]",
                    stack.kind.label(),
                    ammo_kind.short_label(),
                    ammo_count
                )
            } else if stack.count > 1 {
                format!("{} (×{})", stack.kind.label(), stack.count)
            } else {
                stack.kind.label().to_string()
            }
        })
        .unwrap_or_else(|| "Порожні руки".into());

    let env_summary = format!(
        "{} ({}) · {} ({:+.0}°C)",
        format_time_of_day(env.time_hours),
        env.weather.label(),
        env.biome.label(),
        env.ambient_temp_c
    );
    let banner = match &player_survival.notice {
        Some(notice) => {
            format!("У руках: {active_label}  ·  {notice}  ·  {env_summary}  ·  [M] Карта")
        }
        None => format!(
            "У руках: {active_label}  ·  Деревина: {}  ·  Каміння: {}  ·  Метал: {}  ·  Скрап: {}  ·  {env_summary}",
            player_survival.inventory.count_item(ItemKind::Wood),
            player_survival.inventory.count_item(ItemKind::Stone),
            player_survival
                .inventory
                .count_item(ItemKind::MetalFragments),
            player_survival.inventory.count_item(ItemKind::Scrap),
        ),
    };
    egui::Area::new(egui::Id::new("survival-status-banner"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -90.0))
        .interactable(false)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(banner)
                    .size(13.0)
                    .color(egui::Color32::from_rgb(212, 226, 228)),
            );
        });
}

#[allow(clippy::too_many_arguments)]
fn draw_inventory(
    ctx: &egui::Context,
    interface: &mut Interface,
    mode: &SimulationMode,
    clock: &SimulationClock,
    physics: &mut SimulationWorld,
    survival: &mut SimulationSurvival,
    online: &Online,
    local_player: &Query<
        (&mut SimulationPose, &mut CharacterMotion, &mut Transform),
        With<LocalPlayer>,
    >,
    nodes: &mut Query<&mut ResourceNodeMarker>,
) {
    let snapshot = survival.0.player(PlayerId(0)).cloned().unwrap_or_default();
    egui::Window::new("Інвентар та Майстерня")
        .id(egui::Id::new("inventory-window"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(720.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Здоров'я: {:.0}/100  ·  Вода: {:.0}/250  ·  Їжа: {:.0}/500  ·  Радіація: {:.0} РАД",
                    snapshot.vitals.health,
                    snapshot.vitals.hydration,
                    snapshot.vitals.calories,
                    snapshot.vitals.radiation
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Повернутися в гру (Tab / Esc)").clicked() {
                        interface.selected_slot = None;
                        interface.mode = UiMode::Playing;
                    }
                });
            });
            if let Some(notice) = &snapshot.notice {
                ui.colored_label(egui::Color32::from_rgb(106, 207, 183), notice);
            }
            ui.separator();

            ui.columns(2, |columns| {
                let left = &mut columns[0];
                left.heading("Спорядження та Рюкзак");
                left.small("Натисніть на слот, щоб вибрати або поміняти предмети місцями.");
                left.add_space(6.0);
                left.label("Панель швидкого доступу (клавіші 1–6):");
                left.horizontal_wrapped(|ui| {
                    for i in 0..HOTBAR_SLOTS as u8 {
                        let slot_idx = SlotIndex::Hotbar(i);
                        let stack = snapshot.inventory.slot(slot_idx);
                        let is_selected = interface.selected_slot == Some(slot_idx);
                        let is_active = snapshot.inventory.active_slot == i;
                        if draw_slot_button(
                            ui,
                            &format!("{}", i + 1),
                            stack,
                            is_selected,
                            is_active,
                        )
                        .clicked()
                        {
                            if let Some(prev) = interface.selected_slot {
                                if prev == slot_idx {
                                    select_hotbar_slot(i, mode, survival, online);
                                    interface.selected_slot = None;
                                } else {
                                    move_inventory_slot(prev, slot_idx, mode, survival, online);
                                    interface.selected_slot = None;
                                }
                            } else {
                                select_hotbar_slot(i, mode, survival, online);
                                interface.selected_slot = Some(slot_idx);
                            }
                        }
                    }
                });

                left.add_space(8.0);
                left.label("Рюкзак (18 слотів):");
                for row in 0..3 {
                    left.horizontal(|ui| {
                        for col in 0..6 {
                            let idx = (row * 6 + col) as u8;
                            let slot_idx = SlotIndex::Backpack(idx);
                            let stack = snapshot.inventory.slot(slot_idx);
                            let is_selected = interface.selected_slot == Some(slot_idx);
                            if draw_slot_button(ui, "", stack, is_selected, false).clicked() {
                                if let Some(prev) = interface.selected_slot {
                                    if prev == slot_idx {
                                        interface.selected_slot = None;
                                    } else {
                                        move_inventory_slot(prev, slot_idx, mode, survival, online);
                                        interface.selected_slot = None;
                                    }
                                } else {
                                    interface.selected_slot = Some(slot_idx);
                                }
                            }
                        }
                    });
                }

                left.add_space(8.0);
                if let Some(selected) = interface.selected_slot
                    && let Some(stack) = snapshot.inventory.slot(selected)
                {
                    left.horizontal(|ui| {
                        ui.label(format!("Обрано: {} (×{})", stack.kind.label(), stack.count));
                        if matches!(
                            stack.kind,
                            ItemKind::Bandage
                                | ItemKind::WaterBottle
                                | ItemKind::Ration
                                | ItemKind::AntiRadPills
                        ) && ui.button("Використати").clicked()
                        {
                            let hotbar_slot = match selected {
                                SlotIndex::Hotbar(s) => s,
                                SlotIndex::Backpack(_) => {
                                    let active = snapshot.inventory.active_slot;
                                    move_inventory_slot(
                                        selected,
                                        SlotIndex::Hotbar(active),
                                        mode,
                                        survival,
                                        online,
                                    );
                                    active
                                }
                            };
                            select_hotbar_slot(hotbar_slot, mode, survival, online);
                            if let Ok((pose, motion, _)) = local_player.single() {
                                trigger_primary_action(
                                    interface,
                                    mode,
                                    clock,
                                    physics,
                                    survival,
                                    online,
                                    pose.current.to_array(),
                                    motion.0.yaw,
                                    motion.0.pitch,
                                    nodes,
                                );
                            }
                            interface.selected_slot = None;
                        }
                    });
                }

                let right = &mut columns[1];
                right.heading("Майстерня (Крафт)");
                right.label(format!(
                    "Деревина: {}  ·  Каміння: {}  ·  Метал: {}  ·  Скрап: {}",
                    snapshot.inventory.count_item(ItemKind::Wood),
                    snapshot.inventory.count_item(ItemKind::Stone),
                    snapshot.inventory.count_item(ItemKind::MetalFragments),
                    snapshot.inventory.count_item(ItemKind::Scrap),
                ));
                right.add_space(6.0);
                egui::ScrollArea::vertical()
                    .max_height(350.0)
                    .show(right, |right| {
                        for recipe in RecipeId::ALL {
                            let spec = recipe.spec();
                            let can_craft = snapshot.inventory.can_craft(recipe);
                            egui::Frame::group(right.style())
                                .fill(egui::Color32::from_rgb(22, 34, 41))
                                .show(right, |ui| {
                                    ui.horizontal(|ui| {
                                        let (icon_rect, _) = ui.allocate_exact_size(
                                            egui::vec2(36.0, 36.0),
                                            egui::Sense::hover(),
                                        );
                                        hud_painter::paint_item_icon(
                                            ui.painter(),
                                            icon_rect.center(),
                                            1.0,
                                            spec.output,
                                        );
                                        ui.vertical(|ui| {
                                            ui.strong(spec.output.label());
                                            ui.small(spec.description);
                                            let reqs: Vec<String> = spec
                                                .ingredients
                                                .iter()
                                                .map(|(k, n)| format!("{}: {}", k.label(), n))
                                                .collect();
                                            ui.small(format!("Потрібно: {}", reqs.join(" + ")));
                                        });
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .add_enabled(
                                                        can_craft,
                                                        egui::Button::new("Скрафтити"),
                                                    )
                                                    .clicked()
                                                {
                                                    craft_recipe(recipe, mode, survival, online);
                                                }
                                            },
                                        );
                                    });
                                });
                            right.add_space(4.0);
                        }
                    });
            });
        });
    let _ = BACKPACK_SLOTS;
}

fn draw_slot_button(
    ui: &mut egui::Ui,
    corner_label: &str,
    stack: Option<shared::survival::ItemStack>,
    selected: bool,
    active_hotbar: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(50.0, 50.0), egui::Sense::click());
    let painter = ui.painter();
    let fill = if selected {
        egui::Color32::from_rgb(42, 76, 74)
    } else if response.hovered() {
        egui::Color32::from_rgb(35, 53, 60)
    } else {
        egui::Color32::from_rgb(25, 38, 44)
    };
    painter.rect_filled(rect, 5.0, fill);
    let stroke_color = if selected {
        egui::Color32::from_rgb(156, 231, 212)
    } else if active_hotbar {
        egui::Color32::from_rgb(106, 207, 183)
    } else {
        egui::Color32::from_rgb(58, 78, 86)
    };
    painter.rect_stroke(
        rect,
        5.0,
        egui::Stroke::new(
            if selected || active_hotbar {
                1.8_f32
            } else {
                1.0_f32
            },
            stroke_color,
        ),
        egui::StrokeKind::Inside,
    );
    if !corner_label.is_empty() {
        painter.text(
            rect.min + egui::vec2(4.0, 3.0),
            egui::Align2::LEFT_TOP,
            corner_label,
            egui::FontId::proportional(9.5),
            egui::Color32::from_rgb(142, 160, 164),
        );
    }
    if let Some(stack) = stack {
        hud_painter::paint_item_icon(painter, rect.center(), 0.9, stack.kind);
        painter.text(
            rect.center_bottom() - egui::vec2(0.0, 2.0),
            egui::Align2::CENTER_BOTTOM,
            stack.kind.short_label(),
            egui::FontId::proportional(7.5),
            egui::Color32::from_rgb(224, 232, 233),
        );
        if stack.count > 1 {
            painter.text(
                rect.right_top() + egui::vec2(-3.0, 3.0),
                egui::Align2::RIGHT_TOP,
                format!("×{}", stack.count),
                egui::FontId::proportional(9.0),
                egui::Color32::from_rgb(106, 207, 183),
            );
        }
        response
            .clone()
            .on_hover_text(format!("{} (×{})", stack.kind.label(), stack.count));
    }
    response
}

fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(17, 26, 33);
    visuals.panel_fill = egui::Color32::from_rgb(17, 26, 33);
    visuals.selection.bg_fill = egui::Color32::from_rgb(43, 99, 91);
    visuals.selection.stroke.color = egui::Color32::from_rgb(156, 231, 212);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(45, 112, 99);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(38, 66, 71);
    ctx.set_visuals(visuals);
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 7.0);
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(22.0));
    });
}

#[allow(clippy::too_many_arguments)]
fn draw_widgets(
    ctx: &egui::Context,
    interface: &mut Interface,
    telemetry: &Telemetry,
    player_survival: &PlayerSurvival,
    mode: &SimulationMode,
    clock: &SimulationClock,
    physics: &mut SimulationWorld,
    survival: &mut SimulationSurvival,
    online: &Online,
    local_player: &Query<
        (&mut SimulationPose, &mut CharacterMotion, &mut Transform),
        With<LocalPlayer>,
    >,
    nodes: &mut Query<&mut ResourceNodeMarker>,
    map_tex_id: Option<egui::TextureId>,
    marker_tex_id: Option<egui::TextureId>,
    ui_tex_ids: UiTexturesIds,
) {
    let editing = interface.mode == UiMode::Editor;
    let interactive_hud = matches!(
        interface.mode,
        UiMode::Playing | UiMode::Inventory | UiMode::Map
    );
    let viewport_rect = ctx.content_rect();
    let viewport = [viewport_rect.width(), viewport_rect.height()];
    let widgets: Vec<_> = interface
        .editor
        .layout()
        .widgets
        .iter()
        .map(|(&id, config)| (id, config.clone()))
        .collect();
    for (id, config) in widgets {
        if !config.visible && !editing {
            continue;
        }
        let placement = config.placement(id, viewport);
        let clickable = editing
            || (interactive_hud
                && matches!(
                    id,
                    WidgetId::Hotbar
                        | WidgetId::Minimap
                        | WidgetId::InventoryButton
                        | WidgetId::FireButtonLeft
                        | WidgetId::FireButtonRight
                ));
        egui::Area::new(egui::Id::new(("hud-widget", id)))
            .order(egui::Order::Background)
            .fixed_pos(hud_painter::logical_position(
                viewport_rect.min,
                placement.position,
            ))
            .movable(false)
            .constrain(false)
            .interactable(clickable)
            .show(ctx, |ui| {
                let (rect, response) = ui.allocate_exact_size(
                    egui::Vec2::from(placement.size),
                    if editing {
                        egui::Sense::click_and_drag()
                    } else if clickable {
                        egui::Sense::click()
                    } else {
                        egui::Sense::hover()
                    },
                );
                let opacity = if config.visible { config.opacity } else { 0.2 };
                let map_ctx = MinimapContext {
                    player_position: telemetry.player_position,
                    yaw: telemetry.yaw,
                    nodes: &survival.0.nodes,
                    structures: &survival.0.structures,
                    npcs: &survival.0.npcs,
                    vehicles: &survival.0.vehicles,
                    nuclear: survival.0.nuclear,
                    tick: ServerTick(clock.tick),
                    marker: interface.map_marker,
                    map_tex_id,
                    marker_tex_id,
                    cargo_tex_id: None,
                    vehicle_tex_id: None,
                    sound_cues: &interface.sound_cues,
                    ui_tex: ui_tex_ids,
                };
                hud_painter::paint_widget(
                    ui.painter(),
                    rect,
                    id,
                    placement.render_scale,
                    opacity,
                    player_survival,
                    &map_ctx,
                );
                if editing {
                    hud_painter::selection(ui.painter(), rect, interface.editor.selected == id);
                    if response.clicked() || response.dragged() {
                        interface.editor.selected = id;
                    }
                    if response.dragged() {
                        let delta = response.drag_delta();
                        if let Some(layout) = interface.editor.draft_mut()
                            && let Some(widget) = layout.widgets.get_mut(&id)
                        {
                            widget.set_position(
                                id,
                                viewport,
                                [
                                    placement.position[0] + delta.x,
                                    placement.position[1] + delta.y,
                                ],
                            );
                        }
                    }
                    response.on_hover_text(id.label());
                } else if clickable && response.clicked() {
                    match id {
                        WidgetId::InventoryButton => interface.toggle_inventory(),
                        WidgetId::Minimap => interface.toggle_map(),
                        WidgetId::FireButtonLeft | WidgetId::FireButtonRight => {
                            if let Ok((pose, motion, _)) = local_player.single() {
                                trigger_primary_action(
                                    interface,
                                    mode,
                                    clock,
                                    physics,
                                    survival,
                                    online,
                                    pose.current.to_array(),
                                    motion.0.yaw,
                                    motion.0.pitch,
                                    nodes,
                                );
                            }
                        }
                        WidgetId::Hotbar => {
                            if let Some(pointer) = response.interact_pointer_pos() {
                                let rel =
                                    ((pointer.x - rect.min.x) / rect.width()).clamp(0.0, 0.999);
                                let slot = (rel * 6.0) as u8;
                                select_hotbar_slot(slot, mode, survival, online);
                            }
                        }
                        _ => {}
                    }
                }
            });
    }

    if matches!(interface.mode, UiMode::Playing | UiMode::Benchmark) {
        let map_ctx = MinimapContext {
            player_position: telemetry.player_position,
            yaw: telemetry.yaw,
            nodes: &survival.0.nodes,
            structures: &survival.0.structures,
            npcs: &survival.0.npcs,
            vehicles: &survival.0.vehicles,
            nuclear: survival.0.nuclear,
            tick: ServerTick(clock.tick),
            marker: interface.map_marker,
            map_tex_id,
            marker_tex_id,
            cargo_tex_id: None,
            vehicle_tex_id: None,
            sound_cues: &interface.sound_cues,
            ui_tex: ui_tex_ids,
        };
        egui::Area::new(egui::Id::new("hud-compass-bar"))
            .order(egui::Order::Background)
            .fixed_pos(egui::pos2(
                viewport_rect.center().x - 180.0,
                viewport_rect.min.y + 10.0,
            ))
            .interactable(false)
            .show(ctx, |ui| {
                let scale = 1.0;
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(360.0 * scale, 34.0 * scale),
                    egui::Sense::hover(),
                );
                hud_painter::paint_compass_bar(ui.painter(), rect.center_top(), scale, &map_ctx);
            });
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_map(
    ctx: &egui::Context,
    interface: &mut Interface,
    telemetry: &Telemetry,
    clock: &SimulationClock,
    survival: &SimulationSurvival,
    player_survival: &PlayerSurvival,
    map_tex_id: Option<egui::TextureId>,
    marker_tex_id: Option<egui::TextureId>,
    cargo_tex_id: Option<egui::TextureId>,
    vehicle_tex_id: Option<egui::TextureId>,
    ui_tex_ids: UiTexturesIds,
    audio_assets: Option<&AudioAssets>,
    commands: &mut Commands,
    mut perspective: Option<ResMut<CameraPerspective>>,
) {
    let env = survival
        .0
        .environment_at(ServerTick(clock.tick), telemetry.player_position);
    egui::Window::new("Тактична карта острова (A-01..D-56)")
        .id(egui::Id::new("tactical-map-window"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(540.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Час: {} ({})  ·  Біом: {}  ·  Погода: {}",
                    format_time_of_day(env.time_hours),
                    if env.is_night { "Ніч" } else { "День" },
                    env.biome.label(),
                    env.weather.label()
                ));
                if let Some(pers) = perspective.as_deref_mut() {
                    let label = match *pers {
                        CameraPerspective::FirstPerson => "Камера: 1-е лице [V]",
                        CameraPerspective::ThirdPerson => "Камера: 3-є лице [V]",
                    };
                    if ui.button(label).clicked() {
                        *pers = match *pers {
                            CameraPerspective::FirstPerson => CameraPerspective::ThirdPerson,
                            CameraPerspective::ThirdPerson => CameraPerspective::FirstPerson,
                        };
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Закрити (M / Esc)").clicked() {
                        interface.mode = UiMode::Playing;
                    }
                });
            });
            ui.small(format!(
                "Повітря: {:+.1}°C  ·  Тіло: {:.1}°C  ·  Комфорт: {:.0}%  ·  Координати: X {:.1}, Z {:.1}",
                env.ambient_temp_c,
                player_survival.vitals.body_temp_c,
                player_survival.vitals.comfort,
                telemetry.player_position[0],
                telemetry.player_position[2]
            ));
            ui.separator();

            let (map_rect, response) =
                ui.allocate_exact_size(egui::vec2(460.0, 460.0), egui::Sense::click());
            let map_ctx = MinimapContext {
                player_position: telemetry.player_position,
                yaw: telemetry.yaw,
                nodes: &survival.0.nodes,
                structures: &survival.0.structures,
                npcs: &survival.0.npcs,
                vehicles: &survival.0.vehicles,
                nuclear: survival.0.nuclear,
                tick: ServerTick(clock.tick),
                marker: interface.map_marker,
                map_tex_id,
                marker_tex_id,
                cargo_tex_id,
                vehicle_tex_id,
                sound_cues: &interface.sound_cues,
                ui_tex: ui_tex_ids,
            };
            hud_painter::paint_tactical_map(ui.painter(), map_rect, 1.0, &map_ctx, true);

            if response.clicked()
                && let Some(pointer) = response.interact_pointer_pos()
            {
                let prev = interface.map_marker;
                let new_pos = hud_painter::map_pos_to_world(map_rect, pointer);
                if prev != Some(new_pos) {
                    interface.map_marker = Some(new_pos);
                    if let Some(audio) = audio_assets {
                        commands.spawn((
                            AudioPlayer::new(audio.map_pin.clone()),
                            PlaybackSettings::DESPAWN,
                        ));
                    }
                }
            }
            if response.secondary_clicked() && interface.map_marker.is_some() {
                interface.map_marker = None;
                if let Some(audio) = audio_assets {
                    commands.spawn((
                        AudioPlayer::new(audio.map_unpin.clone()),
                        PlaybackSettings::DESPAWN,
                    ));
                }
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if let Some(marker) = interface.map_marker {
                    let dx = marker[0] - telemetry.player_position[0];
                    let dz = marker[1] - telemetry.player_position[2];
                    let dist = (dx * dx + dz * dz).sqrt();
                    ui.colored_label(
                        egui::Color32::from_rgb(248, 206, 78),
                        format!(
                            "Маркер: X {:.1}, Z {:.1} (відстань {:.1} м)",
                            marker[0], marker[1], dist
                        ),
                    );
                    if ui.button("Прибрати маркер").clicked() {
                        interface.map_marker = None;
                        if let Some(audio) = audio_assets {
                            commands.spawn((
                                AudioPlayer::new(audio.map_unpin.clone()),
                                PlaybackSettings::DESPAWN,
                            ));
                        }
                    }
                } else {
                    ui.small("ЛКМ по карті — встановити мітку. ПКМ — прибрати. [V / F5] — перемкнути камеру (1-е/3-є лице).");
                }
            });
        });
}

fn draw_editor(ctx: &egui::Context, interface: &mut Interface, storage: &dyn StorageService) {
    let content = ctx.content_rect();
    let viewport = [content.width(), content.height()];
    egui::Window::new("Редактор HUD")
        .id(egui::Id::new("hud-editor"))
        .collapsible(false)
        .default_width(310.0)
        .default_pos(content.min + egui::vec2((content.width() - 310.0) / 2.0, 42.0))
        .max_height((content.height() - 70.0).max(100.0))
        .vscroll(true)
        .show(ctx, |ui| {
            ui.label("Перетягніть віджет або виберіть його зі списку. Приховані елементи показані контуром.");
            ui.horizontal_wrapped(|ui| {
                for preset in HudPreset::ALL {
                    if ui
                        .selectable_label(
                            interface.editor.layout().preset == preset,
                            preset.label(),
                        )
                        .clicked()
                    {
                        interface.editor.apply_preset(preset);
                    }
                }
            });
            ui.separator();
            egui::ComboBox::from_label("Віджет")
                .selected_text(interface.editor.selected.label())
                .show_ui(ui, |ui| {
                    for id in WidgetId::ALL {
                        ui.selectable_value(&mut interface.editor.selected, id, id.label());
                    }
                });
            let selected = interface.editor.selected;
            let mut widget = interface.editor.layout().widgets[&selected].clone();
            ui.checkbox(&mut widget.visible, "Показувати під час гри");
            let mut anchor = widget.anchor;
            egui::ComboBox::from_label("Прив’язка")
                .selected_text(anchor.label())
                .show_ui(ui, |ui| {
                    for candidate in WidgetAnchor::ALL {
                        ui.selectable_value(&mut anchor, candidate, candidate.label());
                    }
                });
            if anchor != widget.anchor {
                widget.reanchor(selected, viewport, anchor);
            }
            ui.add(egui::Slider::new(&mut widget.scale, 0.5..=2.5).text("Розмір"));
            ui.add(egui::Slider::new(&mut widget.opacity, 0.1..=1.0).text("Прозорість"));
            if let Some(layout) = interface.editor.draft_mut() {
                layout.widgets.insert(selected, widget);
            }
            if ui.button("Скинути поточний пресет").clicked() {
                interface
                    .editor
                    .apply_preset(interface.editor.layout().preset);
            }
            if interface.editor.layout().preset != HudPreset::Desktop {
                ui.small("Мобільні кнопки тут налаштовують розкладку. Керування на Linux — клавіатура й миша.");
            }
            ui.separator();
            if let Some(error) = &interface.editor.error {
                ui.colored_label(egui::Color32::from_rgb(240, 133, 127), error);
            }
            ui.horizontal(|ui| {
                if ui.button("Зберегти").clicked() && interface.editor.save(storage) {
                    interface.notice = None;
                    interface.mode = UiMode::Menu;
                }
                if ui.button("Скасувати").clicked() {
                    interface.editor.cancel();
                    interface.mode = UiMode::Menu;
                }
            });
            ui.small("Esc — скасувати зміни. Налаштування записуються лише кнопкою «Зберегти».");
        });
}

#[allow(clippy::too_many_arguments, clippy::collapsible_if)]
fn draw_gm_panel(
    ctx: &egui::Context,
    interface: &mut Interface,
    clock: &mut SimulationClock,
    physics: &mut SimulationWorld,
    survival: &mut SimulationSurvival,
    local_player: &mut Query<
        (&mut SimulationPose, &mut CharacterMotion, &mut Transform),
        With<LocalPlayer>,
    >,
    telemetry: &Telemetry,
    windows: &mut Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    commands: &mut Commands,
) {
    let mut open = interface.show_gm_panel;
    let scr = ctx.content_rect();
    egui::Window::new("⚡ GM Admin Panel (GMPanel)")
        .id(egui::Id::new("gm-admin-panel"))
        .open(&mut open)
        .collapsible(true)
        .resizable(true)
        .default_width(540.0)
        .default_height(480.0)
        .max_width((scr.width() - 30.0).max(300.0))
        .max_height((scr.height() - 50.0).max(240.0))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-20.0, 45.0))
        .vscroll(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut interface.gm_active_tab, 0, "🔥 Cheats & Vitals");
                ui.selectable_value(&mut interface.gm_active_tab, 1, "🎒 Gear & Kits");
                ui.selectable_value(&mut interface.gm_active_tab, 2, "🚀 Teleportation");
                ui.selectable_value(&mut interface.gm_active_tab, 3, "🌍 World & Time");
                ui.selectable_value(&mut interface.gm_active_tab, 4, "📦 Item Spawner");
            });
            ui.separator();

            match interface.gm_active_tab {
                0 => {
                    ui.heading("Cheats & Player Vitals (+buff / 飞天 / 自杀)");
                    ui.label("Configure invulnerability, survival vitals, flight mode, and respawn.");
                    ui.add_space(8.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 32.0],
                            egui::Button::new("🗑️ Delete Looked Target / Видалити об'єкт під прицілом")
                                .fill(egui::Color32::from_rgb(175, 45, 45)),
                        )
                        .on_hover_text(
                            "Instantly delete whatever object, structure, or node is currently in crosshairs.",
                        )
                        .clicked()
                    {
                        if let Some(ref looked) = telemetry.looked_object {
                            if let Some(msg) = delete_looked_object(looked, survival, physics, Some(commands)) {
                                if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                                    player.notice = Some(msg);
                                }
                            }
                        }
                    }
                    ui.add_space(4.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 32.0],
                            egui::Button::new("⚡ Godmode (+Buff / Full Vitals)"),
                        )
                        .on_hover_text(
                            "Instantly heal to 100 HP, quench thirst, satisfy hunger, and purge all radiation poisoning.",
                        )
                        .clicked()
                    {
                        if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                            player.vitals = shared::survival::Vitals::default();
                            player.notice = Some("GM: Full vitals restored (+buff)!".into());
                        }
                    }
                    ui.add_space(4.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 32.0],
                            egui::Button::new("🔫 Equip Teleport Gun"),
                        )
                        .on_hover_text(
                            "Equip the Teleport Gun into hotbar slot 1. Left-click anywhere in the world to instantly warp.",
                        )
                        .clicked()
                    {
                        if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                            player.inventory.hotbar[0] = Some(shared::survival::ItemStack {
                                kind: shared::survival::ItemKind::TeleportGun,
                                count: 1,
                            });
                            player.inventory.select_slot(0);
                            player.notice = Some(
                                "GM: Teleport Gun equipped in Hotbar Slot 1! Aim and Left-Click to warp."
                                    .into(),
                            );
                        }
                    }
                    ui.add_space(4.0);

                    let fly_title = if interface.gm_fly_mode {
                        "🕊️ Fly Mode (飞天): [ENABLED] (Click to Disable)"
                    } else {
                        "🕊️ Fly Mode (飞天): [DISABLED] (Click to Enable)"
                    };
                    if ui
                        .add_sized([ui.available_width(), 32.0], egui::Button::new(fly_title))
                        .on_hover_text(
                            "Toggle noclip flight mode. Hold Space to ascend, hold C or Ctrl to descend.",
                        )
                        .clicked()
                    {
                        interface.gm_fly_mode = !interface.gm_fly_mode;
                        if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                            player.notice = Some(format!(
                                "GM: Fly mode {} (Space: Ascend, C/Ctrl: Descend)",
                                if interface.gm_fly_mode {
                                    "ENABLED"
                                } else {
                                    "DISABLED"
                                }
                            ));
                        }
                    }
                    ui.add_space(4.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 32.0],
                            egui::Button::new("☠️ Suicide / Coastal Respawn (自杀)"),
                        )
                        .on_hover_text(
                            "Reset character status and respawn with fresh vitals at the northern coastline.",
                        )
                        .clicked()
                    {
                        if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                            player.vitals = shared::survival::Vitals::default();
                            player.notice = Some("GM: Respawned at northern shoreline.".into());
                        }
                        if let Ok((mut pose, mut motion, mut transform)) =
                            local_player.single_mut()
                        {
                            let spawn = Vec3::new(0.0, 5.0, -350.0);
                            pose.current = spawn;
                            pose.previous = spawn;
                            transform.translation = spawn;
                            motion.0.vertical_velocity = 0.0;
                            physics.0.set_player(
                                PlayerId(0),
                                shared::protocol::PlayerState {
                                    position: spawn.to_array(),
                                    motion: motion.0,
                                    velocity: [0.0; 3],
                                },
                            );
                        }
                    }
                }
                1 => {
                    ui.heading("Kits & Equipment (全武器 / 全子弹 / 全材料)");
                    ui.label("Grant weapon armaments, stockpiles of ammunition, and building materials.");
                    ui.add_space(8.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new("⚔️ All Weapons (全武器)"),
                        )
                        .on_hover_text(
                            "Grant AK-47, Revolver, Hunting Bow, Spear, Hatchet, Pickaxe, and Teleport Gun.",
                        )
                        .clicked()
                    {
                        if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                            let weapons = [
                                (ItemKind::AssaultRifle, 1),
                                (ItemKind::Revolver, 1),
                                (ItemKind::HuntingBow, 1),
                                (ItemKind::WoodenSpear, 1),
                                (ItemKind::StoneHatchet, 1),
                                (ItemKind::StonePickaxe, 1),
                                (ItemKind::TeleportGun, 1),
                            ];
                            for (kind, count) in weapons {
                                player.inventory.add_item(kind, count);
                            }
                            player.notice = Some("GM: All weapons granted to inventory!".into());
                        }
                    }
                    ui.add_space(4.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new("🎯 All Ammo (全子弹)"),
                        )
                        .on_hover_text(
                            "Stockpile 500x 5.56mm Rifle Ammo, 500x 9mm Pistol Ammo, and 200x Arrows.",
                        )
                        .clicked()
                    {
                        if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                            player.inventory.add_item(ItemKind::RifleAmmo, 500);
                            player.inventory.add_item(ItemKind::PistolAmmo, 500);
                            player.inventory.add_item(ItemKind::Arrow, 200);
                            player.notice = Some("GM: All ammunition types granted!".into());
                        }
                    }
                    ui.add_space(4.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new("🧱 All Materials (全材料)"),
                        )
                        .on_hover_text(
                            "Stockpile 1000 Wood, 1000 Stone, 500 Metal Fragments, 250 Scrap, and 100 Fuel.",
                        )
                        .clicked()
                    {
                        if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                            player.inventory.add_item(ItemKind::Wood, 1000);
                            player.inventory.add_item(ItemKind::Stone, 1000);
                            player.inventory.add_item(ItemKind::MetalFragments, 500);
                            player.inventory.add_item(ItemKind::Scrap, 250);
                            player.inventory.add_item(ItemKind::LowGradeFuel, 100);
                            player.notice = Some("GM: Raw building materials granted!".into());
                        }
                    }
                    ui.add_space(4.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new("🩹 All Supplies (全补给)"),
                        )
                        .on_hover_text(
                            "Add 5 Military Rations, 5 Water Bottles, 10 Bandages, and 10 Anti-Rad Pills.",
                        )
                        .clicked()
                    {
                        if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                            player.inventory.add_item(ItemKind::Ration, 5);
                            player.inventory.add_item(ItemKind::WaterBottle, 5);
                            player.inventory.add_item(ItemKind::Bandage, 10);
                            player.inventory.add_item(ItemKind::AntiRadPills, 10);
                            player.notice = Some("GM: Survival rations and medicine granted!".into());
                        }
                    }
                    ui.add_space(4.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new("📜 Unlock Blueprints (蓝图全解)"),
                        )
                        .on_hover_text(
                            "Grant massive crafting material stocks enabling instant fabrication of all 15 recipes.",
                        )
                        .clicked()
                    {
                        if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                            player.inventory.add_item(ItemKind::Wood, 3000);
                            player.inventory.add_item(ItemKind::Stone, 2000);
                            player.inventory.add_item(ItemKind::MetalFragments, 1000);
                            player.inventory.add_item(ItemKind::Scrap, 500);
                            player.inventory.add_item(ItemKind::ElectricFuse, 10);
                            player.inventory.add_item(ItemKind::LowGradeFuel, 300);
                            player.inventory.add_item(ItemKind::C4Explosive, 5);
                            player.notice =
                                Some("GM: Full blueprint crafting resources unlocked!".into());
                        }
                    }
                    ui.add_space(4.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new("🗑️ Clear Inventory (清空背包)"),
                        )
                        .on_hover_text("Wipe all items from hotbar and backpack.")
                        .clicked()
                    {
                        if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                            player.inventory = shared::survival::Inventory::empty();
                            player.notice = Some("GM: Inventory cleared.".into());
                        }
                    }
                }
                2 => {
                    ui.heading("Island Teleportation (传送)");
                    ui.label("Instantly warp across key monuments, sectors, and biomes.");
                    ui.add_space(8.0);

                    let locations: [(&str, [f32; 3], &str); 6] = [
                        (
                            "🏝️ Island Center (Ground Zero)",
                            [0.0, 10.0, 0.0],
                            "Warp to island coordinates (0, 0).",
                        ),
                        (
                            "🌲 A-57 Pine Forest Valley",
                            [0.0, 3.0, 15.0],
                            "Warp to A-57 pine forest sector valley.",
                        ),
                        (
                            "🌊 Northern Shoreline (Beach)",
                            [0.0, 5.0, -350.0],
                            "Warp to northern spawn shoreline.",
                        ),
                        (
                            "🌊 Southern Ocean Coast",
                            [0.0, 5.0, 350.0],
                            "Warp to southern maritime shoreline.",
                        ),
                        (
                            "🌅 Eastern Coastline",
                            [350.0, 5.0, 0.0],
                            "Warp to eastern coast beach.",
                        ),
                        (
                            "🌄 Western Mountain Highlands",
                            [-350.0, 15.0, 0.0],
                            "Warp to western ridge overlook.",
                        ),
                    ];

                    for (name, coords, hint) in locations {
                        if ui
                            .add_sized([ui.available_width(), 28.0], egui::Button::new(name))
                            .on_hover_text(hint)
                            .clicked()
                        {
                            if let Ok((mut pose, mut motion, mut transform)) =
                                local_player.single_mut()
                            {
                                let dest = Vec3::from_array(coords);
                                pose.current = dest;
                                pose.previous = dest;
                                transform.translation = dest;
                                motion.0.vertical_velocity = 0.0;
                                physics.0.set_player(
                                    PlayerId(0),
                                    shared::protocol::PlayerState {
                                        position: coords,
                                        motion: motion.0,
                                        velocity: [0.0; 3],
                                    },
                                );
                                if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                                    player.notice = Some(format!("Teleported to {}", name));
                                }
                            }
                        }
                        ui.add_space(2.0);
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("Custom Coordinates Warp").strong());
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut interface.gm_target_coords[0])
                                .speed(1.0)
                                .prefix("X: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut interface.gm_target_coords[1])
                                .speed(1.0)
                                .prefix("Y: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut interface.gm_target_coords[2])
                                .speed(1.0)
                                .prefix("Z: "),
                        );
                    });
                    if ui
                        .add_sized(
                            [ui.available_width(), 28.0],
                            egui::Button::new("🚀 Teleport to Custom [X, Y, Z]"),
                        )
                        .on_hover_text(
                            "Instantly teleport to the exact custom coordinates specified above.",
                        )
                        .clicked()
                    {
                        let coords = interface.gm_target_coords;
                        if let Ok((mut pose, mut motion, mut transform)) =
                            local_player.single_mut()
                        {
                            let dest = Vec3::from_array(coords);
                            pose.current = dest;
                            pose.previous = dest;
                            transform.translation = dest;
                            motion.0.vertical_velocity = 0.0;
                            physics.0.set_player(
                                PlayerId(0),
                                shared::protocol::PlayerState {
                                    position: coords,
                                    motion: motion.0,
                                    velocity: [0.0; 3],
                                },
                            );
                            if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                                player.notice = Some(format!(
                                    "Teleported to [{:.1}, {:.1}, {:.1}]",
                                    coords[0], coords[1], coords[2]
                                ));
                            }
                        }
                    }
                }
                3 => {
                    ui.heading("World Environment & Atmosphere (+1小时 / 显隐雾 / 显隐UI)");
                    ui.label("Control world time of day, fog rendering, and user interface visibility.");
                    ui.add_space(8.0);

                    let hours = shared::survival::time_of_day_hours(ServerTick(clock.tick));
                    ui.label(
                        egui::RichText::new(format!(
                            "Simulation Time: {} (Tick: {})",
                            format_time_of_day(hours),
                            clock.tick
                        ))
                        .strong(),
                    );
                    ui.add_space(4.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new("⏩ Advance Time (+1 Hour / +1小时)"),
                        )
                        .on_hover_text("Advance current game time forward by 1 hour (750 ticks).")
                        .clicked()
                    {
                        clock.tick += shared::survival::DAY_CYCLE_TICKS / 24;
                    }
                    ui.add_space(4.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new("☀️ Set Noon (12:00)"),
                        )
                        .on_hover_text("Set sun to high noon for maximum daylight illumination.")
                        .clicked()
                    {
                        let cycle = shared::survival::DAY_CYCLE_TICKS;
                        let day_idx = clock.tick / cycle;
                        clock.tick = day_idx * cycle + (cycle * 2) / 24;
                    }
                    ui.add_space(4.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new("🌙 Set Midnight (00:00)"),
                        )
                        .on_hover_text("Set time to midnight darkness.")
                        .clicked()
                    {
                        let cycle = shared::survival::DAY_CYCLE_TICKS;
                        let day_idx = clock.tick / cycle;
                        clock.tick = day_idx * cycle + (cycle * 14) / 24;
                    }
                    ui.add_space(8.0);
                    ui.separator();

                    let fog_status = if interface.gm_fog_enabled {
                        "🌫️ Atmospheric Fog (显隐雾): [ENABLED] (Click to Clear)"
                    } else {
                        "🌫️ Atmospheric Fog (显隐雾): [DISABLED] (Click to Enable)"
                    };
                    if ui
                        .add_sized([ui.available_width(), 30.0], egui::Button::new(fog_status))
                        .on_hover_text(
                            "Toggle distance atmospheric fog on and off for crystal-clear long-range vision.",
                        )
                        .clicked()
                    {
                        interface.gm_fog_enabled = !interface.gm_fog_enabled;
                    }
                    ui.add_space(4.0);

                    let cloud_quality_label = match interface.volumetric_fog_steps {
                        12 => "☁️ Volumetric Clouds: [12 STEPS - LOW/MOBILE]",
                        48 => "☁️ Volumetric Clouds: [48 STEPS - ULTRA/CINEMATIC]",
                        _ => "☁️ Volumetric Clouds: [24 STEPS - BALANCED/DEFAULT]",
                    };
                    if ui
                        .add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new(cloud_quality_label),
                        )
                        .on_hover_text(
                            "Cycle raymarching step count for 3D volumetric clouds: 12 (Mobile), 24 (Balanced), or 48 (Cinematic).",
                        )
                        .clicked()
                    {
                        interface.volumetric_fog_steps = match interface.volumetric_fog_steps {
                            12 => 24,
                            24 => 48,
                            _ => 12,
                        };
                    }
                    ui.add_space(4.0);

                    let hud_status = if interface.gm_hud_enabled {
                        "🖥️ HUD Display (显隐UI): [VISIBLE] (Click to Hide)"
                    } else {
                        "🖥️ HUD Display (显隐UI): [HIDDEN] (Click to Show)"
                    };
                    if ui
                        .add_sized([ui.available_width(), 30.0], egui::Button::new(hud_status))
                        .on_hover_text(
                            "Toggle all on-screen widgets, crosshair, and telemetry overlays.",
                        )
                        .clicked()
                    {
                        interface.gm_hud_enabled = !interface.gm_hud_enabled;
                    }
                    ui.add_space(8.0);
                    ui.separator();

                    let is_fullscreen = windows
                        .single()
                        .ok()
                        .is_some_and(|w| !matches!(w.mode, bevy::window::WindowMode::Windowed));
                    let win_btn = if is_fullscreen {
                        "🪟 Switch to Windowed 1280x720 (F11)"
                    } else {
                        "🖥️ Switch to Fullscreen Borderless (F11)"
                    };
                    if ui
                        .add_sized([ui.available_width(), 30.0], egui::Button::new(win_btn))
                        .on_hover_text("Toggle borderless fullscreen mode for smooth Linux desktop integration.")
                        .clicked()
                    {
                        if let Ok(mut win) = windows.single_mut() {
                            win.mode = if is_fullscreen {
                                bevy::window::WindowMode::Windowed
                            } else {
                                bevy::window::WindowMode::BorderlessFullscreen(
                                    bevy::window::MonitorSelection::Current,
                                )
                            };
                        }
                    }
                }
                4 => {
                    ui.heading("Item Spawner (GMTool / GMToolPanel)");
                    ui.label("Select any survival item and spawn quantities directly into inventory.");
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Item:");
                        egui::ComboBox::from_id_salt("gm_tool_item_select")
                            .selected_text(format!(
                                "{} ({})",
                                interface.gm_selected_item.label_en(),
                                interface.gm_selected_item.label()
                            ))
                            .show_ui(ui, |ui| {
                                for item in shared::survival::ItemKind::ALL {
                                    ui.selectable_value(
                                        &mut interface.gm_selected_item,
                                        item,
                                        format!("{} ({})", item.label_en(), item.label()),
                                    );
                                }
                            });
                    });
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label("Quantity:");
                        ui.add(
                            egui::Slider::new(&mut interface.gm_spawn_amount, 1..=500).text("units"),
                        );
                    });
                    ui.add_space(8.0);

                    if ui
                        .add_sized([ui.available_width(), 34.0], egui::Button::new("📦 Spawn to Backpack"))
                        .on_hover_text(
                            "Spawn the configured amount of the selected item directly into your backpack.",
                        )
                        .clicked()
                    {
                        if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                            let leftover = player
                                .inventory
                                .add_item(interface.gm_selected_item, interface.gm_spawn_amount);
                            let spawned = interface.gm_spawn_amount.saturating_sub(leftover);
                            player.notice = Some(format!(
                                "GM: Spawned {}x {} into backpack!",
                                spawned,
                                interface.gm_selected_item.label_en()
                            ));
                        }
                    }
                }
                _ => {}
            }
        });
    interface.show_gm_panel = open;
}

fn draw_looked_object_hud(ctx: &egui::Context, looked: &LookedObjectInfo, is_teleport_gun: bool) {
    let screen = ctx.content_rect();
    let center_x = screen
        .center()
        .x
        .clamp(140.0, (screen.width() - 140.0).max(140.0));
    let center_y = (screen.center().y + 36.0).clamp(30.0, (screen.height() - 95.0).max(30.0));

    egui::Area::new(egui::Id::new("crosshair_target_info"))
        .fixed_pos(egui::pos2(center_x, center_y))
        .pivot(egui::Align2::CENTER_TOP)
        .interactable(false)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(12, 17, 24, 215))
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    egui::Color32::from_rgb(60, 110, 160),
                ))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(14, 7))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(&looked.title)
                                .strong()
                                .size(14.0)
                                .color(egui::Color32::from_rgb(255, 230, 130)),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(&looked.detail)
                                .size(11.5)
                                .color(egui::Color32::from_rgb(190, 215, 230)),
                        );
                        ui.add_space(3.0);
                        ui.horizontal(|ui| {
                            if let Some(prompt) = &looked.interact_prompt {
                                egui::Frame::new()
                                    .fill(egui::Color32::from_rgba_unmultiplied(35, 75, 45, 180))
                                    .stroke(egui::Stroke::new(
                                        1.0_f32,
                                        egui::Color32::from_rgb(80, 200, 120),
                                    ))
                                    .corner_radius(4.0)
                                    .inner_margin(egui::Margin::symmetric(8, 3))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(prompt)
                                                .strong()
                                                .size(11.5)
                                                .color(egui::Color32::from_rgb(140, 255, 170)),
                                        );
                                    });
                            }
                            if looked.target_kind.is_some() {
                                let (del_text, del_color) = if is_teleport_gun {
                                    (
                                        "💥 [ЛКМ] Видалити об'єкт",
                                        egui::Color32::from_rgb(255, 120, 100),
                                    )
                                } else {
                                    (
                                        "🗑️ [Delete / X] Видалити",
                                        egui::Color32::from_rgb(220, 140, 120),
                                    )
                                };
                                egui::Frame::new()
                                    .fill(egui::Color32::from_rgba_unmultiplied(75, 25, 25, 180))
                                    .stroke(egui::Stroke::new(
                                        1.0_f32,
                                        egui::Color32::from_rgb(200, 80, 80),
                                    ))
                                    .corner_radius(4.0)
                                    .inner_margin(egui::Margin::symmetric(8, 3))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(del_text)
                                                .strong()
                                                .size(11.5)
                                                .color(del_color),
                                        );
                                    });
                            }
                        });
                    });
                });
        });
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

const LOADING_TIPS: &[(&str, &str)] = &[
    (
        "Головна мета виживання — захопити емблему відродження до завершення зворотного відліку.",
        "The ultimate goal of survival is to capture the Restart Badge before the countdown ends.",
    ),
    (
        "Регулярно перевіряйте шафу інструментів (TC): якщо ресурси утримання вичерпаються, база почне гнити.",
        "Check your Tool Cupboard regularly: once upkeep runs out, structures will slowly decay.",
    ),
    (
        "Скрап та запчастини — найважливіші матеріали для покращення верстата та вивчення креслень.",
        "Scrap and components are essential for researching blueprints and upgrading workbenches.",
    ),
    (
        "Картки доступу та запобіжники потрібні для відкриття захищених сховищ і кімнат Радтауну.",
        "Keycards and electrical fuses are required to breach sealed bunker rooms in Radtown.",
    ),
    (
        "Зони навколо військових кімнат мають смертельну радіацію: надягайте захисний костюм або протигаз.",
        "Areas around puzzle rooms have lethal radiation: wear a hazmat suit or gas mask.",
    ),
    (
        "Зомбі, Bradley та боси можуть залишати після себе ключі від військових сейфів та елітний лут.",
        "Zombies, Bradley APCs, and bosses can drop high-tier bunker keys and military loot.",
    ),
    (
        "Транспорт (F) суттєво прискорює подорожі островом, але вимагає регулярної заправки паливом.",
        "Vehicles greatly speed up travel across the island, but require low grade fuel.",
    ),
    (
        "У разі важкого поранення соратники по команді можуть підняти вас — зберігайте терпіння.",
        "Teammates can revive you when you are downed — hold on and stay calm.",
    ),
    (
        "Ядерна ракета в Радтауні вимагає повної подачі живлення на підстанції перед запуском консолі.",
        "The Radtown nuclear missile requires restoring substation power before launching.",
    ),
    (
        "Сокири та кайла мають різну ефективність: якісний інструмент суттєво пришвидшує фарм ресурсів.",
        "Axes and pickaxes vary in efficiency: higher tier tools significantly speed up harvesting.",
    ),
];

fn draw_loading(
    ctx: &egui::Context,
    interface: &mut Interface,
    loading_bg_id: Option<egui::TextureId>,
    bar_bg_id: Option<egui::TextureId>,
    bar_fill_id: Option<egui::TextureId>,
) {
    let dt = ctx.input(|i| i.stable_dt).min(0.1);
    interface.loading_progress = (interface.loading_progress + dt * 0.45).min(1.0);
    interface.loading_timer += dt;
    if interface.loading_timer >= 4.0 {
        interface.loading_timer = 0.0;
        interface.loading_tip_idx = (interface.loading_tip_idx + 1) % LOADING_TIPS.len();
    }
    ctx.request_repaint();

    let screen_rect = ctx.content_rect();
    let avail_width = (screen_rect.width() - 48.0).clamp(280.0, 580.0);
    let bottom_margin = (screen_rect.height() * 0.05).clamp(24.0, 48.0);
    let tip_margin = bottom_margin + (screen_rect.height() * 0.12).clamp(70.0, 110.0);
    let top_margin = (screen_rect.height() * 0.05).clamp(20.0, 48.0);

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("loading_bg_layer"),
    ));

    // Fullscreen Background
    if let Some(tex) = loading_bg_id {
        painter.image(
            tex,
            screen_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        // Vignette overlay
        painter.rect_filled(
            screen_rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(10, 14, 20, 110),
        );
    } else {
        painter.rect_filled(screen_rect, 0.0, egui::Color32::from_rgb(12, 16, 22));
    }

    // Top branding banner
    egui::Area::new(egui::Id::new("loading_header"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, top_margin))
        .interactable(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("MIDNIGHT SURVIVAL")
                        .size(36.0)
                        .strong()
                        .color(egui::Color32::from_rgb(255, 235, 160)),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("ВІДРОДЖЕННЯ ЦИВІЛІЗАЦІЇ · BATTLE ISLAND SIMULATION")
                        .size(13.0)
                        .color(egui::Color32::from_rgb(180, 210, 230)),
                );
            });
        });

    // Middle Tip Card (Rotating Authentic Tips)
    static AUTHENTIC_TIPS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    let auth_tips = AUTHENTIC_TIPS.get_or_init(shared::level::load_imported_loading_tips);
    let tip_idx = interface.loading_tip_idx % LOADING_TIPS.len();
    let (tip_ua, tip_en) = LOADING_TIPS[tip_idx];
    let auth_tip = if !auth_tips.is_empty() {
        auth_tips.get(interface.loading_tip_idx % auth_tips.len()).map(|s| s.as_str())
    } else {
        None
    };

    egui::Area::new(egui::Id::new("loading_tip_panel"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -tip_margin))
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(14, 20, 28, 220))
                .stroke(egui::Stroke::new(
                    1.2_f32,
                    egui::Color32::from_rgb(70, 120, 170),
                ))
                .corner_radius(8.0)
                .inner_margin(egui::Margin::symmetric(24, 14))
                .show(ui, |ui| {
                    ui.set_max_width(avail_width);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "💡 ПОРАДА З ВИЖИВАННЯ (SURVIVAL TIP #{})",
                                tip_idx + 1
                            ))
                            .size(12.5)
                            .strong()
                            .color(egui::Color32::from_rgb(255, 210, 90)),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(tip_ua)
                            .size(14.0)
                            .color(egui::Color32::from_rgb(240, 245, 250)),
                        );
                        ui.add_space(3.0);
                        ui.label(
                            egui::RichText::new(tip_en)
                                .size(11.5)
                                .italics()
                                .color(egui::Color32::from_rgb(160, 185, 205)),
                        );
                        if let Some(auth) = auth_tip {
                            ui.add_space(3.0);
                            ui.label(
                                egui::RichText::new(format!("原版提示: {}", auth))
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(230, 200, 140)),
                            );
                        }
                    });
                });
        });

    // Bottom Bar, Progress Text, and Action Button
    let progress = interface.loading_progress;
    let pct = (progress * 100.0).clamp(0.0, 100.0) as u32;

    let stage_text = if progress < 0.20 {
        "Ініціалізація воксельного рушія та фізики острова... (Initializing Voxel Engine)"
    } else if progress < 0.45 {
        "Завантаження текстур чанків та розбивка секторів A1-D57... (Loading Terrain Chunks)"
    } else if progress < 0.70 {
        "Генерація лісів, кам'яних жил та монументів РТ... (Spawning Flora & Monuments)"
    } else if progress < 0.90 {
        "Синхронізація об'єктів бази, техніки та мережі... (Syncing Entities & Network)"
    } else if progress < 0.99 {
        "Фіналізація шейдерів та підготовка до висадки... (Finalizing Island Shaders)"
    } else {
        "Острів готовий до висадки! (Ready for Deployment)"
    };

    egui::Area::new(egui::Id::new("loading_progress_area"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -bottom_margin))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.set_width(avail_width);
                    ui.label(
                        egui::RichText::new(stage_text)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(200, 225, 240)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}%", pct))
                                .size(16.0)
                                .strong()
                                .color(egui::Color32::from_rgb(255, 215, 80)),
                        );
                    });
                });

                ui.add_space(6.0);

                let bar_width = avail_width;
                let bar_height = 24.0;
                let (rect, _response) =
                    ui.allocate_exact_size(egui::vec2(bar_width, bar_height), egui::Sense::hover());

                if let (Some(bg_tex), Some(fill_tex)) = (bar_bg_id, bar_fill_id) {
                    ui.painter().image(
                        bg_tex,
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                    let inner_rect = rect.shrink2(egui::vec2(2.5, 3.0));
                    let fill_w = inner_rect.width() * progress;
                    if fill_w > 1.0 {
                        let fill_rect = egui::Rect::from_min_size(
                            inner_rect.min,
                            egui::vec2(fill_w, inner_rect.height()),
                        );
                        ui.painter().image(
                            fill_tex,
                            fill_rect,
                            egui::Rect::from_min_max(
                                egui::pos2(0.0, 0.0),
                                egui::pos2(progress.clamp(0.01, 1.0), 1.0),
                            ),
                            egui::Color32::WHITE,
                        );
                    }
                } else {
                    ui.painter().rect_filled(
                        rect,
                        6.0,
                        egui::Color32::from_rgba_unmultiplied(20, 28, 38, 220),
                    );
                    ui.painter().rect_stroke(
                        rect,
                        6.0,
                        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(60, 100, 140)),
                        egui::StrokeKind::Inside,
                    );
                    let fill_w = rect.width() * progress;
                    if fill_w > 2.0 {
                        let fill_rect =
                            egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, rect.height()));
                        ui.painter().rect_filled(
                            fill_rect,
                            6.0,
                            egui::Color32::from_rgb(230, 165, 45),
                        );
                    }
                }

                ui.add_space(10.0);

                if progress >= 0.99 {
                    if ui
                        .add_sized(
                            [280.0, 36.0],
                            egui::Button::new(
                                egui::RichText::new("🚀 УВІЙТИ НА ОСТРІВ (ПРОБІЛ)")
                                    .size(14.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(25, 30, 20)),
                            )
                            .fill(egui::Color32::from_rgb(120, 230, 140)),
                        )
                        .clicked()
                    {
                        interface.mode = UiMode::Playing;
                    }
                } else if ui
                    .add_sized(
                        [180.0, 26.0],
                        egui::Button::new(
                            egui::RichText::new("Пропустити (Esc)")
                                .size(11.5)
                                .color(egui::Color32::from_rgb(160, 180, 200)),
                        ),
                    )
                    .clicked()
                {
                    interface.mode = UiMode::Playing;
                }
            });
        });
}
