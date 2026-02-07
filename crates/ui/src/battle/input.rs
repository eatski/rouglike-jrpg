use bevy::prelude::*;

use game::battle::{ActorId, BattleAction, TargetId, TurnRandomFactors, TurnResult};

use crate::app_state::AppState;

use super::scene::{BattlePhase, BattleResource, MessageEffect};

/// 戦闘中の入力処理システム
pub fn battle_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut battle_res: ResMut<BattleResource>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match battle_res.phase.clone() {
        BattlePhase::CommandSelect { member_index } => {
            handle_command_select(&keyboard, &mut battle_res, member_index);
        }
        BattlePhase::TargetSelect { member_index } => {
            handle_target_select(&keyboard, &mut battle_res, member_index);
        }
        BattlePhase::ShowMessage { messages, index } => {
            handle_show_message(&keyboard, &mut battle_res, index, messages.len());
        }
        BattlePhase::BattleOver { .. } => {
            handle_battle_over(&keyboard, &mut next_state);
        }
    }
}

fn is_confirm(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyZ)
}

fn is_cancel(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyX)
}

fn handle_command_select(
    keyboard: &ButtonInput<KeyCode>,
    battle_res: &mut BattleResource,
    member_index: usize,
) {
    // 上下でカーソル移動
    if (keyboard.just_pressed(KeyCode::KeyW) || keyboard.just_pressed(KeyCode::ArrowUp))
        && battle_res.selected_command > 0
    {
        battle_res.selected_command -= 1;
    }
    if (keyboard.just_pressed(KeyCode::KeyS) || keyboard.just_pressed(KeyCode::ArrowDown))
        && battle_res.selected_command < 1
    {
        battle_res.selected_command += 1;
    }

    // キャンセル: 前のメンバーに戻る
    if is_cancel(keyboard) && member_index > 0 {
        // 前のメンバーのコマンドを取り消し
        battle_res.pending_commands.pop();
        // 前の生存メンバーを探す
        let prev = find_prev_alive_member(battle_res, member_index);
        if let Some(prev_idx) = prev {
            battle_res.selected_command = 0;
            battle_res.phase = BattlePhase::CommandSelect {
                member_index: prev_idx,
            };
        }
        return;
    }

    // 決定
    if is_confirm(keyboard) {
        match battle_res.selected_command {
            0 => {
                // たたかう → ターゲット選択へ
                let first_alive = battle_res.state.alive_enemy_indices();
                battle_res.selected_target = first_alive.first().copied().unwrap_or(0);
                battle_res.phase = BattlePhase::TargetSelect { member_index };
            }
            _ => {
                // にげる → 全員Flee確定、即実行
                battle_res.pending_commands.clear();
                for _ in 0..battle_res.state.party.len() {
                    battle_res.pending_commands.push(BattleAction::Flee);
                }
                execute_turn(battle_res);
            }
        }
        battle_res.selected_command = 0;
    }
}

fn handle_target_select(
    keyboard: &ButtonInput<KeyCode>,
    battle_res: &mut BattleResource,
    member_index: usize,
) {
    let alive_enemies = battle_res.state.alive_enemy_indices();
    if alive_enemies.is_empty() {
        return;
    }

    // 左右でターゲット切り替え
    if keyboard.just_pressed(KeyCode::KeyA) || keyboard.just_pressed(KeyCode::ArrowLeft) {
        let current_pos = alive_enemies
            .iter()
            .position(|&i| i == battle_res.selected_target)
            .unwrap_or(0);
        if current_pos > 0 {
            battle_res.selected_target = alive_enemies[current_pos - 1];
        }
    }
    if keyboard.just_pressed(KeyCode::KeyD) || keyboard.just_pressed(KeyCode::ArrowRight) {
        let current_pos = alive_enemies
            .iter()
            .position(|&i| i == battle_res.selected_target)
            .unwrap_or(0);
        if current_pos < alive_enemies.len() - 1 {
            battle_res.selected_target = alive_enemies[current_pos + 1];
        }
    }

    // キャンセル: コマンド選択に戻る
    if is_cancel(keyboard) {
        battle_res.phase = BattlePhase::CommandSelect { member_index };
        return;
    }

    // 決定
    if is_confirm(keyboard) {
        let target = TargetId::Enemy(battle_res.selected_target);
        battle_res
            .pending_commands
            .push(BattleAction::Attack { target });

        // 次の生存メンバーを探す
        let next = find_next_alive_member(battle_res, member_index);
        if let Some(next_idx) = next {
            battle_res.selected_command = 0;
            battle_res.phase = BattlePhase::CommandSelect {
                member_index: next_idx,
            };
        } else {
            // 全員入力完了 → ターン実行
            execute_turn(battle_res);
        }
    }
}

