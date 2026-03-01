use bevy::prelude::*;

use super::scene::{
    enemy_display_names, BattleGameState, BattlePhase, BattleSceneRoot, BattleUIState,
    EnemySprite,
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

/// パーティメンバーブロック値テキストのマーカー
#[derive(Component)]
pub struct PartyMemberBlockText {
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

/// 味方ターゲット選択時のハイライト色
const ALLY_TARGET_HIGHLIGHT: Color = Color::srgb(1.0, 0.9, 0.2);

/// CommandMenuキャッシュ更新システム（command_menu_display_systemの前に実行）
pub fn battle_update_menu_cache(
    game_state: Res<BattleGameState>,
    mut ui_state: ResMut<BattleUIState>,
) {
    ui_state.rebuild_cache(&game_state);
}

/// 戦闘画面のステータス表示を更新するシステム
/// コマンド/呪文/アイテムリスト表示は command_menu_display_system に委譲
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn battle_status_display_system(
    game_state: Res<BattleGameState>,
    ui_state: Res<BattleUIState>,
    mut enemy_name_query: Query<(&EnemyNameLabel, &mut Text, &mut Visibility), (Without<MessageText>, Without<PartyMemberHpText>, Without<EnemySprite>, Without<TargetCursor>, Without<PartyMemberMpText>, Without<PartyMemberNameText>)>,
    mut enemy_sprite_query: Query<(&EnemySprite, &mut Visibility), (Without<EnemyNameLabel>, Without<MessageText>, Without<PartyMemberHpText>, Without<TargetCursor>, Without<PartyMemberMpText>, Without<PartyMemberNameText>)>,
    mut message_query: Query<&mut Text, (With<MessageText>, Without<EnemyNameLabel>, Without<PartyMemberHpText>, Without<PartyMemberMpText>, Without<PartyMemberNameText>)>,
    mut party_hp_query: Query<(&PartyMemberHpText, &mut Text), (Without<EnemyNameLabel>, Without<MessageText>, Without<PartyMemberMpText>, Without<PartyMemberNameText>)>,
    mut party_mp_query: Query<(&PartyMemberMpText, &mut Text), (Without<EnemyNameLabel>, Without<MessageText>, Without<PartyMemberHpText>, Without<PartyMemberNameText>)>,
    mut party_name_query: Query<(&PartyMemberNameText, &mut Text, &mut TextColor), (Without<EnemyNameLabel>, Without<MessageText>, Without<PartyMemberHpText>, Without<PartyMemberMpText>)>,
    mut party_block_query: Query<(&PartyMemberBlockText, &mut Text, &mut Node), (Without<EnemyNameLabel>, Without<MessageText>, Without<PartyMemberHpText>, Without<PartyMemberMpText>, Without<PartyMemberNameText>, Without<PartyMemberHpBarFill>)>,
    mut party_bar_query: Query<(&PartyMemberHpBarFill, &mut Node, &mut BackgroundColor)>,
    mut target_cursor_query: Query<(&TargetCursor, &mut Visibility), (Without<EnemySprite>, Without<EnemyNameLabel>)>,
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
            *color = TextColor(ALLY_TARGET_HIGHLIGHT);
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

    // パーティブロック値更新（block > 0のときのみ表示）
    for (block_text, mut text, mut node) in &mut party_block_query {
        if block_text.index < game_state.state.party_buffs.len() {
            let block = game_state.state.party_buffs[block_text.index].block;
            if block > 0 {
                **text = format!("Block:{}", block);
                node.display = Display::DEFAULT;
            } else {
                node.display = Display::None;
            }
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
