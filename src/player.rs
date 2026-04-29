use crate::realm::chunk_loading::ReloadChunks;
use bevy::camera_controller::free_camera::FreeCamera;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::{AutoPlugin, auto_resource, auto_system};

#[derive(AutoPlugin)]
#[auto_plugin(impl_plugin_trait)]
pub struct PlayerPlugin;

#[derive(Component, Default)]
pub struct Player;

#[auto_system(plugin = PlayerPlugin, schedule = Startup)]
fn setup(mut commands: Commands) {
    let translation = vec3(0.0, 32.0, 0.0);
    commands.spawn((
        Player,
        Name::new("Player"),
        Transform::default().with_translation(translation),
        Camera3d::default(),
        FreeCamera {
            walk_speed: 20.0,
            run_speed: 40.0,
            ..default()
        },
    ));

    let chunk_pos = PlayerChunk::translation_to_chunk_pos(translation);
    commands.trigger(PlayerChunkChanged {
        old_chunk: chunk_pos,
        new_chunk: chunk_pos,
    });
    commands.trigger(ReloadChunks);
}

#[derive(Resource, Default)]
#[auto_resource(plugin = PlayerPlugin, init)]
pub struct PlayerChunk {
    pub pos: IVec3,
}

impl PlayerChunk {
    pub fn translation_to_chunk_pos(translation: Vec3) -> IVec3 {
        IVec3 {
            x: translation.x as i32 / 32,
            y: translation.y as i32 / 32,
            z: translation.z as i32 / 32,
        }
    }
}

#[derive(Event, Eq, PartialEq, Clone, Copy)]
pub struct PlayerChunkChanged {
    pub old_chunk: IVec3,
    pub new_chunk: IVec3,
}

#[auto_system(plugin = PlayerPlugin, schedule = PreUpdate)]
fn player_chunk_changed(
    mut commands: Commands,
    mut chunk: ResMut<PlayerChunk>,
    player: Single<&Transform, (With<Player>, Changed<Transform>)>,
) {
    let pos = PlayerChunk::translation_to_chunk_pos(player.translation);

    if chunk.pos != pos {
        commands.trigger(PlayerChunkChanged {
            old_chunk: chunk.pos,
            new_chunk: pos,
        });

        debug!("Player now in chunk {pos}");

        chunk.pos = pos;
    }
}
