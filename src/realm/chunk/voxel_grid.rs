mod reference;

pub use reference::*;

use crate::realm::block::VoxelId;
use bevy::math::U8Vec3;
use bevy::prelude::*;
use std::fmt::Formatter;
use std::ops::Index;

pub const DIAMETER: usize = 32;
pub const RADIUS: usize = DIAMETER / 2;

pub const VOXEL_COUNT: usize = DIAMETER * DIAMETER * DIAMETER;
pub const STRIDE_X: usize = 1;
pub const STRIDE_Y: usize = DIAMETER;
pub const STRIDE_Z: usize = DIAMETER * DIAMETER;

type VoxelBuffer = [Option<Voxel>; VOXEL_COUNT];
pub type IntoIter = std::array::IntoIter<Option<Voxel>, VOXEL_COUNT>;
pub type Iter<'a> = core::slice::Iter<'a, Option<Voxel>>;
pub type IterMut<'a> = core::slice::IterMut<'a, Option<Voxel>>;

/// A grid of 32^3 voxels.
///
/// # Performance
/// This is the preferred way to instantiate a [`Chunk`](super::Chunk), as modifications made through the chunk's
/// setters must update the [`Octree`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VoxelGrid {
    buffer: VoxelBuffer,
    len: usize,
}

#[macro_export]
macro_rules! debug_assert_valid_voxel_index {
    ($index:expr) => {
        debug_assert!(
            $index < VOXEL_COUNT,
            "Index must be less than {}, got {}",
            VOXEL_COUNT,
            $index,
        );
    };
}

#[macro_export]
macro_rules! debug_asset_valid_voxel_pos {
    ($pos:expr) => {
        debug_assert!(
            $pos.x < DIAMETER as u8,
            "x position must be less than {}, got {}",
            DIAMETER,
            $pos.x,
        );
        debug_assert!(
            $pos.y < DIAMETER as u8,
            "y position must be less than {}, got {}",
            DIAMETER,
            $pos.y,
        );
        debug_assert!(
            $pos.z < DIAMETER as u8,
            "z position must be less than {}, got {}",
            DIAMETER,
            $pos.z,
        );
    };
}

impl VoxelGrid {
    #[inline(always)]
    pub fn index_to_pos(index: usize) -> U8Vec3 {
        crate::debug_assert_valid_voxel_index!(index);
        U8Vec3 {
            x: (index % STRIDE_Y) as u8,
            y: ((index / STRIDE_Y) % STRIDE_Y) as u8,
            z: (index / STRIDE_Z) as u8,
        }
    }

    #[inline(always)]
    pub fn pos_to_index(pos: U8Vec3) -> usize {
        crate::debug_asset_valid_voxel_pos!(pos);
        (pos.z as usize * STRIDE_Z) + (pos.y as usize * STRIDE_Y) + pos.x as usize
    }

    #[inline(always)]
    pub fn vec_to_pos(vec: impl Into<Vec3>) -> Option<U8Vec3> {
        let vec = vec.into();

        if (vec.x >= 0.0 && vec.x <= 32.0)
            && (vec.y >= 0.0 && vec.y <= 32.0)
            && (vec.z >= 0.0 && vec.z <= 32.0)
        {
            Some(vec.as_u8vec3())
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn vec_to_pos_clamped(vec: impl Into<Vec3>) -> U8Vec3 {
        vec.into()
            .as_u8vec3()
            .clamp(U8Vec3::splat(0), U8Vec3::splat(31))
    }

    /// Creates a new grid from a raw [`VoxelBuffer`].
    #[must_use]
    pub fn new(buffer: VoxelBuffer) -> Self {
        Self {
            buffer,
            len: buffer.iter().filter(|i| i.is_some()).count(),
        }
    }

    /// Creates a grid where every block is set to the given voxel.
    ///
    /// For a grid full of air, use [`VoxelGrid::default()`].
    #[must_use]
    pub fn full(voxel: Voxel) -> Self {
        Self {
            buffer: [Some(voxel); VOXEL_COUNT],
            len: VOXEL_COUNT,
        }
    }

    /// Creates a grid of alternating voxels between `a` and `b`.
    #[must_use]
    pub fn checkerboard(a: Option<Voxel>, b: Option<Voxel>) -> Self {
        let mut buffer = [None; VOXEL_COUNT];

        for (i, voxel) in buffer.iter_mut().enumerate() {
            match Self::index_to_pos(i).element_sum().is_multiple_of(2) {
                true => *voxel = a,
                false => *voxel = b,
            };
        }

        Self::new(buffer)
    }

    /// Returns the number of non-empty voxels in the VoxelGrid.
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
        self.len() == VOXEL_COUNT
    }

    /// Returns the percent of voxels that are non-empty.
    #[inline]
    #[must_use]
    pub fn percent_full(&self) -> f32 {
        self.len as f32 / VOXEL_COUNT as f32
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
    ///
    /// # Performance
    /// This is *not* the native indexing format for [`VoxelBuffer`].
    /// Use [`VoxelGrid::get`] where possible.
    #[inline]
    #[must_use]
    pub fn get_pos(&self, pos: U8Vec3) -> &Option<Voxel> {
        self.get(Self::pos_to_index(pos))
    }

    /// Returns a [`VoxelRef`] to the value at a specific index.
    #[inline]
    #[must_use]
    pub fn get_ref(&self, index: usize) -> VoxelRef<'_> {
        VoxelRef::new(self, index)
    }

    /// Returns a [`VoxelRef`] to the value at a specific position.
    ///
    /// # Performance
    /// This is *not* the native indexing format for [`VoxelBuffer`].
    /// Use [`VoxelGrid::get_ref`] where possible.
    #[inline]
    #[must_use]
    pub fn get_ref_pos(&self, pos: U8Vec3) -> VoxelRef<'_> {
        self.get_ref(Self::pos_to_index(pos))
    }

