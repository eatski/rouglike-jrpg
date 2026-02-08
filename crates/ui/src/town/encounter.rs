use bevy::prelude::*;

use game::map::Terrain;

use crate::app_state::AppState;
use crate::components::{OnBoat, Player, TilePosition};
use crate::events::PlayerArrivedEvent;
use crate::resources::MapDataResource;

/// プレイヤーがタイルに到着した際に町タイルかどうかを判定し、町画面に遷移するシステム
/// PlayerArrivedEvent はSmoothMoveアニメーション完了時に発火するため、
/// 視覚的に到着してから遷移する
pub fn check_town_enter_system(
    mut events: MessageReader<PlayerArrivedEvent>,
    player_query: Query<(&TilePosition, Option<&OnBoat>), With<Player>>,
    map_data: Res<MapDataResource>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _event in events.read() {
        let Ok((tile_pos, on_boat)) = player_query.single() else {
            continue;
        };

        // 船乗車中は町に入らない
        if on_boat.is_some() {
            continue;
        }

        let terrain = map_data.grid[tile_pos.y][tile_pos.x];
        if terrain == Terrain::Town {
            next_state.set(AppState::Town);
            return;
        }
    }
}
