use crate::realm::chunk::{
    Chunk, DIAMETER, OCTREE_DEPTH, Octree, OctreeNode, OctreeNodeFlag, STRIDE_X, STRIDE_Y,
    STRIDE_Z, Voxel,
};
use bevy::math::U8Vec3;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

macro_rules! define_dir_fn {
    ($ident:ident, $is_some:ident, $is_none:ident) => {
        fn $ident(&self) -> Option<Self>
        where
            Self: Sized;

        fn $is_some(&self) -> bool
        where
            Self: Sized,
        {
            self.$ident().map(|r| r.voxel().is_some()).unwrap_or(false)
        }

        fn $is_none(&self) -> bool
        where
            Self: Sized,
        {
            self.$ident().map(|r| r.voxel().is_none()).unwrap_or(false)
        }
    };
}

pub trait VoxelGroupRef {
    fn voxel(&self) -> Option<Voxel>;

    fn flags(&self) -> OctreeNodeFlag;

    fn depth(&self) -> u32;

    fn is_voxel(&self) -> bool {
        self.depth() == OCTREE_DEPTH as u32
    }

    fn is_node(&self) -> bool {
        !self.is_voxel()
    }

    fn is_root_node(&self) -> bool {
        self.depth() == 0
    }

    fn is_leaf_node(&self) -> bool {
        self.depth() == (OCTREE_DEPTH as u32)
    }

    /// The chunk-space voxel position of the group's first (minimum corner) voxel.
    fn pos(&self) -> U8Vec3;

    fn size(&self) -> u8;

    define_dir_fn!(right, right_is_some, right_is_none);
    define_dir_fn!(left, left_is_some, left_is_none);
    define_dir_fn!(up, up_is_some, up_is_none);
    define_dir_fn!(down, down_is_some, down_is_none);
    define_dir_fn!(backward, backward_is_some, backward_is_none);
    define_dir_fn!(forward, forward_is_some, forward_is_none);
}

pub enum VoxelGroupIter<'a, C, O>
where
    C: Iterator<Item = ChunkRef<'a>>,
    O: Iterator<Item = OctreeRef<'a>>,
{
    Chunk(C),
    Octree(O),
}

impl<'a, C, O> Iterator for VoxelGroupIter<'a, C, O>
where
    C: Iterator<Item = ChunkRef<'a>>,
    O: Iterator<Item = OctreeRef<'a>>,
{
    type Item = DynVoxelGroupRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Chunk(it) => it.next().map(DynVoxelGroupRef::Chunk),
            Self::Octree(it) => it.next().map(DynVoxelGroupRef::Octree),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynVoxelGroupRef<'a> {
    Chunk(ChunkRef<'a>),
    Octree(OctreeRef<'a>),
}

macro_rules! defer {
    (fn $name:ident(&self) -> $ty:ty) => {
        fn $name(&self) -> $ty {
            match self {
                DynVoxelGroupRef::Chunk(r) => r.$name(),
                DynVoxelGroupRef::Octree(r) => r.$name(),
            }
        }
    };
    (map fn $name:ident(&self) -> $ty:ty) => {
        fn $name(&self) -> $ty {
            match self {
                DynVoxelGroupRef::Chunk(r) => r.$name().map(DynVoxelGroupRef::Chunk),
                DynVoxelGroupRef::Octree(r) => r.$name().map(DynVoxelGroupRef::Octree),
            }
        }
    };
}

impl VoxelGroupRef for DynVoxelGroupRef<'_> {
    defer!(fn voxel(&self) -> Option<Voxel>);
    defer!(fn flags(&self) -> OctreeNodeFlag);
    defer!(fn depth(&self) -> u32);
    defer!(fn pos(&self) -> U8Vec3);
    defer!(fn size(&self) -> u8);
    defer!(map fn right(&self) -> Option<Self>);
    defer!(map fn left(&self) -> Option<Self>);
    defer!(map fn up(&self) -> Option<Self>);
    defer!(map fn down(&self) -> Option<Self>);
    defer!(map fn backward(&self) -> Option<Self>);
    defer!(map fn forward(&self) -> Option<Self>);
}

impl Debug for DynVoxelGroupRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DynVoxelGroupRef::Chunk(r) => write!(f, "Dyn{r:?}"),
            DynVoxelGroupRef::Octree(r) => write!(f, "Dyn{r:?}"),
        }
    }
}

impl<'a> From<ChunkRef<'a>> for DynVoxelGroupRef<'a> {
    fn from(octree: ChunkRef<'a>) -> Self {
        DynVoxelGroupRef::Chunk(octree)
    }
}

