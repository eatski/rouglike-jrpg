use bevy::prelude::*;
use std::collections::HashMap;

use game::map::{MapData, Terrain};

/// マップデータをBevyリソースとしてラップする
#[derive(Resource)]
pub struct MapDataResource {
    pub grid: Vec<Vec<Terrain>>,
    pub spawn_position: (usize, usize),
}

impl From<MapData> for MapDataResource {
    fn from(map_data: MapData) -> Self {
        Self {
            grid: map_data.grid,
            spawn_position: map_data.spawn_position,
        }
    }
}

/// プレイヤーのスポーン位置を保持するリソース
#[derive(Resource)]
pub struct SpawnPosition {
    pub x: usize,
    pub y: usize,
}

/// 移動状態を管理するリソース
#[derive(Resource)]
pub struct MovementState {
    pub timer: Timer,
    pub initial_delay: Timer,
    pub is_repeating: bool,
    pub last_direction: (i32, i32),
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            // リピート間隔: 60ms (約16.7歩/秒) - キビキビした移動感
            timer: Timer::from_seconds(0.06, TimerMode::Repeating),
            // 初回遅延: 150ms - 誤入力防止とレスポンスのバランス
            initial_delay: Timer::from_seconds(0.15, TimerMode::Once),
            is_repeating: false,
            last_direction: (0, 0),
        }
    }
}

/// 船の位置情報を管理するリソース
#[derive(Resource, Default)]
pub struct BoatPositions {
    /// Entity と タイル座標のマッピング
    pub positions: HashMap<Entity, (usize, usize)>,
}
