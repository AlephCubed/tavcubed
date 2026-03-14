use std::num::NonZeroU8;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Voxel {
    id: NonZeroU8,
}

impl Default for Voxel {
    fn default() -> Self {
        Self { id: NonZeroU8::MIN }
    }
}
