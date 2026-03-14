use crate::chunk::{DirtyFlag, VoxelBuffer};
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

#[derive(Component)]
pub struct ChunkMesh(pub Handle<Mesh>);

#[derive(Resource)]
struct ChunkMeshChannel {
    sender: Sender<ChunkMeshFinished>,
    receiver: Receiver<ChunkMeshFinished>,
}

struct ChunkMeshFinished {
    chunk: Entity,
    mesh: Handle<Mesh>,
}

fn mesh_dirty_chunks(
    channel: Res<ChunkMeshChannel>,
    chunks: Query<(Entity, &VoxelBuffer), With<DirtyFlag>>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (chunk, buffer) in &chunks {
        let sender = channel.sender.clone();

        pool.spawn(async move {
            let _ = sender.send(ChunkMeshFinished {
                chunk,
                mesh: Default::default(),
            });
        })
        .detach();
    }
}

fn mesh_finished(
    channel: Res<ChunkMeshChannel>,
    mut meshes: Query<&mut ChunkMesh, With<DirtyFlag>>,
) {
    for msg in channel.receiver.try_iter() {
        let Ok(mut mesh) = meshes.get_mut(msg.chunk) else {
            warn!("Mesh finished for deleted chunk");
            continue;
        };

        mesh.0 = msg.mesh;
    }
}
