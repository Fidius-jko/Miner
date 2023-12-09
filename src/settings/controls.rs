use bevy::prelude::*;

use crate::settings::*;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_controls)
            .add_systems(PreUpdate, check_controls)
            .init_resource::<ControlBinds>()
            .register_type::<ControlBinds>();
    }
}

fn setup_controls(mut binds: ResMut<ControlBinds>) {
    binds.set("move_up", Bind::Key(KeyCode::W));
    binds.set("move_down", Bind::Key(KeyCode::S));
    binds.set("move_left", Bind::Key(KeyCode::A));
    binds.set("move_right", Bind::Key(KeyCode::D));
}

#[derive(Clone, Copy, Reflect)]
pub enum Bind {
    Key(KeyCode),
    Mouse(MouseButton),
    None,
}

#[derive(Reflect)]
pub enum IsRun {
    Not,
    ReleaseRun,
    Run,
    OnceRun,
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ControlBinds {
    binds: HashMap<String, (IsRun, Bind)>,
}

impl ControlBinds {
    pub fn register(&mut self, name: &str) {
        self.binds
            .insert(String::from(name), (IsRun::Not, Bind::None));
    }
    pub fn set(&mut self, name: &str, bind: Bind) {
        match self.binds.get_mut(name) {
            Some((_, s)) => {
                *s = bind;
            }
            None => {
                self.binds.insert(String::from(name), (IsRun::Not, bind));
                info!("Control is not found, register it: {name}");
            }
        }
    }
    pub fn is_pressed(&self, name: &str) -> bool {
        match self.binds.get(name) {
            Some((is, _s)) => !matches!(is, IsRun::Not),
            None => {
                warn!("Control is not registered. Register it!");
                false
            }
        }
    }
    pub fn is_just_pressed(&self, name: &str) -> bool {
        match self.binds.get(name) {
            Some((is, _s)) => matches!(is, IsRun::OnceRun),
            None => {
                warn!("Control is not registered. Register it!");
                false
            }
        }
    }
    pub fn is_just_released(&self, name: &str) -> bool {
        match self.binds.get(name) {
            Some((is, _s)) => matches!(is, IsRun::ReleaseRun),
            None => {
                warn!("Control is not registered. Register it!");
                false
            }
        }
    }
}

fn check_controls(
    mut binds: ResMut<ControlBinds>,
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
) {
    for (_k, (i2, i)) in binds.binds.iter_mut() {
        match i {
            Bind::Key(code) => {
                if keys.just_pressed(*code) {
                    *i2 = IsRun::OnceRun;
                } else if keys.pressed(*code) {
                    *i2 = IsRun::Run;
                } else if keys.just_released(*code) {
                    *i2 = IsRun::ReleaseRun;
                } else {
                    *i2 = IsRun::Not;
                }
            }
            Bind::Mouse(code) => {
                if buttons.just_pressed(*code) {
                    *i2 = IsRun::OnceRun;
                } else if buttons.pressed(*code) {
                    *i2 = IsRun::Run;
                } else if buttons.just_released(*code) {
                    *i2 = IsRun::ReleaseRun;
                } else {
                    *i2 = IsRun::Not;
                }
            }
            _ => {}
        }
    }
}
