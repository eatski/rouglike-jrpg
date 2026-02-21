use rand::Rng;

use terrain::{Terrain, MAP_HEIGHT, MAP_WIDTH};

use std::collections::{HashSet, VecDeque};

use crate::coordinates::{orthogonal_neighbors, wrap_position};
use crate::map::islands::{
    calculate_cave_spawns, calculate_hokora_spawns, calculate_town_spawns, detect_islands,
    validate_connectivity,
};

/// 大大陸の目標タイル数（大陸1,2用。侵食・湖で減るため多めに生成）
const LARGE_CONTINENT_TARGET: usize = 3500;
/// 小大陸の目標タイル数（大陸3,4,5用。侵食・湖で減るため多めに生成）
const SMALL_CONTINENT_TARGET: usize = 2000;
/// 大陸間の最小トーラス距離
const MIN_CONTINENT_DISTANCE: f64 = 30.0;
/// 大陸成長時の拡散確率
const SPREAD_CHANCE: f64 = 0.65;
/// 大陸間の境界バッファ（この距離差以内は海のまま残す）
const CONTINENT_BORDER_GAP: f64 = 4.0;
/// 大陸数
const NUM_CONTINENTS: usize = 5;
/// 森林クラスタ数
const FOREST_CLUSTERS: usize = 45;
/// 山岳クラスタ数
const MOUNTAIN_CLUSTERS: usize = 60;
/// 森林クラスタの最小タイル数
const FOREST_CLUSTER_MIN: usize = 20;
/// 森林クラスタの最大タイル数
const FOREST_CLUSTER_MAX: usize = 80;
/// 山岳クラスタの最小タイル数
const MOUNTAIN_CLUSTER_MIN: usize = 15;
/// 山岳クラスタの最大タイル数
const MOUNTAIN_CLUSTER_MAX: usize = 50;

pub struct MapData {
    pub grid: Vec<Vec<Terrain>>,
    pub spawn_position: (usize, usize),
}

/// トーラス距離を計算
fn torus_distance(x1: usize, y1: usize, x2: usize, y2: usize) -> f64 {
    let dx = (x1 as f64 - x2 as f64)
        .abs()
        .min(MAP_WIDTH as f64 - (x1 as f64 - x2 as f64).abs());
    let dy = (y1 as f64 - y2 as f64)
        .abs()
        .min(MAP_HEIGHT as f64 - (y1 as f64 - y2 as f64).abs());
    (dx * dx + dy * dy).sqrt()
}

