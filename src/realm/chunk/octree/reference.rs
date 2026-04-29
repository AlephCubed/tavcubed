use crate::realm::chunk::reference::{VoxelGroupIter, VoxelGroupRef};
use crate::realm::chunk::{
    OCTREE_DEPTH, Octree, OctreeNode, OctreeNodeFlag, Voxel, VoxelGrid, VoxelRef,
};
use bevy::math::{I8Vec3, U8Vec3};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

/// A reference to a group of voxels represented by an [`Octree`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OctreeRef<'a> {
    octree: &'a Octree,
    pub(super) index: usize,
}

impl Debug for OctreeRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OctreeRef<{:p}>[{}]", &self.octree, self.index)
    }
}

impl<'a> Deref for OctreeRef<'a> {
    type Target = OctreeNode;

    fn deref(&self) -> &Self::Target {
        &self.octree.buffer[self.index]
    }
}

impl VoxelGroupRef for OctreeRef<'_> {
    fn voxel(&self) -> Option<Voxel> {
        self.node().voxel()
    }

    fn flags(&self) -> OctreeNodeFlag {
        self.node().flags()
    }

    fn depth(&self) -> usize {
        Octree::get_depth(self.index)
    }

    fn pos(&self) -> U8Vec3 {
        Octree::node_index_to_pos(self.index)
    }

    fn size(&self) -> u8 {
        Octree::DEPTH_VOXEL_DIAMETER[self.depth()] as u8
    }

    fn right(&self) -> Option<Self> {
        self.offset(I8Vec3::X)
    }

    fn left(&self) -> Option<Self> {
        self.offset(I8Vec3::NEG_X)
    }

    fn up(&self) -> Option<Self> {
        self.offset(I8Vec3::Y)
    }

    fn down(&self) -> Option<Self> {
        self.offset(I8Vec3::NEG_Y)
    }

    fn backward(&self) -> Option<Self> {
        self.offset(I8Vec3::Z)
    }

    fn forward(&self) -> Option<Self> {
        self.offset(I8Vec3::NEG_Z)
    }
}

impl<'a> OctreeRef<'a> {
    pub(super) fn new(octree: &'a Octree, index: usize) -> Self {
        OctreeRef { octree, index }
    }

    /// Creates a new reference to the given voxel's parent node.
    pub fn leaf_from_voxel_pos(octree: &'a Octree, pos: U8Vec3) -> Self {
        Self::new(octree, Octree::pos_to_node_index(pos, OCTREE_DEPTH))
    }

    pub fn node(&self) -> OctreeNode {
        self.octree.buffer[self.index]
    }

    pub fn octree(&self) -> &Octree {
        self.octree
    }

    /// Returns a reference to a sibling node that is `offset` nodes away.
    pub fn offset(self, offset: I8Vec3) -> Option<OctreeRef<'a>> {
        if self.depth() == 0 {
            return None;
        }

        let pos = self
            .pos()
            .checked_add_signed(offset.checked_mul(I8Vec3::splat(
                Octree::DEPTH_VOXEL_DIAMETER[self.depth()] as i8,
            ))?)?;

        let index = Octree::pos_to_node_index(pos, self.depth());

