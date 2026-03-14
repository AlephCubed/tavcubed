mod chunk;

use crate::chunk::mesh::{ChunkMesh, ChunkMeshPlugin};
use crate::chunk::{ChunkPos, VoxelBuffer};
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "tavcubed=debug".to_string(),
            ..default()
        }))
        .add_plugins(FreeCameraPlugin)
        .add_plugins(ChunkMeshPlugin)
        .add_systems(Startup, init_test)
        .run()
}

fn init_test(mut commands: Commands) {
    commands.spawn((
        ChunkPos::default(),
        VoxelBuffer::default(),
        ChunkMesh::default(),
    ));

    commands.spawn((Camera3d::default(), FreeCamera::default()));
}
