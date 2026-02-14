//! 座標計算ユーティリティ
//!
//! game-coreの基本座標関数を再エクスポートし、
//! フィールドマップ固有のラップ関数を提供する。

pub use game_core::coordinates::{
    is_diagonal_movement, wrap_coordinate, Direction, ORTHOGONAL_DIRECTIONS,
};
use game_core::terrain::{MAP_HEIGHT, MAP_WIDTH};

/// 2D座標をトーラスマップ上でラップする
#[inline]
pub fn wrap_position(x: usize, y: usize, dx: i32, dy: i32) -> (usize, usize) {
    (
        wrap_coordinate(x, dx, MAP_WIDTH),
        wrap_coordinate(y, dy, MAP_HEIGHT),
    )
}

/// 指定座標の4近傍（上下左右）の座標を取得する
///
/// トーラスマップのラップアラウンドを考慮。
pub fn orthogonal_neighbors(x: usize, y: usize) -> [(usize, usize); 4] {
    [
        wrap_position(x, y, 0, -1), // 上
        wrap_position(x, y, 0, 1),  // 下
        wrap_position(x, y, -1, 0), // 左
        wrap_position(x, y, 1, 0),  // 右
    ]
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
        assert_eq!(wrap_coordinate(MAP_WIDTH - 1, 1, MAP_WIDTH), 0);
    }

    #[test]
    fn wrap_coordinate_wraps_at_left_edge() {
        assert_eq!(wrap_coordinate(0, -1, MAP_WIDTH), MAP_WIDTH - 1);
    }

    #[test]
    fn wrap_position_wraps_both_axes() {
        let (x, y) = wrap_position(MAP_WIDTH - 1, MAP_HEIGHT - 1, 1, 1);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
    }

    #[test]
    fn orthogonal_neighbors_at_center() {
        let neighbors = orthogonal_neighbors(5, 5);
        assert!(neighbors.contains(&(5, 4))); // 上
        assert!(neighbors.contains(&(5, 6))); // 下
        assert!(neighbors.contains(&(4, 5))); // 左
        assert!(neighbors.contains(&(6, 5))); // 右
    }

    #[test]
    fn orthogonal_neighbors_at_corner() {
        let neighbors = orthogonal_neighbors(0, 0);
        assert!(neighbors.contains(&(0, MAP_HEIGHT - 1))); // 上（ラップ）
        assert!(neighbors.contains(&(0, 1))); // 下
        assert!(neighbors.contains(&(MAP_WIDTH - 1, 0))); // 左（ラップ）
        assert!(neighbors.contains(&(1, 0))); // 右
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
