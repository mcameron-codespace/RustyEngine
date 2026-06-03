use crate::gameplay::command::GameplayCommand;
use crate::renderer::command::RenderCommand;

use crate::commander::CommandBuffer;

/// A central routing surface for typed command buffers.
///
/// Each producer writes into its own typed ring buffer without holding a
/// reference to the consumer, allowing the renderer and gameplay loops to
/// drain commands at frame/tick boundaries.
pub struct CommandDispatcher {
    pub audio: CommandBuffer<crate::audio::bridge::AudioCommand>,
    pub render: CommandBuffer<RenderCommand>,
    pub gameplay: CommandBuffer<GameplayCommand>,
}

impl CommandDispatcher {
    pub fn new(
        audio_capacity: usize,
        render_capacity: usize,
        gameplay_capacity: usize,
    ) -> Self {
        Self {
            audio: CommandBuffer::with_capacity(audio_capacity),
            render: CommandBuffer::with_capacity(render_capacity),
            gameplay: CommandBuffer::with_capacity(gameplay_capacity),
        }
    }

    pub fn drain_render_commands(
        &mut self,
    ) -> impl Iterator<Item = super::buffer::Command<RenderCommand>> + '_ {
        self.render.drain()
    }

    pub fn drain_gameplay_commands(
        &mut self,
    ) -> impl Iterator<Item = super::buffer::Command<GameplayCommand>> + '_ {
        self.gameplay.drain()
    }

    pub fn drain_audio_commands(
        &mut self,
    ) -> impl Iterator<Item = super::buffer::Command<crate::audio::bridge::AudioCommand>> + '_ {
        self.audio.drain()
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::Entity;
    use crate::physics::Transform;

    use super::*;

    #[test]
    fn drain_render_commands_processes_fifo() {
        let mut dispatcher = CommandDispatcher::new(4, 4, 4);
        let entity = Entity {
            id: 1,
            generation: 0,
        };
        let transform = Transform::new([0.0, 1.0, 2.0]);

        assert!(dispatcher.render.push(RenderCommand::Clear).is_ok());
        assert!(
            dispatcher
                .render
                .push(RenderCommand::UpdateTransform { entity, transform })
                .is_ok()
        );

        let drained: Vec<_> = dispatcher.drain_render_commands().collect();
        assert_eq!(drained.len(), 2);
        assert!(matches!(drained[0].payload, RenderCommand::Clear));
        assert!(matches!(
            drained[1].payload,
            RenderCommand::UpdateTransform { .. }
        ));
    }

    #[test]
    fn drain_gameplay_commands_processes_fifo() {
        let mut dispatcher = CommandDispatcher::new(4, 4, 4);

        assert!(dispatcher.gameplay.push(GameplayCommand::Pause).is_ok());
        assert!(dispatcher.gameplay.push(GameplayCommand::Resume).is_ok());

        let drained: Vec<_> = dispatcher.drain_gameplay_commands().collect();
        assert_eq!(drained.len(), 2);
        assert!(matches!(drained[0].payload, GameplayCommand::Pause));
        assert!(matches!(drained[1].payload, GameplayCommand::Resume));
    }
}
