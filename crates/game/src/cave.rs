use rand::Rng;
use std::collections::VecDeque;

pub const CAVE_WIDTH: usize = 30;
pub const CAVE_HEIGHT: usize = 30;

const RANDOM_WALK_STEPS: usize = 400;
const MIN_WARP_DISTANCE: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveTerrain {
    Wall,
    Floor,
    WarpZone,
}

impl CaveTerrain {
    #[inline]
    pub fn is_walkable(self) -> bool {
        matches!(self, CaveTerrain::Floor | CaveTerrain::WarpZone)
    }
}

pub struct CaveMapData {
    pub grid: Vec<Vec<CaveTerrain>>,
    pub width: usize,
    pub height: usize,
    pub spawn_position: (usize, usize),
    pub warp_position: (usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveMoveResult {
    Moved { new_x: usize, new_y: usize },
    Blocked,
}

pub fn try_cave_move(
    current_x: usize,
    current_y: usize,
    dx: i32,
    dy: i32,
    grid: &[Vec<CaveTerrain>],
    width: usize,
    height: usize,
) -> CaveMoveResult {
    // 斜め移動は禁止
    if dx != 0 && dy != 0 {
        return CaveMoveResult::Blocked;
    }

    let new_x = current_x as i32 + dx;
    let new_y = current_y as i32 + dy;

    if new_x < 0 || new_x >= width as i32 || new_y < 0 || new_y >= height as i32 {
        return CaveMoveResult::Blocked;
    }

    let new_x = new_x as usize;
    let new_y = new_y as usize;

    if grid[new_y][new_x].is_walkable() {
        CaveMoveResult::Moved { new_x, new_y }
    } else {
        CaveMoveResult::Blocked
    }
}

pub fn generate_cave_map(rng: &mut impl Rng) -> CaveMapData {
    let mut grid = vec![vec![CaveTerrain::Wall; CAVE_WIDTH]; CAVE_HEIGHT];

    // ランダムウォークで通路を掘る
    let mut x = CAVE_WIDTH / 2;
    let mut y = CAVE_HEIGHT / 2;
    grid[y][x] = CaveTerrain::Floor;

    let directions: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    for _ in 0..RANDOM_WALK_STEPS {
        let (dx, dy) = directions[rng.gen_range(0..4)];
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;

        // 外周1マスは壁として残す
        if nx >= 1 && nx < (CAVE_WIDTH as i32 - 1) && ny >= 1 && ny < (CAVE_HEIGHT as i32 - 1) {
            x = nx as usize;
            y = ny as usize;
            grid[y][x] = CaveTerrain::Floor;
        }
    }

    let spawn_position = (CAVE_WIDTH / 2, CAVE_HEIGHT / 2);

    // ワープゾーンをスポーンから離れたFloorに配置
    let warp_position = find_warp_position(&grid, spawn_position, rng);
    grid[warp_position.1][warp_position.0] = CaveTerrain::WarpZone;

    CaveMapData {
        grid,
        width: CAVE_WIDTH,
        height: CAVE_HEIGHT,
        spawn_position,
        warp_position,
    }
}

fn find_warp_position(
    grid: &[Vec<CaveTerrain>],
    spawn: (usize, usize),
    rng: &mut impl Rng,
) -> (usize, usize) {
    // スポーンから到達可能なFloorを収集し、距離が遠い候補からランダムに選ぶ
    let reachable = flood_fill(grid, spawn);

    let mut candidates: Vec<(usize, usize)> = reachable
        .into_iter()
        .filter(|&(x, y)| {
            let dist = ((x as i32 - spawn.0 as i32).unsigned_abs()
                + (y as i32 - spawn.1 as i32).unsigned_abs()) as usize;
            dist >= MIN_WARP_DISTANCE && (x, y) != spawn
        })
        .collect();

    if candidates.is_empty() {
        // 距離条件を緩和
        candidates = flood_fill(grid, spawn)
            .into_iter()
            .filter(|&pos| pos != spawn)
            .collect();
    }

    if candidates.is_empty() {
        // フォールバック: スポーンの隣
        return (spawn.0.min(CAVE_WIDTH - 2) + 1, spawn.1);
    }

    // 最も遠い候補の上位から選ぶ
    candidates.sort_by_key(|&(x, y)| {
        std::cmp::Reverse(
            (x as i32 - spawn.0 as i32).unsigned_abs()
                + (y as i32 - spawn.1 as i32).unsigned_abs(),
        )
    });
    let top = candidates.len().min(5);
    candidates[rng.gen_range(0..top)]
}

fn flood_fill(grid: &[Vec<CaveTerrain>], start: (usize, usize)) -> Vec<(usize, usize)> {
    let height = grid.len();
    let width = grid[0].len();
    let mut visited = vec![vec![false; width]; height];
    let mut result = Vec::new();
    let mut queue = VecDeque::new();

    visited[start.1][start.0] = true;
    queue.push_back(start);

    while let Some((x, y)) = queue.pop_front() {
        if grid[y][x].is_walkable() {
            result.push((x, y));
        }

        for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let nx = nx as usize;
                let ny = ny as usize;
                if !visited[ny][nx] && grid[ny][nx].is_walkable() {
                    visited[ny][nx] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn create_rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    #[test]
    fn generate_cave_map_correct_size() {
        let mut rng = create_rng(42);
        let map = generate_cave_map(&mut rng);
        assert_eq!(map.grid.len(), CAVE_HEIGHT);
        for row in &map.grid {
            assert_eq!(row.len(), CAVE_WIDTH);
        }
        assert_eq!(map.width, CAVE_WIDTH);
        assert_eq!(map.height, CAVE_HEIGHT);
    }

    #[test]
    fn generate_cave_map_spawn_is_floor() {
        let mut rng = create_rng(42);
        let map = generate_cave_map(&mut rng);
        let (sx, sy) = map.spawn_position;
        assert_eq!(map.grid[sy][sx], CaveTerrain::Floor);
    }

    #[test]
    fn generate_cave_map_warp_exists() {
        let mut rng = create_rng(42);
        let map = generate_cave_map(&mut rng);
        let (wx, wy) = map.warp_position;
        assert_eq!(map.grid[wy][wx], CaveTerrain::WarpZone);
    }

    #[test]
    fn generate_cave_map_warp_reachable_from_spawn() {
        let mut rng = create_rng(42);
        let map = generate_cave_map(&mut rng);
        let reachable = flood_fill(&map.grid, map.spawn_position);
        assert!(
            reachable.contains(&map.warp_position),
            "Warp zone should be reachable from spawn"
        );
    }

    #[test]
    fn generate_cave_map_has_floor_tiles() {
        let mut rng = create_rng(42);
        let map = generate_cave_map(&mut rng);
        let floor_count = map
            .grid
            .iter()
            .flatten()
            .filter(|t| **t == CaveTerrain::Floor)
            .count();
        assert!(floor_count > 50, "Cave should have enough floor tiles");
    }

    #[test]
    fn generate_cave_map_edges_are_walls() {
        let mut rng = create_rng(42);
        let map = generate_cave_map(&mut rng);
        // 外周は壁（ワープゾーンは外周に置かれない）
        for x in 0..CAVE_WIDTH {
            assert_ne!(
                map.grid[0][x],
                CaveTerrain::Floor,
                "Top edge should not be floor"
            );
            assert_ne!(
                map.grid[CAVE_HEIGHT - 1][x],
                CaveTerrain::Floor,
                "Bottom edge should not be floor"
            );
        }
        for y in 0..CAVE_HEIGHT {
            assert_ne!(
                map.grid[y][0],
                CaveTerrain::Floor,
                "Left edge should not be floor"
            );
            assert_ne!(
                map.grid[y][CAVE_WIDTH - 1],
                CaveTerrain::Floor,
                "Right edge should not be floor"
            );
        }
    }

    #[test]
    fn generate_cave_map_deterministic() {
        let mut rng1 = create_rng(123);
        let mut rng2 = create_rng(123);
        let map1 = generate_cave_map(&mut rng1);
        let map2 = generate_cave_map(&mut rng2);
        assert_eq!(map1.spawn_position, map2.spawn_position);
        assert_eq!(map1.warp_position, map2.warp_position);
        assert_eq!(map1.grid, map2.grid);
    }

    #[test]
    fn try_cave_move_normal() {
        let mut grid = vec![vec![CaveTerrain::Wall; 5]; 5];
        grid[2][2] = CaveTerrain::Floor;
        grid[2][3] = CaveTerrain::Floor;

        let result = try_cave_move(2, 2, 1, 0, &grid, 5, 5);
        assert_eq!(result, CaveMoveResult::Moved { new_x: 3, new_y: 2 });
    }

    #[test]
    fn try_cave_move_blocked_by_wall() {
        let mut grid = vec![vec![CaveTerrain::Wall; 5]; 5];
        grid[2][2] = CaveTerrain::Floor;

        let result = try_cave_move(2, 2, 1, 0, &grid, 5, 5);
        assert_eq!(result, CaveMoveResult::Blocked);
    }

    #[test]
    fn try_cave_move_blocked_by_boundary() {
        let grid = vec![vec![CaveTerrain::Floor; 5]; 5];

        let result = try_cave_move(0, 0, -1, 0, &grid, 5, 5);
        assert_eq!(result, CaveMoveResult::Blocked);
    }

    #[test]
    fn try_cave_move_diagonal_blocked() {
        let grid = vec![vec![CaveTerrain::Floor; 5]; 5];

        let result = try_cave_move(2, 2, 1, 1, &grid, 5, 5);
        assert_eq!(result, CaveMoveResult::Blocked);
    }

    #[test]
    fn try_cave_move_to_warp_zone() {
        let mut grid = vec![vec![CaveTerrain::Wall; 5]; 5];
        grid[2][2] = CaveTerrain::Floor;
        grid[2][3] = CaveTerrain::WarpZone;

        let result = try_cave_move(2, 2, 1, 0, &grid, 5, 5);
        assert_eq!(result, CaveMoveResult::Moved { new_x: 3, new_y: 2 });
    }

    #[test]
    fn cave_terrain_walkability() {
        assert!(!CaveTerrain::Wall.is_walkable());
        assert!(CaveTerrain::Floor.is_walkable());
        assert!(CaveTerrain::WarpZone.is_walkable());
    }
}
