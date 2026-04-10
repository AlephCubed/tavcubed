use crate::chunk::voxel::Voxel;
use crate::chunk::{Chunk, ChunkPos};
use crate::player::PlayerChunkChanged;
use bevy::math::u8vec3;
use bevy::prelude::*;
use noiz::prelude::*;

pub struct ChunkGenerationPlugin;

impl Plugin for ChunkGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(generate_nearby_chunks)
            .add_observer(generate_perlin_chunk);
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq)]
pub struct GenerateChunk {
    pub position: IVec3,
}

impl GenerateChunk {
    pub fn new(position: IVec3) -> GenerateChunk {
        GenerateChunk { position }
    }
}

pub const RADIUS: i32 = 16;

fn generate_nearby_chunks(event: On<PlayerChunkChanged>, mut commands: Commands) {
    for x in -RADIUS..=RADIUS {
        for y in -RADIUS..=RADIUS {
            for z in -RADIUS..=RADIUS {
                if (x * x) + (y * y) + (z * z) <= (RADIUS * RADIUS) {
                    commands.trigger(GenerateChunk::new(event.new_chunk + ivec3(x, y, z)));
                }
            }
        }
    }
}

const RESOLUTION: f32 = 16.0;
const BASE: u8 = 16;
const AMPLITUDE: f32 = 8.0;

/// Basic perlin noise heightmap.
fn generate_perlin_chunk(event: On<GenerateChunk>, mut commands: Commands) {
    if event.position.y != 0 {
        return;
    }

    let noise = Noise::<MixCellGradients<OrthoGrid, Smoothstep, QuickGradients>>::default();

    let mut chunk = Chunk::default();

    for x in 0..32 {
        for z in 0..32 {
            let sample: f32 = noise.sample(
                vec2(
                    (event.position.x * 16 + x as i32) as f32,
                    (event.position.z * 16 + z as i32) as f32,
                ) / RESOLUTION,
            );
            let height = BASE + (sample * AMPLITUDE) as u8;

            for y in 0..height {
                chunk[u8vec3(x, y, z)] = Some(Voxel::default());
            }
        }
    }

    commands.spawn((ChunkPos(event.position), chunk));
}