        (Octree::get_depth(index) == self.depth() && index != self.index)
            .then(|| Self::new(self.octree, index))
    }

    /// Returns a reference to the parent node, or `None` if [root](Self::is_root_node).
    pub fn parent(self) -> Option<OctreeRef<'a>> {
        if self.is_root_node() {
            return None;
        }

        Some(OctreeRef {
            octree: self.octree,
            index: Octree::parent_index(self.index),
        })
    }

    /// Returns an iterator over the node's children (nodes or voxels).
    ///
    /// Use [`child_nodes`](Self::child_nodes) to loop over only nodes,
    /// or [`iter_voxels`](Self::iter_voxels) to loop over only the voxels.
    pub fn children(
        &'a self,
        voxels: &'a VoxelGrid,
    ) -> VoxelGroupIter<'a, impl Iterator<Item = VoxelRef<'a>>, impl Iterator<Item = OctreeRef<'a>>>
    {
        match self.is_leaf_node() {
            true => VoxelGroupIter::Chunk(self.iter_voxels(voxels)),
            false => VoxelGroupIter::Octree(self.child_nodes().unwrap()),
        }
    }

    /// Returns an iterator over the node's child nodes. Returns none if the node is a [leaf](Self::is_leaf_node).
    ///
    /// Use [`iter_voxels`](Self::iter_voxels) to loop over only the voxels,
    /// or [`children`](Self::children) to loop over either.
    pub fn child_nodes(&self) -> Option<impl Iterator<Item = OctreeRef<'_>>> {
        if self.is_leaf_node() {
            return None;
        }

        Some(Octree::children_indices(self.index).map(|i| OctreeRef {
            octree: self.octree,
            index: i,
        }))
    }

    /// Returns a [`VoxelRef`] iterator including all voxels that are descendants of the current node.
    pub fn iter_voxels(&'a self, voxels: &'a VoxelGrid) -> impl Iterator<Item = VoxelRef<'a>> {
        Octree::iter_voxel_indices(self.index).map(|i| VoxelRef::new(voxels, i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm::chunk::{Chunk, OCTREE_DEPTH};

    #[test]
    fn octree_ref_right() {
        let octree = Octree::default();
        let r = octree.get_ref_pos(U8Vec3::default(), 1);
        assert_eq!(r.right().unwrap().pos(), U8Vec3::X * 16);
    }

    #[test]
    fn octree_ref_left() {
        let octree = Octree::default();
        let r = octree.get_ref_pos(U8Vec3::X * 16, 1);
        assert_eq!(r.left().unwrap().pos(), U8Vec3::default());
    }

    #[test]
    fn octree_ref_up() {
        let octree = Octree::default();
        let r = octree.get_ref_pos(U8Vec3::default(), 1);
        assert_eq!(r.up().unwrap().pos(), U8Vec3::Y * 16);
    }

    #[test]
    fn octree_ref_down() {
        let octree = Octree::default();
        let r = octree.get_ref_pos(U8Vec3::Y * 16, 1);
        assert_eq!(r.down().unwrap().pos(), U8Vec3::default());
    }

    #[test]
    fn octree_ref_backward() {
        let octree = Octree::default();
        let r = octree.get_ref_pos(U8Vec3::default(), 1);
        assert_eq!(r.backward().unwrap().pos(), U8Vec3::Z * 16);
    }

    #[test]
    fn octree_ref_forward() {
        let octree = Octree::default();
        let r = octree.get_ref_pos(U8Vec3::Z * 16, 1);
        assert_eq!(r.forward().unwrap().pos(), U8Vec3::default());
    }

    #[test]
    fn octree_ref_right_across_parents() {
        let octree = Octree::default();
        let r = octree.get_ref_pos(U8Vec3::X, 2);
        assert_eq!(r.right().unwrap().pos(), U8Vec3::X * 8);
    }

    #[test]
    fn octree_ref_left_across_parents() {
        let octree = Octree::default();
        let r = octree.get_ref_pos(U8Vec3::X * 8, 2);
        assert_eq!(r.left().unwrap().pos(), U8Vec3::default());
    }

    #[test]
    fn octree_ref_up_across_parents() {
        let octree = Octree::default();
        let r = octree.get_ref_pos(U8Vec3::Y, 2);
        assert_eq!(r.up().unwrap().pos(), U8Vec3::Y * 8);
    }

    #[test]
    fn octree_ref_down_across_parents() {
        let octree = Octree::default();
        let r = octree.get_ref_pos(U8Vec3::Y * 8, 2);
        assert_eq!(r.down().unwrap().pos(), U8Vec3::default());
    }

    #[test]
    fn octree_ref_backward_across_parents() {
        let octree = Octree::default();
        let r = octree.get_ref_pos(U8Vec3::Z, 2);
        assert_eq!(r.backward().unwrap().pos(), U8Vec3::Z * 8);
    }

    #[test]
    fn octree_ref_forward_across_parents() {
        let octree = Octree::default();
        let r = octree.get_ref_pos(U8Vec3::Z * 8, 2);
        assert_eq!(r.forward().unwrap().pos(), U8Vec3::default());
    }

    #[test]
    fn octree_ref_right_none() {
        let octree = Octree::default();

        for depth in 0..=OCTREE_DEPTH {
            let r = octree.get_ref_pos(U8Vec3::X * 31, depth);
            assert_eq!(r.right(), None);
        }
    }

    #[test]
    fn octree_ref_left_none() {
        let octree = Octree::default();

        for depth in 0..=OCTREE_DEPTH {
            let r = octree.get_ref_pos(U8Vec3::default(), depth);
            assert_eq!(r.left(), None);
        }
    }

    #[test]
    fn octree_ref_up_none() {
        let octree = Octree::default();

        for depth in 0..=OCTREE_DEPTH {
            let r = octree.get_ref_pos(U8Vec3::Y * 31, depth);
            assert_eq!(r.up(), None);
        }
    }

    #[test]
    fn octree_ref_down_none() {
        let octree = Octree::default();

        for depth in 0..=OCTREE_DEPTH {
            let r = octree.get_ref_pos(U8Vec3::default(), depth);
            assert_eq!(r.down(), None);
        }
    }

    #[test]
    fn octree_ref_backward_none() {
        let octree = Octree::default();

        for depth in 0..=OCTREE_DEPTH {
            let r = octree.get_ref_pos(U8Vec3::Z * 31, depth);
            assert_eq!(r.backward(), None);
        }
    }

    #[test]
    fn octree_ref_forward_none() {
        let octree = Octree::default();

        for depth in 0..=OCTREE_DEPTH {
            let r = octree.get_ref_pos(U8Vec3::default(), depth);
            assert_eq!(r.forward(), None);
        }
    }

    #[test]
    fn octree_ref_directions_root() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 0);

        assert_eq!(r.right(), None);
        assert_eq!(r.left(), None);
        assert_eq!(r.up(), None);
        assert_eq!(r.down(), None);
        assert_eq!(r.backward(), None);
        assert_eq!(r.forward(), None);
    }

    #[test]
    fn octree_ref_size() {
        let octree = Octree::default();
        assert_eq!(OctreeRef::new(&octree, 0).size(), 32);
        assert_eq!(OctreeRef::new(&octree, 1).size(), 16);
        assert_eq!(OctreeRef::new(&octree, 9).size(), 8);
        assert_eq!(OctreeRef::new(&octree, 73).size(), 4);
        assert_eq!(OctreeRef::new(&octree, 585).size(), 2);
    }

    #[test]
    fn octree_ref_pos_first() {
        let chunk = Chunk::default();

        let r = chunk.get_ref(0);
        let mut parent = r.parent(&chunk.octree);

        loop {
            assert_eq!(parent.pos(), U8Vec3::default());

            match parent.parent() {
                Some(p) => parent = p,
                None => break,
            }
        }
    }

    #[test]
    fn octree_ref_pos_last() {
        let chunk = Chunk::default();

        let r = chunk.get_ref_pos(U8Vec3::splat(31));
        let mut parent = r.parent(&chunk.octree);

        // Depth 4
        assert_eq!(parent.pos(), U8Vec3::splat(30));
        parent = parent.parent().unwrap();
        // Depth 3
        assert_eq!(parent.pos(), U8Vec3::splat(28));
        parent = parent.parent().unwrap();
        // Depth 2
        assert_eq!(parent.pos(), U8Vec3::splat(24));
        parent = parent.parent().unwrap();
        // Depth 1
        assert_eq!(parent.pos(), U8Vec3::splat(16));
        parent = parent.parent().unwrap();
        // Depth 0
        assert_eq!(parent.pos(), U8Vec3::splat(0));
        assert!(parent.parent().is_none());
    }

    #[test]
    fn octree_ref_pos_first_not_straight() {
        let chunk = Chunk::default();

        let r = chunk.get_ref_pos(U8Vec3::splat(2));
        let mut parent = r.parent(&chunk.octree);

        // Depth 4
        assert_eq!(parent.pos(), U8Vec3::splat(2), "{parent:?}");
        parent = parent.parent().unwrap();

        loop {
            assert_eq!(parent.pos(), U8Vec3::default(), "{parent:?}");

            match parent.parent() {
                Some(p) => parent = p,
                None => break,
            }
        }
    }

    #[test]
    fn octree_ref_pos_first_not_straight_specific() {
        assert_eq!(
            OctreeRef::new(
                &Octree::default(),
                Octree::pos_to_node_index(U8Vec3::splat(2), 3)
            )
            .pos(),
            U8Vec3::splat(0)
        );
    }

    #[test]
    fn octree_ref_pos_last_not_straight() {
        let chunk = Chunk::default();

        let r = chunk.get_ref_pos(U8Vec3::splat(29));
        let mut parent = r.parent(&chunk.octree);

        // Depth 4
        assert_eq!(parent.pos(), U8Vec3::splat(28), "{parent:?}");
        parent = parent.parent().unwrap();
        // Depth 3
        assert_eq!(parent.pos(), U8Vec3::splat(28), "{parent:?}");
        parent = parent.parent().unwrap();
        // Depth 2
        assert_eq!(parent.pos(), U8Vec3::splat(24), "{parent:?}");
        parent = parent.parent().unwrap();
        // Depth 1
        assert_eq!(parent.pos(), U8Vec3::splat(16), "{parent:?}");
        parent = parent.parent().unwrap();
        // Depth 0
        assert_eq!(parent.pos(), U8Vec3::splat(0), "{parent:?}");
        assert!(parent.parent().is_none());
    }
}
