//! Versioned, transport-independent messages. No client-supplied positions.
use serde::{Deserialize, Serialize};

use crate::{
    input::{PlayerInputCommand, normalized_axes},
    level::MapId,
    movement::{MAX_PITCH, MotionState},
    survival::{
        MonumentPuzzleState, NpcEntity, NuclearStrikeState, PlacedStructure, PlayerSurvival,
        RecipeId, ResourceNode, SlotIndex, VehicleEntity,
    },
};

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_PLAYERS: usize = 150;
pub const INPUT_REDUNDANCY: usize = 6;
pub const INPUT_WINDOW: u64 = 60;
pub const HISTORY_TICKS: u64 = 120;
pub const INPUT_HOLD_TICKS: u64 = 15;
pub const SNAPSHOT_INTERVAL: u64 = 3;
pub const MAX_CONTROL_BYTES: usize = 16 * 1024;
pub const MAX_DATAGRAM_BYTES: usize = 1200;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct PlayerId(pub u64);

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ServerTick(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlayerState {
    pub position: [f32; 3],
    pub motion: MotionState,
    pub velocity: [f32; 3],
}

impl PlayerState {
    pub fn at(position: [f32; 3]) -> Self {
        Self {
            position,
            motion: MotionState::default(),
            velocity: [0.0; 3],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InputFrame {
    pub epoch: u32,
    pub tick: ServerTick,
    pub movement: [f32; 2],
    pub yaw: f32,
    pub pitch: f32,
    pub jump: bool,
    pub sprint: bool,
}

impl InputFrame {
    pub fn neutral(epoch: u32, tick: ServerTick, motion: MotionState) -> Self {
        Self {
            epoch,
            tick,
            movement: [0.0; 2],
            yaw: motion.yaw,
            pitch: motion.pitch,
            jump: false,
            sprint: false,
        }
    }

    pub fn validate(mut self) -> Result<Self, &'static str> {
        if !self
            .movement
            .iter()
            .chain([&self.yaw, &self.pitch])
            .all(|v| v.is_finite())
        {
            return Err("non-finite input");
        }
        self.movement = normalized_axes(self.movement);
        self.yaw = self.yaw.rem_euclid(std::f32::consts::TAU);
        self.pitch = self.pitch.clamp(-MAX_PITCH, MAX_PITCH);
        Ok(self)
    }

    pub fn command(self, motion: MotionState) -> PlayerInputCommand {
        PlayerInputCommand {
            tick_id: self.tick.0,
            move_vec: self.movement,
            look_delta: [self.yaw - motion.yaw, self.pitch - motion.pitch],
            jump: self.jump,
            sprint: self.sprint,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Hello {
    pub protocol: u32,
    pub map_revision: u32,
    pub token: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientControl {
    Resync,
    Ping(u64),
    PrimaryAction { yaw: f32, pitch: f32 },
    SelectSlot(u8),
    MoveSlot { from: SlotIndex, to: SlotIndex },
    Craft(RecipeId),
    CycleStructure,
    RotateStructure,
    MountOrDismount { yaw: f32, pitch: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerControl {
    Welcome {
        player: PlayerId,
        tick: ServerTick,
        epoch: u32,
        map: MapId,
        state: PlayerState,
        capacity: u16,
    },
    Enter {
        player: PlayerId,
        tick: ServerTick,
        state: PlayerState,
    },
    Leave {
        player: PlayerId,
        tick: ServerTick,
    },
    Reset {
        tick: ServerTick,
        epoch: u32,
        state: PlayerState,
    },
    Pong {
        nonce: u64,
        tick: ServerTick,
    },
    SurvivalState(PlayerSurvival),
    ResourceNodes(Vec<ResourceNode>),
    Structures(Vec<PlacedStructure>),
    Monument(MonumentPuzzleState),
    WorldDynamicState {
        npcs: Vec<NpcEntity>,
        vehicles: Vec<VehicleEntity>,
        nuclear: NuclearStrikeState,
    },
    Rejected(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputBatch {
    #[serde(deserialize_with = "bounded_inputs")]
    pub frames: Vec<InputFrame>,
}

fn bounded_inputs<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<InputFrame>, D::Error> {
    struct Frames;
    impl<'de> serde::de::Visitor<'de> for Frames {
        type Value = Vec<InputFrame>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("at most six input frames")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            if sequence.size_hint().is_some_and(|n| n > INPUT_REDUNDANCY) {
                return Err(serde::de::Error::custom("too many input frames"));
            }
            let mut frames = Vec::with_capacity(INPUT_REDUNDANCY);
            while let Some(frame) = sequence.next_element()? {
                if frames.len() == INPUT_REDUNDANCY {
                    return Err(serde::de::Error::custom("too many input frames"));
                }
                frames.push(frame);
            }
            Ok(frames)
        }
    }
    deserializer.deserialize_seq(Frames)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OwnerState {
    pub epoch: u32,
    /// Finalized tick, including ticks simulated with substituted input.
    pub acknowledged: ServerTick,
    pub state: PlayerState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReplicatedPlayer {
    pub player: PlayerId,
    #[serde(with = "compact_remote")]
    pub state: PlayerState,
}

/// Remote presentation/proxy states use centimetres and bounded angles. The
/// owner's reconciliation state remains full precision. Postcard varints keep
/// the dense 150-player case practical without a loss-sensitive delta baseline.
mod compact_remote {
    use super::*;
    #[derive(Serialize, Deserialize)]
    struct Packed {
        position: [i32; 3],
        velocity: [i16; 3],
        look: [i16; 2],
        grounded: bool,
    }

    pub fn serialize<S: serde::Serializer>(
        state: &PlayerState,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        Packed {
            position: state.position.map(|v| (v * 100.0).round() as i32),
            velocity: state
                .velocity
                .map(|v| (v * 100.0).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16),
            look: [
                (state.motion.yaw.rem_euclid(std::f32::consts::TAU) * 5000.0).round() as i16,
                (state.motion.pitch * 5000.0).round() as i16,
            ],
            grounded: state.motion.grounded,
        }
        .serialize(serializer)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<PlayerState, D::Error> {
        let packed = Packed::deserialize(deserializer)?;
        let velocity = packed.velocity.map(|v| v as f32 / 100.0);
        Ok(PlayerState {
            position: packed.position.map(|v| v as f32 / 100.0),
            velocity,
            motion: MotionState {
                yaw: packed.look[0] as f32 / 5000.0,
                pitch: packed.look[1] as f32 / 5000.0,
                vertical_velocity: velocity[1],
                grounded: packed.grounded,
            },
        })
    }
}

/// Independent pages: loss of one datagram never blocks another page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: ServerTick,
    pub population: u16,
    pub owner: Option<OwnerState>,
    pub players: Vec<ReplicatedPlayer>,
}
