//! UI-independent sound and minimap presentation state.
use shared::{
    protocol::ServerTick,
    survival::{NpcEntity, PlacedStructure, ResourceNode, VehicleEntity},
};
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

pub struct MinimapContext<'a> {
    pub player_position: [f32; 3],
    pub yaw: f32,
    pub nodes: &'a [ResourceNode],
    pub structures: &'a [PlacedStructure],
    pub npcs: &'a [NpcEntity],
    pub vehicles: &'a [VehicleEntity],
    pub tick: ServerTick,
    pub marker: Option<[f32; 2]>,
    pub sound_cues: &'a [VisualSoundCue],
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
            tick: ServerTick(0),
            marker: None,
            sound_cues: &[],
        }
    }
}
