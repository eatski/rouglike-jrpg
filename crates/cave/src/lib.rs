use rand::seq::SliceRandom;
use rand::Rng;

use party::{ItemKind, WeaponKind};
use terrain::{Structure, Terrain};

pub const CAVE_WIDTH: usize = 30;
pub const CAVE_HEIGHT: usize = 30;

const RANDOM_WALK_STEPS: usize = 400;
const MAX_TREASURES: usize = 3;

/// 宝箱の中身
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreasureContent {
    Item(ItemKind),
    Weapon(WeaponKind),
}

impl TreasureContent {
    pub fn name(self) -> &'static str {
        match self {
            TreasureContent::Item(item) => item.name(),
            TreasureContent::Weapon(weapon) => weapon.name(),
        }
    }
}

/// 宝箱の定義（位置と中身）
#[derive(Debug, Clone)]
pub struct TreasureChest {
    pub x: usize,
    pub y: usize,
    pub content: TreasureContent,
}

pub struct CaveMapData {
    pub grid: Vec<Vec<Terrain>>,
    pub structures: Vec<Vec<Structure>>,
    pub width: usize,
    pub height: usize,
    pub spawn_position: (usize, usize),
    pub treasures: Vec<TreasureChest>,
}

pub fn generate_cave_map(rng: &mut impl Rng, guaranteed_items: &[TreasureContent]) -> CaveMapData {
    let mut grid = vec![vec![Terrain::CaveWall; CAVE_WIDTH]; CAVE_HEIGHT];

    // ランダムウォークで通路を掘る
    let mut x = CAVE_WIDTH / 2;
    let mut y = CAVE_HEIGHT / 2;
    grid[y][x] = Terrain::CaveFloor;

    let directions: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    for _ in 0..RANDOM_WALK_STEPS {
        let (dx, dy) = directions[rng.gen_range(0..4)];
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;

        // 外周1マスは壁として残す
        if nx >= 1 && nx < (CAVE_WIDTH as i32 - 1) && ny >= 1 && ny < (CAVE_HEIGHT as i32 - 1) {
            x = nx as usize;
            y = ny as usize;
            grid[y][x] = Terrain::CaveFloor;
        }
    }

    let spawn_position = (CAVE_WIDTH / 2, CAVE_HEIGHT / 2);

    // スポーン地点に梯子を配置（structuresレイヤー）
    let mut structures = vec![vec![Structure::None; CAVE_WIDTH]; CAVE_HEIGHT];
    structures[spawn_position.1][spawn_position.0] = Structure::Ladder;

    // 宝箱配置: 床タイルからスポーン地点を除いた候補を収集
    let mut floor_positions: Vec<(usize, usize)> = grid
        .iter()
        .enumerate()
        .flat_map(|(cy, row)| {
            row.iter().enumerate().filter_map(move |(cx, terrain)| {
                if *terrain == Terrain::CaveFloor && (cx, cy) != spawn_position {
                    Some((cx, cy))
                } else {
                    None
                }
            })
        })
        .collect();

    floor_positions.shuffle(rng);
    let random_count = if floor_positions.is_empty() {
        0
    } else {
        rng.gen_range(1..=MAX_TREASURES.min(floor_positions.len()))
    };
    let treasure_count = random_count.max(guaranteed_items.len()).min(floor_positions.len());

    let treasures: Vec<TreasureChest> = floor_positions[..treasure_count]
        .iter()
        .enumerate()
        .map(|(i, &(tx, ty))| TreasureChest {
            x: tx,
            y: ty,
            content: if i < guaranteed_items.len() {
                guaranteed_items[i]
            } else {
                random_treasure_content(rng)
            },
        })
        .collect();

    CaveMapData {
        grid,
        structures,
        width: CAVE_WIDTH,
        height: CAVE_HEIGHT,
        spawn_position,
        treasures,
    }
}

pub struct BossCaveMapData {
    pub grid: Vec<Vec<Terrain>>,
    pub structures: Vec<Vec<Structure>>,
    pub width: usize,
    pub height: usize,
    pub spawn_position: (usize, usize),
    pub boss_position: (usize, usize),
}

const BOSS_CAVE_WALK_STEPS: usize = 600;

pub fn generate_boss_cave_map(rng: &mut impl Rng) -> BossCaveMapData {
    let mut grid = vec![vec![Terrain::BossCaveWall; CAVE_WIDTH]; CAVE_HEIGHT];

    // ランダムウォークで通路を掘る（ステップ数が多いのでより広い洞窟）
    let mut x = CAVE_WIDTH / 2;
    let mut y = CAVE_HEIGHT / 2;
    grid[y][x] = Terrain::BossCaveFloor;

    let directions: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    for _ in 0..BOSS_CAVE_WALK_STEPS {
        let (dx, dy) = directions[rng.gen_range(0..4)];
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;

        if nx >= 1 && nx < (CAVE_WIDTH as i32 - 1) && ny >= 1 && ny < (CAVE_HEIGHT as i32 - 1) {
            x = nx as usize;
            y = ny as usize;
            grid[y][x] = Terrain::BossCaveFloor;
        }
    }

    let spawn_position = (CAVE_WIDTH / 2, CAVE_HEIGHT / 2);

    // スポーン地点に梯子を配置（structuresレイヤー）
    let mut structures = vec![vec![Structure::None; CAVE_WIDTH]; CAVE_HEIGHT];
    structures[spawn_position.1][spawn_position.0] = Structure::Ladder;

    // ボス位置: スポーンから最も遠い床タイル
    let boss_position = find_boss_position(&grid, spawn_position);

    BossCaveMapData {
        grid,
        structures,
        width: CAVE_WIDTH,
        height: CAVE_HEIGHT,
        spawn_position,
        boss_position,
    }
}

