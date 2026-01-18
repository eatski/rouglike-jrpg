use rand::Rng;

pub const MAP_WIDTH: usize = 100;
pub const MAP_HEIGHT: usize = 100;
const TARGET_LAND_TILES: usize = 3_000;
const LAND_SPREAD_CHANCE: f32 = 0.65;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    Plains,
    Mountain,
    Forest,
    Sea,
}

pub fn generate_map(rng: &mut impl Rng) -> Vec<Vec<Terrain>> {
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let center = (MAP_HEIGHT / 2, MAP_WIDTH / 2);
    grid[center.0][center.1] = Terrain::Plains;

    let mut land_tiles = 1usize;
    let mut frontier = vec![center];
    let mut land_positions = vec![center];
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
        center,
    );
    scatter_clusters(
        &mut grid,
        rng,
        &land_positions,
        Terrain::Mountain,
        18,
        10..=45,
        center,
    );

    grid
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

fn neighbors(y: usize, x: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(4);
    let y_i = y as isize;
    let x_i = x as isize;

    for (dy, dx) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let ny = y_i + dy;
        let nx = x_i + dx;

        if ny >= 0 && ny < MAP_HEIGHT as isize && nx >= 0 && nx < MAP_WIDTH as isize {
            result.push((ny as usize, nx as usize));
        }
    }

    result
}
