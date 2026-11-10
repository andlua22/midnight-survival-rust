use shared::{
    map_projection,
    protocol::ServerTick,
    survival::{
        Inventory, ItemKind, MAX_CALORIES, MAX_HEALTH, MAX_HYDRATION, NUCLEAR_STRIKE_CENTER,
        NUCLEAR_STRIKE_RADIUS, NpcEntity, NuclearStrikeState, PlacedStructure, PlayerSurvival,
        ResourceNode, VehicleEntity, Vitals, cargo_ship_transform,
    },
};

use crate::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2,
};

use crate::hud::WidgetId;

const INK: Color32 = Color32::from_rgb(224, 232, 233);
const MUTED: Color32 = Color32::from_rgb(142, 160, 164);
const ACCENT: Color32 = Color32::from_rgb(106, 207, 183);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundCueKind {
    Footstep,
    Gunfire,
    Vehicle,
    Airdrop,
}

#[derive(Clone, Copy, Debug)]
pub struct VisualSoundCue {
    pub kind: SoundCueKind,
    pub world_pos: [f32; 3],
    pub age_ticks: u32,
    pub intensity: f32,
}

#[derive(Clone, Copy, Default)]
pub struct UiTexturesIds {
    pub bar_bg: Option<egui::TextureId>,
    pub bar_fill_hp: Option<egui::TextureId>,
    pub bar_fill_energy: Option<egui::TextureId>,
    pub hotbar_slot: Option<egui::TextureId>,
    pub hotbar_select: Option<egui::TextureId>,
    pub btn_fire_bg: Option<egui::TextureId>,
    pub btn_fire_icon: Option<egui::TextureId>,
    pub btn_aim_bg: Option<egui::TextureId>,
    pub btn_aim_icon: Option<egui::TextureId>,
    pub btn_jump: Option<egui::TextureId>,
    pub btn_crouch: Option<egui::TextureId>,
    pub btn_reload: Option<egui::TextureId>,
    pub joystick_bg: Option<egui::TextureId>,
    pub map_close: Option<egui::TextureId>,
    pub map_zoom_in: Option<egui::TextureId>,
    pub map_zoom_out: Option<egui::TextureId>,
    pub cargo_icon: Option<egui::TextureId>,
}

pub struct MinimapContext<'a> {
    pub player_position: [f32; 3],
    pub yaw: f32,
    pub nodes: &'a [ResourceNode],
    pub structures: &'a [PlacedStructure],
    pub npcs: &'a [NpcEntity],
    pub vehicles: &'a [VehicleEntity],
    pub nuclear: NuclearStrikeState,
    pub tick: ServerTick,
    pub marker: Option<[f32; 2]>,
    pub map_tex_id: Option<egui::TextureId>,
    pub marker_tex_id: Option<egui::TextureId>,
    pub cargo_tex_id: Option<egui::TextureId>,
    pub vehicle_tex_id: Option<egui::TextureId>,
    pub sound_cues: &'a [VisualSoundCue],
    pub ui_tex: UiTexturesIds,
}

impl<'a> Default for MinimapContext<'a> {
    fn default() -> Self {
        Self {
            player_position: [0.0, 2.0, 12.0],
            yaw: 0.0,
            nodes: &[],
            structures: &[],
            npcs: &[],
            vehicles: &[],
            nuclear: NuclearStrikeState::default(),
            tick: ServerTick(0),
            marker: None,
            map_tex_id: None,
            marker_tex_id: None,
            cargo_tex_id: None,
            vehicle_tex_id: None,
            sound_cues: &[],
            ui_tex: UiTexturesIds::default(),
        }
    }
}

pub fn paint_widget(
    painter: &Painter,
    rect: Rect,
    id: WidgetId,
    scale: f32,
    opacity: f32,
    survival: &PlayerSurvival,
    map_ctx: &MinimapContext<'_>,
) {
    let mut painter = painter.with_clip_rect(rect);
    painter.set_opacity(opacity);
    match id {
        WidgetId::HealthBars => health(&painter, rect, scale, &survival.vitals, map_ctx),
        WidgetId::Hotbar => hotbar(&painter, rect, scale, &survival.inventory, map_ctx),
        // The source minimap has one rendering path in draw_widgets.
        WidgetId::Minimap => {}
        WidgetId::VirtualJoystickLeft => joystick(&painter, rect, map_ctx),
        _ => action_button(&painter, rect, id, scale, map_ctx),
    }
}

fn panel(painter: &Painter, rect: Rect, scale: f32) {
    painter.rect_filled(
        rect,
        8.0 * scale,
        Color32::from_rgba_unmultiplied(14, 23, 29, 235),
    );
    painter.rect_stroke(
        rect,
        8.0 * scale,
        Stroke::new(scale, Color32::from_rgb(55, 75, 82)),
        StrokeKind::Inside,
    );
}

