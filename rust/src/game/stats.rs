use bevy::prelude::*;

#[derive(Clone, Copy, Component, Debug)]
pub struct PlayerStats {
    pub health: i32,
    pub attack: i32,
    pub defense: i32,
    pub gold: i32,
    pub yellow_keys: u32,
    pub blue_keys: u32,
    pub red_keys: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            health: 1000,
            attack: 100,
            defense: 100,
            gold: 0,
            yellow_keys: 0,
            blue_keys: 0,
            red_keys: 0,
        }
    }
}
