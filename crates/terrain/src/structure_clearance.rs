//! 構築物周辺のチョークポイント解消ロジック（汎用版）
//!
//! ワールドマップ（トーラス）と洞窟（境界あり）の両方で使用可能。

use std::collections::{HashSet, VecDeque};

use crate::terrain::{Structure, Terrain};

/// 構築物タイルを壁とみなした場合に、歩行可能な4近傍同士がBFSで連結しているか判定する。
///
/// 連結していなければチョークポイント（迂回不能なボトルネック）と判定する。
fn is_structure_chokepoint<N>(
    x: usize,
    y: usize,
    grid: &[Vec<Terrain>],
    structure_set: &HashSet<(usize, usize)>,
    neighbors: &N,
) -> bool
where
    N: Fn(usize, usize) -> Vec<(usize, usize)>,
{
    // 歩行可能な4近傍を収集（構築物タイルは壁扱いで除外）
    let walkable_neighbors: Vec<(usize, usize)> = neighbors(x, y)
        .into_iter()
        .filter(|&(nx, ny)| grid[ny][nx].is_walkable() && !structure_set.contains(&(nx, ny)))
        .collect();

    if walkable_neighbors.len() <= 1 {
        return false;
    }

    // BFSで1つ目の歩行可能隣接から他全てに到達できるか確認
    let start = walkable_neighbors[0];
    let targets: HashSet<(usize, usize)> = walkable_neighbors[1..].iter().copied().collect();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(start);
    queue.push_back(start);

    let mut reached = HashSet::new();

    while let Some((cx, cy)) = queue.pop_front() {
        if targets.contains(&(cx, cy)) {
            reached.insert((cx, cy));
            if reached.len() == targets.len() {
                return false;
            }
        }
        for (nx, ny) in neighbors(cx, cy) {
            if !visited.contains(&(nx, ny))
                && grid[ny][nx].is_walkable()
                && !structure_set.contains(&(nx, ny))
            {
                visited.insert((nx, ny));
                queue.push_back((nx, ny));
            }
        }
    }

    // 到達不能な隣接がある → チョークポイント
    reached.len() < targets.len()
}

/// 構築物周辺の歩行可能性とチョークポイント解消（汎用版）
///
/// 3ステップ:
/// 1. 構築物タイル自体を歩行可能にする
/// 2. 歩行可能隣接が0個の場合、隣接を1つだけfloor化
/// 3. チョークポイントの場合、3x3の歩行不可タイルをfloor化
pub fn clear_around_structures<N, O>(
    grid: &mut [Vec<Terrain>],
    structures: &[Vec<Structure>],
    height: usize,
    width: usize,
    floor_terrain: Terrain,
    neighbors: N,
    offset: O,
) where
    N: Fn(usize, usize) -> Vec<(usize, usize)>,
    O: Fn(usize, usize, i32, i32) -> Option<(usize, usize)>,
{
    let special_tiles: Vec<(usize, usize)> = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .filter(|&(x, y)| structures[y][x] != Structure::None)
        .collect();

    let structure_set: HashSet<(usize, usize)> = special_tiles.iter().copied().collect();

    // Step 1: 構築物タイル自体を歩行可能にする
    for &(sx, sy) in &special_tiles {
        if !grid[sy][sx].is_walkable() {
            grid[sy][sx] = floor_terrain;
        }
    }

    // Step 2: アクセス確保（歩行可能隣接が0個の場合）
    for &(sx, sy) in &special_tiles {
        let walkable_neighbors: Vec<(usize, usize)> = neighbors(sx, sy)
            .into_iter()
            .filter(|&(nx, ny)| grid[ny][nx].is_walkable() && !structure_set.contains(&(nx, ny)))
            .collect();

        if walkable_neighbors.is_empty() {
            // 隣接の非構築物タイルを1つだけfloor化
            for (nx, ny) in neighbors(sx, sy) {
                if !structure_set.contains(&(nx, ny)) {
                    grid[ny][nx] = floor_terrain;
                    break;
                }
            }
        }
    }

    // Step 3: チョークポイント解消
    for &(sx, sy) in &special_tiles {
        if is_structure_chokepoint(sx, sy, grid, &structure_set, &neighbors) {
            // 8近傍(3x3)の歩行不可タイルをfloor化
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    if let Some((nx, ny)) = offset(sx, sy, dx, dy)
                        && !grid[ny][nx].is_walkable()
                    {
                        grid[ny][nx] = floor_terrain;
                    }
                }
            }
        }
    }
}
