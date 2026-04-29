//! Fancy pointers to individual voxels or groups of voxels stored in some data structure.

use crate::realm::chunk::{OCTREE_DEPTH, Octree, OctreeNodeFlag, OctreeRef, Voxel, VoxelRef};
use bevy::math::U8Vec3;
use std::fmt::{Debug, Formatter};

macro_rules! define_dir_fn {
    ($ident:ident, $is_some:ident, $is_none:ident) => {
        /// Returns a reference to the next voxel in that direction, at the same depth.
        fn $ident(&self) -> Option<Self>
        where
            Self: Sized;

        /// Returns true if the voxel type of the next reference in that direction is `Some`.
        /// If there is no reference in that direction, returns false.
        fn $is_some(&self) -> bool
        where
            Self: Sized,
        {
            self.$ident().map(|r| r.voxel().is_some()).unwrap_or(false)
        }

        /// Returns true if the voxel type of the next reference in that direction is `None`.
        /// If there is no reference in that direction, returns false.
        fn $is_none(&self) -> bool
        where
            Self: Sized,
        {
            self.$ident().map(|r| r.voxel().is_none()).unwrap_or(false)
        }
    };
}

/// A fancy pointer to an individual voxel or group of voxels stored in some data structure.
pub trait VoxelGroupRef {
    /// Returns a voxel that represents the entire group being referenced.
    fn voxel(&self) -> Option<Voxel>;

    /// Returns the status flags for the voxel group.
    ///
    /// When referencing a [singular voxel](Self::is_singular_voxel), this will always return [`OctreeNodeFlag::UNIFORM`].
    fn flags(&self) -> OctreeNodeFlag;

    /// The depth of the reference in an [`Octree`].
    fn depth(&self) -> usize;

    /// Returns true if referencing an individual voxel (not a group).
    fn is_singular_voxel(&self) -> bool {
        self.depth() == (OCTREE_DEPTH + 1)
    }

    /// Returns true if referencing a group of voxels (not an individual).
    fn is_multiple_voxel(&self) -> bool {
        !self.is_singular_voxel()
    }

    /// Returns true when referencing the root node of an [`Octree`].
    fn is_root_node(&self) -> bool {
        self.depth() == 0
    }

    /// Returns true when referencing a leaf node of an [`Octree`].
    fn is_leaf_node(&self) -> bool {
        self.depth() == OCTREE_DEPTH
    }

    /// The chunk-space voxel position of the group's first (minimum corner) voxel.
    fn pos(&self) -> U8Vec3;

    /// The diameter of the group measured in voxels.
    ///
    /// When referencing a [singular voxel](Self::is_singular_voxel), this will always return 1.
    fn size(&self) -> u8;

    define_dir_fn!(right, right_is_some, right_is_none);
    define_dir_fn!(left, left_is_some, left_is_none);
    define_dir_fn!(up, up_is_some, up_is_none);
    define_dir_fn!(down, down_is_some, down_is_none);
    define_dir_fn!(backward, backward_is_some, backward_is_none);
    define_dir_fn!(forward, forward_is_some, forward_is_none);
}

/// An iterator over some [`VoxelGroupRef`].
pub enum VoxelGroupIter<'a, C, O>
where
    C: Iterator<Item = VoxelRef<'a>>,
    O: Iterator<Item = OctreeRef<'a>>,
{
    Chunk(C),
    Octree(O),
}

impl<'a, C, O> Iterator for VoxelGroupIter<'a, C, O>
where
    C: Iterator<Item = VoxelRef<'a>>,
    O: Iterator<Item = OctreeRef<'a>>,
{
    type Item = DynVoxelGroupRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Chunk(it) => it.next().map(DynVoxelGroupRef::Voxel),
            Self::Octree(it) => it.next().map(DynVoxelGroupRef::Octree),
        }
    }
}

/// A [`VoxelGroupRef`] that is either a singular voxel ([`VoxelRef`]) or group ([`OctreeRef`]).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynVoxelGroupRef<'a> {
    Voxel(VoxelRef<'a>),
    Octree(OctreeRef<'a>),
}

macro_rules! defer {
    (fn $name:ident(&self) -> $ty:ty) => {
        fn $name(&self) -> $ty {
            match self {
                DynVoxelGroupRef::Voxel(r) => r.$name(),
                DynVoxelGroupRef::Octree(r) => r.$name(),
            }
        }
    };
    (map fn $name:ident(&self) -> $ty:ty) => {
        fn $name(&self) -> $ty {
            match self {
                DynVoxelGroupRef::Voxel(r) => r.$name().map(DynVoxelGroupRef::Voxel),
                DynVoxelGroupRef::Octree(r) => r.$name().map(DynVoxelGroupRef::Octree),
            }
        }
    };
}

impl VoxelGroupRef for DynVoxelGroupRef<'_> {
    defer!(fn voxel(&self) -> Option<Voxel>);
    defer!(fn flags(&self) -> OctreeNodeFlag);
    defer!(fn depth(&self) -> usize);
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
            DynVoxelGroupRef::Voxel(r) => write!(f, "Dyn{r:?}"),
            DynVoxelGroupRef::Octree(r) => write!(f, "Dyn{r:?}"),
        }
    }
}

impl<'a> From<VoxelRef<'a>> for DynVoxelGroupRef<'a> {
    fn from(octree: VoxelRef<'a>) -> Self {
        DynVoxelGroupRef::Voxel(octree)
    }
}

impl<'a> From<OctreeRef<'a>> for DynVoxelGroupRef<'a> {
    fn from(octree: OctreeRef<'a>) -> Self {
        DynVoxelGroupRef::Octree(octree)
    }
}
