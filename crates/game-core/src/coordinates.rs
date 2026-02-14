//! 座標計算ユーティリティ
//!
//! トーラスマップにおける座標ラップ処理と方向定数を提供する。

/// 移動方向を表す型
///
/// `(dx, dy)` の形式で、各値は -1, 0, 1 のいずれか。
/// 斜め移動が禁止されている場合、`dx` と `dy` のどちらかは必ず 0。
pub type Direction = (i32, i32);

/// 4方向（上下左右）の方向ベクトル定数
///
/// 移動や隣接タイル探索に使用する。斜め移動は許可されていない。
pub const ORTHOGONAL_DIRECTIONS: [(i32, i32); 4] = [
    (0, -1), // 上
    (0, 1),  // 下
    (-1, 0), // 左
    (1, 0),  // 右
];

/// 単一の座標値をトーラスマップ上でラップする
///
/// # Arguments
/// * `coord` - 現在の座標値
/// * `delta` - 移動量（負の値も可）
/// * `map_size` - マップのサイズ（幅または高さ）
///
/// # Returns
/// ラップ後の座標値（0 <= result < map_size）
#[inline]
pub fn wrap_coordinate(coord: usize, delta: i32, map_size: usize) -> usize {
    (coord as i32 + delta).rem_euclid(map_size as i32) as usize
}

/// 移動が斜め移動かどうかを判定する
///
/// # Arguments
/// * `dx` - X方向の移動量
/// * `dy` - Y方向の移動量
///
/// # Returns
/// 斜め移動の場合は`true`
#[inline]
pub fn is_diagonal_movement(dx: i32, dy: i32) -> bool {
    dx != 0 && dy != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_coordinate_normal_movement() {
        assert_eq!(wrap_coordinate(5, 1, 100), 6);
        assert_eq!(wrap_coordinate(5, -1, 100), 4);
    }

    #[test]
    fn wrap_coordinate_wraps_at_right_edge() {
        assert_eq!(wrap_coordinate(149, 1, 150), 0);
    }

    #[test]
    fn wrap_coordinate_wraps_at_left_edge() {
        assert_eq!(wrap_coordinate(0, -1, 150), 149);
    }

    #[test]
    fn is_diagonal_movement_detects_diagonal() {
        assert!(is_diagonal_movement(1, 1));
        assert!(is_diagonal_movement(-1, 1));
        assert!(is_diagonal_movement(1, -1));
        assert!(is_diagonal_movement(-1, -1));
    }

    #[test]
    fn is_diagonal_movement_orthogonal_returns_false() {
        assert!(!is_diagonal_movement(1, 0));
        assert!(!is_diagonal_movement(-1, 0));
        assert!(!is_diagonal_movement(0, 1));
        assert!(!is_diagonal_movement(0, -1));
        assert!(!is_diagonal_movement(0, 0));
    }
}
