use bevy::prelude::*;

use game::battle::BattleAction;

use crate::app_state::AppState;

use super::scene::{BattlePhase, BattleResource};

/// 戦闘中の入力処理システム
pub fn battle_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut battle_res: ResMut<BattleResource>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match &battle_res.phase {
        BattlePhase::CommandSelect => {
            handle_command_select(&keyboard, &mut battle_res);
        }
        BattlePhase::ShowMessage { messages, index } => {
            let index = *index;
            let len = messages.len();
            handle_show_message(&keyboard, &mut battle_res, index, len);
        }
        BattlePhase::BattleOver { .. } => {
            handle_battle_over(&keyboard, &mut next_state);
        }
    }
}

fn handle_command_select(
    keyboard: &ButtonInput<KeyCode>,
    battle_res: &mut BattleResource,
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

    // 決定
    if keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyZ)
    {
        let action = match battle_res.selected_command {
            0 => BattleAction::Attack,
            _ => BattleAction::Flee,
        };

        let random_factor = 0.8 + rand::random::<f32>() * 0.4; // 0.8~1.2
        let results = battle_res.state.execute_player_turn(action, random_factor);

        let mut messages = Vec::new();
        let mut battle_over = false;

        for result in &results {
            match result {
                game::battle::TurnResult::PlayerAttack { damage } => {
                    let enemy_name = battle_res.state.enemy.kind.name();
                    messages.push(format!(
                        "{}に {}ダメージ！",
                        enemy_name, damage
                    ));
                }
                game::battle::TurnResult::EnemyDefeated => {
                    let enemy_name = battle_res.state.enemy.kind.name();
                    messages.push(format!("{}を たおした！", enemy_name));
                    battle_over = true;
                }
                game::battle::TurnResult::Fled => {
                    messages.push("うまく にげきれた！".to_string());
                    battle_over = true;
                }
                _ => {}
            }
        }

        // 敵が生きていれば敵のターン
        if !battle_over && battle_res.state.enemy.stats.is_alive() {
            let enemy_random = 0.8 + rand::random::<f32>() * 0.4;
            let enemy_results = battle_res.state.execute_enemy_turn(enemy_random);

            for result in &enemy_results {
                match result {
                    game::battle::TurnResult::EnemyAttack { damage } => {
                        let enemy_name = battle_res.state.enemy.kind.name();
                        messages.push(format!(
                            "{}の こうげき！ {}ダメージ！",
                            enemy_name, damage
                        ));
                    }
                    game::battle::TurnResult::PlayerDefeated => {
                        messages.push("あなたは たおれた...".to_string());
                        battle_over = true;
                    }
                    _ => {}
                }
            }
        }

        if battle_over {
            let last_message = messages.pop().unwrap_or_default();
            if messages.is_empty() {
                battle_res.phase = BattlePhase::BattleOver {
                    message: last_message,
                };
            } else {
                // 途中メッセージを先に表示、最後のメッセージはBattleOverで表示
                // → ShowMessageの最後でBattleOverに遷移
                messages.push(last_message);
                battle_res.phase = BattlePhase::ShowMessage {
                    messages,
                    index: 0,
                };
            }
        } else if messages.is_empty() {
            battle_res.phase = BattlePhase::CommandSelect;
        } else {
            battle_res.phase = BattlePhase::ShowMessage {
                messages,
                index: 0,
            };
        }
    }
}

fn handle_show_message(
    keyboard: &ButtonInput<KeyCode>,
    battle_res: &mut BattleResource,
    index: usize,
    len: usize,
) {
    if keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyZ)
    {
        let next_index = index + 1;
        if next_index >= len {
            // 全メッセージ表示完了
            if battle_res.state.is_over() {
                // 最後のメッセージをBattleOverに
                if let BattlePhase::ShowMessage { messages, .. } = &battle_res.phase {
                    let last_msg = messages.last().cloned().unwrap_or_default();
                    battle_res.phase = BattlePhase::BattleOver { message: last_msg };
                }
            } else {
                battle_res.phase = BattlePhase::CommandSelect;
            }
        } else {
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

fn handle_battle_over(
    keyboard: &ButtonInput<KeyCode>,
    next_state: &mut NextState<AppState>,
) {
    if keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyZ)
    {
        next_state.set(AppState::Exploring);
    }
}
