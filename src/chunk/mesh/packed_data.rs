use crate::chunk::voxel::Voxel;
use bevy::math::U8Vec3;

#[inline]
pub fn pack(position: U8Vec3, facing: Facing, voxel: Voxel) -> u32 {
    (position.x as u32) // 5-bits (0..5)
        | ((position.y as u32) << 5) // 5-bits (5..10)
        | ((position.z as u32) << 10) // 5-bits (10..15)
        | ((facing as u32) << 15) // 3-bits (15..18)
        | ((voxel.id.get() as u32) << 18) // 8-bits (18..26)
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Facing {
    Up,
    Down,
    Right,
    Left,
    Back,
    Front,
}
