mod reference;

pub use reference::*;

use crate::debug_asset_valid_voxel_pos;
use crate::realm::chunk::voxel::Voxel;
use crate::realm::chunk::{Chunk, DIAMETER, STRIDE_X, STRIDE_Y, STRIDE_Z};
use bevy::math::U8Vec3;
use bitflags::bitflags;
use std::ops::Range;

pub const OCTREE_DEPTH: usize = DIAMETER / 8;
// 8(8^n - 1)/7 where n is the depth of the tree.
pub const OCTREE_NODE_COUNT: usize = (8 * 8usize.pow(OCTREE_DEPTH as u32) - 1) / 7;

const LEAF_START: usize = Octree::depth_first_index(OCTREE_DEPTH as u32);
const LEAF_DIAMETER: usize = 2usize.pow(OCTREE_DEPTH as u32);

pub type OctreeBuffer = [OctreeNode; OCTREE_NODE_COUNT];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Octree {
    buffer: OctreeBuffer,
}

impl Octree {
    pub const fn full(voxel: Voxel) -> Self {
        Self {
            buffer: [OctreeNode {
                voxel: Some(voxel),
                flags: OctreeNodeFlag::UNIFORM,
            }; OCTREE_NODE_COUNT],
        }
    }

    /// Returns the depth of a node at the given index.
    #[inline(always)]
    const fn get_depth(index: usize) -> u32 {
        // ilog_8(7i + 1)) using log_8(x) = log_2(x)/3
        (7 * index + 1).ilog2() / 3
    }

    /// Returns the index of the first node at the given depth.
    #[inline(always)]
    const fn depth_first_index(depth: u32) -> usize {
        (Self::depth_size(depth) - 1) / 7
    }

    /// Returns the number of nodes in the given depth.
    #[inline(always)]
    const fn depth_size(depth: u32) -> usize {
        8usize.pow(depth)
    }

    /// Returns width of nodes in the given depth.
    #[inline(always)]
    const fn depth_diameter(depth: u32) -> usize {
        2usize.pow(depth)
    }

    /// Returns the number of voxel descendants each node has at the given depth.
    #[inline(always)]
    const fn depth_voxel_size(depth: u32) -> usize {
        2usize.pow((OCTREE_DEPTH as u32 - depth + 1) * 3)
    }

    /// Returns the width in voxels of each node at the given depth.
    #[inline(always)]
    const fn depth_voxel_diameter(depth: u32) -> usize {
        2usize.pow(OCTREE_DEPTH as u32 - depth + 1)
    }

    /// Returns the index of a node relative to its depth.
    #[inline(always)]
    const fn depth_relative_index(index: usize) -> usize {
        // Todo optimize.
        index - Self::depth_first_index(Self::get_depth(index))
    }

    /// Returns the index of the first child of the given node.
    #[inline]
    const fn child_index(index: usize) -> usize {
        // Todo optimize.
        let depth = Self::get_depth(index);
        let offset = Self::depth_relative_index(index);
        Self::depth_first_index(depth + 1) + offset * 8
    }

    /// Returns the range of indices of the given node's children.
    #[inline]
    const fn children_indices(index: usize) -> Range<usize> {
        let index = Self::child_index(index);
        index..(index + 8)
    }

    /// Returns an iterator over the children of the given node.
    #[inline]
    fn children(&self, index: usize) -> impl Iterator<Item = &OctreeNode> {
        Self::children_indices(index).map(|i| &self.buffer[i])
    }

    /// Returns the index of the parent of the given node.
    /// # Panics
    /// If the node is root (0).
    #[inline]
    fn parent_index(index: usize) -> usize {
        debug_assert_ne!(index, 0, "Root node (0) has no parent");
        // Todo optimize.
        let depth = Self::get_depth(index);
        let offset = Self::depth_relative_index(index);
        Self::depth_first_index(depth - 1) + offset / 8
    }

    /// Returns the parent of the given node.
    #[inline]
    fn parent(&self, index: usize) -> &OctreeNode {
        &self.buffer[Self::parent_index(index)]
    }

    /// Converts a voxel position to its parents octree node index.
    #[inline]
    fn pos_to_leaf_index(mut pos: U8Vec3) -> usize {
        debug_asset_valid_voxel_pos!(pos);
        pos /= U8Vec3::splat(2);
        LEAF_START
            + pos.z as usize * (STRIDE_Z / 4)
            + pos.y as usize * (STRIDE_Y / 2)
            + pos.x as usize
    }

    /// Converts a node index at any layer into the voxel position of its first (minimum corner) voxel.
    #[inline]
    fn node_index_to_pos(index: usize) -> U8Vec3 {
        let depth = Self::get_depth(index);
        let offset = Self::depth_relative_index(index);
        let grid_size = LEAF_DIAMETER >> (OCTREE_DEPTH as u32 - depth); // 2^depth nodes per axis.
        let voxel_size = DIAMETER as u32 >> depth; // Voxels per node side.

        U8Vec3::new(
            (offset % grid_size) as u8,
            ((offset / grid_size) % grid_size) as u8,
            (offset / (grid_size * grid_size)) as u8,
        ) * U8Vec3::splat(voxel_size as u8)
    }

