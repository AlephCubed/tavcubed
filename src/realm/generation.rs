use crate::realm::chunk::voxel::Voxel;
use crate::realm::chunk::{Chunk, ChunkPos};
use bevy::math::u8vec3;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam_channel::{Receiver, Sender};
use noiz::prelude::*;

pub struct ChunkGenerationPlugin;

impl Plugin for ChunkGenerationPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        app.insert_resource(ChunkGenerationChannel { sender, receiver })
            .add_message::<GenerateChunk>()
            .add_systems(Update, (generate_perlin_chunk, generation_finished).chain());
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

/// A channel to send generated chunks through to be spawned.
#[derive(Resource)]
struct ChunkGenerationChannel {
    sender: Sender<ChunkGenerationFinished>,
    receiver: Receiver<ChunkGenerationFinished>,
}

/// A message to send down the [`ChunkGenerationChannel`].
struct ChunkGenerationFinished {
    pos: IVec3,
    chunk: Option<Chunk>,
}

/// Basic perlin noise heightmap.
fn generate_perlin_chunk(
    mut messages: MessageReader<GenerateChunk>,
    channel: Res<ChunkGenerationChannel>,
) {
    let pool = AsyncComputeTaskPool::get();

    for message in messages.read() {
        let sender = channel.sender.clone();

        if message.position.y != 0 {
            _ = sender.send(ChunkGenerationFinished {
                pos: message.position,
                chunk: None,
            });
            continue;
        }

        let message = *message;

        pool.spawn(async move {
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

            _ = sender.send(ChunkGenerationFinished {
                pos: message.position,
                chunk: Some(chunk),
            });
        })
        .detach();
    }
}

fn generation_finished(mut commands: Commands, channel: Res<ChunkGenerationChannel>) {
    for message in channel.receiver.try_iter() {
        match message.chunk {
            None => commands.spawn(ChunkPos(message.pos)),
            Some(chunk) => commands.spawn((ChunkPos(message.pos), chunk)),
        };
    }
}
