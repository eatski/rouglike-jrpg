mod bounce;
mod components;
pub mod constants;
mod events;
mod resources;
mod smooth_move;

pub use bounce::{start_bounce, update_bounce, Bounce};
pub use components::{Boat, MapTile, MovementLocked, OnBoat, PendingMove, Player, TilePosition};
pub use constants::*;
pub use events::{MovementBlockedEvent, PlayerMovedEvent, TileEnteredEvent};
pub use resources::{ActiveMap, MovementState, WorldMapData};
pub use smooth_move::{
    ease_out_quad, start_smooth_move, update_smooth_move, SmoothMove,
    MOVE_DURATION,
};

use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MovementBlockedEvent>()
            .add_message::<PlayerMovedEvent>()
            .add_message::<TileEnteredEvent>()
            .init_resource::<MovementState>()
            .add_systems(PostUpdate, sync_tile_to_transform);
    }
}

/// TilePosition変更時にTransformを自動同期するシステム。
/// SmoothMoveアニメーション中はスキップされる（SmoothMove側がTransformを制御するため）。
#[allow(clippy::type_complexity)]
fn sync_tile_to_transform(
    active_map: Option<Res<ActiveMap>>,
    mut query: Query<
        (&TilePosition, &mut Transform),
        (Changed<TilePosition>, With<Player>, Without<SmoothMove>),
    >,
) {
    let Some(active_map) = active_map else {
        return;
    };
    for (tile_pos, mut transform) in &mut query {
        let (world_x, world_y) = active_map.to_world(tile_pos.x, tile_pos.y);
        transform.translation.x = world_x;
        transform.translation.y = world_y;
    }
}
