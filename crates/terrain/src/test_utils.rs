//! テスト用ユーティリティ
//!
//! 各モジュールのテストで共通して使用するヘルパー関数を提供する。

use crate::terrain::{Terrain, MAP_HEIGHT, MAP_WIDTH};

/// テスト用の標準サイズグリッドを作成する
pub fn create_test_grid(default: Terrain) -> Vec<Vec<Terrain>> {
    vec![vec![default; MAP_WIDTH]; MAP_HEIGHT]
}

/// テスト用の任意サイズグリッドを作成する
pub fn create_sized_grid(width: usize, height: usize, default: Terrain) -> Vec<Vec<Terrain>> {
    vec![vec![default; width]; height]
}
