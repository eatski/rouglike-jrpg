use rand::seq::SliceRandom;
use rand::Rng;

use party::{ItemKind, WeaponKind};
use terrain::Terrain;

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
    pub width: usize,
    pub height: usize,
    pub spawn_position: (usize, usize),
    pub treasures: Vec<TreasureChest>,
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
    grid: &[Vec<Terrain>],
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

    // スポーン地点に梯子を配置
    grid[spawn_position.1][spawn_position.0] = Terrain::Ladder;

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
    let treasure_count = if floor_positions.is_empty() {
        0
    } else {
        rng.gen_range(1..=MAX_TREASURES.min(floor_positions.len()))
    };

    let treasures: Vec<TreasureChest> = floor_positions[..treasure_count]
        .iter()
        .map(|&(tx, ty)| TreasureChest {
            x: tx,
            y: ty,
            content: random_treasure_content(rng),
        })
        .collect();

    CaveMapData {
        grid,
        width: CAVE_WIDTH,
        height: CAVE_HEIGHT,
        spawn_position,
        treasures,
    }
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
        (TreasureContent::Item(ItemKind::CopperKey), 5),
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
        let map = generate_cave_map(&mut rng);
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
        let map = generate_cave_map(&mut rng);
        let (sx, sy) = map.spawn_position;
        assert_eq!(map.grid[sy][sx], Terrain::Ladder);
    }

    #[test]
    fn generate_cave_map_has_floor_tiles() {
        let mut rng = create_rng(42);
        let map = generate_cave_map(&mut rng);
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
        let map = generate_cave_map(&mut rng);
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
        let map1 = generate_cave_map(&mut rng1);
        let map2 = generate_cave_map(&mut rng2);
        assert_eq!(map1.spawn_position, map2.spawn_position);
        assert_eq!(map1.grid, map2.grid);
    }

    #[test]
    fn try_cave_move_normal() {
        let mut grid = vec![vec![Terrain::CaveWall; 5]; 5];
        grid[2][2] = Terrain::CaveFloor;
        grid[2][3] = Terrain::CaveFloor;

        let result = try_cave_move(2, 2, 1, 0, &grid, 5, 5);
        assert_eq!(result, CaveMoveResult::Moved { new_x: 3, new_y: 2 });
    }

    #[test]
    fn try_cave_move_blocked_by_wall() {
        let mut grid = vec![vec![Terrain::CaveWall; 5]; 5];
        grid[2][2] = Terrain::CaveFloor;

        let result = try_cave_move(2, 2, 1, 0, &grid, 5, 5);
        assert_eq!(result, CaveMoveResult::Blocked);
    }

    #[test]
    fn try_cave_move_blocked_by_boundary() {
        let grid = vec![vec![Terrain::CaveFloor; 5]; 5];

        let result = try_cave_move(0, 0, -1, 0, &grid, 5, 5);
        assert_eq!(result, CaveMoveResult::Blocked);
    }

    #[test]
    fn try_cave_move_diagonal_blocked() {
        let grid = vec![vec![Terrain::CaveFloor; 5]; 5];

        let result = try_cave_move(2, 2, 1, 1, &grid, 5, 5);
        assert_eq!(result, CaveMoveResult::Blocked);
    }

    #[test]
    fn try_cave_move_to_ladder() {
        let mut grid = vec![vec![Terrain::CaveWall; 5]; 5];
        grid[2][2] = Terrain::CaveFloor;
        grid[2][3] = Terrain::Ladder;

        let result = try_cave_move(2, 2, 1, 0, &grid, 5, 5);
        assert_eq!(result, CaveMoveResult::Moved { new_x: 3, new_y: 2 });
    }

}
