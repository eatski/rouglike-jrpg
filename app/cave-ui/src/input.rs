use bevy::prelude::*;

use cave::TreasureContent;
use terrain::Structure;

use app_state::{BossBattlePending, OpenedChests, PartyState};
use field_core::{ActiveMap, Player, TilePosition};
use field_walk_ui::{FieldMessageState, SimpleTileMap, TileEnteredEvent};

use crate::scene::{BossCaveState, BossEntity, CaveTreasures};

/// 宝箱取得システム: プレイヤーが宝箱タイルに入ったらアイテムを取得
#[allow(clippy::too_many_arguments)]
pub fn check_chest_system(
    mut commands: Commands,
    mut events: MessageReader<TileEnteredEvent>,
    player_query: Query<&TilePosition, With<Player>>,
    cave_treasures: Option<Res<CaveTreasures>>,
    mut opened_chests: ResMut<OpenedChests>,
    mut party_state: ResMut<PartyState>,
    mut cave_message: ResMut<FieldMessageState>,
    mut active_map: ResMut<ActiveMap>,
    mut tile_map: ResMut<SimpleTileMap>,
) {
    let Some(cave_treasures) = cave_treasures else {
        return;
    };

    for _event in events.read() {
        let Ok(tile_pos) = player_query.single() else {
            continue;
        };

        // Structure::Chest でなければ早期リターン（O(1)判定）
        if active_map.structure_at(tile_pos.x, tile_pos.y) != Structure::Chest {
            continue;
        }

        for (i, treasure) in cave_treasures.treasures.iter().enumerate() {
            if treasure.x != tile_pos.x || treasure.y != tile_pos.y {
                continue;
            }

            // 取得済みチェック
            let cave_pos = cave_treasures.cave_pos;
            if opened_chests
                .chests
                .get(&cave_pos)
                .is_some_and(|set| set.contains(&i))
            {
                continue;
            }

            // アイテム/武器の取得処理
            let chest_item = match treasure.content {
                TreasureContent::Item(item) => item,
                TreasureContent::Weapon(weapon) => item::ItemKind::Weapon(weapon),
            };

            // 先頭メンバーから順にインベントリ追加を試みる
            let mut added = false;
            let mut receiver_name = String::new();
            for member in &mut party_state.members {
                if member.inventory.try_add(chest_item, 1) {
                    receiver_name = member.kind.name().to_string();
                    added = true;
                    break;
                }
            }
            // 全員満杯 → 袋にフォールバック
            if !added && party_state.bag.try_add(chest_item, 1) {
                receiver_name = "ふくろ".to_string();
                added = true;
            }

            if added {
                cave_message.message = Some(format!(
                    "たからばこから {}を てにいれた！（{}）",
                    chest_item.name(),
                    receiver_name,
                ));
                // 取得済みに記録
                opened_chests
                    .chests
                    .entry(cave_pos)
                    .or_default()
                    .insert(i);
            } else {
                cave_message.message =
                    Some("もちものも ふくろも いっぱいだ！".to_string());
                // 取得失敗 → 宝箱は残す（取得済みに記録しない）
                continue;
            }

            // structures を ChestOpen に変更し、タイルを再描画
            active_map.structures[tile_pos.y][tile_pos.x] = Structure::ChestOpen;
            let key = (tile_pos.x as i32, tile_pos.y as i32);
            if let Some(entity) = tile_map.active_tiles.remove(&key) {
                commands.entity(entity).despawn();
            }
            if let Some(entity) = tile_map.structure_overlays.remove(&key) {
                commands.entity(entity).despawn();
            }
            tile_map.last_player_pos = None;
        }
    }
}

/// ボスに隣接したら戦闘を開始するシステム
pub fn check_boss_proximity_system(
    mut commands: Commands,
    mut events: MessageReader<TileEnteredEvent>,
    player_query: Query<&TilePosition, With<Player>>,
    boss_query: Query<&BossEntity>,
    _boss_cave_state: Option<Res<BossCaveState>>,
    mut next_battle_state: ResMut<NextState<app_state::BattleState>>,
) {
    for _event in events.read() {
        let Ok(tile_pos) = player_query.single() else {
            continue;
        };

        for boss in boss_query.iter() {
            let dx = (tile_pos.x as i32 - boss.tile_x as i32).abs();
            let dy = (tile_pos.y as i32 - boss.tile_y as i32).abs();

            // 隣接（上下左右）チェック
            if (dx == 1 && dy == 0) || (dx == 0 && dy == 1) {
                // ボス戦闘トリガーを挿入
                commands.insert_resource(BossBattlePending);
                next_battle_state.set(app_state::BattleState::Active);
                return;
            }
        }
    }
}