/// 指定座標が最も近い大陸中心点のインデックスを返す
fn closest_center_index(x: usize, y: usize, centers: &[(usize, usize)]) -> usize {
    centers
        .iter()
        .enumerate()
        .min_by(|(_, c1), (_, c2)| {
            let d1 = torus_distance(x, y, c1.0, c1.1);
            let d2 = torus_distance(x, y, c2.0, c2.1);
            d1.partial_cmp(&d2).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Phase 1: 大陸中心点を配置する
///
/// メイン大陸はマップ中央付近、残りはトーラス距離MIN_CONTINENT_DISTANCE以上離して配置。
fn place_continent_centers(rng: &mut impl Rng) -> Vec<(usize, usize)> {
    let mut centers: Vec<(usize, usize)> = Vec::with_capacity(NUM_CONTINENTS);

    // メイン大陸: 中央付近
    let main_x = 75usize.saturating_add(rng.gen_range(0..31)).min(MAP_WIDTH - 1);
    let main_y = 75usize.saturating_add(rng.gen_range(0..31)).min(MAP_HEIGHT - 1);
    // -15〜+15 のオフセット
    let main_x = if rng.gen_bool(0.5) {
        main_x.saturating_sub(rng.gen_range(0..16))
    } else {
        (main_x + rng.gen_range(0..16)).min(MAP_WIDTH - 1)
    };
    let main_y = if rng.gen_bool(0.5) {
        main_y.saturating_sub(rng.gen_range(0..16))
    } else {
        (main_y + rng.gen_range(0..16)).min(MAP_HEIGHT - 1)
    };
    centers.push((main_x, main_y));

    // 残り4大陸
    let mut min_distance = MIN_CONTINENT_DISTANCE;
    for _ in 1..NUM_CONTINENTS {
        let mut placed = false;
        for attempt in 0..1000 {
            // 1000回試行後に距離を緩和
            if attempt == 1000 {
                min_distance = (min_distance * 0.8).max(10.0);
            }
            let cx = rng.gen_range(0..MAP_WIDTH);
            let cy = rng.gen_range(0..MAP_HEIGHT);

            let far_enough = centers
                .iter()
                .all(|&(ox, oy)| torus_distance(cx, cy, ox, oy) >= min_distance);

            if far_enough {
                centers.push((cx, cy));
                placed = true;
                break;
            }
        }
        // 1000回試行しても配置できなかった場合は距離を無視して配置
        if !placed {
            let cx = rng.gen_range(0..MAP_WIDTH);
            let cy = rng.gen_range(0..MAP_HEIGHT);
            centers.push((cx, cy));
        }
    }

    centers
}

/// Phase 2: 大陸を成長させる（Random Growth / Frontier拡散法）
fn grow_continents(
    grid: &mut Vec<Vec<Terrain>>,
    centers: &[(usize, usize)],
    rng: &mut impl Rng,
) {
    let targets = [
        LARGE_CONTINENT_TARGET,
        LARGE_CONTINENT_TARGET,
        SMALL_CONTINENT_TARGET,
        SMALL_CONTINENT_TARGET,
        SMALL_CONTINENT_TARGET,
    ];

    // 各大陸のフロンティアと陸地カウント
    let mut frontiers: Vec<Vec<(usize, usize)>> = centers
        .iter()
        .map(|&(cx, cy)| {
            grid[cy][cx] = Terrain::Plains;
            vec![(cx, cy)]
        })
        .collect();

    let mut land_counts: Vec<usize> = vec![1; NUM_CONTINENTS];

    // 全大陸が目標に達するまで、または全フロンティアが空になるまでループ
    loop {
        let all_done = (0..NUM_CONTINENTS).all(|i| {
            land_counts[i] >= targets[i] || frontiers[i].is_empty()
        });
        if all_done {
            break;
        }

        for continent_idx in 0..NUM_CONTINENTS {
            if land_counts[continent_idx] >= targets[continent_idx]
                || frontiers[continent_idx].is_empty()
            {
                continue;
            }

            let frontier = &mut frontiers[continent_idx];
            let idx = rng.gen_range(0..frontier.len());
            let (x, y) = frontier[idx];

            let mut expanded = false;
            let neighbors = orthogonal_neighbors(x, y);

            for (nx, ny) in neighbors {
                if grid[ny][nx] != Terrain::Sea {
                    continue;
                }
                if rng.gen_bool(1.0 - SPREAD_CHANCE) {
                    continue;
                }
                // 侵食防止チェック: 拡散先が自分の大陸に最も近いか
                if closest_center_index(nx, ny, centers) != continent_idx {
                    continue;
                }
                // 境界バッファ: 他の大陸中心との距離差が小さい場所は海のまま残す
                let dist_own =
                    torus_distance(nx, ny, centers[continent_idx].0, centers[continent_idx].1);
                let dist_nearest_other = centers
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != continent_idx)
                    .map(|(_, &(cx, cy))| torus_distance(nx, ny, cx, cy))
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(f64::MAX);
                if dist_nearest_other - dist_own < CONTINENT_BORDER_GAP {
                    continue;
                }
                grid[ny][nx] = Terrain::Plains;
                frontiers[continent_idx].push((nx, ny));
                land_counts[continent_idx] += 1;
                expanded = true;

                if land_counts[continent_idx] >= targets[continent_idx] {
                    break;
                }
            }

            if !expanded || rng.gen_bool(0.1) {
                frontiers[continent_idx].swap_remove(idx);
            }
        }
    }
}

/// Phase 2.5a: 海岸線を侵食して入り江・湾を作る
///
/// 海に隣接する陸地タイルを確率的に海に戻す。
/// 2パス実行することで、より複雑な海岸線を生成する。
fn erode_coastline(
    grid: &mut Vec<Vec<Terrain>>,
    spawn_position: (usize, usize),
    rng: &mut impl Rng,
) {
    for _ in 0..2 {
        let coastal: Vec<(usize, usize)> = (0..MAP_HEIGHT)
            .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                if grid[y][x] == Terrain::Sea || (x, y) == spawn_position {
                    return false;
                }
                // 隣接に海が2つ以上あるタイルを優先的に侵食
                orthogonal_neighbors(x, y)
                    .iter()
                    .any(|&(nx, ny)| grid[ny][nx] == Terrain::Sea)
            })
            .collect();

        for &(x, y) in &coastal {
            if (x, y) == spawn_position {
                continue;
            }
            let sea_neighbors = orthogonal_neighbors(x, y)
                .iter()
                .filter(|&&(nx, ny)| grid[ny][nx] == Terrain::Sea)
                .count();
            // 海に面する数が多いほど侵食されやすい
            let erode_chance = match sea_neighbors {
                3..=4 => 0.6,
                2 => 0.3,
                _ => 0.12,
            };
            if rng.gen_bool(erode_chance) {
                grid[y][x] = Terrain::Sea;
            }
        }
    }
}

