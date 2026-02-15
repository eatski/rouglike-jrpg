use bevy::prelude::*;

use battle::{ActorId, BattleAction, SpellKind, TargetId, TurnRandomFactors, TurnResult};

use app_state::BattleState;

use super::scene::{BattleGameState, BattlePhase, BattleUIState, MessageEffect};

/// 戦闘中の入力処理システム
pub fn battle_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<BattleGameState>,
    mut ui_state: ResMut<BattleUIState>,
    mut next_state: ResMut<NextState<BattleState>>,
) {
    match ui_state.phase.clone() {
        BattlePhase::CommandSelect { member_index } => {
            handle_command_select(&keyboard, &mut game_state, &mut ui_state, member_index);
        }
        BattlePhase::SpellSelect { member_index } => {
            handle_spell_select(&keyboard, &game_state, &mut ui_state, member_index);
        }
        BattlePhase::TargetSelect { member_index } => {
            handle_target_select(&keyboard, &mut game_state, &mut ui_state, member_index);
        }
        BattlePhase::AllyTargetSelect { member_index } => {
            handle_ally_target_select(&keyboard, &mut game_state, &mut ui_state, member_index);
        }
        BattlePhase::ShowMessage { messages, index } => {
            handle_show_message(&keyboard, &game_state, &mut ui_state, index, messages.len());
        }
        BattlePhase::BattleOver { .. } => {
            handle_battle_over(&keyboard, &mut next_state);
        }
    }
}

fn handle_command_select(
    keyboard: &ButtonInput<KeyCode>,
    game_state: &mut BattleGameState,
    ui_state: &mut BattleUIState,
    member_index: usize,
) {
    // 上下でカーソル移動 (0=たたかう, 1=じゅもん, 2=にげる)
    if input_ui::is_up_just_pressed(keyboard) && ui_state.selected_command > 0 {
        ui_state.selected_command -= 1;
    }
    if input_ui::is_down_just_pressed(keyboard) && ui_state.selected_command < 2 {
        ui_state.selected_command += 1;
    }

    // キャンセル: 前のメンバーに戻る
    if input_ui::is_cancel_just_pressed(keyboard) {
        let prev = find_prev_alive_member(game_state, member_index);
        if let Some(prev_idx) = prev {
            ui_state.pending_commands.remove(prev_idx);
            ui_state.selected_command = 0;
            ui_state.phase = BattlePhase::CommandSelect {
                member_index: prev_idx,
            };
        }
        return;
    }

    // 決定
    if input_ui::is_confirm_just_pressed(keyboard) {
        match ui_state.selected_command {
            0 => {
                // たたかう → ターゲット選択へ
                let first_alive = game_state.state.alive_enemy_indices();
                ui_state.selected_target = first_alive.first().copied().unwrap_or(0);
                ui_state.pending_spell = None;
                ui_state.phase = BattlePhase::TargetSelect { member_index };
            }
            1 => {
                // じゅもん → 呪文がないクラスは遷移しない
                let member_kind = game_state.state.party[member_index].kind;
                let spells = battle::spell::available_spells(member_kind);
                if spells.is_empty() {
                    return;
                }
                ui_state.selected_spell = 0;
                ui_state.phase = BattlePhase::SpellSelect { member_index };
            }
            _ => {
                // にげる → 全員Flee確定、即実行
                ui_state.pending_commands.clear();
                for i in 0..game_state.state.party.len() {
                    ui_state.pending_commands.set(i, BattleAction::Flee);
                }
                execute_turn(game_state, ui_state);
            }
        }
        ui_state.selected_command = 0;
    }
}

