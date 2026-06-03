use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

use super::entity::Entity;
use super::store::ComponentStore;

/// Object-safe trait wrapper used to store different generic types `ComponentStore<T>`
/// in a single unified collection (e.g., inside a HashMap).
pub trait ComponentStoreTrait: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove_entity(&mut self, entity: Entity);
}

impl<T: Any> ComponentStoreTrait for ComponentStore<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.remove(entity);
    }
}

/// The main World container organizing Entities, Components, and allocations.
pub struct World {
    generations: Vec<u32>,
    free_entities: Vec<u32>,
    stores: HashMap<TypeId, RefCell<Box<dyn ComponentStoreTrait>>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            generations: Vec::with_capacity(1024),
            free_entities: Vec::with_capacity(128),
            stores: HashMap::new(),
        }
    }

    /// Spawn a fresh entity and return its handle.
    pub fn spawn_entity(&mut self) -> Entity {
        if let Some(id) = self.free_entities.pop() {
            let generation = self.generations[id as usize];
            Entity { id, generation }
        } else {
            let id = self.generations.len() as u32;
            self.generations.push(1);
            Entity { id, generation: 1 }
        }
    }

    /// Backward-compatible alias retained during rollout.
    #[deprecated = "Renamed to spawn_entity; spawn_entity preserves the same behavior."]
    pub fn create_entity(&mut self) -> Entity {
        self.spawn_entity()
    }

    /// Count how many distinct entity generations are currently tracked.
    pub fn alive_entity_count(&self) -> usize {
        self.generations.len() - self.free_entities.len()
    }

    /// Count registered component archetypes currently known to this world.
    pub fn registered_component_count(&self) -> usize {
        self.stores.len()
    }

    /// Reset the world to an empty state, clearing all entities and components.
    pub fn clear(&mut self) {
        self.generations.clear();
        self.free_entities.clear();
        self.stores.clear();
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        let index = entity.id as usize;
        if index >= self.generations.len() || self.generations[index] != entity.generation {
            return;
        }

        for store in self.stores.values() {
            store.borrow_mut().remove_entity(entity);
        }

        self.generations[index] += 1;
        self.free_entities.push(entity.id);
    }

    pub fn register_component<T: Any>(&mut self, capacity: usize) {
        let type_id = TypeId::of::<T>();
        if !self.stores.contains_key(&type_id) {
            let store = ComponentStore::<T>::with_capacity(capacity);
            let boxed_store: Box<dyn ComponentStoreTrait> = Box::new(store);
            self.stores.insert(type_id, RefCell::new(boxed_store));
        }
    }

    pub fn try_borrow_store<T: Any>(&self) -> Option<Ref<'_, ComponentStore<T>>> {
        let store_cell = self.stores.get(&TypeId::of::<T>())?;

        Some(Ref::map(store_cell.borrow(), |boxed| {
            boxed
                .as_any()
                .downcast_ref::<ComponentStore<T>>()
                .expect("Downcast assertion failed.")
        }))
    }

    pub fn borrow_store<T: Any>(&self) -> Ref<'_, ComponentStore<T>> {
        self.try_borrow_store::<T>()
            .expect("Error: Tried to access unregistered component store.")
    }

    pub fn borrow_store_mut<T: Any>(&self) -> RefMut<'_, ComponentStore<T>> {
        let store_cell = self.stores
            .get(&TypeId::of::<T>())
            .expect("Error: Tried to access unregistered component store.");
        
        RefMut::map(store_cell.borrow_mut(), |boxed| {
            boxed
                .as_any_mut()
                .downcast_mut::<ComponentStore<T>>()
                .expect("Downcast assertion failed.")
        })
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    struct Foo(u32);

    #[test]
    fn test_spawn_then_destroy_then_spawn_generations() {
        let mut world = World::new();
        world.register_component::<Foo>(4);

        let first = world.spawn_entity();
        {
            let mut store = world.borrow_store_mut::<Foo>();
            store.insert(first, Foo(1));
        }
        assert_eq!(world.alive_entity_count(), 1);

        world.destroy_entity(first);
        assert_eq!(world.alive_entity_count(), 0);

        let recycled = world.spawn_entity();
        assert_eq!(recycled.id, first.id);
        assert_eq!(recycled.generation, first.generation + 1);
    }

    #[test]
    fn test_alive_count_with_multiple_entities() {
        let mut world = World::new();
        world.register_component::<Foo>(8);

        for i in 0..4u32 {
            let entity = world.spawn_entity();
            let mut store = world.borrow_store_mut::<Foo>();
            store.insert(entity, Foo(i));
        }
        assert_eq!(world.alive_entity_count(), 4);
        assert_eq!(world.registered_component_count(), 1);
    }

    #[test]
    fn test_clear_resets_world() {
        let mut world = World::new();
        world.register_component::<Foo>(4);

        {
            let entity = world.spawn_entity();
            let mut store = world.borrow_store_mut::<Foo>();
            store.insert(entity, Foo(10));
            assert!(!store.is_empty());
        }

        world.clear();
        assert_eq!(world.alive_entity_count(), 0);
        assert_eq!(world.registered_component_count(), 0);

        let _b = world.spawn_entity();
        assert_eq!(world.alive_entity_count(), 1);
    }
}