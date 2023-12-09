use bevy::{asset::LoadState, prelude::*};

use crate::{resources::*, GameState, Tick};

const MAP_LOAD_NAME: &str = "map";
const MAP_TSET: &str = "Graphics/tiles.tset.ron";
const MAP_SIZE: i32 = 100;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MapAssets>()
            .add_systems(OnEnter(crate::GameState::Playing), setup_map)
            .add_systems(
                Update,
                (
                    check_load.run_if(in_state(crate::GameState::Loading)),
                    update_tiles.run_if(in_state(GameState::Playing)),
                ),
            )
            .add_systems(OnEnter(crate::GameState::Loading), load_assets);
    }
}

#[derive(Component)]
pub struct Tile;

// logic
fn setup_map(mut commands: Commands, assets: Res<MapAssets>) {
    commands
        .spawn((
            TransformBundle {
                local: Transform::from_xyz(
                    -(MAP_SIZE * 32) as f32 / 2.,
                    -(MAP_SIZE * 32) as f32 / 2.,
                    100.,
                ),
                ..Default::default()
            },
            VisibilityBundle::default(),
            Name::new("Map"),
        ))
        .with_children(|parent| {
            for x in 0..MAP_SIZE {
                for y in 0..MAP_SIZE {
                    let rnd = rand::random::<u32>() % 5;
                    let mut manager = TSetManager::new(
                        assets.tileset.clone(),
                        "grass",
                        TSetTile::Variant("var1".to_string()),
                    );
                    if rnd == 1 {
                        manager.set_tile("converyor", TSetTile::Animated(1));
                    } else if rnd == 2 {
                        manager.set_tile("water", TSetTile::Animated(1));
                    } else if rnd == 3 {
                        manager.set_tile("grass_to_water/left", TSetTile::Animated(1))
                    } else if rnd == 4 {
                        manager.set_tile("grass", TSetTile::Variant("var2".to_string()));
                    }
                    parent.spawn((
                        SpriteSheetBundle {
                            transform: Transform::from_xyz(32. * x as f32, 32. * y as f32, 0.),
                            sprite: TextureAtlasSprite {
                                custom_size: Some(Vec2::new(32., 32.)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        Tile,
                        manager,
                    ));
                }
            }
        });
}
fn update_tiles(mut tiles: Query<&mut TSetManager, With<Tile>>, tick: Res<Tick>) {
    println!("{}", tick.0);
    tiles.par_iter_mut().for_each(|mut tile| {
        tile.update_frame(tick.0);
    });
}

// Map assets
#[derive(Resource, Reflect)]
struct MapAssets {
    tileset: Handle<crate::resources::TextureSetAsset>,
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut load: ResMut<crate::LoadProcess>,
) {
    commands.insert_resource(MapAssets {
        tileset: asset_server.load(MAP_TSET),
    });
    load.add(MAP_LOAD_NAME);
}
fn check_load(
    mut assets: ResMut<MapAssets>,
    server: Res<AssetServer>,
    atlases: ResMut<'_, Assets<TextureAtlas>>,
    images: ResMut<'_, Assets<Image>>,
    mut tsets: ResMut<Assets<crate::resources::TextureSetAsset>>,
    mut load: ResMut<crate::LoadProcess>,
) {
    if let Some(s) = server.get_load_state(assets.tileset.clone()) {
        match s {
            LoadState::NotLoaded => {
                assets.tileset = server.load(MAP_TSET);
            }
            LoadState::Loading => {}
            LoadState::Failed => {
                warn!("Failed to load player texture set, using default");
                assets.tileset = tsets.add(TextureSetAsset::default(server, atlases));
                load.set(MAP_LOAD_NAME);
            }
            LoadState::Loaded => {
                let tset = tsets.get_mut(assets.tileset.clone()).unwrap();
                tset.check_or_build(images, atlases);
                load.set(MAP_LOAD_NAME);
            }
        }
    }
}
