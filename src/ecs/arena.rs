/// A safe wrapper managing transient per-frame memory allocation using `bumpalo`.
/// This prevents memory allocations/deallocations from touching the system heap
/// inside high-frequency game loops.
pub struct FrameArena {
    arena: bumpalo::Bump,
}

impl FrameArena {
    /// Initialize a transient bump allocation arena with a pre-reserved capacity.
    pub fn with_capacity(capacity_bytes: usize) -> Self {
        Self {
            arena: bumpalo::Bump::with_capacity(capacity_bytes),
        }
    }

    /// Allocate a value inside the transient arena. The value resides in this block
    /// until the frame loop explicitly executes a reset.
    #[inline]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        self.arena.alloc(value)
    }

    /// Reset the bump pointer of the arena back to zero.
    /// Instant O(1) deallocation of all frame contents.
    #[inline]
    pub fn reset(&mut self) {
        self.arena.reset();
    }
}