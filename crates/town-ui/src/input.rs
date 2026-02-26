use bevy::prelude::*;
use rand::prelude::SliceRandom;

use input_ui::{is_cancel_just_pressed, is_confirm_just_pressed, is_down_just_pressed, is_up_just_pressed};
use party::{talk_to_candidate, PartyMember, TalkResult};
use town::{buy_item, buy_weapon, candidate_first_dialogue, candidate_join_dialogue, cave_hint_dialogue, companion_hint_dialogue, heal_party, hokora_hint_dialogue, sell_item, BuyResult, BuyWeaponResult, SellResult, INN_PRICE, TAVERN_PRICE};
use town::{tavern_bounty_item, bounty_offer_dialogue, bounty_has_item_dialogue, bounty_sold_dialogue, sell_bounty_item};

use app_state::SceneState;
use field_core::{ActiveMap, Player, TilePosition};
use app_state::{ContinentMap, HeardTavernHints, PartyState, RecruitmentMap, TavernBounties, TavernHintKind};

use crate::scene::{build_town_commands, shop_goods, ShopGoods, TownCommand, TownMenuPhase, TownResource};

/// 町画面の入力処理システム
#[allow(clippy::too_many_arguments)]
pub fn town_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut town_res: ResMut<TownResource>,
    mut next_state: ResMut<NextState<SceneState>>,
    mut party_state: ResMut<PartyState>,
    mut recruitment_map: ResMut<RecruitmentMap>,
    active_map: Res<ActiveMap>,
    player_query: Query<&TilePosition, With<Player>>,
    mut heard_hints: ResMut<HeardTavernHints>,
    continent_map: Option<Res<ContinentMap>>,
    mut tavern_bounties: ResMut<TavernBounties>,
) {
    match town_res.phase.clone() {
        TownMenuPhase::MenuSelect => {
            let max_index = town_res.commands.len().saturating_sub(1);
            // 上下でカーソル移動
            if is_up_just_pressed(&keyboard) {
                town_res.selected_item = if town_res.selected_item > 0 { town_res.selected_item - 1 } else { max_index };
            }
            if is_down_just_pressed(&keyboard) {
                town_res.selected_item = if town_res.selected_item < max_index { town_res.selected_item + 1 } else { 0 };
            }

            if is_confirm_just_pressed(&keyboard) {
                match &town_res.commands[town_res.selected_item].clone() {
                    TownCommand::Inn => {
                        // やどや → ゴールド消費してHP/MPを全回復
                        if party_state.gold < INN_PRICE {
                            town_res.phase = TownMenuPhase::ShowMessage {
                                message: "おかねが たりない！".to_string(),
                            };
                        } else {
                            party_state.gold -= INN_PRICE;
                            heal_party(&mut party_state.members);
                            town_res.phase = TownMenuPhase::ShowMessage {
                                message: format!("{}G はらって ゆっくり やすんだ。\nHP と MP が かいふくした！", INN_PRICE),
                            };
                        }
                    }
                    TownCommand::Shop => {
                        // よろず屋 → かう/うる選択へ
                        town_res.phase = TownMenuPhase::ShopModeSelect { selected: 0 };
                    }
                    TownCommand::Tavern => {
                        // 居酒屋 → 仲間候補がいればイベント、いなければヒント（ランダム1つ）
                        if let Ok(pos) = player_query.single() {
                            let town_pos = (pos.x, pos.y);
                            if let Some(&candidate_idx) =
                                recruitment_map.town_to_candidate.get(&town_pos)
                            {
                                // 仲間候補がいる場合: ゴールド消費して会話
                                if party_state.gold < TAVERN_PRICE {
                                    town_res.phase = TownMenuPhase::ShowMessage {
                                        message: "おかねが たりない！".to_string(),
                                    };
                                } else {
                                    party_state.gold -= TAVERN_PRICE;
                                    let result =
                                        talk_to_candidate(&mut party_state.candidates[candidate_idx]);
                                    let kind = party_state.candidates[candidate_idx].kind;
                                    match result {
                                        TalkResult::BecameAcquaintance => {
                                            let msg = candidate_first_dialogue(kind);
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
                                }
                            } else {
                                // ヒント: 未聞のものからランダムで1つ選んで表示
                                let heard_set = heard_hints.heard.entry(town_pos).or_default();
                                let mut unheard = Vec::new();
                                if !heard_set.contains(&TavernHintKind::Cave) {
                                    unheard.push(TavernHintKind::Cave);
                                }
                                if !heard_set.contains(&TavernHintKind::Hokora) {
                                    unheard.push(TavernHintKind::Hokora);
                                }
                                // 同じ大陸内に仲間候補がいる場合のみ Companion ヒントを候補に含める
                                let companion_towns = collect_companion_towns(
                                    pos,
                                    &recruitment_map,
                                    &continent_map,
                                    &party_state.candidates,
                                );
                                if !companion_towns.is_empty()
                                    && !heard_set.contains(&TavernHintKind::Companion)
                                {
                                    unheard.push(TavernHintKind::Companion);
                                }
                                // Bounty ヒントを候補に含める
                                if !heard_set.contains(&TavernHintKind::Bounty) {
                                    unheard.push(TavernHintKind::Bounty);
                                }

                                if unheard.is_empty() {
                                    // 全て既読 → ゴールド消費なし
                                    town_res.phase = TownMenuPhase::ShowMessage {
                                        message: "もう あたらしい はなしは ないな".to_string(),
                                    };
                                } else if party_state.gold < TAVERN_PRICE {
                                    town_res.phase = TownMenuPhase::ShowMessage {
                                        message: "おかねが たりない！".to_string(),
                                    };
                                } else {
                                    party_state.gold -= TAVERN_PRICE;
                                    let mut rng = rand::thread_rng();
                                    let &chosen = unheard.choose(&mut rng).unwrap();
                                    heard_set.insert(chosen);
                                    let cf = continent_map.as_ref().and_then(|cm| {
                                        cm.get(pos.x, pos.y).map(|cid| (cm.as_raw(), cid))
                                    });
                                    let dialogue = match chosen {
                                        TavernHintKind::Cave => {
                                            cave_hint_dialogue(&active_map.structures, pos.x, pos.y, cf)
                                        }
                                        TavernHintKind::Hokora => {
                                            hokora_hint_dialogue(&active_map.structures, pos.x, pos.y, cf)
                                        }
                                        TavernHintKind::Companion => {
                                            companion_hint_dialogue(pos.x, pos.y, &companion_towns)
                                                .unwrap_or_else(|| "もう あたらしい はなしは ないな".to_string())
                                        }
                                        TavernHintKind::Bounty => {
                                            let item = tavern_bounty_item(pos.x, pos.y);
                                            let has_item = party_state.members.iter().any(|m| m.inventory.count(item) > 0);
                                            tavern_bounties.active.insert(town_pos, item);
                                            town_res.commands = build_town_commands(Some(item));
                                            if has_item {
                                                bounty_has_item_dialogue(item)
                                            } else {
                                                bounty_offer_dialogue(item)
                                            }
                                        }
                                    };
                                    town_res.phase =
                                        TownMenuPhase::ShowMessage { message: dialogue };
                                }
                            }
                        }
                    }
                    TownCommand::SellBounty(item) => {
                        // 買い取り依頼 → キャラクター選択へ
                        town_res.phase = TownMenuPhase::BountyCharacterSelect {
                            item: *item,
                            selected: 0,
                        };
                    }
                    TownCommand::Leave => {
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
        TownMenuPhase::ShopModeSelect { selected } => {
            // かう/うる選択
            if is_up_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::ShopModeSelect {
                    selected: if selected > 0 { selected - 1 } else { 1 },
                };
            }
            if is_down_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::ShopModeSelect {
                    selected: if selected < 1 { selected + 1 } else { 0 },
                };
            }
            if is_cancel_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::MenuSelect;
            }
            if is_confirm_just_pressed(&keyboard) {
                match selected {
                    0 => {
                        // かう → 商品選択へ
                        town_res.phase = TownMenuPhase::ShopSelect { selected: 0 };
                    }
                    _ => {
                        // うる → 売却キャラ選択へ
                        town_res.phase = TownMenuPhase::SellCharacterSelect { selected: 0 };
                    }
                }
            }
        }
        TownMenuPhase::ShopSelect { selected } => {
            let goods_list = shop_goods();
            let max_index = goods_list.len().saturating_sub(1);

            if is_up_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::ShopSelect {
                    selected: if selected > 0 { selected - 1 } else { max_index },
                };
            }
            if is_down_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::ShopSelect {
                    selected: if selected < max_index { selected + 1 } else { 0 },
                };
            }

            // キャンセル → かう/うる選択に戻る
            if is_cancel_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::ShopModeSelect { selected: 0 };
            }

            if is_confirm_just_pressed(&keyboard) {
                let goods = goods_list[selected];
                if party_state.gold < goods.price() {
                    town_res.phase = TownMenuPhase::ShopMessage {
                        message: "おかねが たりない！".to_string(),
                    };
                } else {
                    town_res.phase = TownMenuPhase::ShopCharacterSelect {
                        goods,
                        selected: 0,
                    };
                }
            }
        }
        TownMenuPhase::ShopCharacterSelect { goods, selected } => {
            handle_shop_character_select(
                &keyboard,
                &mut town_res,
                &mut party_state,
                goods,
                selected,
            );
        }
        TownMenuPhase::ShopMessage { .. } => {
            // メッセージ確認後、かう/うる選択に戻る
            if is_confirm_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::ShopModeSelect { selected: 0 };
            }
        }
        TownMenuPhase::SellCharacterSelect { selected } => {
            handle_sell_character_select(
                &keyboard,
                &mut town_res,
                &party_state,
                selected,
            );
        }
        TownMenuPhase::SellItemSelect {
            member_index,
            selected,
        } => {
            handle_sell_item_select(
                &keyboard,
                &mut town_res,
                &mut party_state,
                member_index,
                selected,
            );
        }
        TownMenuPhase::RecruitMessage { .. } => {
            if is_confirm_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::MenuSelect;
            }
        }
        TownMenuPhase::BountyCharacterSelect { item, selected } => {
            let max_index = party_state.members.len().saturating_sub(1);

            if is_up_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::BountyCharacterSelect {
                    item,
                    selected: if selected > 0 { selected - 1 } else { max_index },
                };
            }
            if is_down_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::BountyCharacterSelect {
                    item,
                    selected: if selected < max_index { selected + 1 } else { 0 },
                };
            }

            if is_cancel_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::MenuSelect;
                return;
            }

            if is_confirm_just_pressed(&keyboard) {
                match sell_bounty_item(item, &mut party_state.members[selected].inventory) {
                    SellResult::Success { earned_gold } => {
                        party_state.gold += earned_gold;
                        town_res.phase = TownMenuPhase::BountyMessage {
                            message: bounty_sold_dialogue(item),
                        };
                    }
                    SellResult::NotOwned => {
                        let name = party_state.members[selected].kind.name();
                        town_res.phase = TownMenuPhase::BountyMessage {
                            message: format!("{} は {} を もっていない！", name, item.name()),
                        };
                    }
                    SellResult::CannotSell => {}
                }
            }
        }
        TownMenuPhase::BountyMessage { .. } => {
            if is_confirm_just_pressed(&keyboard) {
                town_res.phase = TownMenuPhase::MenuSelect;
            }
        }
    }
}

