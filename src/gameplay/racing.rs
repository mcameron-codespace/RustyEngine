/// Engine-friendly coefficients representing track grip conditions (Pacejka Magic Formula parameters)
#[derive(Debug, Clone, Copy)]
pub struct PacejkaCoefficients {
    pub b_stiffness: f32,
    pub c_shape: f32,
    pub d_peak: f32,
    pub e_curvature: f32,
}

impl PacejkaCoefficients {
    /// Friction parameters for clean asphalt dry tires
    pub fn dry_asphalt() -> Self {
        Self {
            b_stiffness: 10.0,
            c_shape: 1.9,
            d_peak: 1.0,
            e_curvature: 0.97,
        }
    }

    /// Friction parameters for dirt/gravel tracks
    pub fn loose_dirt() -> Self {
        Self {
            b_stiffness: 6.0,
            c_shape: 1.5,
            d_peak: 0.6,
            e_curvature: 0.95,
        }
    }
}

/// Car suspension spring state parameters
#[derive(Debug, Clone, Copy)]
pub struct SuspensionSpring {
    pub rest_length: f32,
    pub spring_constant_k: f32,
    pub damper_constant_c: f32,
}

/// Component modeling lateral and longitudinal wheel telemetry state.
#[derive(Debug, Clone)]
pub struct WheelTelemetry {
    pub local_offset: [f32; 3],
    pub suspension: SuspensionSpring,
    pub tire_grip: PacejkaCoefficients,
    /// Calculated runtime variables
    pub slip_angle: f32,
    pub suspension_compression: f32,
}

impl WheelTelemetry {
    pub fn new(offset: [f32; 3], grip: PacejkaCoefficients) -> Self {
        Self {
            local_offset: offset,
            suspension: SuspensionSpring {
                rest_length: 0.5,
                spring_constant_k: 15000.0, // 15 kN/m stiffness
                damper_constant_c: 1200.0,  // Viscous damping constant
            },
            tire_grip: grip,
            slip_angle: 0.0,
            suspension_compression: 0.0,
        }
    }

    /// Calculate the lateral tire grip force using the Pacejka Magic Formula.
    /// $$F_y = D \sin\left(C \arctan\left(B \alpha - E (B \alpha - \arctan(B \alpha))\right)\right)$$
    pub fn calculate_lateral_force(&mut self, slip_angle_radians: f32, normal_load_newtons: f32) -> f32 {
        self.slip_angle = slip_angle_radians;
        
        let b = self.tire_grip.b_stiffness;
        let c = self.tire_grip.c_shape;
        let d = self.tire_grip.d_peak;
        let e = self.tire_grip.e_curvature;
        let alpha = slip_angle_radians;

        let bx = b * alpha;
        let inner_term = bx - e * (bx - bx.atan());
        let normalized_lateral_force = d * (c * inner_term.atan()).sin();

        // True physical lateral force scales linearly with the normal load on the tire
        normalized_lateral_force * normal_load_newtons
    }

    /// Solve for suspension force using hookes law and damper rates.
    pub fn calculate_suspension_force(&mut self, compression: f32, compression_velocity: f32) -> f32 {
        self.suspension_compression = compression.clamp(0.0, self.suspension.rest_length);
        
        let spring_force = self.suspension_compression * self.suspension.spring_constant_k;
        let damper_force = compression_velocity * self.suspension.damper_constant_c;

        // Return combined vertical restoring force (clamped non-negative)
        (spring_force + damper_force).max(0.0)
    }
}