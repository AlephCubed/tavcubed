use crate::player::{PlayerChunk, PlayerChunkChanged};
use crate::realm::generation::GenerateChunk;
use bevy::prelude::*;

pub const RADIUS: i32 = 16;

pub struct ChunkLoadingPlugin;

impl Plugin for ChunkLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(generate_nearby_chunks)
            .add_observer(reload_chunks);
    }
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