fn health(painter: &Painter, rect: Rect, scale: f32, vitals: &Vitals, ctx: &MinimapContext<'_>) {
    panel(painter, rect, scale);
    let origin = rect.min;
    let text = |x, y, value: &str, color, size| {
        painter.text(
            origin + Vec2::new(x, y) * scale,
            Align2::LEFT_TOP,
            value,
            FontId::proportional(size * scale),
            color,
        );
    };
    text(12.0, 9.0, "СТАН", INK, 11.0);
    let temp_str = if vitals.radiation > 1.0 {
        format!("{:.1}°C | {:.0} РАД", vitals.body_temp_c, vitals.radiation)
    } else {
        format!("{:.1}°C", vitals.body_temp_c)
    };
    text(58.0, 9.5, &temp_str, MUTED, 9.0);
    let (status_label, status_color) = if vitals.health < 30.0 {
        ("КРИТИЧНО", Color32::from_rgb(232, 95, 92))
    } else if vitals.radiation > 10.0 {
        ("РАДІАЦІЯ", Color32::from_rgb(192, 242, 68))
    } else if !vitals.can_sprint() {
        ("СПРАГА", Color32::from_rgb(235, 176, 86))
    } else if vitals.body_temp_c < 35.4 {
        ("ХОЛОД", Color32::from_rgb(122, 196, 245))
    } else if vitals.body_temp_c > 37.8 {
        ("СПЕКА", Color32::from_rgb(242, 152, 82))
    } else if vitals.comfort > 0.0 {
        ("КОМФОРТ", Color32::from_rgb(240, 204, 112))
    } else {
        ("НОРМА", ACCENT)
    };
    text(158.0, 9.0, status_label, status_color, 10.0);
    for (index, (label, current, max, color, fill_tex)) in [
        (
            "ЗДОРОВ’Я",
            vitals.health,
            MAX_HEALTH,
            ACCENT,
            ctx.ui_tex.bar_fill_hp,
        ),
        (
            "ВОДА",
            vitals.hydration,
            MAX_HYDRATION,
            Color32::from_rgb(104, 166, 223),
            ctx.ui_tex.bar_fill_energy,
        ),
        (
            "ЇЖА",
            vitals.calories,
            MAX_CALORIES,
            Color32::from_rgb(224, 178, 96),
            ctx.ui_tex.bar_fill_energy,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let top = 30.0 + index as f32 * 24.0;
        let value = format!("{:.0}", current.clamp(0.0, max));
        text(12.0, top, label, MUTED, 10.0);
        text(190.0, top, &value, INK, 11.0);
        let track = Rect::from_min_size(
            origin + Vec2::new(90.0, top + 3.0) * scale,
            Vec2::new(88.0, 8.0) * scale,
        );
        if let Some(bg_tex) = ctx.ui_tex.bar_bg {
            painter.image(
                bg_tex,
                track,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.rect_filled(track, 2.0 * scale, Color32::from_rgb(28, 41, 47));
        }
        let ratio = (current / max).clamp(0.0, 1.0);
        if ratio > 0.0 {
            let fill =
                Rect::from_min_size(track.min, Vec2::new(track.width() * ratio, track.height()));
            if let Some(f_tex) = fill_tex {
                painter.image(
                    f_tex,
                    fill,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(ratio, 1.0)),
                    Color32::WHITE,
                );
            } else {
                painter.rect_filled(fill, 2.0 * scale, color);
            }
        }
    }
}

fn hotbar(
    painter: &Painter,
    rect: Rect,
    scale: f32,
    inventory: &Inventory,
    ctx: &MinimapContext<'_>,
) {
    panel(painter, rect, scale);
    let padding = 5.0 * scale;
    let width = (rect.width() - padding * 7.0) / 6.0;
    for slot in 0..6 {
        let slot_rect = Rect::from_min_size(
            rect.min + Vec2::new(padding + slot as f32 * (width + padding), padding),
            Vec2::new(width, rect.height() - padding * 2.0),
        );
        if let Some(slot_tex) = ctx.ui_tex.hotbar_slot {
            painter.image(
                slot_tex,
                slot_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.rect_filled(slot_rect, 4.0 * scale, Color32::from_rgb(29, 42, 47));
        }
        if slot == inventory.active_slot as usize {
            if let Some(sel_tex) = ctx.ui_tex.hotbar_select {
                painter.image(
                    sel_tex,
                    slot_rect.expand(2.0 * scale),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::from_rgba_unmultiplied(255, 230, 120, 220),
                );
            } else {
                painter.rect_stroke(
                    slot_rect,
                    4.0 * scale,
                    Stroke::new(1.6 * scale, ACCENT),
                    StrokeKind::Inside,
                );
            }
        }
        if let Some(stack) = inventory.hotbar[slot] {
            paint_item_icon(painter, slot_rect.center(), scale * 0.9, stack.kind);
            painter.text(
                slot_rect.center_bottom() - Vec2::new(0.0, 3.0 * scale),
                Align2::CENTER_BOTTOM,
                stack.kind.short_label(),
                FontId::proportional(7.5 * scale),
                INK,
            );
            if stack.count > 1 {
                painter.text(
                    slot_rect.right_top() + Vec2::new(-3.0 * scale, 3.0 * scale),
                    Align2::RIGHT_TOP,
                    format!("×{}", stack.count),
                    FontId::proportional(9.0 * scale),
                    ACCENT,
                );
            }
        }
        painter.text(
            slot_rect.min + Vec2::splat(4.0 * scale),
            Align2::LEFT_TOP,
            (slot + 1).to_string(),
            FontId::proportional(10.0 * scale),
            MUTED,
        );
    }
}

pub fn paint_item_icon(painter: &Painter, center: Pos2, scale: f32, kind: ItemKind) {
    match kind {
        ItemKind::Rock | ItemKind::Stone => {
            let points = [
                [-12.0, 4.0],
                [-8.0, -7.0],
                [1.0, -10.0],
                [12.0, -1.0],
                [8.0, 10.0],
                [-4.0, 11.0],
            ]
            .map(|p| center + Vec2::from(p) * scale)
            .to_vec();
            let fill = if kind == ItemKind::Rock {
                Color32::from_rgb(122, 132, 134)
            } else {
                Color32::from_rgb(102, 128, 146)
            };
            painter.add(Shape::convex_polygon(
                points,
                fill,
                Stroke::new(scale, MUTED),
            ));
        }
        ItemKind::StoneHatchet | ItemKind::StonePickaxe => {
            painter.line_segment(
                [
                    center + Vec2::new(-9.0, 10.0) * scale,
                    center + Vec2::new(7.0, -8.0) * scale,
                ],
                Stroke::new(3.0 * scale, Color32::from_rgb(168, 124, 79)),
            );
            let blade = if kind == ItemKind::StoneHatchet {
                vec![
                    center + Vec2::new(2.0, -11.0) * scale,
                    center + Vec2::new(12.0, -4.0) * scale,
                    center + Vec2::new(5.0, 3.0) * scale,
                ]
            } else {
                vec![
                    center + Vec2::new(-3.0, -11.0) * scale,
                    center + Vec2::new(13.0, -9.0) * scale,
                    center + Vec2::new(9.0, 4.0) * scale,
                ]
            };
            painter.add(Shape::convex_polygon(
                blade,
                Color32::from_rgb(138, 150, 156),
                Stroke::new(scale, INK),
            ));
        }
        ItemKind::WoodenSpear => {
            painter.line_segment(
                [
                    center + Vec2::new(-11.0, 11.0) * scale,
                    center + Vec2::new(9.0, -9.0) * scale,
                ],
                Stroke::new(2.6 * scale, Color32::from_rgb(176, 132, 82)),
            );
            let tip = vec![
                center + Vec2::new(12.0, -12.0) * scale,
                center + Vec2::new(10.0, -4.0) * scale,
                center + Vec2::new(4.0, -10.0) * scale,
            ];
            painter.add(Shape::convex_polygon(tip, INK, Stroke::NONE));
        }
        ItemKind::BuildingPlan => {
            let sheet = Rect::from_center_size(center, Vec2::new(19.0, 16.0) * scale);
            painter.rect_filled(sheet, 2.5 * scale, Color32::from_rgb(44, 114, 178));
            painter.rect_stroke(
                sheet,
                2.5 * scale,
                Stroke::new(scale, Color32::from_rgb(166, 218, 252)),
                StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    center + Vec2::new(-5.0, 0.0) * scale,
                    center + Vec2::new(5.0, 0.0) * scale,
                ],
                Stroke::new(0.9 * scale, INK),
            );
            painter.line_segment(
                [
                    center + Vec2::new(0.0, -4.0) * scale,
                    center + Vec2::new(0.0, 4.0) * scale,
                ],
                Stroke::new(0.9 * scale, INK),
            );
        }
        ItemKind::Hammer => {
            painter.line_segment(
                [
                    center + Vec2::new(-8.0, 10.0) * scale,
                    center + Vec2::new(4.0, -4.0) * scale,
                ],
                Stroke::new(2.8 * scale, Color32::from_rgb(168, 124, 79)),
            );
            let head = Rect::from_center_size(
                center + Vec2::new(4.0, -6.0) * scale,
                Vec2::new(13.0, 6.5) * scale,
            );
            painter.rect_filled(head, 1.5 * scale, Color32::from_rgb(198, 152, 96));
            painter.rect_stroke(
                head,
                1.5 * scale,
                Stroke::new(scale, INK),
                StrokeKind::Inside,
            );
        }
        ItemKind::Wood => {
            for offset in [-4.0, 3.0] {
                let log = Rect::from_center_size(
                    center + Vec2::new(0.0, offset * scale),
                    Vec2::new(22.0, 6.0) * scale,
                );
                painter.rect_filled(log, 2.0 * scale, Color32::from_rgb(154, 106, 66));
                painter.rect_stroke(
                    log,
                    2.0 * scale,
                    Stroke::new(0.8 * scale, Color32::from_rgb(198, 152, 104)),
                    StrokeKind::Inside,
                );
            }
        }
        ItemKind::Bandage => {
            let box_rect = Rect::from_center_size(center, Vec2::new(18.0, 14.0) * scale);
            painter.rect_filled(box_rect, 3.0 * scale, Color32::from_rgb(214, 220, 218));
            let cross = Color32::from_rgb(212, 74, 74);
            painter.rect_filled(
                Rect::from_center_size(center, Vec2::new(8.0, 2.6) * scale),
                1.0 * scale,
                cross,
            );
            painter.rect_filled(
                Rect::from_center_size(center, Vec2::new(2.6, 8.0) * scale),
                1.0 * scale,
                cross,
            );
        }
        ItemKind::WaterBottle => {
            let body = Rect::from_center_size(
                center + Vec2::new(0.0, 2.0 * scale),
                Vec2::new(13.0, 16.0) * scale,
            );
            painter.rect_filled(body, 3.0 * scale, Color32::from_rgb(84, 156, 214));
            let cap = Rect::from_center_size(
                center + Vec2::new(0.0, -8.0 * scale),
                Vec2::new(6.0, 4.0) * scale,
            );
            painter.rect_filled(cap, 1.0 * scale, INK);
        }
        ItemKind::Ration => {
            let pack = Rect::from_center_size(center, Vec2::new(18.0, 15.0) * scale);
            painter.rect_filled(pack, 3.0 * scale, Color32::from_rgb(208, 158, 78));
            painter.rect_stroke(
                pack,
                3.0 * scale,
                Stroke::new(scale, Color32::from_rgb(238, 198, 126)),
                StrokeKind::Inside,
            );
        }
        ItemKind::HuntingBow => {
            painter.line_segment(
                [
                    center + Vec2::new(-7.0, -11.0) * scale,
                    center + Vec2::new(-7.0, 11.0) * scale,
                ],
                Stroke::new(1.2 * scale, INK),
            );
            let arc = [
                [-7.0, -11.0],
                [4.0, -6.0],
                [8.0, 0.0],
                [4.0, 6.0],
                [-7.0, 11.0],
            ]
            .map(|p| center + Vec2::from(p) * scale);
            for pair in arc.windows(2) {
                painter.line_segment(
                    [pair[0], pair[1]],
                    Stroke::new(2.5 * scale, Color32::from_rgb(176, 126, 72)),
                );
            }
            painter.line_segment(
                [
                    center + Vec2::new(-10.0, 0.0) * scale,
                    center + Vec2::new(11.0, 0.0) * scale,
                ],
                Stroke::new(1.6 * scale, ACCENT),
            );
        }
        ItemKind::Revolver => {
            let barrel = Rect::from_min_size(
                center + Vec2::new(-8.0, -6.0) * scale,
                Vec2::new(17.0, 5.0) * scale,
            );
            let grip = Rect::from_min_size(
                center + Vec2::new(-8.0, -2.0) * scale,
                Vec2::new(6.0, 11.0) * scale,
            );
            painter.rect_filled(barrel, 1.5 * scale, Color32::from_rgb(154, 168, 176));
            painter.rect_filled(grip, 1.5 * scale, Color32::from_rgb(138, 92, 58));
        }
        ItemKind::AssaultRifle => {
            let body = Rect::from_center_size(
                center + Vec2::new(0.0, -2.0 * scale),
                Vec2::new(22.0, 5.0) * scale,
            );
            let mag = Rect::from_center_size(
                center + Vec2::new(2.0 * scale, 5.0 * scale),
                Vec2::new(4.5, 9.0) * scale,
            );
            let stock = Rect::from_center_size(
                center + Vec2::new(-9.0 * scale, 2.0 * scale),
                Vec2::new(6.0, 5.0) * scale,
            );
            painter.rect_filled(body, 1.2 * scale, Color32::from_rgb(92, 106, 114));
            painter.rect_filled(mag, 1.2 * scale, Color32::from_rgb(208, 134, 58));
            painter.rect_filled(stock, 1.2 * scale, Color32::from_rgb(148, 98, 56));
        }
        ItemKind::MetalFragments => {
            let shard = vec![
                center + Vec2::new(-9.0, 6.0) * scale,
                center + Vec2::new(-2.0, -10.0) * scale,
                center + Vec2::new(10.0, -3.0) * scale,
                center + Vec2::new(4.0, 9.0) * scale,
            ];
            painter.add(Shape::convex_polygon(
                shard,
                Color32::from_rgb(156, 182, 198),
                Stroke::new(scale, INK),
            ));
        }
        ItemKind::Scrap => {
            painter.circle_filled(center, 8.5 * scale, Color32::from_rgb(166, 122, 68));
            painter.circle_stroke(center, 8.5 * scale, Stroke::new(1.8 * scale, INK));
            painter.circle_filled(center, 3.5 * scale, Color32::from_rgb(28, 41, 47));
        }
        ItemKind::Arrow => {
            painter.line_segment(
                [
                    center + Vec2::new(-9.0, 9.0) * scale,
                    center + Vec2::new(9.0, -9.0) * scale,
                ],
                Stroke::new(2.0 * scale, Color32::from_rgb(188, 146, 92)),
            );
            painter.circle_filled(center + Vec2::new(9.0, -9.0) * scale, 2.5 * scale, INK);
        }
        ItemKind::PistolAmmo | ItemKind::RifleAmmo => {
            let tint = if kind == ItemKind::PistolAmmo {
                Color32::from_rgb(224, 182, 82)
            } else {
                Color32::from_rgb(236, 144, 68)
            };
            for ox in [-5.0, 0.0, 5.0] {
                let shell = Rect::from_center_size(
                    center + Vec2::new(ox * scale, 1.0 * scale),
                    Vec2::new(3.4, 12.0) * scale,
                );
                painter.rect_filled(shell, 1.2 * scale, tint);
            }
        }
        ItemKind::ElectricFuse => {
            let body = Rect::from_center_size(center, Vec2::new(9.0, 18.0) * scale);
            painter.rect_filled(body, 2.5 * scale, Color32::from_rgb(92, 198, 236));
            painter.rect_stroke(
                body,
                2.5 * scale,
                Stroke::new(1.2 * scale, Color32::from_rgb(238, 192, 84)),
                StrokeKind::Inside,
            );
        }
        ItemKind::GreenKeycard => {
            let card = Rect::from_center_size(center, Vec2::new(20.0, 13.0) * scale);
            painter.rect_filled(card, 2.5 * scale, Color32::from_rgb(38, 168, 88));
            painter.rect_stroke(
                card,
                2.5 * scale,
                Stroke::new(scale, INK),
                StrokeKind::Inside,
            );
            let stripe = Rect::from_center_size(
                center + Vec2::new(0.0, -2.5 * scale),
                Vec2::new(18.0, 2.5) * scale,
            );
            painter.rect_filled(stripe, 0.5 * scale, Color32::from_rgb(18, 44, 28));
        }
        ItemKind::AntiRadPills => {
            let pill = Rect::from_center_size(center, Vec2::new(16.0, 9.0) * scale);
            painter.rect_filled(pill, 4.5 * scale, Color32::from_rgb(238, 206, 56));
            painter.rect_stroke(
                pill,
                4.5 * scale,
                Stroke::new(scale, Color32::from_rgb(218, 72, 62)),
                StrokeKind::Inside,
            );
        }
        ItemKind::LowGradeFuel => {
            let can = Rect::from_center_size(
                center + Vec2::new(0.0, 1.5 * scale),
                Vec2::new(14.0, 16.0) * scale,
            );
            painter.rect_filled(can, 2.5 * scale, Color32::from_rgb(214, 146, 42));
            painter.rect_stroke(
                can,
                2.5 * scale,
                Stroke::new(scale, Color32::from_rgb(248, 212, 118)),
                StrokeKind::Inside,
            );
            painter.circle_filled(center, 3.0 * scale, Color32::from_rgb(48, 32, 16));
        }
        ItemKind::C4Explosive => {
            let pack = Rect::from_center_size(center, Vec2::new(18.0, 13.0) * scale);
            painter.rect_filled(pack, 2.0 * scale, Color32::from_rgb(112, 122, 98));
            painter.rect_stroke(
                pack,
                2.0 * scale,
                Stroke::new(1.1 * scale, Color32::from_rgb(38, 44, 36)),
                StrokeKind::Inside,
            );
            painter.circle_filled(
                center + Vec2::new(4.0, -2.0) * scale,
                2.4 * scale,
                Color32::from_rgb(244, 62, 56),
            );
        }
        ItemKind::TeleportGun => {
            let body = Rect::from_center_size(
                center + Vec2::new(0.0, 1.0) * scale,
                Vec2::new(18.0, 9.0) * scale,
            );
            painter.rect_filled(body, 2.0 * scale, Color32::from_rgb(28, 44, 62));
            painter.rect_stroke(
                body,
                2.0 * scale,
                Stroke::new(1.2 * scale, Color32::from_rgb(54, 218, 245)),
                StrokeKind::Inside,
            );
            painter.circle_filled(
                center + Vec2::new(5.0, 1.0) * scale,
                3.2 * scale,
                Color32::from_rgb(64, 240, 255),
            );
            painter.circle_stroke(
                center + Vec2::new(5.0, 1.0) * scale,
                3.2 * scale,
                Stroke::new(1.0 * scale, Color32::WHITE),
            );
        }
    }
}

/// World XZ -> map position, with canonical north (-Z) drawn at the top.
/// `MAP_OVERVIEW_HALF_EXTENT` is the current overview calibration.
pub fn world_to_map_pos(map: Rect, world_xz: [f32; 2]) -> Pos2 {
    let [u, v] =
        map_projection::world_to_overview_uv(world_xz, shared::level::MAP_OVERVIEW_HALF_EXTENT)
            .map(|uv| uv.clamp(0.0, 1.0));
    Pos2::new(map.min.x + u * map.width(), map.min.y + v * map.height())
}

pub fn map_pos_to_world(map: Rect, pos: Pos2) -> [f32; 2] {
    let u = ((pos.x - map.min.x) / map.width()).clamp(0.0, 1.0);
    let v = ((pos.y - map.min.y) / map.height()).clamp(0.0, 1.0);
    map_projection::overview_uv_to_world([u, v], shared::level::MAP_OVERVIEW_HALF_EXTENT)
}

pub fn paint_tactical_map(
    painter: &Painter,
    map: Rect,
    scale: f32,
    ctx: &MinimapContext<'_>,
    detailed_labels: bool,
) {
    let _ = detailed_labels;
    painter.rect_filled(map, 4.0 * scale, Color32::from_rgb(18, 30, 36));
    let w = map.width();
    let h = map.height();

    // Macroscopic satellite overview texture or fallback grid
    if let Some(map_tex) = ctx.map_tex_id {
        painter.image(
            map_tex,
            map,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        painter.rect_filled(map, 0.0, Color32::from_rgba_unmultiplied(0, 0, 0, 18));
    } else {
        static MAP_GRID: std::sync::OnceLock<Option<Vec<u8>>> = std::sync::OnceLock::new();
        if let Some(grid_rgb) = MAP_GRID.get_or_init(shared::level::load_imported_map_grid) {
            let cells = 48usize;
            let cw = w / cells as f32;
            let ch = h / cells as f32;
            for gy in 0..cells {
                for gx in 0..cells {
                    let sx = gx * 128 / cells;
                    let sy = gy * 128 / cells;
                    let idx = (sy * 128 + sx) * 3;
                    let r = grid_rgb[idx];
                    let g = grid_rgb[idx + 1];
                    let b = grid_rgb[idx + 2];
                    let cell_rect = Rect::from_min_size(
                        map.min + Vec2::new(gx as f32 * cw, gy as f32 * ch),
                        Vec2::new(cw + 0.6, ch + 0.6),
                    );
                    painter.rect_filled(
                        cell_rect,
                        0.0,
                        Color32::from_rgba_unmultiplied(r, g, b, 220),
                    );
                }
            }
        } else {
            // Fallback biomes if grid file is not present
            let third_h = h / 3.0;
            let winter_rect = Rect::from_min_size(map.min, Vec2::new(w, third_h));
            painter.rect_filled(winter_rect, 0.0, Color32::from_rgb(118, 144, 162));
            let temp_rect = Rect::from_min_size(
                map.min + Vec2::new(0.0, third_h),
                Vec2::new(w * (2.0 / 3.0), third_h),
            );
            painter.rect_filled(temp_rect, 0.0, Color32::from_rgb(46, 74, 54));
            let jungle_rect = Rect::from_min_size(
                map.min + Vec2::new(w * (2.0 / 3.0), third_h),
                Vec2::new(w / 3.0, third_h),
            );
            painter.rect_filled(jungle_rect, 0.0, Color32::from_rgb(22, 108, 52));
            let desert_rect = Rect::from_min_size(
                map.min + Vec2::new(0.0, third_h * 2.0),
                Vec2::new(w, third_h),
            );
            painter.rect_filled(desert_rect, 0.0, Color32::from_rgb(132, 106, 62));
        }
    }

    // 6x6 Sector tactical grid & labels (A-01..D-56) over the 6x6 km main island
    let grid_stroke = Stroke::new(
        scale * 0.7,
        Color32::from_rgba_unmultiplied(210, 228, 220, 45),
    );
    for index in 0..=6 {
        let k = -3000.0 + index as f32 * 1000.0;
        painter.line_segment(
            [
                world_to_map_pos(map, [k, -3000.0]),
                world_to_map_pos(map, [k, 3000.0]),
            ],
            grid_stroke,
        );
        painter.line_segment(
            [
                world_to_map_pos(map, [-3000.0, k]),
                world_to_map_pos(map, [3000.0, k]),
            ],
            grid_stroke,
        );
    }
    let grid_font = FontId::proportional(7.0 * scale);
    for gy in 0..6 {
        for gx in 0..6 {
            // Rows go north -> south; north is -Z in world space.
            let wx = -2500.0 + gx as f32 * 1000.0;
            let wz = -2500.0 + gy as f32 * 1000.0;
            let code = shared::level::sector_label_at([wx, 0.0, wz]);
            let cell_min = world_to_map_pos(map, [wx - 500.0, wz - 500.0]);
            painter.text(
                cell_min + Vec2::new(2.0, 2.0) * scale,
                Align2::LEFT_TOP,
                code,
                grid_font.clone(),
                Color32::from_rgba_unmultiplied(235, 245, 242, 140),
            );
        }
    }

    // Western Military Rocket Base Island (junshijidi) tactical grid & rocket marker
    let mil_grid_stroke = Stroke::new(
        scale * 0.65,
        Color32::from_rgba_unmultiplied(240, 190, 80, 55),
    );
    let mil_col_xs = [-4350.0, -4150.0, -3800.0, -3450.0, -3200.0];
    let mil_row_zs = [-750.0, -570.0, -200.0, 150.0, 320.0];
    for &x in &mil_col_xs {
        painter.line_segment(
            [
                world_to_map_pos(map, [x, -750.0]),
                world_to_map_pos(map, [x, 320.0]),
            ],
            mil_grid_stroke,
        );
    }
    for &z in &mil_row_zs {
        painter.line_segment(
            [
                world_to_map_pos(map, [-4350.0, z]),
                world_to_map_pos(map, [-3200.0, z]),
            ],
            mil_grid_stroke,
        );
    }
    for r in 0..4 {
        for c in 0..4 {
            let wx = (mil_col_xs[c] + mil_col_xs[c + 1]) * 0.5;
            let wz = (mil_row_zs[r] + mil_row_zs[r + 1]) * 0.5;
            let code = shared::level::sector_label_at([wx, 0.0, wz]);
            let cell_min = world_to_map_pos(map, [mil_col_xs[c], mil_row_zs[r]]);
            painter.text(
                cell_min + Vec2::new(1.5, 1.5) * scale,
                Align2::LEFT_TOP,
                code,
                grid_font.clone(),
                Color32::from_rgba_unmultiplied(255, 215, 100, 130),
            );
        }
    }

    // Authentic Rocket Launcher Facility Marker (InitLauncherPos from BattleMapPanel.cs)
    let launcher_pos = world_to_map_pos(map, [-3750.0, -550.0]);
    painter.circle_filled(
        launcher_pos,
        5.5 * scale,
        Color32::from_rgba_unmultiplied(240, 75, 45, 180),
    );
    painter.circle_stroke(
        launcher_pos,
        8.5 * scale,
        Stroke::new(1.4 * scale, Color32::from_rgb(255, 195, 75)),
    );
    painter.line_segment(
        [
            launcher_pos - Vec2::new(11.0 * scale, 0.0),
            launcher_pos + Vec2::new(11.0 * scale, 0.0),
        ],
        Stroke::new(0.9 * scale, Color32::from_rgb(255, 215, 80)),
    );
    painter.line_segment(
        [
            launcher_pos - Vec2::new(0.0, 11.0 * scale),
            launcher_pos + Vec2::new(0.0, 11.0 * scale),
        ],
        Stroke::new(0.9 * scale, Color32::from_rgb(255, 215, 80)),
    );
    painter.text(
        launcher_pos + Vec2::new(0.0, -12.0 * scale),
        Align2::CENTER_BOTTOM,
        "РАКЕТНИЙ КОМПЛЕКС (LAUNCHER)",
        FontId::proportional(8.0 * scale),
        Color32::from_rgb(255, 215, 90),
    );
    painter.text(
        launcher_pos + Vec2::new(0.0, 12.0 * scale),
        Align2::CENTER_TOP,
        "ВІЙСЬКОВА БАЗА [JUNSHIJIDI]",
        FontId::proportional(7.5 * scale),
        Color32::from_rgb(235, 180, 80),
    );

    // Visual sound cues on tactical map
    for cue in ctx.sound_cues {
        let spos = world_to_map_pos(map, [cue.world_pos[0], cue.world_pos[2]]);
        let alpha = (cue.intensity.clamp(0.0, 1.0) * 200.0) as u8;
        let radius = match cue.kind {
            SoundCueKind::Gunfire => 9.0 * scale,
            SoundCueKind::Airdrop => 14.0 * scale,
            SoundCueKind::Vehicle => 7.0 * scale,
            SoundCueKind::Footstep => 4.0 * scale,
        };
        let color = match cue.kind {
            SoundCueKind::Gunfire => Color32::from_rgba_unmultiplied(255, 55, 40, alpha),
            SoundCueKind::Airdrop => Color32::from_rgba_unmultiplied(75, 220, 245, alpha),
            SoundCueKind::Vehicle => Color32::from_rgba_unmultiplied(250, 210, 50, alpha),
            SoundCueKind::Footstep => Color32::from_rgba_unmultiplied(255, 175, 55, alpha),
        };
        painter.circle_stroke(spos, radius, Stroke::new(1.2 * scale, color));
    }

    // Nuclear Strike Blast Zone
    if ctx.nuclear.is_active() {
        let nuke_pos = world_to_map_pos(map, [NUCLEAR_STRIKE_CENTER[0], NUCLEAR_STRIKE_CENTER[2]]);
        let nuke_r = (NUCLEAR_STRIKE_RADIUS
            * (w / (2.0 * shared::level::MAP_OVERVIEW_HALF_EXTENT)))
            .max(14.0 * scale);
        let pulse = ((ctx.tick.0 as f32 * 0.15).sin() * 0.5 + 0.5 * 80.0) as u8 + 45;
        painter.circle_filled(
            nuke_pos,
            nuke_r,
            Color32::from_rgba_unmultiplied(242, 48, 42, pulse),
        );
        painter.circle_stroke(
            nuke_pos,
            nuke_r,
            Stroke::new(1.8 * scale, Color32::from_rgb(255, 82, 72)),
        );
        painter.text(
            nuke_pos,
            Align2::CENTER_CENTER,
            "ЯДЕРНИЙ УДАР!",
            FontId::proportional(10.5 * scale),
            Color32::from_rgb(255, 226, 196),
        );
    }

    // Cargo Ship on the ocean perimeter
    let (ship_pos3, _) = cargo_ship_transform(ctx.tick);
    let ship_lim = shared::level::MAP_OVERVIEW_HALF_EXTENT - 50.0;
    let ship_map = world_to_map_pos(
        map,
        [
            ship_pos3[0].clamp(-ship_lim, ship_lim),
            ship_pos3[2].clamp(-ship_lim, ship_lim),
        ],
    );
    if let Some(cargo_tex) = ctx.cargo_tex_id.or(ctx.ui_tex.cargo_icon) {
        let ship_rect = Rect::from_center_size(ship_map, Vec2::splat(22.0 * scale));
        painter.image(
            cargo_tex,
            ship_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        painter.rect_filled(
            Rect::from_center_size(ship_map, Vec2::new(8.0, 5.0) * scale),
            1.2 * scale,
            Color32::from_rgb(228, 178, 72),
        );
    }
    painter.text(
        ship_map + Vec2::new(0.0, -11.0 * scale),
        Align2::CENTER_BOTTOM,
        "КОРАБЕЛЬ",
        FontId::proportional(8.0 * scale),
        Color32::from_rgb(248, 212, 122),
    );

    // Vehicles on tactical map
    for vehicle in ctx.vehicles {
        if vehicle.health == 0 {
            continue;
        }
        let vpos = world_to_map_pos(map, [vehicle.position[0], vehicle.position[2]]);
        if let Some(veh_tex) = ctx.vehicle_tex_id {
            painter.image(
                veh_tex,
                Rect::from_center_size(vpos, Vec2::splat(18.0 * scale)),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.circle_filled(vpos, 3.2 * scale, Color32::from_rgb(76, 224, 246));
        }
    }

    // Custom Waypoint Marker
    if let Some(marker_xz) = ctx.marker {
        let mpos = world_to_map_pos(map, marker_xz);
        let pin_color = Color32::from_rgb(248, 206, 78);
        if let Some(pin_tex) = ctx.marker_tex_id {
            let pin_size = Vec2::new(18.0, 22.0) * scale;
            let pin_rect =
                Rect::from_min_size(mpos - Vec2::new(pin_size.x * 0.5, pin_size.y), pin_size);
            painter.image(
                pin_tex,
                pin_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.circle_stroke(mpos, 6.5 * scale, Stroke::new(1.6 * scale, pin_color));
            painter.circle_filled(mpos, 2.6 * scale, pin_color);
        }
        painter.text(
            mpos + Vec2::new(0.0, 3.0 * scale),
            Align2::CENTER_TOP,
            "МІТКА",
            FontId::proportional(8.5 * scale),
            pin_color,
        );
    }

    // Local player marker & vision arrow
    let player_pos = world_to_map_pos(map, [ctx.player_position[0], ctx.player_position[2]]);
    let (sin, cos) = map_projection::yaw_to_map_bearing(ctx.yaw).sin_cos();
    // Glowing direction vision cone
    let cone_len = 20.0 * scale;
    let cone_half_w = 7.5 * scale;
    let tip = player_pos + Vec2::new(sin * cone_len, -cos * cone_len);
    let left = player_pos
        + Vec2::new(
            -cos * cone_half_w - sin * 3.0 * scale,
            -sin * cone_half_w + cos * 3.0 * scale,
        );
    let right = player_pos
        + Vec2::new(
            cos * cone_half_w - sin * 3.0 * scale,
            sin * cone_half_w + cos * 3.0 * scale,
        );
    painter.add(Shape::convex_polygon(
        vec![player_pos, left, tip, right],
        Color32::from_rgba_unmultiplied(106, 207, 183, 115),
        Stroke::NONE,
    ));
    let arrow = [[0.0, -11.0], [-5.5, 6.5], [0.0, 3.0], [5.5, 6.5]]
        .map(|[x, y]| player_pos + Vec2::new(cos * x - sin * y, sin * x + cos * y) * scale)
        .to_vec();
    painter.add(Shape::convex_polygon(
        arrow,
        ACCENT,
        Stroke::new(1.2 * scale, Color32::from_rgb(14, 23, 29)),
    ));
    painter.circle_filled(player_pos, 3.2 * scale, Color32::WHITE);
    painter.circle_stroke(player_pos, 5.2 * scale, Stroke::new(1.0 * scale, ACCENT));

    // Cardinal island directions
    let card_font = FontId::proportional(9.0 * scale);
    painter.text(
        map.center_top() + Vec2::new(0.0, 4.0 * scale),
        Align2::CENTER_TOP,
        "ПІВНІЧ (АРКТИКА)",
        card_font.clone(),
        Color32::from_rgb(215, 235, 250),
    );
    painter.text(
        map.center_bottom() - Vec2::new(0.0, 4.0 * scale),
        Align2::CENTER_BOTTOM,
        "ПІВДЕНЬ (ПУСТЕЛЯ)",
        card_font.clone(),
        Color32::from_rgb(245, 220, 160),
    );
    painter.text(
        map.left_center() + Vec2::new(4.0 * scale, 0.0),
        Align2::LEFT_CENTER,
        "ЗАХІД (ВІЙСЬКОВИЙ ОСТРІВ)",
        card_font.clone(),
        Color32::from_rgb(245, 210, 110),
    );
    painter.text(
        map.right_center() - Vec2::new(4.0 * scale, 0.0),
        Align2::RIGHT_CENTER,
        "СХІД (ДЖУНГЛІ)",
        card_font,
        Color32::from_rgb(140, 230, 165),
    );

    painter.rect_stroke(
        map,
        4.0 * scale,
        Stroke::new(scale, Color32::from_rgb(62, 86, 92)),
        StrokeKind::Inside,
    );

    // Authentic Map Zoom Controls from BattleMapPanel
    if let Some(in_tex) = ctx.ui_tex.map_zoom_in {
        let btn_rect = Rect::from_center_size(
            map.right_top() + Vec2::new(-22.0 * scale, 30.0 * scale),
            Vec2::splat(26.0 * scale),
        );
        painter.image(
            in_tex,
            btn_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    if let Some(out_tex) = ctx.ui_tex.map_zoom_out {
        let btn_rect = Rect::from_center_size(
            map.right_top() + Vec2::new(-22.0 * scale, 62.0 * scale),
            Vec2::splat(26.0 * scale),
        );
        painter.image(
            out_tex,
            btn_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    if let Some(close_tex) = ctx.ui_tex.map_close {
        let btn_rect = Rect::from_center_size(
            map.right_top() + Vec2::new(-22.0 * scale, -10.0 * scale),
            Vec2::splat(20.0 * scale),
        );
        painter.image(
            close_tex,
            btn_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
}

/// Horizontal compass ribbon across the top of the HUD (_compassShowAngle / _mSound2IconInCompass from BattleMinimap.cs)
pub fn paint_compass_bar(
    painter: &Painter,
    center_top: Pos2,
    scale: f32,
    ctx: &MinimapContext<'_>,
) {
    let width = 360.0 * scale;
    let height = 30.0 * scale;
    let rect = Rect::from_center_size(
        center_top + Vec2::new(0.0, height * 0.5),
        Vec2::new(width, height),
    );

    // Background tape with subtle border
    painter.rect_filled(
        rect,
        6.0 * scale,
        Color32::from_rgba_unmultiplied(14, 23, 29, 195),
    );
    painter.rect_stroke(
        rect,
        6.0 * scale,
        Stroke::new(
            1.0 * scale,
            Color32::from_rgba_unmultiplied(55, 75, 82, 140),
        ),
        StrokeKind::Inside,
    );

    let player_yaw_deg = map_projection::yaw_to_map_bearing(ctx.yaw).to_degrees();
    let center_x = rect.center().x;
    let fov_deg = 90.0_f32; // 90° FOV ribbon
    let pixels_per_deg = (width * 0.86) / fov_deg;
    let tick_y_bottom = rect.max.y - 3.5 * scale;

    // Center aiming pip / indicator
    painter.line_segment(
        [
            Pos2::new(center_x, rect.min.y + 2.0 * scale),
            Pos2::new(center_x, rect.min.y + 7.0 * scale),
        ],
        Stroke::new(1.8 * scale, ACCENT),
    );

    // Ticks and cardinal labels
    let min_deg = (player_yaw_deg - fov_deg * 0.5).floor() as i32;
    let max_deg = (player_yaw_deg + fov_deg * 0.5).ceil() as i32;
    let start_tick = (min_deg / 5) * 5;

    let font_cardinal = FontId::proportional(8.5 * scale);
    let font_num = FontId::proportional(7.0 * scale);

    for deg_i in (start_tick..=max_deg).step_by(5) {
        let norm_deg = (deg_i as f32).rem_euclid(360.0);
        let diff = (deg_i as f32 - player_yaw_deg + 540.0).rem_euclid(360.0) - 180.0;
        if diff.abs() > fov_deg * 0.5 {
            continue;
        }
        let x = center_x + diff * pixels_per_deg;
        let is_major = (deg_i % 15) == 0;
        let tick_len = if is_major { 7.0 * scale } else { 3.5 * scale };
        let stroke_col = if is_major {
            Color32::from_rgba_unmultiplied(220, 235, 235, 175)
        } else {
            Color32::from_rgba_unmultiplied(140, 160, 165, 95)
        };
        painter.line_segment(
            [
                Pos2::new(x, tick_y_bottom - tick_len),
                Pos2::new(x, tick_y_bottom),
            ],
            Stroke::new(1.0 * scale, stroke_col),
        );

        if is_major {
            let label = match (norm_deg.round() as u32) % 360 {
                0 => Some(("ПН", ACCENT)),
                45 => Some(("ПН-СХ", INK)),
                90 => Some(("СХ", INK)),
                135 => Some(("ПД-СХ", INK)),
                180 => Some(("ПД", INK)),
                225 => Some(("ПД-ЗХ", INK)),
                270 => Some(("ЗХ", INK)),
                315 => Some(("ПН-ЗХ", INK)),
                _ => None,
            };
            if let Some((text, color)) = label {
                painter.text(
                    Pos2::new(x, rect.min.y + 3.0 * scale),
                    Align2::CENTER_TOP,
                    text,
                    font_cardinal.clone(),
                    color,
                );
            } else if (norm_deg.round() as u32) % 30 == 0 {
                painter.text(
                    Pos2::new(x, rect.min.y + 4.0 * scale),
                    Align2::CENTER_TOP,
                    format!("{:.0}", norm_deg),
                    font_num.clone(),
                    MUTED,
                );
            }
        }
    }

    // Directional Sound Icons on Compass Tape (_mSound2IconInCompass from BattleMinimap.cs)
    for cue in ctx.sound_cues {
        let dx = cue.world_pos[0] - ctx.player_position[0];
        let dz = cue.world_pos[2] - ctx.player_position[2];
        let cue_deg = map_projection::world_bearing([dx, dz]).to_degrees();
        let diff = (cue_deg - player_yaw_deg + 540.0).rem_euclid(360.0) - 180.0;
        let clamped_diff = diff.clamp(-fov_deg * 0.48, fov_deg * 0.48);
        let x = center_x + clamped_diff * pixels_per_deg;
        let y = rect.center().y;
        let alpha = (cue.intensity.clamp(0.0, 1.0) * 255.0) as u8;

        match cue.kind {
            SoundCueKind::Gunfire => {
                let col = Color32::from_rgba_unmultiplied(255, 60, 40, alpha);
                painter.circle_filled(Pos2::new(x, y), 3.2 * scale, col);
                painter.circle_stroke(Pos2::new(x, y), 5.5 * scale, Stroke::new(1.2 * scale, col));
            }
            SoundCueKind::Footstep => {
                let col = Color32::from_rgba_unmultiplied(255, 180, 55, alpha);
                painter.circle_filled(Pos2::new(x - 1.5 * scale, y), 1.6 * scale, col);
                painter.circle_filled(Pos2::new(x + 1.5 * scale, y), 1.6 * scale, col);
            }
            SoundCueKind::Vehicle => {
                let col = Color32::from_rgba_unmultiplied(250, 215, 50, alpha);
                painter.rect_filled(
                    Rect::from_center_size(Pos2::new(x, y), Vec2::splat(4.8 * scale)),
                    1.0 * scale,
                    col,
                );
            }
            SoundCueKind::Airdrop => {
                let col = Color32::from_rgba_unmultiplied(75, 220, 245, alpha);
                painter.circle_filled(Pos2::new(x, y), 2.8 * scale, col);
                painter.circle_stroke(Pos2::new(x, y), 5.0 * scale, Stroke::new(1.0 * scale, col));
            }
        }
    }

    // Waypoint Pin Marker on Compass Tape (GBattlePanelMarkDir)
    if let Some(marker) = ctx.marker {
        let dx = marker[0] - ctx.player_position[0];
        let dz = marker[1] - ctx.player_position[2];
        let dist = (dx * dx + dz * dz).sqrt();
        let marker_deg = map_projection::world_bearing([dx, dz]).to_degrees();
        let diff = (marker_deg - player_yaw_deg + 540.0).rem_euclid(360.0) - 180.0;
        let is_edge = diff.abs() > fov_deg * 0.47;
        let clamped_diff = diff.clamp(-fov_deg * 0.47, fov_deg * 0.47);
        let x = center_x + clamped_diff * pixels_per_deg;
        let y = rect.center().y;

        let pin_col = Color32::from_rgb(255, 205, 60);
        painter.circle_filled(Pos2::new(x, y), 3.5 * scale, pin_col);
        painter.circle_stroke(
            Pos2::new(x, y),
            5.2 * scale,
            Stroke::new(1.0 * scale, Color32::BLACK),
        );
        if is_edge {
            let arrow_char = if diff < 0.0 { "◀" } else { "▶" };
            painter.text(
                Pos2::new(x, y - 7.5 * scale),
                Align2::CENTER_CENTER,
                arrow_char,
                FontId::proportional(7.5 * scale),
                pin_col,
            );
        } else {
            painter.text(
                Pos2::new(x, y + 7.5 * scale),
                Align2::CENTER_TOP,
                format!("{:.0}m", dist),
                FontId::proportional(7.0 * scale),
                pin_col,
            );
        }
    }
}

fn joystick(painter: &Painter, rect: Rect, ctx: &MinimapContext<'_>) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.46;
    if let Some(joy_bg) = ctx.ui_tex.joystick_bg {
        painter.image(
            joy_bg,
            Rect::from_center_size(center, Vec2::splat(radius * 2.1)),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::from_rgba_unmultiplied(255, 255, 255, 220),
        );
    } else {
        painter.circle_filled(
            center,
            radius,
            Color32::from_rgba_unmultiplied(14, 23, 29, 190),
        );
        painter.circle_stroke(center, radius, Stroke::new(1.5_f32, MUTED));
        painter.circle_stroke(
            center,
            radius * 0.65,
            Stroke::new(1.0_f32, Color32::from_rgb(58, 83, 88)),
        );
    }
    painter.circle_filled(
        center,
        radius * 0.27,
        Color32::from_rgba_unmultiplied(106, 207, 183, 160),
    );
}

fn action_button(
    painter: &Painter,
    rect: Rect,
    id: WidgetId,
    scale: f32,
    ctx: &MinimapContext<'_>,
) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.46;

    match id {
        WidgetId::FireButtonLeft | WidgetId::FireButtonRight => {
            if let Some(bg_tex) = ctx.ui_tex.btn_fire_bg {
                painter.image(
                    bg_tex,
                    Rect::from_center_size(center, Vec2::splat(radius * 2.0)),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                painter.circle_filled(
                    center,
                    radius,
                    Color32::from_rgba_unmultiplied(14, 23, 29, 220),
                );
                painter.circle_stroke(center, radius, Stroke::new(scale, MUTED));
            }
            if let Some(icon_tex) = ctx.ui_tex.btn_fire_icon {
                painter.image(
                    icon_tex,
                    Rect::from_center_size(center, Vec2::splat(radius * 1.15)),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                let icon = center - Vec2::new(0.0, 6.0 * scale);
                let stroke = Stroke::new(1.8 * scale, INK);
                painter.circle_stroke(icon, 8.0 * scale, stroke);
            }
        }
        WidgetId::AimButton => {
            if let Some(bg_tex) = ctx.ui_tex.btn_aim_bg {
                painter.image(
                    bg_tex,
                    Rect::from_center_size(center, Vec2::splat(radius * 2.0)),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                painter.circle_filled(
                    center,
                    radius,
                    Color32::from_rgba_unmultiplied(14, 23, 29, 220),
                );
                painter.circle_stroke(center, radius, Stroke::new(scale, MUTED));
            }
            if let Some(icon_tex) = ctx.ui_tex.btn_aim_icon {
                painter.image(
                    icon_tex,
                    Rect::from_center_size(center, Vec2::splat(radius * 1.15)),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        }
        WidgetId::JumpButton => {
            if let Some(btn_tex) = ctx.ui_tex.btn_jump {
                painter.image(
                    btn_tex,
                    Rect::from_center_size(center, Vec2::splat(radius * 2.0)),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                painter.circle_filled(
                    center,
                    radius,
                    Color32::from_rgba_unmultiplied(14, 23, 29, 220),
                );
                painter.circle_stroke(center, radius, Stroke::new(scale, MUTED));
            }
        }
        WidgetId::CrouchButton => {
            if let Some(btn_tex) = ctx.ui_tex.btn_crouch {
                painter.image(
                    btn_tex,
                    Rect::from_center_size(center, Vec2::splat(radius * 2.0)),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                painter.circle_filled(
                    center,
                    radius,
                    Color32::from_rgba_unmultiplied(14, 23, 29, 220),
                );
                painter.circle_stroke(center, radius, Stroke::new(scale, MUTED));
            }
        }
        WidgetId::ReloadButton => {
            if let Some(btn_tex) = ctx.ui_tex.btn_reload {
                painter.image(
                    btn_tex,
                    Rect::from_center_size(center, Vec2::splat(radius * 2.0)),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                painter.circle_filled(
                    center,
                    radius,
                    Color32::from_rgba_unmultiplied(14, 23, 29, 220),
                );
                painter.circle_stroke(center, radius, Stroke::new(scale, MUTED));
            }
        }
        _ => {
            painter.circle_filled(
                center,
                radius,
                Color32::from_rgba_unmultiplied(14, 23, 29, 220),
            );
            painter.circle_stroke(center, radius, Stroke::new(scale, MUTED));
            let icon = center - Vec2::new(0.0, 6.0 * scale);
            let stroke = Stroke::new(1.8 * scale, INK);
            painter.rect_stroke(
                Rect::from_center_size(icon, Vec2::new(18.0, 20.0) * scale),
                3.0 * scale,
                stroke,
                StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    icon + Vec2::new(-5.0, -10.0) * scale,
                    icon + Vec2::new(5.0, -10.0) * scale,
                ],
                stroke,
            );
            painter.text(
                center + Vec2::new(0.0, 18.0 * scale),
                Align2::CENTER_CENTER,
                "РЮКЗАК",
                FontId::proportional(8.5 * scale),
                INK,
            );
        }
    }
}

pub fn crosshair(ctx: &egui::Context) {
    let center = ctx.content_rect().center();
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("crosshair"),
    ));
    let stroke = Stroke::new(1.2_f32, INK);
    for direction in [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y] {
        painter.line_segment([center + direction * 4.0, center + direction * 9.0], stroke);
    }
    painter.circle_filled(center, 1.0, INK);
}

pub fn selection(painter: &Painter, rect: Rect, selected: bool) {
    painter.rect_stroke(
        rect,
        5.0,
        Stroke::new(
            if selected { 2.0_f32 } else { 1.0_f32 },
            if selected { ACCENT } else { MUTED },
        ),
        StrokeKind::Inside,
    );
}

pub fn logical_position(origin: Pos2, position: [f32; 2]) -> Pos2 {
    origin + Vec2::from(position)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::HudPreset;

    #[test]
    fn all_widgets_tessellate_into_valid_meshes_without_a_gpu() {
        let context = egui::Context::default();
        for preset in HudPreset::ALL {
            let layout = crate::hud::HudLayout::preset(preset);
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 720.0))),
                    ..Default::default()
                },
                |ui| {
                    let ctx = ui.ctx();
                    let painter = ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Background,
                        egui::Id::new("widget-mesh-test"),
                    ));
                    for (&id, config) in &layout.widgets {
                        let placement = config.placement(id, [1280.0, 720.0]);
                        paint_widget(
                            &painter,
                            Rect::from_min_size(
                                logical_position(Pos2::ZERO, placement.position),
                                Vec2::from(placement.size),
                            ),
                            id,
                            placement.render_scale,
                            config.opacity,
                            &PlayerSurvival::default(),
                            &MinimapContext::default(),
                        );
                    }
                    crosshair(ctx);
                },
            );
            let shapes = std::mem::take(&mut output.shapes);
            let pixels_per_point = output.pixels_per_point;
            output.textures_delta.clear();
            let primitives = context.tessellate(shapes, pixels_per_point);
            assert!(!primitives.is_empty());
            for primitive in primitives {
                if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
                    assert!(!mesh.vertices.is_empty());
                    assert!(
                        mesh.vertices
                            .iter()
                            .all(|v| v.pos.x.is_finite() && v.pos.y.is_finite())
                    );
                    assert!(
                        mesh.indices
                            .iter()
                            .all(|i| (*i as usize) < mesh.vertices.len())
                    );
                }
            }
        }
    }
}