    /// Sets the value at a specific index, returning the previous value.
    #[inline]
    pub fn set(&mut self, index: usize, voxel: Option<Voxel>) -> Option<Voxel> {
        if self.buffer[index] == voxel {
            voxel
        } else {
            match (self.buffer[index].is_some(), voxel.is_some()) {
                (true, false) => self.len -= 1,
                (false, true) => self.len += 1,
                (_, _) => {}
            }

            std::mem::replace(&mut self.buffer[index], voxel)
        }
    }

    /// Sets the value at a specific position, returning the previous value.
    ///
    /// # Performance
    /// This is *not* the native indexing format for [`VoxelBuffer`].
    /// Use [`VoxelGrid::set`] where possible.
    #[inline]
    pub fn set_pos(&mut self, pos: U8Vec3, voxel: Option<Voxel>) -> Option<Voxel> {
        self.set(Self::pos_to_index(pos), voxel)
    }

    /// Adds a voxel at a specific index, if it is empty. Otherwise, will return `Err` with the current voxel.
    #[inline]
    pub fn place(&mut self, index: usize, voxel: Voxel) -> Result<(), Voxel> {
        match self.buffer[index] {
            None => {
                self.buffer[index] = Some(voxel);
                self.len += 1;
                Ok(())
            }
            Some(voxel) => Err(voxel),
        }
    }

    /// Adds a voxel at a specific position, if it is empty. Otherwise, will return `Err` with the current voxel.
    ///
    /// # Performance
    /// This is *not* the native indexing format for [`VoxelBuffer`].
    /// Use [`VoxelGrid::place`] where possible.
    #[inline]
    pub fn place_pos(&mut self, pos: U8Vec3, voxel: Voxel) -> Result<(), Voxel> {
        self.place(Self::pos_to_index(pos), voxel)
    }

    /// Erases the voxel at the specified index and returns it.
    #[inline]
    pub fn erase(&mut self, index: usize) -> Option<Voxel> {
        if self.buffer[index].is_none() {
            None
        } else {
            self.len -= 1;
            self.buffer[index].take()
        }
    }

    /// Erases the voxel at the specified position and returns it.
    ///
    /// # Performance
    /// This is *not* the native indexing format for [`VoxelBuffer`].
    /// Use [`VoxelGrid::erase`] where possible.
    #[inline]
    pub fn erase_pos(&mut self, pos: U8Vec3) -> Option<Voxel> {
        self.erase(Self::pos_to_index(pos))
    }

    /// Removes all voxels from the buffer.
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

impl Index<usize> for VoxelGrid {
    type Output = Option<Voxel>;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl Index<U8Vec3> for VoxelGrid {
    type Output = Option<Voxel>;

    fn index(&self, pos: U8Vec3) -> &Self::Output {
        self.get_pos(pos)
    }
}

impl IntoIterator for VoxelGrid {
    type Item = Option<Voxel>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}

impl Default for VoxelGrid {
    fn default() -> Self {
        Self {
            buffer: [None; VOXEL_COUNT],
            len: 0,
        }
    }
}

impl std::fmt::Display for VoxelGrid {
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

/// The data of a specific voxel index.
///
/// For global data on a block *type*, see [`Block`](crate::realm::block::data::Block).
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Voxel {
    pub id: VoxelId,
}

impl Voxel {
    pub fn new(id: VoxelId) -> Self {
        Self { id }
    }

