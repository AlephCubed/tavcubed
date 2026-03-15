mod chunk;

use crate::chunk::mesh::{ChunkMesh, ChunkMeshPlugin};
use crate::chunk::voxel::Voxel;
use crate::chunk::{CHUNK_VOXEL_COUNT, ChunkPos, VoxelBuffer};
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::log::LogPlugin;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use std::num::NonZeroU8;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "tavcubed=debug".to_string(),
            ..default()
        }))
        .add_plugins((FreeCameraPlugin, WireframePlugin::default()))
        .add_plugins(ChunkMeshPlugin)
        .add_systems(Startup, init_test)
        .add_systems(Update, toggle_wireframe)
        .run()
}

fn init_test(mut commands: Commands) {
    let mut voxels = VoxelBuffer::default();

    for i in 0..(CHUNK_VOXEL_COUNT / 16) {
        voxels.0[i * 3] = Some(Voxel { id: NonZeroU8::MIN });
    }

    commands.spawn((ChunkPos::default(), voxels, ChunkMesh::default()));

    commands.spawn((Camera3d::default(), FreeCamera::default()));
}

fn toggle_wireframe(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        wireframe_config.global = !wireframe_config.global;
    }
}
