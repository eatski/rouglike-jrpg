use bevy::prelude::*;

use town::{cave_hint_dialogue, heal_party};

use app_state::SceneState;
use movement_ui::{Player, TilePosition};
use shared_ui::{ActiveMap, PartyState};

use crate::scene::{TownMenuPhase, TownResource};

fn is_confirm(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyZ)
}

/// 町画面の入力処理システム
pub fn town_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut town_res: ResMut<TownResource>,
    mut next_state: ResMut<NextState<SceneState>>,
    mut party_state: ResMut<PartyState>,
    active_map: Res<ActiveMap>,
    player_query: Query<&TilePosition, With<Player>>,
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
                        // やどや → HP/MPを全回復
                        heal_party(&mut party_state.members);
                        town_res.phase = TownMenuPhase::ShowMessage {
                            message: "ゆっくり やすんだ。\nHP と MP が かいふくした！".to_string(),
                        };
                    }
                    1 => {
                        // 話を聞く → 最寄り洞窟の方角を教える
                        let dialogue = if let Ok(pos) = player_query.single() {
                            cave_hint_dialogue(&active_map.grid, pos.x, pos.y)
                        } else {
                            cave_hint_dialogue(&active_map.grid, 0, 0)
                        };
                        town_res.phase = TownMenuPhase::ShowMessage {
                            message: dialogue,
                        };
                    }
                    _ => {
                        // 街を出る → フィールドに戻る
                        next_state.set(SceneState::Exploring);
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
