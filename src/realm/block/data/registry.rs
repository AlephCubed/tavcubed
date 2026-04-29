use crate::realm::block::data::Block;
use crate::realm::block::{BlockId, BlockPlugin, VoxelId};
use bevy::prelude::*;
use bevy_auto_plugin::prelude::auto_resource;
use std::collections::HashMap;
use std::ops::Index;
use std::sync::Arc;

/// Maps [block](BlockId) and [voxel](VoxelId) IDs to [block data](Block).
#[derive(Resource, Deref, DerefMut, Default)]
#[auto_resource(plugin = BlockPlugin, init)]
pub struct BlockRegistry(pub Arc<BlockRegistryInner>);

impl BlockRegistry {
    pub fn new(inner: BlockRegistryInner) -> Self {
        Self(Arc::new(inner))
    }
}

/// Maps [block](BlockId) and [voxel](VoxelId) IDs to [block data](Block).
#[derive(Default, Debug)]
pub struct BlockRegistryInner {
    blocks: Vec<Block>,
    id_map: HashMap<BlockId, VoxelId>,
    pub textures: Option<Handle<Image>>,
}

impl BlockRegistryInner {
    /// Registers a new block, returning its unstable [`VoxelId`].
    pub fn register(&mut self, block: Block) -> VoxelId {
        let block_id = block.id().clone();
        self.blocks.push(block);

        let voxel_id = VoxelId::new(self.blocks.len() as u16).unwrap();
        self.id_map.insert(block_id, voxel_id);

        voxel_id
    }

    /// Gets the block's unstable [`VoxelId`].
    ///
    /// # Panics
    /// Panics if the block does not exist. Use [`get_voxel_id`](Self::get_voxel_id) instead if you want to handle this case.
    pub fn voxel_id(&self, block_id: &BlockId) -> VoxelId {
        self.id_map[block_id]
    }

    /// Gets the block's unstable [`VoxelId`], if it exists.
    ///
    /// Use [`voxel_id`](Self::voxel_id) instead if you do not want to handle this case.
    pub fn get_voxel_id(&self, block_id: &BlockId) -> Option<&VoxelId> {
        self.id_map.get(block_id)
    }

    /// Gets the block's data, if it exists.
    ///
    /// Use the [`Index`] implementation instead if you do not want to handle this case.
    pub fn get(&self, block_id: &BlockId) -> Option<&Block> {
        let voxel_id = self.get_voxel_id(block_id)?;
        self.get_from_voxel_id(voxel_id)
    }

    /// Gets the block's data from its unstable ID, if it exists.
    ///
    /// Use the [`Index`] implementation instead if you do not want to handle this case.
    pub fn get_from_voxel_id(&self, voxel_id: &VoxelId) -> Option<&Block> {
        self.blocks.get(voxel_id.get() as usize - 1)
    }
}

impl Index<BlockId> for BlockRegistryInner {
    type Output = Block;

    fn index(&self, index: BlockId) -> &Self::Output {
        let voxel_id = self.voxel_id(&index);
        &self[voxel_id]
    }
}

impl Index<VoxelId> for BlockRegistryInner {
    type Output = Block;

    fn index(&self, index: VoxelId) -> &Self::Output {
        &self.blocks[index.get() as usize - 1]
    }
}
