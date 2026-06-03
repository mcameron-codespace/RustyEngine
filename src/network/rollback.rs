use super::protocol::PlayerInputs;

/// Represents a historical frame prediction snapshot stored on the local client
#[derive(Debug, Clone, Copy)]
pub struct PredictedFrame {
    pub tick: u32,
    pub inputs: PlayerInputs,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
}

/// Pre-allocated circular ring buffer holding player input and state history
pub struct PredictionBuffer {
    frames: Vec<Option<PredictedFrame>>,
    capacity: usize,
}

impl PredictionBuffer {
    /// Initialize a fixed circular ring buffer (zero mid-loop allocations)
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            frames: vec![None; capacity],
            capacity,
        }
    }

    /// Store a predicted frame at a specific tick index
    pub fn insert(&mut self, tick: u32, inputs: PlayerInputs, position: [f32; 3], velocity: [f32; 3]) {
        let index = (tick as usize) % self.capacity;
        self.frames[index] = Some(PredictedFrame {
            tick,
            inputs,
            position,
            velocity,
        });
    }

    /// Retrieve a recorded prediction frame for the target tick index.
    /// The buffer stores snapshots in a circular ring, so the lookup key is the
    /// modulo-wrapped slot used during insertion.
    pub fn get(&self, tick: u32) -> Option<PredictedFrame> {
        let index = (tick as usize) % self.capacity;
        self.frames[index]
    }

    /// Compare client-side history against authoritative server coordinates
    /// Returns true if a correction rollback is required (error threshold breached)
    pub fn check_desync(
        &self,
        tick: u32,
        server_position: [f32; 3],
        threshold: f32,
    ) -> Result<bool, &'static str> {
        let predicted = self.get(tick).ok_or("Historical tick not found in prediction buffer")?;
        
        let dx = predicted.position[0] - server_position[0];
        let dy = predicted.position[1] - server_position[1];
        let dz = predicted.position[2] - server_position[2];
        let dist_sq = dx*dx + dy*dy + dz*dz;

        Ok(dist_sq > threshold * threshold)
    }
}