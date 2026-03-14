use crate::chunk::{DirtyFlag, VoxelBuffer};
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

        app.add_systems(Update, (mesh_dirty_chunks, mesh_finished).chain());
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

fn mesh_dirty_chunks(
    channel: Res<ChunkMeshChannel>,
    chunks: Query<(Entity, &VoxelBuffer), With<DirtyFlag>>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (chunk, buffer) in &chunks {
        let sender = channel.sender.clone();

        info!("Meshing {chunk}");

        pool.spawn(async move {
            #[rustfmt::skip]
            let mesh = Mesh::new(
                PrimitiveTopology::TriangleStrip,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_POSITION,
                vec![ // Taken from: https://stackoverflow.com/a/46016469
                    [-0.5,  0.5,  0.5], // Front-top-left
                    [ 0.5,  0.5,  0.5], // Front-top-right
                    [-0.5, -0.5,  0.5], // Front-bottom-left
                    [ 0.5, -0.5,  0.5], // Front-bottom-right
                    [ 0.5, -0.5, -0.5], // Back-bottom-right
                    [ 0.5,  0.5,  0.5], // Front-top-right
                    [ 0.5,  0.5, -0.5], // Back-top-right
                    [-0.5,  0.5,  0.5], // Front-top-left
                    [-0.5,  0.5, -0.5], // Back-top-left
                    [-0.5, -0.5,  0.5], // Front-bottom-le5t
                    [-0.5, -0.5, -0.5], // Back-bottom-left
                    [ 0.5, -0.5, -0.5], // Back-bottom-right
                    [-0.5,  0.5, -0.5], // Back-top-left
                    [ 0.5,  0.5, -0.5], // Back-top-right
                ],
            );

            let _ = sender.send(ChunkMeshFinished { chunk, mesh });
        })
        .detach();
    }
}

fn mesh_finished(
    mut commands: Commands,
    channel: Res<ChunkMeshChannel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_meshes: Query<&mut ChunkMesh, With<DirtyFlag>>,
) {
    for msg in channel.receiver.try_iter() {
        let Ok(mut mesh) = chunk_meshes.get_mut(msg.chunk) else {
            warn!("Mesh finished for deleted chunk {}", msg.chunk);
            continue;
        };

        mesh.0 = meshes.add(msg.mesh);

        commands.entity(msg.chunk).remove::<DirtyFlag>();
    }
}
