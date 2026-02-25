use crate::coordinates::{is_diagonal_movement, wrap_coordinate};
use crate::terrain::Terrain;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Moved { new_x: usize, new_y: usize },
    Blocked,
}

#[allow(clippy::too_many_arguments)]
pub fn try_grid_move(
    x: usize,
    y: usize,
    dx: i32,
    dy: i32,
    grid: &[Vec<Terrain>],
    width: usize,
    height: usize,
    wraps: bool,
    passable: impl Fn(usize, usize, Terrain) -> bool,
) -> MoveResult {
    if is_diagonal_movement(dx, dy) {
        return MoveResult::Blocked;
    }

    let (new_x, new_y) = if wraps {
        (
            wrap_coordinate(x, dx, width),
            wrap_coordinate(y, dy, height),
        )
    } else {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || nx >= width as i32 || ny < 0 || ny >= height as i32 {
            return MoveResult::Blocked;
        }
        (nx as usize, ny as usize)
    };

    if passable(new_x, new_y, grid[new_y][new_x]) {
        MoveResult::Moved { new_x, new_y }
    } else {
        MoveResult::Blocked
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_sized_grid, create_test_grid};
    use crate::terrain::{MAP_HEIGHT, MAP_WIDTH};

    // ============================================
    // wraps=true (フィールド) のテスト
    // ============================================

    #[test]
    fn try_grid_move_wraps_succeeds_on_plains() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Plains;

        let result = try_grid_move(5, 5, 1, 0, &grid, MAP_WIDTH, MAP_HEIGHT, true, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_grid_move_wraps_succeeds_on_forest() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Forest;

        let result = try_grid_move(5, 5, 1, 0, &grid, MAP_WIDTH, MAP_HEIGHT, true, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_grid_move_wraps_blocked_by_mountain() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Mountain;

        let result = try_grid_move(5, 5, 1, 0, &grid, MAP_WIDTH, MAP_HEIGHT, true, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Blocked);
    }

    #[test]
    fn try_grid_move_wraps_blocked_by_sea() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;

        let result = try_grid_move(5, 5, 1, 0, &grid, MAP_WIDTH, MAP_HEIGHT, true, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Blocked);
    }

    #[test]
    fn try_grid_move_wraps_around_right_edge() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][MAP_WIDTH - 1] = Terrain::Plains;
        grid[5][0] = Terrain::Plains;

        let result = try_grid_move(MAP_WIDTH - 1, 5, 1, 0, &grid, MAP_WIDTH, MAP_HEIGHT, true, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Moved { new_x: 0, new_y: 5 });
    }

    #[test]
    fn try_grid_move_wraps_around_left_edge() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][0] = Terrain::Plains;
        grid[5][MAP_WIDTH - 1] = Terrain::Plains;

        let result = try_grid_move(0, 5, -1, 0, &grid, MAP_WIDTH, MAP_HEIGHT, true, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Moved { new_x: MAP_WIDTH - 1, new_y: 5 });
    }

    #[test]
    fn try_grid_move_wraps_around_top_edge() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[MAP_HEIGHT - 1][5] = Terrain::Plains;
        grid[0][5] = Terrain::Plains;

        let result = try_grid_move(5, MAP_HEIGHT - 1, 0, 1, &grid, MAP_WIDTH, MAP_HEIGHT, true, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Moved { new_x: 5, new_y: 0 });
    }

    #[test]
    fn try_grid_move_wraps_around_bottom_edge() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[0][5] = Terrain::Plains;
        grid[MAP_HEIGHT - 1][5] = Terrain::Plains;

        let result = try_grid_move(5, 0, 0, -1, &grid, MAP_WIDTH, MAP_HEIGHT, true, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Moved { new_x: 5, new_y: MAP_HEIGHT - 1 });
    }

    #[test]
    fn try_grid_move_diagonal_is_blocked() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[6][6] = Terrain::Plains;

        let result = try_grid_move(5, 5, 1, 1, &grid, MAP_WIDTH, MAP_HEIGHT, true, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Blocked);
    }

    #[test]
    fn try_grid_move_no_movement() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;

        let result = try_grid_move(5, 5, 0, 0, &grid, MAP_WIDTH, MAP_HEIGHT, true, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Moved { new_x: 5, new_y: 5 });
    }

    // ============================================
    // wraps=false (洞窟) のテスト
    // ============================================

    #[test]
    fn try_grid_move_nowrap_normal() {
        let mut grid = create_sized_grid(5, 5, Terrain::CaveWall);
        grid[2][2] = Terrain::CaveFloor;
        grid[2][3] = Terrain::CaveFloor;

        let result = try_grid_move(2, 2, 1, 0, &grid, 5, 5, false, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Moved { new_x: 3, new_y: 2 });
    }

    #[test]
    fn try_grid_move_nowrap_blocked_by_wall() {
        let mut grid = create_sized_grid(5, 5, Terrain::CaveWall);
        grid[2][2] = Terrain::CaveFloor;

        let result = try_grid_move(2, 2, 1, 0, &grid, 5, 5, false, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Blocked);
    }

    #[test]
    fn try_grid_move_nowrap_blocked_by_boundary() {
        let grid = create_sized_grid(5, 5, Terrain::CaveFloor);

        let result = try_grid_move(0, 0, -1, 0, &grid, 5, 5, false, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Blocked);
    }

    #[test]
    fn try_grid_move_nowrap_diagonal_blocked() {
        let grid = create_sized_grid(5, 5, Terrain::CaveFloor);

        let result = try_grid_move(2, 2, 1, 1, &grid, 5, 5, false, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Blocked);
    }

    #[test]
    fn try_grid_move_nowrap_to_ladder_tile() {
        // 梯子はStructureだが、下地のCaveFloorが歩行可能なので移動可能
        let mut grid = create_sized_grid(5, 5, Terrain::CaveWall);
        grid[2][2] = Terrain::CaveFloor;
        grid[2][3] = Terrain::CaveFloor; // 梯子の下地

        let result = try_grid_move(2, 2, 1, 0, &grid, 5, 5, false, |_x, _y, t| t.is_walkable());
        assert_eq!(result, MoveResult::Moved { new_x: 3, new_y: 2 });
    }

    // ============================================
    // is_navigable 述語のテスト
    // ============================================

    #[test]
    fn try_grid_move_navigable_succeeds_on_sea() {
        let grid = create_test_grid(Terrain::Sea);

        let result = try_grid_move(5, 5, 1, 0, &grid, MAP_WIDTH, MAP_HEIGHT, true, |_x, _y, t| t.is_navigable());
        assert_eq!(result, MoveResult::Moved { new_x: 6, new_y: 5 });
    }

    #[test]
    fn try_grid_move_navigable_blocked_by_land() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][6] = Terrain::Plains;

        let result = try_grid_move(5, 5, 1, 0, &grid, MAP_WIDTH, MAP_HEIGHT, true, |_x, _y, t| t.is_navigable());
        assert_eq!(result, MoveResult::Blocked);
    }
}
