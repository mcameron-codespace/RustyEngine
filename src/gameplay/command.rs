/// Gameplay commands issued by input/network and drained by the gameplay loop.
#[derive(Debug, Clone, Copy)]
pub enum GameplayCommand {
    SpawnVehicle {
        position: [f32; 3],
    },
    Pause,
    Resume,
    SetCheckpoint {
        index: u32,
    },
}
