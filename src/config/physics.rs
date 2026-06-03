use std::collections::HashMap;

use super::parser::parse_f32;

/// Configuration parameters for the Rapier3D physics simulator and fixed loop rates.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsConfig {
    /// Target execution rate of the simulation loop (e.g. 120.0 Hz or 400.0 Hz).
    pub tick_rate_hz: f32,
    pub gravity_y: f32,
    pub max_velocity: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            tick_rate_hz: 120.0,
            gravity_y: -9.81,
            max_velocity: 100.0,
        }
    }
}

impl PhysicsConfig {
    /// Approximate fixed-step duration for this tick rate.
    pub fn dt(&self) -> f32 {
        1.0 / self.tick_rate_hz
    }
    pub fn from_ini_data(ini_data: &HashMap<String, HashMap<String, String>>) -> Self {
        let defaults = Self::default();
        let section = ini_data.get("Physics").cloned().unwrap_or_default();

        Self {
            tick_rate_hz: section
                .get("tick_rate_hz")
                .and_then(|value| parse_f32(value))
                .unwrap_or(defaults.tick_rate_hz),
            gravity_y: section
                .get("gravity_y")
                .and_then(|value| parse_f32(value))
                .unwrap_or(defaults.gravity_y),
            max_velocity: section
                .get("max_velocity")
                .and_then(|value| parse_f32(value))
                .unwrap_or(defaults.max_velocity),
        }
    }
}