use std::fs;
use std::path::Path;

mod common;

mod bat;
mod boat;
mod cave;
mod cave_floor;
mod cave_wall;
mod forest;
mod ghost;
mod goblin;
mod ladder;
mod mountain;
mod plains;
mod player;
mod sea;
mod slime;
mod town;
mod warp_zone;
mod wolf;

fn main() {
    let tiles_dir = Path::new("assets/tiles");
    let chars_dir = Path::new("assets/characters");
    let enemies_dir = Path::new("assets/enemies");
    fs::create_dir_all(tiles_dir).expect("Failed to create tiles directory");
    fs::create_dir_all(chars_dir).expect("Failed to create characters directory");
    fs::create_dir_all(enemies_dir).expect("Failed to create enemies directory");

    // 各地形タイルを生成
    sea::generate_sea(tiles_dir);
    plains::generate_plains(tiles_dir);
    forest::generate_forest(tiles_dir);
    mountain::generate_mountain(tiles_dir);
    boat::generate_boat(tiles_dir);
    town::generate_town(tiles_dir);
    cave::generate_cave(tiles_dir);

    // 洞窟内部タイルを生成
    cave_wall::generate_cave_wall(tiles_dir);
    cave_floor::generate_cave_floor(tiles_dir);
    warp_zone::generate_warp_zone(tiles_dir);
    ladder::generate_ladder(tiles_dir);

    // キャラクターを生成
    player::generate_player(chars_dir);

    // 敵キャラクターを生成
    slime::generate_slime(enemies_dir);
    bat::generate_bat(enemies_dir);
    goblin::generate_goblin(enemies_dir);
    wolf::generate_wolf(enemies_dir);
    ghost::generate_ghost(enemies_dir);

    println!("Assets generated in assets/");
}
