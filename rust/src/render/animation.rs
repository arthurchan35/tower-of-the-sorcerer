use bevy::prelude::*;

use crate::game::{Player, TilePos};

use super::{sprite_catalog::SpriteCatalog, tiles::tile_to_world};

/// Cycles an entity's `Sprite::texture_atlas.index` through a fixed list of
/// frames at a fixed rate. Removed by the caller when an animation should stop
/// (e.g. when the player finishes walking).
#[derive(Component)]
pub struct SpriteAnimation {
    pub frame_indices: Vec<usize>,
    pub seconds_per_frame: f32,
    elapsed: f32,
    current: usize,
}

impl SpriteAnimation {
    pub fn new(frame_indices: Vec<usize>, fps: f32) -> Self {
        let seconds_per_frame = if fps > 0.0 { 1.0 / fps } else { f32::INFINITY };
        Self {
            frame_indices,
            seconds_per_frame,
            elapsed: 0.0,
            current: 0,
        }
    }
}

/// Smoothly interpolates an entity's `Transform.translation` from `start` to
/// `target` over `duration` seconds. Used for the player's tile-to-tile slide.
#[derive(Component)]
pub struct TileTransition {
    pub start: Vec3,
    pub target: Vec3,
    pub elapsed: f32,
    pub duration: f32,
}

const PLAYER_WALK_DURATION: f32 = 0.18;

/// Tick `SpriteAnimation` timers and update each sprite's atlas index. Does
/// not touch `TextureAtlas::layout` — only the frame index changes.
pub fn animate_sprites(
    time: Res<Time>,
    mut q: Query<(&mut SpriteAnimation, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (mut anim, mut sprite) in &mut q {
        if anim.frame_indices.len() <= 1 || !anim.seconds_per_frame.is_finite() {
            continue;
        }
        anim.elapsed += dt;
        while anim.elapsed >= anim.seconds_per_frame {
            anim.elapsed -= anim.seconds_per_frame;
            anim.current = (anim.current + 1) % anim.frame_indices.len();
        }
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = anim.frame_indices[anim.current];
        }
    }
}

/// On `Changed<TilePos>` for the Player, attach (or restart) a `TileTransition`
/// that slides the visual `Transform` from its current position to the new
/// tile's world position, and attach a walking `SpriteAnimation` so the legs
/// alternate during the slide.
pub fn start_player_walk(
    mut commands: Commands,
    catalog: Res<SpriteCatalog>,
    player: Query<
        (Entity, &TilePos, &Transform, Option<&TileTransition>),
        (With<Player>, Changed<TilePos>),
    >,
) {
    let Ok((entity, &pos, tf, existing)) = player.single() else {
        return;
    };
    let target = tile_to_world(pos).with_z(1.0);
    if (target - tf.translation).length_squared() < 0.01 {
        return; // first-frame add: visual is already at the right tile
    }
    commands.entity(entity).insert(TileTransition {
        start: tf.translation,
        target,
        elapsed: 0.0,
        duration: PLAYER_WALK_DURATION,
    });
    if existing.is_none() {
        if let Some((frames, fps)) = catalog.animation("player_walk") {
            commands.entity(entity).insert(SpriteAnimation::new(frames, fps));
        }
    }
}

/// Advance the player's `TileTransition`. When the slide completes, remove
/// the transition + walk animation and reset the sprite to its idle frame.
pub fn tick_player_walk(
    time: Res<Time>,
    mut commands: Commands,
    catalog: Res<SpriteCatalog>,
    mut q: Query<(Entity, &mut Transform, &mut Sprite, &mut TileTransition), With<Player>>,
) {
    let Ok((entity, mut tf, mut sprite, mut t)) = q.single_mut() else {
        return;
    };
    t.elapsed += time.delta_secs();
    let progress = (t.elapsed / t.duration).clamp(0.0, 1.0);
    tf.translation = t.start.lerp(t.target, progress);
    if progress >= 1.0 {
        tf.translation = t.target;
        commands
            .entity(entity)
            .remove::<TileTransition>()
            .remove::<SpriteAnimation>();
        if let Some((frames, _)) = catalog.animation("player") {
            if let (Some(idle_index), Some(atlas)) = (frames.first(), sprite.texture_atlas.as_mut())
            {
                atlas.index = *idle_index;
            }
        }
    }
}
