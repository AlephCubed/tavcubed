use crate::realm::block::VoxelId;
use crate::realm::chunk::{Chunk, DIAMETER};
use bevy::math::U8Vec3;

pub const CHUNK_VOXEL_COUNT: usize = DIAMETER * DIAMETER * DIAMETER;
pub const STRIDE_X: usize = 1;
pub const STRIDE_Y: usize = DIAMETER;
pub const STRIDE_Z: usize = DIAMETER * DIAMETER;

pub type VoxelBuffer = [Option<Voxel>; CHUNK_VOXEL_COUNT];

impl Chunk {
    #[inline]
    pub fn index_to_pos(index: usize) -> U8Vec3 {
        debug_assert!(
            index < CHUNK_VOXEL_COUNT,
            "Index must be less than {CHUNK_VOXEL_COUNT}, got {index}"
        );
        U8Vec3 {
            x: (index % STRIDE_Y) as u8,
            y: ((index / STRIDE_Y) % STRIDE_Y) as u8,
            z: (index / STRIDE_Z) as u8,
        }
    }

    #[inline]
    pub fn pos_to_index(pos: U8Vec3) -> usize {
        debug_assert!(
            pos.x < DIAMETER as u8,
            "x position must be less than {DIAMETER}, got {}",
            pos.x
        );
        debug_assert!(
            pos.y < DIAMETER as u8,
            "y position must be less than {DIAMETER}, got {}",
            pos.y
        );
        debug_assert!(
            pos.z < DIAMETER as u8,
            "z position must be less than {DIAMETER}, got {}",
            pos.z
        );
        (pos.z as usize * STRIDE_Z) + (pos.y as usize * STRIDE_Y) + pos.x as usize
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Voxel {
    pub id: VoxelId,
}

impl Voxel {
    pub fn new(id: VoxelId) -> Self {
        Self { id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_to_pos_x() {
        for x in 0..32 {
            assert_eq!(Chunk::index_to_pos(x), U8Vec3::new(x as u8, 0, 0));
        }
    }

    #[test]
    fn pos_to_index_x() {
        for x in 0..32 {
            assert_eq!(Chunk::pos_to_index(U8Vec3::new(x as u8, 0, 0)), x);
        }
    }

    #[test]
    fn index_to_pos_y() {
        for y in 0..32 {
            assert_eq!(Chunk::index_to_pos(y * 32), U8Vec3::new(0, y as u8, 0));
        }
    }

    #[test]
    fn pos_to_index_y() {
        for y in 0..32 {
            assert_eq!(Chunk::pos_to_index(U8Vec3::new(0, y as u8, 0)), y * 32);
        }
    }

    #[test]
    fn index_to_pos_z() {
        for z in 0..32 {
            assert_eq!(Chunk::index_to_pos(z * 32 * 32), U8Vec3::new(0, 0, z as u8));
        }
    }

    #[test]
    fn pos_to_index_z() {
        for z in 0..32 {
            assert_eq!(Chunk::pos_to_index(U8Vec3::new(0, 0, z as u8)), z * 32 * 32);
        }
    }

    #[test]
    fn index_to_pos_max() {
        assert_eq!(
            Chunk::index_to_pos(CHUNK_VOXEL_COUNT - 1),
            U8Vec3::new(31, 31, 31)
        );
    }

    #[test]
    fn pos_to_index_max() {
        assert_eq!(
            Chunk::pos_to_index(U8Vec3::new(31, 31, 31)),
            CHUNK_VOXEL_COUNT - 1
        );
    }

    #[test]
    #[should_panic(expected = "Index must be less than 32768, got 32768")]
    fn index_to_pos_invalid() {
        _ = Chunk::index_to_pos(CHUNK_VOXEL_COUNT)
    }

    #[test]
    #[should_panic(expected = "x position must be less than 32, got 32")]
    fn pos_to_index_invalid_x() {
        _ = Chunk::pos_to_index(U8Vec3::new(32, 0, 0));
    }

    #[test]
    #[should_panic(expected = "y position must be less than 32, got 32")]
    fn pos_to_index_invalid_y() {
        _ = Chunk::pos_to_index(U8Vec3::new(16, 32, 16));
    }

    #[test]
    #[should_panic(expected = "z position must be less than 32, got 32")]
    fn pos_to_index_invalid_z() {
        _ = Chunk::pos_to_index(U8Vec3::new(31, 31, 32));
    }
}
