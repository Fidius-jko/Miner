use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
pub struct Plugins;

impl Plugin for Plugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            crate::resources::ResourcesPlugin,
            crate::player::PlayerPlugin,
            crate::map::MapPlugin,
            crate::settings::SettingsPlugin,
            crate::LoadPlugin,
            #[cfg(debug_assertions)]
            bevy_inspector_egui::quick::WorldInspectorPlugin::default()
                .run_if(input_toggle_active(true, KeyCode::Escape)),
        ));
    }
}
