use bevy::prelude::*;

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
            timer: Timer::from_seconds(0.08, TimerMode::Repeating),
            initial_delay: Timer::from_seconds(0.2, TimerMode::Once),
            is_repeating: false,
            last_direction: (0, 0),
        }
    }
}