fn handle_spell_select(
    keyboard: &ButtonInput<KeyCode>,
    game_state: &BattleGameState,
    ui_state: &mut BattleUIState,
    member_index: usize,
) {
    let member_kind = game_state.state.party[member_index].kind;
    let spells = battle::spell::available_spells(member_kind);
    let spell_count = spells.len();

    // 上下でカーソル移動
    if input_ui::is_up_just_pressed(keyboard) && ui_state.selected_spell > 0 {
        ui_state.selected_spell -= 1;
    }
    if input_ui::is_down_just_pressed(keyboard)
        && ui_state.selected_spell < spell_count - 1
    {
        ui_state.selected_spell += 1;
    }

    // キャンセル: コマンド選択に戻る
    if input_ui::is_cancel_just_pressed(keyboard) {
        ui_state.phase = BattlePhase::CommandSelect { member_index };
        return;
    }

    // 決定
    if input_ui::is_confirm_just_pressed(keyboard) {
        let spell = spells[ui_state.selected_spell];
        let member_mp = game_state.state.party[member_index].stats.mp;

        // MP不足チェック
        if member_mp < spell.mp_cost() {
            return; // MP不足なら何もしない
        }

        ui_state.pending_spell = Some(spell);

        if spell.is_offensive() {
            // 攻撃呪文 → 敵選択へ
            let first_alive = game_state.state.alive_enemy_indices();
            ui_state.selected_target = first_alive.first().copied().unwrap_or(0);
            ui_state.phase = BattlePhase::TargetSelect { member_index };
        } else {
            // 回復呪文 → 味方選択へ
            let first_alive = game_state.state.alive_party_indices();
            ui_state.selected_ally_target = first_alive.first().copied().unwrap_or(0);
            ui_state.phase = BattlePhase::AllyTargetSelect { member_index };
        }
    }
}

fn handle_target_select(
    keyboard: &ButtonInput<KeyCode>,
    game_state: &mut BattleGameState,
    ui_state: &mut BattleUIState,
    member_index: usize,
) {
    let alive_enemies = game_state.state.alive_enemy_indices();
    if alive_enemies.is_empty() {
        return;
    }

    // 左右でターゲット切り替え
    if input_ui::is_left_just_pressed(keyboard) {
        let current_pos = alive_enemies
            .iter()
            .position(|&i| i == ui_state.selected_target)
            .unwrap_or(0);
        if current_pos > 0 {
            ui_state.selected_target = alive_enemies[current_pos - 1];
        }
    }
    if input_ui::is_right_just_pressed(keyboard) {
        let current_pos = alive_enemies
            .iter()
            .position(|&i| i == ui_state.selected_target)
            .unwrap_or(0);
        if current_pos < alive_enemies.len() - 1 {
            ui_state.selected_target = alive_enemies[current_pos + 1];
        }
    }

    // キャンセル: pending_spellがあれば呪文選択に戻る、なければコマンド選択に戻る
    if input_ui::is_cancel_just_pressed(keyboard) {
        if ui_state.pending_spell.is_some() {
            ui_state.pending_spell = None;
            ui_state.phase = BattlePhase::SpellSelect { member_index };
        } else {
            ui_state.phase = BattlePhase::CommandSelect { member_index };
        }
        return;
    }

    // 決定
    if input_ui::is_confirm_just_pressed(keyboard) {
        let target = TargetId::Enemy(ui_state.selected_target);

        if let Some(spell) = ui_state.pending_spell.take() {
            // 呪文ターゲット決定
            ui_state
                .pending_commands
                .set(member_index, BattleAction::Spell { spell, target });
        } else {
            // 通常攻撃
            ui_state
                .pending_commands
                .set(member_index, BattleAction::Attack { target });
        }

        // 次の生存メンバーを探す
        let next = find_next_alive_member(game_state, member_index);
        if let Some(next_idx) = next {
            ui_state.selected_command = 0;
            ui_state.phase = BattlePhase::CommandSelect {
                member_index: next_idx,
            };
        } else {
            // 全員入力完了 → ターン実行
            execute_turn(game_state, ui_state);
        }
    }
}

fn handle_ally_target_select(
    keyboard: &ButtonInput<KeyCode>,
    game_state: &mut BattleGameState,
    ui_state: &mut BattleUIState,
    member_index: usize,
) {
    let alive_party = game_state.state.alive_party_indices();
    if alive_party.is_empty() {
        return;
    }

    // 左右で味方ターゲット切り替え
    if input_ui::is_left_just_pressed(keyboard) {
        let current_pos = alive_party
            .iter()
            .position(|&i| i == ui_state.selected_ally_target)
            .unwrap_or(0);
        if current_pos > 0 {
            ui_state.selected_ally_target = alive_party[current_pos - 1];
        }
    }
    if input_ui::is_right_just_pressed(keyboard) {
        let current_pos = alive_party
            .iter()
            .position(|&i| i == ui_state.selected_ally_target)
            .unwrap_or(0);
        if current_pos < alive_party.len() - 1 {
            ui_state.selected_ally_target = alive_party[current_pos + 1];
        }
    }

    // キャンセル: 呪文選択に戻る
    if input_ui::is_cancel_just_pressed(keyboard) {
        ui_state.pending_spell = None;
        ui_state.phase = BattlePhase::SpellSelect { member_index };
        return;
    }

    // 決定
    if input_ui::is_confirm_just_pressed(keyboard) {
        let target = TargetId::Party(ui_state.selected_ally_target);

        if let Some(spell) = ui_state.pending_spell.take() {
            ui_state
                .pending_commands
                .set(member_index, BattleAction::Spell { spell, target });
        }

        // 次の生存メンバーを探す
        let next = find_next_alive_member(game_state, member_index);
        if let Some(next_idx) = next {
            ui_state.selected_command = 0;
            ui_state.phase = BattlePhase::CommandSelect {
                member_index: next_idx,
            };
        } else {
            // 全員入力完了 → ターン実行
            execute_turn(game_state, ui_state);
        }
    }
}

