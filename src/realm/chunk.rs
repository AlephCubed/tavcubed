pub mod mesh;
mod octree;
mod reference;
mod voxel_grid;

pub use octree::*;
pub use reference::*;
pub use voxel_grid::*;

use crate::realm::RealmPlugin;
use crate::realm::chunk::mesh::ChunkLOD;
use crate::realm::chunk::mesh::ChunkMesh;
use bevy::math::U8Vec3;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::{AutoPlugin, auto_add_plugin};
use std::ops::Deref;

#[derive(AutoPlugin)]
#[auto_plugin(impl_plugin_trait)]
#[auto_add_plugin(plugin = RealmPlugin)]
pub struct ChunkPlugin;

/// The realm-space position of a chunk, measured in chunks (32^3 voxels).
#[derive(Component, Deref, Default, Debug, Eq, PartialEq, Clone, Copy)]
pub struct ChunkPos(pub IVec3);

/// A 32^3 piece of the realm's voxels.
#[derive(Component, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[require(ChunkMesh, ChunkPos, ChunkLOD)]
pub struct Chunk {
    octree: Octree,
    voxel_grid: VoxelGrid,
}

impl Deref for Chunk {
    type Target = VoxelGrid;

    fn deref(&self) -> &Self::Target {
        &self.voxel_grid
    }
}

impl Chunk {
    /// Creates a new chunk from a [`VoxelGrid`].
    ///
    /// # Performance
    /// This is the preferred way to instantiate a chunk, as modifications made through the chunk's
    /// setters must update the [`Octree`].
    #[must_use]
    pub fn new(voxel_grid: VoxelGrid) -> Self {
        let octree = Octree::new(&voxel_grid);
        Self { voxel_grid, octree }
    }

    /// Creates a chunk where every block is set to the given voxel.
    ///
    /// For a chunk full of air, use [`Chunk::default()`].
    #[must_use]
    pub fn full(voxel: Voxel) -> Self {
        Self {
            octree: Octree::full(voxel),
            voxel_grid: VoxelGrid::full(voxel),
        }
    }

    /// Creates a chunk with alternating voxels between `a` and `b`.
    #[must_use]
    pub fn checkerboard(a: Option<Voxel>, b: Option<Voxel>) -> Self {
        Self::new(VoxelGrid::checkerboard(a, b))
    }

    /// Sets the value at a specific index, returning the previous value.
    pub fn set(&mut self, index: usize, voxel: Option<Voxel>) -> Option<Voxel> {
        let voxel = self.voxel_grid.set(index, voxel);
        self.octree.update(index, &self.voxel_grid);
        voxel
    }

    /// Sets the value at a specific position, returning the previous value.
    #[inline]
    pub fn set_pos(&mut self, pos: U8Vec3, voxel: Option<Voxel>) -> Option<Voxel> {
        self.set(VoxelGrid::pos_to_index(pos), voxel)
    }

    /// Adds a voxel at a specific index, if it is empty; otherwise returns `Err` with the current voxel.
    pub fn place(&mut self, index: usize, voxel: Voxel) -> Result<(), Voxel> {
        self.voxel_grid
            .place(index, voxel)
            .inspect(|_| self.octree.update(index, &self.voxel_grid))
    }

    /// Adds a voxel at a specific position, if it is empty; otherwise, returns `Err` with the current voxel.
    #[inline]
    pub fn place_pos(&mut self, pos: U8Vec3, voxel: Voxel) -> std::result::Result<(), Voxel> {
        self.place(VoxelGrid::pos_to_index(pos), voxel)
    }

    /// Erases the voxel at the specified index and returns it.
    pub fn erase(&mut self, index: usize) -> Option<Voxel> {
        self.voxel_grid
            .erase(index)
            .inspect(|_| self.octree.update(index, &self.voxel_grid))
    }

    /// Erases the voxel at the specified position and returns it.
    #[inline]
    pub fn erase_pos(&mut self, pos: U8Vec3) -> Option<Voxel> {
        self.erase(VoxelGrid::pos_to_index(pos))
    }

    /// Resets the chunk to an empty/default state.
    #[inline]
    pub fn clear(&mut self) {
        self.voxel_grid.clear();
        self.octree.clear();
    }

    /// Returns an iterator over all nodes/voxels at a given depth in the tree.
    ///
    /// # Panics
    /// Panics if `depth` is greater than [`OCTREE_DEPTH + 1`](OCTREE_DEPTH).
    pub fn iter_depth(
        &self,
        depth: usize,
    ) -> VoxelGroupIter<'_, impl Iterator<Item = VoxelRef<'_>>, impl Iterator<Item = OctreeRef<'_>>>
    {
        debug_assert!(
            depth <= OCTREE_DEPTH + 1,
            "Depth must be less than or equal to {}, got {}",
            OCTREE_DEPTH + 1,
            depth
        );

        match depth == OCTREE_DEPTH + 1 {
            true => VoxelGroupIter::Chunk((0..VOXEL_COUNT).map(|i| self.get_ref(i))),
            false => VoxelGroupIter::Octree(self.octree.iter_depth(depth)),
        }
    }
}
