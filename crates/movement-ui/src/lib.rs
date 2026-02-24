mod bounce;
mod components;
pub mod constants;
mod events;
mod execute;
mod input;
mod resources;
mod smooth_move;

pub use bounce::{start_bounce, update_bounce, Bounce};
pub use components::{Boat, MapTile, MovementLocked, OnBoat, PendingMove, Player, TilePosition};
pub use constants::*;
pub use events::{MovementBlockedEvent, PlayerMovedEvent, TileEnteredEvent};
pub use execute::{execute_move, ExecuteMoveResult};
pub use input::{process_movement_input, MovementInput};
pub use resources::{ActiveMap, MovementState, WorldMapData};
pub use terrain::MoveResult;
pub use smooth_move::{
    ease_out_quad, start_smooth_move, update_smooth_move, SmoothMove,
    MOVE_DURATION,
};

use bevy::prelude::*;

/// フィールド離脱時にプレイヤーの移動関連コンポーネントと状態をクリーンアップする。
/// OnExit(InField) で呼ばれ、戦闘開始・町入場・祠入場時のクリーンアップを一元化する。
pub fn cleanup_player_movement(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    mut move_state: ResMut<MovementState>,
) {
    if let Ok(entity) = player_query.single() {
        commands
            .entity(entity)
            .remove::<MovementLocked>()
            .remove::<SmoothMove>()
            .remove::<PendingMove>()
            .remove::<Bounce>();
    }
    *move_state = MovementState::default();
}

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
