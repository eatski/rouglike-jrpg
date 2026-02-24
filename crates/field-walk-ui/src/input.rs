use bevy::prelude::*;

use crate::MovementState;

/// 移動入力の結果
pub struct MovementInput {
    /// 1回目の移動方向
    pub first_dx: i32,
    pub first_dy: i32,
    /// 斜め入力時の2回目の方向（直線入力ならNone）
    pub pending_direction: Option<(i32, i32)>,
}

/// キー入力を処理し、移動すべき方向を返す。
/// None: 移動なし（キー未押下 or タイマー未発火）
pub fn process_movement_input(
    keyboard: &ButtonInput<KeyCode>,
    time: &Time,
    move_state: &mut MovementState,
) -> Option<MovementInput> {
    let mut dx: i32 = 0;
    let mut dy: i32 = 0;

    let x_pressed = input_ui::is_x_pressed(keyboard);
    let y_pressed = input_ui::is_y_pressed(keyboard);
    let x_just_pressed = input_ui::is_x_just_pressed(keyboard);
    let y_just_pressed = input_ui::is_y_just_pressed(keyboard);

    // first_axisの更新
    if x_just_pressed && !y_pressed {
        move_state.first_axis = Some(true); // X軸が先
    } else if y_just_pressed && !x_pressed {
        move_state.first_axis = Some(false); // Y軸が先
    } else if !x_pressed && !y_pressed {
        move_state.first_axis = None; // 両方離されたらリセット
    }

    if input_ui::is_up_pressed(keyboard) {
        dy = 1;
    }
    if input_ui::is_down_pressed(keyboard) {
        dy = -1;
    }
    if input_ui::is_left_pressed(keyboard) {
        dx = -1;
    }
    if input_ui::is_right_pressed(keyboard) {
        dx = 1;
    }

    // 方向キーが押されていない場合はリセット
    if dx == 0 && dy == 0 {
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = (0, 0);
        return None;
    }

    // 斜め入力の分解
    let (first_dx, first_dy, pending_direction) = if dx != 0 && dy != 0 {
        match move_state.first_axis {
            Some(true) => (dx, 0, Some((0, dy))),  // X軸が先
            Some(false) => (0, dy, Some((dx, 0))), // Y軸が先
            None => (dx, 0, Some((0, dy))),        // デフォルトはX軸優先
        }
    } else {
        (dx, dy, None)
    };

    let current_direction = (first_dx, first_dy);
    let direction_changed = current_direction != move_state.last_direction;

    let should_move = if direction_changed {
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = current_direction;
        true
    } else if move_state.is_repeating {
        move_state.timer.tick(time.delta());
        move_state.timer.just_finished()
    } else {
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
        return None;
    }

    Some(MovementInput {
        first_dx,
        first_dy,
        pending_direction,
    })
}
