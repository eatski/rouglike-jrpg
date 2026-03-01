use bevy::prelude::*;

/// 移動処理中かどうかを示すマーカーコンポーネント
#[derive(Component)]
pub struct MovementLocked;

/// 予約された次の移動方向（斜め移動の2回目）
#[derive(Component)]
pub struct PendingMove {
    pub direction: (i32, i32),
}
