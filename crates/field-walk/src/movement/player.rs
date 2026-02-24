use crate::map::{Terrain, MAP_HEIGHT, MAP_WIDTH};

pub use terrain::MoveResult;

/// 移動を試みる（純粋関数）
///
/// タイル座標と方向を受け取り、移動結果を返す。
/// マップ端ではラップアラウンドする。
/// 斜め移動は許可しない（隣接タイルが通行不可でも斜めに抜けられてしまうため）。
pub fn try_move(
    current_x: usize,
    current_y: usize,
    dx: i32,
    dy: i32,
    grid: &[Vec<Terrain>],
) -> MoveResult {
    terrain::try_grid_move(current_x, current_y, dx, dy, grid, MAP_WIDTH, MAP_HEIGHT, true, Terrain::is_walkable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{MAP_HEIGHT, MAP_WIDTH};
    use crate::test_utils::create_test_grid;

    // ============================================
    // try_move のテスト
    // ============================================

    #[test]
    fn try_move_succeeds_on_plains() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Plains;

        let result = try_move(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_move_succeeds_on_forest() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Forest;

        let result = try_move(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_move_blocked_by_mountain() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Mountain;

        let result = try_move(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Blocked);
    }

    #[test]
    fn try_move_blocked_by_sea() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        // grid[5][6] はデフォルトでSea

        let result = try_move(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Blocked);
    }

    #[test]
    fn try_move_wraps_around_right_edge() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][MAP_WIDTH - 1] = Terrain::Plains;
        grid[5][0] = Terrain::Plains; // ラップ先

        let result = try_move(MAP_WIDTH - 1, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 0, new_y: 5 });
    }

    #[test]
    fn try_move_wraps_around_left_edge() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][0] = Terrain::Plains;
        grid[5][MAP_WIDTH - 1] = Terrain::Plains; // ラップ先

        let result = try_move(0, 5, -1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: MAP_WIDTH - 1, new_y: 5 });
    }

    #[test]
    fn try_move_wraps_around_top_edge() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[MAP_HEIGHT - 1][5] = Terrain::Plains;
        grid[0][5] = Terrain::Plains; // ラップ先

        let result = try_move(5, MAP_HEIGHT - 1, 0, 1, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 5, new_y: 0 });
    }

    #[test]
    fn try_move_wraps_around_bottom_edge() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[0][5] = Terrain::Plains;
        grid[MAP_HEIGHT - 1][5] = Terrain::Plains; // ラップ先

        let result = try_move(5, 0, 0, -1, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 5, new_y: MAP_HEIGHT - 1 });
    }

    #[test]
    fn try_move_diagonal_movement_is_blocked() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[6][6] = Terrain::Plains;

        // 斜め移動は禁止されている
        let result = try_move(5, 5, 1, 1, &grid);

        assert_eq!(result, MoveResult::Blocked);
    }

    #[test]
    fn try_move_no_movement() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;

        let result = try_move(5, 5, 0, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 5, new_y: 5 });
    }
}
