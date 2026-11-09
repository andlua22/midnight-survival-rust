//! Client presentation and platform-independent HUD state.
//! The runner provides storage and startup; simulation lives in `shared`.

pub mod application;
pub mod desktop_input;
pub mod editor;
pub mod hud;
mod hud_painter;
mod interface;
pub mod launch;
mod map_texture;
pub mod metrics;
mod online;
mod online_metrics;
pub mod storage;
mod terrain;

pub use application::build_app;
pub use launch::{ClientOptions, LaunchCommand, parse_args};
pub use map_texture::SourceMipImagePlugin;

pub(crate) use bevy_egui::egui;
