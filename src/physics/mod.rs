pub mod component;
pub mod state;

use std::time::Instant;
pub use component::{Transform, RigidBodyRef};
pub use state::{PhysicsState, PhysicsEvent, PhysicsEventPayload};
use crate::commander::{CommandBuffer, GameplayCommand};

/// The central controller coordinating the decoupled simulation clock rates.
pub struct SimulationLoop {
    last_frame_time: Instant,
    accumulator: f32,
    physics_tick_rate: f32, // e.g. 0.00833 for 120Hz, or 0.0025 for 400Hz
}

impl SimulationLoop {
    /// Create a new simulation loop coordinator with a target tick frequency.
    pub fn new(target_hz: f32) -> Self {
        Self {
            last_frame_time: Instant::now(),
            accumulator: 0.0,
            physics_tick_rate: 1.0 / target_hz,
        }
    }

    /// Accumulate the elapsed system time since the last update frame.
    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Clamp extreme lag spikes to prevent "spiral of death" lockups
        let clamped_elapsed = elapsed.min(0.25);
        self.accumulator += clamped_elapsed;

        clamped_elapsed
    }

    /// Check if the simulation loop has accumulated enough time to step another physics update tick.
    /// Consume a slice of tick time if true.
    pub fn tick_accumulated(&mut self) -> bool {
        if self.accumulator >= self.physics_tick_rate {
            self.accumulator -= self.physics_tick_rate;
            true
        } else {
            false
        }
    }

    /// Get the remaining fractional time step (alpha) used to interpolate visuals.
    pub fn interpolation_alpha(&self) -> f32 {
        self.accumulator / self.physics_tick_rate
    }

    /// Get the exact delta-time configuration representing the simulation tick.
    pub fn tick_rate(&self) -> f32 {
        self.physics_tick_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_loop_accumulation() {
        let mut sim_loop = SimulationLoop::new(120.0); // 120Hz (8.33ms ticks)
        assert_eq!(sim_loop.tick_rate(), 1.0 / 120.0);

        // Simulate 20ms of time passing instantly
        sim_loop.accumulator += 0.020;

        // Expect exactly 2 ticks to trigger
        assert!(sim_loop.tick_accumulated()); // Tick 1 (retains 11.66ms)
        assert!(sim_loop.tick_accumulated()); // Tick 2 (retains 3.33ms)
        assert!(!sim_loop.tick_accumulated()); // No third tick

        // Verify remaining fraction (alpha) calculation
        let alpha = sim_loop.interpolation_alpha();
        assert!(alpha > 0.0 && alpha < 1.0);
    }

    #[test]
    fn test_lerp_slerp_interpolation() {
        let mut transform = Transform::new([0.0, 0.0, 0.0]);
        transform.current_position = [10.0, 20.0, 30.0];

        // 50% interpolation factor
        let half_position = transform.interpolate_position(0.5);
        assert_eq!(half_position, [5.0, 10.0, 15.0]);
    }
}