    /// Creates a new voxel from an unchecked ID.
    ///
    /// # Panics
    /// Panics if the ID is zero.
    pub fn new_unwrap(id: u16) -> Self {
        Self::new(VoxelId::new(id).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_to_pos_x() {
        for x in 0..32 {
            assert_eq!(VoxelGrid::index_to_pos(x), U8Vec3::new(x as u8, 0, 0));
        }
    }

    #[test]
    fn pos_to_index_x() {
        for x in 0..32 {
            assert_eq!(VoxelGrid::pos_to_index(U8Vec3::new(x as u8, 0, 0)), x);
        }
    }

    #[test]
    fn index_to_pos_y() {
        for y in 0..32 {
            assert_eq!(VoxelGrid::index_to_pos(y * 32), U8Vec3::new(0, y as u8, 0));
        }
    }

    #[test]
    fn pos_to_index_y() {
        for y in 0..32 {
            assert_eq!(VoxelGrid::pos_to_index(U8Vec3::new(0, y as u8, 0)), y * 32);
        }
    }

    #[test]
    fn index_to_pos_z() {
        for z in 0..32 {
            assert_eq!(
                VoxelGrid::index_to_pos(z * 32 * 32),
                U8Vec3::new(0, 0, z as u8)
            );
        }
    }

    #[test]
    fn pos_to_index_z() {
        for z in 0..32 {
            assert_eq!(
                VoxelGrid::pos_to_index(U8Vec3::new(0, 0, z as u8)),
                z * 32 * 32
            );
        }
    }

    #[test]
    fn index_to_pos_max() {
        assert_eq!(
            VoxelGrid::index_to_pos(VOXEL_COUNT - 1),
            U8Vec3::new(31, 31, 31)
        );
    }

    #[test]
    fn pos_to_index_max() {
        assert_eq!(
            VoxelGrid::pos_to_index(U8Vec3::new(31, 31, 31)),
            VOXEL_COUNT - 1
        );
    }

    #[test]
    #[should_panic(expected = "Index must be less than 32768, got 32768")]
    fn index_to_pos_invalid() {
        _ = VoxelGrid::index_to_pos(VOXEL_COUNT)
    }

    #[test]
    #[should_panic(expected = "x position must be less than 32, got 32")]
    fn pos_to_index_invalid_x() {
        _ = VoxelGrid::pos_to_index(U8Vec3::new(32, 0, 0));
    }

    #[test]
    #[should_panic(expected = "y position must be less than 32, got 32")]
    fn pos_to_index_invalid_y() {
        _ = VoxelGrid::pos_to_index(U8Vec3::new(16, 32, 16));
    }

    #[test]
    #[should_panic(expected = "z position must be less than 32, got 32")]
    fn pos_to_index_invalid_z() {
        _ = VoxelGrid::pos_to_index(U8Vec3::new(31, 31, 32));
    }

    #[test]
    fn set_empty() {
        let mut grid = VoxelGrid::default();

        for i in 0..VOXEL_COUNT {
            assert_eq!(grid.len(), i);
            assert!(grid.get(i).is_none());
            assert!(grid.set(i, Voxel::default().into()).is_none());
            assert_eq!(grid.get(i), &Some(Voxel::default()));
        }
    }

    #[test]
    fn set_full() {
        let mut grid = VoxelGrid::full(Voxel::default());

        for i in 0..VOXEL_COUNT {
            assert_eq!(grid.len(), VOXEL_COUNT);
            assert_eq!(grid.get(i), &Some(Voxel::default()));
            assert_eq!(
                grid.set(i, Voxel::new_unwrap(2).into()),
                Some(Voxel::default())
            );
            assert_eq!(grid.get(i), &Some(Voxel::new_unwrap(2)));
        }
    }

    #[test]
    fn place_success() {
        let mut grid = VoxelGrid::default();

        for i in 0..VOXEL_COUNT {
            assert_eq!(grid.len(), i);
            assert!(grid.get(i).is_none());
            assert!(grid.place(i, Voxel::default().into()).is_ok());
            assert_eq!(grid.get(i), &Some(Voxel::default()));
        }
    }

    #[test]
    fn place_failure() {
        let mut grid = VoxelGrid::full(Voxel::default());

        for i in 0..VOXEL_COUNT {
            assert_eq!(grid.len(), VOXEL_COUNT);
            assert_eq!(grid.get(i), &Voxel::default().into());
            assert_eq!(
                grid.place(i, Voxel::default().into()),
                Err(Voxel::default())
            );
            assert_eq!(grid.get(i), &Voxel::default().into());
        }
    }

    #[test]
    fn percent_empty() {
        let grid = VoxelGrid::default();
        assert!(grid.is_empty());
        assert!(!grid.is_full());
        assert_eq!(grid.percent_full(), 0.0);
        assert_eq!(grid.percent_empty(), 1.0);
    }

    #[test]
    fn percent_full() {
        let grid = VoxelGrid::full(Voxel::default());
        assert!(!grid.is_empty());
        assert!(grid.is_full());
        assert_eq!(grid.percent_full(), 1.0);
        assert_eq!(grid.percent_empty(), 0.0);
    }
}
