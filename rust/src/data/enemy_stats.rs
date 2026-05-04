use std::collections::HashMap;

use serde::Deserialize;

use super::map_info::JsonList;

/// Combat stats for one monster type. The `name` field on disk is preserved
/// for human readability of the data file but is not used at runtime — gameplay
/// keys off the numeric ID.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct EnemyStats {
    pub health: i32,
    pub attack: i32,
    pub defense: i32,
    pub gold: i32,
}

#[derive(Deserialize)]
#[allow(dead_code)] // `name` and `id` are read structurally, not by name in code
struct Row {
    id: u32,
    #[serde(default)]
    name: String,
    health: i32,
    attack: i32,
    defense: i32,
    gold: i32,
}

pub fn load_enemy_stats(json: &str) -> anyhow::Result<HashMap<u32, EnemyStats>> {
    let raw: JsonList<Row> = serde_json::from_str(json)?;
    Ok(raw
        .target
        .into_iter()
        .map(|r| {
            (
                r.id,
                EnemyStats {
                    health: r.health,
                    attack: r.attack,
                    defense: r.defense,
                    gold: r.gold,
                },
            )
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_enemy_stats_json_loads() {
        let json = include_str!("../../assets/data/enemy_stats.json");
        let table = load_enemy_stats(json).expect("enemy_stats.json must parse");
        // The reference data has 34 monster types.
        assert!(table.len() >= 34, "expected at least 34 enemy entries");
        // ID 1 (Green Slime) is on floor 1 — sanity-check its numbers.
        let slime = table.get(&1).expect("enemy ID 1 must exist");
        assert_eq!(slime.health, 35);
        assert_eq!(slime.attack, 18);
    }
}
