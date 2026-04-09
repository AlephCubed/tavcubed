#![doc(alias = "morton_code")]
//! Based on https://fgiesen.wordpress.com/2009/12/13/decoding-morton-codes/

use bevy::math::U8Vec3;
use bevy::prelude::*;

#[rustfmt::skip]
#[inline]
fn inflate(mut n: u32) -> u32 {
    n &= 0x000003ff;                  // x = ---- ---- ---- ---- ---- --98 7654 3210
    n = (n ^ (n << 16)) & 0xff0000ff; // x = ---- --98 ---- ---- ---- ---- 7654 3210
    n = (n ^ (n <<  8)) & 0x0300f00f; // x = ---- --98 ---- ---- 7654 ---- ---- 3210
    n = (n ^ (n <<  4)) & 0x030c30c3; // x = ---- --98 ---- 76-- --54 ---- 32-- --10
    n = (n ^ (n <<  2)) & 0x09249249; // x = ---- 9--8 --7- -6-- 5--4 --3- -2-- 1--0
    n
}

#[rustfmt::skip]
#[inline]
fn deflate(mut n: u32) -> u32 {
    n &= 0x09249249;                  // x = ---- 9--8 --7- -6-- 5--4 --3- -2-- 1--0
    n = (n ^ (n >>  2)) & 0x030c30c3; // x = ---- --98 ---- 76-- --54 ---- 32-- --10
    n = (n ^ (n >>  4)) & 0x0300f00f; // x = ---- --98 ---- ---- 7654 ---- ---- 3210
    n = (n ^ (n >>  8)) & 0xff0000ff; // x = ---- --98 ---- ---- ---- ---- 7654 3210
    n = (n ^ (n >> 16)) & 0x000003ff; // x = ---- ---- ---- ---- ---- --98 7654 3210
    n
}

#[inline]
fn interleave(pos: UVec3) -> u32 {
    (inflate(pos.z) << 2) | (inflate(pos.y) << 1) | inflate(pos.x)
}

fn extract(index: u32) -> UVec3 {
    UVec3 {
        x: deflate(index),
        y: deflate(index >> 1),
        z: deflate(index >> 2),
    }
}

#[inline]
pub(super) fn pos_to_index(pos: U8Vec3) -> usize {
    interleave(pos.into()) as usize
}

#[inline]
pub(super) fn index_to_pos(index: usize) -> U8Vec3 {
    extract(index as u32).try_into().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::CHUNK_VOXEL_COUNT;

    #[test]
    fn first_to_pos() {
        assert_eq!(index_to_pos(0), U8Vec3::default());
    }

    #[test]
    fn last_to_pos() {
        assert_eq!(index_to_pos(CHUNK_VOXEL_COUNT - 1), U8Vec3::new(31, 31, 31));
    }
}
