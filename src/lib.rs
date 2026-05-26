mod debug;
pub mod player;
pub mod realm;

use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(long, env, default_value = "true")]
    vsync: bool,
}

pub fn app() -> App {
    let mut app = App::new();

    let args = Args::parse();

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: if args.vsync {
                        PresentMode::AutoVsync
                    } else {
                        PresentMode::AutoNoVsync
                    },
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins((FreeCameraPlugin, WireframePlugin::default()))
    .add_plugins((player::PlayerPlugin, realm::RealmPlugin, debug::DebugPlugin));

    app
}