/// 次の生存パーティメンバーを探す
fn find_next_alive_member(game_state: &BattleGameState, current: usize) -> Option<usize> {
    let alive = game_state.state.alive_party_indices();
    alive.into_iter().find(|&i| i > current)
}

/// 前の生存パーティメンバーを探す
fn find_prev_alive_member(game_state: &BattleGameState, current: usize) -> Option<usize> {
    let alive = game_state.state.alive_party_indices();
    alive.into_iter().rev().find(|&i| i < current)
}

/// ターンを実行してメッセージフェーズに遷移
fn execute_turn(game_state: &mut BattleGameState, ui_state: &mut BattleUIState) {
    let party_count = game_state.state.party.len();
    let enemy_count = game_state.state.enemies.len();
    let total_actors = party_count + enemy_count;

    let damage_randoms: Vec<f32> = (0..total_actors)
        .map(|_| 0.8 + rand::random::<f32>() * 0.4)
        .collect();
    let flee_random: f32 = rand::random();

    let random_factors = TurnRandomFactors {
        damage_randoms,
        flee_random,
    };

    // ターン実行前のパーティHP/MP状態をスナップショット
    let pre_party_hp: Vec<i32> = game_state.state.party.iter().map(|m| m.stats.hp).collect();
    let pre_party_mp: Vec<i32> = game_state.state.party.iter().map(|m| m.stats.mp).collect();

    let results = game_state
        .state
        .execute_turn(&ui_state.pending_commands.to_commands(), &random_factors);

    ui_state.pending_commands.clear();

    let (messages, effects) =
        results_to_messages(&results, &game_state.state, &pre_party_hp, &pre_party_mp);
    ui_state.message_effects = effects;

    if messages.is_empty() {
        // メッセージがない場合、表示HPを実際のHPに同期
        sync_display_hp(&game_state.state, ui_state);
        if game_state.state.is_over() {
            ui_state.phase = BattlePhase::BattleOver {
                message: "".to_string(),
            };
        } else {
            let first_alive = game_state.state.alive_party_indices();
            ui_state.phase = BattlePhase::CommandSelect {
                member_index: first_alive.first().copied().unwrap_or(0),
            };
        }
        return;
    }

    // 最初のメッセージ(index=0)に対応するエフェクトを処理
    process_message_effects(ui_state, 0);

    if game_state.state.is_over() {
        let last_msg = messages.last().cloned().unwrap_or_default();
        if messages.len() == 1 {
            ui_state.phase = BattlePhase::BattleOver { message: last_msg };
        } else {
            ui_state.phase = BattlePhase::ShowMessage {
                messages,
                index: 0,
            };
        }
    } else {
        ui_state.phase = BattlePhase::ShowMessage {
            messages,
            index: 0,
        };
    }
}

