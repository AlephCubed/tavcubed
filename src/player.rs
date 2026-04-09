use bevy::camera_controller::free_camera::FreeCamera;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
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
