pub mod entity;
pub mod store;
pub mod world;
pub mod arena;

// Re-export core types for ergonomic usage at the crate level
pub use entity::Entity;
pub use store::ComponentStore;
pub use world::{World, ComponentStoreTrait};
pub use arena::FrameArena;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    #[test]
    fn test_modular_ecs_flow() {
        let mut world = World::new();
        world.register_component::<Position>(10);
        world.register_component::<Velocity>(10);

        let entity = world.spawn_entity();

        {
            let mut pos_store = world.borrow_store_mut::<Position>();
            let mut vel_store = world.borrow_store_mut::<Velocity>();

            pos_store.insert(entity, Position { x: 1.0, y: 2.0 });
            vel_store.insert(entity, Velocity { dx: 0.1, dy: 0.2 });
        }

        {
            let pos_store = world.borrow_store::<Position>();
            let vel_store = world.borrow_store::<Velocity>();

            assert_eq!(pos_store.get(entity), Some(&Position { x: 1.0, y: 2.0 }));
            assert_eq!(vel_store.get(entity), Some(&Velocity { dx: 0.1, dy: 0.2 }));
        }
    }
}