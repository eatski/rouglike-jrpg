//! 島検出と船スポーン位置の計算

use super::Terrain;
use crate::coordinates::orthogonal_neighbors;
use rand::prelude::SliceRandom;
use rand::Rng;
use terrain::{MAP_HEIGHT, MAP_WIDTH};

/// 町のスポーン情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TownSpawn {
    /// タイル座標
    pub x: usize,
    pub y: usize,
}

/// 洞窟のスポーン情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaveSpawn {
    /// タイル座標
    pub x: usize,
    pub y: usize,
}

/// 船のスポーン情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoatSpawn {
    /// タイル座標
    pub x: usize,
    pub y: usize,
}

/// 祠のスポーン情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HokoraSpawn {
    /// タイル座標
    pub x: usize,
    pub y: usize,
    /// ワープ先座標
    pub warp_destination: (usize, usize),
}

/// 仲間候補の街割り当て情報
#[derive(Debug, Clone)]
pub struct CandidatePlacement {
    /// 仲間候補のインデックス（candidates配列のインデックス）
    pub candidate_index: usize,
    /// 初回の街の位置
    pub first_town: (usize, usize),
    /// 知り合い後に移動する先の街の位置
    pub second_town: (usize, usize),
}

/// Flood Fillで連結した陸地を島として検出
///
/// Sea以外のタイルを連結成分ごとにグループ化する。
/// 返値はサイズ降順にソートされる。
pub fn detect_islands(grid: &[Vec<Terrain>]) -> Vec<Vec<(usize, usize)>> {
    let mut visited = vec![vec![false; MAP_WIDTH]; MAP_HEIGHT];
    let mut islands: Vec<Vec<(usize, usize)>> = Vec::new();

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            if grid[y][x] != Terrain::Sea && !visited[y][x] {
                // BFSで連結成分を収集
                let mut island = Vec::new();
                let mut queue = std::collections::VecDeque::new();
                queue.push_back((x, y));
                visited[y][x] = true;

                while let Some((cx, cy)) = queue.pop_front() {
                    island.push((cx, cy));
                    for (nx, ny) in orthogonal_neighbors(cx, cy) {
                        if !visited[ny][nx] && grid[ny][nx] != Terrain::Sea {
                            visited[ny][nx] = true;
                            queue.push_back((nx, ny));
                        }
                    }
                }

                islands.push(island);
            }
        }
    }

    // サイズ降順でソート
    islands.sort_by(|a, b| b.len().cmp(&a.len()));
    islands
}

/// Flood Fillで連結した海タイルを海域として検出（内部関数）
///
/// 返値はサイズ降順にソートされる（[0]が最大海域）。
fn detect_sea_regions(grid: &[Vec<Terrain>]) -> Vec<Vec<(usize, usize)>> {
    let mut visited = vec![vec![false; MAP_WIDTH]; MAP_HEIGHT];
    let mut regions: Vec<Vec<(usize, usize)>> = Vec::new();

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            if grid[y][x] == Terrain::Sea && !visited[y][x] {
                let mut region = Vec::new();
                let mut queue = std::collections::VecDeque::new();
                queue.push_back((x, y));
                visited[y][x] = true;

                while let Some((cx, cy)) = queue.pop_front() {
                    region.push((cx, cy));
                    for (nx, ny) in orthogonal_neighbors(cx, cy) {
                        if !visited[ny][nx] && grid[ny][nx] == Terrain::Sea {
                            visited[ny][nx] = true;
                            queue.push_back((nx, ny));
                        }
                    }
                }

                regions.push(region);
            }
        }
    }

    // サイズ降順でソート
    regions.sort_by(|a, b| b.len().cmp(&a.len()));
    regions
}

/// 全ての島が最大海域に隣接するか検証
pub fn validate_connectivity(grid: &[Vec<Terrain>]) -> bool {
    let islands = detect_islands(grid);

    // 島がない場合は true
    if islands.is_empty() {
        return true;
    }

    let sea_regions = detect_sea_regions(grid);

    // 海域が存在しない場合（全陸地）は全島が接続不可
    if sea_regions.is_empty() {
        return false;
    }

    // 最大海域のタイル集合をHashSetに変換
    let max_sea: std::collections::HashSet<(usize, usize)> =
        sea_regions[0].iter().copied().collect();

    // 全ての島が最大海域に隣接しているか確認
    for island in &islands {
        let touches_max_sea = island.iter().any(|&(x, y)| {
            orthogonal_neighbors(x, y)
                .iter()
                .any(|pos| max_sea.contains(pos))
        });
        if !touches_max_sea {
            return false;
        }
    }

    true
}

