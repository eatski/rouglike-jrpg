mod generation;
mod islands;
mod terrain;

pub use generation::{generate_map, MapData, MAP_HEIGHT, MAP_WIDTH};
pub use islands::{calculate_boat_spawns, BoatSpawn};
pub use terrain::Terrain;
