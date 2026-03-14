pub mod mesh;
pub mod voxel;

use crate::chunk::voxel::Voxel;
use bevy::prelude::*;

#[derive(Component)]
pub struct ChunkPos {
    pub x: u32,
    pub y: u32,
}

#[derive(Component)]
pub struct VoxelBuffer(pub [Option<Voxel>; 1024]);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct DirtyFlag;