/// 島の外郭タイル（海に隣接する陸地）を検出し、最大海域に面した海タイルを船スポーン位置として返す
pub fn calculate_boat_spawns(grid: &[Vec<Terrain>], rng: &mut impl Rng) -> Vec<BoatSpawn> {
    let islands = detect_islands(grid);
    let sea_regions = detect_sea_regions(grid);

    if islands.is_empty() || sea_regions.is_empty() {
        return Vec::new();
    }

    let max_sea: std::collections::HashSet<(usize, usize)> =
        sea_regions[0].iter().copied().collect();

    let mut spawns = Vec::new();

    for island in &islands {
        // この島の陸タイルに隣接する最大海域の海タイルを収集
        let mut candidates: Vec<(usize, usize)> = island
            .iter()
            .flat_map(|&(x, y)| {
                orthogonal_neighbors(x, y)
                    .iter()
                    .filter(|pos| max_sea.contains(*pos))
                    .copied()
                    .collect::<Vec<_>>()
            })
            .collect();

        // 重複を除去
        candidates.sort();
        candidates.dedup();

        if let Some(&(x, y)) = candidates.choose(rng) {
            spawns.push(BoatSpawn { x, y });
        }
    }

    spawns
}

/// 大きい島（大陸1,2）の街の数
const TOWNS_LARGE_ISLAND: usize = 5;
/// 小さい島（大陸3,4,5）の街の数
const TOWNS_SMALL_ISLAND: usize = 3;
/// 大きい島とみなすタイル数の閾値
const LARGE_ISLAND_THRESHOLD: usize = 2500;
/// 街を配置する最小島サイズ
const MIN_ISLAND_SIZE_FOR_TOWNS: usize = 100;
/// 街間の最小距離
const MIN_TOWN_DISTANCE: f64 = 10.0;

/// 2点間のユークリッド距離
fn tile_distance(x1: usize, y1: usize, x2: usize, y2: usize) -> f64 {
    let dx = x1 as f64 - x2 as f64;
    let dy = y1 as f64 - y2 as f64;
    (dx * dx + dy * dy).sqrt()
}

/// 各島に町スポーン位置を計算
///
/// 島のサイズに応じて街の数を決定する。
/// 大きい島（2500タイル以上）は5個、小さい島は3個。
/// 極小島（100タイル未満）には街を配置しない。
/// 街同士は最低10タイルの間隔を保つ。
/// `spawn_position` (x, y) はプレイヤー初期位置として除外する。
pub fn calculate_town_spawns(
    grid: &[Vec<Terrain>],
    rng: &mut impl Rng,
    spawn_position: (usize, usize),
) -> Vec<TownSpawn> {
    let islands = detect_islands(grid);
    let mut spawns = Vec::new();

    for island in &islands {
        if island.len() < MIN_ISLAND_SIZE_FOR_TOWNS {
            continue;
        }

        let town_count = if island.len() >= LARGE_ISLAND_THRESHOLD {
            TOWNS_LARGE_ISLAND
        } else {
            TOWNS_SMALL_ISLAND
        };

        let mut candidates: Vec<(usize, usize)> = island
            .iter()
            .filter(|&pos| {
                matches!(grid[pos.1][pos.0], Terrain::Plains | Terrain::DarkPlains)
                    && *pos != spawn_position
            })
            .copied()
            .collect();

        candidates.shuffle(rng);

        let mut placed: Vec<(usize, usize)> = Vec::new();
        for &(x, y) in &candidates {
            if placed.len() >= town_count {
                break;
            }
            let far_enough = placed
                .iter()
                .all(|&(px, py)| tile_distance(x, y, px, py) >= MIN_TOWN_DISTANCE);
            if far_enough {
                placed.push((x, y));
                spawns.push(TownSpawn { x, y });
            }
        }
    }

    spawns
}

/// 島あたりの洞窟の数
const CAVES_PER_ISLAND: usize = 2;

