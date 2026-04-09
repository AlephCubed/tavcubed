pub mod chunk;
pub mod player;

use crate::chunk::generation::{ChunkGenerationPlugin, GenerateChunk};
use crate::chunk::mesh::ChunkMeshPlugin;
use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::log::LogPlugin;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;

pub fn app() -> App {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(LogPlugin {
        filter: "tavcubed=debug".to_string(),
        ..default()
    }))
    .add_plugins((FreeCameraPlugin, WireframePlugin::default()))
    .add_plugins((ChunkMeshPlugin, ChunkGenerationPlugin))
    .add_systems(Startup, init_test)
    .add_systems(Update, toggle_wireframe);

    app
}

const RADIUS: i32 = 16;

fn init_test(mut commands: Commands) {
    for x in -RADIUS..=RADIUS {
        for z in -RADIUS..=RADIUS {
            commands.trigger(GenerateChunk::new(ivec3(x, 0, z)));
        }
    }
}

fn toggle_wireframe(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        wireframe_config.global = !wireframe_config.global;
    }
}
