use terrain::{try_grid_move, MoveResult, Structure, Terrain};

/// フィールド移動判定の結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldMoveResult {
    /// 徒歩で移動した
    Walked { new_x: usize, new_y: usize },
    /// 船で航行した
    Sailed { new_x: usize, new_y: usize },
    /// 船から下船した
    Disembarked { new_x: usize, new_y: usize },
    /// 移動がブロックされた
    Blocked,
}

/// フィールド移動の判定を行う純粋関数
///
/// - `on_boat=true`: 海上移動を試行し、失敗なら下船（徒歩移動）を試行
/// - `on_boat=false`: 徒歩移動のみ試行
#[allow(clippy::too_many_arguments)]
pub fn resolve_field_move(
    grid: &[Vec<Terrain>],
    structures: &[Vec<Structure>],
    width: usize,
    height: usize,
    wraps: bool,
    x: usize,
    y: usize,
    dx: i32,
    dy: i32,
    on_boat: bool,
) -> FieldMoveResult {
    if on_boat {
        // 船モード: まず海上移動を試行
        match try_grid_move(x, y, dx, dy, grid, width, height, wraps, |_nx, _ny, t| {
            t.is_navigable()
        }) {
            MoveResult::Moved { new_x, new_y } => FieldMoveResult::Sailed { new_x, new_y },
            MoveResult::Blocked => {
                // 下船を試行（陸地への移動）
                match try_walkable_move(grid, structures, width, height, wraps, x, y, dx, dy) {
                    MoveResult::Moved { new_x, new_y } => {
                        FieldMoveResult::Disembarked { new_x, new_y }
                    }
                    MoveResult::Blocked => FieldMoveResult::Blocked,
                }
            }
        }
    } else {
        // 徒歩移動
        match try_walkable_move(grid, structures, width, height, wraps, x, y, dx, dy) {
            MoveResult::Moved { new_x, new_y } => FieldMoveResult::Walked { new_x, new_y },
            MoveResult::Blocked => FieldMoveResult::Blocked,
        }
    }
}

/// 徒歩移動の試行（構造物があれば通行可能）
#[allow(clippy::too_many_arguments)]
fn try_walkable_move(
    grid: &[Vec<Terrain>],
    structures: &[Vec<Structure>],
    width: usize,
    height: usize,
    wraps: bool,
    x: usize,
    y: usize,
    dx: i32,
    dy: i32,
) -> MoveResult {
    try_grid_move(x, y, dx, dy, grid, width, height, wraps, |nx, ny, t| {
        structures[ny][nx] != Structure::None || t.is_walkable()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use terrain::test_utils::create_test_grid;
    use terrain::{MAP_HEIGHT, MAP_WIDTH};

    fn test_structures() -> Vec<Vec<Structure>> {
        vec![vec![Structure::None; MAP_WIDTH]; MAP_HEIGHT]
    }

    // ============================================
    // 徒歩移動 (on_boat=false)
    // ============================================

    #[test]
    fn walk_on_plains() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Plains;
        let structures = test_structures();

        let result = resolve_field_move(
            &grid, &structures, MAP_WIDTH, MAP_HEIGHT, true, 5, 5, 1, 0, false,
        );
        assert_eq!(result, FieldMoveResult::Walked { new_x: 6, new_y: 5 });
    }

    #[test]
    fn walk_blocked_by_sea() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        let structures = test_structures();

        let result = resolve_field_move(
            &grid, &structures, MAP_WIDTH, MAP_HEIGHT, true, 5, 5, 1, 0, false,
        );
        assert_eq!(result, FieldMoveResult::Blocked);
    }

    #[test]
    fn walk_to_structure() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        // 構造物があればSeaでも通行可能
        let mut structures = test_structures();
        structures[5][6] = Structure::Town;

        let result = resolve_field_move(
            &grid, &structures, MAP_WIDTH, MAP_HEIGHT, true, 5, 5, 1, 0, false,
        );
        assert_eq!(result, FieldMoveResult::Walked { new_x: 6, new_y: 5 });
    }

    // ============================================
    // 船移動 (on_boat=true)
    // ============================================

    #[test]
    fn sail_on_sea() {
        let grid = create_test_grid(Terrain::Sea);
        let structures = test_structures();

        let result = resolve_field_move(
            &grid, &structures, MAP_WIDTH, MAP_HEIGHT, true, 5, 5, 1, 0, true,
        );
        assert_eq!(result, FieldMoveResult::Sailed { new_x: 6, new_y: 5 });
    }

    #[test]
    fn disembark_to_land() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][6] = Terrain::Plains;
        let structures = test_structures();

        let result = resolve_field_move(
            &grid, &structures, MAP_WIDTH, MAP_HEIGHT, true, 5, 5, 1, 0, true,
        );
        assert_eq!(
            result,
            FieldMoveResult::Disembarked { new_x: 6, new_y: 5 }
        );
    }

    #[test]
    fn sail_blocked_by_mountain() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][6] = Terrain::Mountain;
        let structures = test_structures();

        let result = resolve_field_move(
            &grid, &structures, MAP_WIDTH, MAP_HEIGHT, true, 5, 5, 1, 0, true,
        );
        assert_eq!(result, FieldMoveResult::Blocked);
    }

    #[test]
    fn disembark_to_structure() {
        let grid = create_test_grid(Terrain::Sea);
        // Sea + Structure → 下船可能
        let mut structures = test_structures();
        structures[5][6] = Structure::Town;

        let result = resolve_field_move(
            &grid, &structures, MAP_WIDTH, MAP_HEIGHT, true, 5, 5, 1, 0, true,
        );
        // Sea is navigable, so this should be Sailed (navigable check passes first)
        assert_eq!(result, FieldMoveResult::Sailed { new_x: 6, new_y: 5 });
    }

    #[test]
    fn disembark_to_land_structure() {
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][6] = Terrain::Plains;
        let mut structures = test_structures();
        structures[5][6] = Structure::Town;

        let result = resolve_field_move(
            &grid, &structures, MAP_WIDTH, MAP_HEIGHT, true, 5, 5, 1, 0, true,
        );
        // Plains is not navigable → try disembark → Plains + Structure is walkable
        assert_eq!(
            result,
            FieldMoveResult::Disembarked { new_x: 6, new_y: 5 }
        );
    }

    // ============================================
    // wraps=false (洞窟など)
    // ============================================

    #[test]
    fn walk_nowrap_normal() {
        let grid = vec![vec![Terrain::CaveFloor; 5]; 5];
        let structures = vec![vec![Structure::None; 5]; 5];

        let result =
            resolve_field_move(&grid, &structures, 5, 5, false, 2, 2, 1, 0, false);
        assert_eq!(result, FieldMoveResult::Walked { new_x: 3, new_y: 2 });
    }

    #[test]
    fn walk_nowrap_blocked_by_boundary() {
        let grid = vec![vec![Terrain::CaveFloor; 5]; 5];
        let structures = vec![vec![Structure::None; 5]; 5];

        let result =
            resolve_field_move(&grid, &structures, 5, 5, false, 0, 0, -1, 0, false);
        assert_eq!(result, FieldMoveResult::Blocked);
    }
}
