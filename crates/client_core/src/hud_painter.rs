use shared::{
    protocol::ServerTick,
    survival::{
        Biome, Inventory, ItemKind, MAX_CALORIES, MAX_HEALTH, MAX_HYDRATION, NUCLEAR_STRIKE_CENTER,
        NUCLEAR_STRIKE_RADIUS, NpcEntity, NpcKind, NuclearStrikeState, PlacedStructure,
        PlayerSurvival, ResourceKind, ResourceNode, StructureKind, TC_PRIVILEGE_RADIUS,
        VehicleEntity, Vitals, cargo_ship_transform,
    },
};

use crate::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2,
};

use crate::hud::WidgetId;

const INK: Color32 = Color32::from_rgb(224, 232, 233);
const MUTED: Color32 = Color32::from_rgb(142, 160, 164);
const ACCENT: Color32 = Color32::from_rgb(106, 207, 183);

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
        WidgetId::HealthBars => health(&painter, rect, scale, &survival.vitals),
        WidgetId::Hotbar => hotbar(&painter, rect, scale, &survival.inventory),
        WidgetId::Minimap => minimap(&painter, rect, scale, &survival.vitals, map_ctx),
        WidgetId::VirtualJoystickLeft => joystick(&painter, rect),
        _ => action_button(&painter, rect, id, scale),
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

fn health(painter: &Painter, rect: Rect, scale: f32, vitals: &Vitals) {
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
    for (index, (label, current, max, color)) in [
        ("ЗДОРОВ’Я", vitals.health, MAX_HEALTH, ACCENT),
        (
            "ВОДА",
            vitals.hydration,
            MAX_HYDRATION,
            Color32::from_rgb(104, 166, 223),
        ),
        (
            "ЇЖА",
            vitals.calories,
            MAX_CALORIES,
            Color32::from_rgb(224, 178, 96),
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
            Vec2::new(88.0, 7.0) * scale,
        );
        painter.rect_filled(track, 2.0 * scale, Color32::from_rgb(28, 41, 47));
        let ratio = (current / max).clamp(0.0, 1.0);
        if ratio > 0.0 {
            let fill =
                Rect::from_min_size(track.min, Vec2::new(track.width() * ratio, track.height()));
            painter.rect_filled(fill, 2.0 * scale, color);
        }
    }
}

