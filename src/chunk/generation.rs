use crate::chunk::voxel::Voxel;
use crate::chunk::{Chunk, ChunkPos};
use crate::player::{PlayerChunk, PlayerChunkChanged};
use bevy::math::u8vec3;
use bevy::prelude::*;
use noiz::prelude::*;

pub struct ChunkGenerationPlugin;

impl Plugin for ChunkGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GenerateChunk>()
            .add_observer(generate_nearby_chunks)
            .add_observer(reload_chunks)
            .add_systems(Update, generate_perlin_chunk);
    }
}

#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct GenerateChunk {
    pub position: IVec3,
}

impl GenerateChunk {
    pub fn new(position: IVec3) -> GenerateChunk {
        GenerateChunk { position }
    }
}

pub const RADIUS: i32 = 16;

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

const RESOLUTION: f32 = 16.0;
const BASE: u8 = 16;
const AMPLITUDE: f32 = 8.0;

// Todo Use tasks.
/// Basic perlin noise heightmap.
fn generate_perlin_chunk(mut commands: Commands, mut messages: MessageReader<GenerateChunk>) {
    for message in messages.read() {
        if message.position.y != 0 {
            continue;
        }

        let noise = Noise::<MixCellGradients<OrthoGrid, Smoothstep, QuickGradients>>::default();

        let mut chunk = Chunk::default();

        for x in 0..32 {
            for z in 0..32 {
                let sample: f32 = noise.sample(
                    vec2(
                        (message.position.x * 32 + x as i32) as f32,
                        (message.position.z * 32 + z as i32) as f32,
                    ) / RESOLUTION,
                );
                let height = BASE + (sample * AMPLITUDE) as u8;

                for y in 0..height {
                    chunk[u8vec3(x, y, z)] = Some(Voxel::default());
                }
            }
        }

        commands.spawn((ChunkPos(message.position), chunk));
    }
}
