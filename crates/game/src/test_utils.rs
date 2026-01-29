//! テスト用ユーティリティ
//!
//! 各モジュールのテストで共通して使用するヘルパー関数を提供する。

use crate::map::{Terrain, MAP_HEIGHT, MAP_WIDTH};

/// テスト用の標準サイズグリッドを作成する
///
/// # Arguments
/// * `default` - グリッド全体を埋める地形
///
/// # Returns
/// `MAP_WIDTH x MAP_HEIGHT` のサイズで、指定した地形で埋められたグリッド
pub fn create_test_grid(default: Terrain) -> Vec<Vec<Terrain>> {
    vec![vec![default; MAP_WIDTH]; MAP_HEIGHT]
}

/// テスト用の任意サイズグリッドを作成する
///
/// # Arguments
/// * `width` - グリッドの幅
/// * `height` - グリッドの高さ
/// * `default` - グリッド全体を埋める地形
///
/// # Returns
/// 指定したサイズと地形で埋められたグリッド
pub fn create_sized_grid(width: usize, height: usize, default: Terrain) -> Vec<Vec<Terrain>> {
    vec![vec![default; width]; height]
}
