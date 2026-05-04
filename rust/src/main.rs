use bevy::prelude::*;

use tower_of_the_sorcerer::{game::GamePlugin, input::InputPlugin, render::RenderPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tower of the Sorcerer — Floor 1".into(),
                resolution: (640u32, 600u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((GamePlugin, InputPlugin, RenderPlugin))
        .run();
}
