//! 島検出と船スポーン位置の計算

use super::Terrain;
use crate::coordinates::orthogonal_neighbors;
use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use rand::Rng;
use std::collections::{HashSet, VecDeque};

/// 船のスポーン情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoatSpawn {
    /// タイル座標
    pub x: usize,
    pub y: usize,
}

/// 島の外郭タイル（海に隣接する陸地）を検出し、最大海域に面した海タイルを船スポーン位置として返す
pub fn calculate_boat_spawns(grid: &[Vec<Terrain>], rng: &mut impl Rng) -> Vec<BoatSpawn> {
    let islands = detect_islands(grid);
    let sea_regions = detect_sea_regions(grid);
    let main_sea: HashSet<(usize, usize)> = if sea_regions.is_empty() {
        HashSet::new()
    } else {
        sea_regions[0].iter().copied().collect()
    };

    let mut spawns = Vec::new();

    for island in islands {
        if let Some(spawn) = find_boat_spawn_on_main_sea(&island, grid, &main_sea, rng) {
            spawns.push(spawn);
        }
    }

    spawns
}

/// Flood Fillで連結した陸地を島として検出
pub fn detect_islands(grid: &[Vec<Terrain>]) -> Vec<Vec<(usize, usize)>> {
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
        for (nx, ny) in orthogonal_neighbors(x, y) {
            if !visited[ny][nx] && grid[ny][nx] != Terrain::Sea {
                visited[ny][nx] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    island
}

/// Flood Fillで連結した海タイルを海域として検出（サイズ降順ソート）
pub fn detect_sea_regions(grid: &[Vec<Terrain>]) -> Vec<Vec<(usize, usize)>> {
    let mut visited = vec![vec![false; MAP_WIDTH]; MAP_HEIGHT];
    let mut regions = Vec::new();

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            if !visited[y][x] && grid[y][x] == Terrain::Sea {
                let region = flood_fill_sea(x, y, grid, &mut visited);
                if !region.is_empty() {
                    regions.push(region);
                }
            }
        }
    }

    // サイズ降順ソート（[0]が最大海域）
    regions.sort_by_key(|r| std::cmp::Reverse(r.len()));
    regions
}

/// 全ての島が最大海域に隣接するか検証
pub fn validate_connectivity(grid: &[Vec<Terrain>]) -> bool {
    let islands = detect_islands(grid);
    let sea_regions = detect_sea_regions(grid);

    if islands.is_empty() || sea_regions.is_empty() {
        return islands.is_empty();
    }

    let main_sea: HashSet<(usize, usize)> = sea_regions[0].iter().copied().collect();

    for island in &islands {
        let touches_main_sea = island.iter().any(|&(x, y)| {
            orthogonal_neighbors(x, y)
                .iter()
                .any(|&(nx, ny)| main_sea.contains(&(nx, ny)))
        });
        if !touches_main_sea {
            return false;
        }
    }

    true
}

/// Flood Fillで連結した海タイルを収集
fn flood_fill_sea(
    start_x: usize,
    start_y: usize,
    grid: &[Vec<Terrain>],
    visited: &mut [Vec<bool>],
) -> Vec<(usize, usize)> {
    let mut region = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back((start_x, start_y));
    visited[start_y][start_x] = true;

    while let Some((x, y)) = queue.pop_front() {
        region.push((x, y));

        for (nx, ny) in orthogonal_neighbors(x, y) {
            if !visited[ny][nx] && grid[ny][nx] == Terrain::Sea {
                visited[ny][nx] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    region
}

/// 最大海域に面した外縁から船スポーン位置を決定
fn find_boat_spawn_on_main_sea(
    island: &[(usize, usize)],
    grid: &[Vec<Terrain>],
    main_sea: &HashSet<(usize, usize)>,
    rng: &mut impl Rng,
) -> Option<BoatSpawn> {
    // 最大海域に隣接する陸地タイルとその海タイルを収集
    let mut candidates: Vec<(usize, usize)> = Vec::new();

    for &(x, y) in island {
        for (nx, ny) in orthogonal_neighbors(x, y) {
            if grid[ny][nx] == Terrain::Sea && main_sea.contains(&(nx, ny)) {
                candidates.push((nx, ny));
            }
        }
    }

    candidates.dedup();

    if candidates.is_empty() {
        return None;
    }

    let idx = rng.gen_range(0..candidates.len());
    let (sx, sy) = candidates[idx];
    Some(BoatSpawn { x: sx, y: sy })
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_grid;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

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

    #[test]
    fn detect_sea_regions_single_ocean() {
        // 全て海のグリッドでは海域は1つ
        let grid = create_test_grid(Terrain::Sea);
        let regions = detect_sea_regions(&grid);

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].len(), MAP_WIDTH * MAP_HEIGHT);
    }

    #[test]
    fn detect_sea_regions_split_ocean() {
        // 2本の縦壁で海を分断するケース
        let mut grid = create_test_grid(Terrain::Sea);
        for y in 0..MAP_HEIGHT {
            grid[y][50] = Terrain::Plains;
            grid[y][100] = Terrain::Plains;
        }

        let regions = detect_sea_regions(&grid);

        // 2本の壁で海域が2つに分断される
        assert_eq!(regions.len(), 2);
    }

    #[test]
    fn validate_connectivity_connected() {
        // 全島が同一海域に接する正常ケース
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[20][20] = Terrain::Plains;

        assert!(validate_connectivity(&grid));
    }

    #[test]
    fn validate_connectivity_disconnected() {
        // 孤立島のケース：内陸湖に囲まれた島
        let mut grid = create_test_grid(Terrain::Sea);

        // 大きな陸地ブロックの中に海の「湖」を作り、その中に島を置く
        for y in 10..30 {
            for x in 10..30 {
                grid[y][x] = Terrain::Plains;
            }
        }
        // 陸地ブロック内部に海の湖
        for y in 15..25 {
            for x in 15..25 {
                grid[y][x] = Terrain::Sea;
            }
        }
        // 湖の中に孤立島
        grid[20][20] = Terrain::Plains;

        // 外側の大きな海と湖は陸地で分断されているので
        // 孤立島は最大海域（外側の海）に隣接しない
        assert!(!validate_connectivity(&grid));
    }

    #[test]
    fn boat_spawns_on_main_sea() {
        // 船が最大海域にスポーンすることを確認
        let mut grid = create_test_grid(Terrain::Sea);
        grid[5][5] = Terrain::Plains;
        grid[5][6] = Terrain::Plains;

        let mut rng = create_rng(42);
        let spawns = calculate_boat_spawns(&grid, &mut rng);

        let sea_regions = detect_sea_regions(&grid);
        let main_sea: HashSet<(usize, usize)> = sea_regions[0].iter().copied().collect();

        for spawn in &spawns {
            assert!(
                main_sea.contains(&(spawn.x, spawn.y)),
                "Boat spawn ({}, {}) should be on the main sea",
                spawn.x,
                spawn.y
            );
        }
    }
}
