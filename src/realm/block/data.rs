use crate::realm::block::BlockId;
use serde::Deserialize;

pub mod load;
pub mod registry;

/// Global ephemeral data on a specific block type.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Block {
    id: BlockId,
    pub name: String,
    pub texture: VoxelTexture,
}

impl Block {
    pub fn new(id: BlockId, name: String, texture: VoxelTexture) -> Block {
        Block { id, name, texture }
    }

    pub fn id(&self) -> &BlockId {
        &self.id
    }
}

/// Stores ephemeral texture ID(s) for a block. See [`BlockTexture`] for a stable equivalent.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum VoxelTexture {
    Uniform(u16),
    PerFace {
        top: u16,
        bottom: u16,
        right: u16,
        left: u16,
        back: u16,
        front: u16,
    },
}

impl VoxelTexture {
    #[inline]
    pub fn get_face(&self, face: BlockFace) -> u16 {
        match self {
            VoxelTexture::Uniform(i) => *i,
            VoxelTexture::PerFace {
                top,
                bottom,
                right,
                left,
                back,
                front,
            } => match face {
                BlockFace::Top => *top,
                BlockFace::Bottom => *bottom,
                BlockFace::Right => *left,
                BlockFace::Left => *right,
                BlockFace::Back => *back,
                BlockFace::Front => *front,
            },
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum BlockFace {
    Top,
    Bottom,
    Right,
    Left,
    Back,
    Front,
}

/// Deserialized block config data. See [`Block`] for a [resolved](registry::BlockRegistryInner::register) equivalent.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct BlockData {
    pub id: BlockId,
    pub name: String,
    pub texture: BlockTexture,
}

/// Stores ephemeral texture ID(s) for a block. See [`VoxelTexture`] for a stable equivalent.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(untagged)]
enum BlockTexture {
    Uniform(String),
    PerFace {
        top: String,
        bottom: String,
        right: String,
        left: String,
        back: String,
        front: String,
    },
}