/// 次の生存パーティメンバーを探す
fn find_next_alive_member(battle_res: &BattleResource, current: usize) -> Option<usize> {
    let alive = battle_res.state.alive_party_indices();
    alive.into_iter().find(|&i| i > current)
}

/// 前の生存パーティメンバーを探す
fn find_prev_alive_member(battle_res: &BattleResource, current: usize) -> Option<usize> {
    let alive = battle_res.state.alive_party_indices();
    alive.into_iter().rev().find(|&i| i < current)
}

/// ターンを実行してメッセージフェーズに遷移
fn execute_turn(battle_res: &mut BattleResource) {
    let party_count = battle_res.state.party.len();
    let enemy_count = battle_res.state.enemies.len();
    let total_actors = party_count + enemy_count;

    let damage_randoms: Vec<f32> = (0..total_actors)
        .map(|_| 0.8 + rand::random::<f32>() * 0.4)
        .collect();
    let flee_random: f32 = rand::random();

    let random_factors = TurnRandomFactors {
        damage_randoms,
        flee_random,
    };

    // ターン実行前のパーティHP状態をスナップショット
    let pre_party_hp: Vec<i32> = battle_res.state.party.iter().map(|m| m.stats.hp).collect();

    let results = battle_res
        .state
        .execute_turn(&battle_res.pending_commands.commands, &random_factors);

    battle_res.pending_commands.clear();

    let (messages, effects) =
        results_to_messages(&results, &battle_res.state, &pre_party_hp);
    battle_res.message_effects = effects;

    if messages.is_empty() {
        // メッセージがない場合、表示HPを実際のHPに同期
        sync_display_hp(battle_res);
        if battle_res.state.is_over() {
            battle_res.phase = BattlePhase::BattleOver {
                message: "".to_string(),
            };
        } else {
            let first_alive = battle_res.state.alive_party_indices();
            battle_res.phase = BattlePhase::CommandSelect {
                member_index: first_alive.first().copied().unwrap_or(0),
            };
        }
        return;
    }

    // 最初のメッセージ(index=0)に対応するエフェクトを処理
    process_message_effects(battle_res, 0);

    if battle_res.state.is_over() {
        let last_msg = messages.last().cloned().unwrap_or_default();
        if messages.len() == 1 {
            battle_res.phase = BattlePhase::BattleOver { message: last_msg };
        } else {
            battle_res.phase = BattlePhase::ShowMessage {
                messages,
                index: 0,
            };
        }
    } else {
        battle_res.phase = BattlePhase::ShowMessage {
            messages,
            index: 0,
        };
    }
}

