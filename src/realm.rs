pub mod block;
pub mod chunk;
pub mod chunk_loading;
pub mod generation;

use bevy_auto_plugin::prelude::AutoPlugin;

#[derive(AutoPlugin)]
#[auto_plugin(impl_plugin_trait)]
pub struct RealmPlugin;
