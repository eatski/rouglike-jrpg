mod generation;
mod islands;

pub use terrain::{Terrain, TileAction, MAP_HEIGHT, MAP_WIDTH};
pub use generation::{generate_connected_map, generate_map, MapData};
pub use islands::{
    assign_candidates_to_towns, calculate_boat_spawns, calculate_cave_spawns,
    calculate_town_spawns, detect_islands, place_extra_towns, validate_connectivity, BoatSpawn,
    CandidatePlacement, CaveSpawn, TownSpawn,
};
