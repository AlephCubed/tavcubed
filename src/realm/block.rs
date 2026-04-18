pub mod data;

use crate::realm::block::data::load::load_core_blocks;
use bevy::prelude::*;
use data::registry::BlockRegistry;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Formatter;
use std::num::NonZeroU8;
use std::str::FromStr;

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistry>()
            .add_systems(Startup, load_core_blocks);
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
