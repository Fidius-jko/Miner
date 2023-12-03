use bevy::{asset::LoadState, prelude::*};

use crate::resources::*;

const MAP_LOAD_NAME: &str = "map";
const MAP_TSET: &str = "Graphics/tiles.tset.ron";

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MapAssets>()
            .add_systems(OnEnter(crate::GameState::Playing), setup_map)
            .add_systems(
                Update,
                (check_load.run_if(in_state(crate::GameState::Loading)),),
            )
            .add_systems(OnEnter(crate::GameState::Loading), load_assets);
    }
}

// logic
fn setup_map(mut commands: Commands, assets: Res<MapAssets>) {
    commands.spawn((
        SpriteSheetBundle {
            transform: Transform::from_xyz(0., 0., 100.),
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2::new(32., 32.)),
                ..Default::default()
            },
            ..Default::default()
        },
        TSetManager::new(
            assets.tileset.clone(),
            "grass",
            TSetTile::Variant("var1".to_string()),
        ),
    ));
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
    match server.get_load_state(assets.tileset.clone()) {
        Some(s) => match s {
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
        },
        _ => {}
    }
}
