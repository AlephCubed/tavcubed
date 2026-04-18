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
    pub fn id(&self) -> &BlockId {
        &self.id
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BlockTexture {
    Uniform(usize),
    PerFace {
        top: usize,
        bottom: usize,
        left: usize,
        right: usize,
        front: usize,
        back: usize,
    },
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
        left: String,
        right: String,
        front: String,
        back: String,
    },
}