/// Phase 2.5b: 大陸内部に湖を配置する
///
/// 内陸（海に隣接しない）の陸地にクラスタ状の湖を生成する。
fn place_lakes(
    grid: &mut Vec<Vec<Terrain>>,
    spawn_position: (usize, usize),
    rng: &mut impl Rng,
) {
    const LAKE_CLUSTERS: usize = 60;
    const LAKE_MIN_SIZE: usize = 8;
    const LAKE_MAX_SIZE: usize = 60;

    // 内陸タイルを収集（海に隣接していない陸地）
    let inland: Vec<(usize, usize)> = (0..MAP_HEIGHT)
        .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            if grid[y][x] == Terrain::Sea || (x, y) == spawn_position {
                return false;
            }
            // 全隣接が陸地（= 内陸部）
            orthogonal_neighbors(x, y)
                .iter()
                .all(|&(nx, ny)| grid[ny][nx] != Terrain::Sea)
        })
        .collect();

    if inland.is_empty() {
        return;
    }

    for _ in 0..LAKE_CLUSTERS {
        let lake_size = rng.gen_range(LAKE_MIN_SIZE..=LAKE_MAX_SIZE);
        let start_idx = rng.gen_range(0..inland.len());
        let (sx, sy) = inland[start_idx];

        if grid[sy][sx] == Terrain::Sea {
            continue;
        }

        let mut frontier = vec![(sx, sy)];
        let mut placed = 0usize;

        while placed < lake_size && !frontier.is_empty() {
            let idx = rng.gen_range(0..frontier.len());
            let (cx, cy) = frontier[idx];

            if grid[cy][cx] != Terrain::Sea && (cx, cy) != spawn_position {
                grid[cy][cx] = Terrain::Sea;
                placed += 1;

                for (nx, ny) in orthogonal_neighbors(cx, cy) {
                    if grid[ny][nx] != Terrain::Sea && (nx, ny) != spawn_position {
                        frontier.push((nx, ny));
                    }
                }
            }

            frontier.swap_remove(idx);
        }
    }
}

/// Phase 2.5c: 侵食・湖で生まれた極小の島を海に吸収する
///
/// spawn_position を含む島は保護する。
fn remove_tiny_islands(
    grid: &mut Vec<Vec<Terrain>>,
    spawn_position: (usize, usize),
) {
    const MIN_ISLAND_SIZE: usize = 8;

    let islands = detect_islands(grid);

    for island in &islands {
        if island.len() >= MIN_ISLAND_SIZE {
            continue;
        }
        // spawn_position を含む島は除去しない
        if island.iter().any(|&(x, y)| (x, y) == spawn_position) {
            continue;
        }
        for &(x, y) in island {
            grid[y][x] = Terrain::Sea;
        }
    }
}

