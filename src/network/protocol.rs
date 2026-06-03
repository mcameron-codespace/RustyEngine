use serde::{Deserialize, Serialize};

/// Channel ID configurations for Renet transport
#[repr(u8)]
pub enum NetworkChannel {
    /// Reliable, ordered channel for structural events (e.g., chat, spawning, lobby state)
    ReliableOrdered = 0,
    /// Unreliable, ordered channel for high-frequency synchronization (e.g., positions, telemetry)
    UnreliableOrdered = 1,
    /// Reliable, unordered channel for quick event notifications (e.g., dynamic audio trigger requests)
    ReliableUnordered = 2,
}

impl NetworkChannel {
    pub const ALL_CHANNELS: [Self; 3] = [
        Self::ReliableOrdered,
        Self::UnreliableOrdered,
        Self::ReliableUnordered,
    ];
}

/// Structured keyboard, mouse, and telemetry state representing player commands
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlayerInputs {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub yaw: f32,
    pub pitch: f32,
}

impl PlayerInputs {
    pub fn empty() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

/// Base serialization payload sent from Client to Server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    InputTick {
        tick: u32,
        sequence: u32,
        inputs: PlayerInputs,
    },
    Chat {
        message: String,
    },
    Heartbeat {
        tick: u32,
    },
}

/// Base serialization payload sent from Authoritative Server to Clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    StateSnapshot {
        tick: u32,
        entity_id: u32,
        position: [f32; 3],
        velocity: [f32; 3],
    },
    StateDelta {
        tick: u32,
        base_tick: u32,
        changed_count: u16,
        entity_ids: Vec<u32>,
        positions: Vec<[f32; 3]>,
        velocities: Vec<[f32; 3]>,
    },
    SpawnEntity {
        entity_id: u32,
        position: [f32; 3],
    },
    Heartbeat {
        tick: u32,
    },
}