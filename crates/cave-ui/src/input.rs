use bevy::prelude::*;

use cave::{try_cave_move, CaveMoveResult, TreasureContent};
use terrain::Terrain;

use app_state::{BossBattlePending, OpenedChests, PartyState, SceneState};
use movement_ui::{
    MovementBlockedEvent, MovementLocked, PendingMove, Player,
    PlayerMovedEvent, TileEnteredEvent, TilePosition,
};
use app_state::FieldMenuOpen;
use movement_ui::{ActiveMap, MovementState};

use world_ui::{MapModeState, TileTextures};

use crate::scene::{BossCaveState, BossEntity, CaveMessageState, CaveMessageUI, CaveTreasures, ChestEntity};

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

    if locked.is_some() {
        return;
    }

    // マップモード中は移動を無効化
    if map_mode_state.enabled {
        return;
    }

    // フィールド呪文メニュー中は移動を無効化
    if field_menu_open.is_some() {
        return;
    }

    // メッセージ表示中は移動を無効化
    if cave_message.is_some_and(|m| m.message.is_some()) {
        return;
    }

    let mut dx: i32 = 0;
    let mut dy: i32 = 0;

    let x_pressed = input_ui::is_x_pressed(&keyboard);
    let y_pressed = input_ui::is_y_pressed(&keyboard);
    let x_just_pressed = input_ui::is_x_just_pressed(&keyboard);
    let y_just_pressed = input_ui::is_y_just_pressed(&keyboard);

    if x_just_pressed && !y_pressed {
        move_state.first_axis = Some(true);
    } else if y_just_pressed && !x_pressed {
        move_state.first_axis = Some(false);
    } else if !x_pressed && !y_pressed {
        move_state.first_axis = None;
    }

    if input_ui::is_up_pressed(&keyboard) {
        dy = 1;
    }
    if input_ui::is_down_pressed(&keyboard) {
        dy = -1;
    }
    if input_ui::is_left_pressed(&keyboard) {
        dx = -1;
    }
    if input_ui::is_right_pressed(&keyboard) {
        dx = 1;
    }

    if dx == 0 && dy == 0 {
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = (0, 0);
        return;
    }

    let (first_dx, first_dy, pending_direction) = if dx != 0 && dy != 0 {
        match move_state.first_axis {
            Some(true) => (dx, 0, Some((0, dy))),
            Some(false) => (0, dy, Some((dx, 0))),
            None => (dx, 0, Some((0, dy))),
        }
    } else {
        (dx, dy, None)
    };

    let current_direction = (first_dx, first_dy);
    let direction_changed = current_direction != move_state.last_direction;

    let should_move = if direction_changed {
        move_state.is_repeating = false;
        move_state.initial_delay.reset();
        move_state.timer.reset();
        move_state.last_direction = current_direction;
        true
    } else if move_state.is_repeating {
        move_state.timer.tick(time.delta());
        move_state.timer.just_finished()
    } else {
        move_state.initial_delay.tick(time.delta());
        if move_state.initial_delay.just_finished() {
            move_state.is_repeating = true;
            move_state.timer.reset();
            true
        } else {
            false
        }
    };

    if !should_move {
        return;
    }

    match try_cave_move(
        tile_pos.x,
        tile_pos.y,
        first_dx,
        first_dy,
        &active_map.grid,
        active_map.width,
        active_map.height,
    ) {
        CaveMoveResult::Moved { new_x, new_y } => {
            tile_pos.x = new_x;
            tile_pos.y = new_y;
            moved_events.write(PlayerMovedEvent {
                entity,
                direction: (first_dx, first_dy),
            });
            if let Some(dir) = pending_direction {
                commands.entity(entity).insert(PendingMove { direction: dir });
            }
        }
        CaveMoveResult::Blocked => {
            blocked_events.write(MovementBlockedEvent {
                entity,
                direction: (first_dx, first_dy),
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

            match try_cave_move(
                tile_pos.x,
                tile_pos.y,
                dx,
                dy,
                &active_map.grid,
                active_map.width,
                active_map.height,
            ) {
                CaveMoveResult::Moved { new_x, new_y } => {
                    tile_pos.x = new_x;
                    tile_pos.y = new_y;
                    moved_events.write(PlayerMovedEvent {
                        entity,
                        direction: (dx, dy),
                    });
                }
                CaveMoveResult::Blocked => {
                    blocked_events.write(MovementBlockedEvent {
                        entity,
                        direction: (dx, dy),
                    });
                    // ワープゾーン上でPendingMoveがブロックされた場合にも
                    // 梯子判定を行う（MovementLockedはバウンスが解除）
                    if tile_pos.x < active_map.width && tile_pos.y < active_map.height
                        && active_map.terrain_at(tile_pos.x, tile_pos.y) == Terrain::Ladder
                    {
                        next_state.set(SceneState::Exploring);
                    }
                }
            }
        } else {
            commands.entity(entity).remove::<MovementLocked>();
            let terrain = active_map.terrain_at(tile_pos.x, tile_pos.y);
            if terrain == Terrain::Ladder {
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
    mut events: MessageReader<TileEnteredEvent>,
    player_query: Query<&TilePosition, With<Player>>,
    cave_treasures: Option<Res<CaveTreasures>>,
    mut opened_chests: ResMut<OpenedChests>,
    mut party_state: ResMut<PartyState>,
    mut cave_message: ResMut<CaveMessageState>,
    mut chest_query: Query<(&ChestEntity, &mut Sprite)>,
    tile_textures: Res<TileTextures>,
) {
    let Some(cave_treasures) = cave_treasures else {
        return;
    };

    for _event in events.read() {
        let Ok(tile_pos) = player_query.single() else {
            continue;
        };

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
                    // 武器は装備スロットなので必ず取得可能
                    // 装備なしのメンバーがいれば自動装備
                    let mut equipped_to = None;
                    for member in &mut party_state.members {
                        if member.equipment.weapon.is_none() {
                            member.equipment.equip_weapon(weapon);
                            equipped_to = Some(member.kind.name().to_string());
                            break;
                        }
                    }
                    if let Some(name) = equipped_to {
                        cave_message.message = Some(format!(
                            "たからばこから {}を てにいれた！\n{}が そうびした！",
                            weapon.name(),
                            name,
                        ));
                    } else {
                        // 全員装備済み → 先頭メンバーの武器を上書き
                        if let Some(member) = party_state.members.first_mut() {
                            let old = member.equipment.equip_weapon(weapon);
                            cave_message.message = Some(format!(
                                "たからばこから {}を てにいれた！\n{}が そうびした！（{}を はずした）",
                                weapon.name(),
                                member.kind.name(),
                                old.map_or("なし", |w| w.name()),
                            ));
                        }
                    }
                    // 取得済みに記録
                    opened_chests
                        .chests
                        .entry(cave_pos)
                        .or_default()
                        .insert(i);
                }
            }

            // 宝箱スプライトを開いた状態に変更
            for (chest, mut sprite) in &mut chest_query {
                if chest.treasure_index == i {
                    sprite.image = tile_textures.chest_open.clone();
                }
            }
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
    if input_ui::is_confirm_just_pressed(&keyboard) {
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

