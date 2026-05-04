use std::collections::HashMap;

use bevy::prelude::*;

use crate::data::{self, AtlasFile};

/// Per-sheet handles required to build a sprite that picks one frame out of
/// a spritesheet via Bevy's `TextureAtlas` API.
struct SheetHandles {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

struct AtlasRef {
    sheet: String,
    index: usize,
}

/// Resource resolved at startup from `assets/data/sprite_atlas.json`. Render
/// systems call `make_sprite(key, size_px)` to produce a `Sprite` for the
/// named entity (`"player"`, `"green_slime"`, `"yellow_key"`, etc.).
#[derive(Resource)]
pub struct SpriteCatalog {
    sheets: HashMap<String, SheetHandles>,
    entries: HashMap<String, AtlasRef>,
}

impl SpriteCatalog {
    pub fn make_sprite(&self, key: &str, size_px: f32) -> Option<Sprite> {
        let entry = self.entries.get(key)?;
        let handles = self.sheets.get(&entry.sheet)?;
        Some(Sprite {
            image: handles.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: handles.layout.clone(),
                index: entry.index,
            }),
            custom_size: Some(Vec2::splat(size_px)),
            ..default()
        })
    }
}

pub fn load_sprite_catalog(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let json = include_str!("../../assets/data/sprite_atlas.json");
    let atlas: AtlasFile = data::load_sprite_atlas(json).expect("sprite_atlas.json: parse failed");

    let mut sheets = HashMap::new();
    for (name, dims) in &atlas.sheets {
        let image: Handle<Image> = asset_server.load(format!("sprites/{name}"));
        let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(atlas.tile_size),
            dims.columns,
            dims.rows,
            None,
            None,
        ));
        sheets.insert(name.clone(), SheetHandles { image, layout });
    }

    let entries = atlas
        .entries
        .into_iter()
        .filter_map(|(key, entry)| {
            let dims = atlas.sheets.get(&entry.sheet)?;
            let index = (entry.row * dims.columns + entry.col) as usize;
            Some((
                key,
                AtlasRef {
                    sheet: entry.sheet,
                    index,
                },
            ))
        })
        .collect();

    commands.insert_resource(SpriteCatalog { sheets, entries });
}
