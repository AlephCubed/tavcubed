use crate::realm::block::data::{BlockFace, BlockTexture};
use bevy::math::U8Vec3;

#[inline]
pub fn pack(position: U8Vec3, block_face: BlockFace, texture: BlockTexture) -> u32 {
    (position.x as u32) // 5-bits (0..5)
        | ((position.y as u32) << 5) // 5-bits (5..10)
        | ((position.z as u32) << 10) // 5-bits (10..15)
        | ((block_face as u32) << 15) // 3-bits (15..18)
        | ((texture.get_face(block_face) as u32) << 18) // Todo
}
