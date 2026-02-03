mod generation;
mod islands;
mod terrain;

pub use generation::{generate_connected_map, generate_map, MapData, MAP_HEIGHT, MAP_WIDTH};
pub use islands::{calculate_boat_spawns, detect_islands, validate_connectivity, BoatSpawn};
pub use terrain::Terrain;
