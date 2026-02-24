use bevy::prelude::*;

use std::collections::HashMap;

use world_gen::{
    assign_candidates_to_towns, calculate_boat_spawns, detect_islands, generate_connected_map,
    place_extra_towns,
};

use field_core::{ActiveMap, Player, TilePosition, TILE_SIZE};
use party::default_candidates;
use app_state::{ContinentCavePositions, HokoraPositions, RecruitmentMap};
use terrain::Terrain;

use crate::{load_tile_textures, spawn_boat_entities, BoatSpawnsResource, BossCaveWorldPos};

/// プレイヤーのスポーン位置を保持するリソース
#[derive(Resource)]
pub struct SpawnPosition {
    pub x: usize,
    pub y: usize,
}

/// Rng注入可能なフィールドマップ生成
pub fn spawn_field_map_with_rng(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut impl rand::Rng,
) {
    let tile_textures = load_tile_textures(asset_server);

    let mut map_data = generate_connected_map(rng);

    // スポーン大陸に仲間候補用の追加街を配置
    let candidate_count = default_candidates().len();
    place_extra_towns(
        &mut map_data.grid,
        rng,
        map_data.spawn_position,
        candidate_count,
    );

    // スポーン大陸の座標を収集
    let islands = detect_islands(&map_data.grid);
    let spawn_island: Vec<(usize, usize)> = islands
        .iter()
        .find(|island| island.contains(&map_data.spawn_position))
        .cloned()
        .unwrap_or_default();
    let spawn_island_towns: Vec<(usize, usize)> = spawn_island
        .iter()
        .copied()
        .filter(|&(x, y)| map_data.grid[y][x] == Terrain::Town)
        .collect();

    // 仲間候補を街に割り当て
    let placements = assign_candidates_to_towns(&spawn_island_towns, candidate_count, rng);
    let mut town_to_candidate = HashMap::new();
    let mut candidate_second_town = HashMap::new();
    for p in &placements {
        town_to_candidate.insert(p.first_town, p.candidate_index);
        candidate_second_town.insert(p.candidate_index, p.second_town);
    }
    commands.insert_resource(RecruitmentMap {
        town_to_candidate,
        candidate_second_town,
    });

    // 船のスポーン位置を計算
    let boat_spawns = calculate_boat_spawns(&map_data.grid, rng);

    commands.insert_resource(SpawnPosition {
        x: map_data.spawn_position.0,
        y: map_data.spawn_position.1,
    });

    // 祠の位置とワープ先を生成データから直接取得
    let (hokora_pos_vec, warp_dest_vec): (Vec<_>, Vec<_>) =
        map_data.hokora_spawns.iter().copied().unzip();
    commands.insert_resource(HokoraPositions::new(hokora_pos_vec.clone(), warp_dest_vec));

    // 各祠が所属する大陸の洞窟座標を収集
    let caves_by_continent: Vec<Vec<(usize, usize)>> = hokora_pos_vec
        .iter()
        .map(|hokora_pos| {
            islands
                .iter()
                .find(|island| island.contains(hokora_pos))
                .map(|island| {
                    island
                        .iter()
                        .copied()
                        .filter(|&(x, y)| map_data.grid[y][x] == Terrain::Cave)
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect();
    commands.insert_resource(ContinentCavePositions { caves_by_continent });

    // ボス洞窟座標を保存
    commands.insert_resource(BossCaveWorldPos {
        position: map_data.boss_cave_position,
    });

    let active_map = ActiveMap::from_grid(map_data.grid);

    // 船のスポーン位置を保存してスポーン
    let boat_spawns_resource = BoatSpawnsResource {
        positions: boat_spawns.iter().map(|s| (s.x, s.y)).collect(),
    };
    spawn_boat_entities(commands, &boat_spawns_resource, &tile_textures, &active_map);
    commands.insert_resource(boat_spawns_resource);

    // タイルテクスチャをリソースとして登録（タイルプールで使用）
    commands.insert_resource(tile_textures);
    commands.insert_resource(active_map);
}

/// thread_rng使用の後方互換ラッパー（Bevyシステム用）
pub fn spawn_field_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut rng = rand::thread_rng();
    spawn_field_map_with_rng(&mut commands, &asset_server, &mut rng);
}

pub fn spawn_player(
    mut commands: Commands,
    spawn_pos: Res<SpawnPosition>,
    asset_server: Res<AssetServer>,
    active_map: Res<ActiveMap>,
) {
    let (world_x, world_y) = active_map.to_world(spawn_pos.x, spawn_pos.y);

    let player_texture: Handle<Image> = asset_server.load("characters/player.png");
    let scale = TILE_SIZE / 16.0;

    commands.spawn((
        Player,
        TilePosition {
            x: spawn_pos.x,
            y: spawn_pos.y,
        },
        Sprite::from_image(player_texture),
        Transform::from_xyz(world_x, world_y, 1.0).with_scale(Vec3::splat(scale)),
    ));
}
