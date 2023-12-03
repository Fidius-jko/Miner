use std::path::Path;

use bevy::asset::{AssetPath, ReadAssetBytesError};
use bevy::render::texture::{CompressedImageFormats, ImageType, TextureError};
use bevy::utils::hashbrown::HashMap;
use bevy::utils::thiserror;
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use futures_lite::AsyncReadExt;
use serde::Deserialize;
use thiserror::Error;

const DEFAULT_TEXTURE: &str = "embedded://miner/resources/default.png";

pub struct TSetPlugin;

impl Plugin for TSetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TextureSetAsset>()
            .init_asset_loader::<TextureSetLoader>()
            .register_type::<TSetManager>()
            .add_systems(PostUpdate, update);
        // .add_systems(Update, update);
    }
}
#[derive(Deserialize, Debug, Clone)]
enum SourceConfig {
    TextureAtlas {
        source: String,
        rows: u32,
        columns: u32,
        tile_size: Vec2,
    },
    Texture {
        source: String,
    },
}
#[derive(Deserialize, Debug, Clone)]
pub enum SourceLink {
    Atlas { source: String, index: Vec2 },
    Texture { source: String },
}

type TileGroup = HashMap<String, Tile>;

#[derive(Deserialize, Debug, Clone)]
pub enum Tile {
    Animated {
        frames: Vec<SourceLink>,
        speed: f32,
    },
    Single(SourceLink),
    Variant {
        variants: HashMap<String, SourceLink>,
    },
    Group(TileGroup),
}

#[derive(Deserialize, Debug, Clone)]
struct RonTextureSetAsset {
    sources: HashMap<String, SourceConfig>,
    textures: TileGroup,
}
#[derive(Debug, Clone)]
pub enum PreSource {
    TextureAtlas {
        img: Image,
        hdl: Handle<Image>,
        rows: u32,
        columns: u32,
        tile_size: Vec2,
    },
    Texture {
        img: Image,
        hdl: Handle<Image>,
    },
}
#[derive(Debug)]
pub enum SourceInfo {
    Atlas {
        offset: usize,
        rows: u32,
        columns: u32,
    },
    Texture(usize),
}

#[derive(Asset, TypePath, Debug)]
pub struct TextureSetAsset {
    sources: Option<(Handle<TextureAtlas>, HashMap<String, SourceInfo>)>,
    textures: TileGroup,
    default: Handle<TextureAtlas>,
    pre_src: Option<HashMap<String, PreSource>>,
}

impl TextureSetAsset {
    pub fn get_tile(&self, path: &str) -> Tile {
        match self.get_tile_(path) {
            Some(t) => {
                return t;
            }
            None => {
                warn!("Not found texture using default");
                return Tile::Single(SourceLink::Texture {
                    source: String::from("_@default"),
                });
            }
        }
    }
    fn get_tile_(&self, path: &str) -> Option<Tile> {
        let mut path = path.split('/');
        let mut now = self.textures.get(path.next()?)?;
        for i in path {
            if let Tile::Group(g) = now {
                now = g.get(i)?;
            }
        }
        Some(now.clone())
    }
    pub fn index_and_atlas(&self, link: SourceLink) -> (usize, Handle<TextureAtlas>) {
        let (atlas, offsets) = self.sources.as_ref().unwrap();
        let mut atlas = atlas.clone();
        let mut index = 0;
        if let SourceLink::Texture { source } = link {
            match offsets.get(&source) {
                Some(SourceInfo::Texture(offset)) => {
                    index = *offset;
                }
                _ => {
                    if source == "_@default" {
                        index = 0;
                        atlas = self.default.clone();
                    } else {
                        panic!("Source from link is not exist");
                    }
                }
            }
        } else if let SourceLink::Atlas {
            source,
            index: index2,
        } = link
        {
            match offsets.get(&source) {
                Some(SourceInfo::Atlas {
                    offset,
                    rows: _,
                    columns,
                }) => {
                    index = offset
                        + (index2.y - 1.) as usize * *columns as usize
                        + (index2.x - 1.) as usize;
                }
                _ => {
                    if source == "_@default" {
                        index = 0;
                        atlas = self.default.clone();
                    } else {
                        panic!("Source from link is not exist");
                    }
                }
            }
        }
        (index, atlas.clone())
    }

