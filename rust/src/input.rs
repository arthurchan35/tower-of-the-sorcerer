use bevy::prelude::*;

use crate::game::Direction;

#[derive(Clone, Copy, Debug, Message)]
pub struct MoveIntent(pub Direction);

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MoveIntent>()
            .add_systems(Update, keyboard_to_move_intent);
    }
}

pub fn keyboard_to_move_intent(
    keys: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<MoveIntent>,
) {
    let dir = if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        Direction::Up
    } else if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        Direction::Down
    } else if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
        Direction::Left
    } else if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
        Direction::Right
    } else {
        return;
    };
    writer.write(MoveIntent(dir));
}