    /// Returns a voxel index iterator including all voxels that are descendants of the given node.
    fn iter_voxel_indices(index: usize) -> impl Iterator<Item = usize> {
        let depth = Self::get_depth(index);
        let size = Octree::depth_voxel_size(depth);
        let start = Chunk::pos_to_index(Octree::node_index_to_pos(index));
        let d = Octree::depth_voxel_diameter(depth);

        (0..size).map(move |i| {
            let x = (i % d) * STRIDE_X;
            let y = (i / d) % d * STRIDE_Y;
            let z = (i / (d * d)) * STRIDE_Z;
            start + x + y + z
        })
    }

    /// Returns a reference to the root node.
    #[inline]
    pub fn root(&self) -> OctreeRef<'_> {
        OctreeRef::new(self, 0)
    }

    /// Returns a reference to the parent of the voxel at the given position.
    #[inline]
    pub fn get_leaf_pos(&self, pos: U8Vec3) -> OctreeRef<'_> {
        OctreeRef::new(self, Self::pos_to_leaf_index(pos))
    }

    /// Returns an iterator over all nodes at a given depth in the tree.
    ///
    /// # Panics
    /// Panics if `depth` is greater than or equal to [`OCTREE_DEPTH`].
    pub fn iter_depth(&self, depth: u32) -> impl Iterator<Item = OctreeRef<'_>> {
        debug_assert!(
            depth <= OCTREE_DEPTH as u32,
            "Depth must be less than {OCTREE_DEPTH}, got {depth}"
        );

        let start = Self::depth_first_index(depth);
        let end = start + Self::depth_size(depth);

        (start..end).map(|i| OctreeRef::new(self, i))
    }
}