    pub fn check_or_build(
        &mut self,
        mut assets: ResMut<Assets<Image>>,
        mut atlases: ResMut<Assets<TextureAtlas>>,
    ) {
        if self.pre_src.is_none() {
            return;
        }
        let pre_src = self.pre_src.as_ref().unwrap();
        let mut builder = TextureAtlasBuilder::default();

        for (_src_name, i) in pre_src.iter() {
            match i {
                PreSource::TextureAtlas {
                    img,
                    hdl,
                    rows: _,
                    columns: _,
                    tile_size: _,
                } => {
                    builder.add_texture(hdl.id(), img);
                }
                PreSource::Texture { img, hdl } => {
                    builder.add_texture(hdl.id(), img);
                }
            }
        }

        let mut texture_atlas = builder
            .finish(&mut assets)
            .expect("Cannot build texture atlas");
        let len = texture_atlas.textures.len();
        let mut index = len;
        let mut src_info = HashMap::new();
        for (src_name, i) in pre_src.iter() {
            match i {
                PreSource::TextureAtlas {
                    img: _,
                    hdl,
                    rows,
                    columns,
                    tile_size,
                } => {
                    let index_offset = texture_atlas.get_texture_index(hdl.id()).unwrap();
                    let rect = texture_atlas.textures.get(index_offset).unwrap();
                    let rects = rects(*tile_size, *rows, *columns, rect.min);
                    src_info.insert(
                        src_name.clone(),
                        SourceInfo::Atlas {
                            offset: index,
                            rows: *rows,
                            columns: *columns,
                        },
                    );
                    index += rects.len();
                    for i in rects.iter() {
                        texture_atlas.add_texture(*i);
                    }

                    // src_info.insert(
                    //     src_name.clone(),
                    //     SourceInfo::Texture(texture_atlas.get_texture_index(hdl.id()).unwrap()),
                    // );
                }
                PreSource::Texture { img: _, hdl } => {
                    src_info.insert(
                        src_name.clone(),
                        SourceInfo::Texture(texture_atlas.get_texture_index(hdl.id()).unwrap()),
                    );
                }
            }
        }
        let hdl = atlases.add(texture_atlas);
        self.sources = Some((hdl, src_info));
        self.pre_src = None;
    }
    pub fn default(
        asset_server: Res<AssetServer>,
        mut atlases: ResMut<Assets<TextureAtlas>>,
    ) -> Self {
        let mut sources = HashMap::new();
        sources.insert(String::from("@default"), SourceInfo::Texture(0));
        let atlas = TextureAtlas::from_grid(
            asset_server.load(DEFAULT_TEXTURE),
            Vec2::new(16., 16.),
            1,
            1,
            None,
            None,
        );
        let hdl = atlases.add(atlas);
        Self {
            sources: Some((hdl.clone(), sources)),
            textures: HashMap::new(),
            default: hdl,
            pre_src: None,
        }
    }
}

