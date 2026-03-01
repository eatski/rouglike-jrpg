mod input;
mod scene;

use bevy::prelude::*;
use app_state::SceneState;
use hud_ui::menu_style;

pub use input::hokora_input_system;
pub use scene::{cleanup_hokora_scene, setup_hokora_scene, HokoraResource};

pub struct HokoraPlugin;

impl Plugin for HokoraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SceneState::Hokora), setup_hokora_scene)
            .add_systems(
                Update,
                (hokora_input_system, menu_style::scene_menu_display_system::<HokoraResource>)
                    .chain()
                    .run_if(in_state(SceneState::Hokora)),
            )
            .add_systems(OnExit(SceneState::Hokora), cleanup_hokora_scene);
    }
}
