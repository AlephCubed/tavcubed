use crate::realm::block::BlockId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Block {
    id: BlockId,
    pub name: String,
}

impl Block {
    pub fn new(id: BlockId, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> &BlockId {
        &self.id
    }
}
