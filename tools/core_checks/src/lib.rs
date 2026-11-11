//! Fast checks of the exact production modules without a GPU/Bevy dependency.
//! Rapier integration and rendering are checked by the main workspace instead.

#[path = "../../../crates/shared/src/content.rs"]
pub mod content;

#[path = "../../../crates/client_core/src/editor.rs"]
pub mod editor;
#[path = "../../../crates/client_core/src/hud.rs"]
pub mod hud;
#[path = "../../../crates/shared/src/input.rs"]
pub mod input;
#[path = "../../../crates/client_core/src/launch.rs"]
pub mod launch;
#[path = "../../../crates/shared/src/level.rs"]
pub mod level;
#[path = "../../../crates/shared/src/map_projection.rs"]
pub mod map_projection;
#[path = "../../../crates/client_core/src/metrics.rs"]
pub mod metrics;
#[path = "../../../crates/shared/src/movement.rs"]
pub mod movement;
#[path = "../../../crates/shared/src/protocol.rs"]
pub mod protocol;
#[path = "../../../crates/client_core/src/storage.rs"]
pub mod storage;
#[path = "../../../crates/shared/src/survival.rs"]
pub mod survival;
