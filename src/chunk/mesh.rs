use crate::chunk::{VoxelBuffer, CHUNK_VOXEL_COUNT};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
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

        let buffer = buffer.0.clone();

        pool.spawn(async move {
            let mut positions = Vec::with_capacity(CHUNK_VOXEL_COUNT / 2);

            for (index, _voxel) in buffer
                .iter()
                .enumerate()
                .filter_map(|(i, v)| v.map(|v| (i, v)))
            {
                let pos = VoxelBuffer::index_to_pos(index);

                positions.extend_from_slice(&cube(pos.x as f32, pos.y as f32, pos.z as f32));
            }

            let mesh = Mesh::new(
                PrimitiveTopology::TriangleStrip,
                RenderAssetUsages::default(),
            )
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

fn cube(x: f32, y: f32, z: f32) -> [[f32; 3]; 14] {
    [
        // Taken from: https://stackoverflow.com/a/46016469
        [x - 0.5, y + 0.5, z + 0.5], // Front-top-left
        [x + 0.5, y + 0.5, z + 0.5], // Front-top-right
        [x - 0.5, y - 0.5, z + 0.5], // Front-bottom-left
        [x + 0.5, y - 0.5, z + 0.5], // Front-bottom-right
        [x + 0.5, y - 0.5, z - 0.5], // Back-bottom-right
        [x + 0.5, y + 0.5, z + 0.5], // Front-top-right
        [x + 0.5, y + 0.5, z - 0.5], // Back-top-right
        [x - 0.5, y + 0.5, z + 0.5], // Front-top-left
        [x - 0.5, y + 0.5, z - 0.5], // Back-top-left
        [x - 0.5, y - 0.5, z + 0.5], // Front-bottom-le5t
        [x - 0.5, y - 0.5, z - 0.5], // Back-bottom-left
        [x + 0.5, y - 0.5, z - 0.5], // Back-bottom-right
        [x - 0.5, y + 0.5, z - 0.5], // Back-top-left
        [x + 0.5, y + 0.5, z - 0.5], // Back-top-right
    ]
}
