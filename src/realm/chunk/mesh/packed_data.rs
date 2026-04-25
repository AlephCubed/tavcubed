use crate::realm::block::data::{BlockFace, BlockTexture};
use bevy::math::{U8Vec2, U8Vec3};

#[inline]
pub fn pack(data: VoxelData, block_face: BlockFace) -> [u32; 2] {
    [
        // Low:
        (data.position.x as u32) // 5-bits (0..5)
        | ((data.position.y as u32) << 5) // 5-bits (5..10)
        | ((data.position.z as u32) << 10) // 5-bits (10..15)
        | ((data.size.x as u32 - 1) << 15) // 5-bits (15..20)
        | ((data.size.y as u32 - 1) << 20) // 5-bits (20..25)
        | ((block_face as u32) << 25), // 3-bits (25..28)
        // High:
        data.texture.get_face(block_face) as u32, // 16-bits (0..16)
    ]
}

#[derive(Copy, Clone, Debug)]
pub struct VoxelData {
    pub position: U8Vec3,
    pub size: U8Vec2,
    pub texture: BlockTexture,
}
