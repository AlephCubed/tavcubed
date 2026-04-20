//! Mesh generation for chunks.

mod material;
mod packed_data;

use crate::realm::block::data::BlockFace;
use crate::realm::block::data::registry::{BlockRegistry, BlockRegistryInner};
use crate::realm::chunk::mesh::material::ChunkMaterial;
use crate::realm::chunk::mesh::packed_data::pack;
use crate::realm::chunk::{Chunk, ChunkPos, STRIDE_X, STRIDE_Y, STRIDE_Z};
use bevy::asset::RenderAssetUsages;
use bevy::ecs::bundle::InsertMode;
use bevy::ecs::system::entity_command::insert;
use bevy::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexFormat};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam_channel::{Receiver, Sender};

pub struct ChunkMeshPlugin;

impl Plugin for ChunkMeshPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        app.insert_resource(ChunkMeshChannel { sender, receiver })
            .add_systems(Update, (mesh_changed_chunks, mesh_finished).chain())
            .add_plugins(MaterialPlugin::<ChunkMaterial>::default());
    }
}

pub const INDICES_PER_FACE: usize = 6;
pub const VERTICES_PER_FACE: usize = 4;

#[derive(Component, Default, Debug)]
pub struct ChunkMesh(pub Handle<Mesh>);

impl ChunkMesh {
    pub const ATTRIBUTE_PACKED_DATA: MeshVertexAttribute =
        MeshVertexAttribute::new("packed_data", 806567756968, VertexFormat::Uint32x2);
}

/// A channel to send finished meshes through to be applied.
#[derive(Resource)]
struct ChunkMeshChannel {
    sender: Sender<ChunkMeshFinished>,
    receiver: Receiver<ChunkMeshFinished>,
}

/// A message to send down the [`ChunkMeshChannel`].
struct ChunkMeshFinished {
    chunk: Entity,
    mesh: Mesh,
}

/// Spins up meshing tasks for changed chunks.
fn mesh_changed_chunks(
    channel: Res<ChunkMeshChannel>,
    registry: Res<BlockRegistry>,
    chunks: Query<(Entity, &Chunk), Changed<Chunk>>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (entity, chunk) in &chunks {
        let sender = channel.sender.clone();

        trace!("Meshing {entity}");

        let chunk = *chunk;
        let registry = registry.clone();

        pool.spawn(async move {
            _ = sender.send(ChunkMeshFinished {
                chunk: entity,
                mesh: mesh_chunk(chunk, &registry),
            });
        })
        .detach();
    }
}

/// Creates a mesh from a chunk.
pub fn mesh_chunk(chunk: Chunk, registry: &BlockRegistryInner) -> Mesh {
    let voxel_count = chunk.iter().filter(|v| v.is_some()).count();
    let face_estimate = voxel_count * 3; // Estimate half faces.

    let mut indices = Vec::with_capacity(face_estimate * INDICES_PER_FACE);
    let mut packed: Vec<[u32; 2]> = Vec::with_capacity(face_estimate * VERTICES_PER_FACE);

    for (full_index, voxel) in chunk.iter_full() {
        let pos = Chunk::index_to_pos(full_index);
        let texture = registry[voxel.id].texture;

        if pos.x == 31 || chunk[full_index + STRIDE_X].is_none() {
            indices.extend(get_indices(packed.len() as u32));
            packed.extend([pack(pos, BlockFace::Right, texture); 4]);
        }

        if pos.x == 0 || chunk[full_index - STRIDE_X].is_none() {
            indices.extend(get_indices(packed.len() as u32));
            packed.extend([pack(pos, BlockFace::Left, texture); 4]);
        }

        if pos.y == 31 || chunk[full_index + STRIDE_Y].is_none() {
            indices.extend(get_indices(packed.len() as u32));
            packed.extend([pack(pos, BlockFace::Top, texture); 4]);
        }

        if pos.y == 0 || chunk[full_index - STRIDE_Y].is_none() {
            indices.extend(get_indices(packed.len() as u32));
            packed.extend([pack(pos, BlockFace::Bottom, texture); 4]);
        }

        if pos.z == 31 || chunk[full_index + STRIDE_Z].is_none() {
            indices.extend(get_indices(packed.len() as u32));
            packed.extend([pack(pos, BlockFace::Back, texture); 4]);
        }

        if pos.z == 0 || chunk[full_index - STRIDE_Z].is_none() {
            indices.extend(get_indices(packed.len() as u32));
            packed.extend([pack(pos, BlockFace::Front, texture); 4]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(ChunkMesh::ATTRIBUTE_PACKED_DATA, packed)
}

/// Applies finished meshes to changed chunks.
fn mesh_finished(
    mut commands: Commands,
    channel: Res<ChunkMeshChannel>,
    registry: Res<BlockRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut chunk_meshes: Query<(&mut ChunkMesh, &ChunkPos)>,
) {
    for msg in channel.receiver.try_iter() {
        let Ok((mut mesh, pos)) = chunk_meshes.get_mut(msg.chunk) else {
            warn!("Mesh finished for deleted chunk with ID {}", msg.chunk);
            continue;
        };

        trace!("Finished meshing chunk at {} with ID {}", pos.0, msg.chunk);

        let handle = meshes.add(msg.mesh);
        mesh.0 = handle.clone();
        commands.entity(msg.chunk).queue_handled(
            insert(
                (
                    Mesh3d(handle),
                    MeshMaterial3d(materials.add(ChunkMaterial {
                        chunk_pos: pos.0,
                        texture_array: registry.textures.clone().unwrap(),
                    })),
                ),
                InsertMode::Replace,
            ),
            |_error, _ctx| {
                error!("Unable to insert new mesh!");
            },
        );
    }
}

#[inline]
pub const fn get_indices(index: u32) -> [u32; INDICES_PER_FACE] {
    [
        index + 0,
        index + 3,
        index + 1,
        index + 1,
        index + 3,
        index + 2,
    ]
}
