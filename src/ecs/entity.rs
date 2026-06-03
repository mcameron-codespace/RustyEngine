/// An `Entity` is represented as a 64-bit integer packing:
/// - A 32-bit index (unique position in the allocation tables)
/// - A 32-bit generation (recycles index positions while preventing stale references)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

impl Entity {
    /// Pack a 32-bit index and 32-bit generation into a 64-bit raw representation.
    pub fn to_bits(&self) -> u64 {
        ((self.generation as u64) << 32) | (self.id as u64)
    }

    /// Unpack a 64-bit representation back into an `Entity`.
    pub fn from_bits(bits: u64) -> Self {
        Self {
            id: (bits & 0xFFFFFFFF) as u32,
            generation: (bits >> 32) as u32,
        }
    }
}