impl<'a> From<OctreeRef<'a>> for DynVoxelGroupRef<'a> {
    fn from(octree: OctreeRef<'a>) -> Self {
        DynVoxelGroupRef::Octree(octree)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkRef<'a> {
    chunk: &'a Chunk,
    index: usize,
}

impl Debug for ChunkRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChunkRef<{:p}>[{}]", &self.chunk, self.index)
    }
}

impl<'a> Deref for ChunkRef<'a> {
    type Target = Option<Voxel>;

    fn deref(&self) -> &Self::Target {
        &self.chunk[self.index]
    }
}

#[expect(clippy::manual_is_multiple_of)]
impl VoxelGroupRef for ChunkRef<'_> {
    fn voxel(&self) -> Option<Voxel> {
        self.chunk[self.index]
    }

    fn flags(&self) -> OctreeNodeFlag {
        OctreeNodeFlag::UNIFORM
    }

    fn depth(&self) -> u32 {
        OCTREE_DEPTH as u32 + 1
    }

    fn pos(&self) -> U8Vec3 {
        Chunk::index_to_pos(self.index)
    }

    fn size(&self) -> u8 {
        1
    }

    fn right(&self) -> Option<Self> {
        (self.index % STRIDE_Y < DIAMETER - 1).then(|| self.chunk.get_ref(self.index + STRIDE_X))
    }

    fn left(&self) -> Option<Self> {
        (self.index % STRIDE_Y > 0).then(|| self.chunk.get_ref(self.index - STRIDE_X))
    }

    fn up(&self) -> Option<Self> {
        ((self.index / STRIDE_Y) % STRIDE_Y < DIAMETER - 1)
            .then(|| self.chunk.get_ref(self.index + STRIDE_Y))
    }

    fn down(&self) -> Option<Self> {
        ((self.index / STRIDE_Y) % STRIDE_Y > 0).then(|| self.chunk.get_ref(self.index - STRIDE_Y))
    }

    fn backward(&self) -> Option<Self> {
        (self.index / STRIDE_Z < DIAMETER - 1).then(|| self.chunk.get_ref(self.index + STRIDE_Z))
    }

    fn forward(&self) -> Option<Self> {
        (self.index / STRIDE_Z > 0).then(|| self.chunk.get_ref(self.index - STRIDE_Z))
    }
}

impl<'a> ChunkRef<'a> {
    pub fn new(chunk: &'a Chunk, index: usize) -> Self {
        Self { chunk, index }
    }

    pub fn chunk(self) -> &'a Chunk {
        self.chunk
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn parent(self) -> OctreeRef<'a> {
        OctreeRef {
            octree: &self.chunk.octree,
            index: Octree::pos_to_leaf_index(self.pos()),
        }
    }
}

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

const SIBLING_STRIDE_X: usize = 1;
const SIBLING_STRIDE_Y: usize = 2;
const SIBLING_STRIDE_Z: usize = 4;