/// Phase 3: 地形クラスタを散布する（Forest / Mountain）
///
/// `spawn_position` は Plains のまま保護する。
fn scatter_terrain_clusters(
    grid: &mut Vec<Vec<Terrain>>,
    spawn_position: (usize, usize),
    rng: &mut impl Rng,
) {
    // 陸地タイルリストを構築
    let plains_tiles: Vec<(usize, usize)> = (0..MAP_HEIGHT)
        .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| grid[y][x] == Terrain::Plains && (x, y) != spawn_position)
        .collect();

    if plains_tiles.is_empty() {
        return;
    }

    // Forest クラスタ
    for _ in 0..FOREST_CLUSTERS {
        let cluster_size = rng.gen_range(FOREST_CLUSTER_MIN..=FOREST_CLUSTER_MAX);
        let start_idx = rng.gen_range(0..plains_tiles.len());
        let (sx, sy) = plains_tiles[start_idx];

        if grid[sy][sx] != Terrain::Plains {
            continue;
        }

        // Flood Fill でクラスタ拡張
        let mut cluster_frontier = vec![(sx, sy)];
        let mut placed = 0usize;

        while placed < cluster_size && !cluster_frontier.is_empty() {
            let idx = rng.gen_range(0..cluster_frontier.len());
            let (cx, cy) = cluster_frontier[idx];

            if grid[cy][cx] == Terrain::Plains && (cx, cy) != spawn_position {
                grid[cy][cx] = Terrain::Forest;
                placed += 1;

                for (nx, ny) in orthogonal_neighbors(cx, cy) {
                    if grid[ny][nx] == Terrain::Plains && (nx, ny) != spawn_position {
                        cluster_frontier.push((nx, ny));
                    }
                }
            }

            cluster_frontier.swap_remove(idx);
        }
    }

    // Mountain クラスタ
    let current_plains: Vec<(usize, usize)> = (0..MAP_HEIGHT)
        .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| grid[y][x] == Terrain::Plains && (x, y) != spawn_position)
        .collect();

    if current_plains.is_empty() {
        return;
    }

    for _ in 0..MOUNTAIN_CLUSTERS {
        let cluster_size = rng.gen_range(MOUNTAIN_CLUSTER_MIN..=MOUNTAIN_CLUSTER_MAX);
        let start_idx = rng.gen_range(0..current_plains.len());
        let (sx, sy) = current_plains[start_idx];

        if grid[sy][sx] != Terrain::Plains {
            continue;
        }

        let mut cluster_frontier = vec![(sx, sy)];
        let mut placed = 0usize;

        while placed < cluster_size && !cluster_frontier.is_empty() {
            let idx = rng.gen_range(0..cluster_frontier.len());
            let (cx, cy) = cluster_frontier[idx];

            if grid[cy][cx] == Terrain::Plains && (cx, cy) != spawn_position {
                grid[cy][cx] = Terrain::Mountain;
                placed += 1;

                for (nx, ny) in orthogonal_neighbors(cx, cy) {
                    if grid[ny][nx] == Terrain::Plains && (nx, ny) != spawn_position {
                        cluster_frontier.push((nx, ny));
                    }
                }
            }

            cluster_frontier.swap_remove(idx);
        }
    }
}

/// 特殊タイル（Town/Cave/Hokora）の周囲2マスを歩行可能にする
///
/// 街・洞窟・祠の周囲2マス以内の Mountain や Sea を Plains に変換し、
/// プレイヤーが確実にアクセスできるようにする。
fn clear_around_special_tiles(grid: &mut Vec<Vec<Terrain>>) {
    let special_tiles: Vec<(usize, usize)> = (0..MAP_HEIGHT)
        .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| matches!(grid[y][x], Terrain::Town | Terrain::Cave | Terrain::Hokora))
        .collect();

    for &(sx, sy) in &special_tiles {
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let (nx, ny) = wrap_position(sx, sy, dx, dy);
                if !grid[ny][nx].is_walkable() {
                    grid[ny][nx] = Terrain::Plains;
                }
            }
        }
    }
}

