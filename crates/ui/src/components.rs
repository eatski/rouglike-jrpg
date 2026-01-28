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
