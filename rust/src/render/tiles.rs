use bevy::prelude::*;

use crate::game::{DoorColor, Floor, ItemKind, Player, Tile, TilePos, GRID_SIZE};

use super::TILE_PX;

/// Marker component for the per-tile sprites spawned at the start of a floor.
/// Lets `despawn_cleared_tiles` filter positively instead of relying on
/// negative filters that could accidentally match the Player.
#[derive(Component)]
pub struct TileSprite;

/// Tile coord → world translation. The map is centered horizontally and
/// pushed down a bit to leave room for the HUD at the top of the window.
pub fn tile_to_world(pos: TilePos) -> Vec3 {
    let half = (GRID_SIZE as f32 - 1.0) * 0.5;
    let x = (pos.col as f32 - half) * TILE_PX;
    let y = (pos.row as f32 - half) * TILE_PX - 40.0;
    Vec3::new(x, y, 0.0)
}

pub fn spawn_camera_and_tiles(mut commands: Commands, floor: Res<Floor>) {
    commands.spawn(Camera2d);

    let span = GRID_SIZE as f32 * TILE_PX;
    commands.spawn((
        Sprite {
            color: Color::srgb(0.16, 0.13, 0.10),
            custom_size: Some(Vec2::splat(span)),
            ..default()
        },
        Transform::from_xyz(0.0, -40.0, -1.0),
    ));

    for (pos, tile) in floor.iter() {
        commands.spawn((
            Sprite {
                color: tile_color(tile),
                custom_size: Some(Vec2::splat(TILE_PX * 0.92)),
                ..default()
            },
            Transform::from_translation(tile_to_world(pos)),
            pos,
            TileSprite,
        ));
    }
}

/// Add the visual components to the already-spawned Player entity. The Player
/// is a single entity that holds both its logic state (`PlayerStats`,
/// `TilePos`) and its visuals (`Sprite`, `Transform`).
pub fn outfit_player(mut commands: Commands, player: Query<(Entity, &TilePos), With<Player>>) {
    let Ok((entity, &pos)) = player.single() else {
        return;
    };
    commands.entity(entity).insert((
        Sprite {
            color: Color::srgb(0.95, 0.85, 0.30),
            custom_size: Some(Vec2::splat(TILE_PX * 0.78)),
            ..default()
        },
        Transform::from_translation(tile_to_world(pos).with_z(1.0)),
    ));
}

fn tile_color(tile: &Tile) -> Color {
    match tile {
        Tile::Wall => Color::srgb(0.45, 0.40, 0.42),
        Tile::Door(DoorColor::Yellow) => Color::srgb(0.95, 0.80, 0.20),
        Tile::Door(DoorColor::Blue) => Color::srgb(0.30, 0.45, 0.95),
        Tile::Door(DoorColor::Red) => Color::srgb(0.85, 0.25, 0.25),
        Tile::StairsUp => Color::srgb(0.85, 0.85, 0.88),
        Tile::Item(kind) => match kind {
            ItemKind::YellowKey => Color::srgb(0.98, 0.75, 0.10),
            ItemKind::BlueKey => Color::srgb(0.15, 0.35, 0.90),
            ItemKind::RedKey => Color::srgb(0.85, 0.20, 0.30),
            ItemKind::Potion(_) => Color::srgb(0.85, 0.30, 0.35),
            ItemKind::AttackGem(_) => Color::srgb(0.95, 0.20, 0.20),
            ItemKind::DefenseGem(_) => Color::srgb(0.20, 0.40, 0.95),
            ItemKind::Unimplemented => Color::srgb(0.60, 0.60, 0.60),
        },
        Tile::Enemy(e) => match e.id {
            1 => Color::srgb(0.40, 0.85, 0.45),
            2 => Color::srgb(0.85, 0.40, 0.40),
            3 => Color::srgb(0.55, 0.45, 0.65),
            4 => Color::srgb(0.65, 0.55, 0.85),
            5 => Color::srgb(0.90, 0.90, 0.85),
            6 => Color::srgb(0.75, 0.75, 0.55),
            _ => Color::srgb(0.50, 0.50, 0.50),
        },
    }
}

/// Despawn tile sprites whose backing tile no longer exists in `Floor`
/// (consumed items, opened doors, slain enemies). Filtered to `With<TileSprite>`
/// so the Player entity is never matched.
pub fn despawn_cleared_tiles(
    mut commands: Commands,
    floor: Res<Floor>,
    tiles: Query<(Entity, &TilePos), With<TileSprite>>,
) {
    if !floor.is_changed() {
        return;
    }
    for (entity, pos) in &tiles {
        if floor.get(*pos).is_none() {
            commands.entity(entity).despawn();
        }
    }
}

/// Mirror the Player entity's authoritative `TilePos` onto its `Transform`.
/// Runs only on frames where `TilePos` actually changed.
pub fn sync_player_transform(
    mut player: Query<(&TilePos, &mut Transform), (With<Player>, Changed<TilePos>)>,
) {
    let Ok((&pos, mut tf)) = player.single_mut() else {
        return;
    };
    tf.translation = tile_to_world(pos).with_z(1.0);
}
