//! Client presentation and platform-independent HUD state.
//! The runner provides storage and startup; simulation lives in `shared`.

pub mod application;
pub mod desktop_input;
pub mod editor;
pub mod environment;
pub mod hud;
mod hud_painter;
mod interface;
pub mod launch;
mod map_texture;
pub mod metrics;
mod online;
mod online_metrics;
pub mod source_minimap;
pub mod source_prefabs;
pub mod source_player;
pub mod source_ui;
pub mod storage;
mod terrain;

pub use application::build_app;
pub use launch::{ClientOptions, LaunchCommand, parse_args};
pub use map_texture::SourceMipImagePlugin;
pub(crate) use shared::level;

pub mod native_ui;
pub use native_ui::{NativeUiPlugin, NativeUiSet};
