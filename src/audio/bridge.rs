use rapier3d::geometry::CollisionEvent;

use crate::commander::CommandBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialType {
    Rubber,
    Concrete,
    Metal,
    Wood,
    Dirt,
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicsMaterial {
    pub material: MaterialType,
    pub noise_threshold: f32,
}

#[derive(Debug, Clone)]
pub enum AudioCommand {
    PlaySpatialEvent {
        sound_id: String,
        position: [f32; 3],
        volume: f32,
    },
}

/// Bridge: physics observes collisions and posts audio commands into a
/// `CommandBuffer<AudioCommand>` so the audio system can drain them later.
///
/// Callers feed direct `&mut CommandBuffer<AudioCommand>` references; no channel
/// shim is retained. This keeps producers from holding consumer-side handles
/// while preserving the decoupled contract.
pub struct PhysicsAudioBridge<'a> {
    buffer: &'a mut CommandBuffer<AudioCommand>,
}

impl<'a> PhysicsAudioBridge<'a> {
    pub fn new(buffer: &'a mut CommandBuffer<AudioCommand>) -> Self {
        Self { buffer }
    }

    pub fn process_collision_events(
        &mut self,
        events: &[CollisionEvent],
        get_material: impl Fn(u32) -> Option<PhysicsMaterial>,
        get_position: impl Fn(u32) -> Option<[f32; 3]>,
        impulse_magnitude: f32,
    ) {
        for event in events {
            if let CollisionEvent::Started(handle1, handle2, _flags) = event {
                let mat1 = get_material(handle1.into_raw_parts().0);
                let mat2 = get_material(handle2.into_raw_parts().0);

                if let Some(material_info) = mat1.or(mat2) {
                    if impulse_magnitude > material_info.noise_threshold {
                        let position =
                            get_position(handle1.into_raw_parts().0)
                                .unwrap_or([0.0, 0.0, 0.0]);

                        let sound_id = match material_info.material {
                            MaterialType::Rubber => "tire_screech",
                            MaterialType::Concrete => "concrete_impact",
                            MaterialType::Metal => "metal_clank",
                            MaterialType::Wood => "wood_crack",
                            MaterialType::Dirt => "dirt_kickup",
                        };

                        let max_impulse = 50.0;
                        let volume = (impulse_magnitude / max_impulse).min(1.0);

                        let _ = self.buffer.push(super::AudioCommand::PlaySpatialEvent {
                            sound_id: sound_id.to_string(),
                            position,
                            volume,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commander::CommandBuffer;

    #[test]
    fn test_bridge_triggering_thresholds() {
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

