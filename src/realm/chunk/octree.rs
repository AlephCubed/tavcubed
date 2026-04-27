pub mod debug;
mod reference;

pub use reference::*;

use crate::debug_asset_valid_voxel_pos;
use crate::realm::chunk::voxel::Voxel;
use crate::realm::chunk::{Chunk, DIAMETER, STRIDE_X, STRIDE_Y, STRIDE_Z, VoxelBuffer};
use bevy::math::U8Vec3;
use bitflags::bitflags;
use std::collections::HashMap;
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
    pub fn new(voxels: &VoxelBuffer) -> Self {
        let mut octree = Octree::default();

        for depth in (0..=OCTREE_DEPTH).rev() {
            let start = Self::depth_first_index(depth as u32);
            let size = Self::depth_size(depth as u32);

            for node in start..(start + size) {
                octree.update_node(node, voxels);
            }
        }

        octree
    }

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
    fn pos_to_leaf_index(pos: U8Vec3) -> usize {
        debug_asset_valid_voxel_pos!(pos);

        let mut index = 0;

        for depth in 0..OCTREE_DEPTH {
            let children_voxel_diameter = Self::depth_voxel_diameter(depth as u32 + 1);

            let cx = (pos.x as usize / children_voxel_diameter) & 1;
            let cy = (pos.y as usize / children_voxel_diameter) & 1;
            let cz = (pos.z as usize / children_voxel_diameter) & 1;
            let child_index = cx | (cy << 1) | (cz << 2);

            index = Self::child_index(index) + child_index;
        }

        index
    }

    /// Converts a node index at any layer into the voxel position of its first (minimum corner) voxel.
    #[inline]
    fn node_index_to_pos(index: usize) -> U8Vec3 {
        // Todo optimize.
        if index == 0 {
            return U8Vec3::ZERO;
        }

        let depth = Self::get_depth(index);
        let voxel_diameter = Self::depth_voxel_diameter(depth);
        let child_index = Self::depth_relative_index(index) % 8;

        let local = U8Vec3::new(
            (child_index & 1) as u8,
            ((child_index >> 1) & 1) as u8,
            ((child_index >> 2) & 1) as u8,
        ) * U8Vec3::splat(voxel_diameter as u8);

        Self::node_index_to_pos(Self::parent_index(index)) + local
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

    pub(super) fn update(&mut self, voxel: usize, voxels: &VoxelBuffer) {
        let mut current = Self::pos_to_leaf_index(Chunk::index_to_pos(voxel));

        while self.update_node(current, voxels) && current != 0 {
            current = Self::parent_index(current);
        }
    }

    fn update_node(&mut self, node: usize, voxels: &VoxelBuffer) -> bool {
        let node = OctreeRef::new(self, node);

        let mut counts = HashMap::<Option<Voxel>, u8>::with_capacity(8);
        let mut non_empty_count = 0;
        let mut max = (None, 0u8);
        let mut uniform = true;

        for r in node.children(voxels) {
            if r.voxel().is_none() {
                continue;
            }

            let count = counts.entry(r.voxel()).or_insert(0);
            *count += 1;

            non_empty_count += 1;

            if *count > max.1 {
                max = (r.voxel(), *count);
            }

            if !r.flags().contains(OctreeNodeFlag::UNIFORM) {
                uniform = false;
            }
        }

        let mut flags = OctreeNodeFlag::empty();

        flags.set(
            OctreeNodeFlag::UNIFORM,
            uniform && (max.1 == 8 || non_empty_count == 0),
        );
        flags.set(
            OctreeNodeFlag::MINORITY,
            non_empty_count > 0 && non_empty_count < 4,
        );

        let new_node = OctreeNode {
            voxel: max.0,
            flags,
        };

        if new_node == node.node() {
            false
        } else {
            self.buffer[node.index] = new_node;
            true
        }
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
            buffer: [OctreeNode {
                voxel: None,
                flags: OctreeNodeFlag::UNIFORM,
            }; OCTREE_NODE_COUNT],
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
    use crate::realm::chunk::CHUNK_VOXEL_COUNT;
    use bevy::math::u8vec3;

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
    fn pos_to_leaf_index() {
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
    fn node_index_to_pos_leaf_x() {
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
    fn node_index_to_pos_leaf_y() {
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
    fn node_index_to_pos_leaf_z() {
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
    fn node_index_to_pos_leaf_last() {
        assert_eq!(
            Octree::node_index_to_pos(OCTREE_NODE_COUNT - 1),
            U8Vec3::new(30, 30, 30)
        );
    }

    #[test]
    fn node_index_to_pos_root() {
        assert_eq!(Octree::node_index_to_pos(0), U8Vec3::new(0, 0, 0));
    }

    #[test]
    fn node_index_to_pos_depth_1() {
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

    #[test]
    fn update_empty_uniform() {
        let mut chunk = Chunk::default();

        chunk.octree.update(0, &chunk.buffer);

        for depth in 0..OCTREE_DEPTH {
            assert_eq!(
                chunk.octree.buffer[Octree::depth_first_index(depth as u32)],
                OctreeNode {
                    voxel: None,
                    flags: OctreeNodeFlag::UNIFORM
                },
                "depth: {depth}"
            );
        }
    }

    #[test]
    fn update_empty_uniform_using_parent() {
        for i in 0..CHUNK_VOXEL_COUNT {
            let mut chunk = Chunk::default();

            chunk.octree.update(i, &chunk.buffer);

            let r = chunk.get_ref(i);
            let mut parent = r.parent(&chunk.octree);

            loop {
                assert_eq!(
                    parent.node(),
                    OctreeNode {
                        voxel: None,
                        flags: OctreeNodeFlag::UNIFORM
                    },
                    "Current ref: {parent:?}"
                );

                match parent.parent() {
                    Some(p) => parent = p,
                    None => break,
                }
            }
        }
    }

    #[test]
    fn update_full_uniform() {
        let mut chunk = Chunk::full(Voxel::default());

        chunk.octree.update(0, &chunk.buffer);

        for depth in 0..OCTREE_DEPTH {
            assert_eq!(
                chunk.octree.buffer[Octree::depth_first_index(depth as u32)],
                OctreeNode {
                    voxel: Some(Voxel::default()),
                    flags: OctreeNodeFlag::UNIFORM
                },
                "depth: {depth}"
            );
        }
    }

    #[test]
    fn update_full_uniform_using_parent() {
        for i in 0..CHUNK_VOXEL_COUNT {
            let mut chunk = Chunk::full(Voxel::default());

            chunk.octree.update(i, &chunk.buffer);

            let r = chunk.get_ref(i);
            let mut parent = r.parent(&chunk.octree);

            loop {
                assert_eq!(
                    parent.node(),
                    OctreeNode {
                        voxel: Some(Voxel::default()),
                        flags: OctreeNodeFlag::UNIFORM
                    },
                    "Current ref: {parent:?}"
                );

                match parent.parent() {
                    Some(p) => parent = p,
                    None => break,
                }
            }
        }
    }

    #[test]
    fn update_single_voxel() {
        let mut chunk = Chunk::default();
        chunk.place(0, Voxel::default()).unwrap();

        chunk.octree.update(0, &chunk.buffer);

        for depth in 0..OCTREE_DEPTH {
            assert_eq!(
                chunk.octree.buffer[Octree::depth_first_index(depth as u32)],
                OctreeNode {
                    voxel: Some(Voxel::default()),
                    flags: OctreeNodeFlag::MINORITY
                },
                "depth: {depth}"
            );
        }
    }

    #[test]
    fn update_single_voxel_using_parent() {
        for i in 0..CHUNK_VOXEL_COUNT {
            let mut chunk = Chunk::default();
            chunk.place(i, Voxel::default()).unwrap();

            chunk.octree.update(i, &chunk.buffer);

            let r = chunk.get_ref(i);
            let mut parent = r.parent(&chunk.octree);

            loop {
                assert_eq!(
                    parent.node(),
                    OctreeNode {
                        voxel: Some(Voxel::default()),
                        flags: OctreeNodeFlag::MINORITY
                    },
                    "Current ref: {parent:?}"
                );

                match parent.parent() {
                    Some(p) => parent = p,
                    None => break,
                }
            }
        }
    }

    #[test]
    fn update_leaf_node_mixed() {
        let mut chunk = Chunk::checkerboard(Some(Voxel::default()), Some(Voxel::new_unwrap(2)));

        chunk.octree.update(0, &chunk.buffer);
        for depth in 0..OCTREE_DEPTH {
            assert_eq!(
                chunk.octree.buffer[Octree::depth_first_index(depth as u32)],
                OctreeNode {
                    voxel: Some(Voxel::default()),
                    flags: OctreeNodeFlag::empty()
                },
                "depth: {depth}"
            );
        }
    }

    #[test]
    fn update_leaf_node_mixed_2() {
        let mut chunk = Chunk::checkerboard(Some(Voxel::default()), Some(Voxel::new_unwrap(2)));
        chunk.set(0, Some(Voxel::new_unwrap(2)));

        chunk.octree.update(0, &chunk.buffer);
        assert_eq!(
            chunk.octree.buffer[LEAF_START],
            OctreeNode {
                voxel: Some(Voxel::new_unwrap(2)),
                flags: OctreeNodeFlag::empty()
            }
        )
    }

    #[test]
    fn update_leaf_node_mixed_empty() {
        let mut chunk = Chunk::checkerboard(Some(Voxel::default()), None);

        chunk.octree.update(0, &chunk.buffer);
        assert_eq!(
            chunk.octree.buffer[LEAF_START],
            OctreeNode {
                voxel: Some(Voxel::default()),
                flags: OctreeNodeFlag::empty()
            }
        )
    }

    #[test]
    fn update_leaf_node_mixed_minority() {
        let mut chunk = Chunk::default();
        chunk.set(0, Some(Voxel::default()));

        chunk.octree.update(0, &chunk.buffer);
        assert_eq!(
            chunk.octree.buffer[LEAF_START],
            OctreeNode {
                voxel: Some(Voxel::default()),
                flags: OctreeNodeFlag::MINORITY,
            }
        )
    }

    #[test]
    fn update_leaf_node_unique() {
        let mut chunk = Chunk::default();

        for x in 0..=1 {
            for y in 0..=1 {
                for z in 0..=1 {
                    chunk.set_pos(
                        u8vec3(x, y, z),
                        Some(Voxel::new_unwrap((x + 2 * y + 4 * z) as u16 + 1)),
                    );
                }
            }
        }

        chunk.octree.update(0, &chunk.buffer);
        assert_eq!(
            chunk.octree.buffer[LEAF_START],
            OctreeNode {
                voxel: Some(Voxel::default()),
                flags: OctreeNodeFlag::empty(),
            }
        )
    }
}
