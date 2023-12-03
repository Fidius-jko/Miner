pub mod tset;
pub use tset::*;

use bevy::asset::embedded_asset;
use bevy::prelude::*;
pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "src/", "default.png");
        app.add_plugins(TSetPlugin);
    }
}
