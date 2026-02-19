mod input;
mod scene;

pub use input::town_input_system;
pub use scene::{
    cleanup_town_scene, setup_town_scene, setup_town_scene_with_config, town_display_system,
    ShopGoods, TownMenuPhase, TownResource, TownSceneConfig,
};
