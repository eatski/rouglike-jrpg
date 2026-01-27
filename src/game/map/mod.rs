mod generation;
mod terrain;

pub use generation::{generate_map, MapData, MAP_HEIGHT, MAP_WIDTH};
pub use terrain::Terrain;
