pub mod mesh;
pub mod voxel;

use crate::chunk::voxel::Voxel;
use bevy::math::USizeVec3;
use bevy::prelude::*;

#[derive(Component, Default, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ChunkPos {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

pub const CHUNK_VOXEL_COUNT: usize = 32 * 32 * 32;

#[derive(Component, Debug)]
pub struct VoxelBuffer(pub [Option<Voxel>; CHUNK_VOXEL_COUNT]);

impl VoxelBuffer {
    fn index_to_pos(index: usize) -> USizeVec3 {
        USizeVec3 {
            x: index % 32,
            y: (index / 32) % 32,
            z: (index / 32) / 32,
        }
    }
}

impl Default for VoxelBuffer {
    fn default() -> Self {
        Self([None; CHUNK_VOXEL_COUNT])
    }
}
