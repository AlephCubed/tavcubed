use crate::realm::block::BlockId;
use bevy::math::U8Vec3;
use bevy::prelude::*;
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

/// A specific side of a block/voxel.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum BlockFace {
    /// Facing up (y+).
    #[doc(alias = "Up", alias = "Y")]
    Top,
    /// Facing down (y-).
    #[doc(alias = "Down", alias = "NegY")]
    Bottom,
    /// Facing right (x+).
    #[doc(alias = "X")]
    Right,
    /// Facing left (x-).
    #[doc(alias = "NegX")]
    Left,
    /// Facing away (z+)
    #[doc(alias = "Z")]
    Back,
    /// Facing forward (z-)
    #[doc(alias = "NegZ")]
    Front,
}

impl BlockFace {
    pub fn flip(&self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Right => Self::Left,
            Self::Left => Self::Right,
            Self::Back => Self::Front,
            Self::Front => Self::Back,
        }
    }

    pub fn normal(&self) -> Dir3 {
        match self {
            BlockFace::Top => Dir3::Y,
            BlockFace::Bottom => Dir3::NEG_Z,
            BlockFace::Right => Dir3::X,
            BlockFace::Left => Dir3::NEG_X,
            BlockFace::Back => Dir3::Z,
            BlockFace::Front => Dir3::NEG_Z,
        }
    }

    pub fn from_direction(dir: Dir3) -> Self {
        let abs = dir.abs();

        if abs.x >= abs.y && abs.x >= abs.z {
            if dir.x >= 0.0 {
                Self::Right
            } else {
                Self::Left
            }
        } else if abs.y >= abs.z {
            if dir.y >= 0.0 {
                Self::Top
            } else {
                Self::Bottom
            }
        } else {
            if dir.z >= 0.0 {
                Self::Back
            } else {
                Self::Front
            }
        }
    }

    #[rustfmt::skip]
    pub fn from_axis(axis: VecAxis, vec: U8Vec3) -> Self {
        match axis {
            VecAxis::X => if vec.x > 0 { BlockFace::Right } else { BlockFace::Left }
            VecAxis::Y => if vec.y > 0 { BlockFace::Top } else { BlockFace::Bottom }
            VecAxis::Z => if vec.z > 0 { BlockFace::Back } else { BlockFace::Front }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum VecAxis {
    X,
    Y,
    Z,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_direction() {
        assert_eq!(BlockFace::from_direction(Dir3::X), BlockFace::Right);
        assert_eq!(BlockFace::from_direction(Dir3::NEG_X), BlockFace::Left);
        assert_eq!(BlockFace::from_direction(Dir3::Y), BlockFace::Top);
        assert_eq!(BlockFace::from_direction(Dir3::NEG_Y), BlockFace::Bottom);
        assert_eq!(BlockFace::from_direction(Dir3::Z), BlockFace::Back);
        assert_eq!(BlockFace::from_direction(Dir3::NEG_Z), BlockFace::Front);
    }
}
