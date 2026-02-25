use bevy::prelude::*;

use party::ItemEffect;

use super::scene::{
    enemy_display_names, BattleGameState, BattlePhase, BattleSceneRoot, BattleUIState,
    CommandScrollDown, CommandScrollUp, EnemySprite,
};

/// 敵名ラベルのマーカー
#[derive(Component)]
pub struct EnemyNameLabel {
    pub index: usize,
}

/// メッセージ表示テキストのマーカー
#[derive(Component)]
pub struct MessageText;

/// パーティメンバーHP表示テキストのマーカー
#[derive(Component)]
pub struct PartyMemberHpText {
    pub index: usize,
}

/// コマンドカーソルのマーカー（indexでどのコマンドかを識別）
#[derive(Component)]
pub struct CommandCursor {
    pub index: usize,
}

/// パーティメンバーHPバーの前景（塗り部分）マーカー
#[derive(Component)]
pub struct PartyMemberHpBarFill {
    pub index: usize,
}

/// パーティメンバーMP表示テキストのマーカー
#[derive(Component)]
pub struct PartyMemberMpText {
    pub index: usize,
}

/// パーティメンバー名テキストのマーカー（味方選択時のハイライト用）
#[derive(Component)]
pub struct PartyMemberNameText {
    pub index: usize,
}

/// ターゲットカーソル(▼)のマーカー
#[derive(Component)]
pub struct TargetCursor {
    pub index: usize,
}

/// HP割合に応じた色を返す（>50%=緑, >25%=黄, それ以下=赤）
fn hp_bar_color(ratio: f32) -> Color {
    if ratio > 0.5 {
        Color::srgb(0.2, 0.8, 0.2)
    } else if ratio > 0.25 {
        Color::srgb(0.9, 0.8, 0.1)
    } else {
        Color::srgb(0.9, 0.2, 0.2)
    }
}

const COMMAND_COLOR_SELECTED: Color = Color::srgb(1.0, 0.9, 0.2);
const COMMAND_COLOR_UNSELECTED: Color = Color::srgb(0.6, 0.6, 0.6);
const COMMAND_COLOR_DISABLED: Color = Color::srgb(0.35, 0.35, 0.35);
const VISIBLE_ITEMS: usize = 6;

fn scroll_offset(cursor: usize, total: usize, visible: usize) -> usize {
    if total <= visible {
        return 0;
    }
    let half = visible / 2;
    cursor.saturating_sub(half).min(total - visible)
}

