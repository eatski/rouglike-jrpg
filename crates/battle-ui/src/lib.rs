mod display;
mod field_menu;
mod input;
mod scene;

use bevy::prelude::*;
use app_state::BattleState;

pub use display::{battle_blink_system, battle_display_system, battle_shake_system};
pub use field_menu::{field_menu_display_system, field_menu_input_system};
pub use input::battle_input_system;
pub use scene::{
    cleanup_battle_scene, init_battle_resources, setup_battle_scene, setup_battle_scene_with_config,
    BattleGameState, BattlePhase, BattleSceneConfig, BattleUIState, PendingCommands,
};

use app_state::InField;
use cave_ui::cave_message_input_system;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        register_battle_all_systems(app);

        app.add_systems(
            Update,
            (
                field_menu_input_system.after(cave_message_input_system),
                field_menu_display_system,
            )
                .chain()
                .run_if(in_state(InField)),
        );
    }
}

/// 戦闘ロジックシステムのみ登録（テスト用: レンダリング非依存）
pub fn register_battle_logic_systems(app: &mut App) {
    app.add_systems(
        Update,
        battle_input_system.run_if(in_state(BattleState::Active)),
    );
    app.add_systems(OnExit(BattleState::Active), cleanup_battle_scene);
}

/// 全戦闘システム登録（本番用: レンダリング依存含む）
pub fn register_battle_all_systems(app: &mut App) {
    app.add_systems(OnEnter(BattleState::Active), setup_battle_scene);
    app.add_systems(
        Update,
        (
            battle_input_system,
            battle_display_system,
            battle_blink_system,
            battle_shake_system,
        )
            .chain()
            .run_if(in_state(BattleState::Active)),
    );
    app.add_systems(OnExit(BattleState::Active), cleanup_battle_scene);
}
