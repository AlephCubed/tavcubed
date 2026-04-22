use crate::realm::chunk::{Chunk, OCTREE_DEPTH, Octree, OctreeNode, OctreeNodeFlag, Voxel};
use bevy::math::U8Vec3;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

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
}

impl VoxelGroupRef for DynVoxelGroupRef<'_> {
    defer!(fn voxel(&self) -> Option<Voxel>);
    defer!(fn flags(&self) -> OctreeNodeFlag);
    defer!(fn depth(&self) -> u32);
    defer!(fn pos(&self) -> U8Vec3);
    defer!(fn size(&self) -> u8);
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

    pub fn parent(&self) -> OctreeRef<'a> {
        OctreeRef {
            octree: &self.chunk.octree,
            index: Octree::pos_to_leaf_index(self.pos()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OctreeRef<'a> {
    octree: &'a Octree,
    index: usize,
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

    fn depth(&self) -> u32 {
        Octree::get_depth(self.index)
    }

    fn pos(&self) -> U8Vec3 {
        Octree::node_index_to_pos(self.index)
    }

    fn size(&self) -> u8 {
        2u8.pow(OCTREE_DEPTH as u32 - self.depth() + 1)
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
    ) -> VoxelGroupIter<impl Iterator<Item = ChunkRef<'a>>, impl Iterator<Item = OctreeRef<'a>>>
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
