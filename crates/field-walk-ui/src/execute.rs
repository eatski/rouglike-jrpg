use bevy::prelude::*;

use field_core::{ActiveMap, Boat, OnBoat, Player, TilePosition};
use field_walk::{resolve_field_move, FieldMoveResult};

use crate::{MovementBlockedEvent, PlayerMovedEvent};

/// 移動実行の結果
pub enum ExecuteMoveResult {
    Success,
    Blocked,
}

/// 船なしの単純な移動実行（洞窟・徒歩共通）
pub fn apply_simple_move(
    entity: Entity,
    tile_pos: &mut TilePosition,
    dx: i32,
    dy: i32,
    active_map: &ActiveMap,
    moved_events: &mut MessageWriter<PlayerMovedEvent>,
    blocked_events: &mut MessageWriter<MovementBlockedEvent>,
) -> ExecuteMoveResult {
    let result = resolve_field_move(
        &active_map.grid,
        &active_map.structures,
        active_map.width,
        active_map.height,
        active_map.wraps,
        tile_pos.x,
        tile_pos.y,
        dx,
        dy,
        false,
    );
    match result {
        FieldMoveResult::Walked { new_x, new_y } => {
            tile_pos.x = new_x;
            tile_pos.y = new_y;
            moved_events.write(PlayerMovedEvent {
                entity,
                direction: (dx, dy),
            });
            ExecuteMoveResult::Success
        }
        FieldMoveResult::Blocked => {
            blocked_events.write(MovementBlockedEvent {
                entity,
                direction: (dx, dy),
            });
            ExecuteMoveResult::Blocked
        }
        // on_boat=false では Sailed/Disembarked は発生しない
        _ => unreachable!(),
    }
}

/// 対角線移動の2回目を即時実行（成功する場合のみ）。
/// 成功: true（PlayerMovedEvent発火済み、start_smooth_moveが統合）、
/// 失敗: false（呼び出し元がPendingMoveを追加すべき）。
pub fn try_apply_second_move(
    entity: Entity,
    tile_pos: &mut TilePosition,
    dx: i32,
    dy: i32,
    active_map: &ActiveMap,
    moved_events: &mut MessageWriter<PlayerMovedEvent>,
    blocked_events: &mut MessageWriter<MovementBlockedEvent>,
) -> bool {
    let check = resolve_field_move(
        &active_map.grid,
        &active_map.structures,
        active_map.width,
        active_map.height,
        active_map.wraps,
        tile_pos.x,
        tile_pos.y,
        dx,
        dy,
        false,
    );
    if matches!(check, FieldMoveResult::Walked { .. }) {
        apply_simple_move(entity, tile_pos, dx, dy, active_map, moved_events, blocked_events);
        true
    } else {
        false
    }
}

/// 徒歩・船を統合した移動実行関数
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn execute_move(
    commands: &mut Commands,
    entity: Entity,
    tile_pos: &mut TilePosition,
    dx: i32,
    dy: i32,
    active_map: &ActiveMap,
    on_boat: Option<&OnBoat>,
    boat_query: &mut Query<(Entity, &mut TilePosition), (With<Boat>, Without<Player>)>,
    moved_events: &mut MessageWriter<PlayerMovedEvent>,
    blocked_events: &mut MessageWriter<MovementBlockedEvent>,
) -> ExecuteMoveResult {
    let result = resolve_field_move(
        &active_map.grid,
        &active_map.structures,
        active_map.width,
        active_map.height,
        active_map.wraps,
        tile_pos.x,
        tile_pos.y,
        dx,
        dy,
        on_boat.is_some(),
    );
    match result {
        FieldMoveResult::Walked { new_x, new_y }
        | FieldMoveResult::Disembarked { new_x, new_y } => {
            if matches!(result, FieldMoveResult::Disembarked { .. }) {
                commands.entity(entity).remove::<OnBoat>();
            }
            tile_pos.x = new_x;
            tile_pos.y = new_y;
            moved_events.write(PlayerMovedEvent {
                entity,
                direction: (dx, dy),
            });
            ExecuteMoveResult::Success
        }
        FieldMoveResult::Sailed { new_x, new_y } => {
            if let Some(on_boat) = on_boat
                && let Ok((_, mut boat_pos)) = boat_query.get_mut(on_boat.boat_entity)
            {
                boat_pos.x = new_x;
                boat_pos.y = new_y;
            }
            tile_pos.x = new_x;
            tile_pos.y = new_y;
            moved_events.write(PlayerMovedEvent {
                entity,
                direction: (dx, dy),
            });
            ExecuteMoveResult::Success
        }
        FieldMoveResult::Blocked => {
            blocked_events.write(MovementBlockedEvent {
                entity,
                direction: (dx, dy),
            });
            ExecuteMoveResult::Blocked
        }
    }
}
