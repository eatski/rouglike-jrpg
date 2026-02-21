use bevy::prelude::*;

use input_ui::{is_confirm_just_pressed, is_down_just_pressed, is_up_just_pressed};
use party::ItemKind;

use app_state::{PartyState, SceneState};
use movement_ui::{ActiveMap, Player, TilePosition};

use crate::scene::{HokoraMenuPhase, HokoraResource};

const MENU_ITEM_COUNT: usize = 3;

/// 祠画面の入力処理システム
pub fn hokora_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut hokora_res: ResMut<HokoraResource>,
    mut next_state: ResMut<NextState<SceneState>>,
    party_state: Res<PartyState>,
    mut player_query: Query<(&mut TilePosition, &mut Transform), With<Player>>,
    active_map: Res<ActiveMap>,
) {
    match hokora_res.phase.clone() {
        HokoraMenuPhase::MenuSelect => {
            // 上下でカーソル移動
            if is_up_just_pressed(&keyboard) {
                hokora_res.selected_item = if hokora_res.selected_item > 0 {
                    hokora_res.selected_item - 1
                } else {
                    MENU_ITEM_COUNT - 1
                };
            }
            if is_down_just_pressed(&keyboard) {
                hokora_res.selected_item = if hokora_res.selected_item < MENU_ITEM_COUNT - 1 {
                    hokora_res.selected_item + 1
                } else {
                    0
                };
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
                        handle_open_door(&mut hokora_res, &party_state, &mut player_query, &active_map);
                    }
                    _ => {
                        // 出る
                        next_state.set(SceneState::Exploring);
                    }
                }
            }
        }
        HokoraMenuPhase::ShowMessage { .. } => {
            // メッセージ表示中は Enter でメニューに戻る
            if is_confirm_just_pressed(&keyboard) {
                hokora_res.phase = HokoraMenuPhase::MenuSelect;
            }
        }
    }
}

const MOON_FRAGMENT_REQUIRED: u32 = 3;

fn handle_open_door(
    hokora_res: &mut HokoraResource,
    party_state: &PartyState,
    player_query: &mut Query<(&mut TilePosition, &mut Transform), With<Player>>,
    active_map: &ActiveMap,
) {
    let total_fragments: u32 = party_state
        .members
        .iter()
        .map(|m| m.inventory.count(ItemKind::MoonFragment))
        .sum();

    if total_fragments < MOON_FRAGMENT_REQUIRED {
        hokora_res.phase = HokoraMenuPhase::ShowMessage {
            message: "とびらには ふしぎな ちからが\nつきのかけらが 3こ ひつようだ。".to_string(),
        };
        return;
    }

    let Some((dest_x, dest_y)) = hokora_res.warp_destination else {
        hokora_res.phase = HokoraMenuPhase::ShowMessage {
            message: "とびらが ひらいたが\nなにも おこらなかった…".to_string(),
        };
        return;
    };

    let Ok((mut tile_pos, mut transform)) = player_query.single_mut() else {
        return;
    };

    tile_pos.x = dest_x;
    tile_pos.y = dest_y;
    let (world_x, world_y) = active_map.to_world(dest_x, dest_y);
    transform.translation.x = world_x;
    transform.translation.y = world_y;
    hokora_res.phase = HokoraMenuPhase::ShowMessage {
        message: "つきのかけらが かがやき\nとびらが ひらいた！\nとおい ばしょに ワープした！".to_string(),
    };
}
