pub mod mesh;
mod octree;
pub mod voxel;

use crate::realm::chunk::mesh::ChunkMesh;
use crate::realm::chunk::voxel::Voxel;
use bevy::math::U8Vec3;
use bevy::prelude::*;
use std::fmt::Formatter;
use std::ops::{Index, IndexMut};

#[derive(Component, Deref, Default, Debug, Eq, PartialEq, Clone, Copy)]
pub struct ChunkPos(pub IVec3);

pub const DIAMETER: usize = 32;

pub type IntoIter = std::array::IntoIter<Option<Voxel>, CHUNK_VOXEL_COUNT>;
pub type Iter<'a> = core::slice::Iter<'a, Option<Voxel>>;
pub type IterMut<'a> = core::slice::IterMut<'a, Option<Voxel>>;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[require(ChunkMesh, ChunkPos)]
pub struct Chunk {
    buffer: VoxelBuffer,
}

impl Chunk {
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
