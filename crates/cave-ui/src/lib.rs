mod input;
mod scene;

use bevy::prelude::*;
use app_state::{BattleState, SceneState};
use movement_ui::{start_bounce, update_bounce};
use world_ui::{camera_follow, check_encounter_system, reset_map_mode_system, toggle_map_mode_system};

pub use input::{
    cave_message_display_system, cave_message_input_system, cave_player_movement,
    check_chest_system, check_ladder_system, start_cave_smooth_move, update_cave_smooth_move,
};
pub use scene::{despawn_cave_entities, restore_field_from_cave, setup_cave_scene, update_cave_tiles, CaveTilePool};

pub struct CavePlugin;

impl Plugin for CavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SceneState::Cave), setup_cave_scene)
            .add_systems(
                Update,
                (
                    toggle_map_mode_system,
                    cave_player_movement,
                    start_cave_smooth_move,
                    ApplyDeferred,
                    update_cave_smooth_move,
                    start_bounce,
                    update_bounce,
                    update_cave_tiles,
                    camera_follow,
                    check_chest_system,
                    cave_message_input_system,
                    cave_message_display_system,
                    check_ladder_system,
                    check_encounter_system,
                )
                    .chain()
                    .run_if(in_state(SceneState::Cave).and(in_state(BattleState::None))),
            )
            .add_systems(OnExit(SceneState::Cave), (reset_map_mode_system, despawn_cave_entities, restore_field_from_cave).chain());
    }
}
