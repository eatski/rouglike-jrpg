use bevy::prelude::*;
use bevy::window::{Window, WindowResolution};

use ui::constants::WINDOW_SIZE;
use ui::events::{MovementBlockedEvent, PlayerMovedEvent};
use ui::resources::MovementState;
use ui::{
    camera_follow, player_movement, setup_camera, spawn_field_map, spawn_player, start_bounce,
    start_smooth_move, sync_boat_with_player, tile_culling, update_bounce, update_smooth_move,
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
                start_smooth_move,
                ApplyDeferred,
                update_smooth_move,
                update_bounce,
                sync_boat_with_player,
                camera_follow,
                tile_culling,
            )
                .chain(),
        )
        .run();
}