/// TurnResult列をメッセージ文字列列とMessageEffect列に変換
///
/// pre_party_hp/mp: ターン実行前のパーティHP/MPスナップショット。
/// 各攻撃/呪文メッセージにパーティHP/MP変化のエフェクトを紐付ける。
fn results_to_messages(
    results: &[TurnResult],
    state: &battle::BattleState,
    pre_party_hp: &[i32],
    pre_party_mp: &[i32],
) -> (Vec<String>, Vec<(usize, MessageEffect)>) {
    let enemy_names = enemy_display_names(&state.enemies);
    let mut messages = Vec::new();
    let mut effects: Vec<(usize, MessageEffect)> = Vec::new();

    // パーティHP/MPの「現在の表示値」を追跡
    let mut running_party_hp: Vec<i32> = pre_party_hp.to_vec();
    let mut running_party_mp: Vec<i32> = pre_party_mp.to_vec();

    for result in results {
        match result {
            TurnResult::Attack {
                attacker,
                target,
                damage,
            } => {
                let attacker_name = actor_name(attacker, state, &enemy_names);
                let target_name = target_name_str(target, state, &enemy_names);
                let msg_index = messages.len();
                messages.push(format!(
                    "{}の こうげき！ {}に {}ダメージ！",
                    attacker_name, target_name, damage
                ));

                // ターゲットに応じたエフェクトを紐付け
                match target {
                    TargetId::Enemy(i) => {
                        effects.push((
                            msg_index,
                            MessageEffect::BlinkEnemy { enemy_index: *i },
                        ));
                    }
                    TargetId::Party(i) => {
                        effects.push((msg_index, MessageEffect::Shake));
                        running_party_hp[*i] = (running_party_hp[*i] - damage).max(0);
                        effects.push((
                            msg_index,
                            MessageEffect::UpdatePartyHp {
                                member_index: *i,
                                new_hp: running_party_hp[*i],
                            },
                        ));
                    }
                }
            }
            TurnResult::SpellDamage {
                caster,
                spell,
                target,
                damage,
            } => {
                let caster_name = actor_name(caster, state, &enemy_names);
                let target_name = target_name_str(target, state, &enemy_names);
                let msg_index = messages.len();
                messages.push(format!(
                    "{}は {}を となえた！ {}に {}ダメージ！",
                    caster_name,
                    spell.name(),
                    target_name,
                    damage
                ));

                // キャスターのMP更新エフェクト
                if let ActorId::Party(ci) = caster {
                    running_party_mp[*ci] = (running_party_mp[*ci] - spell.mp_cost()).max(0);
                    effects.push((
                        msg_index,
                        MessageEffect::UpdatePartyMp {
                            member_index: *ci,
                            new_mp: running_party_mp[*ci],
                        },
                    ));
                }

                if let TargetId::Enemy(i) = target {
                    effects.push((
                        msg_index,
                        MessageEffect::BlinkEnemy { enemy_index: *i },
                    ));
                }
            }
            TurnResult::Healed {
                caster,
                target,
                amount,
            } => {
                let caster_name = actor_name(caster, state, &enemy_names);
                let target_name = target_name_str(target, state, &enemy_names);
                let msg_index = messages.len();
                messages.push(format!(
                    "{}は ヒールを となえた！ {}の HPが {}かいふく！",
                    caster_name, target_name, amount
                ));

                // キャスターのMP更新エフェクト
                if let ActorId::Party(ci) = caster {
                    running_party_mp[*ci] =
                        (running_party_mp[*ci] - SpellKind::Heal.mp_cost()).max(0);
                    effects.push((
                        msg_index,
                        MessageEffect::UpdatePartyMp {
                            member_index: *ci,
                            new_mp: running_party_mp[*ci],
                        },
                    ));
                }

                // ターゲットのHP更新エフェクト
                if let TargetId::Party(pi) = target {
                    let max_hp = state.party[*pi].stats.max_hp;
                    running_party_hp[*pi] = (running_party_hp[*pi] + amount).min(max_hp);
                    effects.push((
                        msg_index,
                        MessageEffect::UpdatePartyHp {
                            member_index: *pi,
                            new_hp: running_party_hp[*pi],
                        },
                    ));
                }
            }
            TurnResult::Defeated { target } => {
                let msg_index = messages.len();
                let name = target_name_str(target, state, &enemy_names);
                match target {
                    TargetId::Enemy(i) => {
                        messages.push(format!("{}を たおした！", name));
                        effects.push((
                            msg_index,
                            MessageEffect::HideEnemy { enemy_index: *i },
                        ));
                    }
                    TargetId::Party(_) => {
                        messages.push(format!("{}は たおれた...", name));
                    }
                }
            }
            TurnResult::Fled => {
                messages.push("うまく にげきれた！".to_string());
            }
            TurnResult::FleeFailed => {
                messages.push("にげられなかった！".to_string());
            }
        }
    }

    (messages, effects)
}

