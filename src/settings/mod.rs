use bevy::{prelude::*, utils::HashMap};

pub mod controls;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(controls::ControlsPlugin);
    }
}
