//! 船移動ロジック

use crate::coordinates::{is_diagonal_movement, orthogonal_neighbors, wrap_position, ORTHOGONAL_DIRECTIONS};
use crate::map::Terrain;

use super::player::MoveResult;

/// 船での移動を試みる（海のみ移動可能、斜め移動禁止）
pub fn try_move_on_boat(
    current_x: usize,
    current_y: usize,
    dx: i32,
    dy: i32,
    grid: &[Vec<Terrain>],
) -> MoveResult {
    // 斜め移動は禁止
    if is_diagonal_movement(dx, dy) {
        return MoveResult::Blocked;
    }

    let (new_x, new_y) = wrap_position(current_x, current_y, dx, dy);

    // 船は海のみ移動可能
    if grid[new_y][new_x].is_navigable() {
        MoveResult::Moved { new_x, new_y }
    } else {
        MoveResult::Blocked
    }
}

/// 指定座標に隣接する船の位置を取得
pub fn find_adjacent_boat(
    player_x: usize,
    player_y: usize,
    boat_positions: &[(usize, usize)],
) -> Option<(usize, usize)> {
    for (nx, ny) in orthogonal_neighbors(player_x, player_y) {
        if boat_positions.contains(&(nx, ny)) {
            return Some((nx, ny));
        }
    }
    None
}

/// 下船可能な位置を取得（船の隣の陸地、移動方向優先）
pub fn find_disembark_position(
    boat_x: usize,
    boat_y: usize,
    preferred_dx: i32,
    preferred_dy: i32,
    grid: &[Vec<Terrain>],
) -> Option<(usize, usize)> {
    // 移動方向を優先してチェック
    let directions = if preferred_dx != 0 || preferred_dy != 0 {
        // 移動方向がある場合、その方向を最初にチェック
        let mut dirs = vec![(preferred_dx, preferred_dy)];
        for (dx, dy) in ORTHOGONAL_DIRECTIONS {
            if dx != preferred_dx || dy != preferred_dy {
                dirs.push((dx, dy));
            }
        }
        dirs
    } else {
        // 移動方向がない場合、全方向をチェック
        ORTHOGONAL_DIRECTIONS.to_vec()
    };

    for (dx, dy) in directions {
        let (nx, ny) = wrap_position(boat_x, boat_y, dx, dy);

        if grid[ny][nx].is_walkable() {
            return Some((nx, ny));
        }
    }
    None
}

/// 船移動の結果を表す列挙型
///
/// `MoveResult` と異なり、下船という成功パターンを持つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatMoveResult {
    /// 海上を移動した
    MovedOnSea { new_x: usize, new_y: usize },
    /// 陸地に下船した
    Disembarked { new_x: usize, new_y: usize },
    /// 移動失敗（斜め移動など）
    Blocked,
}

/// 船での移動または下船を試みる
///
/// 海への移動は継続航行、陸地への移動は下船として処理する。
/// 斜め移動は許可されない。
pub fn try_boat_move_or_disembark(
    boat_x: usize,
    boat_y: usize,
    dx: i32,
    dy: i32,
    grid: &[Vec<Terrain>],
) -> BoatMoveResult {
    // 斜め移動は禁止
    if is_diagonal_movement(dx, dy) {
        return BoatMoveResult::Blocked;
    }

    // 移動なし
    if dx == 0 && dy == 0 {
        return BoatMoveResult::MovedOnSea {
            new_x: boat_x,
            new_y: boat_y,
        };
    }

    let (new_x, new_y) = wrap_position(boat_x, boat_y, dx, dy);

    if grid[new_y][new_x].is_navigable() {
        BoatMoveResult::MovedOnSea { new_x, new_y }
    } else {
        BoatMoveResult::Disembarked { new_x, new_y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_grid;

    #[test]
    fn try_move_on_boat_succeeds_on_sea() {
        let grid = create_test_grid(Terrain::Sea);

        let result = try_move_on_boat(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_move_on_boat_blocked_by_land() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][6] = Terrain::Plains;

        let result = try_move_on_boat(5, 5, 1, 0, &grid);

        assert_eq!(result, MoveResult::Blocked);
    }

    #[test]
    fn find_adjacent_boat_finds_boat() {
        let boat_positions = vec![(6, 5), (10, 10)];

        let result = find_adjacent_boat(5, 5, &boat_positions);

        assert_eq!(result, Some((6, 5)));
    }

    #[test]
    fn find_adjacent_boat_returns_none_when_no_boat() {
        let boat_positions = vec![(10, 10)];

        let result = find_adjacent_boat(5, 5, &boat_positions);

        assert_eq!(result, None);
    }

    #[test]
    fn find_disembark_position_prefers_direction() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][6] = Terrain::Plains; // 右に陸地
        grid[4][5] = Terrain::Plains; // 上にも陸地

        // 右方向を優先
        let result = find_disembark_position(5, 5, 1, 0, &grid);
        assert_eq!(result, Some((6, 5)));

        // 上方向を優先
        let result = find_disembark_position(5, 5, 0, -1, &grid);
        assert_eq!(result, Some((5, 4)));
    }

    // ============================================
    // try_boat_move_or_disembark のテスト
    // ============================================

    #[test]
    fn try_boat_move_or_disembark_moves_on_sea() {
        let grid = create_test_grid(Terrain::Sea);

        let result = try_boat_move_or_disembark(5, 5, 1, 0, &grid);

        assert_eq!(result, BoatMoveResult::MovedOnSea { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_boat_move_or_disembark_disembarks_on_land() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][6] = Terrain::Plains;

        let result = try_boat_move_or_disembark(5, 5, 1, 0, &grid);

        assert_eq!(result, BoatMoveResult::Disembarked { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_boat_move_or_disembark_blocks_diagonal() {
        let grid = create_test_grid(Terrain::Sea);

        let result = try_boat_move_or_disembark(5, 5, 1, 1, &grid);

        assert_eq!(result, BoatMoveResult::Blocked);
    }

    #[test]
    fn try_boat_move_or_disembark_no_movement() {
        let grid = create_test_grid(Terrain::Sea);

        let result = try_boat_move_or_disembark(5, 5, 0, 0, &grid);

        assert_eq!(result, BoatMoveResult::MovedOnSea { new_x: 5, new_y: 5 });
    }
}