/// 指定メッセージindexに対応するエフェクトを処理し、表示状態を更新する
fn process_message_effects(ui_state: &mut BattleUIState, message_index: usize) {
    // message_effectsからcloneして取り出す（borrowの競合を避けるため）
    let effects: Vec<MessageEffect> = ui_state
        .message_effects
        .iter()
        .filter(|(idx, _): &&(usize, MessageEffect)| *idx == message_index)
        .map(|(_, effect)| effect.clone())
        .collect();

    for effect in effects {
        match effect {
            MessageEffect::UpdatePartyHp {
                member_index,
                new_hp,
            } => {
                if let Some(hp) = ui_state.display_party_hp.get_mut(member_index) {
                    *hp = new_hp;
                }
            }
            MessageEffect::HideEnemy { enemy_index } => {
                if let Some(hidden) = ui_state.hidden_enemies.get_mut(enemy_index) {
                    *hidden = true;
                }
            }
            MessageEffect::Shake => {
                ui_state.shake_timer =
                    Some(Timer::from_seconds(0.15, TimerMode::Once));
            }
            MessageEffect::BlinkEnemy { enemy_index } => {
                ui_state.blink_timer =
                    Some(Timer::from_seconds(0.3, TimerMode::Once));
                ui_state.blink_enemy = Some(enemy_index);
            }
            MessageEffect::UpdatePartyMp {
                member_index,
                new_mp,
            } => {
                if let Some(mp) = ui_state.display_party_mp.get_mut(member_index) {
                    *mp = new_mp;
                }
            }
        }
    }
}

/// 表示HP/MPを実際のゲーム状態に同期する
fn sync_display_hp(state: &battle::BattleState, ui_state: &mut BattleUIState) {
    for (i, member) in state.party.iter().enumerate() {
        if let Some(hp) = ui_state.display_party_hp.get_mut(i) {
            *hp = member.stats.hp;
        }
        if let Some(mp) = ui_state.display_party_mp.get_mut(i) {
            *mp = member.stats.mp;
        }
    }
}

fn actor_name(
    actor: &ActorId,
    state: &battle::BattleState,
    enemy_names: &[String],
) -> String {
    match actor {
        ActorId::Party(i) => state.party[*i].kind.name().to_string(),
        ActorId::Enemy(i) => enemy_names.get(*i).cloned().unwrap_or_default(),
    }
}

fn target_name_str(
    target: &TargetId,
    state: &battle::BattleState,
    enemy_names: &[String],
) -> String {
    match target {
        TargetId::Enemy(i) => enemy_names.get(*i).cloned().unwrap_or_default(),
        TargetId::Party(i) => state.party[*i].kind.name().to_string(),
    }
}

/// 同種の敵にサフィックスを付与した表示名を生成
fn enemy_display_names(enemies: &[battle::Enemy]) -> Vec<String> {
    use battle::EnemyKind;
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

fn handle_show_message(
    keyboard: &ButtonInput<KeyCode>,
    game_state: &BattleGameState,
    ui_state: &mut BattleUIState,
    index: usize,
    len: usize,
) {
    if input_ui::is_confirm_just_pressed(keyboard) {
        let next_index = index + 1;
        if next_index >= len {
            // 全メッセージ表示完了 — 表示HPを実際のHPに同期
            sync_display_hp(&game_state.state, ui_state);
            if game_state.state.is_over() {
                if let BattlePhase::ShowMessage { messages, .. } = &ui_state.phase {
                    let last_msg = messages.last().cloned().unwrap_or_default();
                    ui_state.phase = BattlePhase::BattleOver { message: last_msg };
                }
            } else {
                let first_alive = game_state.state.alive_party_indices();
                ui_state.phase = BattlePhase::CommandSelect {
                    member_index: first_alive.first().copied().unwrap_or(0),
                };
            }
        } else {
            // 次のメッセージに対応するエフェクトを処理
            process_message_effects(ui_state, next_index);
            ui_state.phase = BattlePhase::ShowMessage {
                messages: match &ui_state.phase {
                    BattlePhase::ShowMessage { messages, .. } => messages.clone(),
                    _ => Vec::new(),
                },
                index: next_index,
            };
        }
    }
}

fn handle_battle_over(
    keyboard: &ButtonInput<KeyCode>,
    next_state: &mut NextState<BattleState>,
) {
    if input_ui::is_confirm_just_pressed(keyboard) {
        next_state.set(BattleState::None);
    }
}
