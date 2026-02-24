mod bounce;
mod camera;
pub mod coast_lookup;
mod components;
pub mod constants;
mod encounter;
mod events;
mod execute;
mod input;
pub mod map_mode;
pub mod rendering;
mod resources;
pub mod simple_tiles;
mod smooth_move;
pub mod tile_pool;

pub use bounce::{start_bounce, update_bounce, Bounce};
pub use camera::{camera_follow, setup_camera};
pub use components::{MovementLocked, PendingMove};
pub use constants::*;
pub use encounter::check_encounter_system;
pub use events::{MovementBlockedEvent, PlayerMovedEvent, TileEnteredEvent};
pub use execute::{execute_move, ExecuteMoveResult};
pub use input::{process_movement_input, MovementInput};
pub use map_mode::{reset_map_mode_system, toggle_map_mode_system, MapModeState, NORMAL_ZOOM};
pub use rendering::{load_tile_textures, spawn_boat_entities, BoatSpawnsResource, BossCaveWorldPos, TileTextures};
pub use resources::MovementState;
pub use simple_tiles::{update_simple_tiles, SimpleTile, SimpleTileMap};
pub use smooth_move::{
    ease_out_quad, start_smooth_move, update_smooth_move, SmoothMove,
    MOVE_DURATION,
};
pub use terrain::MoveResult;
pub use tile_pool::{create_tile_pool, init_tile_pool, update_visible_tiles, PooledTile, TilePool};

use bevy::prelude::*;
use field_core::{ActiveMap, Player, TilePosition};

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
