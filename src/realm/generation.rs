use crate::realm::chunk::voxel::Voxel;
use crate::realm::chunk::{Chunk, ChunkPos};
use bevy::math::u8vec3;
use bevy::prelude::*;
use noiz::prelude::*;

pub struct ChunkGenerationPlugin;

impl Plugin for ChunkGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GenerateChunk>()
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

const RESOLUTION: f32 = 16.0;
const BASE: u8 = 16;
const AMPLITUDE: f32 = 8.0;

// Todo Use tasks.
/// Basic perlin noise heightmap.
fn generate_perlin_chunk(mut commands: Commands, mut messages: MessageReader<GenerateChunk>) {
    for message in messages.read() {
        if message.position.y != 0 {
            commands.spawn(ChunkPos(message.position));
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
