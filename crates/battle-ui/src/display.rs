use bevy::prelude::*;

use battle::EnemyKind;

use super::scene::{BattleGameState, BattlePhase, BattleSceneRoot, BattleUIState, EnemySprite};

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

/// 同種の敵にサフィックスを付与した表示名を生成
fn enemy_display_names(enemies: &[battle::Enemy]) -> Vec<String> {
    let mut kind_counts: std::collections::HashMap<EnemyKind, usize> =
        std::collections::HashMap::new();
    for e in enemies {
        *kind_counts.entry(e.kind).or_insert(0) += 1;
    }

    let suffixes = ['A', 'B', 'C', 'D'];
    let mut kind_indices: std::collections::HashMap<EnemyKind, usize> =
        std::collections::HashMap::new();

    enemies
        .iter()
        .map(|e| {
            let count = kind_counts[&e.kind];
            if count > 1 {
                let idx = kind_indices.entry(e.kind).or_insert(0);
                let suffix = suffixes.get(*idx).unwrap_or(&'?');
                *idx += 1;
                format!("{}{}", e.kind.name(), suffix)
            } else {
                e.kind.name().to_string()
            }
        })
        .collect()
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
    mut party_name_query: Query<(&PartyMemberNameText, &mut TextColor), (Without<EnemyNameLabel>, Without<MessageText>, Without<CommandCursor>, Without<PartyMemberHpText>, Without<PartyMemberMpText>)>,
    mut party_bar_query: Query<(&PartyMemberHpBarFill, &mut Node, &mut BackgroundColor)>,
    mut command_query: Query<(&CommandCursor, &mut Text, &mut TextColor, &mut Visibility), (Without<EnemyNameLabel>, Without<MessageText>, Without<PartyMemberHpText>, Without<PartyMemberMpText>, Without<PartyMemberNameText>)>,
    mut target_cursor_query: Query<(&TargetCursor, &mut Visibility), (Without<EnemySprite>, Without<EnemyNameLabel>, Without<CommandCursor>)>,
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

    // パーティ名前ハイライト（味方ターゲット選択時）
    let is_ally_target_select = matches!(ui_state.phase, BattlePhase::AllyTargetSelect { .. });
    for (name_text, mut color) in &mut party_name_query {
        if is_ally_target_select && name_text.index == ui_state.selected_ally_target {
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
    for (cursor, mut vis) in &mut target_cursor_query {
        if is_target_select
            && cursor.index < enemy_count
            && game_state.state.enemies[cursor.index].stats.is_alive()
            && cursor.index == ui_state.selected_target
        {
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

    // コマンド/呪文表示更新
    let spell_list = battle::spell::all_spells();
    let is_spell_select = matches!(ui_state.phase, BattlePhase::SpellSelect { .. });
    let show_commands = matches!(ui_state.phase, BattlePhase::CommandSelect { .. });

    // 呪文選択フェーズ用: 現在のキャラのMP
    let current_member_mp = match &ui_state.phase {
        BattlePhase::SpellSelect { member_index } => {
            game_state.state.party[*member_index].stats.mp
        }
        _ => 0,
    };

    let commands = ["たたかう", "じゅもん", "にげる"];
    for (cursor, mut text, mut color, mut vis) in &mut command_query {
        if is_spell_select {
            // 呪文選択モード: CommandCursorを呪文名に差し替え
            if cursor.index < spell_list.len() {
                let spell = spell_list[cursor.index];
                let is_selected = cursor.index == ui_state.selected_spell;
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
        } else if show_commands {
            if cursor.index < commands.len() {
                let is_selected = cursor.index == ui_state.selected_command;
                let prefix = if is_selected { "> " } else { "  " };
                **text = format!("{}{}", prefix, commands[cursor.index]);
                *color = if is_selected {
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
