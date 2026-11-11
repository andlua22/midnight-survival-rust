//! Fixed-step simulation used by both the desktop client and the headless runner.
//! Platform events and presentation never enter the simulation directly.

pub mod authority;
mod capsule_query;
pub mod content;
pub mod input;
pub mod level;
pub mod map_projection;
pub mod movement;
pub mod physics;
pub mod prediction;
pub mod protocol;
pub mod replication;
pub mod simulation;
pub mod survival;
pub mod world;

pub use simulation::{
    CharacterMotion, LevelGeometry, LocalPlayer, Player, PlayerIdentity, ResourceNodeMarker,
    SharedSimulationPlugin, SimulationClock, SimulationInput, SimulationMode, SimulationPose,
    SimulationSet, SimulationSurvival, SimulationWorld, StructureMarker, build_headless_app,
};
