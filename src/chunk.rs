pub mod mesh;
pub mod voxel;

use crate::chunk::voxel::Voxel;
use bevy::prelude::*;

#[derive(Component, Default, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ChunkPos {
    pub x: u32,
    pub y: u32,
}

const CHUNK_VOXEL_COUNT: usize = 32 ^ 3;

#[derive(Component, Debug)]
pub struct VoxelBuffer(pub [Option<Voxel>; CHUNK_VOXEL_COUNT]);

impl Default for VoxelBuffer {
    fn default() -> Self {
        Self([None; CHUNK_VOXEL_COUNT])
    }
}
