use bevy::prelude::*;

use input_ui::{is_cancel_just_pressed, is_confirm_just_pressed};
use party::ItemKind;

use app_state::{PartyState, SceneState};
use field_core::{Player, TilePosition};
use hud_ui::menu_style;

use crate::scene::{HokoraMenuPhase, HokoraResource};

/// 祠画面の入力処理システム
pub fn hokora_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut hokora_res: ResMut<HokoraResource>,
    mut next_state: ResMut<NextState<SceneState>>,
    party_state: Res<PartyState>,
    mut player_query: Query<&mut TilePosition, With<Player>>,
) {
    match hokora_res.phase.clone() {
        HokoraMenuPhase::MenuSelect => {
            // 上下でカーソル移動
            menu_style::handle_menu_navigation(&keyboard, &mut *hokora_res);

            if is_cancel_just_pressed(&keyboard) {
                next_state.set(SceneState::Exploring);
                return;
            }

            if is_confirm_just_pressed(&keyboard) {
                match hokora_res.selected_item {
                    0 => {
                        // 様子を見る
                        hokora_res.phase = HokoraMenuPhase::ShowMessage {
                            message: "ふるびた ほこらだ。\nふしぎな ちからを かんじる…".to_string(),
                        };
                    }
                    1 => {
                        // 扉を開ける
                        handle_open_door(&mut hokora_res, &party_state, &mut player_query);
                    }
                    _ => {
                        // 出る
                        next_state.set(SceneState::Exploring);
                    }
                }
            }
        }
        HokoraMenuPhase::ShowMessage { .. } => {
            if is_confirm_just_pressed(&keyboard) || is_cancel_just_pressed(&keyboard) {
                if hokora_res.warped {
                    // ワープ後はフィールドに遷移
                    next_state.set(SceneState::Exploring);
                } else {
                    // 通常メッセージはメニューに戻る
                    hokora_res.phase = HokoraMenuPhase::MenuSelect;
                }
            }
        }
    }
}

fn handle_open_door(
    hokora_res: &mut HokoraResource,
    party_state: &PartyState,
    player_query: &mut Query<&mut TilePosition, With<Player>>,
) {
    let required = ((hokora_res.hokora_index + 1) * 3) as u32;
    let total_fragments: u32 = party_state
        .members
        .iter()
        .map(|m| m.inventory.count(ItemKind::MoonFragment))
        .sum::<u32>()
        + party_state.bag.count(ItemKind::MoonFragment);

    if total_fragments < required {
        hokora_res.phase = HokoraMenuPhase::ShowMessage {
            message: format!(
                "とびらには ふしぎな ちからが\nつきのかけらが {}こ ひつようだ。",
                required
            ),
        };
        return;
    }

    let Some((dest_x, dest_y)) = hokora_res.warp_destination else {
        hokora_res.phase = HokoraMenuPhase::ShowMessage {
            message: "とびらが ひらいたが\nなにも おこらなかった…".to_string(),
        };
        return;
    };

    let Ok(mut tile_pos) = player_query.single_mut() else {
        return;
    };

    tile_pos.x = dest_x;
    tile_pos.y = dest_y;
    hokora_res.warped = true;
    hokora_res.phase = HokoraMenuPhase::ShowMessage {
        message: "つきのかけらが かがやき\nとびらが ひらいた！\nとおい ばしょに ワープした！".to_string(),
    };
}
