use crate::realm::block::VoxelId;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]

pub struct Voxel {
    pub id: VoxelId,
}

impl Voxel {
    pub fn new(id: VoxelId) -> Self {
        Self { id }
    }
}
