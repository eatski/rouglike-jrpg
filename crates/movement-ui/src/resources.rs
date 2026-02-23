use bevy::prelude::*;

use crate::constants::{MOVEMENT_INITIAL_DELAY, MOVEMENT_REPEAT_INTERVAL, TILE_SIZE};
use terrain::{try_grid_move, MoveResult, Terrain};

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
        }
    }
}

/// 現在のアクティブマップ（フィールド or 洞窟）
#[derive(Resource, Clone)]
pub struct ActiveMap {
    pub grid: Vec<Vec<Terrain>>,
    pub width: usize,
    pub height: usize,
    pub origin_x: f32,
    pub origin_y: f32,
    pub wraps: bool,
}

impl ActiveMap {
    /// グリッドからActiveMapを構築（フィールド用: wraps=true）
    pub fn from_grid(grid: Vec<Vec<Terrain>>) -> Self {
        let width = grid[0].len();
        let height = grid.len();
        let origin_x = -(width as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
        let origin_y = -(height as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
        Self {
            grid,
            width,
            height,
            origin_x,
            origin_y,
            wraps: true,
        }
    }

    /// 徒歩移動（is_walkable）
    pub fn try_move(&self, x: usize, y: usize, dx: i32, dy: i32) -> MoveResult {
        try_grid_move(x, y, dx, dy, &self.grid, self.width, self.height, self.wraps, Terrain::is_walkable)
    }

    /// 任意の通行判定で移動試行
    pub fn try_move_with(&self, x: usize, y: usize, dx: i32, dy: i32, passable: impl Fn(Terrain) -> bool) -> MoveResult {
        try_grid_move(x, y, dx, dy, &self.grid, self.width, self.height, self.wraps, passable)
    }

    /// タイル座標の地形を取得
    pub fn terrain_at(&self, x: usize, y: usize) -> Terrain {
        self.grid[y][x]
    }

    /// タイル座標をワールド座標に変換
    pub fn to_world(&self, x: usize, y: usize) -> (f32, f32) {
        (
            self.origin_x + x as f32 * TILE_SIZE,
            self.origin_y + y as f32 * TILE_SIZE,
        )
    }

    /// 論理座標（負の値を許容）をワールド座標に変換
    pub fn to_world_logical(&self, x: i32, y: i32) -> (f32, f32) {
        (
            self.origin_x + x as f32 * TILE_SIZE,
            self.origin_y + y as f32 * TILE_SIZE,
        )
    }
}

/// ワールドマップデータの永続保存用リソース（洞窟進入時に退避）
#[derive(Resource)]
pub struct WorldMapData(pub ActiveMap);
