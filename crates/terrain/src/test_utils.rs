//! テスト用ユーティリティ
//!
//! 各モジュールのテストで共通して使用するヘルパー関数を提供する。

use crate::terrain::{Structure, Terrain, MAP_HEIGHT, MAP_WIDTH};

/// テスト用の標準サイズグリッドを作成する
pub fn create_test_grid(default: Terrain) -> Vec<Vec<Terrain>> {
    vec![vec![default; MAP_WIDTH]; MAP_HEIGHT]
}

/// テスト用の任意サイズグリッドを作成する
pub fn create_sized_grid(width: usize, height: usize, default: Terrain) -> Vec<Vec<Terrain>> {
    vec![vec![default; width]; height]
}

/// テスト用の標準サイズ構造物グリッドを作成する
pub fn create_test_structures() -> Vec<Vec<Structure>> {
    vec![vec![Structure::None; MAP_WIDTH]; MAP_HEIGHT]
}

/// テスト用の任意サイズ構造物グリッドを作成する
pub fn create_sized_structures(width: usize, height: usize) -> Vec<Vec<Structure>> {
    vec![vec![Structure::None; width]; height]
}
