use bevy::prelude::*;

use game::remote::RemoteKey;

use crate::remote_control::VirtualInput;

/// 物理キーボードとリモート入力の両方を統一的に扱うヘルパー関数群
pub fn is_up_pressed(keyboard: &ButtonInput<KeyCode>, vi: Option<&VirtualInput>) -> bool {
    keyboard.pressed(KeyCode::KeyW)
        || keyboard.pressed(KeyCode::ArrowUp)
        || vi.is_some_and(|v| v.pressed.contains(&RemoteKey::Up))
}

pub fn is_down_pressed(keyboard: &ButtonInput<KeyCode>, vi: Option<&VirtualInput>) -> bool {
    keyboard.pressed(KeyCode::KeyS)
        || keyboard.pressed(KeyCode::ArrowDown)
        || vi.is_some_and(|v| v.pressed.contains(&RemoteKey::Down))
}

pub fn is_left_pressed(keyboard: &ButtonInput<KeyCode>, vi: Option<&VirtualInput>) -> bool {
    keyboard.pressed(KeyCode::KeyA)
        || keyboard.pressed(KeyCode::ArrowLeft)
        || vi.is_some_and(|v| v.pressed.contains(&RemoteKey::Left))
}

pub fn is_right_pressed(keyboard: &ButtonInput<KeyCode>, vi: Option<&VirtualInput>) -> bool {
    keyboard.pressed(KeyCode::KeyD)
        || keyboard.pressed(KeyCode::ArrowRight)
        || vi.is_some_and(|v| v.pressed.contains(&RemoteKey::Right))
}

pub fn is_up_just_pressed(keyboard: &ButtonInput<KeyCode>, vi: Option<&VirtualInput>) -> bool {
    keyboard.just_pressed(KeyCode::KeyW)
        || keyboard.just_pressed(KeyCode::ArrowUp)
        || vi.is_some_and(|v| v.just_pressed.contains(&RemoteKey::Up))
}

pub fn is_down_just_pressed(keyboard: &ButtonInput<KeyCode>, vi: Option<&VirtualInput>) -> bool {
    keyboard.just_pressed(KeyCode::KeyS)
        || keyboard.just_pressed(KeyCode::ArrowDown)
        || vi.is_some_and(|v| v.just_pressed.contains(&RemoteKey::Down))
}

pub fn is_left_just_pressed(keyboard: &ButtonInput<KeyCode>, vi: Option<&VirtualInput>) -> bool {
    keyboard.just_pressed(KeyCode::KeyA)
        || keyboard.just_pressed(KeyCode::ArrowLeft)
        || vi.is_some_and(|v| v.just_pressed.contains(&RemoteKey::Left))
}

pub fn is_right_just_pressed(keyboard: &ButtonInput<KeyCode>, vi: Option<&VirtualInput>) -> bool {
    keyboard.just_pressed(KeyCode::KeyD)
        || keyboard.just_pressed(KeyCode::ArrowRight)
        || vi.is_some_and(|v| v.just_pressed.contains(&RemoteKey::Right))
}

pub fn is_confirm_just_pressed(
    keyboard: &ButtonInput<KeyCode>,
    vi: Option<&VirtualInput>,
) -> bool {
    keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyZ)
        || vi.is_some_and(|v| v.just_pressed.contains(&RemoteKey::Confirm))
}

pub fn is_cancel_just_pressed(
    keyboard: &ButtonInput<KeyCode>,
    vi: Option<&VirtualInput>,
) -> bool {
    keyboard.just_pressed(KeyCode::Escape)
        || keyboard.just_pressed(KeyCode::KeyX)
        || vi.is_some_and(|v| v.just_pressed.contains(&RemoteKey::Cancel))
}

pub fn is_map_toggle_just_pressed(
    keyboard: &ButtonInput<KeyCode>,
    vi: Option<&VirtualInput>,
) -> bool {
    keyboard.just_pressed(KeyCode::KeyM)
        || vi.is_some_and(|v| {
            v.just_pressed.contains(&RemoteKey::MapToggle)
        })
}

/// x軸方向のキーが押されているか
pub fn is_x_pressed(keyboard: &ButtonInput<KeyCode>, vi: Option<&VirtualInput>) -> bool {
    is_left_pressed(keyboard, vi) || is_right_pressed(keyboard, vi)
}

/// y軸方向のキーが押されているか
pub fn is_y_pressed(keyboard: &ButtonInput<KeyCode>, vi: Option<&VirtualInput>) -> bool {
    is_up_pressed(keyboard, vi) || is_down_pressed(keyboard, vi)
}

/// x軸方向のキーが今フレームで押されたか
pub fn is_x_just_pressed(keyboard: &ButtonInput<KeyCode>, vi: Option<&VirtualInput>) -> bool {
    is_left_just_pressed(keyboard, vi) || is_right_just_pressed(keyboard, vi)
}

/// y軸方向のキーが今フレームで押されたか
pub fn is_y_just_pressed(keyboard: &ButtonInput<KeyCode>, vi: Option<&VirtualInput>) -> bool {
    is_up_just_pressed(keyboard, vi) || is_down_just_pressed(keyboard, vi)
}
