mod camera;
mod coast_lookup;
mod encounter;
mod hud;
pub mod map_mode;
mod minimap;
mod movement_helpers;
mod player_input;
pub mod rendering;
pub mod resources;
pub mod smooth_move;
pub mod tile_action;
pub mod tile_pool;

pub use camera::{camera_follow, setup_camera};
pub use encounter::check_encounter_system;
pub use hud::{cleanup_hud, setup_hud, toggle_hud_visibility, update_hud};
pub use map_mode::{
    init_exploration_system, toggle_map_mode_system, update_exploration_system, ExplorationData,
    MapModeState,
};
pub use minimap::{init_minimap_system, toggle_minimap_visibility_system, update_minimap_texture_system};
pub use player_input::{player_movement, sync_boat_with_player};
pub use rendering::{spawn_boat_entities, spawn_field_map, spawn_field_map_with_rng, spawn_player, BoatSpawnsResource, TileTextures};
pub use smooth_move::{start_smooth_move, update_smooth_move};
pub use tile_pool::{create_tile_pool, init_tile_pool, update_visible_tiles, PooledTile, TilePool};
pub use tile_action::check_tile_action_system;
pub use resources::SpawnPosition;
