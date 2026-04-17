use crate::realm::block_registry::VoxelId;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]

pub struct Voxel {
    pub id: VoxelId,
}
