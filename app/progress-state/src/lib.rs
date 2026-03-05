use bevy::prelude::*;

use std::collections::{HashMap, HashSet};

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

pub struct ProgressStatePlugin;

impl Plugin for ProgressStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OpenedChests>();
    }
}
