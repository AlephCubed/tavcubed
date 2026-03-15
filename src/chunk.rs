pub mod mesh;
pub mod voxel;

use crate::chunk::voxel::Voxel;
use bevy::math::U8Vec3;
use bevy::prelude::*;
use std::ops::{Index, IndexMut};

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
    #[inline]
    pub fn index_to_pos(index: usize) -> U8Vec3 {
        U8Vec3 {
            x: (index % 32) as u8,
            y: ((index / 32) % 32) as u8,
            z: ((index / 32) / 32) as u8,
        }
    }

    #[inline]
    pub fn pos_to_index(pos: U8Vec3) -> usize {
        (pos.z as usize * 32 * 32) + (pos.y as usize * 32) + pos.x as usize
    }
}

impl Index<usize> for VoxelBuffer {
    type Output = Option<Voxel>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for VoxelBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Index<U8Vec3> for VoxelBuffer {
    type Output = Option<Voxel>;

    fn index(&self, pos: U8Vec3) -> &Self::Output {
        &self.0[Self::pos_to_index(pos)]
    }
}

impl IndexMut<U8Vec3> for VoxelBuffer {
    fn index_mut(&mut self, pos: U8Vec3) -> &mut Self::Output {
        &mut self.0[Self::pos_to_index(pos)]
    }
}

impl Default for VoxelBuffer {
    fn default() -> Self {
        Self([None; CHUNK_VOXEL_COUNT])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_to_pos_x() {
        for x in 0..32 {
            assert_eq!(VoxelBuffer::index_to_pos(x), U8Vec3::new(x as u8, 0, 0));
        }
    }

    #[test]
    fn pos_to_index_x() {
        for x in 0..32 {
            assert_eq!(VoxelBuffer::pos_to_index(U8Vec3::new(x as u8, 0, 0)), x);
        }
    }

    #[test]
    fn index_to_pos_y() {
        for y in 0..32 {
            assert_eq!(
                VoxelBuffer::index_to_pos(y * 32),
                U8Vec3::new(0, y as u8, 0)
            );
        }
    }

    #[test]
    fn pos_to_index_y() {
        for y in 0..32 {
            assert_eq!(
                VoxelBuffer::pos_to_index(U8Vec3::new(0, y as u8, 0)),
                y * 32
            );
        }
    }

    #[test]
    fn index_to_pos_z() {
        for z in 0..32 {
            assert_eq!(
                VoxelBuffer::index_to_pos(z * 32 * 32),
                U8Vec3::new(0, 0, z as u8)
            );
        }
    }

    #[test]
    fn pos_to_index_z() {
        for z in 0..32 {
            assert_eq!(
                VoxelBuffer::pos_to_index(U8Vec3::new(0, 0, z as u8)),
                z * 32 * 32
            );
        }
    }

    #[test]
    fn index_to_pos_max() {
        assert_eq!(
            VoxelBuffer::index_to_pos(CHUNK_VOXEL_COUNT - 1),
            U8Vec3::new(31, 31, 31)
        );
    }

    #[test]
    fn pos_to_index_max() {
        assert_eq!(
            VoxelBuffer::pos_to_index(U8Vec3::new(31, 31, 31)),
            CHUNK_VOXEL_COUNT - 1
        );
    }
}