fn hotbar(painter: &Painter, rect: Rect, scale: f32, inventory: &Inventory) {
    panel(painter, rect, scale);
    let padding = 5.0 * scale;
    let width = (rect.width() - padding * 7.0) / 6.0;
    for slot in 0..6 {
        let slot_rect = Rect::from_min_size(
            rect.min + Vec2::new(padding + slot as f32 * (width + padding), padding),
            Vec2::new(width, rect.height() - padding * 2.0),
        );
        painter.rect_filled(slot_rect, 4.0 * scale, Color32::from_rgb(29, 42, 47));
        if slot == inventory.active_slot as usize {
            painter.rect_stroke(
                slot_rect,
                4.0 * scale,
                Stroke::new(1.6 * scale, ACCENT),
                StrokeKind::Inside,
            );
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

/// World XZ -> map position. North is -Z (the direction the player faces at yaw 0),
/// drawn at the top of the map, matching the compass and heading arrow.
/// The overview texture spans `MAP_OVERVIEW_HALF_EXTENT` metres around the origin.
pub fn world_to_map_pos(map: Rect, world_xz: [f32; 2]) -> Pos2 {
    let half = shared::level::MAP_OVERVIEW_HALF_EXTENT;
    let u = ((world_xz[0] + half) / (2.0 * half)).clamp(0.0, 1.0);
    // Bevy -Z is north; the overview PNG is drawn north-up, so v grows southward.
    let v = ((world_xz[1] + half) / (2.0 * half)).clamp(0.0, 1.0);
    Pos2::new(map.min.x + u * map.width(), map.min.y + v * map.height())
}

pub fn map_pos_to_world(map: Rect, pos: Pos2) -> [f32; 2] {
    let half = shared::level::MAP_OVERVIEW_HALF_EXTENT;
    let u = ((pos.x - map.min.x) / map.width()).clamp(0.0, 1.0);
    let v = ((pos.y - map.min.y) / map.height()).clamp(0.0, 1.0);
    let wx = u * 2.0 * half - half;
    let wz = v * 2.0 * half - half;
    [wx, wz]
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
    if let Some(cargo_tex) = ctx.cargo_tex_id {
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
    let (sin, cos) = (-ctx.yaw).sin_cos();
    // Glowing direction vision cone
    let cone_len = 20.0 * scale;
    let cone_half_w = 7.5 * scale;
    let tip = player_pos + Vec2::new(-sin * cone_len, -cos * cone_len);
    let left = player_pos
        + Vec2::new(
            -cos * cone_half_w + sin * 3.0 * scale,
            sin * cone_half_w + cos * 3.0 * scale,
        );
    let right = player_pos
        + Vec2::new(
            cos * cone_half_w + sin * 3.0 * scale,
            -sin * cone_half_w + cos * 3.0 * scale,
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
        "ЗАХІД",
        card_font.clone(),
        MUTED,
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
}

pub fn paint_radar_minimap(painter: &Painter, map: Rect, scale: f32, ctx: &MinimapContext<'_>) {
    let center = map.center();
    let radius = map.width().min(map.height()) * 0.46;
    let radar_range = 150.0_f32; // 150 meters local radar range

    // Radar circular base & satellite overview sampling
    if let Some(map_tex) = ctx.map_tex_id {
        let span = 2.0 * shared::level::MAP_OVERVIEW_HALF_EXTENT;
        let u_center = (ctx.player_position[0] + span * 0.5) / span;
        let v_center = (ctx.player_position[2] + span * 0.5) / span;
        let half_span = radar_range / span;
        let uv = Rect::from_min_max(
            Pos2::new(
                (u_center - half_span).clamp(0.0, 1.0),
                (v_center - half_span).clamp(0.0, 1.0),
            ),
            Pos2::new(
                (u_center + half_span).clamp(0.0, 1.0),
                (v_center + half_span).clamp(0.0, 1.0),
            ),
        );
        let radar_rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
        let sub_painter = painter.with_clip_rect(radar_rect);
        sub_painter.circle_filled(center, radius, Color32::from_rgb(18, 28, 34));
        sub_painter.image(
            map_tex,
            radar_rect,
            uv,
            Color32::from_rgba_unmultiplied(225, 225, 225, 230),
        );
        // Soft vignette to highlight radar ring
        painter.circle_stroke(
            center,
            radius - 1.0 * scale,
            Stroke::new(
                2.5 * scale,
                Color32::from_rgba_unmultiplied(12, 22, 28, 140),
            ),
        );
    } else {
        painter.circle_filled(
            center,
            radius,
            Color32::from_rgba_unmultiplied(16, 28, 34, 220),
        );
    }

    painter.circle_stroke(
        center,
        radius,
        Stroke::new(1.4 * scale, Color32::from_rgb(52, 82, 88)),
    );
    painter.circle_stroke(
        center,
        radius * 0.66,
        Stroke::new(
            0.8 * scale,
            Color32::from_rgba_unmultiplied(48, 76, 82, 120),
        ),
    );
    painter.circle_stroke(
        center,
        radius * 0.33,
        Stroke::new(
            0.8 * scale,
            Color32::from_rgba_unmultiplied(48, 76, 82, 120),
        ),
    );
    // Subtle crosshair guides
    painter.line_segment(
        [
            center - Vec2::new(radius, 0.0),
            center + Vec2::new(radius, 0.0),
        ],
        Stroke::new(0.6 * scale, Color32::from_rgba_unmultiplied(50, 78, 84, 90)),
    );
    painter.line_segment(
        [
            center - Vec2::new(0.0, radius),
            center + Vec2::new(0.0, radius),
        ],
        Stroke::new(0.6 * scale, Color32::from_rgba_unmultiplied(50, 78, 84, 90)),
    );

    // World coordinate to local radar offset: +X is right (east), -Z is North (up on radar)
    let to_radar = |wx: f32, wz: f32| -> (Vec2, f32) {
        let dx = wx - ctx.player_position[0];
        let dz = wz - ctx.player_position[2];
        let sx = (dx / radar_range) * radius;
        let sy = (dz / radar_range) * radius;
        let offset = Vec2::new(sx, sy);
        let dist = offset.length();
        (offset, dist)
    };

    // Nearby Resource Nodes (trees & rocks)
    for node in ctx.nodes {
        let (offset, dist) = to_radar(node.position[0], node.position[2]);
        if dist <= radius - 2.5 * scale {
            let color = match (node.kind, node.is_available()) {
                (ResourceKind::Tree, true) => Color32::from_rgb(78, 208, 112),
                (ResourceKind::Tree, false) => Color32::from_rgba_unmultiplied(78, 208, 112, 60),
                (ResourceKind::Rock, true) => Color32::from_rgb(204, 212, 218),
                (ResourceKind::Rock, false) => Color32::from_rgba_unmultiplied(204, 212, 218, 60),
            };
            painter.circle_filled(center + offset, 2.5 * scale, color);
        }
    }

    // Nearby Structures & TC privilege radius
    for structure in ctx.structures {
        let (offset, dist) = to_radar(structure.position[0], structure.position[2]);
        if dist <= radius - 2.5 * scale {
            let pos = center + offset;
            match structure.kind {
                StructureKind::ToolCupboard => {
                    painter.circle_filled(pos, 3.5 * scale, Color32::from_rgb(106, 227, 212));
                    let tc_r = (TC_PRIVILEGE_RADIUS / radar_range) * radius;
                    painter.circle_stroke(
                        pos,
                        tc_r,
                        Stroke::new(
                            0.9 * scale,
                            Color32::from_rgba_unmultiplied(106, 207, 183, 75),
                        ),
                    );
                }
                StructureKind::Campfire => {
                    painter.circle_filled(pos, 3.0 * scale, Color32::from_rgb(248, 152, 62));
                }
                _ => {
                    let [r, g, b] = structure.tier.color();
                    painter.rect_filled(
                        Rect::from_center_size(pos, Vec2::splat(4.2 * scale)),
                        1.0 * scale,
                        Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8),
                    );
                }
            }
        }
    }

    // Nearby NPCs & Vehicles
    for npc in ctx.npcs {
        if !npc.is_alive() {
            continue;
        }
        let (offset, dist) = to_radar(npc.position[0], npc.position[2]);
        if dist <= radius - 2.5 * scale {
            let color = match npc.kind {
                NpcKind::BradleyApc => Color32::from_rgb(248, 128, 48),
                NpcKind::PatrolHeli => Color32::from_rgb(252, 214, 64),
                _ => Color32::from_rgb(236, 78, 72),
            };
            painter.circle_filled(center + offset, 3.0 * scale, color);
        }
    }
    for vehicle in ctx.vehicles {
        if vehicle.health == 0 {
            continue;
        }
        let (offset, dist) = to_radar(vehicle.position[0], vehicle.position[2]);
        if dist <= radius - 2.5 * scale {
            painter.circle_filled(
                center + offset,
                3.0 * scale,
                Color32::from_rgb(76, 224, 246),
            );
        }
    }

    // Custom Waypoint indicator (clamped to perimeter with distinct pin)
    if let Some(marker_xz) = ctx.marker {
        let (m_off, m_dist) = to_radar(marker_xz[0], marker_xz[1]);
        let pin_color = Color32::from_rgb(248, 206, 78);
        let pin_pos = if m_dist <= radius - 4.0 * scale {
            center + m_off
        } else {
            let dir = m_off / m_dist;
            center + dir * (radius - 5.0 * scale)
        };
        if let Some(pin_tex) = ctx.marker_tex_id {
            let pin_size = Vec2::new(14.0, 17.0) * scale;
            let pin_rect =
                Rect::from_min_size(pin_pos - Vec2::new(pin_size.x * 0.5, pin_size.y), pin_size);
            painter.image(
                pin_tex,
                pin_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.circle_filled(pin_pos, 3.8 * scale, pin_color);
            painter.circle_stroke(pin_pos, 6.0 * scale, Stroke::new(1.4 * scale, pin_color));
        }
    }

    // Local player in radar center: vision cone + heading arrow
    let (sin, cos) = (-ctx.yaw).sin_cos();
    let cone_len = 16.0 * scale;
    let cone_half_w = 6.0 * scale;
    let tip = center + Vec2::new(-sin * cone_len, -cos * cone_len);
    let left = center
        + Vec2::new(
            -cos * cone_half_w + sin * 2.5 * scale,
            sin * cone_half_w + cos * 2.5 * scale,
        );
    let right = center
        + Vec2::new(
            cos * cone_half_w + sin * 2.5 * scale,
            -sin * cone_half_w + cos * 2.5 * scale,
        );
    painter.add(Shape::convex_polygon(
        vec![center, left, tip, right],
        Color32::from_rgba_unmultiplied(106, 207, 183, 115),
        Stroke::NONE,
    ));
    let arrow = [[0.0, -9.5], [-5.0, 5.5], [0.0, 2.5], [5.0, 5.5]]
        .map(|[x, y]| center + Vec2::new(cos * x - sin * y, sin * x + cos * y) * scale)
        .to_vec();
    painter.add(Shape::convex_polygon(
        arrow,
        ACCENT,
        Stroke::new(1.0 * scale, Color32::from_rgb(14, 23, 29)),
    ));
    painter.circle_filled(center, 2.6 * scale, Color32::WHITE);

    // Compass cardinal markers
    let comp_font = FontId::proportional(8.0 * scale);
    painter.text(
        center - Vec2::new(0.0, radius - 3.0 * scale),
        Align2::CENTER_TOP,
        "ПН",
        comp_font.clone(),
        INK,
    );
    painter.text(
        center + Vec2::new(radius - 8.0 * scale, 0.0),
        Align2::CENTER_CENTER,
        "СХ",
        comp_font.clone(),
        MUTED,
    );
    painter.text(
        center + Vec2::new(0.0, radius - 10.0 * scale),
        Align2::CENTER_BOTTOM,
        "ПД",
        comp_font.clone(),
        MUTED,
    );
    painter.text(
        center - Vec2::new(radius - 8.0 * scale, 0.0),
        Align2::CENTER_CENTER,
        "ЗХ",
        comp_font,
        MUTED,
    );
}

fn minimap(
    painter: &Painter,
    rect: Rect,
    scale: f32,
    _vitals: &Vitals,
    map_ctx: &MinimapContext<'_>,
) {
    panel(painter, rect, scale);
    let map = Rect::from_min_size(
        rect.min + Vec2::splat(8.0 * scale),
        Vec2::splat(156.0 * scale),
    );
    paint_radar_minimap(painter, map, scale, map_ctx);

    // Dynamic compass heading readout at top
    let deg = ((-map_ctx.yaw).to_degrees().rem_euclid(360.0)) as u32;
    let dir_str = match deg {
        338..=360 | 0..=22 => "ПН",
        23..=67 => "ПН-СХ",
        68..=112 => "СХ",
        113..=157 => "ПД-СХ",
        158..=202 => "ПД",
        203..=247 => "ПД-ЗХ",
        248..=292 => "ЗХ",
        _ => "ПН-ЗХ",
    };
    painter.text(
        rect.center_top() + Vec2::new(0.0, 5.0 * scale),
        Align2::CENTER_TOP,
        format!("{dir_str} {deg}°"),
        FontId::proportional(9.0 * scale),
        ACCENT,
    );

    let biome = Biome::at(map_ctx.player_position);
    let sector = shared::level::sector_label_at(map_ctx.player_position);
    painter.text(
        rect.center_bottom() - Vec2::new(0.0, 6.5 * scale),
        Align2::CENTER_BOTTOM,
        format!(
            "[{}] {} · X: {:.0} Z: {:.0} · [M]",
            sector,
            biome.short_label(),
            map_ctx.player_position[0],
            map_ctx.player_position[2],
        ),
        FontId::proportional(8.0 * scale),
        MUTED,
    );
}

fn joystick(painter: &Painter, rect: Rect) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.46;
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
    painter.circle_filled(
        center,
        radius * 0.27,
        Color32::from_rgba_unmultiplied(106, 207, 183, 150),
    );
}

fn action_button(painter: &Painter, rect: Rect, id: WidgetId, scale: f32) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.46;
    painter.circle_filled(
        center,
        radius,
        Color32::from_rgba_unmultiplied(14, 23, 29, 220),
    );
    painter.circle_stroke(center, radius, Stroke::new(scale, MUTED));
    let icon = center - Vec2::new(0.0, 6.0 * scale);
    let stroke = Stroke::new(1.8 * scale, INK);
    let label = match id {
        WidgetId::FireButtonLeft | WidgetId::FireButtonRight | WidgetId::AimButton => {
            painter.circle_stroke(icon, 8.0 * scale, stroke);
            for direction in [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y] {
                painter.line_segment(
                    [
                        icon + direction * 5.0 * scale,
                        icon + direction * 12.0 * scale,
                    ],
                    stroke,
                );
            }
            if id == WidgetId::AimButton {
                "ПРИЦІЛ"
            } else {
                "ВОГОНЬ"
            }
        }
        WidgetId::JumpButton => {
            painter.line_segment(
                [
                    icon + Vec2::new(0.0, 11.0) * scale,
                    icon - Vec2::new(0.0, 10.0) * scale,
                ],
                stroke,
            );
            painter.line_segment(
                [
                    icon + Vec2::new(-8.0, -2.0) * scale,
                    icon + Vec2::new(0.0, -10.0) * scale,
                ],
                stroke,
            );
            painter.line_segment(
                [
                    icon + Vec2::new(8.0, -2.0) * scale,
                    icon + Vec2::new(0.0, -10.0) * scale,
                ],
                stroke,
            );
            "СТРИБОК"
        }
        WidgetId::CrouchButton => {
            painter.circle_stroke(icon - Vec2::new(0.0, 8.0) * scale, 4.0 * scale, stroke);
            painter.line_segment([icon, icon + Vec2::new(8.0, 6.0) * scale], stroke);
            painter.line_segment(
                [
                    icon + Vec2::new(8.0, 6.0) * scale,
                    icon + Vec2::new(-7.0, 11.0) * scale,
                ],
                stroke,
            );
            "СІСТИ"
        }
        WidgetId::ReloadButton => {
            painter.circle_stroke(icon, 10.0 * scale, stroke);
            painter.line_segment(
                [
                    icon + Vec2::new(8.0, -6.0) * scale,
                    icon + Vec2::new(12.0, -11.0) * scale,
                ],
                stroke,
            );
            "ЗАРЯД"
        }
        _ => {
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
            "РЮКЗАК"
        }
    };
    painter.text(
        center + Vec2::new(0.0, 18.0 * scale),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(8.5 * scale),
        INK,
    );
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