/// 各島内の歩行可能タイルが全て連結であることを保証する
///
/// Mountain が歩行不可のため、山の配置で歩行可能エリアが分断される可能性がある。
/// 分断が検出された場合、最大連結成分以外の成分から最大成分へBFSで最短経路を探し、
/// 経路上の Mountain を Plains に変換して道を作る。
fn ensure_walkable_connectivity(grid: &mut Vec<Vec<Terrain>>) {
    let islands = detect_islands(grid);

    for island in &islands {
        // 島内の歩行可能タイルを収集
        let walkable: Vec<(usize, usize)> = island
            .iter()
            .filter(|&&(x, y)| grid[y][x].is_walkable())
            .copied()
            .collect();

        if walkable.is_empty() {
            continue;
        }

        // BFS で歩行可能タイルの連結成分を検出
        let walkable_set: HashSet<(usize, usize)> = walkable.iter().copied().collect();
        let mut visited = HashSet::new();
        let mut components: Vec<Vec<(usize, usize)>> = Vec::new();

        for &start in &walkable {
            if visited.contains(&start) {
                continue;
            }
            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(start);
            visited.insert(start);

            while let Some((x, y)) = queue.pop_front() {
                component.push((x, y));
                for (nx, ny) in orthogonal_neighbors(x, y) {
                    if walkable_set.contains(&(nx, ny)) && !visited.contains(&(nx, ny)) {
                        visited.insert((nx, ny));
                        queue.push_back((nx, ny));
                    }
                }
            }
            components.push(component);
        }

        if components.len() <= 1 {
            continue;
        }

        // 最大連結成分を特定
        let main_idx = components
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| c.len())
            .map(|(i, _)| i)
            .unwrap();
        let main_set: HashSet<(usize, usize)> =
            components[main_idx].iter().copied().collect();

        // 島内のタイルセット（山を通って経路探索するため）
        let island_set: HashSet<(usize, usize)> = island.iter().copied().collect();

        // 各非最大成分から最大成分へ山を通る最短経路を見つけて道を開く
        for (i, component) in components.iter().enumerate() {
            if i == main_idx {
                continue;
            }

            // BFS: 成分の全タイルから開始し、島内の山も通過可能として探索
            // 最大成分のタイルに到達したら経路上の Mountain を Plains に変換
            let mut bfs_visited: HashSet<(usize, usize)> = HashSet::new();
            let mut bfs_queue: VecDeque<(usize, usize)> = VecDeque::new();
            let mut parent: std::collections::HashMap<(usize, usize), (usize, usize)> =
                std::collections::HashMap::new();

            for &tile in component {
                bfs_queue.push_back(tile);
                bfs_visited.insert(tile);
            }

            let mut target = None;
            while let Some((x, y)) = bfs_queue.pop_front() {
                if main_set.contains(&(x, y)) {
                    target = Some((x, y));
                    break;
                }
                for (nx, ny) in orthogonal_neighbors(x, y) {
                    if island_set.contains(&(nx, ny)) && !bfs_visited.contains(&(nx, ny)) {
                        bfs_visited.insert((nx, ny));
                        parent.insert((nx, ny), (x, y));
                        bfs_queue.push_back((nx, ny));
                    }
                }
            }

            // 経路を遡り、Mountain を Plains に変換
            if let Some(end) = target {
                let mut current = end;
                while let Some(&prev) = parent.get(&current) {
                    if grid[current.1][current.0] == Terrain::Mountain {
                        grid[current.1][current.0] = Terrain::Plains;
                    }
                    current = prev;
                }
            }
        }
    }
}

