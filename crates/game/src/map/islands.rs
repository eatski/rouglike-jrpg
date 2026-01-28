//! 島検出と船スポーン位置の計算

use super::{Terrain, MAP_HEIGHT, MAP_WIDTH};
use rand::Rng;
use std::collections::VecDeque;

/// 船のスポーン情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoatSpawn {
    /// タイル座標
    pub x: usize,
    pub y: usize,
}

/// 島の外郭タイル（海に隣接する陸地）を検出し、その隣の海タイルを船スポーン位置として返す
pub fn calculate_boat_spawns(grid: &[Vec<Terrain>], rng: &mut impl Rng) -> Vec<BoatSpawn> {
    let islands = detect_islands(grid);
    let mut spawns = Vec::new();

    for island in islands {
        if let Some(spawn) = find_boat_spawn_for_island(&island, grid, rng) {
            spawns.push(spawn);
        }
    }

    spawns
}

/// Flood Fillで連結した陸地を島として検出
fn detect_islands(grid: &[Vec<Terrain>]) -> Vec<Vec<(usize, usize)>> {
    let mut visited = vec![vec![false; MAP_WIDTH]; MAP_HEIGHT];
    let mut islands = Vec::new();

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            if !visited[y][x] && grid[y][x] != Terrain::Sea {
                let island = flood_fill(x, y, grid, &mut visited);
                if !island.is_empty() {
                    islands.push(island);
                }
            }
        }
    }

    islands
}

/// Flood Fillで連結した陸地タイルを収集
fn flood_fill(
    start_x: usize,
    start_y: usize,
    grid: &[Vec<Terrain>],
    visited: &mut [Vec<bool>],
) -> Vec<(usize, usize)> {
    let mut island = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back((start_x, start_y));
    visited[start_y][start_x] = true;

    while let Some((x, y)) = queue.pop_front() {
        island.push((x, y));

        // 4近傍を探索
        for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            let nx = (x as i32 + dx).rem_euclid(MAP_WIDTH as i32) as usize;
            let ny = (y as i32 + dy).rem_euclid(MAP_HEIGHT as i32) as usize;

            if !visited[ny][nx] && grid[ny][nx] != Terrain::Sea {
                visited[ny][nx] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    island
}

/// 島の外郭から船スポーン位置を決定
fn find_boat_spawn_for_island(
    island: &[(usize, usize)],
    grid: &[Vec<Terrain>],
    rng: &mut impl Rng,
) -> Option<BoatSpawn> {
    // 外郭タイル（海に隣接する陸地）を収集
    let mut perimeter_tiles = Vec::new();

    for &(x, y) in island {
        if has_adjacent_sea(x, y, grid) {
            perimeter_tiles.push((x, y));
        }
    }

    if perimeter_tiles.is_empty() {
        return None;
    }

    // ランダムに外郭タイルを選択
    let idx = rng.gen_range(0..perimeter_tiles.len());
    let (land_x, land_y) = perimeter_tiles[idx];

    // その外郭タイルに隣接する海タイルを船スポーン位置に
    find_adjacent_sea(land_x, land_y, grid)
}

/// タイルが海に隣接しているかチェック
fn has_adjacent_sea(x: usize, y: usize, grid: &[Vec<Terrain>]) -> bool {
    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        let nx = (x as i32 + dx).rem_euclid(MAP_WIDTH as i32) as usize;
        let ny = (y as i32 + dy).rem_euclid(MAP_HEIGHT as i32) as usize;
        if grid[ny][nx] == Terrain::Sea {
            return true;
        }
    }
    false
}

/// 隣接する海タイルを見つける
fn find_adjacent_sea(x: usize, y: usize, grid: &[Vec<Terrain>]) -> Option<BoatSpawn> {
    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        let nx = (x as i32 + dx).rem_euclid(MAP_WIDTH as i32) as usize;
        let ny = (y as i32 + dy).rem_euclid(MAP_HEIGHT as i32) as usize;
        if grid[ny][nx] == Terrain::Sea {
            return Some(BoatSpawn { x: nx, y: ny });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn create_test_grid(default: Terrain) -> Vec<Vec<Terrain>> {
        vec![vec![default; MAP_WIDTH]; MAP_HEIGHT]
    }

    fn create_rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    #[test]
    fn detect_islands_finds_single_island() {
        let mut grid = create_test_grid(Terrain::Sea);
        // 小さな島を作成
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Plains;
        grid[6][5] = Terrain::Plains;

        let islands = detect_islands(&grid);

        assert_eq!(islands.len(), 1);
        assert_eq!(islands[0].len(), 3);
    }

    #[test]
    fn detect_islands_finds_multiple_islands() {
        let mut grid = create_test_grid(Terrain::Sea);
        // 2つの離れた島を作成
        grid[5][5] = Terrain::Plains;
        grid[20][20] = Terrain::Plains;

        let islands = detect_islands(&grid);

        assert_eq!(islands.len(), 2);
    }

    #[test]
    fn boat_spawn_is_on_sea() {
        let mut grid = create_test_grid(Terrain::Sea);
        // 小さな島を作成
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Plains;
        grid[6][5] = Terrain::Plains;

        let mut rng = create_rng(12345);
        let spawns = calculate_boat_spawns(&grid, &mut rng);

        assert_eq!(spawns.len(), 1);
        let spawn = spawns[0];
        assert_eq!(grid[spawn.y][spawn.x], Terrain::Sea);
    }
}
