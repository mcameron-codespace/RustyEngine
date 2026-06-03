pub mod terrain;
pub mod racing;
pub mod command;

pub use terrain::{TerrainGenerator, TerrainStreamingManager, ChunkCoords, ChunkMesh};
pub use racing::{WheelTelemetry, PacejkaCoefficients, SuspensionSpring};

use crate::commander::CommandDispatcher;
use crate::gameplay::command::GameplayCommand;

pub struct GameplaySystem {
    pub dispatcher: CommandDispatcher,
}

impl GameplaySystem {
    pub fn new() -> Self {
        Self {
            dispatcher: CommandDispatcher::new(4, 4, 4),
        }
    }

    pub fn update(&mut self) {
        let commands: Vec<_> = self.dispatcher.drain_gameplay_commands().collect();
        for command in commands {
            match command.payload {
                GameplayCommand::SpawnVehicle { position } => {
                    let _ = position;
                }
                GameplayCommand::Pause => {}
                GameplayCommand::Resume => {}
                GameplayCommand::SetCheckpoint { index } => {
                    let _ = index;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_chunk_generation() {
        let generator = TerrainGenerator::new(32.0, 4); // 4x4 quad chunk
        let coords = ChunkCoords { x: 0, z: 0 };
        let mesh = generator.generate_chunk_mesh(coords);

        // Grid (4+1) x (4+1) = 25 vertices
        assert_eq!(mesh.vertices.len(), 25);
        // 4 * 4 * 2 triangles * 3 indices = 96 indices
        assert_eq!(mesh.indices.len(), 96);
    }

    #[test]
    fn test_pacejka_slip_forces() {
        let grip = PacejkaCoefficients::dry_asphalt();
        let mut wheel = WheelTelemetry::new([1.0, -0.4, 1.5], grip);

        // Assume standard 2500N weight load on the tire
        let normal_load = 2500.0;

        // 1. Zero slip angle means zero lateral force
        let force_zero = wheel.calculate_lateral_force(0.0, normal_load);
        assert_eq!(force_zero, 0.0);

        // 2. Compute force with slip angle (slip = 6 degrees / ~0.105 rad)
        let force_slipping = wheel.calculate_lateral_force(0.105, normal_load);
        assert!(force_slipping > 100.0); // Substantial lateral force generated
        assert!(force_slipping < normal_load); // Peak force is bound by friction limits
    }
}