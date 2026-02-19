use bevy::prelude::*;

use app_state::OpenedChests;
use cave_ui::{setup_cave_scene, update_cave_tiles};
use movement_ui::MovementState;
use screenshot_common::screenshot_app;
use world_ui::{
    camera_follow, init_tile_pool, setup_camera, spawn_field_map, spawn_player, MapModeState,
};

fn main() {
    let mut app = screenshot_app("cave");
    app.init_resource::<MovementState>()
        .init_resource::<MapModeState>()
        .init_resource::<OpenedChests>()
        .add_systems(
            Startup,
            (
                spawn_field_map,
                setup_camera,
                spawn_player,
                init_tile_pool,
                ApplyDeferred,
                setup_cave_scene,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (update_cave_tiles, camera_follow).chain(),
        )
        .run();
}
