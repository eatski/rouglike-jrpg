use bevy::prelude::*;

/// プレイヤーエンティティを識別するマーカーコンポーネント
#[derive(Component)]
pub struct Player;

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
