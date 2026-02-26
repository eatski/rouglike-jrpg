use bevy::prelude::*;

/// キーボード入力のヘルパー関数群
pub fn is_up_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp)
}

pub fn is_down_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown)
}

pub fn is_left_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft)
}

pub fn is_right_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight)
}

pub fn is_up_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::KeyW) || keyboard.just_pressed(KeyCode::ArrowUp)
}

pub fn is_down_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::KeyS) || keyboard.just_pressed(KeyCode::ArrowDown)
}

pub fn is_left_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::KeyA) || keyboard.just_pressed(KeyCode::ArrowLeft)
}

pub fn is_right_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::KeyD) || keyboard.just_pressed(KeyCode::ArrowRight)
}

pub fn is_confirm_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyZ)
}

pub fn clear_confirm_just_pressed(keyboard: &mut ButtonInput<KeyCode>) {
    keyboard.clear_just_pressed(KeyCode::Enter);
    keyboard.clear_just_pressed(KeyCode::Space);
    keyboard.clear_just_pressed(KeyCode::KeyZ);
}

pub fn is_cancel_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyX)
}

pub fn is_menu_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::Tab)
}

pub fn is_map_toggle_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::KeyM)
}

/// x軸方向のキーが押されているか
pub fn is_x_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    is_left_pressed(keyboard) || is_right_pressed(keyboard)
}

/// y軸方向のキーが押されているか
pub fn is_y_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    is_up_pressed(keyboard) || is_down_pressed(keyboard)
}

/// x軸方向のキーが今フレームで押されたか
pub fn is_x_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    is_left_just_pressed(keyboard) || is_right_just_pressed(keyboard)
}

/// y軸方向のキーが今フレームで押されたか
pub fn is_y_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    is_up_just_pressed(keyboard) || is_down_just_pressed(keyboard)
}
