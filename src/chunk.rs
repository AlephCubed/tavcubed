pub mod mesh;
pub mod voxel;

use crate::chunk::voxel::Voxel;
use bevy::prelude::*;

#[derive(Component, Default, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ChunkPos {
    pub x: u32,
    pub y: u32,
}

#[derive(Component, Debug)]
pub struct VoxelBuffer(pub [Option<Voxel>; 1024]);

impl Default for VoxelBuffer {
    fn default() -> Self {
        Self([None; 1024])
    }
}

#[derive(Component, Default, Debug)]
#[component(storage = "SparseSet")]
pub struct DirtyFlag;
