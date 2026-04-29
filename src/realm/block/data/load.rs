use crate::realm::block::BlockPlugin;
use crate::realm::block::data::registry::{BlockRegistry, BlockRegistryInner};
use crate::realm::block::data::{Block, BlockData, BlockTexture, BlockTextureData};
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};
use bevy_auto_plugin::prelude::auto_system;
use std::collections::HashMap;

const CORE_BLOCK_DATA_DIR: &str = "assets/blocks";

#[auto_system(plugin = BlockPlugin, schedule = Startup)]
pub fn load_core_blocks(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    info!("Loading core blocks");

    // Deserialize block configs.
    let mut block_data = Vec::new();

    for entry in std::fs::read_dir(CORE_BLOCK_DATA_DIR).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }

        let content = std::fs::read_to_string(&path).unwrap();
        let data: BlockData = toml::from_str(&content).unwrap();
        block_data.push(data);
    }

    debug!("Loaded {} core block configs", block_data.len());

    // Collect texture paths.
    let mut texture_map = TextureMap::default();

    let texture_refs: Vec<BlockTexture> = block_data
        .iter()
        .map(|block_data| match &block_data.texture {
            BlockTextureData::Uniform(path) => BlockTexture::Uniform(texture_map.resolve(path)),
            BlockTextureData::PerFace {
                top,
                bottom,
                right,
                left,
                back,
                front,
            } => BlockTexture::PerFace {
                top: texture_map.resolve(top),
                bottom: texture_map.resolve(bottom),
                right: texture_map.resolve(right),
                left: texture_map.resolve(left),
                back: texture_map.resolve(back),
                front: texture_map.resolve(front),
            },
        })
        .collect();

    // Load all images.
    let textures: Vec<image::RgbaImage> = texture_map
        .paths
        .iter()
        .map(|path| {
            image::open(format!("assets/{}", path))
                .unwrap_or_else(|_| panic!("Failed to load texture: {}", path))
                .into_rgba8()
        })
        .collect();

    let (width, height) = (textures[0].width(), textures[0].height());
    assert!(
        textures
            .iter()
            .all(|img| img.width() == width && img.height() == height),
        "All block textures must be the same resolution ({width}x{height})"
    );

    let num_textures = textures.len();
    debug!("Loaded {} block textures", num_textures);

    let raw_texture_data: Vec<u8> = textures
        .into_iter()
        .flat_map(|img| img.into_raw())
        .collect();

    let mut texture_array = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: num_textures as u32,
        },
        TextureDimension::D2,
        raw_texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    texture_array.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    });
    texture_array.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..default()
    });

    let mut registry = BlockRegistryInner::default();
    registry.textures = Some(images.add(texture_array));

    debug!("Created texture array");

    // Register all block types.
    for (block_data, texture) in block_data.into_iter().zip(texture_refs) {
        registry.register(Block {
            id: block_data.id,
            name: block_data.name,
            texture,
        });
    }

    commands.insert_resource(BlockRegistry::new(registry));

    debug!("Registered all core blocks");
}

#[derive(Default)]
struct TextureMap {
    map: HashMap<String, usize>,
    paths: Vec<String>,
}

impl TextureMap {
    pub fn resolve(&mut self, path: &str) -> u16 {
        if let Some(idx) = self.map.get(path) {
            return *idx as u16;
        };

        let idx = self.paths.len();
        self.map.insert(path.to_string(), idx);
        self.paths.push(path.to_string());
        idx as u16
    }
}
