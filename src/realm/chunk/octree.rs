use crate::realm::chunk::DIAMETER;
use crate::realm::chunk::voxel::Voxel;
use bitflags::bitflags;

pub const OCTREE_DEPTH: usize = DIAMETER / 8;
// 8(8^n - 1)/7 where n is the depth of the tree.
pub const OCTREE_NODE_COUNT: usize = 8 * (8usize.pow(OCTREE_DEPTH as u32) - 1) / 7;
pub type OctreeBuffer = [OctreeNode; OCTREE_NODE_COUNT];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Octree {
    buffer: OctreeBuffer,
}

impl Octree {
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

    /// Returns the index of a node relative to its depth.
    #[inline(always)]
    const fn relative_index(index: usize) -> usize {
        // Todo optimize.
        index - Self::depth_first_index(Self::get_depth(index))
    }

    /// Returns the index of the first child of the given node.
    #[inline]
    const fn child_index(index: usize) -> usize {
        // Todo optimize.
        let depth = Self::get_depth(index);
        let offset = Self::relative_index(index);
        Self::depth_first_index(depth + 1) + offset * 8
    }

    /// Returns the index of the parent of the given node.
    /// # Panics
    /// If the node is root (0).
    #[inline]
    fn parent_index(index: usize) -> usize {
        debug_assert_ne!(index, 0, "Root node (0) has no parent");
        // Todo optimize.
        let depth = Self::get_depth(index);
        let offset = Self::relative_index(index);
        Self::depth_first_index(depth - 1) + offset / 8
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

bitflags! {
    #[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct OctreeNodeFlag: u8 {
        const UNIFORM_FLAG = 1 << 0;
        const MINORITY_FLAG = 1 << 1;
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
        assert_eq!(Octree::relative_index(0), 0);
    }

    #[test]
    fn relative_index_1() {
        for i in 1..9 {
            assert_eq!(Octree::relative_index(i), i - 1);
        }
    }

    #[test]
    fn relative_index_2() {
        for i in 9..73 {
            assert_eq!(Octree::relative_index(i), i - 9);
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
}
