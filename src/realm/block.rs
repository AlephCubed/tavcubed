//! Data and identifiers for blocks/voxels.
//!
//! In general, "block" refers to stable data stored on disk
//! while "voxel" refers to ephemeral data stored in memory,
//! although this distinction is somewhat fuzzy.

pub mod data;

use crate::realm::RealmPlugin;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::{AutoPlugin, auto_add_plugin};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Formatter;
use std::num::NonZeroU16;
use std::str::FromStr;

#[derive(AutoPlugin)]
#[auto_plugin(impl_plugin_trait)]
#[auto_add_plugin(plugin = RealmPlugin)]
pub struct BlockPlugin;

/// The *unstable* ID of a [`Block`]. This is determined at runtime by [`BlockRegistry::register`].
///
/// See [`BlockId`] for the block's stable ID.
#[derive(Deref, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VoxelId(NonZeroU16);

impl Default for VoxelId {
    fn default() -> Self {
        Self::new(1).unwrap()
    }
}

impl VoxelId {
    /// Creates a new voxel ID if the value is not zero.
    pub fn new(id: u16) -> Option<Self> {
        Some(Self(NonZeroU16::new(id)?))
    }
}

impl From<NonZeroU16> for VoxelId {
    fn from(value: NonZeroU16) -> Self {
        Self(value)
    }
}

/// The *stable* ID of a [`Block`]. This is determined by the [block data](Block),
/// and is constant across sessions.
///
/// See [`VoxelId`] for the block's unstable ID.
#[derive(Serialize, Deref, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BlockId(String);

impl BlockId {
    /// Creates a new block ID from its mod and block portions.
    ///
    /// For creating a block ID from a single string, use [`TryFrom`] instead.
    pub fn new(
        mod_id: impl Into<String>,
        block_id: impl Into<String>,
    ) -> Result<Self, BlockIdError> {
        let mod_id = mod_id.into();
        let block_id = block_id.into();

        if mod_id.is_empty() {
            return Err(BlockIdError::MissingModId);
        }
        if block_id.is_empty() {
            return Err(BlockIdError::MissingBlockId);
        }

        if mod_id.contains(|c: char| c.is_whitespace())
            || block_id.contains(|c: char| c.is_whitespace())
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
        self.split().0
    }

    /// Returns the block portion of the block ID.
    pub fn block_id(&self) -> &str {
        self.split().1
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

impl TryFrom<&str> for BlockId {
    type Error = BlockIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
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
