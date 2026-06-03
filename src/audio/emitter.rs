/// Component representing the player's hearing orientation in 3D space (e.g., camera or driver's head).
#[derive(Debug, Clone, Copy)]
pub struct SpatialAudioListener {
    pub position: [f32; 3],
    pub forward: [f32; 3],
    pub up: [f32; 3],
}

impl SpatialAudioListener {
    /// Create a new listener positioned at the origin facing down the negative Z-axis.
    pub fn new(position: [f32; 3]) -> Self {
        Self {
            position,
            forward: [0.0, 0.0, -1.0],
            up: [0.0, 1.0, 0.0],
        }
    }
}

/// Component representing a localized sound source in the 3D world.
#[derive(Debug, Clone)]
pub struct SpatialAudioEmitter {
    pub position: [f32; 3],
    /// Attenuation model limits
    pub min_distance: f32,
    pub max_distance: f32,
    /// Reference to a registered sound handle
    pub sound_id: String,
}

impl SpatialAudioEmitter {
    pub fn new(sound_id: &str, position: [f32; 3]) -> Self {
        Self {
            position,
            min_distance: 1.0,
            max_distance: 50.0,
            sound_id: sound_id.to_string(),
        }
    }

    /// Calculate attenuation volume based on distance to the listener.
    /// Uses an inverse-distance model:
    /// $$V = \frac{min\_distance}{min\_distance + \text{clamped\_distance}}$$
    pub fn calculate_spatial_properties(&self, listener: &SpatialAudioListener) -> (f32, f32) {
        let dx = self.position[0] - listener.position[0];
        let dy = self.position[1] - listener.position[1];
        let dz = self.position[2] - listener.position[2];
        let distance = (dx*dx + dy*dy + dz*dz).sqrt();

        // Calculate Distance Attenuation (Volume)
        let volume = if distance <= self.min_distance {
            1.0
        } else if distance >= self.max_distance {
            0.0
        } else {
            self.min_distance / (self.min_distance + (distance - self.min_distance))
        };

        // Calculate Stereo Panning
        // Project relative vector onto the listener's right vector (derived via cross-product)
        let right_x = listener.forward[1] * listener.up[2] - listener.forward[2] * listener.up[1];
        let right_y = listener.forward[2] * listener.up[0] - listener.forward[0] * listener.up[2];
        let right_z = listener.forward[0] * listener.up[1] - listener.forward[1] * listener.up[0];
        
        let rel_x = dx;
        let rel_y = dy;
        let rel_z = dz;

        // Dot product to find projection on right vector
        let right_projection = rel_x * right_x + rel_y * right_y + rel_z * right_z;
        let panning = if distance > 0.0 {
            (right_projection / distance).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        (volume, panning)
    }
}