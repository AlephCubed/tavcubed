use crate::realm::chunk::mesh::ChunkLOD;
use crate::realm::chunk::{Chunk, OCTREE_DEPTH};
use crate::realm::chunk_loading::ReloadChunks;
use crate::realm::voxel_query::VoxelQuery;
use bevy::camera_controller::free_camera::FreeCameraState;
use bevy::math::bounding::RayCast3d;
use bevy::pbr::wireframe::WireframeConfig;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use octree::OctreeDebug;

pub mod octree;

#[derive(AutoPlugin)]
#[auto_plugin(impl_plugin_trait)]
pub struct DebugPlugin;

#[auto_system(plugin = DebugPlugin, schedule = Update)]
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

#[auto_system(plugin = DebugPlugin, schedule = Update)]
fn debug_place(
    mouse_input: Res<ButtonInput<MouseButton>>,
    player: Single<(&Transform, &FreeCameraState)>,
    voxel_query: VoxelQuery,
) {
    let (transform, camera_state) = *player;

    if !camera_state.enabled {
        return;
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        let hit = voxel_query.cast_ray(RayCast3d::new(
            transform.translation,
            Dir3::new(transform.rotation * Vec3::Z).unwrap(),
            64.0,
        ));

        if let Some(hit) = hit {
            todo!()
        }
    }
}
