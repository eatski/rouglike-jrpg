pub mod coordinates;
pub mod movement;
pub mod terrain;

pub mod test_utils;

pub use coordinates::{
    is_diagonal_movement, orthogonal_neighbors, wrap_coordinate, wrap_position, Direction,
    ORTHOGONAL_DIRECTIONS,
};
pub use movement::{try_grid_move, MoveResult};
pub use terrain::{Structure, Terrain, TileAction, MAP_HEIGHT, MAP_WIDTH};
