use bevy::prelude::*;

use movement_ui::{
    Boat, MovementBlockedEvent, MovementLocked, OnBoat, PendingMove, Player, PlayerMovedEvent,
    TileEnteredEvent, TilePosition,
};
use movement_ui::{ActiveMap, MovementState};

use crate::movement_helpers::{execute_boat_move, execute_walk_move, ExecuteMoveResult};

/// フィールドでのSmoothMove完了後の処理
///
/// PendingMoveがあれば2回目の移動を試行し、なければロック解除＋TileEnteredEvent発行。
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_field_move_completed(
    mut commands: Commands,
    mut move_state: ResMut<MovementState>,
    active_map: Res<ActiveMap>,
    mut query: Query<
        (
            Entity,
            &mut TilePosition,
            Option<&PendingMove>,
            Option<&OnBoat>,
        ),
        With<Player>,
    >,
    mut boat_query: Query<(Entity, &mut TilePosition), (With<Boat>, Without<Player>)>,
    mut moved_events: MessageWriter<PlayerMovedEvent>,
    mut blocked_events: MessageWriter<MovementBlockedEvent>,
    mut tile_entered_events: MessageWriter<TileEnteredEvent>,
) {
    if let Some(_entity) = move_state.move_just_completed.take() {
        let Ok((entity, mut tile_pos, pending_move, on_boat)) = query.single_mut() else {
            return;
        };

        if let Some(pending) = pending_move {
            let (dx, dy) = pending.direction;
            commands.entity(entity).remove::<PendingMove>();

            let move_success = if let Some(on_boat) = on_boat {
                matches!(
                    execute_boat_move(
                        &mut commands,
                        entity,
                        &mut tile_pos,
                        dx,
                        dy,
                        &active_map.grid,
                        on_boat,
                        &mut boat_query,
                        &mut moved_events,
                        &mut blocked_events,
                    ),
                    ExecuteMoveResult::Success
                )
            } else {
                matches!(
                    execute_walk_move(
                        &mut tile_pos,
                        entity,
                        dx,
                        dy,
                        &active_map.grid,
                        &mut moved_events,
                        &mut blocked_events,
                    ),
                    ExecuteMoveResult::Success
                )
            };

            if !move_success {
                // バウンスが始まるのでロック維持（バウンス終了時に解除）
            }
        } else {
            // PendingMoveがなければロック解除＋到着イベント発火
            commands.entity(entity).remove::<MovementLocked>();
            tile_entered_events.write(TileEnteredEvent { entity });
        }
    }
}
