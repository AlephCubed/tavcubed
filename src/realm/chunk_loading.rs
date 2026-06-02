use crate::player::PlayerChunkChanged;
use crate::realm::chunk::mesh::ChunkLOD;
use crate::realm::chunk::{Chunk, ChunkPlugin, ChunkPos, OCTREE_DEPTH};
use crate::realm::generation::GenerateChunk;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::{auto_observer, auto_resource};
use itertools::iproduct;
use std::collections::HashMap;

/// Stores a grid of chunks around the [player's chunk](PlayerChunk).
#[derive(Resource, Deref, DerefMut, Default, Debug, Clone, PartialEq, Eq)]
#[auto_resource(plugin = ChunkPlugin, init)]
pub struct LoadedChunks {
    pub chunks: HashMap<IVec3, ChunkRef>,
}

/// The current state of a chunk in the loadable area.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChunkRef {
    /// No chunk has been created, nor is in the process of being created.
    #[default]
    None,
    /// [`GenerateChunk`] has been called for the given chunk.
    Generating { lod: u8 },
    /// The chunk was generated, but was empty.
    /// To save memory, the [chunk component](Chunk) may not be present on the entity.
    Empty(Entity),
    /// The chunk is generated at the given entity.
    Some { entity: Entity, lod: u8 },
}

impl ChunkRef {
    pub fn get_entity(&self) -> Option<Entity> {
        match self {
            ChunkRef::None | ChunkRef::Generating { .. } => None,
            ChunkRef::Empty(entity) | ChunkRef::Some { entity, .. } => Some(*entity),
        }
    }
}

#[derive(Resource, Deref, DerefMut, Debug, Clone, PartialEq, Eq)]
#[auto_resource(plugin = ChunkPlugin, init)]
pub struct ChunkLodConfig {
    shells: [LodShell; OCTREE_DEPTH + 2],
}

impl ChunkLodConfig {
    pub fn new(shells: [LodShell; OCTREE_DEPTH + 2]) -> Self {
        for (i, pair) in shells.windows(2).enumerate() {
            assert!(
                pair[0].unload_radius < pair[1].load_radius,
                "LOD shell {} unload radius ({}) must be less than shell {} load radius ({})",
                i,
                pair[0].unload_radius,
                i + 1,
                pair[1].load_radius,
            );
            assert!(
                pair[0].load_radius <= pair[0].unload_radius,
                "LOD shell {i} load radius must be <= its unload radius",
            );
        }

        Self { shells }
    }

    /// Returns the desired LOD for a chunk at the given offset, or None if it should be unloaded entirely.
    pub fn desired_lod(&self, offset: IVec3) -> Option<u8> {
        let d2 = offset.dot(offset) as u16;

        self.shells
            .iter()
            .enumerate()
            .find(|(_, shell)| d2 <= shell.load_radius * shell.load_radius)
            .map(|(i, _)| (OCTREE_DEPTH + 1 - i) as u8)
    }

    /// Returns true if a chunk at the given offset should be unloaded.
    pub fn should_unload(&self, offset: IVec3) -> bool {
        let d2 = offset.dot(offset) as u16;

        self.shells
            .last()
            .map(|s| d2 > s.load_radius * s.load_radius)
            .unwrap_or_default()
    }

    pub fn iter_load_offsets(&self) -> impl Iterator<Item = IVec3> {
        let max = self.shells.last().unwrap().load_radius as i32;

        iproduct!(-max..max, -max..max, -max..max).map(|(x, y, z)| ivec3(x, y, z))
    }
}

impl Default for ChunkLodConfig {
    fn default() -> Self {
        Self::new([
            LodShell::new(4, 5),
            LodShell::new(8, 10),
            LodShell::new(12, 14),
            LodShell::new(16, 18),
            LodShell::new(20, 22),
            LodShell::new(24, 26),
        ])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LodShell {
    pub load_radius: u16,
    pub unload_radius: u16,
}

impl LodShell {
    pub fn new(load_radius: u16, unload_radius: u16) -> Self {
        Self {
            load_radius,
            unload_radius,
        }
    }
}

// Todo Reimplement.
/// Regenerates all loaded chunks.
#[derive(Event, Default, Clone, Copy)]
pub struct ReloadChunks;

/// Generates all new chunks when the player moves between chunk-borders.
#[auto_observer(plugin = ChunkPlugin)]
fn generate_nearby_chunks(
    event: On<PlayerChunkChanged>,
    mut commands: Commands,
    mut loaded_chunks: ResMut<LoadedChunks>,
    lod_config: Res<ChunkLodConfig>,
    mut messages: MessageWriter<GenerateChunk>,
    mut chunk_lods: Query<&mut ChunkLOD>,
) {
    let center = event.new_chunk;

    // Unload
    loaded_chunks.chunks.retain(|&pos, chunk_ref| {
        if !lod_config.should_unload(pos - center) {
            return true;
        }

        if let ChunkRef::Empty(entity) | ChunkRef::Some { entity, .. } = *chunk_ref {
            commands.entity(entity).despawn();
        }

        false
    });

    // Load / Change LOD
    for offset in lod_config.iter_load_offsets() {
        let Some(desired_lod) = lod_config.desired_lod(offset) else {
            continue;
        };

        let position = center + offset;

        // Load
        let chunk = loaded_chunks.chunks.entry(position).or_insert_with(|| {
            messages.write(GenerateChunk {
                position,
                lod: desired_lod,
            });
            ChunkRef::Generating { lod: desired_lod }
        });

        // Change LOD
        if let ChunkRef::Some { entity, lod } = *chunk
            && lod != desired_lod
        {
            chunk_lods.get_mut(entity).unwrap().set(desired_lod);
        }
    }
}

#[auto_observer(plugin = ChunkPlugin)]
fn on_add_chunk(
    event: On<Add, (ChunkPos, Chunk)>,
    mut commands: Commands,
    mut loaded_chunks: ResMut<LoadedChunks>,
    chunks: Query<(&ChunkPos, Has<Chunk>, Option<&ChunkLOD>)>,
) {
    let Ok((pos, chunk, lod)) = chunks.get(event.entity) else {
        return;
    };

    let Some(reference) = loaded_chunks.chunks.get_mut(&pos.0) else {
        return;
    };

    // Remove existing.
    if let Some(entity) = reference.get_entity() {
        // Todo decide on collision.
        commands.entity(entity).despawn();
    }

    match chunk {
        true => {
            *reference = ChunkRef::Some {
                entity: event.entity,
                lod: lod.cloned().unwrap_or_default().get(),
            }
        }
        false => *reference = ChunkRef::Empty(event.entity),
    }
}
