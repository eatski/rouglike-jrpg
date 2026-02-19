use bevy::prelude::*;

use world::wrap_position;

use movement_ui::{
    Boat, MovementBlockedEvent, MovementLocked, OnBoat, PendingMove, Player, PlayerMovedEvent,
    TilePosition,
};
use app_state::FieldMenuOpen;
use movement_ui::{ActiveMap, MovementState};

use crate::map_mode::MapModeState;
use crate::movement_helpers::{execute_boat_move, execute_walk_move, ExecuteMoveResult};

/// プレイヤーの移動入力を処理するシステム
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn player_movement(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    active_map: Res<ActiveMap>,
    map_mode_state: Res<MapModeState>,
    field_menu_open: Res<FieldMenuOpen>,
    mut move_state: ResMut<MovementState>,
    mut query: Query<
        (
            Entity,
            &mut TilePosition,
            Option<&MovementLocked>,
            Option<&OnBoat>,
        ),
        With<Player>,
    >,
    mut boat_query: Query<(Entity, &mut TilePosition), (With<Boat>, Without<Player>)>,
    mut blocked_events: MessageWriter<MovementBlockedEvent>,
    mut moved_events: MessageWriter<PlayerMovedEvent>,
) {
    let Ok((entity, mut tile_pos, locked, on_boat)) = query.single_mut() else {
        return;
    };

    // 移動ロック中は入力を無視
    if locked.is_some() {
        return;
    }

    // マップモード中は移動を無効化
    if map_mode_state.enabled {
        return;
    }

    // フィールド呪文メニュー中は移動を無効化
    if field_menu_open.0 {
        return;
    }

    let mut dx: i32 = 0;
    let mut dy: i32 = 0;

    let x_pressed = input_ui::is_x_pressed(&keyboard);
    let y_pressed = input_ui::is_y_pressed(&keyboard);

    // キー押下順序の追跡
    let x_just_pressed = input_ui::is_x_just_pressed(&keyboard);
    let y_just_pressed = input_ui::is_y_just_pressed(&keyboard);

    // first_axisの更新
    if x_just_pressed && !y_pressed {
        move_state.first_axis = Some(true); // X軸が先
    } else if y_just_pressed && !x_pressed {
        move_state.first_axis = Some(false); // Y軸が先
    } else if !x_pressed && !y_pressed {
        move_state.first_axis = None; // 両方離されたらリセット
    }

    if input_ui::is_up_pressed(&keyboard) {
        dy = 1;
    }
    if input_ui::is_down_pressed(&keyboard) {
        dy = -1;
    }
    if input_ui::is_left_pressed(&keyboard) {
        dx = -1;
    }
    if input_ui::is_right_pressed(&keyboard) {
        dx = 1;
    }

    // 方向キーが押されていない場合はリセット
    if dx == 0 && dy == 0 {
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = (0, 0);
        return;
    }

    // 斜め入力の分解
    let (first_dx, first_dy, pending_direction) = if dx != 0 && dy != 0 {
        // 斜め入力: first_axisに基づいて1回目の方向を決定
        match move_state.first_axis {
            Some(true) => (dx, 0, Some((0, dy))),  // X軸が先
            Some(false) => (0, dy, Some((dx, 0))), // Y軸が先
            None => (dx, 0, Some((0, dy))),        // デフォルトはX軸優先
        }
    } else {
        // 直線入力
        (dx, dy, None)
    };

    let current_direction = (first_dx, first_dy);

    // 方向が変わったか判定（新しい入力として扱う）
    let direction_changed = current_direction != move_state.last_direction;

    let should_move = if direction_changed {
        // 方向変更時は即座に移動、タイマーリセット
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = current_direction;
        true
    } else if move_state.is_repeating {
        // リピート中は通常のタイマーで移動
        move_state.timer.tick(time.delta());
        move_state.timer.just_finished()
    } else {
        // 初回遅延を待つ
        move_state.initial_delay.tick(time.delta());
        if move_state.initial_delay.just_finished() {
            move_state.is_repeating = true;
            move_state.timer.reset();
            true
        } else {
            false
        }
    };

    if !should_move {
        return;
    }

    // 2回目の移動をPendingMoveとして予約するヘルパー
    let add_pending_move = |commands: &mut Commands, entity: Entity, direction: Option<(i32, i32)>| {
        if let Some(dir) = direction {
            commands.entity(entity).insert(PendingMove { direction: dir });
        }
    };

    if let Some(on_boat) = on_boat {
        // === 船での移動 ===
        if let ExecuteMoveResult::Success = execute_boat_move(
            &mut commands, entity, &mut tile_pos, first_dx, first_dy,
            &active_map.grid, on_boat, &mut boat_query,
            &mut moved_events, &mut blocked_events,
        ) {
            add_pending_move(&mut commands, entity, pending_direction);
        }
    } else {
        // === 徒歩での移動 ===
        // 移動先座標を計算
        let (new_x, new_y) = wrap_position(tile_pos.x, tile_pos.y, first_dx, first_dy);

        // 移動先に船があるかチェック（クエリで検索）
        let boat_at_dest = boat_query
            .iter()
            .find(|(_, pos)| pos.x == new_x && pos.y == new_y)
            .map(|(e, _)| e);

        if let Some(boat_entity) = boat_at_dest {
            // 船がある場所への移動 → 乗船
            tile_pos.x = new_x;
            tile_pos.y = new_y;
            add_pending_move(&mut commands, entity, pending_direction);
            moved_events.write(PlayerMovedEvent {
                entity,
                direction: (first_dx, first_dy),
            });
            commands.entity(entity).insert(OnBoat { boat_entity });
        } else {
            // 通常の徒歩移動
            if let ExecuteMoveResult::Success = execute_walk_move(
                &mut tile_pos, entity, first_dx, first_dy,
                &active_map.grid, &mut moved_events, &mut blocked_events,
            ) {
                add_pending_move(&mut commands, entity, pending_direction);
            }
        }
    }
}

/// 船に乗っている間、船のTransformをプレイヤーに同期
pub fn sync_boat_with_player(
    player_query: Query<(&Transform, &OnBoat), With<Player>>,
    mut boat_query: Query<&mut Transform, (With<Boat>, Without<Player>)>,
) {
    let Ok((player_transform, on_boat)) = player_query.single() else {
        return;
    };

    if let Ok(mut boat_transform) = boat_query.get_mut(on_boat.boat_entity) {
        boat_transform.translation.x = player_transform.translation.x;
        boat_transform.translation.y = player_transform.translation.y;
    }
}
