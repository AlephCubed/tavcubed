pub mod mesh;
mod octree;
mod voxel;

pub use octree::*;
pub use voxel::*;

use crate::realm::chunk::mesh::ChunkMesh;
use bevy::math::U8Vec3;
use bevy::prelude::*;
use std::fmt::Formatter;
use std::ops::Index;

#[derive(Component, Deref, Default, Debug, Eq, PartialEq, Clone, Copy)]
pub struct ChunkPos(pub IVec3);

pub const DIAMETER: usize = 32;
pub const RADIUS: usize = DIAMETER / 2;

pub type IntoIter = std::array::IntoIter<Option<Voxel>, CHUNK_VOXEL_COUNT>;
pub type Iter<'a> = core::slice::Iter<'a, Option<Voxel>>;
pub type IterMut<'a> = core::slice::IterMut<'a, Option<Voxel>>;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[require(ChunkMesh, ChunkPos)]
pub struct Chunk {
    octree: Octree,
    buffer: VoxelBuffer,
    len: usize,
}

impl Chunk {
    #[must_use]
    pub fn new(buffer: VoxelBuffer) -> Self {
        Self {
            buffer,
            ..default()
        }
    }

    pub fn full(voxel: Voxel) -> Self {
        Self {
            octree: Octree::full(voxel),
            buffer: [Some(voxel); CHUNK_VOXEL_COUNT],
            len: CHUNK_VOXEL_COUNT,
        }
    }

    #[must_use]
    pub fn checkerboard(a: Option<Voxel>, b: Option<Voxel>) -> Self {
        let mut chunk = Self::default();

        for i in 0..CHUNK_VOXEL_COUNT {
            match Chunk::index_to_pos(i).element_sum().is_multiple_of(2) {
                true => chunk.set(i, a.into()),
                false => chunk.set(i, b.into()),
            };
        }

        chunk
    }

    /// Returns the number of non-empty voxels in the chunk.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if all the voxels are empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if none of the voxels are empty.
    #[inline]
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.len() == CHUNK_VOXEL_COUNT
    }

    /// Returns the percent of voxels that are non-empty.
    #[inline]
    #[must_use]
    pub fn percent_full(&self) -> f32 {
        self.len as f32 / CHUNK_VOXEL_COUNT as f32
    }

    /// Returns the percent of voxels that are empty.
    #[inline]
    #[must_use]
    pub fn percent_empty(&self) -> f32 {
        1.0 - self.percent_full()
    }

    pub fn iter(&'_ self) -> Iter<'_> {
        self.buffer.iter()
    }

    /// Returns an enumerated iterator over all non-empty voxels.
    #[inline]
    pub fn iter_full(&self) -> impl Iterator<Item = (usize, Voxel)> {
        self.iter()
            .enumerate()
            .filter_map(|(i, v)| v.map(|v| (i, v)))
    }

    /// Gets the value at a specific index.
    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> &Option<Voxel> {
        &self.buffer[index]
    }

    /// Gets the value at a specific position.
    #[inline]
    #[must_use]
    pub fn get_pos(&self, pos: U8Vec3) -> &Option<Voxel> {
        self.get(Self::pos_to_index(pos))
    }

    /// Returns a [`ChunkRef`] to the value at a specific index.
    #[inline]
    #[must_use]
    pub fn get_ref(&self, index: usize) -> ChunkRef<'_> {
        ChunkRef::new(self, index)
    }

    /// Returns a [`ChunkRef`] to the value at a specific position.
    #[inline]
    #[must_use]
    pub fn get_ref_pos(&self, pos: U8Vec3) -> ChunkRef<'_> {
        self.get_ref(Self::pos_to_index(pos))
    }

    /// Sets the value at a specific index, returning the previous value.
    #[inline]
    pub fn set(&mut self, index: usize, voxel: Option<Voxel>) -> Option<Voxel> {
        self.len += self[index].is_none() as usize - voxel.is_none() as usize;
        std::mem::replace(&mut self.buffer[index], voxel)
    }

    /// Sets the value at a specific position, returning the previous value.
    #[inline]
    pub fn set_pos(&mut self, pos: U8Vec3, voxel: Option<Voxel>) -> Option<Voxel> {
        self.set(Self::pos_to_index(pos), voxel)
    }

    /// Adds a voxel at a specific index, if it is empty. Otherwise, will return `Err` with the current voxel.
    #[inline]
    pub fn place(&mut self, index: usize, voxel: Voxel) -> Result<(), Voxel> {
        match self[index] {
            None => {
                self.buffer[index] = Some(voxel);
                self.len += 1;
                Ok(())
            }
            Some(voxel) => Err(voxel),
        }
    }

    /// Adds a voxel at a specific position, if it is empty. Otherwise, will return `Err` with the current voxel.
    #[inline]
    pub fn place_pos(&mut self, pos: U8Vec3, voxel: Voxel) -> Result<(), Voxel> {
        self.place(Self::pos_to_index(pos), voxel)
    }

    /// Erases the voxel at the specified index and returns it.
    #[inline]
    pub fn erase(&mut self, index: usize) -> Option<Voxel> {
        let temp = self[index];
        self.buffer[index] = None;
        self.len -= 1;
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
        *self = Self::default();
    }

    /// Returns an iterator over all nodes/voxels at a given depth in the tree.
    ///
    /// # Panics
    /// Panics if `depth` is greater than [`OCTREE_DEPTH`].
    pub fn iter_depth(
        &self,
        depth: u32,
    ) -> VoxelGroupIter<impl Iterator<Item = ChunkRef<'_>>, impl Iterator<Item = OctreeRef<'_>>>
    {
        debug_assert!(
            depth <= (OCTREE_DEPTH as u32 + 1),
            "Depth must be less than or equal to {}, got {}",
            OCTREE_DEPTH + 1,
            depth
        );

        match depth == (OCTREE_DEPTH as u32 + 1) {
            true => VoxelGroupIter::Chunk((0..CHUNK_VOXEL_COUNT).map(|i| self.get_ref(i))),
            false => VoxelGroupIter::Octree(self.octree.iter_depth(depth)),
        }
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
        self.get(index)
    }
}