pub fn generate_map(rng: &mut impl Rng) -> MapData {
    // 全タイルを Sea で初期化
    let mut grid: Vec<Vec<Terrain>> = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];

    // Phase 1: 大陸中心点配置
    let centers = place_continent_centers(rng);

    // spawn_position はメイン大陸（centers[0]）の中心
    let spawn_position = centers[0];

    // Phase 2: 大陸成長
    grow_continents(&mut grid, &centers, rng);

    // spawn_position が Plains であることを保証
    grid[spawn_position.1][spawn_position.0] = Terrain::Plains;

    // Phase 2.5a: 海岸線侵食（入り江・湾の生成）
    erode_coastline(&mut grid, spawn_position, rng);

    // Phase 2.5b: 内陸湖の配置
    place_lakes(&mut grid, spawn_position, rng);

    // Phase 2.5c: 極小の島を除去
    remove_tiny_islands(&mut grid, spawn_position);

    // spawn_position が Plains であることを再保証
    grid[spawn_position.1][spawn_position.0] = Terrain::Plains;

    // Phase 3: 地形散布
    scatter_terrain_clusters(&mut grid, spawn_position, rng);

    // spawn_position が Plains であることを再保証
    grid[spawn_position.1][spawn_position.0] = Terrain::Plains;

    // Phase 4: 町・洞窟配置
    let town_spawns = calculate_town_spawns(&grid, rng, spawn_position);
    for town in &town_spawns {
        grid[town.y][town.x] = Terrain::Town;
    }

    let cave_spawns = calculate_cave_spawns(&grid, rng);
    for cave in &cave_spawns {
        grid[cave.y][cave.x] = Terrain::Cave;
    }

    let hokora_spawns = calculate_hokora_spawns(&grid, rng, &centers, spawn_position);
    for hokora in &hokora_spawns {
        grid[hokora.y][hokora.x] = Terrain::Hokora;
    }

    // spawn_position が Plains であることを最終保証
    grid[spawn_position.1][spawn_position.0] = Terrain::Plains;

    // Phase 4.5: 特殊タイルの周囲2マスを歩行可能にする
    clear_around_special_tiles(&mut grid);

    // Phase 5: 歩行可能タイルの連結性を保証（山で道が塞がれないようにする）
    ensure_walkable_connectivity(&mut grid);

    MapData {
        grid,
        spawn_position,
    }
}

/// 接続性を保証したマップを生成する
///
/// `generate_map` で生成後、全島が最大海域に隣接するかを検証し、
/// 失敗した場合は再生成する（最大10回）。
pub fn generate_connected_map(rng: &mut impl Rng) -> MapData {
    const MAX_RETRIES: usize = 20;

    let mut map = generate_map(rng);
    for _ in 0..MAX_RETRIES {
        if validate_connectivity(&map.grid) {
            return map;
        }
        map = generate_map(rng);
    }

    // 10回失敗しても最後の結果を返す
    map
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

    #[test]
    fn map_stats_island_sizes_and_towns() {
        use crate::map::islands::detect_islands;

        for seed in [42, 123, 456] {
            let mut rng = create_rng(seed);
            let map = generate_connected_map(&mut rng);

            let islands = detect_islands(&map.grid);

            let town_count: usize = map
                .grid
                .iter()
                .flatten()
                .filter(|t| **t == Terrain::Town)
                .count();
            let cave_count: usize = map
                .grid
                .iter()
                .flatten()
                .filter(|t| **t == Terrain::Cave)
                .count();

            eprintln!("=== seed {} ===", seed);
            eprintln!("島数: {}", islands.len());
            for (i, island) in islands.iter().enumerate() {
                let towns_on_island = island
                    .iter()
                    .filter(|&&(x, y)| map.grid[y][x] == Terrain::Town)
                    .count();
                let caves_on_island = island
                    .iter()
                    .filter(|&&(x, y)| map.grid[y][x] == Terrain::Cave)
                    .count();
                eprintln!(
                    "  島{}: {}タイル, 街{}個, 洞窟{}個",
                    i + 1,
                    island.len(),
                    towns_on_island,
                    caves_on_island
                );
            }
            eprintln!("合計: 街{}個, 洞窟{}個", town_count, cave_count);

            assert!(town_count > 0, "seed {}: 街が1つもない", seed);

            // 街間の最小距離を検証
            let towns: Vec<(usize, usize)> = (0..MAP_HEIGHT)
                .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
                .filter(|&(x, y)| map.grid[y][x] == Terrain::Town)
                .collect();

            for i in 0..towns.len() {
                for j in (i + 1)..towns.len() {
                    let (x1, y1) = towns[i];
                    let (x2, y2) = towns[j];
                    let dx = x1 as f64 - x2 as f64;
                    let dy = y1 as f64 - y2 as f64;
                    let dist = (dx * dx + dy * dy).sqrt();
                    assert!(
                        dist >= 10.0,
                        "seed {}: 街({},{})と({},{})の距離が{:.1}で近すぎる",
                        seed, x1, y1, x2, y2, dist
                    );
                }
            }
        }
    }
}
