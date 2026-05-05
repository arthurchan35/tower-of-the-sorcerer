pub mod animation;
pub mod hud;
pub mod sprite_catalog;
pub mod tiles;

use bevy::prelude::*;

use crate::app_state::AppState;
use crate::game::GameSetup;

/// Side length of one rendered tile in pixels. Source spritesheets are 32 px;
/// 64 keeps the scaling at an integer multiple (2×) for crisp pixel art when
/// `ImagePlugin::default_nearest()` is enabled in `main.rs`.
pub const TILE_PX: f32 = 64.0;

/// System set every render-side startup system runs in. The atlas-loading
/// system is in this set, so every other render setup system can `.after()`
/// the set and depend on `SpriteCatalog` being available.
#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RenderSetup;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                sprite_catalog::load_sprite_catalog
                    .in_set(RenderSetup)
                    .after(GameSetup),
                tiles::spawn_camera_and_tiles.after(RenderSetup),
                tiles::outfit_player.after(RenderSetup),
                hud::spawn_hud,
            ),
        )
        .add_systems(
            Update,
            (
                tiles::despawn_cleared_tiles,
                animation::start_player_walk,
                animation::tick_player_walk.after(animation::start_player_walk),
                animation::animate_sprites.after(animation::tick_player_walk),
                hud::sync_stats,
                hud::sync_status,
            ),
        )
        .add_systems(OnEnter(AppState::FloorCleared), hud::announce_floor_cleared)
        .add_systems(OnEnter(AppState::GameOver), hud::announce_game_over);
    }
}
