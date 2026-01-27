use rand::Rng;

pub const MAP_WIDTH: usize = 150;
pub const MAP_HEIGHT: usize = 150;
const TARGET_LAND_TILES: usize = 6000;
const LAND_SPREAD_CHANCE: f32 = 0.65;
const ISLAND_COUNT: usize = 20;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    Plains,
    Mountain,
    Forest,
    Sea,
}

#[derive(bevy::prelude::Resource)]
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

        for (ny, nx) in neighbors(y, x) {
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

    MapData {
        grid,
        spawn_position,
    }
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

            for (ny, nx) in neighbors(y, x) {
                if grid[ny][nx] == Terrain::Plains && rng.gen_bool(0.7) {
                    stack.push((ny, nx));
                }
            }
        }
    }
}

fn neighbors(y: usize, x: usize) -> [(usize, usize); 4] {
    [
        ((y + MAP_HEIGHT - 1) % MAP_HEIGHT, x), // 上 (ラップ)
        ((y + 1) % MAP_HEIGHT, x),              // 下 (ラップ)
        (y, (x + MAP_WIDTH - 1) % MAP_WIDTH),   // 左 (ラップ)
        (y, (x + 1) % MAP_WIDTH),               // 右 (ラップ)
    ]
}