/// スポーンから最も遠い床タイルをボス位置とする（BFS距離）
fn find_boss_position(grid: &[Vec<Terrain>], spawn: (usize, usize)) -> (usize, usize) {
    use std::collections::VecDeque;

    let mut dist = vec![vec![u32::MAX; CAVE_WIDTH]; CAVE_HEIGHT];
    let mut queue = VecDeque::new();

    dist[spawn.1][spawn.0] = 0;
    queue.push_back(spawn);

    let mut farthest = spawn;
    let mut max_dist = 0u32;

    while let Some((cx, cy)) = queue.pop_front() {
        let d = dist[cy][cx];
        for &(dx, dy) in &[(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || nx >= CAVE_WIDTH as i32 || ny < 0 || ny >= CAVE_HEIGHT as i32 {
                continue;
            }
            let (ux, uy) = (nx as usize, ny as usize);
            if dist[uy][ux] != u32::MAX {
                continue;
            }
            if !grid[uy][ux].is_walkable() {
                continue;
            }
            dist[uy][ux] = d + 1;
            if d + 1 > max_dist {
                max_dist = d + 1;
                farthest = (ux, uy);
            }
            queue.push_back((ux, uy));
        }
    }

    farthest
}

fn random_treasure_content(rng: &mut impl Rng) -> TreasureContent {
    // (中身, 重み)
    let table: &[(TreasureContent, u32)] = &[
        (TreasureContent::Item(ItemKind::Herb), 25),
        (TreasureContent::Item(ItemKind::HighHerb), 15),
        (TreasureContent::Item(ItemKind::MagicStone), 25),
        (TreasureContent::Item(ItemKind::SilverOre), 15),
        (TreasureContent::Item(ItemKind::AncientCoin), 10),
        (TreasureContent::Item(ItemKind::DragonScale), 3),
        (TreasureContent::Item(ItemKind::MoonFragment), 10),
        (TreasureContent::Weapon(WeaponKind::WoodenSword), 2),
    ];
    let total: u32 = table.iter().map(|(_, w)| w).sum();
    let mut roll = rng.gen_range(0..total);
    for (content, weight) in table {
        if roll < *weight {
            return *content;
        }
        roll -= weight;
    }
    table.last().unwrap().0
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
        let map = generate_cave_map(&mut rng, &[]);
        assert_eq!(map.grid.len(), CAVE_HEIGHT);
        for row in &map.grid {
            assert_eq!(row.len(), CAVE_WIDTH);
        }
        assert_eq!(map.width, CAVE_WIDTH);
        assert_eq!(map.height, CAVE_HEIGHT);
    }

    #[test]
    fn generate_cave_map_spawn_is_ladder() {
        let mut rng = create_rng(42);
        let map = generate_cave_map(&mut rng, &[]);
        let (sx, sy) = map.spawn_position;
        assert_eq!(map.structures[sy][sx], Structure::Ladder);
    }

    #[test]
    fn generate_cave_map_has_floor_tiles() {
        let mut rng = create_rng(42);
        let map = generate_cave_map(&mut rng, &[]);
        let floor_count = map
            .grid
            .iter()
            .flatten()
            .filter(|t| **t == Terrain::CaveFloor)
            .count();
        assert!(floor_count > 50, "Cave should have enough floor tiles");
    }

    #[test]
    fn generate_cave_map_edges_are_walls() {
        let mut rng = create_rng(42);
        let map = generate_cave_map(&mut rng, &[]);
        // 外周は壁（ワープゾーンは外周に置かれない）
        for x in 0..CAVE_WIDTH {
            assert_ne!(
                map.grid[0][x],
                Terrain::CaveFloor,
                "Top edge should not be floor"
            );
            assert_ne!(
                map.grid[CAVE_HEIGHT - 1][x],
                Terrain::CaveFloor,
                "Bottom edge should not be floor"
            );
        }
        for y in 0..CAVE_HEIGHT {
            assert_ne!(
                map.grid[y][0],
                Terrain::CaveFloor,
                "Left edge should not be floor"
            );
            assert_ne!(
                map.grid[y][CAVE_WIDTH - 1],
                Terrain::CaveFloor,
                "Right edge should not be floor"
            );
        }
    }

    #[test]
    fn generate_cave_map_deterministic() {
        let mut rng1 = create_rng(123);
        let mut rng2 = create_rng(123);
        let map1 = generate_cave_map(&mut rng1, &[]);
        let map2 = generate_cave_map(&mut rng2, &[]);
        assert_eq!(map1.spawn_position, map2.spawn_position);
        assert_eq!(map1.grid, map2.grid);
    }

}
