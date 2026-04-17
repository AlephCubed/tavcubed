pub mod player;
pub mod realm;

use crate::player::PlayerPlugin;
use crate::realm::block_registry::BlockRegistryPlugin;
use crate::realm::chunk_loading::{ChunkLoadingPlugin, ReloadChunks};
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
    .add_plugins((
        PlayerPlugin,
        ChunkMeshPlugin,
        ChunkLoadingPlugin,
        ChunkGenerationPlugin,
        BlockRegistryPlugin,
    ))
    .add_systems(Update, debug_keybinds);

    app
}

fn debug_keybinds(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        info!("Toggling Wireframe");
        wireframe_config.global = !wireframe_config.global;
    }

    if keyboard_input.pressed(KeyCode::SuperLeft) & keyboard_input.just_pressed(KeyCode::KeyR) {
        commands.trigger(ReloadChunks);
    }
}
