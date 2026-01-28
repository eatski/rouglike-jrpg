use bevy::prelude::*;

use game::movement::{try_move, MoveResult};

use crate::components::{MovementLocked, Player, TilePosition};
use crate::events::{MovementBlockedEvent, PlayerMovedEvent};
use crate::resources::{MapDataResource, MovementState};

/// プレイヤーの移動入力を処理するシステム
pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    map_data: Res<MapDataResource>,
    mut move_state: ResMut<MovementState>,
    mut query: Query<(Entity, &mut TilePosition, Option<&MovementLocked>), With<Player>>,
    mut blocked_events: MessageWriter<MovementBlockedEvent>,
    mut moved_events: MessageWriter<PlayerMovedEvent>,
) {
    let Ok((entity, mut tile_pos, locked)) = query.single_mut() else {
        return;
    };

    // 移動ロック中は入力を無視
    if locked.is_some() {
        return;
    }

    let mut dx: i32 = 0;
    let mut dy: i32 = 0;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        dy = 1;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        dy = -1;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dx = -1;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dx = 1;
    }

    let current_direction = (dx, dy);

    // 方向キーが押されていない場合はリセット
    if dx == 0 && dy == 0 {
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = (0, 0);
        return;
    }

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

    if should_move {
        match try_move(tile_pos.x, tile_pos.y, dx, dy, &map_data.grid) {
            MoveResult::Blocked => {
                blocked_events.write(MovementBlockedEvent {
                    entity,
                    direction: (dx, dy),
                });
            }
            MoveResult::Moved { new_x, new_y } => {
                tile_pos.x = new_x;
                tile_pos.y = new_y;
                moved_events.write(PlayerMovedEvent {
                    entity,
                    direction: (dx, dy),
                });
            }
        }
    }
}
