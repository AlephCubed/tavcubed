//! Mesh generation for chunks.

pub mod cube;

use crate::chunk::mesh::cube::{
    INDICES_PER_FACE, VERTICES_PER_FACE, face_back, face_bottom, face_front, face_left, face_right,
    face_top, get_indices_neg, get_indices_pos,
};
use crate::chunk::{STRIDE_X, STRIDE_Y, STRIDE_Z, VoxelBuffer};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam_channel::{Receiver, Sender};

pub struct ChunkMeshPlugin;

impl Plugin for ChunkMeshPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        app.insert_resource(ChunkMeshChannel { sender, receiver });

        app.add_systems(Update, (mesh_changed_chunks, mesh_finished).chain());
    }
}

#[derive(Component, Default, Debug)]
pub struct ChunkMesh(pub Handle<Mesh>);

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
    chunks: Query<(Entity, &VoxelBuffer), Changed<VoxelBuffer>>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (chunk, buffer) in &chunks {
        let sender = channel.sender.clone();

        debug!("Meshing {chunk}");

        debug!("Buffer: {buffer}");

        let buffer = buffer.clone();

        pool.spawn(async move {
            let mesh = mesh_chunk(buffer);
            let _ = sender.send(ChunkMeshFinished { chunk, mesh });
        })
        .detach();
    }
}

/// Creates a mesh from a chunk's voxel buffer.
pub fn mesh_chunk(buffer: VoxelBuffer) -> Mesh {
    let voxel_count = buffer.0.iter().filter(|v| v.is_some()).count();
    let face_estimate = voxel_count * 3; // Estimate half faces.

    let mut indices = Vec::with_capacity(face_estimate * INDICES_PER_FACE);
    let mut positions = Vec::with_capacity(face_estimate * VERTICES_PER_FACE);

    for full_index in buffer
        .0
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|_| i))
    {
        let pos = VoxelBuffer::index_to_pos(full_index);

        let (x, y, z) = (pos.x as f32, pos.y as f32, pos.z as f32);

        if pos.x == 31 || buffer[full_index + STRIDE_X].is_none() {
            indices.extend(get_indices_pos(positions.len() as u32));
            positions.extend(face_right(x, y, z));
        }

        if pos.x == 0 || buffer[full_index - STRIDE_X].is_none() {
            indices.extend(get_indices_neg(positions.len() as u32));
            positions.extend(face_left(x, y, z));
        }

        if pos.y == 31 || buffer[full_index + STRIDE_Y].is_none() {
            indices.extend(get_indices_pos(positions.len() as u32));
            positions.extend(face_top(x, y, z));
        }

        if pos.y == 0 || buffer[full_index - STRIDE_Y].is_none() {
            indices.extend(get_indices_neg(positions.len() as u32));
            positions.extend(face_bottom(x, y, z));
        }

        if pos.z == 31 || buffer[full_index + STRIDE_Z].is_none() {
            indices.extend(get_indices_pos(positions.len() as u32));
            positions.extend(face_back(x, y, z));
        }

        if pos.z == 0 || buffer[full_index - STRIDE_Z].is_none() {
            indices.extend(get_indices_neg(positions.len() as u32));
            positions.extend(face_front(x, y, z));
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
}

/// Applies finished meshes to changed chunks.
fn mesh_finished(
    mut commands: Commands,
    channel: Res<ChunkMeshChannel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_meshes: Query<&mut ChunkMesh>,
) {
    for msg in channel.receiver.try_iter() {
        let Ok(mut mesh) = chunk_meshes.get_mut(msg.chunk) else {
            warn!("Mesh finished for deleted chunk {}", msg.chunk);
            continue;
        };

        debug!("Finished meshing {}", msg.chunk);

        let handle = meshes.add(msg.mesh);
        mesh.0 = handle.clone();
        commands.entity(msg.chunk).insert((
            Mesh3d(handle),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
        ));
    }
}
