//! Keyboard/mouse adapter installed by the desktop runner.
//! An Android runner can install a touch adapter feeding SimulationInput instead.

use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_egui::EguiContexts;
use shared::{
    CharacterMotion, LocalPlayer, ResourceNodeMarker, SimulationClock, SimulationInput,
    SimulationMode, SimulationPose, SimulationSurvival, SimulationWorld, protocol::PlayerId,
    survival::HOTBAR_SLOTS,
};

use crate::{
    application::ScenicPlacementMarker,
    interface::{
        Interface, LookedTargetKind, Telemetry, UiMode, cycle_structure_kind, delete_looked_object,
        rotate_structure_socket, select_hotbar_slot, toggle_mount_vehicle, trigger_primary_action,
    },
    online::Online,
};

const MOUSE_SENSITIVITY: f32 = 0.002;

pub struct DesktopInputPlugin;

impl Plugin for DesktopInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, collect_input.after(bevy::input::InputSystems));
    }
}

#[allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::collapsible_if
)]
fn collect_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut windows: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
    mut interface: ResMut<Interface>,
    mut input: ResMut<SimulationInput>,
    mode: Res<SimulationMode>,
    clock: Res<SimulationClock>,
    mut physics: ResMut<SimulationWorld>,
    mut survival: ResMut<SimulationSurvival>,
    online: Res<Online>,
    mut local_player: Query<
        (&mut SimulationPose, &mut CharacterMotion, &mut Transform),
        With<LocalPlayer>,
    >,
    mut nodes: Query<&mut ResourceNodeMarker>,
    mut contexts: EguiContexts,
    (telemetry, mut commands, scenic_entities, mut exit): (
        Res<Telemetry>,
        Commands,
        Query<(Entity, &ScenicPlacementMarker)>,
        MessageWriter<AppExit>,
    ),
) {
    let Ok((mut window, mut cursor)) = windows.single_mut() else {
        input.0.clear();
        return;
    };
    if window.focused
        && (keyboard.just_pressed(KeyCode::F11)
            || ((keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight))
                && keyboard.just_pressed(KeyCode::Enter)))
    {
        window.mode = if matches!(window.mode, bevy::window::WindowMode::Windowed) {
            bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current)
        } else {
            bevy::window::WindowMode::Windowed
        };
    }
    if window.focused
        && matches!(interface.mode, UiMode::Playing)
        && (keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::KeyX))
    {
        if let Some(ref looked) = telemetry.looked_object {
            let mut deleted = false;
            if !matches!(
                looked.target_kind,
                Some(LookedTargetKind::Terrain(_)) | None
            ) {
                if let Some(msg) =
                    delete_looked_object(looked, &mut survival, &mut physics, Some(&mut commands))
                {
                    if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                        player.notice = Some(msg);
                    }
                    deleted = true;
                }
            }
            if !deleted {
                for (entity, marker) in &scenic_entities {
                    let d = (marker.position - Vec3::from_array(looked.hit_coords)).length();
                    if d < 3.5 {
                        commands.entity(entity).despawn();
                        if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                            player.notice = Some("🗑️ Декоративний об'єкт видалено!".to_string());
                        }
                        break;
                    }
                }
            }
        }
    }
    if window.focused && keyboard.just_pressed(KeyCode::Escape) {
        if interface.mode == UiMode::Benchmark {
            info!("Benchmark cancelled; no report written.");
            interface.mode = UiMode::Menu;
            exit.write(AppExit::Success);
        } else {
            interface.escape();
        }
    }
    if window.focused
        && interface.mode == UiMode::Loading
        && (keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter))
        && interface.loading_progress >= 0.95
    {
        interface.mode = UiMode::Playing;
    }
    if window.focused
        && (keyboard.just_pressed(KeyCode::F1) || keyboard.just_pressed(KeyCode::Backquote))
        && matches!(
            interface.mode,
            UiMode::Playing | UiMode::Menu | UiMode::Inventory | UiMode::Map
        )
    {
        interface.show_gm_panel = !interface.show_gm_panel;
    }
    if window.focused
        && keyboard.just_pressed(KeyCode::F2)
        && matches!(
            interface.mode,
            UiMode::Playing | UiMode::Menu | UiMode::Inventory | UiMode::Map
        )
    {
        interface.begin_editing();
    }
    if window.focused
        && (keyboard.just_pressed(KeyCode::Tab) || keyboard.just_pressed(KeyCode::KeyI))
        && matches!(
            interface.mode,
            UiMode::Playing | UiMode::Inventory | UiMode::Map
        )
    {
        interface.toggle_inventory();
    }
    if window.focused
        && keyboard.just_pressed(KeyCode::KeyM)
        && matches!(
            interface.mode,
            UiMode::Playing | UiMode::Inventory | UiMode::Map
        )
    {
        interface.toggle_map();
    }
    if !window.focused
        && matches!(
            interface.mode,
            UiMode::Playing | UiMode::Inventory | UiMode::Map
        )
    {
        interface.mode = UiMode::Menu;
    }

    for (slot, key) in [
        (0, KeyCode::Digit1),
        (1, KeyCode::Digit2),
        (2, KeyCode::Digit3),
        (3, KeyCode::Digit4),
        (4, KeyCode::Digit5),
        (5, KeyCode::Digit6),
    ] {
        if window.focused
            && keyboard.just_pressed(key)
            && matches!(interface.mode, UiMode::Playing | UiMode::Inventory)
        {
            select_hotbar_slot(slot, &mode, &mut survival, &online);
        }
    }

    let ui_wants_input = contexts
        .ctx_mut()
        .is_ok_and(|ctx| ctx.egui_wants_keyboard_input() || ctx.egui_wants_pointer_input());
    let playing = window.focused
        && interface.mode == UiMode::Playing
        && !ui_wants_input
        && !interface.show_gm_panel;
    let grab_mode = if playing {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if cursor.grab_mode != grab_mode {
        cursor.grab_mode = grab_mode;
    }
    if cursor.visible == playing {
        cursor.visible = !playing;
    }
    if !playing {
        input.0.clear();
        return;
    }

    if scroll.delta.y.abs() > 0.01 {
        let current = survival
            .0
            .player(PlayerId(0))
            .map(|p| p.inventory.active_slot as i32)
            .unwrap_or(0);
        let step = if scroll.delta.y > 0.0 { -1 } else { 1 };
        let next = (current + step).rem_euclid(HOTBAR_SLOTS as i32) as u8;
        select_hotbar_slot(next, &mode, &mut survival, &online);
    }

    let is_teleport_gun = survival
        .0
        .player(PlayerId(0))
        .and_then(|p| p.inventory.active_item())
        .is_some_and(|s| s.kind == shared::survival::ItemKind::TeleportGun);

    let lmb_pressed = mouse_buttons.just_pressed(MouseButton::Left);
    let rmb_pressed = mouse_buttons.just_pressed(MouseButton::Right);
    let key_e_pressed = keyboard.just_pressed(KeyCode::KeyE);

    if is_teleport_gun && (lmb_pressed || rmb_pressed || key_e_pressed) {
        if let Ok((mut pose, mut motion, mut transform)) = local_player.single_mut() {
            let mut deleted_object = false;
            // On LMB, attempt to delete looked object first
            if lmb_pressed && !rmb_pressed {
                if let Some(ref looked) = telemetry.looked_object {
                    if !matches!(
                        looked.target_kind,
                        Some(LookedTargetKind::Terrain(_)) | None
                    ) {
                        if let Some(msg) = delete_looked_object(
                            looked,
                            &mut survival,
                            &mut physics,
                            Some(&mut commands),
                        ) {
                            if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                                player.notice = Some(msg);
                            }
                            deleted_object = true;
                        }
                    }
                    if !deleted_object {
                        for (entity, marker) in &scenic_entities {
                            let d =
                                (marker.position - Vec3::from_array(looked.hit_coords)).length();
                            if d < 3.5 {
                                commands.entity(entity).despawn();
                                if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                                    player.notice =
                                        Some("🗑️ Декоративний об'єкт видалено!".to_string());
                                }
                                deleted_object = true;
                                break;
                            }
                        }
                    }
                }
            }

            if deleted_object {
                interface.swing_timer = 0.25;
                interface.recoil_timer = 0.18;
            } else {
                // Teleport to target or forward
                let dir = shared::survival::look_direction(motion.0.yaw, motion.0.pitch);
                let eye = [pose.current.x, pose.current.y + 0.6, pose.current.z];
                let hit_target = physics.0.cast_ray(eye, dir, 800.0, Some(PlayerId(0)));
                let dest = if let Some((hit, norm, _)) = hit_target {
                    [
                        hit[0] + norm[0] * 0.15,
                        hit[1] + 0.85,
                        hit[2] + norm[2] * 0.15,
                    ]
                } else if dir[1] < -0.02 {
                    let t = ((-22.0) - eye[1]) / dir[1];
                    let t = t.clamp(1.0, 800.0);
                    [eye[0] + dir[0] * t, -21.6, eye[2] + dir[2] * t]
                } else {
                    [
                        eye[0] + dir[0] * 40.0,
                        eye[1] + dir[1] * 40.0,
                        eye[2] + dir[2] * 40.0,
                    ]
                };
                let dest_vec = Vec3::from_array(dest);
                pose.current = dest_vec;
                pose.previous = dest_vec;
                transform.translation = dest_vec;
                motion.0.vertical_velocity = 0.0;
                physics.0.set_player(
                    PlayerId(0),
                    shared::protocol::PlayerState {
                        position: dest,
                        motion: motion.0,
                        velocity: [0.0, 0.0, 0.0],
                    },
                );
                interface.last_shot_trace = Some((eye, dest, 0.35));
                interface.swing_timer = 0.25;
                interface.recoil_timer = 0.18;
                if let Some(player) = survival.0.player_mut(PlayerId(0)) {
                    player.notice = Some(format!(
                        "Teleported to [{:.1}, {:.1}, {:.1}]",
                        dest[0], dest[1], dest[2]
                    ));
                }
            }
        }
    } else if (lmb_pressed || key_e_pressed)
        && let Ok((pose, motion, _)) = local_player.single()
    {
        let cur_pos = pose.current.to_array();
        let cur_yaw = motion.0.yaw;
        let cur_pitch = motion.0.pitch;
        trigger_primary_action(
            &mut interface,
            &mode,
            &clock,
            &mut physics,
            &mut survival,
            &online,
            cur_pos,
            cur_yaw,
            cur_pitch,
            &mut nodes,
        );
    }
    if (!is_teleport_gun && mouse_buttons.just_pressed(MouseButton::Right))
        || keyboard.just_pressed(KeyCode::KeyQ)
    {
        rotate_structure_socket(&mode, &mut survival, &online);
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        cycle_structure_kind(&mode, &mut survival, &online);
    }
    if keyboard.just_pressed(KeyCode::KeyF)
        && let Ok((pose, motion, _)) = local_player.single()
    {
        toggle_mount_vehicle(
            &mode,
            &mut survival,
            &online,
            pose.current.to_array(),
            motion.0.yaw,
            motion.0.pitch,
        );
    }

    let axis = |positive, negative| {
        let positive = if keyboard.pressed(positive) { 1.0 } else { 0.0 };
        let negative = if keyboard.pressed(negative) { 1.0 } else { 0.0 };
        positive - negative
    };
    let move_vec = [
        axis(KeyCode::KeyD, KeyCode::KeyA),
        axis(KeyCode::KeyW, KeyCode::KeyS),
    ];
    let is_mounted = survival
        .0
        .player(PlayerId(0))
        .is_some_and(|p| p.mounted_vehicle.is_some());
    input.0.set_movement(
        move_vec,
        is_mounted || keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight),
    );
    input.0.add_look([
        -mouse.delta.x * MOUSE_SENSITIVITY,
        -mouse.delta.y * MOUSE_SENSITIVITY,
    ]);
    if keyboard.just_pressed(KeyCode::Space) {
        input.0.queue_jump();
    }

    if interface.gm_fly_mode
        && let Ok((mut pose, mut motion, mut transform)) = local_player.single_mut()
    {
        motion.0.grounded = true;
        motion.0.vertical_velocity = 0.0;
        let speed = if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)
        {
            0.85
        } else {
            0.38
        };
        if keyboard.pressed(KeyCode::Space) {
            pose.current.y += speed;
            pose.previous.y = pose.current.y;
            transform.translation.y = pose.current.y;
        } else if keyboard.pressed(KeyCode::KeyC) || keyboard.pressed(KeyCode::ControlLeft) {
            pose.current.y = (pose.current.y - speed).max(0.5);
            pose.previous.y = pose.current.y;
            transform.translation.y = pose.current.y;
        }
    }
}
