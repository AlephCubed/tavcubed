pub mod mesh;
pub mod voxel;

use crate::realm::chunk::mesh::ChunkMesh;
use crate::realm::chunk::voxel::Voxel;
use bevy::math::U8Vec3;
use bevy::prelude::*;
use std::fmt::Formatter;
use std::ops::{Index, IndexMut};

#[derive(Component, Deref, Default, Debug, Eq, PartialEq, Clone, Copy)]
pub struct ChunkPos(pub IVec3);

pub const CHUNK_VOXEL_COUNT: usize = 32 * 32 * 32;
pub const STRIDE_X: usize = 1;
pub const STRIDE_Y: usize = 32;
pub const STRIDE_Z: usize = 32 * 32;

pub type VoxelBuffer = [Option<Voxel>; CHUNK_VOXEL_COUNT];
pub type IntoIter = std::array::IntoIter<Option<Voxel>, CHUNK_VOXEL_COUNT>;
pub type Iter<'a> = core::slice::Iter<'a, Option<Voxel>>;
pub type IterMut<'a> = core::slice::IterMut<'a, Option<Voxel>>;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[require(ChunkMesh, ChunkPos)]
pub struct Chunk {
    buffer: VoxelBuffer,
}

impl Chunk {
    #[inline]
    pub fn index_to_pos(index: usize) -> U8Vec3 {
        debug_assert!(
            index < CHUNK_VOXEL_COUNT,
            "Index must be less than 32^3, got {index}"
        );
        U8Vec3 {
            x: (index % 32) as u8,
            y: ((index / 32) % 32) as u8,
            z: (index / (32 * 32)) as u8,
        }
    }

    #[inline]
    pub fn pos_to_index(pos: U8Vec3) -> usize {
        debug_assert!(pos.x < 32, "x position must be less than 32, got {}", pos.x);
        debug_assert!(pos.y < 32, "y position must be less than 32, got {}", pos.y);
        debug_assert!(pos.z < 32, "z position must be less than 32, got {}", pos.z);
        ((pos.z as usize) << 10) + ((pos.y as usize) << 5) + pos.x as usize
    }

    pub fn new(buffer: VoxelBuffer) -> Self {
        Self { buffer }
    }

    pub fn checkerboard() -> Self {
        let mut chunk = Self::default();

        for (index, voxel) in &mut chunk.iter_mut().enumerate() {
            if Chunk::index_to_pos(index).element_sum().is_multiple_of(2) {
                *voxel = Some(Voxel::default())
            }
        }

        chunk
    }

    pub fn iter(&'_ self) -> Iter<'_> {
        self.buffer.iter()
    }

    pub fn iter_mut(&'_ mut self) -> IterMut<'_> {
        self.buffer.iter_mut()
    }

    /// Returns an enumerated iterator over all non-empty voxels.
    #[inline]
    pub fn iter_full(&self) -> impl Iterator<Item = (usize, Voxel)> {
        self.iter()
            .enumerate()
            .filter_map(|(i, v)| v.map(|v| (i, v)))
    }

    /// Sets the value at a specific index, returning the previous value.
    #[inline]
    pub fn swap(&mut self, index: usize, mut voxel: Option<Voxel>) -> Option<Voxel> {
        std::mem::swap(&mut voxel, &mut self[index]);
        voxel
    }

    /// Sets the value at a specific position, returning the previous value.
    #[inline]
    pub fn swap_pos(&mut self, pos: U8Vec3, voxel: Option<Voxel>) -> Option<Voxel> {
        self.swap(Self::pos_to_index(pos), voxel)
    }

    /// Adds a voxel at a specific index, if it is empty. Otherwise, will return `Err` with the current voxel.
    #[inline]
    pub fn place(&mut self, index: usize, voxel: Option<Voxel>) -> Result<(), Voxel> {
        match self[index] {
            None => {
                self[index] = voxel;
                Ok(())
            }
            Some(voxel) => Err(voxel),
        }
    }

    /// Adds a voxel at a specific position, if it is empty. Otherwise, will return `Err` with the current voxel.
    #[inline]
    pub fn place_pos(&mut self, pos: U8Vec3, voxel: Option<Voxel>) -> Result<(), Voxel> {
        self.place(Self::pos_to_index(pos), voxel)
    }

    /// Erases the voxel at the specified index and returns it.
    #[inline]
    pub fn erase(&mut self, index: usize) -> Option<Voxel> {
        let temp = self[index];
        self[index] = None;
        temp
    }

    /// Erases the voxel at the specified position and returns it.
    #[inline]
    pub fn erase_pos(&mut self, pos: U8Vec3) -> Option<Voxel> {
        self.erase(Self::pos_to_index(pos))
    }

    /// Removes all voxels from the buffer.
    #[inline]
    pub fn clear(&mut self) {
        self.buffer = [None; CHUNK_VOXEL_COUNT];
    }
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, v) in self.iter().enumerate() {
            if i % 32 == 0 {
                writeln!(f)?;
            }

            if i % (32 * 32) == 0 {
                writeln!(f, "Z={}", i / (32 * 32))?;
            }

            write!(f, "{:x}", v.map(|v| v.id.get()).unwrap_or(0))?;
        }

        Ok(())
    }
}

impl Index<usize> for Chunk {
    type Output = Option<Voxel>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index]
    }
}

impl IndexMut<usize> for Chunk {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer[index]
    }
}

impl Index<U8Vec3> for Chunk {
    type Output = Option<Voxel>;

    fn index(&self, pos: U8Vec3) -> &Self::Output {
        &self.buffer[Self::pos_to_index(pos)]
    }
}

impl IndexMut<U8Vec3> for Chunk {
    fn index_mut(&mut self, pos: U8Vec3) -> &mut Self::Output {
        &mut self.buffer[Self::pos_to_index(pos)]
    }
}

impl IntoIterator for Chunk {
    type Item = Option<Voxel>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            buffer: [None; CHUNK_VOXEL_COUNT],
        }
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
    #[should_panic(expected = "Index must be less than 32^3, got 32768")]
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