/// 各島に洞窟スポーン位置を計算
///
/// 各島の Mountain タイルからランダムに `CAVES_PER_ISLAND` 個選択する。
/// `boss_continent_center` が指定された場合、その大陸中心を含む島はスキップする。
pub fn calculate_cave_spawns(
    grid: &[Vec<Terrain>],
    rng: &mut impl Rng,
    boss_continent_center: Option<(usize, usize)>,
) -> Vec<CaveSpawn> {
    let islands = detect_islands(grid);
    let mut spawns = Vec::new();

    for island in &islands {
        if island.len() < MIN_ISLAND_SIZE_FOR_TOWNS {
            continue;
        }

        // ボス大陸の島には通常洞窟を配置しない
        if let Some(center) = boss_continent_center {
            if island.contains(&center) {
                continue;
            }
        }

        let mut candidates: Vec<(usize, usize)> = island
            .iter()
            .filter(|&&(x, y)| grid[y][x] == Terrain::Mountain)
            .copied()
            .collect();

        candidates.shuffle(rng);
        for &(x, y) in candidates.iter().take(CAVES_PER_ISLAND) {
            spawns.push(CaveSpawn { x, y });
        }
    }

    spawns
}

/// ボス洞窟のスポーン位置を計算
///
/// ボス大陸中心を含む島の Mountain タイルから1つ選択する。
pub fn calculate_boss_cave_spawn(
    grid: &[Vec<Terrain>],
    rng: &mut impl Rng,
    boss_continent_center: (usize, usize),
) -> Option<(usize, usize)> {
    let islands = detect_islands(grid);
    let island = islands
        .iter()
        .find(|island| island.contains(&boss_continent_center))?;

    let mut candidates: Vec<(usize, usize)> = island
        .iter()
        .filter(|&&(x, y)| grid[y][x] == Terrain::Mountain)
        .copied()
        .collect();

    candidates.shuffle(rng);
    candidates.first().copied()
}

/// 大陸1と大陸2に祠を1つずつ配置する
///
/// 各大陸中心を含む島から Plains タイルを選んで祠を配置する。
/// `spawn_position` は除外する。
///
/// ワープ先:
/// - 大陸1の祠 → 大陸2のPlainsタイル（何もない場所）
/// - 大陸2の祠 → 大陸1の祠の座標
pub fn calculate_hokora_spawns(
    grid: &[Vec<Terrain>],
    rng: &mut impl Rng,
    continent_centers: &[(usize, usize)],
    spawn_position: (usize, usize),
) -> Vec<HokoraSpawn> {
    let islands = detect_islands(grid);

    // まず各大陸の祠位置を決定
    let mut hokora_positions: Vec<(usize, usize)> = Vec::new();
    // 大陸2のPlainsタイル候補（ワープ先用）
    let mut continent2_plains: Vec<(usize, usize)> = Vec::new();

    for (idx, &center) in continent_centers.iter().take(2).enumerate() {
        let island = islands.iter().find(|island| island.contains(&center));
        if let Some(island) = island {
            let mut candidates: Vec<(usize, usize)> = island
                .iter()
                .filter(|&&(x, y)| {
                    grid[y][x] == Terrain::Plains && (x, y) != spawn_position
                })
                .copied()
                .collect();
            candidates.shuffle(rng);
            if let Some(&pos) = candidates.first() {
                hokora_positions.push(pos);
            }
            // 大陸2のPlains候補を保存
            if idx == 1 {
                continent2_plains = candidates;
            }
        }
    }

    if hokora_positions.len() < 2 {
        // 2つの大陸に配置できなかった場合はフォールバック
        return hokora_positions
            .into_iter()
            .map(|(x, y)| HokoraSpawn {
                x,
                y,
                warp_destination: (x, y),
            })
            .collect();
    }

    let hokora0 = hokora_positions[0]; // 大陸1の祠
    let hokora1 = hokora_positions[1]; // 大陸2の祠

    // 大陸1の祠のワープ先: 大陸2のPlainsタイル（祠自体は除外）
    let warp_dest_0 = continent2_plains
        .iter()
        .find(|&&pos| pos != hokora1)
        .copied()
        .unwrap_or(hokora1); // フォールバック: 大陸2の祠

    // 大陸2の祠のワープ先: 大陸1の祠
    let warp_dest_1 = hokora0;

    vec![
        HokoraSpawn {
            x: hokora0.0,
            y: hokora0.1,
            warp_destination: warp_dest_0,
        },
        HokoraSpawn {
            x: hokora1.0,
            y: hokora1.1,
            warp_destination: warp_dest_1,
        },
    ]
}

