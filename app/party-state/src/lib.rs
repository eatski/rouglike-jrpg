use bevy::prelude::*;

use item::{Inventory, BAG_CAPACITY};
use item_data::ItemKey;
use party::{default_candidates, initial_party, CharacterParamTable, PartyMember, RecruitCandidate};

/// パーティの永続的な状態を管理するリソース（戦闘間でHP/MPを引き継ぐ）
#[derive(Resource)]
pub struct PartyState {
    pub members: Vec<PartyMember>,
    pub gold: u32,
    /// 仲間候補の一覧（状態付き）
    pub candidates: Vec<RecruitCandidate>,
    /// パーティ共有の袋
    pub bag: Inventory<ItemKey>,
}

impl PartyState {
    pub fn new(table: &CharacterParamTable) -> Self {
        let mut members = initial_party(table);
        members[0].inventory.add(ItemKey::Herb, 2);
        Self {
            members,
            gold: 100,
            candidates: default_candidates(),
            bag: Inventory::with_capacity(BAG_CAPACITY),
        }
    }
}

/// キャラクターパラメータの Bevy Resource ラッパー
#[derive(Resource)]
pub struct CharacterParams(pub party::CharacterParamTable);

impl std::ops::Deref for CharacterParams {
    type Target = party::CharacterParamTable;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
