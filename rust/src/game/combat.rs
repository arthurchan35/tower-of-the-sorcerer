use crate::data::EnemyStats;

use super::stats::PlayerStats;

#[derive(Clone, Copy, Debug)]
pub enum CombatOutcome {
    /// Hero kills the enemy. State is mutated by the caller using these deltas.
    PlayerWins { gold_gained: i32, hp_lost: i32 },
    /// Hero's attack does not exceed enemy defense — fight cannot be won.
    /// The caller should refuse to enter the fight.
    Impossible,
    /// Hero would die before killing the enemy. State is not mutated.
    PlayerDies { hp_needed: i32 },
}

/// Deterministic turn-based combat: hero strikes first, both sides apply
/// `max(0, atk - def)` per round until one HP reaches 0. The hero takes one
/// fewer counter-attack than the number of hits required to kill the enemy.
pub fn resolve_combat(player: &PlayerStats, enemy: &EnemyStats) -> CombatOutcome {
    let player_dmg = (player.attack - enemy.defense).max(0);
    if player_dmg == 0 {
        return CombatOutcome::Impossible;
    }
    let enemy_dmg = (enemy.attack - player.defense).max(0);
    let hits_to_kill = (enemy.health + player_dmg - 1) / player_dmg;
    let hp_lost = enemy_dmg.saturating_mul(hits_to_kill - 1);

    if player.health - hp_lost <= 0 {
        CombatOutcome::PlayerDies {
            hp_needed: hp_lost - player.health + 1,
        }
    } else {
        CombatOutcome::PlayerWins {
            gold_gained: enemy.gold,
            hp_lost,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn green_slime_dies_in_one_hit_no_counter() {
        // Default hero (1000/100/100) vs green slime (35/18/1):
        //   player_dmg = 100 - 1 = 99
        //   hits_to_kill = ceil(35 / 99) = 1
        //   hp_lost = enemy_dmg * (hits_to_kill - 1) = 0
        let p = PlayerStats::default();
        let e = EnemyStats { health: 35, attack: 18, defense: 1, gold: 1 };
        match resolve_combat(&p, &e) {
            CombatOutcome::PlayerWins { gold_gained, hp_lost } => {
                assert_eq!(gold_gained, 1);
                assert_eq!(hp_lost, 0);
            }
            other => panic!("expected PlayerWins, got {other:?}"),
        }
    }

    #[test]
    fn impossible_when_attack_below_defense() {
        let p = PlayerStats { attack: 10, defense: 100, ..PlayerStats::default() };
        let e = EnemyStats { health: 50, attack: 50, defense: 100, gold: 0 };
        assert!(matches!(resolve_combat(&p, &e), CombatOutcome::Impossible));
    }

    #[test]
    fn lethal_fight_does_not_mutate() {
        // Glass-cannon hero against a strong enemy that requires multiple hits.
        let p = PlayerStats { health: 10, attack: 20, defense: 0, ..PlayerStats::default() };
        let e = EnemyStats { health: 100, attack: 50, defense: 5, gold: 999 };
        let CombatOutcome::PlayerDies { hp_needed } = resolve_combat(&p, &e) else {
            panic!("expected PlayerDies");
        };
        assert!(hp_needed > 0);
    }
}
