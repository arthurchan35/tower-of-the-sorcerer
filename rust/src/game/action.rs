use super::combat::{resolve_combat, CombatOutcome};
use super::floor::{Direction, DoorColor, Floor, ItemKind, Tile, TilePos};
use super::stats::PlayerStats;

#[derive(Clone, Debug)]
pub enum ActionResult {
    Moved { to: TilePos },
    Bumped,
    Fought { at: TilePos, outcome: CombatOutcome },
    PickedUp { at: TilePos, item: ItemKind },
    OpenedDoor { at: TilePos, color: DoorColor },
    LockedDoor { color: DoorColor },
    EnteredStairs,
}

/// Resolve one player input into a single discrete game step. Mutates the
/// floor and player state in place; returns a description of what happened
/// for the UI / status layer to consume.
pub fn step_player(
    floor: &mut Floor,
    stats: &mut PlayerStats,
    pos: &mut TilePos,
    dir: Direction,
) -> ActionResult {
    let to = pos.step(dir);
    if !to.in_bounds() {
        return ActionResult::Bumped;
    }

    match floor.get(to).copied() {
        None => {
            *pos = to;
            ActionResult::Moved { to }
        }
        Some(Tile::Wall) => ActionResult::Bumped,
        Some(Tile::Door(color)) => {
            let key = match color {
                DoorColor::Yellow => &mut stats.yellow_keys,
                DoorColor::Blue => &mut stats.blue_keys,
                DoorColor::Red => &mut stats.red_keys,
            };
            if *key > 0 {
                *key -= 1;
                floor.remove(to);
                *pos = to;
                ActionResult::OpenedDoor { at: to, color }
            } else {
                ActionResult::LockedDoor { color }
            }
        }
        Some(Tile::StairsUp) => {
            *pos = to;
            ActionResult::EnteredStairs
        }
        Some(Tile::Item(kind)) => {
            apply_item(stats, kind);
            floor.remove(to);
            *pos = to;
            ActionResult::PickedUp { at: to, item: kind }
        }
        Some(Tile::Enemy(enemy)) => {
            let outcome = resolve_combat(stats, &enemy.stats);
            if let CombatOutcome::PlayerWins { gold_gained, hp_lost } = outcome {
                stats.health -= hp_lost;
                stats.gold += gold_gained;
                floor.remove(to);
                *pos = to;
            }
            ActionResult::Fought { at: to, outcome }
        }
    }
}

fn apply_item(stats: &mut PlayerStats, kind: ItemKind) {
    match kind {
        ItemKind::YellowKey => stats.yellow_keys += 1,
        ItemKind::BlueKey => stats.blue_keys += 1,
        ItemKind::RedKey => stats.red_keys += 1,
        ItemKind::Potion(amount) => stats.health += amount,
        ItemKind::AttackGem(amount) => stats.attack += amount,
        ItemKind::DefenseGem(amount) => stats.defense += amount,
        ItemKind::Unimplemented => {}
    }
}
