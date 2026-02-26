pub mod coordinates;
pub mod movement;
pub mod structure_clearance;
pub mod terrain;

pub mod test_utils;

pub use coordinates::{
    bounded_offset, bounded_orthogonal_neighbors, is_diagonal_movement, orthogonal_neighbors,
    wrap_coordinate, wrap_position, Direction, ORTHOGONAL_DIRECTIONS,
};
pub use movement::{try_grid_move, MoveResult};
pub use structure_clearance::clear_around_structures;
pub use terrain::{Structure, Terrain, TileAction, MAP_HEIGHT, MAP_WIDTH};
