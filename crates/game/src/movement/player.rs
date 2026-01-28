use crate::map::{Terrain, MAP_HEIGHT, MAP_WIDTH};

/// 移動試行の結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    /// 移動成功: 新しい位置
    Moved { new_x: usize, new_y: usize },
    /// 移動失敗: 海に阻まれた
    Blocked,
}

/// 移動を試みる（純粋関数）
///
/// タイル座標と方向を受け取り、移動結果を返す。
/// マップ端ではラップアラウンドする。
pub fn try_move(
    current_x: usize,
    current_y: usize,
    dx: i32,
    dy: i32,
    grid: &[Vec<Terrain>],
) -> MoveResult {
    let new_x = ((current_x as i32 + dx).rem_euclid(MAP_WIDTH as i32)) as usize;
    let new_y = ((current_y as i32 + dy).rem_euclid(MAP_HEIGHT as i32)) as usize;

    if grid[new_y][new_x] == Terrain::Sea {
        MoveResult::Blocked
    } else {
        MoveResult::Moved { new_x, new_y }
    }
}

/// 地形が通行可能かどうかを判定する
pub fn is_passable(terrain: Terrain) -> bool {
    terrain != Terrain::Sea
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用の小さなグリッドを作成するヘルパー
    fn create_test_grid(width: usize, height: usize, default: Terrain) -> Vec<Vec<Terrain>> {
        vec![vec![default; width]; height]
    }

    // ============================================
    // try_move のテスト
    // ============================================

    #[test]
    fn try_move_succeeds_on_plains() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Plains;

        let result = try_move(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_move_succeeds_on_forest() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Forest;

        let result = try_move(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_move_succeeds_on_mountain() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Mountain;

        let result = try_move(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_move_blocked_by_sea() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        // grid[5][6] はデフォルトでSea

        let result = try_move(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Blocked);
    }

    #[test]
    fn try_move_wraps_around_right_edge() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][MAP_WIDTH - 1] = Terrain::Plains;
        grid[5][0] = Terrain::Plains; // ラップ先

        let result = try_move(MAP_WIDTH - 1, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 0, new_y: 5 });
    }

    #[test]
    fn try_move_wraps_around_left_edge() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][0] = Terrain::Plains;
        grid[5][MAP_WIDTH - 1] = Terrain::Plains; // ラップ先

        let result = try_move(0, 5, -1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: MAP_WIDTH - 1, new_y: 5 });
    }

    #[test]
    fn try_move_wraps_around_top_edge() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[MAP_HEIGHT - 1][5] = Terrain::Plains;
        grid[0][5] = Terrain::Plains; // ラップ先

        let result = try_move(5, MAP_HEIGHT - 1, 0, 1, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 5, new_y: 0 });
    }

    #[test]
    fn try_move_wraps_around_bottom_edge() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[0][5] = Terrain::Plains;
        grid[MAP_HEIGHT - 1][5] = Terrain::Plains; // ラップ先

        let result = try_move(5, 0, 0, -1, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 5, new_y: MAP_HEIGHT - 1 });
    }

    #[test]
    fn try_move_diagonal_movement() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[6][6] = Terrain::Plains;

        let result = try_move(5, 5, 1, 1, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 6 });
    }

    #[test]
    fn try_move_no_movement() {
        let mut grid = create_test_grid(MAP_WIDTH, MAP_HEIGHT, Terrain::Sea);
        grid[5][5] = Terrain::Plains;

        let result = try_move(5, 5, 0, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 5, new_y: 5 });
    }

    // ============================================
    // is_passable のテスト
    // ============================================

    #[test]
    fn is_passable_plains() {
        assert!(is_passable(Terrain::Plains));
    }

    #[test]
    fn is_passable_forest() {
        assert!(is_passable(Terrain::Forest));
    }

    #[test]
    fn is_passable_mountain() {
        assert!(is_passable(Terrain::Mountain));
    }

    #[test]
    fn is_passable_sea_returns_false() {
        assert!(!is_passable(Terrain::Sea));
    }
}
