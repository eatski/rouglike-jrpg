use bevy::prelude::*;

use std::collections::{HashMap, HashSet};

use item::{Inventory, ItemKind, BAG_CAPACITY};
use party::{default_candidates, initial_party, PartyMember, RecruitCandidate};

/// パーティの永続的な状態を管理するリソース（戦闘間でHP/MPを引き継ぐ）
#[derive(Resource)]
pub struct PartyState {
    pub members: Vec<PartyMember>,
    pub gold: u32,
    /// 仲間候補の一覧（状態付き）
    pub candidates: Vec<RecruitCandidate>,
    /// パーティ共有の袋
    pub bag: Inventory,
}

impl Default for PartyState {
    fn default() -> Self {
        let mut members = initial_party();
        members[0].inventory.add(ItemKind::Herb, 2);
        Self {
            members,
            gold: 100,
            candidates: default_candidates(),
            bag: Inventory::with_capacity(BAG_CAPACITY),
        }
    }
}

/// 街座標 → その街にいる仲間候補のインデックスのマッピング
#[derive(Resource, Default)]
pub struct RecruitmentMap {
    /// key: 街のタイル座標 (x, y), value: candidates配列のインデックス
    pub town_to_candidate: HashMap<(usize, usize), usize>,
    /// key: candidates配列のインデックス, value: 知り合い後の移動先街座標
    pub candidate_second_town: HashMap<usize, (usize, usize)>,
    /// 雇用可能なキャラ: key: 街座標 (x, y), value: candidates配列のインデックス
    pub hire_available: HashMap<(usize, usize), usize>,
}

/// フィールドメニュー開閉のマーカーリソース（存在=開、不在=閉）
#[derive(Resource)]
pub struct FieldMenuOpen;

/// 祠のワールドマップ座標を保持するリソース
#[derive(Resource)]
pub struct HokoraPositions {
    pub positions: Vec<(usize, usize)>,
    pub warp_destinations: Vec<(usize, usize)>,
}

/// 各大陸にある洞窟のワールドマップ座標（インデックス0=大陸1, 1=大陸2, 2=大陸3）
#[derive(Resource, Default)]
pub struct ContinentCavePositions {
    pub caves_by_continent: Vec<Vec<(usize, usize)>>,
}

/// 各タイルが属する大陸IDを保持するリソース（ワールドマップ用）
#[derive(Resource)]
pub struct ContinentMap {
    pub map: Vec<Vec<Option<u8>>>,
}

/// 現在のエンカウントゾーン（戦闘開始時にどの敵が出現するかを決定する）
#[derive(Resource, Clone, Default)]
pub struct EncounterZone {
    pub continent_id: u8,
    pub is_cave: bool,
}

/// ボス撃破フラグ（存在=撃破済み）
#[derive(Resource)]
pub struct BossDefeated;

/// ボス戦闘トリガー（ボスに隣接した時に挿入、battle-uiが消費）
#[derive(Resource)]
pub struct BossBattlePending;

/// 取得済み宝箱を管理するリソース
/// key: ワールドマップ上の洞窟座標 (cave_x, cave_y)
/// value: その洞窟内で取得済みの宝箱インデックスの集合
#[derive(Resource, Default)]
pub struct OpenedChests {
    pub chests: HashMap<(usize, usize), HashSet<usize>>,
}

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
    pub active: HashMap<(usize, usize), ItemKind>,
}
