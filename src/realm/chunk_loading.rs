use crate::player::{PlayerChunk, PlayerChunkChanged};
use crate::realm::generation::GenerateChunk;
use bevy::math::U8Vec3;
use bevy::prelude::*;
use std::ops::{Index, IndexMut};

pub const RADIUS: i32 = 16;
pub const DIAMETER: i32 = RADIUS * 2;
pub const BUFFER_DIAMETER: usize = DIAMETER as usize + 1;

pub const BUFFER_SIZE: usize = BUFFER_DIAMETER * BUFFER_DIAMETER * BUFFER_DIAMETER;
pub const STRIDE_X: usize = 1;
pub const STRIDE_Y: usize = BUFFER_DIAMETER;
pub const STRIDE_Z: usize = BUFFER_DIAMETER * BUFFER_DIAMETER;

pub struct ChunkLoadingPlugin;

impl Plugin for ChunkLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadedChunks>()
            .add_observer(generate_nearby_chunks)
            .add_observer(reload_chunks);
    }
}

pub type ChunkBuffer = [ChunkRef; BUFFER_SIZE];
pub type IntoIter = std::array::IntoIter<ChunkRef, BUFFER_SIZE>;
pub type Iter<'a> = core::slice::Iter<'a, ChunkRef>;
pub type IterMut<'a> = core::slice::IterMut<'a, ChunkRef>;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LoadedChunks {
    buffer: ChunkBuffer,
    /// The position in chunk-space of the center of the loaded area.
    chunk_center: IVec3,
    /// The index in buffer-space of the center of the loaded area.
    buffer_center: usize,
}

impl LoadedChunks {
    /// Converts a buffer index to an absolute position.
    #[inline]
    fn index_to_abs_pos(mut index: usize) -> U8Vec3 {
        index %= BUFFER_SIZE;
        U8Vec3 {
            x: (index % STRIDE_Y) as u8,
            y: ((index / STRIDE_Y) % STRIDE_Y) as u8,
            z: (index / STRIDE_Z) as u8,
        }
    }

    /// Converts a buffer position to an absolute index.
    #[inline]
    fn pos_to_abs_index(pos: IVec3) -> usize {
        let x = pos.x.rem_euclid(BUFFER_DIAMETER as i32) as usize * STRIDE_X;
        let y = pos.y.rem_euclid(BUFFER_DIAMETER as i32) as usize * STRIDE_Y;
        let z = pos.z.rem_euclid(BUFFER_DIAMETER as i32) as usize * STRIDE_Z;
        z + y + x
    }

    pub fn iter(&'_ self) -> Iter<'_> {
        self.buffer.iter()
    }

    pub fn iter_mut(&'_ mut self) -> IterMut<'_> {
        self.buffer.iter_mut()
    }
}

impl Default for LoadedChunks {
    fn default() -> Self {
        Self {
            buffer: [ChunkRef::default(); BUFFER_SIZE],
            chunk_center: IVec3::default(),
            buffer_center: 0,
        }
    }
}

impl Index<usize> for LoadedChunks {
    type Output = ChunkRef;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index % BUFFER_DIAMETER]
    }
}

impl IndexMut<usize> for LoadedChunks {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer[index % BUFFER_SIZE]
    }
}

impl Index<IVec3> for LoadedChunks {
    type Output = ChunkRef;

    fn index(&self, pos: IVec3) -> &Self::Output {
        &self.buffer[Self::pos_to_abs_index(pos)]
    }
}

impl IndexMut<IVec3> for LoadedChunks {
    fn index_mut(&mut self, pos: IVec3) -> &mut Self::Output {
        &mut self.buffer[Self::pos_to_abs_index(pos)]
    }
}

