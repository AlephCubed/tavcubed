mod debug;
pub mod player;
pub mod realm;

use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;

pub fn app() -> App {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((FreeCameraPlugin, WireframePlugin::default()))
        .add_plugins((player::PlayerPlugin, realm::RealmPlugin, debug::DebugPlugin));

    app
}