/// 同じ大陸内の仲間候補がいる街を収集する
fn collect_companion_towns(
    pos: &TilePosition,
    recruitment_map: &RecruitmentMap,
    continent_map: &Option<Res<ContinentMap>>,
    candidates: &[party::RecruitCandidate],
) -> Vec<(usize, usize, party::PartyMemberKind)> {
    let current_continent = continent_map
        .as_ref()
        .and_then(|cm| cm.get(pos.x, pos.y));
    let Some(current_cid) = current_continent else {
        return Vec::new();
    };
    recruitment_map
        .town_to_candidate
        .iter()
        .filter(|&(&(tx, ty), _)| {
            (tx, ty) != (pos.x, pos.y)
                && continent_map
                    .as_ref()
                    .and_then(|cm| cm.get(tx, ty))
                    == Some(current_cid)
        })
        .map(|(&(tx, ty), &idx)| {
            let kind = candidates[idx].kind;
            (tx, ty, kind)
        })
        .collect()
}

fn handle_shop_character_select(
    keyboard: &ButtonInput<KeyCode>,
    town_res: &mut TownResource,
    party_state: &mut PartyState,
    goods: ShopGoods,
    selected: usize,
) {
    let max_index = party_state.members.len().saturating_sub(1);

    // 上下でカーソル移動
    if is_up_just_pressed(keyboard) {
        town_res.phase = TownMenuPhase::ShopCharacterSelect {
            goods,
            selected: if selected > 0 { selected - 1 } else { max_index },
        };
    }
    if is_down_just_pressed(keyboard) {
        town_res.phase = TownMenuPhase::ShopCharacterSelect {
            goods,
            selected: if selected < max_index { selected + 1 } else { 0 },
        };
    }

    // キャンセル → 商品選択に戻る
    if is_cancel_just_pressed(keyboard) {
        town_res.phase = TownMenuPhase::ShopSelect { selected: 0 };
        return;
    }

    // 決定 → 購入処理
    if is_confirm_just_pressed(keyboard) {
        let member_name = party_state.members[selected].kind.name();
        match goods {
            ShopGoods::Item(item) => {
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
            ShopGoods::Weapon(weapon) => {
                match buy_weapon(weapon, party_state.gold, &mut party_state.members[selected]) {
                    BuyWeaponResult::Success { remaining_gold } => {
                        party_state.gold = remaining_gold;
                        town_res.phase = TownMenuPhase::ShopMessage {
                            message: format!("{}が {} を そうびした！", member_name, weapon.name()),
                        };
                    }
                    BuyWeaponResult::InsufficientGold => {
                        town_res.phase = TownMenuPhase::ShopMessage {
                            message: "おかねが たりない！".to_string(),
                        };
                    }
                }
            }
        }
    }
}

fn handle_sell_character_select(
    keyboard: &ButtonInput<KeyCode>,
    town_res: &mut TownResource,
    party_state: &PartyState,
    selected: usize,
) {
    let max_index = party_state.members.len().saturating_sub(1);

    if is_up_just_pressed(keyboard) {
        town_res.phase = TownMenuPhase::SellCharacterSelect {
            selected: if selected > 0 { selected - 1 } else { max_index },
        };
    }
    if is_down_just_pressed(keyboard) {
        town_res.phase = TownMenuPhase::SellCharacterSelect {
            selected: if selected < max_index { selected + 1 } else { 0 },
        };
    }

    if is_cancel_just_pressed(keyboard) {
        town_res.phase = TownMenuPhase::ShopModeSelect { selected: 1 };
        return;
    }

    if is_confirm_just_pressed(keyboard) {
        let sellable: Vec<_> = party_state.members[selected]
            .inventory
            .owned_items()
            .into_iter()
            .filter(|i| i.sell_price() > 0)
            .collect();
        if sellable.is_empty() {
            town_res.phase = TownMenuPhase::ShopMessage {
                message: "うれる アイテムが ない！".to_string(),
            };
        } else {
            town_res.phase = TownMenuPhase::SellItemSelect {
                member_index: selected,
                selected: 0,
            };
        }
    }
}

fn handle_sell_item_select(
    keyboard: &ButtonInput<KeyCode>,
    town_res: &mut TownResource,
    party_state: &mut PartyState,
    member_index: usize,
    selected: usize,
) {
    let sellable: Vec<_> = party_state.members[member_index]
        .inventory
        .owned_items()
        .into_iter()
        .filter(|i| i.sell_price() > 0)
        .collect();

    if sellable.is_empty() {
        town_res.phase = TownMenuPhase::SellCharacterSelect { selected: member_index };
        return;
    }

    let max_index = sellable.len().saturating_sub(1);

    if is_up_just_pressed(keyboard) {
        town_res.phase = TownMenuPhase::SellItemSelect {
            member_index,
            selected: if selected > 0 { selected - 1 } else { max_index },
        };
    }
    if is_down_just_pressed(keyboard) {
        town_res.phase = TownMenuPhase::SellItemSelect {
            member_index,
            selected: if selected < max_index { selected + 1 } else { 0 },
        };
    }

    if is_cancel_just_pressed(keyboard) {
        town_res.phase = TownMenuPhase::SellCharacterSelect { selected: member_index };
        return;
    }

    if is_confirm_just_pressed(keyboard) {
        let item = sellable[selected];
        match sell_item(item, &mut party_state.members[member_index].inventory) {
            SellResult::Success { earned_gold } => {
                party_state.gold += earned_gold;
                town_res.phase = TownMenuPhase::ShopMessage {
                    message: format!(
                        "{} を {}Gで うった！",
                        item.name(),
                        earned_gold
                    ),
                };
            }
            SellResult::CannotSell => {
                town_res.phase = TownMenuPhase::ShopMessage {
                    message: format!("{} は うれない！", item.name()),
                };
            }
            SellResult::NotOwned => {
                town_res.phase = TownMenuPhase::ShopMessage {
                    message: format!(
                        "{} を もっていない！",
                        item.name()
                    ),
                };
            }
        }
    }
}
