use bevy::{prelude::*, utils::HashMap};

pub mod controls;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(controls::ControlsPlugin)
            .init_resource::<ScaleSize>()
            .register_type::<ScaleSize>();
    }
}

#[derive(Reflect, Resource)]
#[reflect(Default, Resource)]
pub struct ScaleSize(pub f32);
impl Default for ScaleSize {
    fn default() -> Self {
        Self(5.)
    }
}
