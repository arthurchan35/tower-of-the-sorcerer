use serde::Deserialize;

use super::resource_info::ResourceKind;

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct MapResourceInfo {
    #[serde(rename = "Level")]
    pub level: i32,
    #[serde(rename = "Type")]
    pub kind: ResourceKind,
    #[serde(rename = "ID")]
    pub id: u32,
    #[serde(rename = "Point")]
    pub point: Point,
}

/// Shared envelope used by every JSON data file in `assets/data/`:
/// `{"target": [ ... ]}`. Deliberately generic so the same wrapper works for
/// map / property / enemy-stats files.
#[derive(Deserialize)]
pub(crate) struct JsonList<T> {
    pub target: Vec<T>,
}

pub fn load_map(json: &str) -> anyhow::Result<Vec<MapResourceInfo>> {
    let raw: JsonList<MapResourceInfo> = serde_json::from_str(json)?;
    Ok(raw.target)
}

pub fn filter_floor(all: &[MapResourceInfo], level: i32) -> Vec<MapResourceInfo> {
    all.iter().copied().filter(|m| m.level == level).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_map_json_has_floor_one() {
        let json = include_str!("../../assets/data/map.json");
        let all = load_map(json).expect("map.json must parse");
        let floor1 = filter_floor(&all, 1);
        assert!(!floor1.is_empty(), "expected at least one tile on floor 1");
        let actor_count = floor1
            .iter()
            .filter(|m| matches!(m.kind, ResourceKind::Actor) && m.id == 1)
            .count();
        assert_eq!(actor_count, 1, "floor 1 must place exactly one player");
    }
}
