mod chunk;

use crate::chunk::mesh::{ChunkMesh, ChunkMeshPlugin};
use crate::chunk::{ChunkPos, DirtyFlag, VoxelBuffer};
use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ChunkMeshPlugin)
        .add_systems(Startup, init_test)
        .run()
}

fn init_test(mut commands: Commands) {
    commands.spawn((
        ChunkPos::default(),
        VoxelBuffer::default(),
        ChunkMesh::default(),
        DirtyFlag,
    ));
}
