mod boat;
mod events;
mod player;

pub use boat::{find_adjacent_boat, find_disembark_position, try_move_on_boat};
pub use events::Direction;
pub use player::{is_passable, try_move, MoveResult};
