use bevy::prelude::*;

use crate::constants::TILE_SIZE;
use terrain::{try_grid_move, MoveResult, Structure, Terrain, TileAction};

/// 現在のアクティブマップ（フィールド or 洞窟）
#[derive(Resource, Clone)]
pub struct ActiveMap {
    pub grid: Vec<Vec<Terrain>>,
    pub structures: Vec<Vec<Structure>>,
    pub width: usize,
    pub height: usize,
    pub origin_x: f32,
    pub origin_y: f32,
    pub wraps: bool,
}

impl ActiveMap {
    /// グリッドからActiveMapを構築（フィールド用: wraps=true）
    pub fn from_grid(grid: Vec<Vec<Terrain>>, structures: Vec<Vec<Structure>>) -> Self {
        let width = grid[0].len();
        let height = grid.len();
        let origin_x = -(width as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
        let origin_y = -(height as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
        Self {
            grid,
            structures,
            width,
            height,
            origin_x,
            origin_y,
            wraps: true,
        }
    }

    /// タイル座標の構造物を取得
    pub fn structure_at(&self, x: usize, y: usize) -> Structure {
        self.structures[y][x]
    }

    /// エンカウント率を返す（構造物がある場合は0.0）
    pub fn encounter_rate_at(&self, x: usize, y: usize) -> f32 {
        if self.structures[y][x] != Structure::None {
            return 0.0;
        }
        self.grid[y][x].encounter_rate()
    }

    /// タイルアクションを返す（構造物に基づく）
    pub fn tile_action_at(&self, x: usize, y: usize) -> TileAction {
        self.structures[y][x].tile_action()
    }

    /// 徒歩移動（is_walkable + 構造物は通行可能）
    pub fn try_move(&self, x: usize, y: usize, dx: i32, dy: i32) -> MoveResult {
        let s = &self.structures;
        try_grid_move(x, y, dx, dy, &self.grid, self.width, self.height, self.wraps,
            |nx, ny, t| s[ny][nx] != Structure::None || t.is_walkable())
    }

    /// 任意の通行判定で移動試行
    pub fn try_move_with(&self, x: usize, y: usize, dx: i32, dy: i32, passable: impl Fn(usize, usize, Terrain) -> bool) -> MoveResult {
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
