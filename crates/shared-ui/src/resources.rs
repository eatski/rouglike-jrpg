use bevy::prelude::*;

use crate::constants::{MOVEMENT_INITIAL_DELAY, MOVEMENT_REPEAT_INTERVAL};
use party::{default_party, PartyMember};
use terrain::Terrain;
use world::map::MapData;

/// パーティの永続的な状態を管理するリソース（戦闘間でHP/MPを引き継ぐ）
#[derive(Resource)]
pub struct PartyState {
    pub members: Vec<PartyMember>,
}

impl Default for PartyState {
    fn default() -> Self {
        Self {
            members: default_party(),
        }
    }
}

/// 移動状態を管理するリソース
#[derive(Resource)]
pub struct MovementState {
    pub timer: Timer,
    pub initial_delay: Timer,
    pub is_repeating: bool,
    pub last_direction: (i32, i32),
    /// 先に押された軸（Some(true)=X軸、Some(false)=Y軸、None=なし）
    pub first_axis: Option<bool>,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(MOVEMENT_REPEAT_INTERVAL, TimerMode::Repeating),
            initial_delay: Timer::from_seconds(MOVEMENT_INITIAL_DELAY, TimerMode::Once),
            is_repeating: false,
            last_direction: (0, 0),
            first_axis: None,
        }
    }
}

/// マップデータをBevyリソースとしてラップする
#[derive(Resource)]
pub struct MapDataResource {
    pub grid: Vec<Vec<Terrain>>,
    pub spawn_position: (usize, usize),
}

impl From<MapData> for MapDataResource {
    fn from(map_data: MapData) -> Self {
        Self {
            grid: map_data.grid,
            spawn_position: map_data.spawn_position,
        }
    }
}
