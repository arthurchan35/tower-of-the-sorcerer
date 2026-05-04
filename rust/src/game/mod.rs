pub mod action;
pub mod combat;
pub mod floor;
pub mod stats;

use std::collections::HashMap;

use bevy::prelude::*;

pub use action::{step_player, ActionResult};
pub use combat::{resolve_combat, CombatOutcome};
pub use floor::{Direction, DoorColor, EnemyTile, Floor, ItemKind, Tile, TilePos, GRID_SIZE};
pub use stats::PlayerStats;

use crate::app_state::AppState;
use crate::data::{self, EnemyStats, ResourceKind};
use crate::input::MoveIntent;

const FLOOR_TO_PLAY: i32 = 1;

#[derive(Component)]
pub struct Player;

#[derive(Default, Resource)]
pub struct StatusMessage(pub String);

/// Lookup table from monster ID to combat stats. Loaded once at plugin build
/// from `assets/data/enemy_stats.json`.
#[derive(Resource)]
pub struct EnemyStatsTable(pub HashMap<u32, EnemyStats>);

/// System set every game-startup system runs in. Render systems that need the
/// initial `Floor` and `Player` to exist depend on this set with `.after()`.
#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GameSetup;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let stats_json = include_str!("../../assets/data/enemy_stats.json");
        let stats_table = data::load_enemy_stats(stats_json)
            .expect("enemy_stats.json: failed to parse");

        app.init_state::<AppState>()
            .init_resource::<Floor>()
            .init_resource::<StatusMessage>()
            .insert_resource(EnemyStatsTable(stats_table))
            .add_systems(Startup, setup_floor.in_set(GameSetup))
            .add_systems(
                Update,
                advance_turn
                    .after(crate::input::keyboard_to_move_intent)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn setup_floor(
    mut commands: Commands,
    mut floor: ResMut<Floor>,
    mut status: ResMut<StatusMessage>,
    enemy_stats: Res<EnemyStatsTable>,
) {
    let map_json = include_str!("../../assets/data/map.json");
    let all_map = data::load_map(map_json).expect("map.json: failed to parse");
    let placements = data::filter_floor(&all_map, FLOOR_TO_PLAY);

    let mut player_pos = None;

    for m in &placements {
        let pos = TilePos::from_origin_centered(m.point.x, m.point.y);
        match m.kind {
            ResourceKind::Actor => {
                if m.id == 1 {
                    player_pos = Some(pos);
                }
            }
            ResourceKind::Environment => {
                if let Some(tile) = environment_tile(m.id) {
                    floor.set(pos, tile);
                }
            }
            ResourceKind::Item => {
                if let Some(kind) = item_kind(m.id) {
                    floor.set(pos, Tile::Item(kind));
                }
            }
            ResourceKind::Enemy => {
                if let Some(&stats) = enemy_stats.0.get(&m.id) {
                    floor.set(pos, Tile::Enemy(EnemyTile { id: m.id, stats }));
                }
            }
        }
    }

    let pos = player_pos.expect("no player placement on selected floor");
    commands.spawn((Player, PlayerStats::default(), pos));
    status.0 = format!("Floor {FLOOR_TO_PLAY} — find the stairs.");
}

fn advance_turn(
    mut intents: MessageReader<MoveIntent>,
    mut floor: ResMut<Floor>,
    mut status: ResMut<StatusMessage>,
    mut next_state: ResMut<NextState<AppState>>,
    mut player_q: Query<(&mut TilePos, &mut PlayerStats), With<Player>>,
) {
    let Ok((mut pos, mut stats)) = player_q.single_mut() else {
        return;
    };
    for intent in intents.read() {
        let result = step_player(&mut floor, &mut stats, &mut pos, intent.0);
        status.0 = describe(&result);
        if matches!(result, ActionResult::EnteredStairs) {
            next_state.set(AppState::FloorCleared);
        } else if stats.health <= 0 {
            next_state.set(AppState::GameOver);
        }
    }
}

fn describe(result: &ActionResult) -> String {
    match result {
        ActionResult::Moved { to } => format!("Moved to ({}, {}).", to.col, to.row),
        ActionResult::Bumped => "Blocked.".into(),
        ActionResult::OpenedDoor { at, color } => {
            format!("Opened {color:?} door at ({}, {}).", at.col, at.row)
        }
        ActionResult::LockedDoor { color } => format!("Need a {color:?} key."),
        ActionResult::PickedUp { at, item } => {
            format!("Picked up {item:?} at ({}, {}).", at.col, at.row)
        }
        ActionResult::EnteredStairs => "Floor cleared!".into(),
        ActionResult::Fought { at, outcome } => match outcome {
            CombatOutcome::PlayerWins { gold_gained, hp_lost } => format!(
                "Won at ({}, {}): -{hp_lost} HP, +{gold_gained} gold.",
                at.col, at.row
            ),
            CombatOutcome::Impossible => "Cannot damage that monster.".into(),
            CombatOutcome::PlayerDies { hp_needed } => {
                format!("That fight would kill you (need +{hp_needed} HP).")
            }
        },
    }
}

fn environment_tile(env_id: u32) -> Option<Tile> {
    Some(match env_id {
        1 => Tile::Door(DoorColor::Yellow),
        2 => Tile::Door(DoorColor::Blue),
        3 => Tile::Door(DoorColor::Red),
        6 => Tile::Wall,
        7 => Tile::StairsUp,
        _ => return None,
    })
}

fn item_kind(item_id: u32) -> Option<ItemKind> {
    Some(match item_id {
        1 => ItemKind::YellowKey,
        2 => ItemKind::BlueKey,
        3 => ItemKind::RedKey,
        5 => ItemKind::Potion(200),
        6 => ItemKind::Potion(500),
        7 => ItemKind::AttackGem(3),
        8 => ItemKind::DefenseGem(3),
        9 => ItemKind::Unimplemented,
        _ => return None,
    })
}
