use crate::ecs::Entity;
use crate::physics::Transform;

/// Renderer commands issued by gameplay/physics and drained by the render loop.
#[derive(Debug, Clone)]
pub enum RenderCommand {
    DrawMesh {
        entity: Entity,
    },
    UpdateTransform {
        entity: Entity,
        transform: Transform,
    },
    Clear,
}
