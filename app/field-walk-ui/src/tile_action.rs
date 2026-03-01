use bevy::prelude::*;

use terrain::TileAction;

use app_state::SceneState;
use field_core::{ActiveMap, OnBoat, Player, TilePosition};
use crate::TileEnteredEvent;

/// プレイヤーがフィールドのタイルに歩いて到着した際に、
/// 地形に応じた状態遷移を行うシステム
pub fn check_tile_action_system(
    mut events: MessageReader<TileEnteredEvent>,
    player_query: Query<(&TilePosition, Option<&OnBoat>), With<Player>>,
    active_map: Res<ActiveMap>,
    mut next_state: ResMut<NextState<SceneState>>,
) {
    for _event in events.read() {
        let Ok((tile_pos, on_boat)) = player_query.single() else {
            continue;
        };

        // 船乗車中は地形アクションなし
        if on_boat.is_some() {
            continue;
        }

        match active_map.tile_action_at(tile_pos.x, tile_pos.y) {
            TileAction::EnterTown => {
                next_state.set(SceneState::Town);
                return;
            }
            TileAction::EnterCave => {
                next_state.set(SceneState::Cave);
                return;
            }
            TileAction::EnterBossCave => {
                next_state.set(SceneState::BossCave);
                return;
            }
            TileAction::EnterHokora => {
                next_state.set(SceneState::Hokora);
                return;
            }
            TileAction::ExitCave | TileAction::None => {}
        }
    }
}
