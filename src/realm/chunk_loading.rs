use crate::player::{PlayerChunk, PlayerChunkChanged};
use crate::realm::chunk::{Chunk, ChunkPlugin, ChunkPos};
use crate::realm::generation::GenerateChunk;
use bevy::math::U8Vec3;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::{auto_observer, auto_resource};
use std::fmt::Formatter;
use std::ops::{Index, IndexMut};

pub const RADIUS: i32 = 16;
pub const DIAMETER: i32 = RADIUS * 2;
pub const BUFFER_DIAMETER: usize = DIAMETER as usize + 1;

pub const BUFFER_SIZE: usize = BUFFER_DIAMETER * BUFFER_DIAMETER * BUFFER_DIAMETER;
pub const STRIDE_X: usize = 1;
pub const STRIDE_Y: usize = BUFFER_DIAMETER;
pub const STRIDE_Z: usize = BUFFER_DIAMETER * BUFFER_DIAMETER;

pub type ChunkBuffer = [ChunkRef; BUFFER_SIZE];
pub type IntoIter = std::array::IntoIter<ChunkRef, BUFFER_SIZE>;
pub type Iter<'a> = core::slice::Iter<'a, ChunkRef>;
pub type IterMut<'a> = core::slice::IterMut<'a, ChunkRef>;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[auto_resource(plugin = ChunkPlugin, init)]
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

impl std::fmt::Display for LoadedChunks {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..BUFFER_DIAMETER {
            writeln!(f, "y: {y}")?;
            for x in 0..BUFFER_DIAMETER {
                write!(f, "\t")?;
                for z in 0..BUFFER_DIAMETER {
                    write!(f, "{}", self[ivec3(x as i32, y as i32, z as i32)])?;
                }
                writeln!(f)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChunkRef {
    #[default]
    None,
    Generating,
    Empty(Entity),
    Some(Entity),
}

impl From<Option<Entity>> for ChunkRef {
    fn from(value: Option<Entity>) -> Self {
        match value {
            None => Self::None,
            Some(entity) => Self::Some(entity),
        }
    }
}

impl From<ChunkRef> for Option<Entity> {
    fn from(value: ChunkRef) -> Self {
        match value {
            ChunkRef::None | ChunkRef::Generating => None,
            ChunkRef::Empty(entity) | ChunkRef::Some(entity) => Some(entity),
        }
    }
}

impl std::fmt::Display for ChunkRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkRef::None => write!(f, "o"),
            ChunkRef::Generating => write!(f, "-"),
            ChunkRef::Empty(_) => write!(f, "."),
            ChunkRef::Some(_) => write!(f, "#"),
        }
    }
}

#[derive(Event, Default, Clone, Copy)]
pub struct ReloadChunks;

/// Generates all chunks in a radius around the player.
#[auto_observer(plugin = ChunkPlugin)]
fn reload_chunks(
    _event: On<ReloadChunks>,
    mut commands: Commands,
    player_chunk: Res<PlayerChunk>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut messages: MessageWriter<GenerateChunk>,
) {
    info!("Reloading chunks");

    for x in -RADIUS..=RADIUS {
        for y in -RADIUS..=RADIUS {
            for z in -RADIUS..=RADIUS {
                let pos = player_chunk.pos + ivec3(x, y, z);

                match loaded_chunks[pos] {
                    ChunkRef::None => {}
                    ChunkRef::Generating => continue,
                    ChunkRef::Empty(e) | ChunkRef::Some(e) => commands.entity(e).despawn(),
                }

                loaded_chunks[pos] = ChunkRef::Generating;
                messages.write(GenerateChunk::new(pos));
            }
        }
    }
}

/// Generates all new chunks when the player moves between chunk-borders.
#[auto_observer(plugin = ChunkPlugin)]
fn generate_nearby_chunks(
    event: On<PlayerChunkChanged>,
    mut commands: Commands,
    mut loaded_chunks: ResMut<LoadedChunks>,
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

                match loaded_chunks[pos] {
                    ChunkRef::None => {}
                    ChunkRef::Generating => continue,
                    ChunkRef::Empty(e) | ChunkRef::Some(e) => commands.entity(e).despawn(),
                }

                loaded_chunks[pos] = ChunkRef::Generating;
                messages.write(GenerateChunk::new(pos));
            }
        }
    }
}

#[auto_observer(plugin = ChunkPlugin)]
fn on_add_chunk(
    event: On<Add, (ChunkPos, Chunk)>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    chunks: Query<(&ChunkPos, Has<Chunk>)>,
) {
    let Ok((pos, chunk)) = chunks.get(event.entity) else {
        return;
    };

    match chunk {
        true => loaded_chunks[pos.0] = ChunkRef::Some(event.entity),
        false => loaded_chunks[pos.0] = ChunkRef::Empty(event.entity),
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
