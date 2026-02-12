use rand::Rng;

use crate::coordinates::orthogonal_neighbors;

use super::islands::{calculate_cave_spawns, calculate_town_spawns, validate_connectivity};
use super::terrain::Terrain;

pub const MAP_WIDTH: usize = 150;
pub const MAP_HEIGHT: usize = 150;
const TARGET_LAND_TILES: usize = 6000;
const LAND_SPREAD_CHANCE: f32 = 0.65;
const ISLAND_COUNT: usize = 20;

pub struct MapData {
    pub grid: Vec<Vec<Terrain>>,
    pub spawn_position: (usize, usize),
}

pub fn generate_map(rng: &mut impl Rng) -> MapData {
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let mut land_tiles = 0usize;
    let mut frontier = Vec::new();
    let mut land_positions = Vec::new();
    let mut seeds = Vec::with_capacity(ISLAND_COUNT);

    while seeds.len() < ISLAND_COUNT {
        let y = rng.gen_range(0..MAP_HEIGHT);
        let x = rng.gen_range(0..MAP_WIDTH);

        if grid[y][x] == Terrain::Sea {
            let pos = (y, x);
            grid[y][x] = Terrain::Plains;
            seeds.push(pos);
            frontier.push(pos);
            land_positions.push(pos);
            land_tiles += 1;
        }
    }

    let protected = seeds[0];
    let spawn_position = (protected.1, protected.0); // (x, y)に変換
    let target_land = TARGET_LAND_TILES.min(MAP_WIDTH * MAP_HEIGHT);

    while land_tiles < target_land {
        if frontier.is_empty() {
            let seed = land_positions[rng.gen_range(0..land_positions.len())];
            frontier.push(seed);
        }

        let idx = rng.gen_range(0..frontier.len());
        let (y, x) = frontier[idx];
        let mut removed = true;

        for (ny, nx) in neighbors_yx(y, x) {
            if grid[ny][nx] == Terrain::Sea && rng.r#gen::<f32>() < LAND_SPREAD_CHANCE {
                grid[ny][nx] = Terrain::Plains;
                land_tiles += 1;
                frontier.push((ny, nx));
                land_positions.push((ny, nx));
                removed = false;

                if land_tiles >= target_land {
                    break;
                }
            }
        }

        if removed || rng.gen_bool(0.35) {
            frontier.swap_remove(idx);
        }
    }

    scatter_clusters(
        &mut grid,
        rng,
        &land_positions,
        Terrain::Forest,
        35,
        20..=80,
        protected,
    );
    scatter_clusters(
        &mut grid,
        rng,
        &land_positions,
        Terrain::Mountain,
        18,
        10..=45,
        protected,
    );

    // 各島に町を1つ配置
    let town_spawns = calculate_town_spawns(&grid, rng, spawn_position);
    for ts in &town_spawns {
        grid[ts.y][ts.x] = Terrain::Town;
    }

    // テスト用: スポーン位置近くに町を強制配置
    let (spawn_x, spawn_y) = spawn_position;
    for dy in 0..15 {
        for dx in 0..15 {
            let y = if spawn_y + dy < MAP_HEIGHT {
                spawn_y + dy
            } else {
                continue;
            };
            let x = if spawn_x + dx < MAP_WIDTH {
                spawn_x + dx
            } else {
                continue;
            };

            if grid[y][x] == Terrain::Plains && (x, y) != spawn_position {
                grid[y][x] = Terrain::Town;
                // 1つ配置したら終了
                break;
            }
        }
        // 既に配置済みなら外側のループも抜ける
        if (0..15).any(|dx| {
            let y = if spawn_y + dy < MAP_HEIGHT {
                spawn_y + dy
            } else {
                return false;
            };
            let x = if spawn_x + dx < MAP_WIDTH {
                spawn_x + dx
            } else {
                return false;
            };
            grid[y][x] == Terrain::Town && (x, y) != spawn_position
        }) {
            break;
        }
    }

    // 各島に洞窟を1つ配置（山タイルから選択）
    let cave_spawns = calculate_cave_spawns(&grid, rng);
    for cs in &cave_spawns {
        grid[cs.y][cs.x] = Terrain::Cave;
    }

    MapData {
        grid,
        spawn_position,
    }
}

const MAX_RETRY: usize = 10;

/// 接続性を保証したマップを生成する
///
/// `generate_map` で生成後、全島が最大海域に隣接するかを検証し、
/// 失敗した場合は再生成する（最大10回）。
pub fn generate_connected_map(rng: &mut impl Rng) -> MapData {
    let mut map = generate_map(rng);

    for _ in 1..MAX_RETRY {
        if validate_connectivity(&map.grid) {
            return map;
        }
        map = generate_map(rng);
    }

    map
}

