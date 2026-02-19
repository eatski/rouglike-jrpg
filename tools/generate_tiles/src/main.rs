use std::fs;
use std::path::Path;

mod generators;

use generators::*;

fn main() {
    let tiles_dir = Path::new("assets/tiles");
    let chars_dir = Path::new("assets/characters");
    let enemies_dir = Path::new("assets/enemies");
    fs::create_dir_all(tiles_dir).expect("Failed to create tiles directory");
    fs::create_dir_all(chars_dir).expect("Failed to create characters directory");
    fs::create_dir_all(enemies_dir).expect("Failed to create enemies directory");

    // 各地形タイルを生成
    generate_sea(tiles_dir);
    generate_plains(tiles_dir);
    generate_forest(tiles_dir);
    generate_mountain(tiles_dir);
    generate_boat(tiles_dir);
    generate_town(tiles_dir);
    generate_cave(tiles_dir);
    generate_coast_tiles(tiles_dir);

    // 洞窟内部タイルを生成
    generate_cave_wall(tiles_dir);
    generate_cave_floor(tiles_dir);
    generate_warp_zone(tiles_dir);
    generate_ladder(tiles_dir);
    generate_chest(tiles_dir);
    generate_chest_open(tiles_dir);

    // キャラクターを生成
    generate_player(chars_dir);

    // 敵キャラクターを生成
    generate_slime(enemies_dir);
    generate_bat(enemies_dir);
    generate_goblin(enemies_dir);
    generate_wolf(enemies_dir);
    generate_ghost(enemies_dir);

    println!("Assets generated in assets/");
}
