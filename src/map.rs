use rand::Rng;

pub const MAP_WIDTH: usize = 100;
pub const MAP_HEIGHT: usize = 100;

#[derive(Clone, Copy)]
pub enum Terrain {
    Plains,
    Mountain,
    Forest,
    Sea,
}

pub fn generate_map(rng: &mut impl Rng) -> Vec<Vec<Terrain>> {
    (0..MAP_HEIGHT)
        .map(|_| (0..MAP_WIDTH).map(|_| pick_terrain(rng)).collect())
        .collect()
}

fn pick_terrain(rng: &mut impl Rng) -> Terrain {
    let roll: f32 = rng.gen_range(0.0..1.0);

    if roll < 0.25 {
        Terrain::Sea
    } else if roll < 0.55 {
        Terrain::Forest
    } else if roll < 0.85 {
        Terrain::Plains
    } else {
        Terrain::Mountain
    }
}
