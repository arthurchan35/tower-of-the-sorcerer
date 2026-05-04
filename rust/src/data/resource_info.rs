use std::collections::HashMap;

use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize_repr)]
#[repr(u8)]
pub enum ResourceKind {
    Environment = 0,
    Item = 1,
    Actor = 2,
    Enemy = 3,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)] // name/info/path/icon_path will be used once artwork is wired up
pub struct ResourceInfo {
    #[serde(rename = "Type")]
    pub kind: ResourceKind,
    #[serde(rename = "ID")]
    pub id: u32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Info", default)]
    pub info: String,
    #[serde(rename = "Path", default)]
    pub path: String,
    #[serde(rename = "IconPath", default)]
    pub icon_path: String,
}

use super::map_info::JsonList;

pub type PropertyTable = HashMap<(ResourceKind, u32), ResourceInfo>;

pub fn load_properties(json: &str) -> anyhow::Result<PropertyTable> {
    let raw: JsonList<ResourceInfo> = serde_json::from_str(json)?;
    Ok(raw
        .target
        .into_iter()
        .map(|r| ((r.kind, r.id), r))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_properties_json_parses() {
        let json = include_str!("../../assets/data/properties.json");
        let table = load_properties(json).expect("properties.json must parse");
        assert!(!table.is_empty(), "expected at least one resource entry");
    }
}
