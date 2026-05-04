use std::collections::HashMap;

use serde::Deserialize;

/// Schema of `assets/data/sprite_atlas.json`. Each `entry` names a logical
/// sprite (e.g. `"green_slime"`) and points at a (col, row) cell in one of the
/// declared `sheets`. All cells are square; the side length is `tile_size`.
#[derive(Clone, Debug, Deserialize)]
pub struct AtlasFile {
    pub tile_size: u32,
    pub sheets: HashMap<String, SheetDims>,
    pub entries: HashMap<String, AtlasEntry>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct SheetDims {
    pub columns: u32,
    pub rows: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AtlasEntry {
    pub sheet: String,
    pub col: u32,
    pub row: u32,
}

pub fn load_sprite_atlas(json: &str) -> anyhow::Result<AtlasFile> {
    Ok(serde_json::from_str(json)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_sprite_atlas_parses() {
        let json = include_str!("../../assets/data/sprite_atlas.json");
        let atlas = load_sprite_atlas(json).expect("sprite_atlas.json must parse");
        assert!(atlas.tile_size > 0);
        assert!(!atlas.sheets.is_empty());
        // Floor-1 must at minimum have the player and the green slime resolved.
        let player = atlas.entries.get("player").expect("player entry");
        let slime = atlas.entries.get("green_slime").expect("green_slime entry");
        assert!(atlas.sheets.contains_key(&player.sheet));
        assert!(atlas.sheets.contains_key(&slime.sheet));
    }
}
