//! 船移動ロジック

use crate::map::{Terrain, MAP_HEIGHT, MAP_WIDTH};

use super::player::MoveResult;

/// 船での移動を試みる（海のみ移動可能）
pub fn try_move_on_boat(
    current_x: usize,
    current_y: usize,
    dx: i32,
    dy: i32,
    grid: &[Vec<Terrain>],
) -> MoveResult {
    let new_x = ((current_x as i32 + dx).rem_euclid(MAP_WIDTH as i32)) as usize;
    let new_y = ((current_y as i32 + dy).rem_euclid(MAP_HEIGHT as i32)) as usize;

    // 船は海のみ移動可能
    if grid[new_y][new_x] == Terrain::Sea {
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
    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        let nx = (player_x as i32 + dx).rem_euclid(MAP_WIDTH as i32) as usize;
        let ny = (player_y as i32 + dy).rem_euclid(MAP_HEIGHT as i32) as usize;

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
        for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            if dx != preferred_dx || dy != preferred_dy {
                dirs.push((dx, dy));
            }
        }
        dirs
    } else {
        // 移動方向がない場合、全方向をチェック
        vec![(0, -1), (0, 1), (-1, 0), (1, 0)]
    };

    for (dx, dy) in directions {
        let nx = (boat_x as i32 + dx).rem_euclid(MAP_WIDTH as i32) as usize;
        let ny = (boat_y as i32 + dy).rem_euclid(MAP_HEIGHT as i32) as usize;

        if grid[ny][nx] != Terrain::Sea {
            return Some((nx, ny));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_grid(default: Terrain) -> Vec<Vec<Terrain>> {
        vec![vec![default; MAP_WIDTH]; MAP_HEIGHT]
    }

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
}
