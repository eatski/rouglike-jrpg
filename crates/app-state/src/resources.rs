use bevy::prelude::*;

use std::collections::{HashMap, HashSet};

use party::{default_candidates, initial_party, ItemKind, PartyMember, RecruitCandidate};

/// パーティの永続的な状態を管理するリソース（戦闘間でHP/MPを引き継ぐ）
#[derive(Resource)]
pub struct PartyState {
    pub members: Vec<PartyMember>,
    pub gold: u32,
    /// 仲間候補の一覧（状態付き）
    pub candidates: Vec<RecruitCandidate>,
}

impl Default for PartyState {
    fn default() -> Self {
        let mut members = initial_party();
        members[0].inventory.add(ItemKind::Herb, 2);
        Self {
            members,
            gold: 100,
            candidates: default_candidates(),
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
}

/// フィールドメニュー開閉のマーカーリソース（存在=開、不在=閉）
#[derive(Resource)]
pub struct FieldMenuOpen;

/// 祠のワールドマップ座標を保持するリソース
#[derive(Resource, Default)]
pub struct HokoraPositions {
    pub positions: Vec<(usize, usize)>,
    /// 各祠のワープ先座標（positionsと同じ順序）
    pub warp_destinations: Vec<(usize, usize)>,
}

/// 大陸1にある洞窟のワールドマップ座標
#[derive(Resource, Default)]
pub struct Continent1CavePositions {
    pub positions: Vec<(usize, usize)>,
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
