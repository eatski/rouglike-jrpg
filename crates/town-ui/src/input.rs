use bevy::prelude::*;

use input_ui::{is_cancel_just_pressed, is_confirm_just_pressed, is_down_just_pressed, is_up_just_pressed};
use party::{shop_items, talk_to_candidate, ItemKind, PartyMember, TalkResult};
use town::{buy_item, candidate_first_dialogue, candidate_join_dialogue, cave_hint_dialogue, heal_party, BuyResult};

use app_state::SceneState;
use movement_ui::{Player, TilePosition};
use shared_ui::{ActiveMap, PartyState, RecruitmentMap};

use crate::scene::{TownMenuPhase, TownResource};

/// 町画面の入力処理システム
pub fn town_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut town_res: ResMut<TownResource>,
    mut next_state: ResMut<NextState<SceneState>>,
    mut party_state: ResMut<PartyState>,
    mut recruitment_map: ResMut<RecruitmentMap>,
    active_map: Res<ActiveMap>,
    player_query: Query<&TilePosition, With<Player>>,
) {
    match town_res.phase.clone() {
        TownMenuPhase::MenuSelect => {
            // 上下でカーソル移動
            if is_up_just_pressed(&keyboard) && town_res.selected_item > 0 {
                town_res.selected_item -= 1;
            }
            if is_down_just_pressed(&keyboard) && town_res.selected_item < 3 {
                town_res.selected_item += 1;
            }

            if is_confirm_just_pressed(&keyboard) {
                match town_res.selected_item {
                    0 => {
                        // やどや → HP/MPを全回復
                        heal_party(&mut party_state.members);
                        town_res.phase = TownMenuPhase::ShowMessage {
                            message: "ゆっくり やすんだ。\nHP と MP が かいふくした！".to_string(),
                        };
                    }
                    1 => {
                        // 道具屋 → ショップ画面へ
                        town_res.phase = TownMenuPhase::ShopSelect { selected: 0 };
                    }
                    2 => {
                        // 話を聞く → 仲間候補がいればイベント、いなければ洞窟ヒント
                        if let Ok(pos) = player_query.single() {
                            let town_pos = (pos.x, pos.y);
                            if let Some(&candidate_idx) =
                                recruitment_map.town_to_candidate.get(&town_pos)
                            {
                                let result =
                                    talk_to_candidate(&mut party_state.candidates[candidate_idx]);
                                let kind = party_state.candidates[candidate_idx].kind;
                                match result {
                                    TalkResult::BecameAcquaintance => {
                                        let msg = candidate_first_dialogue(kind);
                                        // 候補を次の街に移動
                                        recruitment_map.town_to_candidate.remove(&town_pos);
                                        if let Some(&second_town) =
                                            recruitment_map.candidate_second_town.get(&candidate_idx)
                                        {
                                            recruitment_map
                                                .town_to_candidate
                                                .insert(second_town, candidate_idx);
                                        }
                                        town_res.phase =
                                            TownMenuPhase::RecruitMessage { message: msg };
                                    }
                                    TalkResult::Recruited => {
                                        party_state.members.push(PartyMember::from_kind(kind));
                                        recruitment_map.town_to_candidate.remove(&town_pos);
                                        recruitment_map.candidate_second_town.remove(&candidate_idx);
                                        let msg = candidate_join_dialogue(kind);
                                        town_res.phase =
                                            TownMenuPhase::RecruitMessage { message: msg };
                                    }
                                    TalkResult::AlreadyRecruited => {}
                                }
                            } else {
                                let dialogue =
                                    cave_hint_dialogue(&active_map.grid, pos.x, pos.y);
                                town_res.phase =
                                    TownMenuPhase::ShowMessage { message: dialogue };
                            }
                        } else {
                            let dialogue = cave_hint_dialogue(&active_map.grid, 0, 0);
                            town_res.phase = TownMenuPhase::ShowMessage { message: dialogue };
                        }
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
            if is_confirm_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::MenuSelect;
            }
        }
        TownMenuPhase::ShopSelect { selected } => {
            let items = shop_items();
            let max_index = items.len().saturating_sub(1);

            // 上下でカーソル移動
            if is_up_just_pressed(&keyboard) && selected > 0 {
                town_res.phase = TownMenuPhase::ShopSelect {
                    selected: selected - 1,
                };
            }
            if is_down_just_pressed(&keyboard) && selected < max_index {
                town_res.phase = TownMenuPhase::ShopSelect {
                    selected: selected + 1,
                };
            }

            // キャンセル → メインメニューに戻る
            if is_cancel_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::MenuSelect;
            }

            // 決定 → ゴールドチェック後キャラ選択へ
            if is_confirm_just_pressed(&keyboard) {
                let item = items[selected];
                if party_state.gold < item.price() {
                    town_res.phase = TownMenuPhase::ShopMessage {
                        message: "おかねが たりない！".to_string(),
                    };
                } else {
                    town_res.phase = TownMenuPhase::ShopCharacterSelect {
                        item,
                        selected: 0,
                    };
                }
            }
        }
        TownMenuPhase::ShopCharacterSelect { item, selected } => {
            handle_shop_character_select(
                &keyboard,
                &mut town_res,
                &mut party_state,
                item,
                selected,
            );
        }
        TownMenuPhase::ShopMessage { .. } => {
            // メッセージ確認後、ショップ選択に戻る
            if is_confirm_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::ShopSelect { selected: 0 };
            }
        }
        TownMenuPhase::RecruitMessage { .. } => {
            // 仲間メッセージ確認後、メインメニューに戻る
            if is_confirm_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::MenuSelect;
            }
        }
    }
}

fn handle_shop_character_select(
    keyboard: &ButtonInput<KeyCode>,
    town_res: &mut TownResource,
    party_state: &mut PartyState,
    item: ItemKind,
    selected: usize,
) {
    let max_index = party_state.members.len().saturating_sub(1);

    // 上下でカーソル移動
    if is_up_just_pressed(keyboard) && selected > 0 {
        town_res.phase = TownMenuPhase::ShopCharacterSelect {
            item,
            selected: selected - 1,
        };
    }
    if is_down_just_pressed(keyboard) && selected < max_index {
        town_res.phase = TownMenuPhase::ShopCharacterSelect {
            item,
            selected: selected + 1,
        };
    }

    // キャンセル → ショップ選択に戻る
    if is_cancel_just_pressed(keyboard) {
        town_res.phase = TownMenuPhase::ShopSelect { selected: 0 };
        return;
    }

    // 決定 → 購入処理
    if is_confirm_just_pressed(keyboard) {
        let member_name = party_state.members[selected].kind.name();
        match buy_item(item, party_state.gold, &mut party_state.members[selected].inventory) {
            BuyResult::Success { remaining_gold } => {
                party_state.gold = remaining_gold;
                town_res.phase = TownMenuPhase::ShopMessage {
                    message: format!("{}が {} を てにいれた！", member_name, item.name()),
                };
            }
            BuyResult::InsufficientGold => {
                town_res.phase = TownMenuPhase::ShopMessage {
                    message: "おかねが たりない！".to_string(),
                };
            }
            BuyResult::InventoryFull => {
                town_res.phase = TownMenuPhase::ShopMessage {
                    message: format!("{}の もちものが いっぱいだ！", member_name),
                };
            }
        }
    }
}
