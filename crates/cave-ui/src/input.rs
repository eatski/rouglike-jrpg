use bevy::prelude::*;

use cave::TreasureContent;
use terrain::Structure;

use app_state::{BossBattlePending, OpenedChests, PartyState, SceneState};
use field_core::{ActiveMap, Player, TilePosition};
use field_walk_ui::{
    process_movement_input,
    MovementBlockedEvent, MovementLocked, MoveResult, MovementState,
    PendingMove, PlayerMovedEvent, TileEnteredEvent,
};
use app_state::FieldMenuOpen;

use field_walk_ui::{MapModeState, SimpleTileMap};

use crate::scene::{BossCaveState, BossEntity, CaveMessageState, CaveMessageUI, CaveTreasures};

/// 洞窟内のプレイヤー移動入力を処理
#[allow(clippy::too_many_arguments)]
pub fn cave_player_movement(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    active_map: Res<ActiveMap>,
    map_mode_state: Res<MapModeState>,
    field_menu_open: Option<Res<FieldMenuOpen>>,
    cave_message: Option<Res<CaveMessageState>>,
    mut move_state: ResMut<MovementState>,
    mut query: Query<
        (Entity, &mut TilePosition, Option<&MovementLocked>),
        With<Player>,
    >,
    mut blocked_events: MessageWriter<MovementBlockedEvent>,
    mut moved_events: MessageWriter<PlayerMovedEvent>,
) {
    let Ok((entity, mut tile_pos, locked)) = query.single_mut() else {
        return;
    };

    // ガード条件
    if locked.is_some() {
        return;
    }
    if map_mode_state.enabled {
        return;
    }
    if field_menu_open.is_some() {
        return;
    }
    if cave_message.is_some_and(|m| m.message.is_some()) {
        return;
    }

    let Some(input) = process_movement_input(&keyboard, &time, &mut move_state) else {
        return;
    };

    // 洞窟では船なし: execute_moveに直接委譲
    // boat_queryはダミー（洞窟に船はない）
    // on_boat=Noneなのでboat_queryは使われない
    match active_map.try_move(tile_pos.x, tile_pos.y, input.first_dx, input.first_dy) {
        MoveResult::Moved { new_x, new_y } => {
            tile_pos.x = new_x;
            tile_pos.y = new_y;
            moved_events.write(PlayerMovedEvent {
                entity,
                direction: (input.first_dx, input.first_dy),
            });
            if let Some(dir) = input.pending_direction {
                commands.entity(entity).insert(PendingMove { direction: dir });
            }
        }
        MoveResult::Blocked => {
            blocked_events.write(MovementBlockedEvent {
                entity,
                direction: (input.first_dx, input.first_dy),
            });
        }
    }
}

/// 洞窟でのSmoothMove完了後の処理
///
/// PendingMoveがあれば2回目の移動を試行し、なければロック解除＋到着判定。
/// 梯子タイル上にいればフィールドに戻る。
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_cave_move_completed(
    mut commands: Commands,
    mut move_state: ResMut<MovementState>,
    active_map: Res<ActiveMap>,
    mut query: Query<
        (
            Entity,
            &mut TilePosition,
            Option<&PendingMove>,
        ),
        With<Player>,
    >,
    mut moved_events: MessageWriter<PlayerMovedEvent>,
    mut blocked_events: MessageWriter<MovementBlockedEvent>,
    mut tile_entered_events: MessageWriter<TileEnteredEvent>,
    mut next_state: ResMut<NextState<SceneState>>,
) {
    if let Some(_entity) = move_state.move_just_completed.take() {
        let Ok((entity, mut tile_pos, pending_move)) = query.single_mut() else {
            return;
        };

        if let Some(pending) = pending_move {
            let (dx, dy) = pending.direction;
            commands.entity(entity).remove::<PendingMove>();

            match active_map.try_move(tile_pos.x, tile_pos.y, dx, dy) {
                MoveResult::Moved { new_x, new_y } => {
                    tile_pos.x = new_x;
                    tile_pos.y = new_y;
                    moved_events.write(PlayerMovedEvent {
                        entity,
                        direction: (dx, dy),
                    });
                }
                MoveResult::Blocked => {
                    blocked_events.write(MovementBlockedEvent {
                        entity,
                        direction: (dx, dy),
                    });
                    // ワープゾーン上でPendingMoveがブロックされた場合にも
                    // 梯子判定を行う（MovementLockedはバウンスが解除）
                    if tile_pos.x < active_map.width && tile_pos.y < active_map.height
                        && active_map.structure_at(tile_pos.x, tile_pos.y) == Structure::Ladder
                    {
                        next_state.set(SceneState::Exploring);
                    }
                }
            }
        } else {
            commands.entity(entity).remove::<MovementLocked>();
            let structure = active_map.structure_at(tile_pos.x, tile_pos.y);
            if structure == Structure::Ladder {
                // 梯子タイル上 → フィールドに戻る
                next_state.set(SceneState::Exploring);
                return;
            }
            tile_entered_events.write(TileEnteredEvent { entity });
        }
    }
}

