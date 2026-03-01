mod input;
mod scene;

use bevy::prelude::*;
use app_state::{HeardTavernHints, SceneState};
use hud_ui::menu_style;

pub use input::town_input_system;
pub use scene::{
    build_town_commands, cleanup_town_scene, setup_town_scene, setup_town_scene_with_config,
    town_extra_display_system, ShopGoods, TownCommand, TownMenuPhase, TownResource, TownSceneConfig,
};

pub struct TownPlugin;

impl Plugin for TownPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HeardTavernHints>()
            .add_systems(OnEnter(SceneState::Town), (setup_town_scene, hud_ui::setup_hud))
            .add_systems(
                Update,
                (
                    town_input_system,
                    menu_style::scene_menu_display_system::<TownResource>,
                    town_extra_display_system,
                    hud_ui::update_hud,
                )
                    .chain()
                    .run_if(in_state(SceneState::Town)),
            )
            .add_systems(OnExit(SceneState::Town), (cleanup_town_scene, hud_ui::cleanup_hud));
    }
}