fn scatter_clusters(
    grid: &mut [Vec<Terrain>],
    rng: &mut impl Rng,
    seeds: &[(usize, usize)],
    terrain: Terrain,
    cluster_count: usize,
    size_range: std::ops::RangeInclusive<usize>,
    protected: (usize, usize),
) {
    if seeds.is_empty() {
        return;
    }

    for _ in 0..cluster_count {
        let (mut y, mut x) = seeds[rng.gen_range(0..seeds.len())];
        let mut stack = vec![(y, x)];
        let mut remaining = rng.gen_range(size_range.clone());

        while remaining > 0 && !stack.is_empty() {
            let idx = rng.gen_range(0..stack.len());
            (y, x) = stack.swap_remove(idx);

            if (y, x) == protected || grid[y][x] != Terrain::Plains {
                continue;
            }

            grid[y][x] = terrain;
            remaining -= 1;

            for (ny, nx) in neighbors_yx(y, x) {
                if grid[ny][nx] == Terrain::Plains && rng.gen_bool(0.7) {
                    stack.push((ny, nx));
                }
            }
        }
    }
}

/// `orthogonal_neighbors` を (y, x) 順序で返すラッパー
///
/// generation モジュール内部は (y, x) 順序を使用するため、
/// (x, y) 順序の `orthogonal_neighbors` の結果を変換する。
fn neighbors_yx(y: usize, x: usize) -> [(usize, usize); 4] {
    let xy_neighbors = orthogonal_neighbors(x, y);
    [
        (xy_neighbors[0].1, xy_neighbors[0].0),
        (xy_neighbors[1].1, xy_neighbors[1].0),
        (xy_neighbors[2].1, xy_neighbors[2].0),
        (xy_neighbors[3].1, xy_neighbors[3].0),
    ]
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
    fn generate_map_creates_correct_size_grid() {
        let mut rng = create_rng(12345);
        let map = generate_map(&mut rng);

        assert_eq!(map.grid.len(), MAP_HEIGHT);
        for row in &map.grid {
            assert_eq!(row.len(), MAP_WIDTH);
        }
    }

    #[test]
    fn generate_map_has_landmass() {
        let mut rng = create_rng(12345);
        let map = generate_map(&mut rng);

        let land_count = map
            .grid
            .iter()
            .flatten()
            .filter(|t| **t != Terrain::Sea)
            .count();

        assert!(land_count > 0, "Map should have some land");
    }

    #[test]
    fn generate_map_spawn_position_is_on_land() {
        let mut rng = create_rng(12345);
        let map = generate_map(&mut rng);

        let (x, y) = map.spawn_position;
        let terrain = map.grid[y][x];

        assert_ne!(
            terrain,
            Terrain::Sea,
            "Spawn position should be on land, not sea"
        );
    }

    #[test]
    fn generate_map_spawn_position_is_plains() {
        let mut rng = create_rng(12345);
        let map = generate_map(&mut rng);

        let (x, y) = map.spawn_position;
        let terrain = map.grid[y][x];

        assert_eq!(
            terrain,
            Terrain::Plains,
            "Spawn position should be on Plains (protected from forest/mountain scatter)"
        );
    }

    #[test]
    fn generate_map_has_forests_and_mountains() {
        let mut rng = create_rng(12345);
        let map = generate_map(&mut rng);

        let forest_count = map
            .grid
            .iter()
            .flatten()
            .filter(|t| **t == Terrain::Forest)
            .count();

        let mountain_count = map
            .grid
            .iter()
            .flatten()
            .filter(|t| **t == Terrain::Mountain)
            .count();

        assert!(forest_count > 0, "Map should have some forests");
        assert!(mountain_count > 0, "Map should have some mountains");
    }

    #[test]
    fn generate_map_is_deterministic_with_same_seed() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let map1 = generate_map(&mut rng1);
        let map2 = generate_map(&mut rng2);

        assert_eq!(map1.spawn_position, map2.spawn_position);
        assert_eq!(map1.grid, map2.grid);
    }

    #[test]
    fn generate_map_different_seeds_produce_different_maps() {
        let mut rng1 = create_rng(1);
        let mut rng2 = create_rng(2);

        let map1 = generate_map(&mut rng1);
        let map2 = generate_map(&mut rng2);

        // 異なるシードでは異なるマップが生成されるはず
        // spawn_positionかgridのどちらかが異なれば良い
        let is_different =
            map1.spawn_position != map2.spawn_position || map1.grid != map2.grid;

        assert!(is_different, "Different seeds should produce different maps");
    }

    #[test]
    fn spawn_position_is_within_map_bounds() {
        let mut rng = create_rng(99999);
        let map = generate_map(&mut rng);

        let (x, y) = map.spawn_position;

        assert!(x < MAP_WIDTH, "Spawn x should be within map width");
        assert!(y < MAP_HEIGHT, "Spawn y should be within map height");
    }

    #[test]
    fn generate_connected_map_produces_valid_map() {
        use crate::map::islands::validate_connectivity;

        // 複数シードで接続性を満たすことを確認
        for seed in [42, 123, 456, 789, 9999] {
            let mut rng = create_rng(seed);
            let map = generate_connected_map(&mut rng);

            assert!(
                validate_connectivity(&map.grid),
                "seed {} should produce a connected map",
                seed
            );
        }
    }
}
