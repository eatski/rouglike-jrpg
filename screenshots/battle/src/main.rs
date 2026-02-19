use bevy::prelude::*;

use app_state::PartyState;
use battle::Enemy;
use battle_ui::{
    battle_display_system, setup_battle_scene_with_config, BattlePhase, BattleSceneConfig,
};
use screenshot_common::{setup_camera, ScreenshotAppBuilder};

fn make_enemies() -> Vec<Enemy> {
    vec![Enemy::slime(), Enemy::goblin()]
}

fn make_config(variant: &str) -> BattleSceneConfig {
    let phase = match variant {
        "command" => BattlePhase::CommandSelect { member_index: 0 },
        "spell" => BattlePhase::SpellSelect { member_index: 0 },
        "target" => BattlePhase::TargetSelect { member_index: 0 },
        "message" => BattlePhase::ShowMessage {
            messages: vec!["スライムに 5の ダメージ！".to_string()],
            index: 0,
        },
        "victory" => BattlePhase::BattleOver {
            message: "まものたちを やっつけた！".to_string(),
        },
        other => panic!("Unknown variant: {other}. Use: command, spell, target, message, victory"),
    };

    BattleSceneConfig {
        enemies: make_enemies(),
        initial_phase: Some(phase),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let variant = args.get(1).map(|s| s.as_str()).unwrap_or("command");

    let output_name = format!("battle_{}", variant);
    let mut app = ScreenshotAppBuilder::new(&output_name).build();

    app.insert_resource(make_config(variant))
        .init_resource::<PartyState>()
        .add_systems(
            Startup,
            (setup_camera, setup_battle_scene_with_config).chain(),
        )
        .add_systems(Update, battle_display_system)
        .run();
}