impl Default for Octree {
    fn default() -> Self {
        Self {
            buffer: [OctreeNode::default(); OCTREE_NODE_COUNT],
        }
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct OctreeNode {
    voxel: Option<Voxel>,
    flags: OctreeNodeFlag,
}

impl OctreeNode {
    pub fn voxel(&self) -> Option<Voxel> {
        self.voxel
    }

    pub fn flags(&self) -> OctreeNodeFlag {
        self.flags
    }
}

impl From<Option<Voxel>> for OctreeNode {
    fn from(value: Option<Voxel>) -> Self {
        Self {
            voxel: value,
            flags: OctreeNodeFlag::default(),
        }
    }
}

bitflags! {
    #[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct OctreeNodeFlag: u8 {
        const UNIFORM = 1 << 0;
        const MINORITY = 1 << 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_depth_0() {
        assert_eq!(Octree::get_depth(0), 0);
    }

    #[test]
    fn get_depth_1() {
        for i in 1..9 {
            assert_eq!(Octree::get_depth(i), 1);
        }
    }

    #[test]
    fn get_depth_2() {
        for i in 9..73 {
            assert_eq!(Octree::get_depth(i), 2);
        }
    }

    #[test]
    fn depth_first_index() {
        assert_eq!(Octree::depth_first_index(0), 0);
        assert_eq!(Octree::depth_first_index(1), 1);
        assert_eq!(Octree::depth_first_index(2), 9);
        assert_eq!(Octree::depth_first_index(3), 73);
    }

    #[test]
    fn relative_index_0() {
        assert_eq!(Octree::depth_relative_index(0), 0);
    }

    #[test]
    fn relative_index_1() {
        for i in 1..9 {
            assert_eq!(Octree::depth_relative_index(i), i - 1);
        }
    }

    #[test]
    fn relative_index_2() {
        for i in 9..73 {
            assert_eq!(Octree::depth_relative_index(i), i - 9);
        }
    }

    #[test]
    fn child_index_0() {
        assert_eq!(Octree::child_index(0), 1);
    }

    #[test]
    fn child_index_1() {
        for i in 1..9 {
            assert_eq!(Octree::child_index(i), i * 8 + 1);
        }
    }

    #[test]
    fn child_index_2() {
        assert_eq!(Octree::child_index(9), 73);
        assert_eq!(Octree::child_index(72), 73 + 512 - 8);
    }

    #[test]
    #[should_panic(expected = "Root node (0) has no parent")]
    fn parent_index_0() {
        Octree::parent_index(0);
    }

    #[test]
    fn parent_index_1() {
        for i in 1..9 {
            assert_eq!(Octree::parent_index(i), 0);
        }
    }

    #[test]
    fn parent_index_2() {
        for i in 9..(9 + 8) {
            assert_eq!(Octree::parent_index(i), 1);
        }
        for i in (73 - 8)..73 {
            assert_eq!(Octree::parent_index(i), 8);
        }
    }

    #[test]
    fn pos_to_leaf() {
        assert_eq!(Octree::pos_to_leaf_index(U8Vec3::new(0, 0, 0)), LEAF_START);
        assert_eq!(Octree::pos_to_leaf_index(U8Vec3::new(1, 0, 0)), LEAF_START);
        assert_eq!(Octree::pos_to_leaf_index(U8Vec3::new(0, 1, 0)), LEAF_START);
        assert_eq!(Octree::pos_to_leaf_index(U8Vec3::new(1, 1, 0)), LEAF_START);
        assert_eq!(Octree::pos_to_leaf_index(U8Vec3::new(0, 0, 1)), LEAF_START);
        assert_eq!(Octree::pos_to_leaf_index(U8Vec3::new(1, 0, 1)), LEAF_START);
        assert_eq!(Octree::pos_to_leaf_index(U8Vec3::new(0, 1, 1)), LEAF_START);
        assert_eq!(Octree::pos_to_leaf_index(U8Vec3::new(1, 1, 1)), LEAF_START);
    }

    #[test]
    fn leaf_to_pos_x() {
        assert_eq!(Octree::node_index_to_pos(LEAF_START), U8Vec3::new(0, 0, 0));

        assert_eq!(
            Octree::node_index_to_pos(LEAF_START + 1),
            U8Vec3::new(2, 0, 0)
        );
        assert_eq!(
            Octree::node_index_to_pos(LEAF_START + 2),
            U8Vec3::new(4, 0, 0)
        );
        assert_eq!(
            Octree::node_index_to_pos(LEAF_START + 3),
            U8Vec3::new(6, 0, 0)
        );
    }

    #[test]
    fn leaf_to_pos_y() {
        assert_eq!(
            Octree::node_index_to_pos(LEAF_START + LEAF_DIAMETER),
            U8Vec3::new(0, 2, 0)
        );
        assert_eq!(
            Octree::node_index_to_pos(LEAF_START + LEAF_DIAMETER * 2),
            U8Vec3::new(0, 4, 0)
        );
        assert_eq!(
            Octree::node_index_to_pos(LEAF_START + LEAF_DIAMETER * 3),
            U8Vec3::new(0, 6, 0)
        );
    }

    #[test]
    fn leaf_to_pos_z() {
        assert_eq!(
            Octree::node_index_to_pos(LEAF_START + LEAF_DIAMETER * LEAF_DIAMETER),
            U8Vec3::new(0, 0, 2)
        );
        assert_eq!(
            Octree::node_index_to_pos(LEAF_START + LEAF_DIAMETER * LEAF_DIAMETER * 2),
            U8Vec3::new(0, 0, 4)
        );
        assert_eq!(
            Octree::node_index_to_pos(LEAF_START + LEAF_DIAMETER * LEAF_DIAMETER * 3),
            U8Vec3::new(0, 0, 6)
        );
    }

    #[test]
    fn leaf_to_pos_last() {
        assert_eq!(
            Octree::node_index_to_pos(OCTREE_NODE_COUNT - 1),
            U8Vec3::new(30, 30, 30)
        );
    }

    #[test]
    fn leaf_to_pos_root() {
        assert_eq!(Octree::node_index_to_pos(0), U8Vec3::new(0, 0, 0));
    }

    #[test]
    fn leaf_to_pos_depth_1() {
        assert_eq!(Octree::node_index_to_pos(1), U8Vec3::new(00, 00, 00));
        assert_eq!(Octree::node_index_to_pos(2), U8Vec3::new(16, 00, 00));
        assert_eq!(Octree::node_index_to_pos(3), U8Vec3::new(00, 16, 00));
        assert_eq!(Octree::node_index_to_pos(4), U8Vec3::new(16, 16, 00));
        assert_eq!(Octree::node_index_to_pos(5), U8Vec3::new(00, 00, 16));
        assert_eq!(Octree::node_index_to_pos(6), U8Vec3::new(16, 00, 16));
        assert_eq!(Octree::node_index_to_pos(7), U8Vec3::new(00, 16, 16));
        assert_eq!(Octree::node_index_to_pos(8), U8Vec3::new(16, 16, 16));
    }

    #[test]
    fn depth_measurements() {
        assert_eq!(Octree::depth_size(0), 1);
        assert_eq!(Octree::depth_size(1), 8);
        assert_eq!(Octree::depth_size(2), 64);
        assert_eq!(Octree::depth_size(3), 512);

        assert_eq!(Octree::depth_diameter(0), 1);
        assert_eq!(Octree::depth_diameter(1), 2);
        assert_eq!(Octree::depth_diameter(2), 4);
        assert_eq!(Octree::depth_diameter(3), 8);
    }

    #[test]
    fn octree_ref_voxel_indices() {
        assert_eq!(
            Octree::iter_voxel_indices(LEAF_START).collect::<Vec<_>>(),
            vec![0, 1, 32, 33, 1024, 1025, 1056, 1057]
        );
    }
}