#[expect(clippy::manual_is_multiple_of)]
impl VoxelGroupRef for OctreeRef<'_> {
    fn voxel(&self) -> Option<Voxel> {
        self.node().voxel()
    }

    fn flags(&self) -> OctreeNodeFlag {
        self.node().flags()
    }

    fn depth(&self) -> u32 {
        Octree::get_depth(self.index)
    }

    fn pos(&self) -> U8Vec3 {
        Octree::node_index_to_pos(self.index)
    }

    fn size(&self) -> u8 {
        2u8.pow(OCTREE_DEPTH as u32 - self.depth() + 1)
    }

    fn right(&self) -> Option<Self> {
        if self.is_root_node() {
            return None;
        }

        let index = Octree::depth_relative_index(self.index);
        let depth_size = Octree::depth_size(self.depth());
        let cousin_stride_x = 8 - SIBLING_STRIDE_X;

        if index % 2 == 0 {
            // Sibling
            Some(Self::new(self.octree, self.index + SIBLING_STRIDE_X))
        } else if index % (depth_size / 4) < depth_size / 8 {
            // Cousin
            Some(Self::new(self.octree, self.index + cousin_stride_x))
        } else {
            None
        }
    }

    fn left(&self) -> Option<Self> {
        if self.is_root_node() {
            return None;
        }

        let index = Octree::depth_relative_index(self.index);
        let depth_size = Octree::depth_size(self.depth());
        let cousin_stride_x = 8 - SIBLING_STRIDE_X;

        if index % 2 != 0 {
            // Sibling
            Some(Self::new(self.octree, self.index - SIBLING_STRIDE_X))
        } else if index % (depth_size / 4) >= (depth_size / 8) {
            // Cousin
            Some(Self::new(self.octree, self.index - cousin_stride_x))
        } else {
            None
        }
    }

    fn up(&self) -> Option<Self> {
        if self.is_root_node() {
            return None;
        }

        let index = Octree::depth_relative_index(self.index);
        let depth_size = Octree::depth_size(self.depth());
        let cousin_stride_y = 8 * (Octree::depth_diameter(self.depth()) / 2) - SIBLING_STRIDE_Y;

        if (index / 2) % 2 == 0 {
            // Sibling
            Some(Self::new(self.octree, self.index + SIBLING_STRIDE_Y))
        } else if index % (depth_size / 2) < depth_size / 4 {
            // Cousin
            Some(Self::new(self.octree, self.index + cousin_stride_y))
        } else {
            None
        }
    }

    fn down(&self) -> Option<Self> {
        if self.is_root_node() {
            return None;
        }

        let index = Octree::depth_relative_index(self.index);
        let depth_size = Octree::depth_size(self.depth());
        let cousin_stride_y = 8 * (Octree::depth_diameter(self.depth()) / 2) - SIBLING_STRIDE_Y;

        if (index / 2) % 2 != 0 {
            // Sibling
            Some(Self::new(self.octree, self.index - SIBLING_STRIDE_Y))
        } else if index % (depth_size / 2) >= depth_size / 4 {
            // Cousin
            Some(Self::new(self.octree, self.index - cousin_stride_y))
        } else {
            None
        }
    }

    fn backward(&self) -> Option<Self> {
        if self.is_root_node() {
            return None;
        }

        let index = Octree::depth_relative_index(self.index);
        let depth_size = Octree::depth_size(self.depth());
        let cousin_stride_z = 8 * Octree::depth_diameter(self.depth()) - SIBLING_STRIDE_Z;

        if (index / 4) % 2 == 0 {
            // Sibling
            Some(Self::new(self.octree, self.index + SIBLING_STRIDE_Z))
        } else if index % depth_size < depth_size / 2 {
            // Cousin
            Some(Self::new(self.octree, self.index + cousin_stride_z))
        } else {
            None
        }
    }

    fn forward(&self) -> Option<Self> {
        if self.is_root_node() {
            return None;
        }

        let index = Octree::depth_relative_index(self.index);
        let depth_size = Octree::depth_size(self.depth());
        let cousin_stride_z = 8 * Octree::depth_diameter(self.depth()) - SIBLING_STRIDE_Z;

        if (index / 4) % 2 != 0 {
            // Sibling
            Some(Self::new(self.octree, self.index - SIBLING_STRIDE_Z))
        } else if index % depth_size >= depth_size / 2 {
            // Cousin
            Some(Self::new(self.octree, self.index - cousin_stride_z))
        } else {
            None
        }
    }
}

impl<'a> OctreeRef<'a> {
    pub(super) fn new(octree: &'a Octree, index: usize) -> Self {
        OctreeRef { octree, index }
    }

    pub fn node(&self) -> OctreeNode {
        self.octree.buffer[self.index]
    }

