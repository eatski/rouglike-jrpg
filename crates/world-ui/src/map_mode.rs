use bevy::prelude::*;

use field_walk::exploration::{ExplorationMap, VIEW_RADIUS};
use terrain::{MAP_HEIGHT, MAP_WIDTH};

use field_core::{Player, TilePosition};
use field_walk_ui::PlayerMovedEvent;

use crate::resources::SpawnPosition;

/// 探索データを管理するリソース
#[derive(Resource)]
pub struct ExplorationData {
    pub map: ExplorationMap,
}

/// マップ生成時にExplorationDataを初期化するシステム
pub fn init_exploration_system(mut commands: Commands, spawn_pos: Res<SpawnPosition>) {
    let mut exploration_map = ExplorationMap::new(MAP_WIDTH, MAP_HEIGHT);

    // プレイヤーの初期位置周辺を探索済みにする
    exploration_map.update_visibility(spawn_pos.x, spawn_pos.y, VIEW_RADIUS);

    commands.insert_resource(ExplorationData {
        map: exploration_map,
    });
}

/// プレイヤー移動時に視界を更新するシステム
pub fn update_exploration_system(
    mut exploration_data: ResMut<ExplorationData>,
    player_query: Query<&TilePosition, With<Player>>,
    mut moved_events: MessageReader<PlayerMovedEvent>,
) {
    for _event in moved_events.read() {
        if let Ok(tile_pos) = player_query.single() {
            exploration_data
                .map
                .update_visibility(tile_pos.x, tile_pos.y, VIEW_RADIUS);
        }
    }
}
