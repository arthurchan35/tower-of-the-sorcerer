use std::collections::HashMap;

use bevy::prelude::*;

use crate::data::{self, AtlasFile};

/// Per-sheet handles required to address one frame inside a spritesheet.
struct SheetHandles {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    columns: u32,
}

struct ResolvedEntry {
    sheet: String,
    /// Pre-computed atlas indices (`row * columns + col`) so the render systems
    /// don't reinterpret `(col, row)` every frame.
    frame_indices: Vec<usize>,
    fps: f32,
}

/// Resolved sprite atlas — handles + frame indices for every named animation
/// declared in `assets/data/sprite_atlas.json`.
#[derive(Resource)]
pub struct SpriteCatalog {
    sheets: HashMap<String, SheetHandles>,
    entries: HashMap<String, ResolvedEntry>,
}

impl SpriteCatalog {
    /// Build a `Sprite` ready for spawning. The resulting sprite is parked on
    /// the first frame of the entry's animation; cycling is the responsibility
    /// of the animation system.
    pub fn make_sprite(&self, key: &str, size_px: f32) -> Option<Sprite> {
        let entry = self.entries.get(key)?;
        let handles = self.sheets.get(&entry.sheet)?;
        Some(Sprite {
            image: handles.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: handles.layout.clone(),
                index: *entry.frame_indices.first()?,
            }),
            custom_size: Some(Vec2::splat(size_px)),
            ..default()
        })
    }

    /// `(frame_indices, fps)` for the animation; `None` if the key isn't known.
    pub fn animation(&self, key: &str) -> Option<(Vec<usize>, f32)> {
        let entry = self.entries.get(key)?;
        Some((entry.frame_indices.clone(), entry.fps))
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
        sheets.insert(
            name.clone(),
            SheetHandles {
                image,
                layout,
                columns: dims.columns,
            },
        );
    }

    let mut entries = HashMap::new();
    for (key, entry) in atlas.entries {
        let Some(s) = sheets.get(&entry.sheet) else {
            warn!("sprite_atlas: entry {key:?} references unknown sheet {:?}", entry.sheet);
            continue;
        };
        let frame_indices = entry
            .frames
            .iter()
            .map(|[col, row]| (row * s.columns + col) as usize)
            .collect();
        entries.insert(
            key,
            ResolvedEntry {
                sheet: entry.sheet,
                frame_indices,
                fps: entry.fps,
            },
        );
    }

    commands.insert_resource(SpriteCatalog { sheets, entries });
}
