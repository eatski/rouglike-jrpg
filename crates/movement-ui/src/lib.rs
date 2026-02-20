mod bounce;
mod components;
pub mod constants;
mod events;
mod resources;
mod smooth_move;

pub use bounce::{start_bounce, update_bounce, Bounce};
pub use components::{Boat, MapTile, MovementLocked, OnBoat, PendingMove, Player, TilePosition};
pub use constants::*;
pub use events::{MovementBlockedEvent, PlayerArrivedEvent, PlayerMovedEvent, TileEnteredEvent};
pub use resources::{ActiveMap, MovementState, WorldMapData};
pub use smooth_move::{ease_out_quad, SmoothMove};

use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MovementBlockedEvent>()
            .add_message::<PlayerMovedEvent>()
            .add_message::<PlayerArrivedEvent>()
            .add_message::<TileEnteredEvent>()
            .init_resource::<MovementState>();
    }
}