/// 戦闘画面の表示を更新するシステム
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn battle_display_system(
    game_state: Res<BattleGameState>,
    ui_state: Res<BattleUIState>,
    mut enemy_name_query: Query<(&EnemyNameLabel, &mut Text, &mut Visibility), (Without<MessageText>, Without<PartyMemberHpText>, Without<CommandCursor>, Without<EnemySprite>, Without<TargetCursor>, Without<PartyMemberMpText>, Without<PartyMemberNameText>)>,
    mut enemy_sprite_query: Query<(&EnemySprite, &mut Visibility), (Without<EnemyNameLabel>, Without<MessageText>, Without<PartyMemberHpText>, Without<CommandCursor>, Without<TargetCursor>, Without<PartyMemberMpText>, Without<PartyMemberNameText>)>,
    mut message_query: Query<&mut Text, (With<MessageText>, Without<EnemyNameLabel>, Without<PartyMemberHpText>, Without<CommandCursor>, Without<PartyMemberMpText>, Without<PartyMemberNameText>)>,
    mut party_hp_query: Query<(&PartyMemberHpText, &mut Text), (Without<EnemyNameLabel>, Without<MessageText>, Without<CommandCursor>, Without<PartyMemberMpText>, Without<PartyMemberNameText>)>,
    mut party_mp_query: Query<(&PartyMemberMpText, &mut Text), (Without<EnemyNameLabel>, Without<MessageText>, Without<CommandCursor>, Without<PartyMemberHpText>, Without<PartyMemberNameText>)>,
    mut party_name_query: Query<(&PartyMemberNameText, &mut Text, &mut TextColor), (Without<EnemyNameLabel>, Without<MessageText>, Without<CommandCursor>, Without<PartyMemberHpText>, Without<PartyMemberMpText>)>,
    mut party_bar_query: Query<(&PartyMemberHpBarFill, &mut Node, &mut BackgroundColor)>,
    mut command_query: Query<(&CommandCursor, &mut Text, &mut TextColor, &mut Visibility, &mut Node), (Without<EnemyNameLabel>, Without<MessageText>, Without<PartyMemberHpText>, Without<PartyMemberMpText>, Without<PartyMemberNameText>, Without<CommandScrollUp>, Without<CommandScrollDown>, Without<PartyMemberHpBarFill>)>,
    mut target_cursor_query: Query<(&TargetCursor, &mut Visibility), (Without<EnemySprite>, Without<EnemyNameLabel>, Without<CommandCursor>, Without<CommandScrollUp>, Without<CommandScrollDown>)>,
    mut cmd_scroll_up_query: Query<(&mut Visibility, &mut Node), (With<CommandScrollUp>, Without<CommandCursor>, Without<CommandScrollDown>, Without<EnemyNameLabel>, Without<EnemySprite>, Without<TargetCursor>, Without<PartyMemberHpBarFill>)>,
    mut cmd_scroll_down_query: Query<(&mut Visibility, &mut Node), (With<CommandScrollDown>, Without<CommandCursor>, Without<CommandScrollUp>, Without<EnemyNameLabel>, Without<EnemySprite>, Without<TargetCursor>, Without<PartyMemberHpBarFill>)>,
) {
    let display_names = enemy_display_names(&game_state.state.enemies);
    let enemy_count = game_state.state.enemies.len();

    // 敵スプライト表示/非表示（hidden_enemiesで制御）
    for (sprite, mut vis) in &mut enemy_sprite_query {
        let hidden = ui_state.hidden_enemies.get(sprite.index).copied().unwrap_or(true);
        if sprite.index < enemy_count && !hidden {
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    // 敵名ラベル更新
    for (label, mut text, mut vis) in &mut enemy_name_query {
        let hidden = ui_state.hidden_enemies.get(label.index).copied().unwrap_or(true);
        if label.index < enemy_count && !hidden {
            **text = display_names[label.index].clone();
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    // パーティHP更新（表示用HPを使用）
    for (hp_text, mut text) in &mut party_hp_query {
        if hp_text.index < game_state.state.party.len() {
            let display_hp = ui_state.display_party_hp.get(hp_text.index).copied().unwrap_or(0);
            let max_hp = game_state.state.party[hp_text.index].stats.max_hp;
            **text = format!("HP:{}/{}", display_hp, max_hp);
        }
    }

    // パーティMP更新（表示用MPを使用）
    for (mp_text, mut text) in &mut party_mp_query {
        if mp_text.index < game_state.state.party.len() {
            let display_mp = ui_state.display_party_mp.get(mp_text.index).copied().unwrap_or(0);
            let max_mp = game_state.state.party[mp_text.index].stats.max_mp;
            **text = format!("MP:{}/{}", display_mp, max_mp);
        }
    }

    // パーティ名前+レベル更新＆ハイライト（味方ターゲット選択時）
    let is_ally_target_select = matches!(ui_state.phase, BattlePhase::AllyTargetSelect { .. });
    let alive_party = game_state.state.alive_party_indices();
    for (name_text, mut text, mut color) in &mut party_name_query {
        if name_text.index < game_state.state.party.len() {
            let member = &game_state.state.party[name_text.index];
            **text = format!("{} Lv.{}", member.kind.name(), member.level);
        }
        if is_ally_target_select && alive_party.get(ui_state.ally_target_offset) == Some(&name_text.index) {
            *color = TextColor(COMMAND_COLOR_SELECTED); // 黄色ハイライト
        } else {
            *color = TextColor(Color::WHITE);
        }
    }

    // パーティHPバー更新（表示用HPを使用）
    for (bar, mut node, mut bg) in &mut party_bar_query {
        if bar.index < game_state.state.party.len() {
            let display_hp = ui_state.display_party_hp.get(bar.index).copied().unwrap_or(0);
            let max_hp = game_state.state.party[bar.index].stats.max_hp;
            let ratio = display_hp as f32 / max_hp as f32;
            node.width = Val::Percent(ratio * 100.0);
            *bg = BackgroundColor(hp_bar_color(ratio));
        }
    }

    // ターゲットカーソル更新
    let is_target_select = matches!(ui_state.phase, BattlePhase::TargetSelect { .. });
    let alive_enemies = game_state.state.alive_enemy_indices();
    for (cursor, mut vis) in &mut target_cursor_query {
        if is_target_select && alive_enemies.get(ui_state.target_offset) == Some(&cursor.index) {
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    // メッセージ更新
    for mut text in &mut message_query {
        match &ui_state.phase {
            BattlePhase::CommandSelect { member_index } => {
                let member_name = game_state.state.party[*member_index].kind.name();
                **text = format!("{}の コマンド？", member_name);
            }
            BattlePhase::SpellSelect { member_index } => {
                let member_name = game_state.state.party[*member_index].kind.name();
                **text = format!("{}は どの じゅもんを つかう？", member_name);
            }
            BattlePhase::ItemSelect { member_index } => {
                let member_name = game_state.state.party[*member_index].kind.name();
                **text = format!("{}は どの どうぐを つかう？", member_name);
            }
            BattlePhase::TargetSelect { .. } => {
                if ui_state.pending_spell.is_some() {
                    **text = "だれに つかう？".to_string();
                } else {
                    **text = "だれに こうげきする？".to_string();
                }
            }
            BattlePhase::AllyTargetSelect { .. } => {
                **text = "だれに つかう？".to_string();
            }
            BattlePhase::ShowMessage { messages, index } => {
                if let Some(msg) = messages.get(*index) {
                    **text = msg.clone();
                }
            }
            BattlePhase::BattleOver { message } => {
                **text = message.clone();
            }
        }
    }

    // コマンド/呪文/アイテム表示更新
    let is_spell_select = matches!(ui_state.phase, BattlePhase::SpellSelect { .. });
    let is_item_select = matches!(ui_state.phase, BattlePhase::ItemSelect { .. });
    let show_commands = matches!(ui_state.phase, BattlePhase::CommandSelect { .. });

    // 呪文選択フェーズ用: クラス別呪文リストと現在のキャラのMP
    let (spell_list, current_member_mp) = match &ui_state.phase {
        BattlePhase::SpellSelect { member_index } => {
            let member = &game_state.state.party[*member_index];
            (battle::spell::available_spells(member.kind, member.level), member.stats.mp)
        }
        _ => (vec![], 0),
    };

    // コマンド選択フェーズ用: 呪文なしクラスかどうか、アイテムなしかどうか
    let (has_no_spells, has_no_items) = match &ui_state.phase {
        BattlePhase::CommandSelect { member_index } => (
            battle::spell::available_spells(game_state.state.party[*member_index].kind, game_state.state.party[*member_index].level).is_empty(),
            game_state.state.party[*member_index].inventory.is_empty(),
        ),
        _ => (false, false),
    };

    // アイテム選択フェーズ用: 所持アイテム一覧
    let owned_items = match &ui_state.phase {
        BattlePhase::ItemSelect { member_index } => {
            game_state.state.party[*member_index].inventory.owned_items()
        }
        _ => vec![],
    };

    let commands = ["たたかう", "じゅもん", "どうぐ", "にげる"];
    for (cursor, mut text, mut color, mut vis, mut cmd_node) in &mut command_query {
        if is_spell_select {
            // 呪文選択モード: CommandCursorを呪文名に差し替え（スクロール対応）
            let offset = scroll_offset(ui_state.selected_spell, spell_list.len(), VISIBLE_ITEMS);
            let data_index = offset + cursor.index;
            if cursor.index < VISIBLE_ITEMS && data_index < spell_list.len() {
                let spell = spell_list[data_index];
                let is_selected = data_index == ui_state.selected_spell;
                let can_use = current_member_mp >= spell.mp_cost();
                let prefix = if is_selected { "> " } else { "  " };
                **text = format!("{}{} ({})", prefix, spell.name(), spell.mp_cost());
                *color = if !can_use {
                    TextColor(Color::srgb(0.4, 0.4, 0.4)) // 灰色（使用不可）
                } else if is_selected {
                    TextColor(COMMAND_COLOR_SELECTED)
                } else {
                    TextColor(COMMAND_COLOR_UNSELECTED)
                };
                *vis = Visibility::Inherited;
            } else {
                *vis = Visibility::Hidden;
            }
        } else if is_item_select {
            // アイテム選択モード: CommandCursorをアイテム名+個数に差し替え（スクロール対応）
            let offset = scroll_offset(ui_state.selected_item, owned_items.len(), VISIBLE_ITEMS);
            let data_index = offset + cursor.index;
            if cursor.index < VISIBLE_ITEMS && data_index < owned_items.len() {
                let item = owned_items[data_index];
                let count = match &ui_state.phase {
                    BattlePhase::ItemSelect { member_index } => {
                        game_state.state.party[*member_index].inventory.count(item)
                    }
                    _ => 0,
                };
                let is_selected = data_index == ui_state.selected_item;
                let can_use = !matches!(item.effect(), ItemEffect::KeyItem | ItemEffect::Material);
                let prefix = if is_selected { "> " } else { "  " };
                **text = format!("{}{} x{}", prefix, item.name(), count);
                *color = if !can_use {
                    TextColor(COMMAND_COLOR_DISABLED)
                } else if is_selected {
                    TextColor(COMMAND_COLOR_SELECTED)
                } else {
                    TextColor(COMMAND_COLOR_UNSELECTED)
                };
                *vis = Visibility::Inherited;
            } else {
                *vis = Visibility::Hidden;
            }
        } else if show_commands {
            if cursor.index < commands.len() {
                let is_selected = cursor.index == ui_state.selected_command;
                let prefix = if is_selected { "> " } else { "  " };
                **text = format!("{}{}", prefix, commands[cursor.index]);
                // 呪文なしクラスは「じゅもん」を灰色表示、アイテムなしは「どうぐ」を灰色表示
                let is_disabled = (cursor.index == 1 && has_no_spells)
                    || (cursor.index == 2 && has_no_items);
                *color = if is_disabled {
                    TextColor(Color::srgb(0.4, 0.4, 0.4))
                } else if is_selected {
                    TextColor(COMMAND_COLOR_SELECTED)
                } else {
                    TextColor(COMMAND_COLOR_UNSELECTED)
                };
                *vis = Visibility::Inherited;
            }
        } else if cursor.index < commands.len() {
            **text = format!("  {}", commands[cursor.index]);
            *color = TextColor(COMMAND_COLOR_UNSELECTED);
            *vis = Visibility::Inherited;
        }
        // Display::NoneでレイアウトからもHiddenアイテムを除外
        cmd_node.display = if *vis == Visibility::Hidden { Display::None } else { Display::DEFAULT };
    }

    // コマンドスクロールインジケータ更新
    let (scroll_total, scroll_cursor) = if is_spell_select {
        (spell_list.len(), ui_state.selected_spell)
    } else if is_item_select {
        (owned_items.len(), ui_state.selected_item)
    } else {
        (0, 0)
    };

    if scroll_total > VISIBLE_ITEMS {
        let offset = scroll_offset(scroll_cursor, scroll_total, VISIBLE_ITEMS);
        for (mut vis, mut node) in &mut cmd_scroll_up_query {
            *vis = if offset > 0 { Visibility::Inherited } else { Visibility::Hidden };
            node.display = if *vis == Visibility::Hidden { Display::None } else { Display::DEFAULT };
        }
        for (mut vis, mut node) in &mut cmd_scroll_down_query {
            *vis = if offset + VISIBLE_ITEMS < scroll_total { Visibility::Inherited } else { Visibility::Hidden };
            node.display = if *vis == Visibility::Hidden { Display::None } else { Display::DEFAULT };
        }
    } else {
        for (mut vis, mut node) in &mut cmd_scroll_up_query {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }
        for (mut vis, mut node) in &mut cmd_scroll_down_query {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }
    }
}

/// 画面シェイクシステム: BattleSceneRootのStyleにleftオフセットを適用
pub fn battle_shake_system(
    time: Res<Time>,
    mut ui_state: ResMut<BattleUIState>,
    mut query: Query<&mut Node, With<BattleSceneRoot>>,
) {
    let Some(timer) = &mut ui_state.shake_timer else {
        return;
    };

    timer.tick(time.delta());

    let Ok(mut node) = query.single_mut() else {
        return;
    };

    if timer.just_finished() {
        node.left = Val::Px(0.0);
        ui_state.shake_timer = None;
        return;
    }

    // sin波(2往復) × 減衰(1→0)、振幅3px、X方向のみ
    let progress = timer.fraction();
    let decay = 1.0 - progress;
    let wave = (progress * 2.0 * std::f32::consts::TAU).sin();
    node.left = Val::Px(wave * 3.0 * decay);
}

/// 敵スプライト点滅システム: ダメージを受けた敵を高速で明滅させる
pub fn battle_blink_system(
    time: Res<Time>,
    mut ui_state: ResMut<BattleUIState>,
    mut query: Query<(&EnemySprite, &mut Visibility)>,
) {
    let (Some(enemy_index), Some(timer)) =
        (ui_state.blink_enemy, ui_state.blink_timer.as_mut())
    else {
        return;
    };

    timer.tick(time.delta());
    let finished = timer.just_finished();
    let elapsed = timer.elapsed_secs();

    if finished {
        ui_state.blink_timer = None;
        ui_state.blink_enemy = None;
    }

    // 0.05秒ごとに表示/非表示を切り替え（3回点滅）、完了時は表示に戻す
    let visible = finished || ((elapsed / 0.05) as u32).is_multiple_of(2);

    for (sprite, mut vis) in &mut query {
        if sprite.index == enemy_index {
            *vis = if visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}
