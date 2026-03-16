pub mod mesh;
pub mod voxel;

use crate::chunk::voxel::Voxel;
use bevy::math::U8Vec3;
use bevy::prelude::*;
use std::fmt::Formatter;
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

impl Clone for VoxelBuffer {
    fn clone(&self) -> Self {
        Self { 0: self.0.clone() }
    }
}

impl VoxelBuffer {
    #[inline]
    pub fn index_to_pos(index: usize) -> U8Vec3 {
        assert!(
            index < CHUNK_VOXEL_COUNT,
            "Index must be less than 32^3, got {index}"
        );
        U8Vec3 {
            x: (index % 32) as u8,
            y: ((index / 32) % 32) as u8,
            z: ((index / 32) / 32) as u8,
        }
    }

    #[inline]
    pub fn pos_to_index(pos: U8Vec3) -> usize {
        assert!(pos.x < 32, "x position must be less than 32, got {}", pos.x);
        assert!(pos.y < 32, "y position must be less than 32, got {}", pos.y);
        assert!(pos.z < 32, "z position must be less than 32, got {}", pos.z);
        ((pos.z as usize) << 10) + ((pos.y as usize) << 5) + pos.x as usize
    }
}

impl std::fmt::Display for VoxelBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, v) in self.0.iter().enumerate() {
            if i % 32 == 0 {
                writeln!(f, "")?;
            }

            if i % (32 * 32) == 0 {
                writeln!(f, "Z={}", i / (32 * 32))?;
            }

            write!(f, "{:x}", v.map(|v| v.id.get()).unwrap_or(0))?;
        }

        Ok(())
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

    #[test]
    #[should_panic(expected = "Index must be less than 32^3, got 32768")]
    fn index_to_pos_invalid() {
        _ = VoxelBuffer::index_to_pos(CHUNK_VOXEL_COUNT)
    }

    #[test]
    #[should_panic(expected = "x position must be less than 32, got 32")]
    fn pos_to_index_invalid_x() {
        _ = VoxelBuffer::pos_to_index(U8Vec3::new(32, 0, 0));
    }

    #[test]
    #[should_panic(expected = "y position must be less than 32, got 32")]
    fn pos_to_index_invalid_y() {
        _ = VoxelBuffer::pos_to_index(U8Vec3::new(16, 32, 16));
    }

    #[test]
    #[should_panic(expected = "z position must be less than 32, got 32")]
    fn pos_to_index_invalid_z() {
        _ = VoxelBuffer::pos_to_index(U8Vec3::new(31, 31, 32));
    }
}
