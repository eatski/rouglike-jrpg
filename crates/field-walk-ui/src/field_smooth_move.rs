use bevy::prelude::*;

use terrain::TileAction;

use app_state::SceneState;
use field_core::{ActiveMap, Boat, OnBoat, Player, TilePosition};
use crate::{
    execute_move, process_simple_move_completed, ExecuteMoveResult, MovementBlockedEvent,
    MovementLocked, MovementState, PendingMove, PlayerMovedEvent, TileEnteredEvent,
};

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

            let _move_success = matches!(
                execute_move(
                    &mut commands,
                    entity,
                    &mut tile_pos,
                    dx,
                    dy,
                    &active_map,
                    on_boat,
                    &mut boat_query,
                    &mut moved_events,
                    &mut blocked_events,
                ),
                ExecuteMoveResult::Success
            );
            // ブロック時はバウンスが始まるのでロック維持（バウンス終了時に解除）
        } else {
            // PendingMoveがなければロック解除＋到着イベント発火
            commands.entity(entity).remove::<MovementLocked>();
            tile_entered_events.write(TileEnteredEvent { entity });
        }
    }
}

/// 船なしのSmoothMove完了後の処理（洞窟等で利用）。
///
/// PendingMoveがあれば2回目の移動を試行し、なければロック解除。
/// 到着時にタイルアクションを直接判定し、遷移またはTileEnteredEventを発火する。
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_simple_move_completed(
    mut commands: Commands,
    mut move_state: ResMut<MovementState>,
    active_map: Res<ActiveMap>,
    mut query: Query<
        (
            Entity,
            &mut TilePosition,
            Option<&PendingMove>,
        ),
        With<Player>,
    >,
    mut moved_events: MessageWriter<PlayerMovedEvent>,
    mut blocked_events: MessageWriter<MovementBlockedEvent>,
    mut tile_entered_events: MessageWriter<TileEnteredEvent>,
    mut next_state: ResMut<NextState<SceneState>>,
) {
    if move_state.move_just_completed.take().is_none() {
        return;
    }
    let Ok((entity, mut tile_pos, pending_move)) = query.single_mut() else {
        return;
    };

    let arrived = process_simple_move_completed(
        &mut commands, entity, &mut tile_pos, pending_move,
        &active_map, &mut moved_events, &mut blocked_events,
    );

    if arrived {
        match active_map.tile_action_at(tile_pos.x, tile_pos.y) {
            TileAction::EnterTown => next_state.set(SceneState::Town),
            TileAction::EnterCave => next_state.set(SceneState::Cave),
            TileAction::EnterBossCave => next_state.set(SceneState::BossCave),
            TileAction::EnterHokora => next_state.set(SceneState::Hokora),
            TileAction::ExitCave => next_state.set(SceneState::Exploring),
            TileAction::None => {
                tile_entered_events.write(TileEnteredEvent { entity });
            }
        }
    }
}