/// TurnResult列をメッセージ文字列列とMessageEffect列に変換
///
/// pre_party_hp: ターン実行前のパーティHPスナップショット。
/// 各攻撃メッセージにパーティHP変化のエフェクトを紐付ける。
fn results_to_messages(
    results: &[TurnResult],
    state: &game::battle::BattleState,
    pre_party_hp: &[i32],
) -> (Vec<String>, Vec<(usize, MessageEffect)>) {
    let enemy_names = enemy_display_names(&state.enemies);
    let mut messages = Vec::new();
    let mut effects: Vec<(usize, MessageEffect)> = Vec::new();

    // パーティHPの「現在の表示HP」を追跡（攻撃ごとに減算していく）
    let mut running_party_hp: Vec<i32> = pre_party_hp.to_vec();

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
fn process_message_effects(battle_res: &mut BattleResource, message_index: usize) {
    // message_effectsからcloneして取り出す（borrowの競合を避けるため）
    let effects: Vec<MessageEffect> = battle_res
        .message_effects
        .iter()
        .filter(|(idx, _)| *idx == message_index)
        .map(|(_, effect)| effect.clone())
        .collect();

    for effect in effects {
        match effect {
            MessageEffect::UpdatePartyHp {
                member_index,
                new_hp,
            } => {
                if let Some(hp) = battle_res.display_party_hp.get_mut(member_index) {
                    *hp = new_hp;
                }
            }
            MessageEffect::HideEnemy { enemy_index } => {
                if let Some(hidden) = battle_res.hidden_enemies.get_mut(enemy_index) {
                    *hidden = true;
                }
            }
            MessageEffect::Shake => {
                battle_res.shake_timer =
                    Some(Timer::from_seconds(0.15, TimerMode::Once));
            }
            MessageEffect::BlinkEnemy { enemy_index } => {
                battle_res.blink_timer =
                    Some(Timer::from_seconds(0.3, TimerMode::Once));
                battle_res.blink_enemy = Some(enemy_index);
            }
        }
    }
}

/// 表示HPを実際のゲーム状態HPに同期する
fn sync_display_hp(battle_res: &mut BattleResource) {
    for (i, member) in battle_res.state.party.iter().enumerate() {
        if let Some(hp) = battle_res.display_party_hp.get_mut(i) {
            *hp = member.stats.hp;
        }
    }
}

fn actor_name(
    actor: &ActorId,
    state: &game::battle::BattleState,
    enemy_names: &[String],
) -> String {
    match actor {
        ActorId::Party(i) => state.party[*i].kind.name().to_string(),
        ActorId::Enemy(i) => enemy_names.get(*i).cloned().unwrap_or_default(),
    }
}

fn target_name_str(
    target: &TargetId,
    state: &game::battle::BattleState,
    enemy_names: &[String],
) -> String {
    match target {
        TargetId::Enemy(i) => enemy_names.get(*i).cloned().unwrap_or_default(),
        TargetId::Party(i) => state.party[*i].kind.name().to_string(),
    }
}

/// 同種の敵にサフィックスを付与した表示名を生成
fn enemy_display_names(enemies: &[game::battle::Enemy]) -> Vec<String> {
    use game::battle::EnemyKind;
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
    battle_res: &mut BattleResource,
    index: usize,
    len: usize,
) {
    if is_confirm(keyboard) {
        let next_index = index + 1;
        if next_index >= len {
            // 全メッセージ表示完了 — 表示HPを実際のHPに同期
            sync_display_hp(battle_res);
            if battle_res.state.is_over() {
                if let BattlePhase::ShowMessage { messages, .. } = &battle_res.phase {
                    let last_msg = messages.last().cloned().unwrap_or_default();
                    battle_res.phase = BattlePhase::BattleOver { message: last_msg };
                }
            } else {
                let first_alive = battle_res.state.alive_party_indices();
                battle_res.phase = BattlePhase::CommandSelect {
                    member_index: first_alive.first().copied().unwrap_or(0),
                };
            }
        } else {
            // 次のメッセージに対応するエフェクトを処理
            process_message_effects(battle_res, next_index);
            battle_res.phase = BattlePhase::ShowMessage {
                messages: match &battle_res.phase {
                    BattlePhase::ShowMessage { messages, .. } => messages.clone(),
                    _ => Vec::new(),
                },
                index: next_index,
            };
        }
    }
}

fn handle_battle_over(keyboard: &ButtonInput<KeyCode>, next_state: &mut NextState<AppState>) {
    if is_confirm(keyboard) {
        next_state.set(AppState::Exploring);
    }
}
