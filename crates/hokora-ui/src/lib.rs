mod input;
mod scene;

use bevy::prelude::*;
use app_state::SceneState;

pub use input::hokora_input_system;
pub use scene::{cleanup_hokora_scene, hokora_display_system, setup_hokora_scene};

pub struct HokoraPlugin;

impl Plugin for HokoraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SceneState::Hokora), setup_hokora_scene)
            .add_systems(
                Update,
                (hokora_input_system, hokora_display_system)
                    .chain()
                    .run_if(in_state(SceneState::Hokora)),
            )
            .add_systems(OnExit(SceneState::Hokora), cleanup_hokora_scene);
    }
}