/// 宝箱取得システム: プレイヤーが宝箱タイルに入ったらアイテムを取得
#[allow(clippy::too_many_arguments)]
pub fn check_chest_system(
    mut commands: Commands,
    mut events: MessageReader<TileEnteredEvent>,
    player_query: Query<&TilePosition, With<Player>>,
    cave_treasures: Option<Res<CaveTreasures>>,
    mut opened_chests: ResMut<OpenedChests>,
    mut party_state: ResMut<PartyState>,
    mut cave_message: ResMut<CaveMessageState>,
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
            match treasure.content {
                TreasureContent::Item(item) => {
                    // 先頭メンバーから順にインベントリ追加を試みる
                    let mut added = false;
                    let mut receiver_name = String::new();
                    for member in &mut party_state.members {
                        if member.inventory.try_add(item, 1) {
                            receiver_name = member.kind.name().to_string();
                            added = true;
                            break;
                        }
                    }
                    if added {
                        cave_message.message = Some(format!(
                            "たからばこから {}を てにいれた！（{}）",
                            item.name(),
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
                            Some("もちものが いっぱいだ！".to_string());
                        // 取得失敗 → 宝箱は残す（取得済みに記録しない）
                        continue;
                    }
                }
                TreasureContent::Weapon(weapon) => {
                    // 先頭メンバーから順にインベントリ追加を試みる
                    let weapon_item = party::ItemKind::Weapon(weapon);
                    let mut added = false;
                    let mut receiver_name = String::new();
                    for member in &mut party_state.members {
                        if member.inventory.try_add(weapon_item, 1) {
                            receiver_name = member.kind.name().to_string();
                            added = true;
                            break;
                        }
                    }
                    if added {
                        cave_message.message = Some(format!(
                            "たからばこから {}を てにいれた！（{}）",
                            weapon.name(),
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
                            Some("もちものが いっぱいだ！".to_string());
                        // 取得失敗 → 宝箱は残す（取得済みに記録しない）
                        continue;
                    }
                }
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

/// メッセージ確認入力システム: 確認キーでメッセージをクリア
pub fn cave_message_input_system(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut cave_message: Option<ResMut<CaveMessageState>>,
) {
    let Some(ref mut cave_message) = cave_message else {
        return;
    };
    if cave_message.message.is_none() {
        return;
    }
    if input_ui::is_confirm_just_pressed(&keyboard) || input_ui::is_cancel_just_pressed(&keyboard) {
        cave_message.message = None;
        // フィールドメニューが同フレームで反応しないよう確認キーを消費
        input_ui::clear_confirm_just_pressed(&mut keyboard);
    }
}

/// メッセージ表示UIシステム
pub fn cave_message_display_system(
    mut commands: Commands,
    cave_message: Option<Res<CaveMessageState>>,
    existing_ui: Query<Entity, With<CaveMessageUI>>,
) {
    let Some(cave_message) = cave_message else {
        // リソースがない場合はUI削除
        for entity in &existing_ui {
            commands.entity(entity).despawn();
        }
        return;
    };

    match &cave_message.message {
        Some(msg) => {
            // 既存UIがあれば削除して再生成
            for entity in &existing_ui {
                commands.entity(entity).despawn();
            }

            commands
                .spawn((
                    CaveMessageUI,
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(88.0),
                        left: Val::Px(16.0),
                        right: Val::Px(16.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                    GlobalZIndex(60),
                ))
                .with_child((
                    Text::new(msg.clone()),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
        }
        None => {
            for entity in &existing_ui {
                commands.entity(entity).despawn();
            }
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
