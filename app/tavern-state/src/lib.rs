use bevy::prelude::*;

use std::collections::{HashMap, HashSet};

use item_data::ItemKey;

/// 居酒屋ヒントの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TavernHintKind {
    Cave,
    Hokora,
    Companion,
    Bounty,
}

/// 居酒屋で聞いたヒントの既読管理リソース
#[derive(Resource, Default)]
pub struct HeardTavernHints {
    /// key: 町座標, value: 聞いたヒント種類の集合
    pub heard: HashMap<(usize, usize), HashSet<TavernHintKind>>,
}

/// 居酒屋で受けた買い取り依頼を管理するリソース
#[derive(Resource, Default)]
pub struct TavernBounties {
    /// 居酒屋で依頼を聞いた街 → 対象アイテム
    pub active: HashMap<(usize, usize), ItemKey>,
}

pub struct TavernStatePlugin;

impl Plugin for TavernStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TavernBounties>();
    }
}
