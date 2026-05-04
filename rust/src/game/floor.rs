use std::collections::HashMap;

use bevy::prelude::*;

use crate::data::EnemyStats;

pub const GRID_SIZE: i32 = 11;

/// Integer tile coordinate within a single floor. Origin is bottom-left,
/// columns and rows run `0..GRID_SIZE`.
#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub struct TilePos {
    pub col: i32,
    pub row: i32,
}

impl TilePos {
    pub fn new(col: i32, row: i32) -> Self {
        Self { col, row }
    }

    /// Convert from origin-centered float coordinates (e.g. `(-5.0..=5.0)` on
    /// an 11×11 grid) to integer tile coordinates with origin at the bottom
    /// left.
    pub fn from_origin_centered(x: f32, y: f32) -> Self {
        let half = (GRID_SIZE as f32 - 1.0) * 0.5;
        Self::new((x + half).round() as i32, (y + half).round() as i32)
    }

    pub fn step(self, dir: Direction) -> Self {
        match dir {
            Direction::Up => Self::new(self.col, self.row + 1),
            Direction::Down => Self::new(self.col, self.row - 1),
            Direction::Left => Self::new(self.col - 1, self.row),
            Direction::Right => Self::new(self.col + 1, self.row),
        }
    }

    pub fn in_bounds(self) -> bool {
        (0..GRID_SIZE).contains(&self.col) && (0..GRID_SIZE).contains(&self.row)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoorColor {
    Yellow,
    Blue,
    Red,
}

#[derive(Clone, Copy, Debug)]
pub enum ItemKind {
    YellowKey,
    BlueKey,
    RedKey,
    Potion(i32),
    AttackGem(i32),
    DefenseGem(i32),
    /// Items whose effect we haven't implemented yet (e.g. teleporters).
    Unimplemented,
}

#[derive(Clone, Copy, Debug)]
pub struct EnemyTile {
    pub id: u32,
    pub stats: EnemyStats,
}

#[derive(Clone, Copy, Debug)]
pub enum Tile {
    Wall,
    Door(DoorColor),
    StairsUp,
    Item(ItemKind),
    Enemy(EnemyTile),
}

/// Sparse tile map for one floor — empty positions are walkable open ground.
#[derive(Default, Resource)]
pub struct Floor {
    tiles: HashMap<TilePos, Tile>,
}

impl Floor {
    pub fn set(&mut self, pos: TilePos, tile: Tile) {
        self.tiles.insert(pos, tile);
    }

    pub fn get(&self, pos: TilePos) -> Option<&Tile> {
        self.tiles.get(&pos)
    }

    pub fn remove(&mut self, pos: TilePos) -> Option<Tile> {
        self.tiles.remove(&pos)
    }

    pub fn iter(&self) -> impl Iterator<Item = (TilePos, &Tile)> + '_ {
        self.tiles.iter().map(|(&k, v)| (k, v))
    }
}
