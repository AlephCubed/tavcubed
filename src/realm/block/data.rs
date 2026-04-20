use crate::realm::block::BlockId;
use serde::Deserialize;

pub mod load;
pub mod registry;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Block {
    id: BlockId,
    pub name: String,
    pub texture: BlockTexture,
}

impl Block {
    pub fn new(id: BlockId, name: String, texture: BlockTexture) -> Block {
        Block { id, name, texture }
    }

    pub fn id(&self) -> &BlockId {
        &self.id
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BlockTexture {
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

impl BlockTexture {
    #[inline]
    pub fn get_face(&self, face: BlockFace) -> u16 {
        match self {
            BlockTexture::Uniform(i) => *i,
            BlockTexture::PerFace {
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

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct BlockData {
    pub id: BlockId,
    pub name: String,
    pub texture: BlockTextureData,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(untagged)]
enum BlockTextureData {
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
