use bevy::prelude::*;

use crate::constants::{MOVEMENT_INITIAL_DELAY, MOVEMENT_REPEAT_INTERVAL};

/// 移動状態を管理するリソース
#[derive(Resource)]
pub struct MovementState {
    pub timer: Timer,
    pub initial_delay: Timer,
    pub is_repeating: bool,
    pub last_direction: (i32, i32),
    /// 先に押された軸（Some(true)=X軸、Some(false)=Y軸、None=なし）
    pub first_axis: Option<bool>,
    /// SmoothMoveアニメーション完了時にセットされるフラグ（完了したEntity）
    pub move_just_completed: Option<Entity>,
    /// 中間タイル到着後、次フレームでPendingMoveを実行するためのフラグ
    pub pending_move_ready: Option<Entity>,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(MOVEMENT_REPEAT_INTERVAL, TimerMode::Repeating),
            initial_delay: Timer::from_seconds(MOVEMENT_INITIAL_DELAY, TimerMode::Once),
            is_repeating: false,
            last_direction: (0, 0),
            first_axis: None,
            move_just_completed: None,
            pending_move_ready: None,
        }
    }
}