#[derive(Default)]
struct TextureSetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TextureSetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
    #[error("Read image error: {0}")]
    ReadAssetBytesError(#[from] ReadAssetBytesError),
    #[error("Failed to make image from bytes: {0}")]
    TextureError(#[from] TextureError),
}

impl AssetLoader for TextureSetLoader {
    type Asset = TextureSetAsset;
    type Settings = ();
    type Error = TextureSetLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let path2 = load_context.path().to_str().unwrap().to_string();

            let mut path = String::new();
            match path2.rsplit_once('/') {
                Some(s) => {
                    path = s.0.to_string() + "/";
                }
                None => path = String::from(""),
            }
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let custom_asset = ron::de::from_bytes::<RonTextureSetAsset>(&bytes)?;
            let mut pre_src = HashMap::new();

            for (src_name, cfg) in custom_asset.sources.iter() {
                match cfg {
                    SourceConfig::TextureAtlas {
                        source,
                        rows,
                        columns,
                        tile_size,
                    } => {
                        let img = load_image(load_context, path.to_string() + &source).await?;
                        let hdl = load_context
                            .add_labeled_asset(format!("img_{}", src_name), img.clone());
                        pre_src.insert(
                            src_name.clone(),
                            PreSource::TextureAtlas {
                                img,
                                hdl,
                                rows: *rows,
                                columns: *columns,
                                tile_size: *tile_size,
                            },
                        );
                    }
                    SourceConfig::Texture { source } => {
                        let img = load_image(load_context, path.to_string() + &source).await?;
                        let hdl = load_context
                            .add_labeled_asset(format!("img_{}", src_name), img.clone());
                        pre_src.insert(src_name.clone(), PreSource::Texture { img, hdl });
                    }
                }
            }
            let atlas = TextureAtlas::from_grid(
                load_context.load(DEFAULT_TEXTURE),
                Vec2::new(16., 16.),
                1,
                1,
                None,
                None,
            );
            let hdl = load_context.add_labeled_asset("default_img".to_string(), atlas);

            Ok(Self::Asset {
                sources: None,
                textures: custom_asset.textures,
                default: hdl,
                pre_src: Some(pre_src),
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tset.ron", "TSet.ron", "tset"]
    }
}

async fn load_image<'a>(
    load_context: &'a mut LoadContext<'_>,
    source: String,
) -> Result<Image, TextureSetLoaderError> {
    let extension = source.rsplit_once('.').unwrap().1;
    let buffer = load_context
        .read_asset_bytes(AssetPath::from_path(Path::new(&source)))
        .await?;
    let image = Image::from_buffer(
        buffer.clone().as_slice(),
        ImageType::Extension(extension),
        CompressedImageFormats::default(),
        true,
        bevy::render::texture::ImageSampler::Default,
    )?;
    Ok(image)
}

fn rects(size: Vec2, rows: u32, columns: u32, offset: Vec2) -> Vec<Rect> {
    let mut sprites = Vec::new();

    for y in 0..rows {
        for x in 0..columns {
            let cell = Vec2::new(x as f32, y as f32);

            let rect_min = (size) * cell + offset;

            sprites.push(Rect {
                min: rect_min,
                max: rect_min + size,
            });
        }
    }
    sprites
}

#[derive(Clone, Reflect)]
pub enum TSetTile {
    Single,
    Variant(String),
    Animated(i32),
}

#[derive(Component, Reflect)]
pub struct TSetManager {
    pub tset: Handle<TextureSetAsset>,
    tile_name: String,
    data: TSetTile,
}

impl TSetManager {
    pub fn new(tset: Handle<TextureSetAsset>, name: &str, data: TSetTile) -> Self {
        Self {
            tset,
            tile_name: name.to_string(),
            data,
        }
    }
    pub fn set_tile(&mut self, name: &str) {
        self.tile_name = name.to_string();
    }
}

fn update(
    mut query: Query<
        (
            &TSetManager,
            &mut Handle<TextureAtlas>,
            &mut TextureAtlasSprite,
        ),
        Changed<TSetManager>,
    >,
    tsets: Res<Assets<TextureSetAsset>>,
) {
    query
        .par_iter_mut()
        .for_each(
            |(manager, mut atlas, mut sprite)| match tsets.get(manager.tset.clone()) {
                Some(n) => match n.get_tile(&manager.tile_name) {
                    Tile::Animated { frames, speed: _ } => {
                        if let TSetTile::Animated(frame) = manager.data {
                            if frames.len() > 0 {
                                let (index, atlas2) = n.index_and_atlas(
                                    frames.get(frame as usize % frames.len()).unwrap().clone(),
                                );
                                *atlas = atlas2;
                                sprite.index = index;
                            } else {
                                warn!("Animated tile doesn't have frames! Using default");

                                let (index, atlas2) = n.index_and_atlas(SourceLink::Texture {
                                    source: "_@default".to_string(),
                                });
                                *atlas = atlas2;
                                sprite.index = index;
                            }
                        } else {
                            warn!("Uncorrect tile!");
                        }
                    }
                    Tile::Single(link) => {
                        let (index, atlas2) = n.index_and_atlas(link.clone());
                        *atlas = atlas2;
                        sprite.index = index;
                    }
                    Tile::Group(_map) => {
                        warn!("Tile is group. Don't use tile. Using default");
                        let (index, atlas2) = n.index_and_atlas(SourceLink::Texture {
                            source: "_@default".to_string(),
                        });
                        *atlas = atlas2;
                        sprite.index = index;
                    }
                    Tile::Variant { variants } => {
                        if let TSetTile::Variant(name) = manager.data.clone() {
                            let variant = match variants.get(&name.clone()) {
                                Some(var) => var.clone(),
                                None => {
                                    warn!("Variant is not exist. Using default");
                                    SourceLink::Texture {
                                        source: "_@default".to_string(),
                                    }
                                }
                            };
                            let (index, atlas2) = n.index_and_atlas(variant.clone());
                            *atlas = atlas2;
                            sprite.index = index;
                        } else {
                            warn!("Uncorrect tile");
                        }
                    }
                },
                None => {
                    warn!("texture set is not loaded!");
                }
            },
        );
}
