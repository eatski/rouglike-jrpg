use bevy::prelude::*;

use std::collections::HashMap;

use world_gen::{
    assign_candidates_to_towns, calculate_boat_spawns, detect_islands, generate_connected_map,
    place_extra_towns,
};

use field_core::{ActiveMap, Player, TilePosition, TILE_SIZE};
use party::default_candidates;
use app_state::{ContinentCavePositions, ContinentMap, EncounterZone, HokoraPositions, RecruitmentMap};
use terrain::Structure;

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

    let islands = detect_islands(&map_data.grid);
    let candidate_count = default_candidates().len();

    // 祠の位置を先に取得（大陸特定に使用）
    let (hokora_pos_vec, warp_dest_vec): (Vec<_>, Vec<_>) =
        map_data.hokora_spawns.iter().copied().unzip();

    // 各大陸に仲間候補を3名ずつ分散配置
    let mut town_to_candidate = HashMap::new();
    let mut candidate_second_town = HashMap::new();
    let candidates_per_continent = 3;
    let num_continents = hokora_pos_vec.len().min(candidate_count / candidates_per_continent);

    for (continent_idx, &continent_pos) in hokora_pos_vec.iter().enumerate().take(num_continents) {
        let island = match islands.iter().find(|island| island.contains(&continent_pos)) {
            Some(island) => island,
            None => continue,
        };

        // 必要に応じて追加街を配置
        let start = continent_idx * candidates_per_continent;
        let end = (start + candidates_per_continent).min(candidate_count);
        let needed = end - start;
        let town_count = island
            .iter()
            .filter(|&&(x, y)| map_data.structures[y][x] == Structure::Town)
            .count();
        if town_count < needed + 1 {
            place_extra_towns(
                &map_data.grid,
                &mut map_data.structures,
                rng,
                continent_pos,
                needed,
            );
        }

        // この大陸の街を収集
        let island_towns: Vec<(usize, usize)> = island
            .iter()
            .copied()
            .filter(|&(x, y)| map_data.structures[y][x] == Structure::Town)
            .collect();

        // 候補インデックスを割り当て
        let candidate_indices: Vec<usize> = (start..end).collect();
        let placements = assign_candidates_to_towns(&island_towns, &candidate_indices, rng);
        for p in &placements {
            town_to_candidate.insert(p.first_town, p.candidate_index);
            candidate_second_town.insert(p.candidate_index, p.second_town);
        }
    }

    commands.insert_resource(RecruitmentMap {
        town_to_candidate,
        candidate_second_town,
        hire_available: HashMap::new(),
    });

    // 船のスポーン位置を計算
    let boat_spawns = calculate_boat_spawns(&map_data.grid, rng);

    commands.insert_resource(SpawnPosition {
        x: map_data.spawn_position.0,
        y: map_data.spawn_position.1,
    });

    // 祠の位置とワープ先をリソースとして登録
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
                        .filter(|&(x, y)| map_data.structures[y][x] == Structure::Cave)
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect();
    commands.insert_resource(ContinentCavePositions { caves_by_continent });

    // 大陸マップをリソースとして保存
    commands.insert_resource(ContinentMap::new(map_data.continent_map));
    commands.insert_resource(EncounterZone::default());

    // ボス洞窟座標を保存
    commands.insert_resource(BossCaveWorldPos {
        position: map_data.boss_cave_position,
    });

    let active_map = ActiveMap::from_grid(map_data.grid, map_data.structures);

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
