use bevy::prelude::*;

use app_state::PartyState;
use party::ItemKind;
use screenshot_common::{setup_camera, ScreenshotAppBuilder};
use town_ui::{
    setup_town_scene_with_config, town_extra_display_system, ShopGoods, TownMenuPhase,
    TownResource, TownSceneConfig,
};

fn make_config(variant: &str) -> TownSceneConfig {
    let (phase, selected) = match variant {
        "menu" => (TownMenuPhase::MenuSelect, 0),
        "shop" => (TownMenuPhase::ShopSelect { selected: 0 }, 1),
        "shop_char" => (
            TownMenuPhase::ShopCharacterSelect {
                goods: ShopGoods::Item(ItemKind::Herb),
                selected: 0,
            },
            1,
        ),
        "inn" => (
            TownMenuPhase::ShowMessage {
                message: "ゆっくり おやすみなさい… HPとMPが かいふくした！".to_string(),
            },
            0,
        ),
        other => panic!("Unknown variant: {other}. Use: menu, shop, shop_char, inn"),
    };

    TownSceneConfig {
        initial_phase: phase,
        selected_item: selected,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let variant = args.get(1).map(|s| s.as_str()).unwrap_or("menu");

    let output_name = format!("town_{}", variant);
    let mut app = ScreenshotAppBuilder::new(&output_name).build();

    app.insert_resource(make_config(variant))
        .init_resource::<PartyState>()
        .add_systems(
            Startup,
            (setup_camera, setup_town_scene_with_config, hud_ui::setup_hud).chain(),
        )
        .add_systems(
            Update,
            (
                hud_ui::menu_style::scene_menu_display_system::<TownResource>,
                town_extra_display_system,
                hud_ui::update_hud,
            ),
        )
        .run();
}
