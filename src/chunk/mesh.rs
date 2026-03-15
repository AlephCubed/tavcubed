use crate::chunk::VoxelBuffer;
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

// Taken from: https://github.com/bevyengine/bevy/blob/main/examples/3d/generate_custom_mesh.rs
fn cube(x: f32, y: f32, z: f32, index: u32) -> ([u32; 36], [[f32; 3]; 24]) {
    (
        [
            // top (+y).
            index + 0,
            index + 3,
            index + 1,
            index + 1,
            index + 3,
            index + 2,
            // bottom (-y)
            index + 4,
            index + 5,
            index + 7,
            index + 5,
            index + 6,
            index + 7,
            // right (+x)
            index + 8,
            index + 11,
            index + 9,
            index + 9,
            index + 11,
            index + 10,
            // left (-x)
            index + 12,
            index + 13,
            index + 15,
            index + 13,
            index + 14,
            index + 15,
            // back (+z)
            index + 16,
            index + 19,
            index + 17,
            index + 17,
            index + 19,
            index + 18,
            // forward (-z)
            index + 20,
            index + 21,
            index + 23,
            index + 21,
            index + 22,
            index + 23,
        ],
        [
            // top (+y)
            [x - 0.5, y + 0.5, z - 0.5],
            [x + 0.5, y + 0.5, z - 0.5],
            [x + 0.5, y + 0.5, z + 0.5],
            [x - 0.5, y + 0.5, z + 0.5],
            // bottom (-y)
            [x - 0.5, y - 0.5, z - 0.5],
            [x + 0.5, y - 0.5, z - 0.5],
            [x + 0.5, y - 0.5, z + 0.5],
            [x - 0.5, y - 0.5, z + 0.5],
            // right (+x)
            [x + 0.5, y - 0.5, z - 0.5],
            [x + 0.5, y - 0.5, z + 0.5],
            [x + 0.5, y + 0.5, z + 0.5],
            [x + 0.5, y + 0.5, z - 0.5],
            // left (-x)
            [x - 0.5, y - 0.5, z - 0.5],
            [x - 0.5, y - 0.5, z + 0.5],
            [x - 0.5, y + 0.5, z + 0.5],
            [x - 0.5, y + 0.5, z - 0.5],
            // back (+z)
            [x - 0.5, y - 0.5, z + 0.5],
            [x - 0.5, y + 0.5, z + 0.5],
            [x + 0.5, y + 0.5, z + 0.5],
            [x + 0.5, y - 0.5, z + 0.5],
            // forward (-z)
            [x - 0.5, y - 0.5, z - 0.5],
            [x - 0.5, y + 0.5, z - 0.5],
            [x + 0.5, y + 0.5, z - 0.5],
            [x + 0.5, y - 0.5, z - 0.5],
        ],
    )
}
