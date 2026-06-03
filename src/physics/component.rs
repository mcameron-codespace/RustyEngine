/// Transform component managing dual-state positions and rotations.
/// This structure holds both the `current` physics frame state and the `previous` 
/// physics frame state, allowing the renderer to calculate visual interpolation (LERP/SLERP).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub previous_position: [f32; 3],
    pub current_position: [f32; 3],
    
    /// Stored as unit quaternions [x, y, z, w]
    pub previous_rotation: [f32; 4],
    pub current_rotation: [f32; 4],
}

impl Transform {
    /// Create a static transform component initialized at a specific position.
    pub fn new(position: [f32; 3]) -> Self {
        Self {
            previous_position: position,
            current_position: position,
            previous_rotation: [0.0, 0.0, 0.0, 1.0],
            current_rotation: [0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Update state transitions at the start of a new fixed simulation tick.
    /// Moves current values to previous values to prepare for the next step.
    pub fn step(&mut self) {
        self.previous_position = self.current_position;
        self.previous_rotation = self.current_rotation;
    }

    /// Calculate the interpolated visual position using the accumulation fraction (alpha).
    /// $$P_{render} = (1.0 - \alpha)P_{prev} + \alpha P_{curr}$$
    pub fn interpolate_position(&self, alpha: f32) -> [f32; 3] {
        [
            self.previous_position[0] + alpha * (self.current_position[0] - self.previous_position[0]),
            self.previous_position[1] + alpha * (self.current_position[1] - self.previous_position[1]),
            self.previous_position[2] + alpha * (self.current_position[2] - self.previous_position[2]),
        ]
    }

    /// Calculate the interpolated visual rotation using spherical linear interpolation (SLERP).
    pub fn interpolate_rotation(&self, alpha: f32) -> [f32; 4] {
        let q1 = self.previous_rotation;
        let q2 = self.current_rotation;

        // Compute the dot product between quaternions
        let mut dot = q1[0] * q2[0] + q1[1] * q2[1] + q1[2] * q2[2] + q1[3] * q2[3];

        // If the dot product is negative, slerp won't take the shorter path.
        // We invert one quaternion to fix this.
        let mut q2_adjusted = q2;
        if dot < 0.0 {
            dot = -dot;
            q2_adjusted = [-q2[0], -q2[1], -q2[2], -q2[3]];
        }

        const DOT_THRESHOLD: f32 = 0.9995;
        if dot > DOT_THRESHOLD {
            // If the inputs are too close, use linear interpolation to avoid division by zero
            let mut result = [
                q1[0] + alpha * (q2_adjusted[0] - q1[0]),
                q1[1] + alpha * (q2_adjusted[1] - q1[1]),
                q1[2] + alpha * (q2_adjusted[2] - q1[2]),
                q1[3] + alpha * (q2_adjusted[3] - q1[3]),
            ];
            
            // Normalize result
            let len = (result[0]*result[0] + result[1]*result[1] + result[2]*result[2] + result[3]*result[3]).sqrt();
            result[0] /= len;
            result[1] /= len;
            result[2] /= len;
            result[3] /= len;
            return result;
        }

        // Standard SLERP calculations
        let theta_0 = dot.acos();
        let theta = theta_0 * alpha;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();

        let s0 = (theta_0 - theta).sin() / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;

        [
            s0 * q1[0] + s1 * q2_adjusted[0],
            s0 * q1[1] + s1 * q2_adjusted[1],
            s0 * q1[2] + s1 * q2_adjusted[2],
            s0 * q1[3] + s1 * q2_adjusted[3],
        ]
    }
}

/// Dynamic marker linking our ECS Entity to a Rapier3D RigidBody handle index.
#[derive(Debug, Clone, Copy)]
pub struct RigidBodyRef {
    pub handle_index: u32,
}