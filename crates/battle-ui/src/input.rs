use bevy::prelude::*;

use battle::{ActorId, BattleAction, BuffStat, SpellKind, SpellTarget, TargetId, TurnRandomFactors, TurnResult};
use party::ItemEffect;

use app_state::BattleState;

use super::scene::{
    enemy_display_names, BattleGameState, BattlePhase, BattleUIState, MessageEffect,
};

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
            handle_spell_select(&keyboard, &mut game_state, &mut ui_state, member_index);
        }
        BattlePhase::ItemSelect { member_index } => {
            handle_item_select(&keyboard, &game_state, &mut ui_state, member_index);
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
    // 上下でカーソル移動 (0=たたかう, 1=じゅもん, 2=どうぐ, 3=にげる)
    if input_ui::is_up_just_pressed(keyboard) {
        ui_state.selected_command = if ui_state.selected_command > 0 { ui_state.selected_command - 1 } else { 3 };
    }
    if input_ui::is_down_just_pressed(keyboard) {
        ui_state.selected_command = if ui_state.selected_command < 3 { ui_state.selected_command + 1 } else { 0 };
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
                ui_state.target_offset = 0;
                ui_state.pending_spell = None;
                ui_state.phase = BattlePhase::TargetSelect { member_index };
            }
            1 => {
                // じゅもん → 呪文がないクラスは遷移しない
                let member_kind = game_state.state.party[member_index].kind;
                let spells = party::available_spells(member_kind, game_state.state.party[member_index].level);
                if spells.is_empty() {
                    return;
                }
                ui_state.selected_spell = 0;
                ui_state.phase = BattlePhase::SpellSelect { member_index };
            }
            2 => {
                // どうぐ → アイテム選択へ（空なら即戻る）
                if game_state.state.party[member_index].inventory.is_empty() {
                    return;
                }
                ui_state.selected_item = 0;
                ui_state.phase = BattlePhase::ItemSelect { member_index };
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
    game_state: &mut BattleGameState,
    ui_state: &mut BattleUIState,
    member_index: usize,
) {
    let member_kind = game_state.state.party[member_index].kind;
    let spells = party::available_spells(member_kind, game_state.state.party[member_index].level);
    let spell_count = spells.len();

    // 上下でカーソル移動
    if input_ui::is_up_just_pressed(keyboard) {
        ui_state.selected_spell = if ui_state.selected_spell > 0 { ui_state.selected_spell - 1 } else { spell_count - 1 };
    }
    if input_ui::is_down_just_pressed(keyboard) {
        ui_state.selected_spell = if ui_state.selected_spell < spell_count - 1 { ui_state.selected_spell + 1 } else { 0 };
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

        match spell.target_type() {
            SpellTarget::SingleEnemy => {
                // 単体攻撃 → 敵選択へ
                ui_state.target_offset = 0;
                ui_state.phase = BattlePhase::TargetSelect { member_index };
            }
            SpellTarget::AllEnemies => {
                // 全体攻撃 → ターゲット選択スキップ、ダミーtargetで即登録
                ui_state.pending_spell = None;
                ui_state.pending_commands.set(
                    member_index,
                    BattleAction::Spell {
                        spell,
                        target: TargetId::Enemy(0),
                    },
                );
                advance_to_next_member(game_state, ui_state, member_index);
            }
            SpellTarget::SingleAlly => {
                // 単体味方 → 味方選択へ
                ui_state.ally_target_offset = 0;
                ui_state.phase = BattlePhase::AllyTargetSelect { member_index };
            }
            SpellTarget::AllAllies => {
                // 全体味方 → ターゲット選択スキップ、ダミーtargetで即登録
                ui_state.pending_spell = None;
                ui_state.pending_commands.set(
                    member_index,
                    BattleAction::Spell {
                        spell,
                        target: TargetId::Party(0),
                    },
                );
                advance_to_next_member(game_state, ui_state, member_index);
            }
        }
    }
}

/// 次の生存メンバーに進む、全員入力済みならターン実行
fn advance_to_next_member(
    game_state: &mut BattleGameState,
    ui_state: &mut BattleUIState,
    current_member: usize,
) {
    let next = find_next_alive_member(game_state, current_member);
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

fn handle_item_select(
    keyboard: &ButtonInput<KeyCode>,
    game_state: &BattleGameState,
    ui_state: &mut BattleUIState,
    member_index: usize,
) {
    let owned = game_state.state.party[member_index].inventory.owned_items();
    if owned.is_empty() {
        ui_state.phase = BattlePhase::CommandSelect { member_index };
        return;
    }
    let item_count = owned.len();

    // 上下でカーソル移動
    if input_ui::is_up_just_pressed(keyboard) {
        ui_state.selected_item = if ui_state.selected_item > 0 { ui_state.selected_item - 1 } else { item_count - 1 };
    }
    if input_ui::is_down_just_pressed(keyboard) {
        ui_state.selected_item = if ui_state.selected_item < item_count - 1 { ui_state.selected_item + 1 } else { 0 };
    }

    // キャンセル: コマンド選択に戻る
    if input_ui::is_cancel_just_pressed(keyboard) {
        ui_state.phase = BattlePhase::CommandSelect { member_index };
        return;
    }

    // 決定
    if input_ui::is_confirm_just_pressed(keyboard) {
        let item = owned[ui_state.selected_item];

        // キーアイテム・素材・装備は戦闘中使用不可
        if matches!(item.effect(), ItemEffect::KeyItem | ItemEffect::Material | ItemEffect::Equip) {
            return;
        }

        ui_state.pending_item = Some(item);

        // 回復アイテム → 味方選択へ
        ui_state.ally_target_offset = 0;
        ui_state.phase = BattlePhase::AllyTargetSelect { member_index };
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
        ui_state.target_offset = if ui_state.target_offset > 0 {
            ui_state.target_offset - 1
        } else {
            alive_enemies.len() - 1
        };
    }
    if input_ui::is_right_just_pressed(keyboard) {
        ui_state.target_offset = if ui_state.target_offset < alive_enemies.len() - 1 {
            ui_state.target_offset + 1
        } else {
            0
        };
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
        let target = TargetId::Enemy(alive_enemies[ui_state.target_offset]);

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
        ui_state.ally_target_offset = if ui_state.ally_target_offset > 0 {
            ui_state.ally_target_offset - 1
        } else {
            alive_party.len() - 1
        };
    }
    if input_ui::is_right_just_pressed(keyboard) {
        ui_state.ally_target_offset = if ui_state.ally_target_offset < alive_party.len() - 1 {
            ui_state.ally_target_offset + 1
        } else {
            0
        };
    }

    // キャンセル: 呪文選択またはアイテム選択に戻る
    if input_ui::is_cancel_just_pressed(keyboard) {
        if ui_state.pending_item.is_some() {
            ui_state.pending_item = None;
            ui_state.phase = BattlePhase::ItemSelect { member_index };
        } else {
            ui_state.pending_spell = None;
            ui_state.phase = BattlePhase::SpellSelect { member_index };
        }
        return;
    }

    // 決定
    if input_ui::is_confirm_just_pressed(keyboard) {
        let target = TargetId::Party(alive_party[ui_state.ally_target_offset]);

        if let Some(spell) = ui_state.pending_spell.take() {
            ui_state
                .pending_commands
                .set(member_index, BattleAction::Spell { spell, target });
        } else if let Some(item) = ui_state.pending_item.take() {
            ui_state
                .pending_commands
                .set(member_index, BattleAction::UseItem { item, target });
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

    let spell_randoms: Vec<f32> = (0..enemy_count)
        .map(|_| rand::random::<f32>())
        .collect();

    let random_factors = TurnRandomFactors {
        damage_randoms,
        flee_random,
        spell_randoms,
    };

    // ターン実行前のパーティHP/MP状態をスナップショット
    let pre_party_hp: Vec<i32> = game_state.state.party.iter().map(|m| m.stats.hp).collect();
    let pre_party_mp: Vec<i32> = game_state.state.party.iter().map(|m| m.stats.mp).collect();

    let results = game_state
        .state
        .execute_turn(&ui_state.pending_commands.to_commands(), &random_factors);

    ui_state.pending_commands.clear();

    let (mut messages, effects) =
        results_to_messages(&results, &game_state.state, &pre_party_hp, &pre_party_mp);
    ui_state.message_effects = effects;

    // 勝利時: 経験値獲得メッセージとレベルアップ処理を追加
    if game_state.state.is_victory() {
        let total_exp = game_state.state.total_exp_reward();
        messages.push(format!("けいけんち {}ポイント かくとく！", total_exp));

        // 生存メンバーに経験値を付与してレベルアップ判定
        let alive = game_state.state.alive_party_indices();
        for &i in &alive {
            let member = &mut game_state.state.party[i];
            let old_level = member.level;
            let level_ups = member.gain_exp(total_exp);
            if level_ups > 0 {
                messages.push(format!(
                    "{}は レベル{}に あがった！",
                    member.kind.name(),
                    member.level
                ));
                // レベルアップで新しく習得した呪文をチェック
                for lvl in (old_level + 1)..=member.level {
                    for spell in party::spells_learned_at_level(member.kind, lvl) {
                        messages.push(format!(
                            "{}は {}を おぼえた！",
                            member.kind.name(),
                            spell.name()
                        ));
                    }
                }
            }
        }
    }

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
                member_index: first_alive.first().copied().expect("戦闘継続中に生存味方がいない"),
            };
        }
        return;
    }

    // 最初のメッセージ(index=0)に対応するエフェクトを処理
    process_message_effects(ui_state, 0);

    if game_state.state.is_over() {
        let last_msg = messages.last().cloned().expect("メッセージが空");
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

    // AoEメッセージ最適化: 同じキャスター+呪文の連続SpellDamage/Healed/Buffedの
    // 2件目以降は詠唱メッセージを省略
    let mut last_aoe_caster_spell: Option<(ActorId, SpellKind)> = None;

    for result in results {
        match result {
            TurnResult::Attack {
                attacker,
                target,
                damage,
            } => {
                last_aoe_caster_spell = None;
                let attacker_name = actor_name(attacker, state, &enemy_names);
                let target_name = target_name_str(target, state, &enemy_names);
                let msg_index = messages.len();
                messages.push(format!(
                    "{}の こうげき！ {}に {}ダメージ！",
                    attacker_name, target_name, damage
                ));

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
                let target_name = target_name_str(target, state, &enemy_names);
                let msg_index = messages.len();

                let is_continuation = last_aoe_caster_spell == Some((*caster, *spell));
                if is_continuation {
                    messages.push(format!("{}に {}ダメージ！", target_name, damage));
                } else {
                    let caster_name = actor_name(caster, state, &enemy_names);
                    messages.push(format!(
                        "{}は {}を となえた！ {}に {}ダメージ！",
                        caster_name,
                        spell.name(),
                        target_name,
                        damage
                    ));
                }
                last_aoe_caster_spell = Some((*caster, *spell));

                // キャスターのMP更新エフェクト（最初のヒットのみ）
                if !is_continuation
                    && let ActorId::Party(ci) = caster
                {
                    running_party_mp[*ci] = (running_party_mp[*ci] - spell.mp_cost()).max(0);
                    effects.push((
                        msg_index,
                        MessageEffect::UpdatePartyMp {
                            member_index: *ci,
                            new_mp: running_party_mp[*ci],
                        },
                    ));
                }

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
            TurnResult::Healed {
                caster,
                spell,
                target,
                amount,
            } => {
                let target_name = target_name_str(target, state, &enemy_names);
                let msg_index = messages.len();

                let is_continuation = last_aoe_caster_spell == Some((*caster, *spell));
                if is_continuation {
                    messages.push(format!(
                        "{}の HPが {}かいふく！",
                        target_name, amount
                    ));
                } else {
                    let caster_name = actor_name(caster, state, &enemy_names);
                    messages.push(format!(
                        "{}は {}を となえた！ {}の HPが {}かいふく！",
                        caster_name,
                        spell.name(),
                        target_name,
                        amount
                    ));
                }
                last_aoe_caster_spell = Some((*caster, *spell));

                // キャスターのMP更新エフェクト（最初のヒットのみ）
                if !is_continuation
                    && let ActorId::Party(ci) = caster
                {
                    running_party_mp[*ci] =
                        (running_party_mp[*ci] - spell.mp_cost()).max(0);
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
            TurnResult::Buffed {
                caster,
                spell,
                target,
                amount,
            } => {
                let target_name = target_name_str(target, state, &enemy_names);
                let msg_index = messages.len();

                let stat_name = match spell.effect() {
                    battle::SpellEffect::AttackBuff => "こうげきりょく",
                    battle::SpellEffect::DefenseBuff => "しゅびりょく",
                    _ => "",
                };

                let is_continuation = last_aoe_caster_spell == Some((*caster, *spell));
                if is_continuation {
                    messages.push(format!(
                        "{}の {}が {}あがった！",
                        target_name, stat_name, amount
                    ));
                } else {
                    let caster_name = actor_name(caster, state, &enemy_names);
                    messages.push(format!(
                        "{}は {}を となえた！ {}の {}が {}あがった！",
                        caster_name,
                        spell.name(),
                        target_name,
                        stat_name,
                        amount
                    ));
                }
                last_aoe_caster_spell = Some((*caster, *spell));

                // キャスターのMP更新エフェクト（最初のヒットのみ）
                if !is_continuation
                    && let ActorId::Party(ci) = caster
                {
                    running_party_mp[*ci] =
                        (running_party_mp[*ci] - spell.mp_cost()).max(0);
                    effects.push((
                        msg_index,
                        MessageEffect::UpdatePartyMp {
                            member_index: *ci,
                            new_mp: running_party_mp[*ci],
                        },
                    ));
                }
            }
            TurnResult::BuffExpired { target, stat } => {
                last_aoe_caster_spell = None;
                let target_name = target_name_str(target, state, &enemy_names);
                let stat_name = match stat {
                    BuffStat::Attack => "こうげきりょく",
                    BuffStat::Defense => "しゅびりょく",
                };
                messages.push(format!(
                    "{}の {}アップの こうかが きれた！",
                    target_name, stat_name
                ));
            }
            TurnResult::ItemUsed {
                user,
                item,
                target,
                amount,
            } => {
                last_aoe_caster_spell = None;
                let user_name = actor_name(user, state, &enemy_names);
                let target_name = target_name_str(target, state, &enemy_names);
                let msg_index = messages.len();
                messages.push(format!(
                    "{}は {}を つかった！ {}の HPが {}かいふく！",
                    user_name,
                    item.name(),
                    target_name,
                    amount
                ));

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
            TurnResult::MpDrained {
                caster,
                spell,
                target,
                amount,
            } => {
                let target_name = target_name_str(target, state, &enemy_names);
                let msg_index = messages.len();

                let is_continuation = last_aoe_caster_spell == Some((*caster, *spell));
                if is_continuation {
                    messages.push(format!(
                        "{}の MPが {}へった！",
                        target_name, amount
                    ));
                } else {
                    let caster_name = actor_name(caster, state, &enemy_names);
                    messages.push(format!(
                        "{}は {}を となえた！ {}の MPが {}へった！",
                        caster_name,
                        spell.name(),
                        target_name,
                        amount
                    ));
                }
                last_aoe_caster_spell = Some((*caster, *spell));

                // キャスターのMP更新エフェクト（最初のヒットのみ）
                if !is_continuation
                    && let ActorId::Party(ci) = caster
                {
                    running_party_mp[*ci] =
                        (running_party_mp[*ci] - spell.mp_cost()).max(0);
                    effects.push((
                        msg_index,
                        MessageEffect::UpdatePartyMp {
                            member_index: *ci,
                            new_mp: running_party_mp[*ci],
                        },
                    ));
                }

                // ターゲットのMP更新エフェクト
                match target {
                    TargetId::Party(pi) => {
                        running_party_mp[*pi] = (running_party_mp[*pi] - amount).max(0);
                        effects.push((
                            msg_index,
                            MessageEffect::UpdatePartyMp {
                                member_index: *pi,
                                new_mp: running_party_mp[*pi],
                            },
                        ));
                    }
                    TargetId::Enemy(i) => {
                        effects.push((
                            msg_index,
                            MessageEffect::BlinkEnemy { enemy_index: *i },
                        ));
                    }
                }
            }
            TurnResult::Defeated { target } => {
                last_aoe_caster_spell = None;
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
            TurnResult::AilmentInflicted {
                caster,
                spell,
                target,
                ailment,
            } => {
                let caster_name = actor_name(caster, state, &enemy_names);
                let target_name = target_name_str(target, state, &enemy_names);
                let msg_index = messages.len();

                let is_continuation = last_aoe_caster_spell == Some((*caster, *spell));
                if is_continuation {
                    messages.push(format!(
                        "{}は {}になった！",
                        target_name,
                        ailment.name()
                    ));
                } else {
                    messages.push(format!(
                        "{}は {}を となえた！ {}は {}になった！",
                        caster_name,
                        spell.name(),
                        target_name,
                        ailment.name()
                    ));
                }
                last_aoe_caster_spell = Some((*caster, *spell));

                // キャスターのMP更新エフェクト（最初のヒットのみ）
                if !is_continuation
                    && let ActorId::Party(ci) = caster
                {
                    running_party_mp[*ci] =
                        (running_party_mp[*ci] - spell.mp_cost()).max(0);
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
            TurnResult::AilmentResisted {
                caster,
                spell,
                target,
            } => {
                let target_name = target_name_str(target, state, &enemy_names);
                let msg_index = messages.len();

                let is_continuation = last_aoe_caster_spell == Some((*caster, *spell));
                if is_continuation {
                    messages.push(format!(
                        "しかし {}には きかなかった！",
                        target_name
                    ));
                } else {
                    let caster_name = actor_name(caster, state, &enemy_names);
                    messages.push(format!(
                        "{}は {}を となえた！ しかし {}には きかなかった！",
                        caster_name,
                        spell.name(),
                        target_name
                    ));
                }
                last_aoe_caster_spell = Some((*caster, *spell));

                // キャスターのMP更新エフェクト（最初のヒットのみ）
                if !is_continuation
                    && let ActorId::Party(ci) = caster
                {
                    running_party_mp[*ci] =
                        (running_party_mp[*ci] - spell.mp_cost()).max(0);
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
            TurnResult::Sleeping { actor } => {
                last_aoe_caster_spell = None;
                let name = actor_name(actor, state, &enemy_names);
                messages.push(format!("{}は ねむっている…", name));
            }
            TurnResult::PoisonDamage { target, damage } => {
                last_aoe_caster_spell = None;
                let target_name = target_name_str(target, state, &enemy_names);
                let msg_index = messages.len();
                messages.push(format!(
                    "{}は どくで {}ダメージ！",
                    target_name, damage
                ));

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
            TurnResult::AilmentCured { target, ailment } => {
                last_aoe_caster_spell = None;
                let target_name = target_name_str(target, state, &enemy_names);
                messages.push(format!(
                    "{}の {}が なおった！",
                    target_name,
                    ailment.name()
                ));
            }
            TurnResult::Fled => {
                last_aoe_caster_spell = None;
                messages.push("うまく にげきれた！".to_string());
            }
            TurnResult::FleeFailed => {
                last_aoe_caster_spell = None;
                messages.push("にげられなかった！".to_string());
            }
        }
    }

    (messages, effects)
}

/// 指定メッセージindexに対応するエフェクトを処理し、表示状態を更新する
fn process_message_effects(ui_state: &mut BattleUIState, message_index: usize) {
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
        ActorId::Enemy(i) => enemy_names[*i].clone(),
    }
}

fn target_name_str(
    target: &TargetId,
    state: &battle::BattleState,
    enemy_names: &[String],
) -> String {
    match target {
        TargetId::Enemy(i) => enemy_names[*i].clone(),
        TargetId::Party(i) => state.party[*i].kind.name().to_string(),
    }
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
                    let last_msg = messages.last().cloned().expect("メッセージが空");
                    ui_state.phase = BattlePhase::BattleOver { message: last_msg };
                }
            } else {
                let first_alive = game_state.state.alive_party_indices();
                ui_state.phase = BattlePhase::CommandSelect {
                    member_index: first_alive.first().copied().expect("戦闘継続中に生存味方がいない"),
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
