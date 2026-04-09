//! Mesh generation for chunks.

pub mod cube;
mod material;
mod packed_data;

use crate::chunk::mesh::cube::{
    INDICES_PER_FACE, VERTICES_PER_FACE, get_indices_neg, get_indices_pos,
};
use crate::chunk::mesh::material::ChunkMaterial;
use crate::chunk::mesh::packed_data::{Facing, pack};
use crate::chunk::{Chunk, ChunkPos, STRIDE_X, STRIDE_Y, STRIDE_Z};
use bevy::asset::RenderAssetUsages;
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

#[derive(Component, Default, Debug)]
pub struct ChunkMesh(pub Handle<Mesh>);

impl ChunkMesh {
    pub const ATTRIBUTE_PACKED_DATA: MeshVertexAttribute =
        MeshVertexAttribute::new("packed_data", 806567756968, VertexFormat::Uint32);
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
    chunks: Query<(Entity, &Chunk), Changed<Chunk>>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (entity, chunk) in &chunks {
        let sender = channel.sender.clone();

        debug!("Meshing {entity}");

        let chunk = *chunk;

        pool.spawn(async move {
            let mesh = mesh_chunk(chunk);
            let _ = sender.send(ChunkMeshFinished {
                chunk: entity,
                mesh,
            });
        })
        .detach();
    }
}

/// Creates a mesh from a chunk.
pub fn mesh_chunk(chunk: Chunk) -> Mesh {
    let voxel_count = chunk.iter().filter(|v| v.is_some()).count();
    let face_estimate = voxel_count * 3; // Estimate half faces.

    let mut indices = Vec::with_capacity(face_estimate * INDICES_PER_FACE);
    let mut packed: Vec<u32> = Vec::with_capacity(face_estimate * VERTICES_PER_FACE);

    for (full_index, voxel) in chunk.iter_full() {
        let pos = Chunk::index_to_pos(full_index);

        if pos.x == 31 || chunk[full_index + STRIDE_X].is_none() {
            indices.extend(get_indices_pos(packed.len() as u32));
            packed.extend([pack(pos, Facing::Right, voxel); 4]);
        }

        if pos.x == 0 || chunk[full_index - STRIDE_X].is_none() {
            indices.extend(get_indices_neg(packed.len() as u32));
            packed.extend([pack(pos, Facing::Left, voxel); 4]);
        }

        if pos.y == 31 || chunk[full_index + STRIDE_Y].is_none() {
            indices.extend(get_indices_pos(packed.len() as u32));
            packed.extend([pack(pos, Facing::Up, voxel); 4]);
        }

        if pos.y == 0 || chunk[full_index - STRIDE_Y].is_none() {
            indices.extend(get_indices_neg(packed.len() as u32));
            packed.extend([pack(pos, Facing::Down, voxel); 4]);
        }

        if pos.z == 31 || chunk[full_index + STRIDE_Z].is_none() {
            indices.extend(get_indices_pos(packed.len() as u32));
            packed.extend([pack(pos, Facing::Back, voxel); 4]);
        }

        if pos.z == 0 || chunk[full_index - STRIDE_Z].is_none() {
            indices.extend(get_indices_neg(packed.len() as u32));
            packed.extend([pack(pos, Facing::Front, voxel); 4]);
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut chunk_meshes: Query<(&mut ChunkMesh, &ChunkPos)>,
) {
    for msg in channel.receiver.try_iter() {
        let Ok((mut mesh, pos)) = chunk_meshes.get_mut(msg.chunk) else {
            warn!("Mesh finished for deleted chunk with ID {}", msg.chunk);
            continue;
        };

        debug!("Finished meshing chunk at {} with ID {}", pos.0, msg.chunk);

        let handle = meshes.add(msg.mesh);
        mesh.0 = handle.clone();
        commands.entity(msg.chunk).insert((
            Mesh3d(handle),
            MeshMaterial3d(materials.add(ChunkMaterial { chunk_pos: pos.0 })),
        ));
    }
}
