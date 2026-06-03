use crate::ecs::{Entity, World};
use crate::physics::Transform;

/// Minimal mesh data that the renderer can consume from the ECS.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshComponent {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// A renderable entity snapshot that pairs ECS transform data with mesh data.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderableMesh {
    pub entity: Entity,
    pub transform: Transform,
    pub mesh: MeshComponent,
}

/// Collect ECS entities that have both a transform and a mesh component.
pub fn collect_renderables(world: &World) -> Vec<RenderableMesh> {
    let Some(transform_store) = world.try_borrow_store::<Transform>() else {
        return Vec::new();
    };
    let Some(mesh_store) = world.try_borrow_store::<MeshComponent>() else {
        return Vec::new();
    };

    transform_store
        .iter()
        .filter_map(|(entity, transform)| {
            mesh_store
                .get(*entity)
                .cloned()
                .map(|mesh| RenderableMesh {
                    entity: *entity,
                    transform: *transform,
                    mesh,
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;
    use crate::physics::Transform;

    #[test]
    fn collects_only_entities_with_mesh_and_transform() {
        let mut world = World::new();
        world.register_component::<Transform>(8);
        world.register_component::<MeshComponent>(8);

        let entity_with_mesh = world.spawn_entity();
        let entity_without_mesh = world.spawn_entity();

        {
            let mut transform_store = world.borrow_store_mut::<Transform>();
            transform_store.insert(entity_with_mesh, Transform::new([1.0, 2.0, 3.0]));
            transform_store.insert(entity_without_mesh, Transform::new([4.0, 5.0, 6.0]));
        }

        {
            let mut mesh_store = world.borrow_store_mut::<MeshComponent>();
            mesh_store.insert(
                entity_with_mesh,
                MeshComponent {
                    vertices: vec![[0.0, 0.0, 0.0]],
                    indices: vec![0],
                },
            );
        }

        let renderables = collect_renderables(&world);

        assert_eq!(renderables.len(), 1);
        assert_eq!(renderables[0].entity, entity_with_mesh);
        assert_eq!(renderables[0].mesh.indices, vec![0]);
    }

    #[test]
    fn returns_empty_list_when_no_renderables_exist() {
        let world = World::new();

        assert!(collect_renderables(&world).is_empty());
    }
}
