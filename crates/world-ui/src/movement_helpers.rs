//! 移動処理の共通ヘルパー関数
//!
//! `player_input` と `smooth_move` で共通する移動実行ロジックを提供する。

use bevy::prelude::*;

use world::movement::{try_boat_move_or_disembark, try_move, BoatMoveResult, MoveResult};
use terrain::Terrain;

use components_ui::{Boat, OnBoat, Player, TilePosition};
use events_ui::{MovementBlockedEvent, PlayerMovedEvent};

/// 移動実行の結果
pub enum ExecuteMoveResult {
    /// 移動成功
    Success,
    /// 移動失敗（ブロックされた）
    Blocked,
}

/// 船での移動を実行する共通ヘルパー
///
/// `try_boat_move_or_disembark` の結果を処理し、タイル座標の更新・イベント発行を行う。
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn execute_boat_move(
    commands: &mut Commands,
    entity: Entity,
    tile_pos: &mut TilePosition,
    dx: i32,
    dy: i32,
    grid: &[Vec<Terrain>],
    on_boat: &OnBoat,
    boat_query: &mut Query<(Entity, &mut TilePosition), (With<Boat>, Without<Player>)>,
    moved_events: &mut MessageWriter<PlayerMovedEvent>,
    blocked_events: &mut MessageWriter<MovementBlockedEvent>,
) -> ExecuteMoveResult {
    match try_boat_move_or_disembark(tile_pos.x, tile_pos.y, dx, dy, grid) {
        BoatMoveResult::MovedOnSea { new_x, new_y } => {
            if let Ok((_, mut boat_tile_pos)) = boat_query.get_mut(on_boat.boat_entity) {
                boat_tile_pos.x = new_x;
                boat_tile_pos.y = new_y;
            }
            tile_pos.x = new_x;
            tile_pos.y = new_y;
            moved_events.write(PlayerMovedEvent {
                entity,
                direction: (dx, dy),
            });
            ExecuteMoveResult::Success
        }
        BoatMoveResult::Disembarked { new_x, new_y } => {
            commands.entity(entity).remove::<OnBoat>();
            tile_pos.x = new_x;
            tile_pos.y = new_y;
            moved_events.write(PlayerMovedEvent {
                entity,
                direction: (dx, dy),
            });
            ExecuteMoveResult::Success
        }
        BoatMoveResult::Blocked => {
            blocked_events.write(MovementBlockedEvent {
                entity,
                direction: (dx, dy),
            });
            ExecuteMoveResult::Blocked
        }
    }
}

/// 徒歩での移動を実行する共通ヘルパー
///
/// `try_move` の結果を処理し、タイル座標の更新・イベント発行を行う。
pub fn execute_walk_move(
    tile_pos: &mut TilePosition,
    entity: Entity,
    dx: i32,
    dy: i32,
    grid: &[Vec<Terrain>],
    moved_events: &mut MessageWriter<PlayerMovedEvent>,
    blocked_events: &mut MessageWriter<MovementBlockedEvent>,
) -> ExecuteMoveResult {
    match try_move(tile_pos.x, tile_pos.y, dx, dy, grid) {
        MoveResult::Moved { new_x, new_y } => {
            tile_pos.x = new_x;
            tile_pos.y = new_y;
            moved_events.write(PlayerMovedEvent {
                entity,
                direction: (dx, dy),
            });
            ExecuteMoveResult::Success
        }
        MoveResult::Blocked => {
            blocked_events.write(MovementBlockedEvent {
                entity,
                direction: (dx, dy),
            });
            ExecuteMoveResult::Blocked
        }
    }
}
