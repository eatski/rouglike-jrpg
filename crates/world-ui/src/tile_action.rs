use bevy::prelude::*;

use terrain::TileAction;

use app_state::AppState;
use movement_ui::{OnBoat, Player, TileEnteredEvent, TilePosition};
use shared_ui::MapDataResource;

/// プレイヤーがフィールドのタイルに歩いて到着した際に、
/// 地形に応じた状態遷移を行うシステム
pub fn check_tile_action_system(
    mut events: MessageReader<TileEnteredEvent>,
    player_query: Query<(&TilePosition, Option<&OnBoat>), With<Player>>,
    map_data: Res<MapDataResource>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _event in events.read() {
        let Ok((tile_pos, on_boat)) = player_query.single() else {
            continue;
        };

        // 船乗車中は地形アクションなし
        if on_boat.is_some() {
            continue;
        }

        let terrain = map_data.grid[tile_pos.y][tile_pos.x];
        match terrain.tile_action() {
            TileAction::EnterTown => {
                next_state.set(AppState::Town);
                return;
            }
            TileAction::EnterCave => {
                next_state.set(AppState::Cave);
                return;
            }
            TileAction::None => {}
        }
    }
}
