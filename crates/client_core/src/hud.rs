use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const HUD_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetId {
    HealthBars,
    Hotbar,
    Minimap,
    VirtualJoystickLeft,
    FireButtonLeft,
    FireButtonRight,
    AimButton,
    JumpButton,
    CrouchButton,
    ReloadButton,
    InventoryButton,
}

impl WidgetId {
    pub const ALL: [Self; 11] = [
        Self::HealthBars,
        Self::Hotbar,
        Self::Minimap,
        Self::VirtualJoystickLeft,
        Self::FireButtonLeft,
        Self::FireButtonRight,
        Self::AimButton,
        Self::JumpButton,
        Self::CrouchButton,
        Self::ReloadButton,
        Self::InventoryButton,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::HealthBars => "Здоров’я, вода, їжа",
            Self::Hotbar => "Панель предметів",
            Self::Minimap => "Мінікарта",
            Self::VirtualJoystickLeft => "Джойстик",
            Self::FireButtonLeft => "Вогонь зліва",
            Self::FireButtonRight => "Вогонь справа",
            Self::AimButton => "Прицілювання",
            Self::JumpButton => "Стрибок",
            Self::CrouchButton => "Присідання",
            Self::ReloadButton => "Перезарядка",
            Self::InventoryButton => "Інвентар",
        }
    }

    pub fn size(self) -> [f32; 2] {
        match self {
            Self::HealthBars => [232.0, 108.0],
            Self::Hotbar => [384.0, 62.0],
            Self::Minimap => [172.0, 190.0],
            Self::VirtualJoystickLeft => [126.0, 126.0],
            _ => [68.0, 68.0],
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetAnchor {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl WidgetAnchor {
    pub const ALL: [Self; 9] = [
        Self::TopLeft,
        Self::TopCenter,
        Self::TopRight,
        Self::CenterLeft,
        Self::Center,
        Self::CenterRight,
        Self::BottomLeft,
        Self::BottomCenter,
        Self::BottomRight,
    ];

    pub fn unit(self) -> [f32; 2] {
        match self {
            Self::TopLeft => [0.0, 0.0],
            Self::TopCenter => [0.5, 0.0],
            Self::TopRight => [1.0, 0.0],
            Self::CenterLeft => [0.0, 0.5],
            Self::Center => [0.5, 0.5],
            Self::CenterRight => [1.0, 0.5],
            Self::BottomLeft => [0.0, 1.0],
            Self::BottomCenter => [0.5, 1.0],
            Self::BottomRight => [1.0, 1.0],
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::TopLeft => "Вгорі зліва",
            Self::TopCenter => "Вгорі по центру",
            Self::TopRight => "Вгорі справа",
            Self::CenterLeft => "По центру зліва",
            Self::Center => "У центрі",
            Self::CenterRight => "По центру справа",
            Self::BottomLeft => "Внизу зліва",
            Self::BottomCenter => "Внизу по центру",
            Self::BottomRight => "Внизу справа",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HudWidgetConfig {
    pub anchor: WidgetAnchor,
    /// Logical points relative to the anchor, independent of physical DPI.
    pub offset: [f32; 2],
    pub scale: f32,
    pub opacity: f32,
    pub visible: bool,
}

impl Default for HudWidgetConfig {
    fn default() -> Self {
        Self {
            anchor: WidgetAnchor::TopLeft,
            offset: [20.0; 2],
            scale: 1.0,
            opacity: 0.9,
            visible: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub render_scale: f32,
}

impl HudWidgetConfig {
    pub fn sanitize(&mut self) {
        self.scale = finite_or(self.scale, 1.0).clamp(0.5, 2.5);
        self.opacity = finite_or(self.opacity, 0.9).clamp(0.1, 1.0);
        self.offset = self
            .offset
            .map(|value| finite_or(value, 0.0).clamp(-100_000.0, 100_000.0));
    }

    /// Clamp only the displayed position. Resizing never destroys the saved
    /// placement for a larger screen. Oversized widgets shrink proportionally.
    pub fn placement(&self, id: WidgetId, viewport: [f32; 2]) -> Placement {
        let viewport = viewport.map(|value| finite_or(value, 1.0).max(1.0));
        let margin = viewport.map(|value| 8.0_f32.min(value * 0.05));
        let base = id.size();
        let scale = finite_or(self.scale, 1.0).clamp(0.5, 2.5);
        let fit_x = (viewport[0] - 2.0 * margin[0]) / (base[0] * scale);
        let fit_y = (viewport[1] - 2.0 * margin[1]) / (base[1] * scale);
        let render_scale = scale * fit_x.min(fit_y).min(1.0);
        let size = base.map(|value| value * render_scale);
        let anchor = self.anchor.unit();
        let mut position = [0.0; 2];
        for axis in 0..2 {
            position[axis] = (anchor[axis] * (viewport[axis] - size[axis])
                + finite_or(self.offset[axis], 0.0))
            .clamp(
                margin[axis],
                (viewport[axis] - size[axis] - margin[axis]).max(margin[axis]),
            );
        }
        Placement {
            position,
            size,
            render_scale,
        }
    }

    pub fn set_position(&mut self, id: WidgetId, viewport: [f32; 2], position: [f32; 2]) {
        let placement = self.placement(id, viewport);
        let anchor = self.anchor.unit();
        for axis in 0..2 {
            self.offset[axis] =
                position[axis] - anchor[axis] * (viewport[axis] - placement.size[axis]);
        }
        self.sanitize();
        let clamped = self.placement(id, viewport);
        for axis in 0..2 {
            self.offset[axis] =
                clamped.position[axis] - anchor[axis] * (viewport[axis] - clamped.size[axis]);
        }
    }

    pub fn reanchor(&mut self, id: WidgetId, viewport: [f32; 2], anchor: WidgetAnchor) {
        let position = self.placement(id, viewport).position;
        self.anchor = anchor;
        self.set_position(id, viewport, position);
    }
}

fn finite_or(value: f32, default: f32) -> f32 {
    if value.is_finite() { value } else { default }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HudPreset {
    #[default]
    Desktop,
    TwoFingers,
    ThreeFingers,
    FourFingers,
}

impl HudPreset {
    pub const ALL: [Self; 4] = [
        Self::Desktop,
        Self::TwoFingers,
        Self::ThreeFingers,
        Self::FourFingers,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Desktop => "ПК",
            Self::TwoFingers => "2 пальці",
            Self::ThreeFingers => "3 пальці",
            Self::FourFingers => "4 пальці",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HudLayout {
    pub version: u32,
    pub preset: HudPreset,
    #[serde(default)]
    pub widgets: BTreeMap<WidgetId, HudWidgetConfig>,
}

impl Default for HudLayout {
    fn default() -> Self {
        Self::preset(HudPreset::Desktop)
    }
}

impl HudLayout {
    pub fn preset(preset: HudPreset) -> Self {
        use WidgetAnchor::*;
        use WidgetId::*;
        let mut widgets = BTreeMap::new();
        for id in WidgetId::ALL {
            widgets.insert(id, HudWidgetConfig::default());
        }
        let mut put = |id, anchor, offset, scale, visible| {
            widgets.insert(
                id,
                HudWidgetConfig {
                    anchor,
                    offset,
                    scale,
                    opacity: 0.9,
                    visible,
                },
            );
        };
        let mobile = preset != HudPreset::Desktop;
        put(
            HealthBars,
            if mobile { TopLeft } else { BottomLeft },
            if mobile { [20.0, 20.0] } else { [20.0, -20.0] },
            if mobile { 0.85 } else { 1.0 },
            true,
        );
        put(
            Hotbar,
            BottomCenter,
            [0.0, -20.0],
            if mobile { 0.8 } else { 1.0 },
            true,
        );
        put(
            Minimap,
            TopRight,
            [-20.0, 20.0],
            if mobile { 0.8 } else { 1.0 },
            true,
        );
        put(VirtualJoystickLeft, BottomLeft, [32.0, -48.0], 1.1, mobile);
        put(
            FireButtonLeft,
            TopLeft,
            [32.0, 144.0],
            1.1,
            matches!(preset, HudPreset::ThreeFingers | HudPreset::FourFingers),
        );
        put(FireButtonRight, BottomRight, [-24.0, -136.0], 1.1, mobile);
        put(AimButton, BottomRight, [-116.0, -112.0], 1.0, mobile);
        put(JumpButton, BottomRight, [-24.0, -40.0], 1.0, mobile);
        put(CrouchButton, BottomRight, [-116.0, -24.0], 1.0, mobile);
        put(ReloadButton, BottomRight, [-204.0, -56.0], 1.0, mobile);
        put(
            InventoryButton,
            if mobile { TopRight } else { BottomRight },
            if mobile {
                [-176.0, 20.0]
            } else {
                [-20.0, -20.0]
            },
            0.85,
            true,
        );
        if preset == HudPreset::FourFingers {
            put(AimButton, TopRight, [-260.0, 20.0], 1.1, true);
            put(FireButtonLeft, TopLeft, [32.0, 146.0], 1.2, true);
        }
        Self {
            version: HUD_VERSION,
            preset,
            widgets,
        }
    }

    pub fn normalize(&mut self) -> Result<bool, String> {
        if self.version != HUD_VERSION {
            return Err(format!("Непідтримувана версія HUD: {}", self.version));
        }
        let before = self.clone();
        let defaults = Self::preset(self.preset);
        for id in WidgetId::ALL {
            self.widgets
                .entry(id)
                .or_insert_with(|| defaults.widgets[&id].clone())
                .sanitize();
        }
        Ok(*self != before)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_widget_stays_accessible_at_resolutions_and_dpi_scales() {
        for preset in HudPreset::ALL {
            let layout = HudLayout::preset(preset);
            assert_eq!(layout.widgets.len(), WidgetId::ALL.len());
            for physical in [
                [320.0, 240.0],
                [1280.0, 720.0],
                [1920.0, 1080.0],
                [2560.0, 1080.0],
            ] {
                for dpi in [1.0, 1.5, 2.0, 3.0] {
                    let viewport = physical.map(|axis| axis / dpi);
                    for (&id, config) in &layout.widgets {
                        let mut oversized = config.clone();
                        oversized.scale = 2.5;
                        oversized.offset = [-100_000.0, 100_000.0];
                        let p = oversized.placement(id, viewport);
                        for (axis, limit) in viewport.iter().enumerate() {
                            assert!(p.position[axis] >= 0.0);
                            assert!(p.position[axis] + p.size[axis] <= limit + 0.001);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn anchor_change_preserves_position_and_resize_preserves_saved_offsets() {
        let mut config = HudLayout::default().widgets[&WidgetId::Minimap].clone();
        let viewport = [1280.0, 720.0];
        let before = config.placement(WidgetId::Minimap, viewport);
        config.reanchor(WidgetId::Minimap, viewport, WidgetAnchor::BottomLeft);
        assert_eq!(
            config.placement(WidgetId::Minimap, viewport).position,
            before.position
        );
        let saved = config.clone();
        config.placement(WidgetId::Minimap, [200.0, 100.0]);
        assert_eq!(config, saved);
    }

    #[test]
    fn drag_is_clamped_and_missing_widgets_are_restored() {
        let mut layout = HudLayout::default();
        layout.widgets.remove(&WidgetId::ReloadButton);
        let config = layout.widgets.get_mut(&WidgetId::Hotbar).unwrap();
        config.set_position(WidgetId::Hotbar, [800.0, 600.0], [-100.0, 5000.0]);
        assert!(config.placement(WidgetId::Hotbar, [800.0, 600.0]).position[1] < 600.0);
        assert!(layout.normalize().unwrap());
        assert!(layout.widgets.contains_key(&WidgetId::ReloadButton));
    }

    #[test]
    fn invalid_values_are_repaired_but_unknown_versions_are_rejected() {
        let mut layout = HudLayout::default();
        let config = layout.widgets.get_mut(&WidgetId::HealthBars).unwrap();
        config.scale = f32::NAN;
        config.opacity = -1.0;
        assert!(layout.normalize().unwrap());
        assert_eq!(layout.widgets[&WidgetId::HealthBars].scale, 1.0);
        assert_eq!(layout.widgets[&WidgetId::HealthBars].opacity, 0.1);
        layout.version = 99;
        assert!(layout.normalize().is_err());
    }
}
