use bevy::prelude::*;

use app_state::PartyState;
use screenshot_common::screenshot_app;
use world_ui::{
    camera_follow, init_exploration_system, init_minimap_system, init_tile_pool, setup_camera,
    setup_hud, spawn_field_map, spawn_player, update_visible_tiles, MapModeState,
};

fn main() {
    let mut app = screenshot_app("field");
    app.init_resource::<MapModeState>()
        .init_resource::<PartyState>()
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
        .add_systems(Startup, setup_hud)
        .add_systems(
            Update,
            (update_visible_tiles, camera_follow).chain(),
        )
        .run();
}
