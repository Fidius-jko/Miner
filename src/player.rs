use crate::settings::controls::*;
use crate::{resources::*, settings::ScaleSize};
use bevy::{asset::LoadState, input::mouse::MouseWheel, prelude::*};

const PLAYER_LOAD_NAME: &str = "player";
const PLAYER_TSET: &str = "Graphics/robot.tset.ron";

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerAssets>()
            .register_type::<Player>()
            .add_systems(OnEnter(crate::GameState::Playing), setup_player)
            .add_systems(
                Update,
                (
                    check_load.run_if(in_state(crate::GameState::Loading)),
                    (move_player, scale_cam).run_if(in_state(crate::GameState::Playing)),
                ),
            )
            .add_systems(OnEnter(crate::GameState::Loading), load_assets);
    }
}

// logic
fn setup_player(mut commands: Commands, assets: Res<PlayerAssets>) {
    commands.spawn((
        Player { speed: 5. },
        SpriteSheetBundle {
            transform: Transform::from_xyz(0., 0., 999.),
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2::new(64., 64.)),
                ..Default::default()
            },
            ..Default::default()
        },
        Name::new("Player"),
        TSetManager::new(assets.tileset.clone(), "left", TSetTile::Single),
    ));
}

#[derive(Component, Reflect)]
struct Player {
    speed: f32,
}

fn move_player(
    mut player: Query<(&Player, &mut Transform, &mut TSetManager)>,
    mut cam: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    controls: Res<ControlBinds>,
) {
    let move_x = controls.is_pressed("move_right") as i32 - controls.is_pressed("move_left") as i32;
    let move_y = controls.is_pressed("move_up") as i32 - controls.is_pressed("move_down") as i32;

    let (pl, mut trans, mut manager) = player.single_mut();
    if move_x > 0 {
        manager.set_tile("right", TSetTile::Single);
    } else if move_x < 0 {
        manager.set_tile("left", TSetTile::Single);
    } else if move_y > 0 {
        manager.set_tile("up", TSetTile::Single);
    } else if move_y < 0 {
        manager.set_tile("down", TSetTile::Single);
    }

    trans.translation.x += move_x as f32 * pl.speed;
    trans.translation.y += move_y as f32 * pl.speed;

    let mut cam = cam.single_mut();
    cam.translation = Vec3::new(trans.translation.x, trans.translation.y, cam.translation.z);
}

#[derive(Component)]
pub struct PlayerCamera {
    scale: f32,
}
impl Default for PlayerCamera {
    fn default() -> Self {
        Self { scale: 1. }
    }
}

fn scale_cam(
    mut cam: Query<(&mut Transform, &mut PlayerCamera), (With<Camera>, Without<Player>)>,
    mut scroll_evr: EventReader<MouseWheel>,
    scale: Res<ScaleSize>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    let (mut trans, mut cam) = cam.single_mut();
    for ev in scroll_evr.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                cam.scale -= scale.0 as f32 * ev.y;
            }
            MouseScrollUnit::Pixel => {
                cam.scale -= scale.0 as f32 * ev.y;
            }
        }
    }
    if cam.scale < 55. {
        cam.scale = 55.;
    }
    if cam.scale > 200. {
        cam.scale = 200.;
    }
    trans.scale = Vec3::new(cam.scale / 100., cam.scale / 100., 1.);
}

// PL assets
#[derive(Resource, Reflect)]
struct PlayerAssets {
    tileset: Handle<crate::resources::TextureSetAsset>,
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut load: ResMut<crate::LoadProcess>,
) {
    commands.insert_resource(PlayerAssets {
        tileset: asset_server.load(PLAYER_TSET),
    });
    load.add(PLAYER_LOAD_NAME);
}
fn check_load(
    mut assets: ResMut<PlayerAssets>,
    server: Res<AssetServer>,
    atlases: ResMut<'_, Assets<TextureAtlas>>,
    images: ResMut<'_, Assets<Image>>,
    mut tsets: ResMut<Assets<crate::resources::TextureSetAsset>>,
    mut load: ResMut<crate::LoadProcess>,
) {
    if let Some(s) = server.get_load_state(assets.tileset.clone()) {
        match s {
            LoadState::NotLoaded => {
                assets.tileset = server.load(PLAYER_TSET);
            }
            LoadState::Loading => {}
            LoadState::Failed => {
                warn!("Failed to load player texture set, using default");
                assets.tileset = tsets.add(TextureSetAsset::default(server, atlases));
                load.set(PLAYER_LOAD_NAME);
            }
            LoadState::Loaded => {
                let tset = tsets.get_mut(assets.tileset.clone()).unwrap();
                tset.check_or_build(images, atlases);
                load.set(PLAYER_LOAD_NAME);
            }
        }
    }
}
