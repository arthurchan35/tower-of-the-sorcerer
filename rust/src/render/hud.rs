use bevy::prelude::*;

use crate::game::{Player, PlayerStats, StatusMessage};

#[derive(Component)]
pub struct StatsLabel;

#[derive(Component)]
pub struct StatusLabel;

pub fn spawn_hud(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            right: Val::Px(8.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
                StatsLabel,
            ));
            parent.spawn((
                Text::new(""),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.75, 0.75, 0.80)),
                StatusLabel,
            ));
        });
}

pub fn sync_stats(
    player: Query<&PlayerStats, (With<Player>, Changed<PlayerStats>)>,
    mut stats_label: Query<&mut Text, With<StatsLabel>>,
) {
    let Ok(p) = player.single() else { return };
    let Ok(mut text) = stats_label.single_mut() else { return };
    **text = format!(
        "HP {}   ATK {}   DEF {}   GOLD {}   KEYS Y{} B{} R{}",
        p.health, p.attack, p.defense, p.gold, p.yellow_keys, p.blue_keys, p.red_keys,
    );
}

pub fn sync_status(
    status: Res<StatusMessage>,
    mut q: Query<&mut Text, With<StatusLabel>>,
) {
    if !status.is_changed() {
        return;
    }
    let Ok(mut text) = q.single_mut() else { return };
    **text = status.0.clone();
}

pub fn announce_floor_cleared(mut status: ResMut<StatusMessage>) {
    status.0 = "Floor cleared! Press Esc to quit.".into();
}

pub fn announce_game_over(mut status: ResMut<StatusMessage>) {
    status.0 = "You died. Press Esc to quit.".into();
}
