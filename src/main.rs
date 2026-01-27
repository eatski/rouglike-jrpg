mod game;
mod ui;

use bevy::prelude::*;
use bevy::window::{Window, WindowResolution};

use crate::game::movement::{
    player_movement, MovementBlockedEvent, MovementState, PlayerMovedEvent,
};
use crate::ui::constants::WINDOW_SIZE;
use crate::ui::{
    camera_follow, setup_camera, spawn_field_map, spawn_player, start_bounce, update_bounce,
    update_player_position,
};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_message::<MovementBlockedEvent>()
        .add_message::<PlayerMovedEvent>()
        .init_resource::<MovementState>()
        .add_systems(Startup, (spawn_field_map, setup_camera, spawn_player).chain())
        .add_systems(
            Update,
            (
                player_movement,
                start_bounce,
                update_player_position,
                update_bounce,
                camera_follow,
            )
                .chain(),
        )
        .run();
}
