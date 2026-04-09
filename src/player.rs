use bevy::camera_controller::free_camera::FreeCamera;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .init_resource::<PlayerChunk>()
            .add_systems(Update, player_chunk_changed);
    }
}

#[derive(Component, Default)]
pub struct Player;

fn setup(mut commands: Commands) {
    commands.spawn((
        Player,
        Name::new("Player"),
        Transform::default().with_translation(vec3(0.0, 32.0, 0.0)),
        Camera3d::default(),
        FreeCamera {
            walk_speed: 20.0,
            run_speed: 40.0,
            ..default()
        },
    ));
}

#[derive(Resource, Default)]
pub struct PlayerChunk {
    pub pos: IVec3,
}

#[derive(Event, Eq, PartialEq, Clone, Copy)]
pub struct PlayerChunkChanged {
    pub old_chunk: IVec3,
    pub new_chunk: IVec3,
}

fn player_chunk_changed(
    mut commands: Commands,
    mut chunk: ResMut<PlayerChunk>,
    player: Single<&Transform, (With<Player>, Changed<Transform>)>,
) {
    let pos = IVec3 {
        x: player.translation.x as i32 / 32,
        y: player.translation.y as i32 / 32,
        z: player.translation.z as i32 / 32,
    };

    if chunk.pos != pos {
        commands.trigger(PlayerChunkChanged {
            old_chunk: chunk.pos,
            new_chunk: pos,
        });

        chunk.pos = pos;
    }
}
