use crate::GameState;
use bevy::{prelude::*, utils::HashMap};

const FONT: &str = "fonts/FiraSans-Bold.ttf";

#[derive(Resource)]
pub struct LoadProcess {
    steps: HashMap<String, bool>,
}
impl LoadProcess {
    pub fn add(&mut self, step_name: &str) {
        self.steps.insert(step_name.to_string(), false);
    }
    pub fn set(&mut self, step_name: &str) {
        *self.steps.get_mut(step_name).unwrap() = true;
    }
    fn new() -> Self {
        Self {
            steps: HashMap::new(),
        }
    }
}

pub struct LoadPlugin;

impl Plugin for LoadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoadProcess::new())
            .add_systems(OnEnter(GameState::Loading), spawn_progress_bar)
            .add_systems(
                Update,
                check_load_process.run_if(in_state(GameState::Loading)),
            )
            .add_systems(
                OnExit(GameState::Loading),
                (set_load_process, despawn_progress_bar),
            );
    }
}

#[derive(Component)]
struct ProgressBar;

fn spawn_progress_bar(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(FONT);
    commands
        .spawn(TextBundle::from_section(
            "0/0",
            TextStyle {
                font: font.clone(),
                font_size: 32.,
                color: Color::WHITE,
            },
        ))
        .insert(ProgressBar);
}
fn despawn_progress_bar(mut commands: Commands, query: Query<Entity, With<ProgressBar>>) {
    let id = query.single();
    commands.entity(id).despawn();
}

fn check_load_process(
    process: Res<LoadProcess>,
    mut state: ResMut<NextState<GameState>>,
    mut query: Query<&mut Text, With<ProgressBar>>,
) {
    let end = process.steps.len();
    let mut now = 0;

    for (_, i) in process.steps.iter() {
        if *i {
            now += 1;
        }
    }
    let mut text = query.single_mut();
    text.sections[0].value = format!("{now}/{end}");
    if now == end {
        state.set(GameState::Playing);
        info!("Loading is ended! {}", text.sections[0].value);
    }
}
fn set_load_process(mut process: ResMut<LoadProcess>) {
    process.steps = LoadProcess::new().steps;
}
