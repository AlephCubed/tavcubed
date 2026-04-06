use bevy::math::U8Vec3;

pub fn pack(position: U8Vec3, facing: Facing) -> u32 {
    (position.x as u32)
        | ((position.y as u32) << 5)
        | ((position.z as u32) << 10)
        | ((facing as u32) << 15)
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
