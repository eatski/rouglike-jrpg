use bevy::prelude::*;

use crate::constants::{MOVEMENT_INITIAL_DELAY, MOVEMENT_REPEAT_INTERVAL};
use party::{default_party, PartyMember};
use terrain::Terrain;
use world::map::MapData;

/// パーティの永続的な状態を管理するリソース（戦闘間でHP/MPを引き継ぐ）
#[derive(Resource)]
pub struct PartyState {
    pub members: Vec<PartyMember>,
}

impl Default for PartyState {
    fn default() -> Self {
        Self {
            members: default_party(),
        }
    }
}

/// 移動状態を管理するリソース
#[derive(Resource)]
pub struct MovementState {
    pub timer: Timer,
    pub initial_delay: Timer,
    pub is_repeating: bool,
    pub last_direction: (i32, i32),
    /// 先に押された軸（Some(true)=X軸、Some(false)=Y軸、None=なし）
    pub first_axis: Option<bool>,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(MOVEMENT_REPEAT_INTERVAL, TimerMode::Repeating),
            initial_delay: Timer::from_seconds(MOVEMENT_INITIAL_DELAY, TimerMode::Once),
            is_repeating: false,
            last_direction: (0, 0),
            first_axis: None,
        }
    }
}

/// 現在のアクティブマップ（フィールド or 洞窟）
#[derive(Resource)]
pub struct ActiveMap {
    pub grid: Vec<Vec<Terrain>>,
    pub width: usize,
    pub height: usize,
    pub origin_x: f32,
    pub origin_y: f32,
}

impl ActiveMap {
    /// タイル座標の地形を取得
    pub fn terrain_at(&self, x: usize, y: usize) -> Terrain {
        self.grid[y][x]
    }

    /// タイル座標をワールド座標に変換
    pub fn to_world(&self, x: usize, y: usize) -> (f32, f32) {
        (
            self.origin_x + x as f32 * crate::TILE_SIZE,
            self.origin_y + y as f32 * crate::TILE_SIZE,
        )
    }

    /// 論理座標（負の値を許容）をワールド座標に変換
    pub fn to_world_logical(&self, x: i32, y: i32) -> (f32, f32) {
        (
            self.origin_x + x as f32 * crate::TILE_SIZE,
            self.origin_y + y as f32 * crate::TILE_SIZE,
        )
    }
}

impl From<MapData> for ActiveMap {
    fn from(map_data: MapData) -> Self {
        let width = map_data.grid[0].len();
        let height = map_data.grid.len();
        let origin_x = -(width as f32 * crate::TILE_SIZE) / 2.0 + crate::TILE_SIZE / 2.0;
        let origin_y = -(height as f32 * crate::TILE_SIZE) / 2.0 + crate::TILE_SIZE / 2.0;
        Self {
            grid: map_data.grid,
            width,
            height,
            origin_x,
            origin_y,
        }
    }
}

/// ワールドマップデータの永続保存用リソース（洞窟進入時に退避）
#[derive(Resource)]
pub struct WorldMapData {
    pub grid: Vec<Vec<Terrain>>,
    pub width: usize,
    pub height: usize,
    pub origin_x: f32,
    pub origin_y: f32,
}
