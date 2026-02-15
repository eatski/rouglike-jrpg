use bevy::prelude::*;

use world::should_encounter;

use app_state::BattleState;
use movement_ui::{OnBoat, Player, TileEnteredEvent, TilePosition};
use shared_ui::ActiveMap;

/// プレイヤーがタイルに到着した際にエンカウント判定を行うシステム
pub fn check_encounter_system(
    mut events: MessageReader<TileEnteredEvent>,
    player_query: Query<(&TilePosition, Option<&OnBoat>), With<Player>>,
    active_map: Res<ActiveMap>,
    mut next_state: ResMut<NextState<BattleState>>,
) {
    for _event in events.read() {
        let Ok((tile_pos, on_boat)) = player_query.single() else {
            continue;
        };

        let terrain = active_map.terrain_at(tile_pos.x, tile_pos.y);
        let on_boat = on_boat.is_some();
        let random_value: f32 = rand::random();

        if should_encounter(terrain, on_boat, random_value) {
            next_state.set(BattleState::Active);
            return;
        }
    }
}
