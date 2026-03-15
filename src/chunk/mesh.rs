mod cube;

use crate::chunk::VoxelBuffer;
use crate::chunk::mesh::cube::cube;
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

#[derive(Resource)]
struct ChunkMeshChannel {
    sender: Sender<ChunkMeshFinished>,
    receiver: Receiver<ChunkMeshFinished>,
}

struct ChunkMeshFinished {
    chunk: Entity,
    mesh: Mesh,
}

fn mesh_changed_chunks(
    channel: Res<ChunkMeshChannel>,
    chunks: Query<(Entity, &VoxelBuffer), Changed<VoxelBuffer>>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (chunk, buffer) in &chunks {
        let sender = channel.sender.clone();

        debug!("Meshing {chunk}");

        debug!("Buffer: {buffer}");

        let buffer = buffer.0.clone();

        pool.spawn(async move {
            let mut indices = Vec::new();
            let mut positions = Vec::new();

            for (index, _voxel) in buffer
                .iter()
                .enumerate()
                .filter_map(|(i, v)| v.map(|v| (i, v)))
            {
                let pos = VoxelBuffer::index_to_pos(index);

                let (i, p) = cube(
                    pos.x as f32,
                    pos.y as f32,
                    pos.z as f32,
                    indices.len() as u32 * 36,
                );

                indices.extend_from_slice(&i);
                positions.extend_from_slice(&p);
            }

            let mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_indices(Indices::U32(indices))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions);

            let _ = sender.send(ChunkMeshFinished { chunk, mesh });
        })
        .detach();
    }
}

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
