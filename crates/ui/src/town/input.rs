use bevy::prelude::*;

use game::town::{heal_party, townsperson_dialogue};

use crate::app_state::AppState;

use super::scene::{TownMenuPhase, TownResource};

fn is_confirm(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyZ)
}

/// 町画面の入力処理システム
pub fn town_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut town_res: ResMut<TownResource>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match &town_res.phase {
        TownMenuPhase::MenuSelect => {
            // 上下でカーソル移動
            if (keyboard.just_pressed(KeyCode::KeyW)
                || keyboard.just_pressed(KeyCode::ArrowUp))
                && town_res.selected_item > 0
            {
                town_res.selected_item -= 1;
            }
            if (keyboard.just_pressed(KeyCode::KeyS)
                || keyboard.just_pressed(KeyCode::ArrowDown))
                && town_res.selected_item < 2
            {
                town_res.selected_item += 1;
            }

            if is_confirm(&keyboard) {
                match town_res.selected_item {
                    0 => {
                        // やどや → 回復メッセージ表示（暫定: HP/MPは永続化されていないため表示のみ）
                        let mut party = game::battle::default_party();
                        heal_party(&mut party);
                        town_res.phase = TownMenuPhase::ShowMessage {
                            message: "ゆっくり やすんだ。\nHP と MP が かいふくした！".to_string(),
                        };
                    }
                    1 => {
                        // 話を聞く → NPC台詞表示
                        let dialogue = townsperson_dialogue();
                        town_res.phase = TownMenuPhase::ShowMessage {
                            message: dialogue.to_string(),
                        };
                    }
                    _ => {
                        // 街を出る → フィールドに戻る
                        next_state.set(AppState::Exploring);
                    }
                }
            }
        }
        TownMenuPhase::ShowMessage { .. } => {
            // メッセージ表示中は Enter でメニューに戻る
            if is_confirm(&keyboard) {
                town_res.phase = TownMenuPhase::MenuSelect;
            }
        }
    }
}
