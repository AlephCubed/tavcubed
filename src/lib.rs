mod debug;
pub mod player;
pub mod realm;

use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;

pub fn app() -> App {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins((FreeCameraPlugin, WireframePlugin::default()))
    .add_plugins((player::PlayerPlugin, realm::RealmPlugin, debug::DebugPlugin));

    app
}
