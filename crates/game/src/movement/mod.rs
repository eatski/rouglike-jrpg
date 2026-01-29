mod boat;
mod events;
mod player;

pub use boat::{
    find_adjacent_boat, find_disembark_position, try_boat_move_or_disembark, try_move_on_boat,
    BoatMoveResult,
};
pub use events::Direction;
pub use player::{try_move, MoveResult};
