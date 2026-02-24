use bevy::prelude::*;

use terrain::{MoveResult, Terrain};

use field_core::{ActiveMap, Boat, OnBoat, Player, TilePosition};

use crate::{MovementBlockedEvent, PlayerMovedEvent};

/// 移動実行の結果
pub enum ExecuteMoveResult {
    Success,
    Blocked,
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
    if let Some(on_boat) = on_boat {
        // 船モード: まず海上移動を試行
        match active_map.try_move_with(tile_pos.x, tile_pos.y, dx, dy, Terrain::is_navigable) {
            MoveResult::Moved { new_x, new_y } => {
                if let Ok((_, mut boat_pos)) = boat_query.get_mut(on_boat.boat_entity) {
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
            MoveResult::Blocked => {
                // 下船を試行（陸地への移動）
                match active_map.try_move(tile_pos.x, tile_pos.y, dx, dy) {
                    MoveResult::Moved { new_x, new_y } => {
                        commands.entity(entity).remove::<OnBoat>();
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
        }
    } else {
        // 徒歩移動
        match active_map.try_move(tile_pos.x, tile_pos.y, dx, dy) {
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
}
