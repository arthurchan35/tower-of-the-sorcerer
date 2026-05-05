use std::collections::HashMap;

use serde::Deserialize;

/// Schema of `assets/data/sprite_atlas.json`. Each entry is a named animation:
/// 1+ frames in the same spritesheet plus a frames-per-second cycle rate.
/// `fps == 0.0` means static (no cycling); a single-frame entry is a still image.
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
    /// Each `[col, row]` pair selects one cell of the sheet (0-indexed, top-left origin).
    pub frames: Vec<[u32; 2]>,
    #[serde(default)]
    pub fps: f32,
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

        let player = atlas.entries.get("player").expect("player entry");
        let walk = atlas.entries.get("player_walk").expect("player_walk entry");
        let slime = atlas.entries.get("green_slime").expect("green_slime entry");
        assert_eq!(player.frames.len(), 1, "idle is single frame");
        assert!(walk.frames.len() >= 2, "walk needs multiple frames");
        assert!(walk.fps > 0.0, "walk needs nonzero fps");
        assert_eq!(slime.frames.len(), 2, "slime bobbing is 2 frames");
        assert!(slime.fps > 0.0);
    }
}
