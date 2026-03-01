use bevy::prelude::*;

use terrain::TileAction;

use app_state::SceneState;
use field_core::{ActiveMap, Boat, OnBoat, Player, TilePosition};
use crate::{
    apply_simple_move, execute_move, MovementBlockedEvent,
    MovementLocked, MovementState, PendingMove, PlayerMovedEvent, TileEnteredEvent,
};

/// フィールドでのSmoothMove完了後の処理（2フェーズ）
///
/// Phase 1: SmoothMove完了時 — PendingMoveがあれば中間タイルのTileEnteredEventを発火し、
///          PendingMoveの実行を次フレームに遅延する（エンカウント/TileAction判定を保証）。
/// Phase 2: 次フレーム — PendingMoveを実行して2回目のSmoothMoveを開始。
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
    // Phase 2: 前フレームで中間タイルに到着済み → PendingMoveを実行
    if let Some(_entity) = move_state.pending_move_ready.take() {
        let Ok((entity, mut tile_pos, pending_move, on_boat)) = query.single_mut() else {
            return;
        };
        if let Some(pending) = pending_move {
            let (dx, dy) = pending.direction;
            commands.entity(entity).remove::<PendingMove>();
            execute_move(
                &mut commands, entity, &mut tile_pos, dx, dy,
                &active_map, on_boat, &mut boat_query,
                &mut moved_events, &mut blocked_events,
            );
        }
        return;
    }

    // Phase 1: SmoothMove完了
    if let Some(_entity) = move_state.move_just_completed.take() {
        let Ok((entity, _tile_pos, pending_move, _on_boat)) = query.single_mut() else {
            return;
        };

        if pending_move.is_some() {
            // 中間タイル到着: TileEnteredEvent発火 + PendingMoveは次フレームで実行
            tile_entered_events.write(TileEnteredEvent { entity });
            move_state.pending_move_ready = Some(entity);
        } else {
            // 最終タイル到着: ロック解除 + TileEnteredEvent発火
            commands.entity(entity).remove::<MovementLocked>();
            tile_entered_events.write(TileEnteredEvent { entity });
        }
    }
}

/// 船なしのSmoothMove完了後の処理（洞窟等で利用、2フェーズ）
///
/// Phase 1: SmoothMove完了時 — PendingMoveがあれば中間タイルの到着処理を行い、
///          PendingMoveの実行を次フレームに遅延する。
/// Phase 2: 次フレーム — PendingMoveを実行して2回目のSmoothMoveを開始。
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
    // Phase 2: 前フレームで中間タイルに到着済み → PendingMoveを実行
    if let Some(_entity) = move_state.pending_move_ready.take() {
        let Ok((entity, mut tile_pos, pending_move)) = query.single_mut() else {
            return;
        };
        if let Some(pending) = pending_move {
            let (dx, dy) = pending.direction;
            commands.entity(entity).remove::<PendingMove>();
            apply_simple_move(
                entity, &mut tile_pos, dx, dy,
                &active_map, &mut moved_events, &mut blocked_events,
            );
        }
        return;
    }

    // Phase 1: SmoothMove完了
    if move_state.move_just_completed.take().is_none() {
        return;
    }
    let Ok((entity, tile_pos, pending_move)) = query.single_mut() else {
        return;
    };

    if pending_move.is_some() {
        // 中間タイル到着: TileAction判定 + TileEnteredEvent発火
        match active_map.tile_action_at(tile_pos.x, tile_pos.y) {
            TileAction::EnterTown => next_state.set(SceneState::Town),
            TileAction::EnterCave => next_state.set(SceneState::Cave),
            TileAction::EnterBossCave => next_state.set(SceneState::BossCave),
            TileAction::EnterHokora => next_state.set(SceneState::Hokora),
            TileAction::ExitCave => next_state.set(SceneState::Exploring),
            TileAction::None => {
                tile_entered_events.write(TileEnteredEvent { entity });
                move_state.pending_move_ready = Some(entity);
            }
        }
    } else {
        // 最終タイル到着: ロック解除 + TileAction判定
        commands.entity(entity).remove::<MovementLocked>();
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
