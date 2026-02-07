use bevy::prelude::*;

use game::battle::should_encounter;

use crate::app_state::AppState;
use crate::components::{OnBoat, Player, TilePosition};
use crate::events::PlayerMovedEvent;
use crate::resources::MapDataResource;

/// プレイヤー移動時にエンカウント判定を行うシステム
pub fn check_encounter_system(
    mut events: MessageReader<PlayerMovedEvent>,
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
