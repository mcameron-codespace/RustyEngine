use rapier3d::prelude::*;
use crate::audio::bridge::AudioCommand;
use crate::commander::CommandBuffer;

/// The central Rapier3D physics state wrapper.
/// Owns the entire raw 3D physics pipeline, managing colliders, rigid bodies, and joints.
pub struct PhysicsState {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub gravity: Vector<f32>,
    pub audio_buffer: CommandBuffer<AudioCommand>,
}

impl PhysicsState {
    /// Validate a fixed-tick delta-time before stepping the simulator.
    pub fn validate_dt(dt: f32) -> Result<f32, &'static str> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err("Physics dt must be positive and finite");
        }
        if dt > 1.0 {
            return Err("Physics dt must not exceed 1.0 seconds");
        }
        Ok(dt)
    }

    /// Initialize a standard physical sandbox with gravity.
    pub fn new(gravity_y: f32) -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            gravity: vector![0.0, gravity_y, 0.0],
            audio_buffer: CommandBuffer::with_capacity(64),
        }
    }

    /// Post an audio event produced by the physics step.
    pub fn post_audio_command(&mut self, payload: AudioCommand) {
        let _ = self.audio_buffer.push(payload);
    }

    pub fn drain_audio_commands(&mut self) -> impl Iterator<Item = AudioCommand> + '_ {
        self.audio_buffer.drain().map(|cmd| cmd.payload)
    }

    pub fn step(&mut self, dt: f32) -> Result<(), &'static str> {
        let dt = Self::validate_dt(dt)?;
        // Set the custom delta time step parameter
        self.integration_parameters.dt = dt;

        // Execute the pipeline step
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );

        Ok(())
    }

    /// Single-step helper retained for internal/unsafe fallback paths.
    #[deprecated = "Use step() which validates dt; calls step() internally."]
    pub fn step_unchecked(&mut self, dt: f32) {
        let _ = self.step(dt);
    }
}