impl Index<U8Vec3> for Chunk {
    type Output = Option<Voxel>;

    fn index(&self, pos: U8Vec3) -> &Self::Output {
        self.get_pos(pos)
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
            octree: Octree::default(),
            len: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_empty_chunk() {
        let mut chunk = Chunk::default();

        for i in 0..CHUNK_VOXEL_COUNT {
            assert_eq!(chunk.len(), i);
            assert!(chunk.get(i).is_none());
            assert!(chunk.set(i, Voxel::default().into()).is_none());
            assert_eq!(chunk.get(i), &Some(Voxel::default()));
        }
    }

    #[test]
    fn set_full_chunk() {
        let mut chunk = Chunk::full(Voxel::default());

        for i in 0..CHUNK_VOXEL_COUNT {
            assert_eq!(chunk.len(), CHUNK_VOXEL_COUNT);
            assert_eq!(chunk.get(i), &Some(Voxel::default()));
            assert_eq!(
                chunk.set(i, Voxel::new_unwrap(2).into()),
                Some(Voxel::default())
            );
            assert_eq!(chunk.get(i), &Some(Voxel::new_unwrap(2)));
        }
    }

    #[test]
    fn place_success() {
        let mut chunk = Chunk::default();

        for i in 0..CHUNK_VOXEL_COUNT {
            assert_eq!(chunk.len(), i);
            assert!(chunk.get(i).is_none());
            assert!(chunk.place(i, Voxel::default().into()).is_ok());
            assert_eq!(chunk.get(i), &Some(Voxel::default()));
        }
    }

    #[test]
    fn place_failure() {
        let mut chunk = Chunk::full(Voxel::default());

        for i in 0..CHUNK_VOXEL_COUNT {
            assert_eq!(chunk.len(), CHUNK_VOXEL_COUNT);
            assert_eq!(chunk.get(i), &Voxel::default().into());
            assert_eq!(
                chunk.place(i, Voxel::default().into()),
                Err(Voxel::default())
            );
            assert_eq!(chunk.get(i), &Voxel::default().into());
        }
    }

    #[test]
    fn percent_empty_chunk() {
        let empty = Chunk::default();
        assert!(empty.is_empty());
        assert!(!empty.is_full());
        assert_eq!(empty.percent_full(), 0.0);
        assert_eq!(empty.percent_empty(), 1.0);
    }

    #[test]
    fn percent_full_chunk() {
        let empty = Chunk::full(Voxel::default());
        assert!(!empty.is_empty());
        assert!(empty.is_full());
        assert_eq!(empty.percent_full(), 1.0);
        assert_eq!(empty.percent_empty(), 0.0);
    }
}
