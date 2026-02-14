pub mod coordinates;
pub mod terrain;

pub mod test_utils;

pub use coordinates::{
    is_diagonal_movement, wrap_coordinate, Direction, ORTHOGONAL_DIRECTIONS,
};
pub use terrain::{Terrain, TileAction, MAP_HEIGHT, MAP_WIDTH};
