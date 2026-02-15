pub mod coordinates;
pub mod exploration;
pub mod map;
pub mod movement;

#[cfg(test)]
pub mod test_utils;

pub use coordinates::{
    is_diagonal_movement, orthogonal_neighbors, wrap_coordinate, wrap_position, Direction,
    ORTHOGONAL_DIRECTIONS,
};
