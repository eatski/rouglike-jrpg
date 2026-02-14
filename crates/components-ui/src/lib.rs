use bevy::prelude::*;

/// プレイヤーエンティティを識別するマーカーコンポーネント
#[derive(Component)]
pub struct Player;

/// 移動処理中かどうかを示すマーカーコンポーネント
#[derive(Component)]
pub struct MovementLocked;

/// タイル座標を保持するコンポーネント
#[derive(Component)]
pub struct TilePosition {
    pub x: usize,
    pub y: usize,
}

/// マップタイルを識別するマーカーコンポーネント
#[derive(Component)]
pub struct MapTile;

/// 船エンティティを識別するマーカーコンポーネント
#[derive(Component)]
pub struct Boat;

/// プレイヤーが船に乗っている状態を示すコンポーネント
#[derive(Component)]
pub struct OnBoat {
    /// 乗っている船のエンティティ
    pub boat_entity: Entity,
}

/// 予約された次の移動方向（斜め移動の2回目）
#[derive(Component)]
pub struct PendingMove {
    pub direction: (i32, i32),
}
