pub mod hud;
pub mod tiles;

use bevy::prelude::*;

use crate::app_state::AppState;
use crate::game::GameSetup;

/// Side length of one rendered tile in pixels. The map is `GRID_SIZE × GRID_SIZE`.
pub const TILE_PX: f32 = 40.0;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                tiles::spawn_camera_and_tiles.after(GameSetup),
                tiles::outfit_player.after(GameSetup),
                hud::spawn_hud,
            ),
        )
        .add_systems(
            Update,
            (
                tiles::despawn_cleared_tiles,
                tiles::sync_player_transform,
                hud::sync_stats,
                hud::sync_status,
            ),
        )
        .add_systems(OnEnter(AppState::FloorCleared), hud::announce_floor_cleared)
        .add_systems(OnEnter(AppState::GameOver), hud::announce_game_over);
    }
}
