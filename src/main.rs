use bevy::prelude::*;
use bevy::window::{Window, WindowResolution};

use ui::constants::WINDOW_SIZE;
use ui::events::{MovementBlockedEvent, PlayerMovedEvent};
use ui::resources::MovementState;
use ui::{
    camera_follow, init_exploration_system, init_minimap_system, init_tile_pool, player_movement,
    setup_camera, spawn_field_map, spawn_player, start_bounce, start_smooth_move,
    sync_boat_with_player, toggle_map_mode_system, toggle_minimap_visibility_system, update_bounce,
    update_exploration_system, update_minimap_texture_system, update_smooth_move,
    update_visible_tiles, MapModeState,
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
        .init_resource::<MapModeState>()
        .add_systems(
            Startup,
            (
                spawn_field_map,
                setup_camera,
                spawn_player,
                init_tile_pool,
                init_exploration_system,
                init_minimap_system,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                toggle_map_mode_system,
                toggle_minimap_visibility_system,
                player_movement,
                start_bounce,
                start_smooth_move,
                ApplyDeferred,
                update_smooth_move,
                update_bounce,
                update_visible_tiles,
                update_exploration_system,
                update_minimap_texture_system,
                sync_boat_with_player,
                camera_follow,
            )
                .chain(),
        )
        .run();
}
