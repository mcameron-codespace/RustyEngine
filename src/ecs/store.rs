use super::entity::Entity;

/// A high-performance, cache-friendly storage container for a single component type `T`.
/// Maps entities to their components using a Dense array (fully packed) and a Sparse array.
pub struct ComponentStore<T> {
    dense: Vec<T>,
    dense_entities: Vec<Entity>,
    pub(crate) sparse: Vec<Option<usize>>,
}

impl<T> ComponentStore<T> {
    /// Create a component store pre-allocating memory up front to eliminate mid-loop allocations.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            dense: Vec::with_capacity(capacity),
            dense_entities: Vec::with_capacity(capacity),
            sparse: Vec::with_capacity(capacity),
        }
    }

    /// Insert or overwrite a component for a given Entity.
    pub fn insert(&mut self, entity: Entity, component: T) {
        let index = entity.id as usize;
        
        if index >= self.sparse.len() {
            self.sparse.resize(index + 1, None);
        }

        if let Some(dense_idx) = self.sparse[index] {
            self.dense[dense_idx] = component;
            self.dense_entities[dense_idx] = entity;
        } else {
            let dense_idx = self.dense.len();
            self.dense.push(component);
            self.dense_entities.push(entity);
            self.sparse[index] = Some(dense_idx);
        }
    }

    /// Remove a component from a given Entity, utilizing the swap-and-remove strategy.
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let index = entity.id as usize;
        if index >= self.sparse.len() {
            return None;
        }

        let dense_idx = self.sparse[index]?;
        self.sparse[index] = None;

        let component = self.dense.swap_remove(dense_idx);
        let _removed_entity = self.dense_entities.swap_remove(dense_idx);

        if dense_idx < self.dense.len() {
            let swapped_entity = self.dense_entities[dense_idx];
            self.sparse[swapped_entity.id as usize] = Some(dense_idx);
        }

        Some(component)
    }

    /// Retrieve a read-only reference to an entity's component.
    pub fn get(&self, entity: Entity) -> Option<&T> {
        let index = entity.id as usize;
        let dense_idx = self.sparse.get(index)?.as_ref()?;
        Some(&self.dense[*dense_idx])
    }

    /// Retrieve a mutable reference to an entity's component.
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let index = entity.id as usize;
        let dense_idx = self.sparse.get(index)?.as_ref()?;
        Some(&mut self.dense[*dense_idx])
    }

    pub fn dense_entities(&self) -> &[Entity] {
        &self.dense_entities
    }

    pub fn dense_components(&self) -> &[T] {
        &self.dense
    }

    pub fn len(&self) -> usize {
        self.dense.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Entity, &T)> {
        self.dense_entities.iter().zip(self.dense.iter())
    }
}