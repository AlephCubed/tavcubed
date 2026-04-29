use crate::realm::block::data::registry::BlockRegistry;
use crate::realm::chunk::{Chunk, ChunkPlugin, ChunkPos, Voxel, VoxelGrid};
use bevy::math::u8vec3;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy_auto_plugin::prelude::{auto_message, auto_resource, auto_system};
use crossbeam_channel::{Receiver, Sender};
use noiz::prelude::common_noise::Simplex;
use noiz::prelude::*;

/// Generate a chunk at the given position.
#[derive(Message, Clone, Copy, Debug, PartialEq)]
#[auto_message(plugin = ChunkPlugin)]
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
#[auto_resource(plugin = ChunkPlugin, init)]
struct ChunkGenerationChannel {
    sender: Sender<ChunkGenerationFinished>,
    receiver: Receiver<ChunkGenerationFinished>,
}

impl Default for ChunkGenerationChannel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        ChunkGenerationChannel { sender, receiver }
    }
}

/// A message to send down the [`ChunkGenerationChannel`].
struct ChunkGenerationFinished {
    pos: IVec3,
    chunk: Option<Chunk>,
}

/// Basic perlin noise heightmap.
#[auto_system(plugin = ChunkPlugin, schedule = Update, config(before = generation_finished))]
fn generate_perlin_chunk(
    mut messages: MessageReader<GenerateChunk>,
    channel: Res<ChunkGenerationChannel>,
    registry: Res<BlockRegistry>,
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
        let registry = registry.clone();

        pool.spawn(async move {
            let noise = Noise::<Simplex>::default();

            let mut voxel_grid = VoxelGrid::default();

            for x in 0..32 {
                for z in 0..32 {
                    let sample: f32 = noise.sample(
                        vec2(
                            (message.position.x * 32 + x as i32) as f32,
                            (message.position.z * 32 + z as i32) as f32,
                        ) / RESOLUTION,
                    );
                    let height = BASE + (sample * AMPLITUDE) as u8;

                    for y in 0..(height - 1) {
                        voxel_grid.set_pos(
                            u8vec3(x, y, z),
                            Voxel::new(registry.voxel_id(&"core:stone".try_into().unwrap())).into(),
                        );
                    }

                    voxel_grid.set_pos(
                        u8vec3(x, height - 1, z),
                        Voxel::new(registry.voxel_id(&"core:grass".try_into().unwrap())).into(),
                    );
                }
            }

            _ = sender.send(ChunkGenerationFinished {
                pos: message.position,
                chunk: Some(Chunk::new(voxel_grid)),
            });
        })
        .detach();
    }
}

#[auto_system(plugin = ChunkPlugin, schedule = Update)]
fn generation_finished(mut commands: Commands, channel: Res<ChunkGenerationChannel>) {
    for message in channel.receiver.try_iter() {
        match message.chunk {
            None => commands.spawn(ChunkPos(message.pos)),
            Some(chunk) => commands.spawn((ChunkPos(message.pos), chunk)),
        };
    }
}
