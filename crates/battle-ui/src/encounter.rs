use bevy::prelude::*;

use battle::should_encounter;

use app_state::AppState;
use components_ui::{OnBoat, Player, TilePosition};
use events_ui::TileEnteredEvent;
use shared_ui::MapDataResource;

/// プレイヤーがタイルに到着した際にエンカウント判定を行うシステム
/// PlayerArrivedEvent はSmoothMoveアニメーション完了時に発火するため、
/// 視覚的に到着してから判定される
pub fn check_encounter_system(
    mut events: MessageReader<TileEnteredEvent>,
    player_query: Query<(&TilePosition, Option<&OnBoat>), With<Player>>,
    map_data: Res<MapDataResource>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _event in events.read() {
        let Ok((tile_pos, on_boat)) = player_query.single() else {
            continue;
        };

        // 船乗車中はエンカウントなし
        if on_boat.is_some() {
            continue;
        }

        let terrain = map_data.grid[tile_pos.y][tile_pos.x];
        let random_value: f32 = rand::random();

        if should_encounter(terrain, random_value) {
            next_state.set(AppState::Battle);
            return;
        }
    }
}
