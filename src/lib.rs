pub mod player;
pub mod realm;

use crate::player::{Player, PlayerPlugin};
use crate::realm::block::BlockPlugin;
use crate::realm::chunk::debug::{OctreeDebug, OctreeDebugPlugin};
use crate::realm::chunk::mesh::ChunkLOD;
use crate::realm::chunk::{Chunk, OCTREE_DEPTH, Voxel};
use crate::realm::chunk_loading::{ChunkLoadingPlugin, ReloadChunks};
use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::log::LogPlugin;
use bevy::math::u8vec3;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use realm::chunk::mesh::ChunkMeshPlugin;
use realm::generation::ChunkGenerationPlugin;

pub fn app() -> App {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                filter: "tavcubed=debug".to_string(),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins((FreeCameraPlugin, WireframePlugin::default()))
    .add_plugins((
        PlayerPlugin,
        ChunkMeshPlugin,
        ChunkLoadingPlugin,
        ChunkGenerationPlugin,
        BlockPlugin,
        OctreeDebugPlugin,
    ))
    .add_systems(Update, (debug_keybinds, debug_place_block));

    app
}

fn debug_keybinds(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut octree_debug: ResMut<OctreeDebug>,
    mut lods: Query<&mut ChunkLOD>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        info!("Toggling Wireframe");
        wireframe_config.global = !wireframe_config.global;
    }

    if keyboard_input.pressed(KeyCode::SuperLeft) & keyboard_input.just_pressed(KeyCode::KeyR) {
        commands.trigger(ReloadChunks);
    }

    let nums = [
        KeyCode::Digit6,
        KeyCode::Digit5,
        KeyCode::Digit4,
        KeyCode::Digit3,
        KeyCode::Digit2,
        KeyCode::Digit1,
    ];

    for (index, num) in nums.iter().enumerate() {
        if keyboard_input.just_pressed(*num) {
            // Change all chunk's LOD.
            if keyboard_input.pressed(KeyCode::SuperLeft) {
                for mut lod in &mut lods {
                    if lod.get() != index {
                        lod.set(index)
                    }
                }
                info!("Changed LOD to {index}");
            }

            // Octree debug level.
            if keyboard_input.pressed(KeyCode::AltLeft) {
                if index == OCTREE_DEPTH + 1 {
                    octree_debug.reset();
                } else {
                    match keyboard_input.pressed(KeyCode::ShiftLeft) {
                        true => octree_debug.add(index as u32),
                        false => octree_debug.set(index as u32),
                    }
                }

                info!("Changed octree debug flags to {:?}", *octree_debug);
            }
        }
    }
}

fn debug_place_block(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<&Transform, With<Player>>,
    mut chunk: Single<&mut Chunk>,
) {
    if !keyboard_input.just_pressed(KeyCode::KeyG) {
        return;
    }

    let pos = u8vec3(
        player.translation.x as u8,
        player.translation.y as u8,
        player.translation.z as u8,
    );

    if pos.x < 32 && pos.y < 32 && pos.z < 32 {
        match chunk.get_pos(pos).is_some() {
            true => chunk.set_pos(pos, None),
            false => chunk.set_pos(pos, Voxel::new_unwrap(2).into()),
        };
    }
}
