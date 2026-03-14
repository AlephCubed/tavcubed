mod chunk;

use crate::chunk::mesh::ChunkMeshPlugin;
use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ChunkMeshPlugin)
        .run()
}
