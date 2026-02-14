use bevy::prelude::*;

/// プレイヤーのスポーン位置を保持するリソース
#[derive(Resource)]
pub struct SpawnPosition {
    pub x: usize,
    pub y: usize,
}