impl IntoIterator for LoadedChunks {
    type Item = ChunkRef;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChunkRef {
    #[default]
    None,
    Empty(Entity),
    Some(Entity),
}

#[derive(Event, Default, Clone, Copy)]
pub struct ReloadChunks;

/// Generates all chunks in a radius around the player.
fn reload_chunks(
    _event: On<ReloadChunks>,
    player_chunk: Res<PlayerChunk>,
    mut messages: MessageWriter<GenerateChunk>,
) {
    for x in -RADIUS..=RADIUS {
        for y in -RADIUS..=RADIUS {
            for z in -RADIUS..=RADIUS {
                messages.write(GenerateChunk::new(player_chunk.pos + ivec3(x, y, z)));
            }
        }
    }
}

/// Generates all new chunks when the player moves between chunk-borders.
fn generate_nearby_chunks(
    event: On<PlayerChunkChanged>,
    mut messages: MessageWriter<GenerateChunk>,
) {
    let diff = event.new_chunk - event.old_chunk;
    let new = event.new_chunk;

    for axis in 0..3 {
        let delta = diff[axis];
        if delta == 0 {
            continue;
        }

        let slab_coord = new[axis] + delta.signum() * RADIUS;

        for a in -RADIUS..=RADIUS {
            for b in -RADIUS..=RADIUS {
                let pos = match axis {
                    0 => IVec3::new(slab_coord, new.y + a, new.z + b),
                    1 => IVec3::new(new.x + a, slab_coord, new.z + b),
                    2 => IVec3::new(new.x + a, new.y + b, slab_coord),
                    _ => unreachable!(),
                };
                messages.write(GenerateChunk::new(pos));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: i32 = -10;
    const MAX: i32 = 10;

    #[test]
    fn index_to_abs_pos_x() {
        for x in 0..DIAMETER {
            assert_eq!(
                LoadedChunks::index_to_abs_pos(x as usize),
                U8Vec3::new(x as u8, 0, 0)
            );
        }
    }

    #[test]
    fn pos_to_abs_index_x() {
        for x in MIN..MAX {
            assert_eq!(
                LoadedChunks::pos_to_abs_index(IVec3::new(x, 0, 0)),
                x.rem_euclid(STRIDE_Y as i32) as usize,
            );
        }
    }

    #[test]
    fn index_to_abs_pos_y() {
        for y in 0..DIAMETER {
            assert_eq!(
                LoadedChunks::index_to_abs_pos(y as usize * STRIDE_Y),
                U8Vec3::new(0, y as u8, 0)
            );
        }
    }

    #[test]
    fn pos_to_abs_index_y() {
        for y in MIN..MAX {
            assert_eq!(
                LoadedChunks::pos_to_abs_index(IVec3::new(0, y, 0)),
                (y * STRIDE_Y as i32).rem_euclid(STRIDE_Z as i32) as usize,
            );
        }
    }

    #[test]
    fn index_to_abs_pos_z() {
        for z in 0..DIAMETER {
            assert_eq!(
                LoadedChunks::index_to_abs_pos(z as usize * STRIDE_Z),
                U8Vec3::new(0, 0, z as u8)
            );
        }
    }

    #[test]
    fn pos_to_abs_index_z() {
        for z in MIN..MAX {
            assert_eq!(
                LoadedChunks::pos_to_abs_index(IVec3::new(0, 0, z)),
                (z * STRIDE_Z as i32).rem_euclid(BUFFER_SIZE as i32) as usize,
            );
        }
    }

    #[test]
    fn index_to_abs_pos_max() {
        assert_eq!(
            LoadedChunks::index_to_abs_pos(BUFFER_SIZE - 1),
            U8Vec3::new(
                BUFFER_DIAMETER as u8 - 1,
                BUFFER_DIAMETER as u8 - 1,
                BUFFER_DIAMETER as u8 - 1,
            )
        );
    }

    #[test]
    fn index_to_abs_pos_wrap() {
        assert_eq!(
            LoadedChunks::index_to_abs_pos(BUFFER_SIZE),
            U8Vec3::new(0, 0, 0)
        );
    }
}
