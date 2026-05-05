use bevy::prelude::*;

use crate::game::{DoorColor, Floor, ItemKind, Player, Tile, TilePos, GRID_SIZE};

use super::{animation::SpriteAnimation, sprite_catalog::SpriteCatalog, TILE_PX};

/// Marker component for the per-tile sprites spawned at the start of a floor.
/// Lets `despawn_cleared_tiles` filter positively instead of relying on
/// negative filters that could accidentally match the Player.
#[derive(Component)]
pub struct TileSprite;

/// Vertical offset (world units) applied to the whole map so the HUD has room
/// at the top of the window without overlapping the top row of tiles. Must be
/// small enough that the bottom row stays inside the camera's vertical view —
/// with `TILE_PX = 64` on an 800 px window, the camera shows world `y ∈ [-400, +400]`,
/// the map is `11 × 64 = 704` px tall (so half-height = 352), leaving 48 px
/// of slack on each side of the centered map. We spend up to 32 px of the top
/// slack on the HUD and keep 16 px free at the bottom for breathing room.
const MAP_Y_OFFSET: f32 = -16.0;

/// Tile coord → world translation.
pub fn tile_to_world(pos: TilePos) -> Vec3 {
    let half = (GRID_SIZE as f32 - 1.0) * 0.5;
    let x = (pos.col as f32 - half) * TILE_PX;
    let y = (pos.row as f32 - half) * TILE_PX + MAP_Y_OFFSET;
    Vec3::new(x, y, 0.0)
}

pub fn spawn_camera_and_tiles(
    mut commands: Commands,
    floor: Res<Floor>,
    catalog: Res<SpriteCatalog>,
) {
    commands.spawn(Camera2d);

    let span = GRID_SIZE as f32 * TILE_PX;
    commands.spawn((
        Sprite {
            color: Color::srgb(0.16, 0.13, 0.10),
            custom_size: Some(Vec2::splat(span)),
            ..default()
        },
        Transform::from_xyz(0.0, MAP_Y_OFFSET, -1.0),
    ));

    for (pos, tile) in floor.iter() {
        let key = tile_atlas_key(tile);
        let sprite = key
            .and_then(|k| catalog.make_sprite(k, TILE_PX))
            .unwrap_or_else(|| fallback_color_sprite(tile));
        let mut entity = commands.spawn((
            sprite,
            Transform::from_translation(tile_to_world(pos)),
            pos,
            TileSprite,
        ));
        if let Some((frames, fps)) = key.and_then(|k| catalog.animation(k)) {
            if frames.len() > 1 && fps > 0.0 {
                entity.insert(SpriteAnimation::new(frames, fps));
            }
        }
    }
}

/// Add the visual components to the already-spawned Player entity.
pub fn outfit_player(
    mut commands: Commands,
    player: Query<(Entity, &TilePos), With<Player>>,
    catalog: Res<SpriteCatalog>,
) {
    let Ok((entity, &pos)) = player.single() else {
        return;
    };
    let sprite = catalog
        .make_sprite("player", TILE_PX)
        .unwrap_or_else(|| Sprite {
            color: Color::srgb(0.95, 0.85, 0.30),
            custom_size: Some(Vec2::splat(TILE_PX * 0.78)),
            ..default()
        });
    commands.entity(entity).insert((
        sprite,
        Transform::from_translation(tile_to_world(pos).with_z(1.0)),
    ));
}

/// Map a `Tile` to the catalog key used to look up its sprite. Returns `None`
/// for tiles we haven't yet got artwork for, so the caller can fall back to a
/// plain colored sprite without panicking.
fn tile_atlas_key(tile: &Tile) -> Option<&'static str> {
    Some(match tile {
        Tile::Wall => "wall",
        Tile::Door(DoorColor::Yellow) => "door_yellow",
        Tile::Door(DoorColor::Blue) => return None,
        Tile::Door(DoorColor::Red) => return None,
        Tile::StairsUp => "stairs_up",
        Tile::Item(kind) => match kind {
            ItemKind::YellowKey => "yellow_key",
            ItemKind::BlueKey => return None,
            ItemKind::RedKey => return None,
            ItemKind::Potion(amount) => {
                if *amount <= 200 {
                    "small_potion"
                } else {
                    "large_potion"
                }
            }
            ItemKind::AttackGem(_) => "red_gem",
            ItemKind::DefenseGem(_) => "blue_gem",
            ItemKind::Unimplemented => "pharaoh_scepter",
        },
        Tile::Enemy(e) => match e.id {
            1 => "green_slime",
            2 => "red_slime",
            3 => "small_bat",
            4 => "apprentice_mage",
            5 => "skeleton",
            6 => "skeleton_soldier",
            _ => return None,
        },
    })
}

fn fallback_color_sprite(tile: &Tile) -> Sprite {
    let color = match tile {
        Tile::Wall => Color::srgb(0.45, 0.40, 0.42),
        Tile::Door(DoorColor::Yellow) => Color::srgb(0.95, 0.80, 0.20),
        Tile::Door(DoorColor::Blue) => Color::srgb(0.30, 0.45, 0.95),
        Tile::Door(DoorColor::Red) => Color::srgb(0.85, 0.25, 0.25),
        Tile::StairsUp => Color::srgb(0.85, 0.85, 0.88),
        Tile::Item(_) => Color::srgb(0.80, 0.55, 0.20),
        Tile::Enemy(_) => Color::srgb(0.50, 0.55, 0.45),
    };
    Sprite {
        color,
        custom_size: Some(Vec2::splat(TILE_PX * 0.92)),
        ..default()
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

