//! Native Bevy UI for retained Unity panels and desktop interaction.
use crate::{
    hud::{HudPreset, WidgetAnchor, WidgetId},
    interface::{
        Interface, PlatformStorage, Telemetry, UiMode, craft_recipe, move_inventory_slot,
        select_hotbar_slot,
    },
    online::Online,
    source_minimap::{NativeMinimap, SOURCE_UI_PACKAGE, SourceMinimapHud, state_from_simulation},
    source_ui::{
        PanelFrame, PanelRenderReport, PreparedGraphic, SourcePanelRenderer, SourceUiBindings,
        SourceUiMaterial,
    },
};
use bevy::{
    asset::RenderAssetUsages,
    input::{ButtonState, keyboard::KeyboardInput, mouse::AccumulatedMouseScroll},
    input_focus::{
        FocusedInput, InputFocus, InputFocusSystems, InputFocusVisible,
        tab_navigation::{
            NavAction, TabGroup, TabIndex, TabNavigation, acquire_focus_tab_index,
            handle_tab_navigation,
        },
    },
    picking::{Pickable, hover::Hovered},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    text::{EditableText, EditableTextFilter, TextCursorStyle, TextEditChange},
    ui_widgets::{Activate, Button, ScrollArea, TextInput},
};
use midnight_ui_data::{ComponentProjection, HierarchyNode, layout::CanvasViewport};
use shared::{
    SimulationClock, SimulationMode, SimulationSurvival,
    protocol::{PlayerId, ServerTick},
    survival::{BACKPACK_SLOTS, HOTBAR_SLOTS, RecipeId, SlotIndex},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum NativeUiSet {
    Input,
    Presentation,
}
#[derive(Resource, Default)]
pub struct NativeUiInput {
    pub blocks_gameplay: bool,
}
#[derive(Resource, Default)]
struct UiActions(Vec<UiAction>);
#[derive(Clone, Debug)]
enum UiAction {
    Play,
    Menu,
    Settings,
    Inventory,
    Map,
    Editor,
    SaveEditor,
    CancelEditor,
    Preset(HudPreset),
    SelectWidget(WidgetId),
    Scale(f32),
    Opacity(f32),
    Anchor,
    Visibility,
    Clouds,
    Fly,
    Fog,
    Hud,
    Quit,
    Hotbar(u8),
    Slot(SlotIndex),
    Craft(RecipeId),
    MapZoom(f32),
    ClearMarker,
    CatalogPage(i32),
    DrawingPage(i32),
    Connect,
    Disconnect,
}
#[derive(Component)]
struct Action(UiAction);
#[derive(Component)]
struct SourceVisual;
#[derive(Component)]
struct Overlay;
#[derive(Component, Clone, Copy)]
enum OnlineField {
    Address,
    Invitation,
}
struct GraphicEntity {
    entity: Entity,
    coordinates: Option<Handle<Image>>,
    material: Option<Handle<SourceUiMaterial>>,
    clip: Option<Entity>,
}
#[derive(Default)]
pub struct SourcePanelView {
    nodes: BTreeMap<String, GraphicEntity>,
    active: BTreeSet<String>,
    key: String,
}
#[derive(Resource, Default)]
struct NativeUiRuntime {
    renderers: BTreeMap<String, SourcePanelRenderer>,
    caches: BTreeMap<String, SourcePanelView>,
    errors: BTreeMap<String, String>,
    reports: BTreeMap<String, PanelRenderReport>,
    overlay_key: String,
    map_zoom: f32,
    map_offset: [f32; 2],
    dragging: Option<(WidgetId, Vec2)>,
    menu_settings: bool,
    widget_rects: BTreeMap<WidgetId, Rect>,
    catalog_page: usize,
    drawing_page: usize,
    minimap_rect: Option<Rect>,
    map_dragging: Option<Vec2>,
    map_hit: Option<MapHit>,
}
struct MapHit {
    clip: Rect,
    local_from_screen: Mat4,
    rect: midnight_ui_data::layout::UiRect,
}

pub struct NativeUiPlugin;
impl Plugin for NativeUiPlugin {
    fn build(&self, app: &mut App) {
        configure_tab_navigation(app);
        app.init_resource::<NativeUiRuntime>()
            .init_resource::<NativeUiInput>()
            .init_resource::<UiActions>()
            .init_resource::<SourceMinimapHud>()
            .add_plugins(UiMaterialPlugin::<SourceUiMaterial>::default())
            .add_systems(Startup, load_panels)
            .add_systems(
                PreUpdate,
                (apply_actions, update_input_boundary)
                    .chain()
                    .after(bevy::picking::PickingSystems::Last)
                    .in_set(NativeUiSet::Input),
            )
            .add_systems(
                Update,
                (editor_drag, render_panels, render_overlay)
                    .chain()
                    .after(shared::source_inventory::SourceInventorySet::Apply)
                    .in_set(NativeUiSet::Presentation),
            );
    }
}

pub(crate) fn uses_ui_tab_navigation(interface: &Interface) -> bool {
    !matches!(interface.mode, UiMode::Playing | UiMode::Benchmark) || interface.show_gm_panel
}

fn configure_tab_navigation(app: &mut App) {
    app.add_observer(acquire_focus_tab_index)
        .add_systems(Startup, setup_tab_navigation)
        .configure_sets(
            PreUpdate,
            NativeUiSet::Input.after(InputFocusSystems::Dispatch),
        );
}

fn setup_tab_navigation(
    mut commands: Commands,
    windows: Query<Entity, With<bevy::window::PrimaryWindow>>,
) {
    for window in &windows {
        commands
            .entity(window)
            .observe(handle_native_tab_navigation);
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn handle_native_tab_navigation(
    event: On<FocusedInput<KeyboardInput>>,
    nav: TabNavigation,
    mut focus: ResMut<InputFocus>,
    visible: ResMut<InputFocusVisible>,
    keys: Res<ButtonInput<KeyCode>>,
    interface: Res<Interface>,
    groups: Query<Entity, (With<Overlay>, With<TabGroup>, With<Children>)>,
    windows: Query<(), With<Window>>,
) {
    if !uses_ui_tab_navigation(&interface)
        || groups.is_empty()
        || event.input.key_code != KeyCode::Tab
        || event.input.state != ButtonState::Pressed
        || event.input.repeat
    {
        return;
    }
    if !groups
        .iter()
        .any(|group| nav.initialize(group, NavAction::First).is_ok())
    {
        return;
    }
    // Initial focus belongs to the window; enter the overlay's first control.
    if focus.get().is_some_and(|entity| windows.contains(entity)) {
        focus.clear();
    }
    handle_tab_navigation(event, nav, focus, visible, keys);
}

fn load_panels(
    mut runtime: ResMut<NativeUiRuntime>,
    mut minimap: ResMut<SourceMinimapHud>,
    mut images: ResMut<Assets<Image>>,
    mut fonts: ResMut<Assets<Font>>,
) {
    let root = shared::level::asset_root().join(SOURCE_UI_PACKAGE);
    for name in [
        "BattlePanel",
        "BattleMapPanel",
        "BagAndBuildPanel",
        "BattleLoadingPanel",
        "MainPanel",
        "SettingPanel",
        "DeathPanel",
        "BluePrintBuildPanel",
    ] {
        match SourcePanelRenderer::load(&root, name, &mut images, &mut fonts) {
            Ok(renderer) => {
                runtime.renderers.insert(name.into(), renderer);
            }
            Err(e) => {
                runtime.errors.insert(name.into(), e.to_string());
            }
        }
    }
    match NativeMinimap::load(&root, &mut images, &mut fonts) {
        Ok(native) => minimap.minimap = Some(native),
        Err(e) => minimap.error = Some(e.to_string()),
    }
    runtime.map_zoom = 1.0;
    for (name, error) in &runtime.errors {
        warn!("Native UI panel {name} pending: {error}");
    }
}
fn update_input_boundary(
    interface: Res<Interface>,
    mut input: ResMut<NativeUiInput>,
    hovered: Query<&Hovered, With<Button>>,
    focus: Option<Res<InputFocus>>,
    fields: Query<(), With<TextInput>>,
) {
    input.blocks_gameplay = !matches!(interface.mode, UiMode::Playing | UiMode::Benchmark)
        || interface.show_gm_panel
        || hovered.iter().any(Hovered::get);
    input.blocks_gameplay |= focus
        .as_ref()
        .and_then(|f| f.get())
        .is_some_and(|e| fields.contains(e));
}
#[allow(clippy::too_many_arguments)]
fn apply_actions(
    mut actions: ResMut<UiActions>,
    mut interface: ResMut<Interface>,
    storage: Res<PlatformStorage>,
    mode: Res<SimulationMode>,
    mut survival: ResMut<SimulationSurvival>,
    mut online: ResMut<Online>,
    mut runtime: ResMut<NativeUiRuntime>,
    mut exit: MessageWriter<AppExit>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let viewport = windows
        .single()
        .map(|w| [w.width(), w.height()])
        .unwrap_or([1920.0, 1080.0]);
    for action in std::mem::take(&mut actions.0) {
        match action {
            UiAction::Play => {
                interface.mode = UiMode::Playing;
                interface.show_gm_panel = false;
            }
            UiAction::Menu => {
                interface.mode = UiMode::Menu;
                runtime.menu_settings = false;
            }
            UiAction::Settings => {
                interface.mode = UiMode::Menu;
                runtime.menu_settings = true;
            }
            UiAction::Inventory => {
                if interface.mode == UiMode::Menu {
                    interface.mode = UiMode::Inventory
                } else {
                    interface.toggle_inventory()
                }
            }
            UiAction::Map => {
                if interface.mode == UiMode::Menu {
                    interface.mode = UiMode::Map
                } else {
                    interface.toggle_map()
                }
            }
            UiAction::Editor => interface.begin_editing(),
            UiAction::SaveEditor => {
                if interface.editor.save(storage.0.as_ref()) {
                    interface.mode = UiMode::Menu;
                }
            }
            UiAction::CancelEditor => {
                interface.editor.cancel();
                interface.mode = UiMode::Menu;
            }
            UiAction::Preset(p) => interface.editor.apply_preset(p),
            UiAction::SelectWidget(w) => interface.editor.selected = w,
            UiAction::Scale(delta) => {
                let selected = interface.editor.selected;
                if let Some(config) = interface
                    .editor
                    .draft_mut()
                    .and_then(|l| l.widgets.get_mut(&selected))
                {
                    config.scale += delta;
                    config.sanitize();
                }
            }
            UiAction::Opacity(delta) => {
                let selected = interface.editor.selected;
                if let Some(config) = interface
                    .editor
                    .draft_mut()
                    .and_then(|l| l.widgets.get_mut(&selected))
                {
                    config.opacity += delta;
                    config.sanitize();
                }
            }
            UiAction::Anchor => {
                let selected = interface.editor.selected;
                if let Some(config) = interface
                    .editor
                    .draft_mut()
                    .and_then(|l| l.widgets.get_mut(&selected))
                {
                    let i = WidgetAnchor::ALL
                        .iter()
                        .position(|a| *a == config.anchor)
                        .unwrap_or(0);
                    config.reanchor(
                        selected,
                        viewport,
                        WidgetAnchor::ALL[(i + 1) % WidgetAnchor::ALL.len()],
                    );
                }
            }
            UiAction::Visibility => {
                let selected = interface.editor.selected;
                if let Some(config) = interface
                    .editor
                    .draft_mut()
                    .and_then(|l| l.widgets.get_mut(&selected))
                {
                    config.visible = !config.visible;
                }
            }
            UiAction::Clouds => {
                interface.volumetric_fog_steps = match interface.volumetric_fog_steps {
                    0..=12 => 24,
                    13..=24 => 48,
                    _ => 12,
                }
            }
            UiAction::Fly => interface.gm_fly_mode = !interface.gm_fly_mode,
            UiAction::Fog => interface.gm_fog_enabled = !interface.gm_fog_enabled,
            UiAction::Hud => interface.gm_hud_enabled = !interface.gm_hud_enabled,
            UiAction::Quit => {
                exit.write(AppExit::Success);
            }
            UiAction::Hotbar(slot) => select_hotbar_slot(slot, &mode, &mut survival, &online),
            UiAction::Slot(slot) => {
                if let Some(from) = interface.selected_slot.take() {
                    if from != slot {
                        move_inventory_slot(from, slot, &mode, &mut survival, &online)
                    }
                } else {
                    interface.selected_slot = Some(slot)
                }
            }
            UiAction::Craft(recipe) => craft_recipe(recipe, &mode, &mut survival, &online),
            UiAction::MapZoom(delta) => {
                runtime.map_zoom = (runtime.map_zoom + delta).clamp(0.5, 4.0)
            }
            UiAction::ClearMarker => interface.map_marker = None,
            UiAction::CatalogPage(delta) => {
                runtime.catalog_page = if delta >= 0 {
                    runtime.catalog_page.saturating_add(delta as usize)
                } else {
                    runtime
                        .catalog_page
                        .saturating_sub(delta.unsigned_abs() as usize)
                };
            }
            UiAction::DrawingPage(delta) => {
                runtime.drawing_page = if delta >= 0 {
                    runtime.drawing_page.saturating_add(delta as usize)
                } else {
                    runtime
                        .drawing_page
                        .saturating_sub(delta.unsigned_abs() as usize)
                };
            }
            UiAction::Connect => online.connect(),
            UiAction::Disconnect => online.disconnect(),
        }
    }
}

fn visit_nodes(node: &HierarchyNode, path: &str, f: &mut impl FnMut(&HierarchyNode, &str)) {
    let path = if path.is_empty() {
        node.name.clone()
    } else {
        format!("{path}/{}", node.name)
    };
    f(node, &path);
    for child in &node.children {
        visit_nodes(child, &path, f)
    }
}
fn main_hud_bindings(
    renderer: &SourcePanelRenderer,
    interface: &Interface,
    survival: &SimulationSurvival,
) -> SourceUiBindings {
    let mut bindings = SourceUiBindings::default();
    let player = survival.0.player(PlayerId(0));
    visit_nodes(&renderer.panel.root, "", &mut |node, path| {
        let visible = match path {
            "BattlePanel/m_root/m_zhu/chat"
            | "BattlePanel/m_root/m_zhu/m_headInfo"
            | "BattlePanel/m_root/m_zhu/m_boss_headInfo"
            | "BattlePanel/m_root/m_zhu/m_operatesPanel"
            | "BattlePanel/m_root/m_zhu/m_shuixia"
            | "BattlePanel/m_root/m_leftTop"
            | "BattlePanel/m_root/m_tips"
            | "BattlePanel/m_root/BlackBase"
            | "BattlePanel/m_root/m_qiangzhan"
            | "BattlePanel/m_root/m_quick_mark_style"
            | "BattlePanel/m_root/m_rightTop/m_map" => Some(false),
            "BattlePanel/m_root/m_zhu/xia/build" => Some(false),
            "BattlePanel/m_root/m_zhu/xia/GameObject"
            | "BattlePanel/m_root/m_waterEffect"
            | "BattlePanel/m_root/m_mobileWatch/m_InLand/m_swimPanel"
            | "BattlePanel/m_root/m_mobileWatch/m_InLand/m_dolphinBtns"
            | "BattlePanel/m_root/m_mobileWatch/m_InLand/Shoot/m_shootPanel/m_OperationBtns/GameObject/btn_build"
            | "BattlePanel/m_root/m_mobileWatch/m_InLand/Shoot/m_shootPanel/m_OperationBtns/GameObject/m_skill"
            | "BattlePanel/m_root/m_mobileWatch/m_InLand/Shoot/m_shootPanel/m_OperationBtns/m_bulletsSelect"
            | "BattlePanel/m_root/m_mobileWatch/m_InLand/Shoot/m_shootPanel/btn_openGlider"
            | "BattlePanel/m_root/m_mobileWatch/m_InLand/Shoot/btn_HelpOther"
            | "BattlePanel/m_root/m_mobileWatch/m_InLand/GameObject" => Some(false),
            _ => None,
        };
        if let Some(value) = visible {
            bindings.visibility.insert(node.source.clone(), value);
        }
        if path.starts_with("BattlePanel/m_root/m_mobileWatch/")
            && !path.starts_with("BattlePanel/m_root/m_mobileWatch/m_InLand")
            && !path.starts_with("BattlePanel/m_root/m_mobileWatch/yaogan")
            && !path.starts_with("BattlePanel/m_root/m_mobileWatch/m_zhunxing")
        {
            bindings.visibility.insert(node.source.clone(), false);
        }
        if let Some(player) = player {
            for component in &node.components {
                if let ComponentProjection::Text(_) = component.projection {
                    let value = match path {
                        "BattlePanel/m_root/m_rightTop/xue_je_fs/m_xue/txt_mybloodNum" => {
                            Some(format!("{:.0}", player.vitals.health))
                        }
                        "BattlePanel/m_root/m_rightTop/xue_je_fs/m_bodyWater/txt_bodyWater" => {
                            Some(format!("{:.0}", player.vitals.hydration))
                        }
                        "BattlePanel/m_root/m_rightTop/xue_je_fs/m_jie/txt_hunger" => {
                            Some(format!("{:.0}", player.vitals.calories))
                        }
                        _ => None,
                    };
                    if let Some(value) = value {
                        bindings.text.insert(component.source.clone(), value);
                    }
                }
            }
            let fill = match path {
                "BattlePanel/m_root/m_rightTop/xue_je_fs/m_xue/m_mybloodSlilder/Fill Area/m_mybloodFillImage" => {
                    Some(player.vitals.health / shared::survival::MAX_HEALTH)
                }
                "BattlePanel/m_root/m_rightTop/xue_je_fs/m_jie/m_sld_hunger/Fill Area/m_myhungerFillImage" => {
                    Some(player.vitals.calories / shared::survival::MAX_CALORIES)
                }
                "BattlePanel/m_root/m_rightTop/xue_je_fs/m_bodyWater/m_sld_water/Fill Area/m_myWaterFillImage" => {
                    Some(player.vitals.hydration / shared::survival::MAX_HYDRATION)
                }
                _ => None,
            };
            if let Some(fill) = fill {
                bind_slider_fill(node, fill, &mut bindings);
            }
            if let Some(rest) =
                path.strip_prefix("BattlePanel/m_root/m_zhu/xia/wupin/m_bagItems/m_Item")
            {
                let mut parts = rest.splitn(2, '/');
                let index = parts
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .and_then(|n| n.checked_sub(1));
                if let Some(index) = index {
                    let content = parts.next().unwrap_or("");
                    let stack = (index < HOTBAR_SLOTS)
                        .then(|| player.inventory.slot(SlotIndex::Hotbar(index as u8)))
                        .flatten();
                    if content == "m_s" {
                        bindings
                            .visibility
                            .insert(node.source.clone(), stack.is_some());
                    }
                    if content.ends_with("m_durationSlider") {
                        bindings.visibility.insert(node.source.clone(), false);
                    }
                    if content.ends_with("txt_num") {
                        for c in &node.components {
                            if matches!(c.projection, ComponentProjection::Text(_)) {
                                bindings.text.insert(
                                    c.source.clone(),
                                    stack.map(|s| s.count.to_string()).unwrap_or_default(),
                                );
                            }
                        }
                    }
                    if content.ends_with("m_select") {
                        bindings.visibility.insert(
                            node.source.clone(),
                            index == player.inventory.active_slot as usize,
                        );
                    }
                    if content.ends_with("m_icon") {
                        // Source item-to-sprite bindings are still unobserved.
                        bindings.visibility.insert(node.source.clone(), false);
                    }
                }
            }
        }
    });
    for widget in WidgetId::ALL {
        if let Some(config) = interface.editor.layout().widgets.get(&widget)
            && !config.visible
            && let Some(prefix) = widget_source_path(widget)
        {
            visit_nodes(&renderer.panel.root, "", &mut |node, path| {
                if path == prefix {
                    bindings.visibility.insert(node.source.clone(), false);
                }
            });
        }
    }
    bindings
}

fn bind_slider_fill(node: &HierarchyNode, fill: f32, bindings: &mut SourceUiBindings) {
    let fill = fill.clamp(0.0, 1.0);
    for c in &node.components {
        if let ComponentProjection::Image(image) = &c.projection {
            if image.image_type == 3 {
                bindings.fill_amounts.insert(c.source.clone(), fill);
                bindings
                    .rect_anchors
                    .insert(node.transform.source.clone(), [[0.0; 2], [1.0; 2]]);
            } else {
                bindings
                    .rect_anchors
                    .insert(node.transform.source.clone(), [[0.0; 2], [fill, 1.0]]);
            }
        }
    }
}

fn widget_source_path(widget: WidgetId) -> Option<&'static str> {
    Some(match widget {
        WidgetId::HealthBars => "BattlePanel/m_root/m_rightTop/xue_je_fs",
        WidgetId::Hotbar => "BattlePanel/m_root/m_zhu/xia/wupin",
        WidgetId::VirtualJoystickLeft => "BattlePanel/m_root/m_mobileWatch/yaogan/m_yaogan",
        WidgetId::FireButtonLeft => {
            "BattlePanel/m_root/m_mobileWatch/m_InLand/Shoot/m_shootPanel/m_OperationBtns/btn_leftFire"
        }
        WidgetId::FireButtonRight => {
            "BattlePanel/m_root/m_mobileWatch/m_InLand/Shoot/m_shootPanel/m_OperationBtns/GameObject/btn_fire"
        }
        WidgetId::AimButton => {
            "BattlePanel/m_root/m_mobileWatch/m_InLand/Shoot/m_shootPanel/m_OperationBtns/btn_kaijing"
        }
        WidgetId::JumpButton => {
            "BattlePanel/m_root/m_mobileWatch/m_InLand/Shoot/m_shootPanel/m_ChangeType2/btn_up2"
        }
        WidgetId::CrouchButton => {
            "BattlePanel/m_root/m_mobileWatch/m_InLand/Shoot/m_shootPanel/m_ChangeType2/btn_crouch2"
        }
        WidgetId::InventoryButton => "BattlePanel/m_root/m_zhu/btn_zhizao",
        WidgetId::Minimap | WidgetId::ReloadButton => return None,
    })
}

fn source_bounds(layouts: &[midnight_ui_data::layout::ResolvedNode], prefix: &str) -> Option<Rect> {
    let mut bounds: Option<Rect> = None;
    for layout in layouts.iter().filter(|layout| {
        layout.active
            && (layout.path == prefix
                || layout
                    .path
                    .strip_prefix(prefix)
                    .is_some_and(|s| s.starts_with('/')))
    }) {
        if let Some(corners) = layout.screen_corners {
            for corner in corners {
                let point = Vec2::from_array(corner);
                if let Some(rect) = &mut bounds {
                    rect.min = rect.min.min(point);
                    rect.max = rect.max.max(point);
                } else {
                    bounds = Some(Rect::from_corners(point, point));
                }
            }
        }
    }
    bounds.filter(|b| b.size().min_element() > 0.0)
}

fn apply_hud_layout(
    renderer: &SourcePanelRenderer,
    interface: &Interface,
    viewport: CanvasViewport,
    dpi: f32,
    bindings: &mut SourceUiBindings,
) -> Result<(), midnight_ui_data::UiError> {
    let mut panel = renderer.panel.clone();
    crate::source_ui::apply_bindings(&mut panel.root, bindings);
    let layouts = midnight_ui_data::layout::resolve_panel(&panel, viewport)?;
    for widget in WidgetId::ALL {
        let Some(prefix) = widget_source_path(widget) else {
            continue;
        };
        let Some(config) = interface.editor.layout().widgets.get(&widget) else {
            continue;
        };
        let Some(bounds) = source_bounds(&layouts, prefix) else {
            continue;
        };
        let Some(layout) = layouts.iter().find(|l| l.path == prefix) else {
            continue;
        };
        let Some(parent) = layout.parent.and_then(|i| layouts.get(i)) else {
            continue;
        };
        let Some(node) = crate::source_ui::find_node(&panel.root, &layout.source) else {
            continue;
        };
        let Some(rect) = node.transform.rect.as_ref() else {
            continue;
        };
        let placement = config.placement(widget, viewport.pixels.map(|p| p as f32 / dpi));
        let scale = (Vec2::from_array(placement.size) * dpi / bounds.size()).min_element();
        let pivot = Vec2::from_array(layout.to_screen([0.0; 2]));
        let target = Vec2::from_array(placement.position) * dpi + (pivot - bounds.min) * scale;
        let delta = Mat4::from_cols_array(&parent.screen_from_local)
            .inverse()
            .transform_vector3((target - pivot).extend(0.0))
            .truncate();
        bindings.anchored_positions.insert(
            node.transform.source.clone(),
            (Vec2::from_array(rect.anchored_position) + delta).to_array(),
        );
        bindings.local_scales.insert(
            node.transform.source.clone(),
            node.transform.local_scale.map(|v| v * scale),
        );
        if config.opacity < 1.0 {
            visit_nodes(node, "", &mut |n, _| {
                for c in &n.components {
                    let graphic = match &c.projection {
                        ComponentProjection::Image(i) => Some(&i.graphic),
                        ComponentProjection::RawImage(i) => Some(&i.graphic),
                        ComponentProjection::Text(t) => Some(&t.graphic),
                        _ => None,
                    };
                    if let Some(graphic) = graphic {
                        let mut color = graphic.color;
                        color[3] *= config.opacity;
                        bindings.colors.insert(c.source.clone(), color);
                    }
                }
            });
        }
    }
    Ok(())
}

fn source_action(path: &str) -> Option<UiAction> {
    Some(match path {
        "BattlePanel/m_root/m_rightTop/btn_setting" => UiAction::Menu,
        "BattlePanel/m_root/m_zhu/btn_zhizao" => UiAction::Inventory,
        "BattleMinimap" => UiAction::Map,
        "BattleMapPanel/GameObject/you/btn_close"
        | "BagAndBuildPanel/shang (3)/btn_back"
        | "MainPanel/m_youxiajiao/btn_Back_to_game" => UiAction::Play,
        "BattleMapPanel/GameObject/zuo/m_map_slide_root/btn_up" => UiAction::MapZoom(0.25),
        "BattleMapPanel/GameObject/zuo/m_map_slide_root/btn_down" => UiAction::MapZoom(-0.25),
        "BattleMapPanel/GameObject/zhong/btn_delete_mark"
        | "BattleMapPanel/GameObject/zhong/btn_clear_marks" => UiAction::ClearMarker,
        "MainPanel/m_ding/you/GameObject/btn_setting" => UiAction::Settings,
        "SettingPanel/shang/btn_back" => UiAction::Menu,
        "SettingPanel/GameObject/left/GameObject/btn_operation" => UiAction::Editor,
        _ => {
            let suffix =
                path.strip_prefix("BattlePanel/m_root/m_zhu/xia/wupin/m_bagItems/m_Item")?;
            let slot = suffix.parse::<u8>().ok()?.checked_sub(1)?;
            if usize::from(slot) >= HOTBAR_SLOTS {
                return None;
            }
            UiAction::Hotbar(slot)
        }
    })
}

fn update_source_controls(
    commands: &mut Commands,
    cache: &mut SourcePanelView,
    layouts: &[midnight_ui_data::layout::ResolvedNode],
    dpi: f32,
) {
    for layout in layouts {
        let Some(action) = source_action(&layout.path) else {
            continue;
        };
        let Some(corners) = layout.screen_corners.filter(|_| layout.active) else {
            continue;
        };
        let min = corners
            .iter()
            .map(|p| Vec2::from_array(*p))
            .reduce(Vec2::min)
            .unwrap_or(Vec2::ZERO);
        let max = corners
            .iter()
            .map(|p| Vec2::from_array(*p))
            .reduce(Vec2::max)
            .unwrap_or(Vec2::ZERO);
        if (max - min).min_element() <= 0.0 {
            continue;
        }
        let key = format!("control:{}", layout.path);
        let node = position_node(min / dpi, (max - min) / dpi);
        if let Some(entry) = cache.nodes.get(&key) {
            commands
                .entity(entry.entity)
                .insert((node, Visibility::Inherited));
        } else {
            let entity = commands
                .spawn((
                    Button,
                    Hovered::default(),
                    Action(action),
                    node,
                    Name::new(format!("Source action: {}", layout.path)),
                    GlobalZIndex(10_000),
                ))
                .observe(
                    |event: On<Activate>, query: Query<&Action>, mut actions: ResMut<UiActions>| {
                        if let Ok(action) = query.get(event.entity) {
                            actions.0.push(action.0.clone());
                        }
                    },
                )
                .id();
            cache.nodes.insert(
                key.clone(),
                GraphicEntity {
                    entity,
                    coordinates: None,
                    material: None,
                    clip: None,
                },
            );
        }
        cache.active.insert(key);
    }
}

fn screen_bindings(renderer: &SourcePanelRenderer, interface: &Interface) -> SourceUiBindings {
    let mut bindings = SourceUiBindings::default();
    visit_nodes(&renderer.panel.root, "", &mut |node, path| {
        let hidden = match renderer.panel.name.as_str() {
            "BagAndBuildPanel" => {
                matches!(
                    path,
                    "BagAndBuildPanel/m_produce_page"
                        | "BagAndBuildPanel/m_buid_bg"
                        | "BagAndBuildPanel/m_bag_page/m_weapon"
                        | "BagAndBuildPanel/m_bag_page/m_equip_desc"
                        | "BagAndBuildPanel/m_bag_page/m_desc"
                        | "BagAndBuildPanel/m_bag_page/m_quick_use_mask"
                        | "BagAndBuildPanel/m_bag_page/m_drag_Canvas"
                        | "BagAndBuildPanel/m_bag_page/m_equipment"
                ) || path.ends_with("/m_cell")
                    || path.ends_with("/shortcut_bar_item")
            }
            "MainPanel" => {
                matches!(
                    path,
                    "MainPanel/m_hero_space"
                        | "MainPanel/m_posture_tip"
                        | "MainPanel/m_zuobian"
                        | "MainPanel/m_zuoxia"
                        | "MainPanel/m_youshangjiao"
                        | "MainPanel/down"
                        | "MainPanel/btn_openserver"
                        | "MainPanel/btn_Gm"
                        | "MainPanel/m_ding/zuo"
                        | "MainPanel/m_youxiajiao/btn_server"
                        | "MainPanel/m_youxiajiao/btn_start_game"
                ) || (path.starts_with("MainPanel/m_ding/you/GameObject/")
                    && !path.starts_with("MainPanel/m_ding/you/GameObject/btn_setting"))
            }
            "SettingPanel" => matches!(
                path,
                "SettingPanel/GameObject/GameObject" | "SettingPanel/shang/buff"
            ),
            _ => false,
        };
        if hidden {
            bindings.visibility.insert(node.source.clone(), false);
        }
        if renderer.panel.name == "BattleLoadingPanel" {
            if node.name == "txt_loding" {
                for c in &node.components {
                    if matches!(c.projection, ComponentProjection::Text(_)) {
                        bindings.text.insert(
                            c.source.clone(),
                            format!("{:.0}%", interface.loading_progress * 100.0),
                        );
                    }
                }
            }
            if path.ends_with("/Fill Area/Fill") {
                bind_slider_fill(node, interface.loading_progress, &mut bindings);
            }
        }
    });
    bindings
}
fn map_bindings(
    renderer: &SourcePanelRenderer,
    runtime: &NativeUiRuntime,
    telemetry: &Telemetry,
    interface: &Interface,
) -> SourceUiBindings {
    let mut bindings = SourceUiBindings::default();
    visit_nodes(&renderer.panel.root, "", &mut |node, path| {
        if path == "BattleMapPanel/GameObject/m_map_scroll/m_Map" {
            bindings
                .local_scales
                .insert(node.transform.source.clone(), [0.125 * runtime.map_zoom; 3]);
            bindings
                .anchored_positions
                .insert(node.transform.source.clone(), runtime.map_offset);
            for component in &node.components {
                if matches!(component.projection, ComponentProjection::RawImage(_)) {
                    bindings.raw_images.insert(
                        component.source.clone(),
                        crate::source_minimap::map_texture(),
                    );
                }
            }
        }
        if path.starts_with("BattleMapPanel/GameObject/m_map_scroll/m_Map/") {
            let group = "BattleMapPanel/GameObject/m_map_scroll/m_Map/m_teamMapInfos";
            let player = format!("{group}/user1");
            let visible = path == group
                || path == player
                || path == format!("{player}/m_pos")
                || (interface.map_marker.is_some()
                    && (path == format!("{player}/m_single_mark")
                        || path.starts_with(&format!("{player}/m_single_mark/"))));
            bindings.visibility.insert(node.source.clone(), visible);
            let factor = 8640.0 / (shared::level::MAP_OVERVIEW_HALF_EXTENT * 2.0);
            if path == player {
                bindings.anchored_positions.insert(
                    node.transform.source.clone(),
                    [
                        telemetry.player_position[0] * factor,
                        -telemetry.player_position[2] * factor,
                    ],
                );
                bindings
                    .local_scales
                    .insert(node.transform.source.clone(), [1.0; 3]);
            }
            if path == format!("{player}/m_pos") {
                bindings.local_rotations.insert(
                    node.transform.source.clone(),
                    Quat::from_rotation_z(telemetry.yaw).to_array(),
                );
                bindings
                    .local_scales
                    .insert(node.transform.source.clone(), [2.4 / runtime.map_zoom; 3]);
            }
            if path == format!("{player}/m_single_mark")
                && let Some(marker) = interface.map_marker
            {
                bindings.anchored_positions.insert(
                    node.transform.source.clone(),
                    [
                        (marker[0] - telemetry.player_position[0]) * factor,
                        -(marker[1] - telemetry.player_position[2]) * factor,
                    ],
                );
                bindings
                    .local_scales
                    .insert(node.transform.source.clone(), [2.0 / runtime.map_zoom; 3]);
            }
        }
        if matches!(
            path,
            "BattleMapPanel/GameObject/you/m_shouhuoji"
                | "BattleMapPanel/GameObject/you/m_fanmaiji"
                | "BattleMapPanel/GameObject/you/m_fanzhongli"
                | "BattleMapPanel/GameObject/zuo/m_refresh"
        ) {
            bindings.visibility.insert(node.source.clone(), false);
        }
    });
    bindings
}

#[allow(clippy::too_many_arguments)]
fn render_panels(
    mut commands: Commands,
    mut runtime: ResMut<NativeUiRuntime>,
    mut minimap: ResMut<SourceMinimapHud>,
    interface: Res<Interface>,
    telemetry: Res<Telemetry>,
    survival: Res<SimulationSurvival>,
    clock: Res<SimulationClock>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<SourceUiMaterial>>,
    mut nodes: Query<&mut Node>,
    mut text: Query<&mut Text>,
    mut fonts: Query<&mut TextFont>,
    mut visibility: Query<&mut Visibility>,
) {
    let Ok(window) = windows.single() else { return };
    let dpi = window.scale_factor();
    let viewport = CanvasViewport {
        pixels: [window.physical_width(), window.physical_height()],
        scale_factor: (window.width() / 1920.0).min(window.height() / 1080.0) * dpi,
    };
    let names: Vec<_> = runtime.renderers.keys().cloned().collect();
    let mut used = BTreeSet::new();
    for name in names {
        let active = match name.as_str() {
            "BattlePanel" => {
                matches!(
                    interface.mode,
                    UiMode::Playing | UiMode::Benchmark | UiMode::Editor
                ) && interface.gm_hud_enabled
            }
            "BattleMapPanel" => interface.mode == UiMode::Map,
            "BagAndBuildPanel" => interface.mode == UiMode::Inventory,
            "BattleLoadingPanel" => interface.mode == UiMode::Loading,
            "MainPanel" => interface.mode == UiMode::Menu && !runtime.menu_settings,
            "SettingPanel" => interface.mode == UiMode::Menu && runtime.menu_settings,
            _ => false,
        };
        if !active {
            continue;
        }
        used.insert(name.clone());
        let mut bindings = if name == "BattlePanel" {
            main_hud_bindings(&runtime.renderers[&name], &interface, &survival)
        } else if name == "BattleMapPanel" {
            map_bindings(&runtime.renderers[&name], &runtime, &telemetry, &interface)
        } else {
            screen_bindings(&runtime.renderers[&name], &interface)
        };
        if name == "BattlePanel"
            && let Err(error) = apply_hud_layout(
                &runtime.renderers[&name],
                &interface,
                viewport,
                dpi,
                &mut bindings,
            )
        {
            runtime.errors.insert(name, error.to_string());
            continue;
        }
        let key = format!(
            "{}:{}:{}:{}:{}:{}",
            viewport.pixels[0],
            viewport.pixels[1],
            viewport.scale_factor,
            dpi,
            interface.mode as u8,
            crate::source_ui::binding_evidence(&bindings)
        );
        if runtime.caches.get(&name).is_some_and(|c| c.key == key) {
            continue;
        }
        let Some(renderer) = runtime.renderers.get_mut(&name) else {
            continue;
        };
        match renderer.frame(viewport, 100.0, &bindings, &mut images) {
            Ok(frame) => {
                let report = frame.report.clone();
                let layouts = frame.layouts.clone();
                if name == "BattleMapPanel" {
                    runtime.map_hit = layouts
                        .iter()
                        .find(|l| l.path == "BattleMapPanel/GameObject/m_map_scroll/m_Map")
                        .and_then(|map| {
                            let mask = layouts
                                .iter()
                                .find(|l| l.path == "BattleMapPanel/GameObject/m_map_scroll")?;
                            let corners = mask.screen_corners?;
                            let min = corners
                                .iter()
                                .map(|p| Vec2::from_array(*p))
                                .reduce(Vec2::min)?;
                            let max = corners
                                .iter()
                                .map(|p| Vec2::from_array(*p))
                                .reduce(Vec2::max)?;
                            Some(MapHit {
                                clip: Rect::from_corners(min, max),
                                local_from_screen: Mat4::from_cols_array(&map.screen_from_local)
                                    .inverse(),
                                rect: map.local_rect?,
                            })
                        });
                }
                if name == "BattlePanel" {
                    runtime.widget_rects.clear();
                    for widget in WidgetId::ALL {
                        if let Some(prefix) = widget_source_path(widget)
                            && let Some(rect) = source_bounds(&layouts, prefix)
                        {
                            runtime
                                .widget_rects
                                .insert(widget, Rect::from_corners(rect.min / dpi, rect.max / dpi));
                        }
                    }
                }
                let cache = runtime.caches.entry(name.clone()).or_default();
                update_source_frame(
                    &mut commands,
                    cache,
                    frame,
                    dpi,
                    &mut images,
                    &mut materials,
                    &mut nodes,
                    &mut text,
                    &mut fonts,
                    &mut visibility,
                );
                if interface.mode != UiMode::Editor {
                    update_source_controls(&mut commands, cache, &layouts, dpi);
                }
                cache.key = key;
                runtime.reports.insert(name, report);
            }
            Err(error) => {
                runtime.errors.insert(name, error.to_string());
            }
        }
    }
    if matches!(
        interface.mode,
        UiMode::Playing | UiMode::Benchmark | UiMode::Editor
    ) && interface.gm_hud_enabled
        && let Some(native) = minimap.minimap.as_mut()
    {
        let config = interface.editor.layout().widgets.get(&WidgetId::Minimap);
        if let Some(config) = config
            && config.visible
        {
            let placement = config.placement(WidgetId::Minimap, [window.width(), window.height()]);
            let rect = Rect::from_corners(
                Vec2::from_array(placement.position),
                Vec2::from_array(placement.position) + Vec2::from_array(placement.size),
            );
            runtime.minimap_rect = Some(rect);
            let state = state_from_simulation(&crate::hud_painter::MinimapContext {
                player_position: telemetry.player_position,
                yaw: telemetry.yaw,
                nodes: &survival.0.nodes,
                structures: &survival.0.structures,
                npcs: &survival.0.npcs,
                vehicles: &survival.0.vehicles,
                tick: ServerTick(clock.tick),
                marker: interface.map_marker,
                sound_cues: &interface.sound_cues,
            });
            let v = CanvasViewport {
                scale_factor: placement.render_scale * dpi,
                ..viewport
            };
            let name = "BattleMinimap".to_string();
            used.insert(name.clone());
            let bindings = native.bindings(rect, v, &state, dpi);
            if let Ok(bindings) = bindings {
                let key = format!(
                    "{}:{}:{}:{}:{}:{}:{}",
                    v.pixels[0],
                    v.pixels[1],
                    v.scale_factor,
                    dpi,
                    config.opacity,
                    interface.mode as u8,
                    crate::source_ui::binding_evidence(&bindings)
                );
                if !runtime.caches.get(&name).is_some_and(|c| c.key == key) {
                    match native.renderer.frame(v, 100.0, &bindings, &mut images) {
                        Ok(mut frame) => {
                            for graphic in &mut frame.graphics {
                                match graphic {
                                    PreparedGraphic::Image { tint, .. } => tint.w *= config.opacity,
                                    PreparedGraphic::Text { color, .. } => {
                                        *color = color.with_alpha(color.alpha() * config.opacity)
                                    }
                                }
                            }
                            frame.report.bindings["native_panel_opacity"] =
                                serde_json::json!(config.opacity);
                            let report = frame.report.clone();
                            let layouts = frame.layouts.clone();
                            let cache = runtime.caches.entry(name.clone()).or_default();
                            update_source_frame(
                                &mut commands,
                                cache,
                                frame,
                                dpi,
                                &mut images,
                                &mut materials,
                                &mut nodes,
                                &mut text,
                                &mut fonts,
                                &mut visibility,
                            );
                            if interface.mode != UiMode::Editor {
                                update_source_controls(&mut commands, cache, &layouts, dpi);
                            }
                            cache.key = key;
                            runtime.reports.insert(name, report);
                        }
                        Err(e) => {
                            minimap.error = Some(e.to_string());
                        }
                    }
                }
            }
        }
    }
    for (name, cache) in &runtime.caches {
        for (key, entry) in &cache.nodes {
            if let Ok(mut visible) = visibility.get_mut(entry.entity) {
                *visible = if used.contains(name) && cache.active.contains(key) {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
/// Applies a retained source frame to native Bevy UI entities. Capture tools use
/// this same presentation path as the game, including source texture sampling.
pub fn update_source_frame(
    commands: &mut Commands,
    cache: &mut SourcePanelView,
    frame: PanelFrame,
    dpi: f32,
    images: &mut Assets<Image>,
    materials: &mut Assets<SourceUiMaterial>,
    nodes: &mut Query<&mut Node>,
    texts: &mut Query<&mut Text>,
    fonts: &mut Query<&mut TextFont>,
    visibility: &mut Query<&mut Visibility>,
) {
    let mut alive = BTreeSet::new();
    let screen_size = Vec2::new(
        frame.report.viewport.pixels[0] as f32,
        frame.report.viewport.pixels[1] as f32,
    );
    for (index, graphic) in frame.graphics.into_iter().enumerate() {
        match graphic {
            PreparedGraphic::Image {
                key,
                source: _,
                path,
                origin,
                size,
                atlas,
                coordinates,
                tint,
            } => {
                alive.insert(key.clone());
                let node = position_node(origin / dpi, size.as_vec2() / dpi);
                if let Some(entry) = cache.nodes.get(&key) {
                    let mut coordinates_changed = false;
                    if let Ok(mut n) = nodes.get_mut(entry.entity) {
                        *n = node;
                    }
                    if let Some(handle) = &entry.coordinates
                        && let Some(mut image) = images.get_mut(handle)
                        && (image.data.as_ref() != Some(&coordinates)
                            || image.texture_descriptor.size.width != size.x
                            || image.texture_descriptor.size.height != size.y)
                    {
                        image.data = Some(coordinates);
                        image.texture_descriptor.size.width = size.x;
                        image.texture_descriptor.size.height = size.y;
                        coordinates_changed = true;
                    }
                    if let Some(handle) = &entry.material
                        && let Some(mut material) = materials.get_mut(handle)
                    {
                        // Updating an Image replaces its GPU texture view. Refresh
                        // the cached material bind group even if its handles match.
                        if coordinates_changed || material.tint != tint || material.atlas != atlas {
                            material.tint = tint;
                            material.atlas = atlas;
                        }
                    }
                    commands.entity(entry.entity).insert(ZIndex(index as i32));
                } else {
                    let mut image = Image::new_uninit(
                        Extent3d {
                            width: size.x,
                            height: size.y,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        TextureFormat::Rgba32Float,
                        RenderAssetUsages::default(),
                    );
                    image.data = Some(coordinates);
                    let coordinates = images.add(image);
                    let material = materials.add(SourceUiMaterial {
                        tint,
                        atlas,
                        coordinates: coordinates.clone(),
                    });
                    let entity = commands
                        .spawn((
                            Name::new(path),
                            SourceVisual,
                            node,
                            MaterialNode(material.clone()),
                            ZIndex(index as i32),
                            Pickable::IGNORE,
                        ))
                        .id();
                    cache.nodes.insert(
                        key,
                        GraphicEntity {
                            entity,
                            coordinates: Some(coordinates),
                            material: Some(material),
                            clip: None,
                        },
                    );
                }
            }
            PreparedGraphic::Text {
                key,
                source: _,
                path,
                origin,
                size,
                text,
                font,
                font_size,
                color,
                alignment,
                rotation,
                clips,
            } => {
                let half = size * 0.5;
                let rotated_half = Rot2::radians(rotation) * half;
                let clips =
                    clips.unwrap_or_else(|| vec![Rect::from_corners(Vec2::ZERO, screen_size)]);
                for (clip_index, clip) in clips.into_iter().enumerate() {
                    let key = format!("{key}:text-clip:{clip_index}");
                    alive.insert(key.clone());
                    let mut clip_node = position_node(clip.min / dpi, clip.size() / dpi);
                    clip_node.overflow = Overflow::clip();
                    let node =
                        position_node((origin - half + rotated_half - clip.min) / dpi, size / dpi);
                    let justify = match alignment.rem_euclid(3) {
                        1 => Justify::Center,
                        2 => Justify::Right,
                        _ => Justify::Left,
                    };
                    if let Some(entry) = cache.nodes.get(&key) {
                        if let Some(parent) = entry.clip {
                            if let Ok(mut n) = nodes.get_mut(parent) {
                                *n = clip_node;
                            }
                            commands.entity(parent).insert(ZIndex(index as i32));
                        }
                        if let Ok(mut n) = nodes.get_mut(entry.entity) {
                            *n = node;
                        }
                        if let Ok(mut t) = texts.get_mut(entry.entity)
                            && t.0 != text
                        {
                            t.0 = text.clone();
                        }
                        if let Ok(mut f) = fonts.get_mut(entry.entity) {
                            f.font = font.clone().into();
                            f.font_size = FontSize::Px(font_size / dpi);
                        }
                        commands.entity(entry.entity).insert((
                            TextColor(color),
                            TextLayout::justify(justify),
                            UiTransform::from_rotation(Rot2::radians(rotation)),
                            ZIndex(0),
                        ));
                    } else {
                        let parent = commands
                            .spawn((
                                clip_node,
                                SourceVisual,
                                ZIndex(index as i32),
                                Pickable::IGNORE,
                            ))
                            .id();
                        let entity = commands
                            .spawn((
                                Name::new(path.clone()),
                                SourceVisual,
                                node,
                                Text::new(text.clone()),
                                TextFont {
                                    font: font.clone().into(),
                                    font_size: FontSize::Px(font_size / dpi),
                                    ..default()
                                },
                                TextColor(color),
                                TextLayout::justify(justify),
                                UiTransform::from_rotation(Rot2::radians(rotation)),
                                ZIndex(0),
                                Pickable::IGNORE,
                            ))
                            .id();
                        commands.entity(parent).add_child(entity);
                        cache.nodes.insert(
                            key,
                            GraphicEntity {
                                entity,
                                coordinates: None,
                                material: None,
                                clip: Some(parent),
                            },
                        );
                    }
                }
            }
        }
    }
    for (key, entry) in &cache.nodes {
        if let Ok(mut visible) = visibility.get_mut(entry.entity) {
            *visible = if alive.contains(key) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
    let retired: Vec<_> = cache
        .nodes
        .keys()
        .filter(|key| !alive.contains(*key) && !key.starts_with("control:"))
        .cloned()
        .collect();
    for key in retired {
        if let Some(entry) = cache.nodes.remove(&key) {
            commands.entity(entry.entity).despawn();
            if let Some(parent) = entry.clip {
                commands.entity(parent).despawn();
            }
            if let Some(coordinates) = entry.coordinates {
                images.remove(coordinates.id());
            }
            if let Some(material) = entry.material {
                materials.remove(material.id());
            }
        }
    }
    cache.active = alive;
}
fn position_node(origin: Vec2, size: Vec2) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(origin.x),
        top: Val::Px(origin.y),
        width: Val::Px(size.x),
        height: Val::Px(size.y),
        ..default()
    }
}

fn button(commands: &mut Commands, parent: Entity, label: &str, action: UiAction) {
    let entity = commands
        .spawn((
            Button,
            Hovered::default(),
            TabIndex(0),
            Action(action),
            Node {
                width: percent(100),
                min_height: px(32),
                flex_shrink: 0.0,
                padding: UiRect::all(px(6)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.12, 0.16, 0.94)),
            ZIndex(20_000),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ));
        })
        .observe(
            |event: On<Activate>, query: Query<&Action>, mut actions: ResMut<UiActions>| {
                if let Ok(action) = query.get(event.entity) {
                    actions.0.push(action.0.clone());
                }
            },
        )
        .id();
    commands.entity(parent).add_child(entity);
}
fn label(commands: &mut Commands, parent: Entity, value: impl Into<String>) -> Entity {
    let child = commands
        .spawn((
            Text::new(value),
            Node {
                flex_shrink: 0.0,
                ..default()
            },
            TextFont {
                font_size: FontSize::Px(15.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(parent).add_child(child);
    child
}

fn catalog_label(
    commands: &mut Commands,
    parent: Entity,
    value: impl Into<String>,
    font: Option<&Handle<Font>>,
) {
    let entity = label(commands, parent, value);
    if let Some(font) = font {
        commands.entity(entity).insert(TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(15.0),
            ..default()
        });
    }
}

fn online_field(commands: &mut Commands, parent: Entity, field: OnlineField, value: &str) {
    let mut text = EditableText::new(value);
    text.allow_newlines = false;
    text.max_characters = Some(match field {
        OnlineField::Address => 255,
        OnlineField::Invitation => 4096,
    });
    let entity = commands
        .spawn((
            field,
            TextInput,
            text,
            EditableTextFilter::new(|c| !c.is_control()),
            TextCursorStyle::default(),
            TabIndex(0),
            Node {
                width: percent(100),
                min_height: px(34),
                flex_shrink: 0.0,
                padding: UiRect::all(px(6)),
                ..default()
            },
            TextLayout::no_wrap(),
            TextFont {
                font_size: FontSize::Px(15.0),
                ..default()
            },
            TextColor(Color::WHITE),
            BackgroundColor(Color::srgb(0.08, 0.10, 0.14)),
        ))
        .observe(
            |event: On<TextEditChange>,
             fields: Query<(&OnlineField, &EditableText)>,
             mut online: ResMut<Online>| {
                if let Ok((field, text)) = fields.get(event.event_target()) {
                    let value = text.value().to_string();
                    match field {
                        OnlineField::Address => online.address = value,
                        OnlineField::Invitation => online.code = value,
                    }
                }
            },
        )
        .id();
    commands.entity(parent).add_child(entity);
}

#[allow(clippy::too_many_arguments)]
fn render_overlay(
    mut commands: Commands,
    mut runtime: ResMut<NativeUiRuntime>,
    mut interface: ResMut<Interface>,
    survival: Res<SimulationSurvival>,
    world: Option<Res<shared::world::LoadedWorld>>,
    time: Res<Time>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    old: Query<Entity, With<Overlay>>,
    online: Res<Online>,
    source_inventory: Option<Res<shared::source_inventory::SourceInventoryWorld>>,
) {
    let Ok(window) = windows.single() else { return };
    interface.tick_sound_cues();
    interface.loading_timer += time.delta_secs().min(0.1);
    interface.loading_progress = if world.is_some() && runtime.renderers.contains_key("BattlePanel")
    {
        1.0
    } else {
        0.0
    };
    let state = survival.0.player(PlayerId(0));
    let mut key = format!(
        "{:?}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}:{:?}:{}:{}:{}:{}:{}:{}",
        interface.mode as u8,
        interface.show_gm_panel,
        interface.volumetric_fog_steps,
        interface.editor.selected as u8,
        serde_json::to_string(interface.editor.layout()).unwrap_or_default(),
        interface.editor.error.as_deref().unwrap_or(""),
        state
            .map(|s| format!("{:?}", s.inventory))
            .unwrap_or_default(),
        interface.gm_fly_mode,
        interface.gm_fog_enabled,
        interface.gm_hud_enabled,
        window.width(),
        window.height(),
        interface.loading_progress,
        interface.selected_slot,
        runtime.errors,
        runtime.catalog_page,
        runtime.drawing_page,
        runtime.menu_settings,
        online.connected,
        online.connecting,
        online.status
    );
    if let Some(inventory) = &source_inventory {
        key.push_str(&format!(
            ":source-inventory:{}:{}",
            online.local_player_id().0,
            inventory.revision()
        ));
    }
    if runtime.overlay_key == key {
        return;
    }
    runtime.overlay_key = key;
    for entity in &old {
        commands.entity(entity).despawn();
    }
    if !uses_ui_tab_navigation(&interface) {
        return;
    }
    let width = 340.0_f32.min(window.width() - 20.0).max(180.0);
    let parent = commands
        .spawn((
            Overlay,
            ScrollArea,
            TabGroup::default(),
            Name::new("Desktop controls"),
            Node {
                position_type: PositionType::Absolute,
                right: px(10),
                top: px(10),
                width: px(width),
                max_height: percent(95),
                overflow: Overflow::scroll_y(),
                flex_direction: FlexDirection::Column,
                row_gap: px(5),
                padding: UiRect::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.025, 0.04, 0.95)),
            GlobalZIndex(20_000),
        ))
        .id();
    match interface.mode {
        UiMode::Loading => {
            label(&mut commands, parent, "MIDNIGHT SURVIVAL");
            label(
                &mut commands,
                parent,
                if interface.loading_progress >= 1.0 {
                    "World ready"
                } else {
                    "Loading world…"
                },
            );
            if interface.loading_progress >= 1.0 {
                button(&mut commands, parent, "Enter world", UiAction::Play);
            }
        }
        UiMode::Menu => {
            label(&mut commands, parent, "MIDNIGHT SURVIVAL");
            button(&mut commands, parent, "Resume", UiAction::Play);
            button(&mut commands, parent, "Inventory", UiAction::Inventory);
            button(&mut commands, parent, "Map", UiAction::Map);
            button(&mut commands, parent, "HUD editor", UiAction::Editor);
            button(&mut commands, parent, "Settings", UiAction::Settings);
            label(&mut commands, parent, "Server address");
            online_field(&mut commands, parent, OnlineField::Address, &online.address);
            label(&mut commands, parent, "Invitation code");
            online_field(&mut commands, parent, OnlineField::Invitation, &online.code);
            label(&mut commands, parent, &online.status);
            if online.connected || online.connecting {
                button(&mut commands, parent, "Disconnect", UiAction::Disconnect);
            } else {
                button(&mut commands, parent, "Connect", UiAction::Connect);
            }
            button(
                &mut commands,
                parent,
                &format!("Cloud quality: {} steps", interface.volumetric_fog_steps),
                UiAction::Clouds,
            );
            button(&mut commands, parent, "Quit", UiAction::Quit);
        }
        UiMode::Inventory => {
            label(&mut commands, parent, "Inventory");
            if let Some(catalog) = world.as_ref().and_then(|w| w.gameplay.as_ref()) {
                let catalog_font = runtime
                    .renderers
                    .get("BagAndBuildPanel")
                    .and_then(SourcePanelRenderer::embedded_font);
                let bag = source_inventory
                    .as_ref()
                    .and_then(|inventory| inventory.player(online.local_player_id()));
                if let Some(snapshot) = bag.and_then(|bag| bag.snapshot()) {
                    label(
                        &mut commands,
                        parent,
                        format!(
                            "Backpack · {} / {}",
                            snapshot.bag_items.len(),
                            bag.and_then(|bag| bag.visible_capacity()).unwrap_or(0)
                        ),
                    );
                    for item in snapshot.bag_items.iter().take(20) {
                        let name = catalog
                            .items
                            .get(&item.item_id)
                            .map(|item| item.name.as_str())
                            .unwrap_or("Unknown item");
                        catalog_label(
                            &mut commands,
                            parent,
                            format!("{name} ×{}", item.number),
                            catalog_font.as_ref(),
                        );
                    }
                    if snapshot.bag_items.len() > 20 {
                        label(
                            &mut commands,
                            parent,
                            format!("{} more items", snapshot.bag_items.len() - 20),
                        );
                    }
                    for (index, item) in snapshot.quick_use_items.iter().take(20) {
                        let name = catalog
                            .items
                            .get(&item.item_id)
                            .map(|item| item.name.as_str())
                            .unwrap_or("Unknown item");
                        catalog_label(
                            &mut commands,
                            parent,
                            format!("Quick {index} · {name} ×{}", item.number),
                            catalog_font.as_ref(),
                        );
                    }
                    for item in snapshot.equip_items.iter().take(20) {
                        let name = catalog
                            .items
                            .get(&item.item_id)
                            .map(|item| item.name.as_str())
                            .unwrap_or("Unknown item");
                        catalog_label(
                            &mut commands,
                            parent,
                            format!("Equipped · {name}"),
                            catalog_font.as_ref(),
                        );
                    }
                } else {
                    label(&mut commands, parent, "Waiting for inventory");
                }
                const PAGE_SIZE: usize = 20;
                let pages = catalog.items.len().div_ceil(PAGE_SIZE).max(1);
                let page = runtime.catalog_page.min(pages - 1);
                label(&mut commands, parent, "Item catalog · read-only");
                label(
                    &mut commands,
                    parent,
                    format!(
                        "Page {} / {} · {} original IDs",
                        page + 1,
                        pages,
                        catalog.items.len()
                    ),
                );
                for item in catalog
                    .items
                    .values()
                    .skip(page * PAGE_SIZE)
                    .take(PAGE_SIZE)
                {
                    catalog_label(
                        &mut commands,
                        parent,
                        format!("{} · {}", item.id, item.name),
                        catalog_font.as_ref(),
                    );
                }
                if page > 0 {
                    button(
                        &mut commands,
                        parent,
                        "Previous page",
                        UiAction::CatalogPage(-1),
                    );
                }
                if page + 1 < pages {
                    button(&mut commands, parent, "Next page", UiAction::CatalogPage(1));
                }
                label(&mut commands, parent, "Special recipes · read-only");
                for recipe in catalog.recipes.values().take(PAGE_SIZE) {
                    label(
                        &mut commands,
                        parent,
                        format!(
                            "{} ×{} · {} s · {:?}",
                            recipe.id, recipe.output_count, recipe.time, recipe.material
                        ),
                    );
                }
                let pages = catalog.drawings.len().div_ceil(PAGE_SIZE).max(1);
                let page = runtime.drawing_page.min(pages - 1);
                label(
                    &mut commands,
                    parent,
                    format!("Blueprints · read-only · page {} / {}", page + 1, pages),
                );
                for drawing in catalog
                    .drawings
                    .values()
                    .skip(page * PAGE_SIZE)
                    .take(PAGE_SIZE)
                {
                    catalog_label(
                        &mut commands,
                        parent,
                        format!(
                            "{} · {} → {} ×{}",
                            drawing.id, drawing.name, drawing.target_item_id, drawing.product_count
                        ),
                        catalog_font.as_ref(),
                    );
                    label(
                        &mut commands,
                        parent,
                        format!(
                            "{} s · materials {:?} · workbench {} level {}",
                            drawing.time,
                            drawing.material,
                            drawing.workbench,
                            drawing.workbench_level
                        ),
                    );
                    label(
                        &mut commands,
                        parent,
                        format!(
                            "Level {} · toolbox {} · normal {} · civilization {} · nostalgia {} · firepower {}",
                            drawing.level_limit,
                            drawing.toolbox_level,
                            drawing.normal,
                            drawing.civilization_normal,
                            drawing.nostalgia,
                            drawing.firepower_normal
                        ),
                    );
                }
                if page > 0 {
                    button(
                        &mut commands,
                        parent,
                        "Previous blueprints",
                        UiAction::DrawingPage(-1),
                    );
                }
                if page + 1 < pages {
                    button(
                        &mut commands,
                        parent,
                        "Next blueprints",
                        UiAction::DrawingPage(1),
                    );
                }
            } else if let Some(player) = state {
                for slot in 0..HOTBAR_SLOTS {
                    let index = SlotIndex::Hotbar(slot as u8);
                    let value = player
                        .inventory
                        .slot(index)
                        .map(|s| format!("{} ×{}", s.kind.label_en(), s.count))
                        .unwrap_or_else(|| "Empty".into());
                    button(
                        &mut commands,
                        parent,
                        &format!("{}: {value}", slot + 1),
                        UiAction::Slot(index),
                    );
                }
                label(&mut commands, parent, "Backpack");
                for slot in 0..BACKPACK_SLOTS {
                    let index = SlotIndex::Backpack(slot as u8);
                    let value = player
                        .inventory
                        .slot(index)
                        .map(|s| format!("{} ×{}", s.kind.label_en(), s.count))
                        .unwrap_or_else(|| "Empty".into());
                    button(
                        &mut commands,
                        parent,
                        &format!("{}: {value}", slot + 1),
                        UiAction::Slot(index),
                    );
                }
                if world
                    .as_ref()
                    .is_some_and(|w| w.descriptor.map != shared::level::MapId::ImportedIsland)
                {
                    for recipe in RecipeId::ALL {
                        button(
                            &mut commands,
                            parent,
                            recipe.spec().output.label_en(),
                            UiAction::Craft(recipe),
                        );
                    }
                }
            }
            button(&mut commands, parent, "Back", UiAction::Play);
        }
        UiMode::Map => {
            label(&mut commands, parent, "Map");
            button(&mut commands, parent, "Zoom in", UiAction::MapZoom(0.25));
            button(&mut commands, parent, "Zoom out", UiAction::MapZoom(-0.25));
            button(&mut commands, parent, "Clear marker", UiAction::ClearMarker);
            button(&mut commands, parent, "Back", UiAction::Play);
        }
        UiMode::Editor => {
            label(&mut commands, parent, "HUD editor");
            for widget in WidgetId::ALL {
                button(
                    &mut commands,
                    parent,
                    widget.label(),
                    UiAction::SelectWidget(widget),
                );
            }
            for preset in [
                HudPreset::Desktop,
                HudPreset::TwoFingers,
                HudPreset::ThreeFingers,
                HudPreset::FourFingers,
            ] {
                button(
                    &mut commands,
                    parent,
                    preset.label(),
                    UiAction::Preset(preset),
                );
            }
            button(&mut commands, parent, "Scale +", UiAction::Scale(0.1));
            button(&mut commands, parent, "Scale −", UiAction::Scale(-0.1));
            button(&mut commands, parent, "Opacity +", UiAction::Opacity(0.1));
            button(&mut commands, parent, "Opacity −", UiAction::Opacity(-0.1));
            button(&mut commands, parent, "Change anchor", UiAction::Anchor);
            button(
                &mut commands,
                parent,
                "Toggle visibility",
                UiAction::Visibility,
            );
            if let Some(error) = &interface.editor.error {
                label(&mut commands, parent, error);
            }
            button(&mut commands, parent, "Save", UiAction::SaveEditor);
            button(&mut commands, parent, "Cancel", UiAction::CancelEditor);
        }
        _ => {}
    }
    if interface.show_gm_panel {
        label(&mut commands, parent, "Developer controls");
        button(&mut commands, parent, "Fly", UiAction::Fly);
        button(&mut commands, parent, "Fog", UiAction::Fog);
        button(&mut commands, parent, "HUD", UiAction::Hud);
        button(
            &mut commands,
            parent,
            &format!("Cloud quality: {}", interface.volumetric_fog_steps),
            UiAction::Clouds,
        );
    }
}

fn editor_drag(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut interface: ResMut<Interface>,
    mut runtime: ResMut<NativeUiRuntime>,
    scroll: Res<AccumulatedMouseScroll>,
    mut minimap: ResMut<SourceMinimapHud>,
    hovered: Query<&Hovered, With<Button>>,
) {
    let Ok(window) = windows.single() else { return };
    if interface.mode == UiMode::Map {
        runtime.map_zoom = (runtime.map_zoom + scroll.delta.y * 0.1).clamp(0.5, 4.0);
        if let Some(pointer) = window.cursor_position() {
            if mouse.just_pressed(MouseButton::Left)
                && !hovered.iter().any(Hovered::get)
                && pointer.x < window.width() - 360.0
                && let Some(hit) = &runtime.map_hit
                && hit.clip.contains(pointer * window.scale_factor())
            {
                let local = hit
                    .local_from_screen
                    .transform_point3((pointer * window.scale_factor()).extend(0.0))
                    .truncate();
                let uv = (local - Vec2::from_array(hit.rect.min)) / Vec2::from_array(hit.rect.size);
                if (0.0..=1.0).contains(&uv.x) && (0.0..=1.0).contains(&uv.y) {
                    interface.map_marker = Some(shared::map_projection::overview_uv_to_world(
                        [uv.x, 1.0 - uv.y],
                        shared::level::MAP_OVERVIEW_HALF_EXTENT,
                    ));
                }
            }
            if mouse.pressed(MouseButton::Right) {
                if let Some(previous) = runtime.map_dragging {
                    let delta = (pointer - previous)
                        / (window.width() / 1920.0).min(window.height() / 1080.0);
                    runtime.map_offset[0] += delta.x;
                    runtime.map_offset[1] -= delta.y;
                }
                runtime.map_dragging = Some(pointer);
            } else {
                runtime.map_dragging = None;
            }
        }
        return;
    }
    runtime.map_dragging = None;
    if matches!(interface.mode, UiMode::Playing | UiMode::Benchmark)
        && let Some(pointer) = window.cursor_position()
        && runtime
            .minimap_rect
            .is_some_and(|rect| rect.contains(pointer))
        && scroll.delta.y != 0.0
        && let Some(native) = minimap.minimap.as_mut()
    {
        native.set_zoom(native.zoom() + scroll.delta.y);
    }
    if interface.mode != UiMode::Editor || !mouse.pressed(MouseButton::Left) {
        runtime.dragging = None;
        return;
    }
    let Some(pointer) = window.cursor_position() else {
        return;
    };
    let viewport = [window.width(), window.height()];
    if let Some((widget, last)) = runtime.dragging {
        let delta = pointer - last;
        if let Some(config) = interface
            .editor
            .draft_mut()
            .and_then(|l| l.widgets.get_mut(&widget))
        {
            let p = config.placement(widget, viewport);
            config.set_position(
                widget,
                viewport,
                [p.position[0] + delta.x, p.position[1] + delta.y],
            );
        }
        runtime.dragging = Some((widget, pointer));
    } else if mouse.just_pressed(MouseButton::Left) && pointer.x < window.width() - 360.0 {
        let selected = interface.editor.selected;
        if selected != WidgetId::Minimap {
            if runtime
                .widget_rects
                .get(&selected)
                .is_some_and(|rect| rect.contains(pointer))
            {
                runtime.dragging = Some((selected, pointer));
            }
        } else if let Some(config) = interface.editor.layout().widgets.get(&selected) {
            let p = config.placement(selected, viewport);
            let rect = Rect::from_corners(
                Vec2::from_array(p.position),
                Vec2::from_array(p.position) + Vec2::from_array(p.size),
            );
            if rect.contains(pointer) {
                runtime.dragging = Some((selected, pointer));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{asset::AssetEvent, ecs::message::MessageCursor};
    struct TestStorage;
    impl crate::storage::StorageService for TestStorage {
        fn load_hud(&self) -> Result<crate::storage::LoadOutcome, crate::storage::StorageError> {
            Ok(crate::storage::LoadOutcome {
                layout: crate::hud::HudLayout::default(),
                notice: None,
            })
        }
        fn save_hud(&self, _: &crate::hud::HudLayout) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }
    }
    #[test]
    fn source_snapshot_and_size_update_refresh_native_inventory_in_the_application_frame() {
        use shared::source_inventory::{
            GameplayCatalog,
            inventory_wire::{S_BAG_SIZE_CHANGE_TYPE_ID, S_ITEMS_TYPE_ID},
        };
        use shared::{
            source_inventory::{
                SourceInventoryBodyMessage, SourceInventoryPlugin, SourceInventorySet,
                SourceInventoryWorld,
            },
            world::LoadedWorld,
        };
        use std::sync::Arc;
        let catalog = Arc::new(GameplayCatalog::default());
        let mut inventory = SourceInventoryWorld::from_catalog(catalog.clone());
        inventory.join(PlayerId(0));
        let mut interface = Interface::new(&TestStorage, false);
        interface.mode = UiMode::Inventory;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SourceInventoryPlugin))
            .insert_resource(inventory)
            .init_resource::<NativeUiRuntime>()
            .insert_resource(interface)
            .insert_resource(SimulationSurvival(shared::survival::SurvivalWorld::new(
                shared::level::MapId::Polygon,
            )))
            .insert_resource(Online::new(&crate::launch::ClientOptions::default()))
            .insert_resource(LoadedWorld {
                descriptor: shared::world::WorldDescriptor::fixture(shared::level::MapId::Polygon),
                terrain: None,
                gameplay: Some(catalog),
                spawn: [0.0, 1.0, 0.0],
            })
            .add_systems(Update, render_overlay.after(SourceInventorySet::Apply));
        app.world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow));
        // Independent empty SItems source body, capacity 2.
        app.world_mut().write_message(SourceInventoryBodyMessage {
            player: PlayerId(0),
            type_id: S_ITEMS_TYPE_ID,
            body: vec![
                123, 123, 123, 123, 14, 14, 14, 14, 7, 7, 7, 7, 63, 63, 63, 63, 81, 104, 106, 106,
                106,
            ],
        });
        app.update();
        let mut texts = app.world_mut().query::<&Text>();
        assert!(
            texts
                .iter(app.world())
                .any(|text| text.0 == "Backpack · 0 / 2")
        );
        app.world_mut().write_message(SourceInventoryBodyMessage {
            player: PlayerId(0),
            type_id: S_BAG_SIZE_CHANGE_TYPE_ID,
            body: vec![78, 77, 77, 77],
        });
        app.update();
        assert!(
            texts
                .iter(app.world())
                .any(|text| text.0 == "Backpack · 0 / 3")
        );
        assert!(
            !texts
                .iter(app.world())
                .any(|text| text.0 == "Backpack · 0 / 2")
        );
        let revision = app.world().resource::<SourceInventoryWorld>().revision();
        app.world_mut().write_message(SourceInventoryBodyMessage {
            player: PlayerId(0),
            type_id: S_BAG_SIZE_CHANGE_TYPE_ID,
            body: vec![0],
        });
        app.update();
        assert_eq!(
            app.world().resource::<SourceInventoryWorld>().revision(),
            revision
        );
        assert!(
            texts
                .iter(app.world())
                .any(|text| text.0 == "Backpack · 0 / 3")
        );
    }
    #[test]
    fn tab_opens_inventory_once_then_navigates_inventory_and_gm_controls() {
        use bevy::{
            camera::visibility::VisibilityPlugin,
            input::{InputPlugin, keyboard::Key},
            input_focus::{InputDispatchPlugin, InputFocusPlugin},
            mesh::skinning::SkinnedMeshInverseBindposes,
            time::TimeUpdateStrategy,
        };
        use shared::{SimulationInput, SimulationWorld, level::MapId, physics::CollisionWorld};
        use std::time::Duration;

        let mut app = App::new();
        let mut survival = shared::survival::SurvivalWorld::new(MapId::Polygon);
        survival
            .players
            .insert(PlayerId(0), shared::survival::PlayerSurvival::default());
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            TransformPlugin,
            VisibilityPlugin,
            InputPlugin,
            InputFocusPlugin,
            InputDispatchPlugin,
        ))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
        .init_asset::<Mesh>()
        .init_asset::<SkinnedMeshInverseBindposes>()
        .init_resource::<NativeUiRuntime>()
        .init_resource::<NativeUiInput>()
        .init_resource::<UiActions>()
        .init_resource::<Telemetry>()
        .init_resource::<SimulationInput>()
        .init_resource::<SimulationClock>()
        .init_resource::<SimulationMode>()
        .insert_resource(SimulationWorld(CollisionWorld::new(MapId::Polygon)))
        .insert_resource(SimulationSurvival(survival))
        .insert_resource(Interface::new(&TestStorage, false))
        .insert_resource(Online::new(&crate::launch::ClientOptions::default()))
        .add_message::<AppExit>()
        .add_systems(PreUpdate, update_input_boundary.in_set(NativeUiSet::Input))
        .add_systems(Update, render_overlay)
        .add_plugins(crate::desktop_input::DesktopInputPlugin);
        configure_tab_navigation(&mut app);
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: true,
                    ..default()
                },
                bevy::window::PrimaryWindow,
                bevy::window::CursorOptions::default(),
            ))
            .id();
        let key = |app: &mut App, key_code: KeyCode, state: ButtonState| {
            let logical_key = match key_code {
                KeyCode::Tab => Key::Tab,
                KeyCode::ShiftLeft => Key::Shift,
                KeyCode::KeyI => Key::Character("i".into()),
                KeyCode::Escape => Key::Escape,
                _ => unreachable!("unsupported fixture key"),
            };
            app.world_mut().write_message(KeyboardInput {
                key_code,
                logical_key,
                state,
                text: None,
                repeat: false,
                window,
            });
            app.update();
        };
        let tap = |app: &mut App, key_code| {
            key(app, key_code, ButtonState::Pressed);
            key(app, key_code, ButtonState::Released);
        };

        app.update();
        // Loading has a real overlay but no focusable controls yet.
        tap(&mut app, KeyCode::Tab);
        assert!(app.world().resource::<Interface>().mode == UiMode::Loading);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(window));

        app.world_mut().resource_mut::<Interface>().mode = UiMode::Playing;
        app.update();
        key(&mut app, KeyCode::Tab, ButtonState::Pressed);
        assert!(app.world().resource::<Interface>().mode == UiMode::Inventory);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(window));
        app.update();
        assert!(app.world().resource::<Interface>().mode == UiMode::Inventory);
        key(&mut app, KeyCode::Tab, ButtonState::Released);

        tap(&mut app, KeyCode::Tab);
        let first = app.world().resource::<InputFocus>().get().unwrap();
        assert!(app.world().get::<TabIndex>(first).is_some());
        assert!(app.world().resource::<Interface>().mode == UiMode::Inventory);
        tap(&mut app, KeyCode::Tab);
        let second = app.world().resource::<InputFocus>().get().unwrap();
        assert_ne!(first, second);
        assert!(app.world().resource::<Interface>().mode == UiMode::Inventory);
        key(&mut app, KeyCode::ShiftLeft, ButtonState::Pressed);
        tap(&mut app, KeyCode::Tab);
        key(&mut app, KeyCode::ShiftLeft, ButtonState::Released);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(first));

        tap(&mut app, KeyCode::KeyI);
        assert!(app.world().resource::<Interface>().mode == UiMode::Playing);
        tap(&mut app, KeyCode::KeyI);
        assert!(app.world().resource::<Interface>().mode == UiMode::Inventory);
        tap(&mut app, KeyCode::Escape);
        assert!(app.world().resource::<Interface>().mode == UiMode::Playing);

        app.world_mut().resource_mut::<Interface>().show_gm_panel = true;
        app.update();
        tap(&mut app, KeyCode::Tab);
        let first = app.world().resource::<InputFocus>().get().unwrap();
        assert!(app.world().get::<TabIndex>(first).is_some());
        tap(&mut app, KeyCode::Tab);
        assert_ne!(app.world().resource::<InputFocus>().get(), Some(first));
        let interface = app.world().resource::<Interface>();
        assert!(interface.mode == UiMode::Playing && interface.show_gm_panel);
        tap(&mut app, KeyCode::Escape);
        let interface = app.world().resource::<Interface>();
        assert!(interface.mode == UiMode::Playing && !interface.show_gm_panel);
    }
    #[test]
    fn modal_boundary_and_escape_restore_cursor_gameplay_state() {
        let mut app = App::new();
        app.init_resource::<NativeUiInput>();
        let mut interface = Interface::new(&TestStorage, false);
        interface.mode = UiMode::Playing;
        app.insert_resource(interface);
        app.add_systems(Update, update_input_boundary);
        app.update();
        assert!(!app.world().resource::<NativeUiInput>().blocks_gameplay);
        let button = app.world_mut().spawn((Button, Hovered(true))).id();
        app.update();
        assert!(app.world().resource::<NativeUiInput>().blocks_gameplay);
        app.world_mut().entity_mut(button).insert(Hovered(false));
        app.update();
        assert!(!app.world().resource::<NativeUiInput>().blocks_gameplay);
        app.world_mut().resource_mut::<Interface>().begin_editing();
        app.update();
        assert!(app.world().resource::<NativeUiInput>().blocks_gameplay);
        app.world_mut().resource_mut::<Interface>().escape();
        assert!(!app.world().resource::<Interface>().editor.is_editing());
        app.update();
        assert!(app.world().resource::<NativeUiInput>().blocks_gameplay);
        app.world_mut().resource_mut::<Interface>().escape();
        app.update();
        assert!(!app.world().resource::<NativeUiInput>().blocks_gameplay);
    }
    #[test]
    fn desktop_overlays_spawn_and_retire_native_controls_across_modes() {
        let mut app = App::new();
        let mut survival = shared::survival::SurvivalWorld::new(shared::level::MapId::Polygon);
        survival
            .players
            .insert(PlayerId(0), shared::survival::PlayerSurvival::default());
        app.init_resource::<NativeUiRuntime>()
            .init_resource::<Time>()
            .insert_resource(Interface::new(&TestStorage, false))
            .insert_resource(SimulationSurvival(survival))
            .insert_resource(Online::new(&crate::launch::ClientOptions::default()))
            .add_systems(Update, render_overlay);
        app.world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow));
        for mode in [
            UiMode::Loading,
            UiMode::Menu,
            UiMode::Inventory,
            UiMode::Map,
            UiMode::Editor,
        ] {
            app.world_mut().resource_mut::<Interface>().mode = mode;
            app.update();
            let mut overlays = app.world_mut().query_filtered::<Entity, With<Overlay>>();
            assert_eq!(overlays.iter(app.world()).count(), 1);
            let mut fields = app.world_mut().query::<(&OnlineField, &EditableText)>();
            let fields: Vec<_> = fields.iter(app.world()).collect();
            assert_eq!(fields.len(), if mode == UiMode::Menu { 2 } else { 0 });
            for (field, text) in fields {
                assert!(!text.allow_newlines);
                assert_eq!(
                    text.max_characters,
                    Some(match field {
                        OnlineField::Address => 255,
                        OnlineField::Invitation => 4096,
                    })
                );
            }
        }
        app.world_mut().resource_mut::<Interface>().mode = UiMode::Playing;
        app.update();
        let mut overlays = app.world_mut().query_filtered::<Entity, With<Overlay>>();
        assert_eq!(overlays.iter(app.world()).count(), 0);
        app.world_mut().resource_mut::<Interface>().show_gm_panel = true;
        app.update();
        assert_eq!(overlays.iter(app.world()).count(), 1);
    }
    #[test]
    fn cached_native_material_refreshes_when_coordinate_pixels_change() {
        #[derive(Resource, Default)]
        struct TestView(SourcePanelView);
        #[derive(Resource, Default)]
        struct PendingFrame(Option<PanelFrame>);
        #[allow(clippy::too_many_arguments)]
        fn present(
            mut commands: Commands,
            mut frame: ResMut<PendingFrame>,
            mut view: ResMut<TestView>,
            mut images: ResMut<Assets<Image>>,
            mut materials: ResMut<Assets<SourceUiMaterial>>,
            mut nodes: Query<&mut Node>,
            mut texts: Query<&mut Text>,
            mut fonts: Query<&mut TextFont>,
            mut visibility: Query<&mut Visibility>,
        ) {
            if let Some(frame) = frame.0.take() {
                update_source_frame(
                    &mut commands,
                    &mut view.0,
                    frame,
                    1.0,
                    &mut images,
                    &mut materials,
                    &mut nodes,
                    &mut texts,
                    &mut fonts,
                    &mut visibility,
                );
            }
        }
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Image>()
            .init_asset::<SourceUiMaterial>()
            .init_resource::<TestView>()
            .init_resource::<PendingFrame>()
            .add_systems(Update, present);
        let atlas = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::default());
        let frame = |u: f32| PanelFrame {
            report: PanelRenderReport {
                panel: "fixture".into(),
                source_apk_sha256: String::new(),
                layout_sha256: String::new(),
                viewport: CanvasViewport {
                    pixels: [4, 4],
                    scale_factor: 1.0,
                },
                reference_pixels_per_unit: 100.0,
                source_nodes: 1,
                decoded_atlases: 1,
                embedded_fonts: 0,
                bound_components: 0,
                bindings: serde_json::Value::Null,
                masks: 0,
                graphics: Vec::new(),
                limitations: Vec::new(),
            },
            graphics: vec![PreparedGraphic::Image {
                key: "fixture".into(),
                source: midnight_map_data::SourceObject {
                    bundle: "fixture".into(),
                    cab: "CAB-fixture".into(),
                    path_id: "1".into(),
                },
                path: "fixture".into(),
                origin: Vec2::ZERO,
                size: UVec2::ONE,
                atlas: atlas.clone(),
                coordinates: [u, 0.0, 0.0, 1.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
                tint: Vec4::ONE,
            }],
            layouts: Vec::new(),
        };
        let mut cursor = MessageCursor::<AssetEvent<SourceUiMaterial>>::default();
        app.world_mut().resource_mut::<PendingFrame>().0 = Some(frame(0.0));
        app.update();
        let material = app.world().resource::<TestView>().0.nodes["fixture"]
            .material
            .as_ref()
            .expect("native material")
            .id();
        cursor.read(app.world().resource()).for_each(drop);
        app.world_mut().resource_mut::<PendingFrame>().0 = Some(frame(0.5));
        app.update();
        assert!(
            cursor
                .read(app.world().resource())
                .any(|event| { matches!(event, AssetEvent::Modified { id } if *id == material) })
        );
        app.world_mut().resource_mut::<PendingFrame>().0 = Some(frame(0.5));
        app.update();
        assert!(
            !cursor
                .read(app.world().resource())
                .any(|event| { matches!(event, AssetEvent::Modified { id } if *id == material) })
        );
    }
}