/// スポーン大陸に追加の街を配置する
///
/// `spawn_position` が属する島にPlainsタイルからランダムに追加の街を配置する。
/// 既存のTownタイルとspawn_positionを避ける。
pub fn place_extra_towns(
    grid: &mut [Vec<Terrain>],
    rng: &mut impl Rng,
    spawn_position: (usize, usize),
    extra_count: usize,
) -> Vec<TownSpawn> {
    let (sx, sy) = spawn_position;

    // spawn_positionが属する島を検出
    let islands = detect_islands(grid);
    let spawn_island = islands
        .into_iter()
        .find(|island| island.contains(&(sx, sy)));

    let spawn_island = match spawn_island {
        Some(island) => island,
        None => return Vec::new(),
    };

    let mut placed = Vec::new();

    // 既存の街座標を収集（間隔チェック用）
    let mut existing_towns: Vec<(usize, usize)> = spawn_island
        .iter()
        .filter(|&&(x, y)| grid[y][x] == Terrain::Town)
        .copied()
        .collect();

    for _ in 0..extra_count {
        let mut candidates: Vec<(usize, usize)> = spawn_island
            .iter()
            .filter(|&&(x, y)| {
                grid[y][x] == Terrain::Plains && (x, y) != spawn_position
            })
            .copied()
            .collect();

        candidates.shuffle(rng);

        let picked = candidates.iter().find(|&&(x, y)| {
            existing_towns
                .iter()
                .all(|&(tx, ty)| tile_distance(x, y, tx, ty) >= MIN_TOWN_DISTANCE)
        });

        if let Some(&(x, y)) = picked {
            grid[y][x] = Terrain::Town;
            placed.push(TownSpawn { x, y });
            existing_towns.push((x, y));
        } else {
            break;
        }
    }

    placed
}

/// 仲間候補をスポーン大陸の街に割り当てる
///
/// 各候補に first_town（初対面）と second_town（加入）を割り当てる。
/// 候補数 + 1 以上の街が必要。
pub fn assign_candidates_to_towns(
    spawn_island_towns: &[(usize, usize)],
    candidate_count: usize,
    rng: &mut impl Rng,
) -> Vec<CandidatePlacement> {
    // 候補数 + 1 以上の街が必要
    if spawn_island_towns.len() < candidate_count + 1 {
        return Vec::new();
    }

    let mut towns: Vec<(usize, usize)> = spawn_island_towns.to_vec();
    towns.shuffle(rng);

    let mut placements = Vec::new();

    // 各候補に対してfirst_townとsecond_townを割り当て
    // towns[0], towns[1] -> 候補0
    // towns[1], towns[2] -> 候補1
    // ...のようにスライドウィンドウで割り当てる
    for i in 0..candidate_count {
        placements.push(CandidatePlacement {
            candidate_index: i,
            first_town: towns[i],
            second_town: towns[i + 1],
        });
    }

    placements
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
    fn place_extra_towns_adds_towns_on_spawn_island() {
        let mut grid = create_test_grid(Terrain::Sea);
        // スポーン大陸（十分な広さ: 30x30）
        for y in 5..35 {
            for x in 5..35 {
                grid[y][x] = Terrain::Plains;
            }
        }
        let spawn_pos = (20, 20);

        let mut rng = create_rng(42);
        let extra = place_extra_towns(&mut grid, &mut rng, spawn_pos, 2);

        assert_eq!(extra.len(), 2);
        for town in &extra {
            assert_eq!(grid[town.y][town.x], Terrain::Town);
            assert_ne!((town.x, town.y), spawn_pos);
        }

        // 街間の距離が最小距離以上であることを検証
        let dx = extra[0].x as f64 - extra[1].x as f64;
        let dy = extra[0].y as f64 - extra[1].y as f64;
        let dist = (dx * dx + dy * dy).sqrt();
        assert!(
            dist >= MIN_TOWN_DISTANCE,
            "街間の距離が{:.1}で近すぎる",
            dist
        );
    }

    #[test]
    fn assign_candidates_to_towns_creates_placements() {
        let towns = vec![(5, 5), (10, 5), (15, 5)];
        let mut rng = create_rng(42);

        let placements = assign_candidates_to_towns(&towns, 2, &mut rng);

        assert_eq!(placements.len(), 2);
        // 各候補のfirst_townとsecond_townが異なることを確認
        for p in &placements {
            assert_ne!(p.first_town, p.second_town);
        }
    }

    #[test]
    fn assign_candidates_insufficient_towns() {
        let towns = vec![(5, 5)]; // 街が1つだけ
        let mut rng = create_rng(42);

        let placements = assign_candidates_to_towns(&towns, 2, &mut rng);
        assert!(placements.is_empty());
    }
}
