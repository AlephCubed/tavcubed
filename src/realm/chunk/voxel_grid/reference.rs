use crate::realm::chunk::{
    DIAMETER, OCTREE_DEPTH, Octree, OctreeNodeFlag, OctreeRef, STRIDE_X, STRIDE_Y, STRIDE_Z, Voxel,
    VoxelGrid, VoxelGroupRef,
};
use bevy::math::U8Vec3;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

/// A reference to a singular voxel in a [`VoxelGrid`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VoxelRef<'a> {
    voxels: &'a VoxelGrid,
    index: usize,
}

impl Debug for VoxelRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "VoxelRef<{:p}>[{}]", &self.voxels, self.index)
    }
}

impl<'a> Deref for VoxelRef<'a> {
    type Target = Option<Voxel>;

    fn deref(&self) -> &Self::Target {
        &self.voxels[self.index]
    }
}

#[expect(clippy::manual_is_multiple_of)]
impl VoxelGroupRef for VoxelRef<'_> {
    fn voxel(&self) -> Option<Voxel> {
        self.voxels[self.index]
    }

    fn flags(&self) -> OctreeNodeFlag {
        OctreeNodeFlag::UNIFORM
    }

    fn depth(&self) -> usize {
        OCTREE_DEPTH + 1
    }

    fn pos(&self) -> U8Vec3 {
        VoxelGrid::index_to_pos(self.index)
    }

    fn size(&self) -> u8 {
        1
    }

    fn right(&self) -> Option<Self> {
        (self.index % STRIDE_Y < DIAMETER - 1)
            .then(|| VoxelRef::new(self.voxels, self.index + STRIDE_X))
    }

    fn left(&self) -> Option<Self> {
        (self.index % STRIDE_Y > 0).then(|| VoxelRef::new(self.voxels, self.index - STRIDE_X))
    }

    fn up(&self) -> Option<Self> {
        ((self.index / STRIDE_Y) % STRIDE_Y < DIAMETER - 1)
            .then(|| VoxelRef::new(self.voxels, self.index + STRIDE_Y))
    }

    fn down(&self) -> Option<Self> {
        ((self.index / STRIDE_Y) % STRIDE_Y > 0)
            .then(|| VoxelRef::new(self.voxels, self.index - STRIDE_Y))
    }

    fn backward(&self) -> Option<Self> {
        (self.index / STRIDE_Z < DIAMETER - 1)
            .then(|| VoxelRef::new(self.voxels, self.index + STRIDE_Z))
    }

    fn forward(&self) -> Option<Self> {
        (self.index / STRIDE_Z > 0).then(|| VoxelRef::new(self.voxels, self.index - STRIDE_Z))
    }
}

impl<'a> VoxelRef<'a> {
    pub fn new(voxels: &'a VoxelGrid, index: usize) -> Self {
        Self { voxels, index }
    }

    pub fn voxels(self) -> &'a VoxelGrid {
        self.voxels
    }

    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns an [`OctreeRef`] to the voxels parent node.
    pub fn parent(self, octree: &'a Octree) -> OctreeRef<'a> {
        OctreeRef::leaf_from_voxel_pos(octree, self.pos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm::chunk::Chunk;

    #[test]
    fn voxel_grid_ref_right() {
        let voxel_grid = VoxelGrid::default();
        let r = voxel_grid.get_ref_pos(U8Vec3::default());
        assert_eq!(r.right().unwrap().pos(), U8Vec3::X);
    }

    #[test]
    fn voxel_grid_ref_left() {
        let voxel_grid = VoxelGrid::default();
        let r = voxel_grid.get_ref_pos(U8Vec3::X);
        assert_eq!(r.left().unwrap().pos(), U8Vec3::default());
    }

    #[test]
    fn voxel_grid_ref_up() {
        let voxel_grid = VoxelGrid::default();
        let r = voxel_grid.get_ref_pos(U8Vec3::default());
        assert_eq!(r.up().unwrap().pos(), U8Vec3::Y);
    }

    #[test]
    fn voxel_grid_ref_down() {
        let voxel_grid = VoxelGrid::default();
        let r = voxel_grid.get_ref_pos(U8Vec3::Y);
        assert_eq!(r.down().unwrap().pos(), U8Vec3::default());
    }

    #[test]
    fn voxel_grid_ref_backward() {
        let voxel_grid = VoxelGrid::default();
        let r = voxel_grid.get_ref_pos(U8Vec3::default());
        assert_eq!(r.backward().unwrap().pos(), U8Vec3::Z);
    }

    #[test]
    fn voxel_grid_ref_forward() {
        let voxel_grid = VoxelGrid::default();
        let r = voxel_grid.get_ref_pos(U8Vec3::Z);
        assert_eq!(r.forward().unwrap().pos(), U8Vec3::default());
    }
}
