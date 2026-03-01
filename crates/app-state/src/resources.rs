use bevy::prelude::*;

use std::collections::{HashMap, HashSet};

use party::{default_candidates, initial_party, Inventory, ItemKind, PartyMember, RecruitCandidate, BAG_CAPACITY};

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
    positions: Vec<(usize, usize)>,
    warp_destinations: Vec<(usize, usize)>,
}

impl HokoraPositions {
    pub fn new(positions: Vec<(usize, usize)>, warp_destinations: Vec<(usize, usize)>) -> Self {
        assert!(!positions.is_empty(), "ホコラは最低1つ必要");
        assert_eq!(
            positions.len(),
            warp_destinations.len(),
            "positionsとwarp_destinationsの長さが不一致"
        );
        Self {
            positions,
            warp_destinations,
        }
    }

    /// 最寄りの祠インデックスを返す（コンストラクタで非空を保証済み）
    pub fn nearest(&self, player_x: usize, player_y: usize) -> usize {
        self.positions
            .iter()
            .enumerate()
            .min_by_key(|&(_, &(hx, hy))| {
                let dx = player_x as isize - hx as isize;
                let dy = player_y as isize - hy as isize;
                dx * dx + dy * dy
            })
            .map(|(i, _)| i)
            .unwrap() // positions非空はコンストラクタで保証
    }

    pub fn warp_destination(&self, index: usize) -> Option<(usize, usize)> {
        self.warp_destinations.get(index).copied()
    }

    pub fn positions(&self) -> &[(usize, usize)] {
        &self.positions
    }
}

/// 各大陸にある洞窟のワールドマップ座標（インデックス0=大陸1, 1=大陸2, 2=大陸3）
#[derive(Resource, Default)]
pub struct ContinentCavePositions {
    pub caves_by_continent: Vec<Vec<(usize, usize)>>,
}

/// 各タイルが属する大陸IDを保持するリソース（ワールドマップ用）
#[derive(Resource)]
pub struct ContinentMap {
    map: Vec<Vec<Option<u8>>>,
}

impl ContinentMap {
    pub fn new(map: Vec<Vec<Option<u8>>>) -> Self {
        Self { map }
    }

    /// 指定座標の大陸IDを返す（海や範囲外は None）
    pub fn get(&self, x: usize, y: usize) -> Option<u8> {
        self.map.get(y).and_then(|row| row.get(x)).copied().flatten()
    }

    /// 生データへの参照を返す（Bevy非依存のドメイン層に渡すため）
    pub fn as_raw(&self) -> &[Vec<Option<u8>>] {
        &self.map
    }
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
