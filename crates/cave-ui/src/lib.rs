mod input;
mod scene;

use bevy::prelude::*;
use app_state::{BattleState, SceneState};
use input_ui::InputSystemSet;
use field_walk_ui::{start_bounce, start_smooth_move, update_bounce, update_smooth_move};
use field_walk_ui::{camera_follow, check_encounter_system, reset_map_mode_system, toggle_map_mode_system, update_simple_tiles};

pub use input::{
    cave_message_display_system, cave_message_input_system, cave_player_movement,
    check_boss_proximity_system, check_chest_system,
    handle_cave_move_completed,
};
pub use scene::{
    despawn_cave_entities, restore_field_from_cave, setup_boss_cave_scene, setup_cave_scene,
    BossCaveState,
};

pub struct CavePlugin;

impl Plugin for CavePlugin {
    fn build(&self, app: &mut App) {
        // OnEnter
        app.add_systems(OnEnter(SceneState::Cave), setup_cave_scene);
        app.add_systems(OnEnter(SceneState::BossCave), setup_boss_cave_scene);

        // Cave/BossCave 共通Updateシステム（1チェーンに統合）
        // - check_chest_system: BossCaveでは宝箱エンティティがないためno-op
        // - check_encounter_system: BossCaveではencounter_rate=0.0のためno-op
        // - check_boss_proximity_system: CaveではBossEntityがないためno-op
        app.add_systems(
            Update,
            (
                toggle_map_mode_system,
                cave_player_movement,
                start_smooth_move,
                ApplyDeferred,
                update_smooth_move,
                handle_cave_move_completed,
                start_bounce,
                update_bounce,
                update_simple_tiles,
                camera_follow,
                check_chest_system,
                cave_message_input_system.in_set(InputSystemSet::MessageInput),
                cave_message_display_system,
                check_encounter_system,
                check_boss_proximity_system,
            )
                .chain()
                .run_if(
                    (in_state(SceneState::Cave).or(in_state(SceneState::BossCave)))
                        .and(in_state(BattleState::None)),
                ),
        );

        // OnExit
        app.add_systems(
            OnExit(SceneState::Cave),
            (reset_map_mode_system, despawn_cave_entities, restore_field_from_cave).chain(),
        );
        app.add_systems(
            OnExit(SceneState::BossCave),
            (reset_map_mode_system, despawn_cave_entities, restore_field_from_cave).chain(),
        );
    }
}
