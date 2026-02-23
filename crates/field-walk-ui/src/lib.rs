mod camera;
pub mod coast_lookup;
mod encounter;
pub mod map_mode;
pub mod rendering;
pub mod tile_pool;

pub use camera::{camera_follow, setup_camera};
pub use encounter::check_encounter_system;
pub use map_mode::{reset_map_mode_system, toggle_map_mode_system, MapModeState, NORMAL_ZOOM};
pub use rendering::{load_tile_textures, spawn_boat_entities, BoatSpawnsResource, BossCaveWorldPos, TileTextures};
pub use tile_pool::{create_tile_pool, init_tile_pool, update_visible_tiles, PooledTile, TilePool};
