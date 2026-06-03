pub mod emitter;
pub mod bridge;

use std::collections::HashMap;
use kira::{
    manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
};
pub use crate::commander::CommandBuffer;

pub use emitter::{SpatialAudioEmitter, SpatialAudioListener};
pub use bridge::{PhysicsAudioBridge, AudioCommand, PhysicsMaterial, MaterialType};

pub struct AudioSystem {
    command_buffer: CommandBuffer<AudioCommand>,
    manager: AudioManager<CpalBackend>,
    sound_library: HashMap<String, StaticSoundData>,
    active_sounds: Vec<StaticSoundHandle>,
}

impl AudioSystem {
    pub fn new() -> Self {
        let manager =
            AudioManager::<CpalBackend>::new(AudioManagerSettings::default())
                .expect("Failed to initialize CPAL/Kira Audio Backend");

        Self {
            command_buffer: CommandBuffer::with_capacity(64),
            manager,
            sound_library: HashMap::new(),
            active_sounds: Vec::with_capacity(64),
        }
    }

    pub fn command_buffer(&mut self) -> &mut CommandBuffer<AudioCommand> {
        &mut self.command_buffer
    }

    pub fn load_sound_asset(&mut self, id: &str, path: &str) {
        if let Ok(sound_data) = StaticSoundData::from_file(path, StaticSoundSettings::default()) {
            self.sound_library.insert(id.to_string(), sound_data);
        }
    }

    pub fn update(&mut self, listener: &SpatialAudioListener) {
        for command in self.command_buffer.drain() {
            match command.payload {
                AudioCommand::PlaySpatialEvent {
                    sound_id,
                    position,
                    volume,
                } => {
                    if let Some(sound_data) = self.sound_library.get(&sound_id) {
                        let emitter = SpatialAudioEmitter::new(&sound_id, position);
                        let (dist_vol, panning) =
                            emitter.calculate_spatial_properties(listener);

                        let mut dynamic_sound = sound_data.clone();
                        dynamic_sound.settings.volume =
                            kira::tween::Value::Fixed(kira::Volume::Amplitude(
                                (volume * dist_vol) as f64,
                            ));
                        dynamic_sound.settings.panning =
                            kira::tween::Value::Fixed(panning as f64);

                        if let Ok(handle) = self.manager.play(dynamic_sound) {
                            self.active_sounds.push(handle);
                        }
                    }
                }
            }
        }

        self.active_sounds
            .retain(|handle| handle.state() != kira::sound::PlaybackState::Stopped);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commander::CommandBuffer;

    #[test]
    fn test_spatial_panning_calculations() {
        let listener = SpatialAudioListener::new([0.0, 0.0, 0.0]);
        let emitter = SpatialAudioEmitter::new("impact", [10.0, 0.0, 0.0]);
        let (volume, panning) = emitter.calculate_spatial_properties(&listener);

        assert!(volume < 1.0);
        assert!(panning > 0.0);
    }

    #[test]
    fn test_bridge_triggering_thresholds() {
        use rapier3d::geometry::CollisionEvent;

        let mut buffer = CommandBuffer::with_capacity(4);

        let mock_material = PhysicsMaterial {
            material: MaterialType::Metal,
            noise_threshold: 10.0,
        };

        let mock_events = vec![CollisionEvent::Started(
            rapier3d::geometry::ColliderHandle::from_raw_parts(1, 1),
            rapier3d::geometry::ColliderHandle::from_raw_parts(2, 1),
            rapier3d::geometry::CollisionEventFlags::empty(),
        )];

        {
            let mut bridge = PhysicsAudioBridge::new(&mut buffer);
            bridge.process_collision_events(
                &mock_events,
                |_| Some(mock_material),
                |_| Some([0.0, 0.0, 0.0]),
                5.0,
            );
        }
        assert!(buffer.is_empty());

        {
            let mut bridge = PhysicsAudioBridge::new(&mut buffer);
            bridge.process_collision_events(
                &mock_events,
                |_| Some(mock_material),
                |_| Some([0.0, 0.0, 0.0]),
                15.0,
            );
        }

        let drained: Vec<_> = buffer.drain().collect();
        assert_eq!(drained.len(), 1);
        match drained[0].payload {
            AudioCommand::PlaySpatialEvent { ref sound_id, .. } => {
                assert_eq!(sound_id, "metal_clank");
            }
        }
    }
}
