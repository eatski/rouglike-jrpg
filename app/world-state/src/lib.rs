use bevy::prelude::*;

use std::collections::HashMap;

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