    pub fn octree(&self) -> &Octree {
        self.octree
    }

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
        chunk: &'a Chunk,
    ) -> VoxelGroupIter<'a, impl Iterator<Item = ChunkRef<'a>>, impl Iterator<Item = OctreeRef<'a>>>
    {
        match self.is_leaf_node() {
            true => VoxelGroupIter::Chunk(self.iter_voxels(chunk)),
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

    /// Returns a [`ChunkRef`] iterator including all voxels that are descendants of the current node.
    pub fn iter_voxels(&'a self, chunk: &'a Chunk) -> impl Iterator<Item = ChunkRef<'a>> {
        Octree::iter_voxel_indices(self.index).map(|i| chunk.get_ref(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm::chunk::OCTREE_NODE_COUNT;
    use crate::realm::chunk::octree::{LEAF_DIAMETER, LEAF_START};

    #[test]
    fn chunk_ref_right() {
        let chunk = Chunk::default();
        let r = chunk.get_ref_pos(U8Vec3::default());
        assert_eq!(r.right().unwrap().pos(), U8Vec3::X);
    }

    #[test]
    fn chunk_ref_left() {
        let chunk = Chunk::default();
        let r = chunk.get_ref_pos(U8Vec3::X);
        assert_eq!(r.left().unwrap().pos(), U8Vec3::default());
    }

    #[test]
    fn chunk_ref_up() {
        let chunk = Chunk::default();
        let r = chunk.get_ref_pos(U8Vec3::default());
        assert_eq!(r.up().unwrap().pos(), U8Vec3::Y);
    }

    #[test]
    fn chunk_ref_down() {
        let chunk = Chunk::default();
        let r = chunk.get_ref_pos(U8Vec3::Y);
        assert_eq!(r.down().unwrap().pos(), U8Vec3::default());
    }

    #[test]
    fn chunk_ref_backward() {
        let chunk = Chunk::default();
        let r = chunk.get_ref_pos(U8Vec3::default());
        assert_eq!(r.backward().unwrap().pos(), U8Vec3::Z);
    }

    #[test]
    fn chunk_ref_forward() {
        let chunk = Chunk::default();
        let r = chunk.get_ref_pos(U8Vec3::Z);
        assert_eq!(r.forward().unwrap().pos(), U8Vec3::default());
    }

    #[test]
    fn octree_ref_right() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 1);
        assert_eq!(r.right().unwrap().index, 2);
    }

    #[test]
    fn octree_ref_left() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 2);
        assert_eq!(r.left().unwrap().index, 1);
    }

    #[test]
    fn octree_ref_up() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 1);
        assert_eq!(r.up().unwrap().index, 3);
    }

    #[test]
    fn octree_ref_down() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 3);
        assert_eq!(r.down().unwrap().index, 1);
    }

    #[test]
    fn octree_ref_backward() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 1);
        assert_eq!(r.backward().unwrap().index, 5);
    }

    #[test]
    fn octree_ref_forward() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 5);
        assert_eq!(r.forward().unwrap().index, 1);
    }

    #[test]
    fn octree_ref_right_across_parents() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 1);
        assert_eq!(r.right().unwrap().index, 9 + 8);
    }

    #[test]
    fn octree_ref_left_across_parents() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 8);
        assert_eq!(r.left().unwrap().index, 9 + 1);
    }

    #[test]
    fn octree_ref_up_across_parents() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 2);
        assert_eq!(r.up().unwrap().index, 9 + 16);
    }

    #[test]
    fn octree_ref_down_across_parents() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 16);
        assert_eq!(r.down().unwrap().index, 9 + 2);
    }

    #[test]
    fn octree_ref_backward_across_parents() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 4);
        assert_eq!(r.backward().unwrap().index, 9 + 32);
    }

    #[test]
    fn octree_ref_forward_across_parents() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 32);
        assert_eq!(r.forward().unwrap().index, 9 + 4);
    }

    #[test]
    fn octree_ref_right_across_parents_2() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 3);
        assert_eq!(r.right().unwrap().index, 9 + 8 + 2);
    }

    #[test]
    fn octree_ref_left_across_parents_2() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 8 + 2);
        assert_eq!(r.left().unwrap().index, 9 + 3);
    }

    #[test]
    fn octree_ref_up_across_parents_2() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 3);
        assert_eq!(r.up().unwrap().index, 9 + 16 + 1);
    }

    #[test]
    fn octree_ref_down_across_parents_2() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 16 + 1);
        assert_eq!(r.down().unwrap().index, 9 + 3);
    }

    #[test]
    fn octree_ref_backward_across_parents_2() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 5);
        assert_eq!(r.backward().unwrap().index, 9 + 32 + 1);
    }

    #[test]
    fn octree_ref_forward_across_parents_2() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 32 + 1);
        assert_eq!(r.forward().unwrap().index, 9 + 5);
    }

    #[test]
    fn octree_ref_backward_across_parents_3() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 6);
        assert_eq!(r.backward().unwrap().index, 9 + 32 + 2);
    }

    #[test]
    fn octree_ref_forward_across_parents_3() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 32 + 2);
        assert_eq!(r.forward().unwrap().index, 9 + 6);
    }

    #[test]
    fn octree_ref_backward_across_parents_4() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 7);
        assert_eq!(r.backward().unwrap().index, 9 + 32 + 3);
    }

    #[test]
    fn octree_ref_forward_across_parents_4() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 32 + 3);
        assert_eq!(r.forward().unwrap().index, 9 + 7);
    }

    #[test]
    fn octree_ref_right_none() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 8 + 1);
        assert_eq!(r.right(), None);
    }

    #[test]
    fn octree_ref_left_none() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9);
        assert_eq!(r.left(), None);
    }

    #[test]
    fn octree_ref_up_none() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 16 + 2);
        assert_eq!(r.up(), None);
    }

    #[test]
    fn octree_ref_down_none() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9);
        assert_eq!(r.down(), None);
    }

    #[test]
    fn octree_ref_backward_none() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9 + 32 + 4);
        assert_eq!(r.backward(), None);
    }

    #[test]
    fn octree_ref_forward_none() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, 9);
        assert_eq!(r.forward(), None);
    }

    #[test]
    fn octree_ref_backward_bounds() {
        let octree = Octree::default();
        let r = OctreeRef::new(&octree, OCTREE_NODE_COUNT - 124);
        assert_eq!(r.backward(), None);
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
        assert_eq!(OctreeRef::new(&octree, LEAF_START).size(), 2);
    }
}
