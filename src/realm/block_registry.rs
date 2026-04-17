use bevy::prelude::*;
use bevy::reflect::erased_serde::__private::serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::num::NonZeroU8;
use std::ops::Index;
use std::str::FromStr;

pub struct BlockRegistryPlugin;

impl Plugin for BlockRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistry>()
            .add_systems(Startup, load_core_blocks);
    }
}

const BLOCK_DATA_DIR: &str = "assets/blocks";

fn load_core_blocks(mut registry: ResMut<BlockRegistry>) {
    info!("Loading core blocks");

    for entry in std::fs::read_dir(BLOCK_DATA_DIR).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }

        let content = std::fs::read_to_string(&path).unwrap();
        let block: Block = toml::from_str(&content).unwrap();

        debug!("Loading block {}", block.id);

        registry.register(block);
    }
}

/// Maps [block](BlockId) and [voxel](VoxelId) IDs to [block data](Block).
#[derive(Resource, Default, Debug)]
pub struct BlockRegistry {
    blocks: Vec<Block>,
    id_map: HashMap<BlockId, VoxelId>,
}

impl BlockRegistry {
    /// Registers a new block, returning its unstable [`VoxelId`].
    pub fn register(&mut self, block: Block) -> VoxelId {
        let block_id = block.id.clone();
        self.blocks.push(block);

        let voxel_id = VoxelId::new(self.blocks.len() as u8).unwrap();
        self.id_map.insert(block_id, voxel_id);

        voxel_id
    }

    /// Gets the block's unstable [`VoxelId`].
    ///
    /// # Panics
    /// Panics if the block does not exist. Use [`get_voxel_id`](Self::get_voxel_id) instead if you want to handle this case.
    pub fn voxel_id(&self, block_id: &BlockId) -> VoxelId {
        self.id_map[block_id]
    }

    /// Gets the block's unstable [`VoxelId`], if it exists.
    ///
    /// Use [`voxel_id`](Self::voxel_id) instead if you do not want to handle this case.
    pub fn get_voxel_id(&self, block_id: &BlockId) -> Option<&VoxelId> {
        self.id_map.get(block_id)
    }

    /// Gets the block's data, if it exists.
    ///
    /// Use the [`Index`] implementation instead if you do not want to handle this case.
    pub fn get(&self, block_id: &BlockId) -> Option<&Block> {
        let voxel_id = self.get_voxel_id(block_id)?;
        self.get_from_voxel_id(voxel_id)
    }

    /// Gets the block's data from its unstable ID, if it exists.
    ///
    /// Use the [`Index`] implementation instead if you do not want to handle this case.
    pub fn get_from_voxel_id(&self, voxel_id: &VoxelId) -> Option<&Block> {
        self.blocks.get(voxel_id.get() as usize - 1)
    }
}

impl Index<BlockId> for BlockRegistry {
    type Output = Block;

    fn index(&self, index: BlockId) -> &Self::Output {
        let voxel_id = self.voxel_id(&index);
        &self[voxel_id]
    }
}

impl Index<VoxelId> for BlockRegistry {
    type Output = Block;

    fn index(&self, index: VoxelId) -> &Self::Output {
        &self.blocks[index.get() as usize - 1]
    }
}

/// The *unstable* ID of a [`Block`]. This is determined at runtime by [`BlockRegistry::register`].
///
/// See [`BlockId`] for the block's stable ID.
#[derive(Deref, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VoxelId(NonZeroU8);

impl Default for VoxelId {
    fn default() -> Self {
        Self::new(1).unwrap()
    }
}

impl VoxelId {
    /// Creates a new voxel ID if the value is not zero.
    pub fn new(id: u8) -> Option<Self> {
        Some(Self(NonZeroU8::new(id)?))
    }
}

/// The *stable* ID of a [`Block`]. This is determined by the [block data](Block),
/// and is constant across sessions.
///
/// See [`VoxelId`] for the block's unstable ID.
#[derive(Serialize, Deref, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BlockId(String);

impl BlockId {
    pub fn new(
        mod_id: impl AsRef<String>,
        block_id: impl AsRef<String>,
    ) -> Result<Self, BlockIdError> {
        let mod_id = mod_id.as_ref();
        let block_id = block_id.as_ref();

        if mod_id.is_empty() {
            return Err(BlockIdError::MissingModId);
        }
        if block_id.is_empty() {
            return Err(BlockIdError::MissingBlockId);
        }

        if mod_id.contains(|c: char| c.is_whitespace())
            || block_id.contains(|c: char| !c.is_whitespace())
        {
            return Err(BlockIdError::ContainsWhitespace);
        }

        Ok(Self(format!("{}:{}", mod_id, block_id)))
    }

    /// Splits the block ID into the mod and block portions.
    pub fn split(&self) -> (&str, &str) {
        self.0.split_once(':').unwrap()
    }

    /// Returns the mod portion of the block ID.
    pub fn mod_id(&self) -> &str {
        &self.split().0
    }

    /// Returns the block portion of the block ID.
    pub fn block_id(&self) -> &str {
        &self.split().1
    }
}

impl<'de> Deserialize<'de> for BlockId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for BlockId {
    type Err = BlockIdError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.contains(|c: char| c.is_whitespace()) {
            return Err(BlockIdError::ContainsWhitespace);
        }

        let (mod_id, block_id) = s.split_once(':').ok_or(BlockIdError::MissingColon)?;

        if mod_id.is_empty() {
            return Err(BlockIdError::MissingModId);
        }
        if block_id.is_empty() {
            return Err(BlockIdError::MissingBlockId);
        }

        Ok(Self(s.to_owned()))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlockIdError {
    ContainsWhitespace,
    MissingColon,
    MissingModId,
    MissingBlockId,
}

impl std::fmt::Display for BlockIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockIdError::ContainsWhitespace => write!(f, "BlockID contains whitespace!"),
            BlockIdError::MissingColon => write!(f, "BlockId is missing colon!"),
            BlockIdError::MissingModId => write!(f, "BlockId is missing mod portion of ID!"),
            BlockIdError::MissingBlockId => write!(f, "BlockId is missing block portion of ID!"),
        }
    }
}

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
