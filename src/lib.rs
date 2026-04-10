pub mod player;
pub mod realm;

use crate::player::PlayerPlugin;
use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::log::LogPlugin;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use realm::chunk::mesh::ChunkMeshPlugin;
use realm::generation::ChunkGenerationPlugin;

pub fn app() -> App {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(LogPlugin {
        filter: "tavcubed=debug".to_string(),
        ..default()
    }))
    .add_plugins((FreeCameraPlugin, WireframePlugin::default()))
    .add_plugins((PlayerPlugin, ChunkMeshPlugin, ChunkGenerationPlugin))
    .add_systems(Update, toggle_wireframe);

    app
}

fn toggle_wireframe(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        wireframe_config.global = !wireframe_config.global;
    }
}
