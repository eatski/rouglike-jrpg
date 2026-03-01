use bevy::prelude::*;

use app_state::HokoraPositions;
use field_core::{Player, TilePosition};
use hud_ui::command_menu::CommandMenu;
use hud_ui::menu_style::{self, SceneMenu};

/// 祠シーンのルートUIエンティティを識別するマーカー
#[derive(Component)]
pub struct HokoraSceneRoot;

/// 祠メニューのフェーズ
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HokoraMenuPhase {
    /// メニュー選択中
    MenuSelect,
    /// メッセージ表示中
    ShowMessage { message: String },
}

/// 祠の状態管理リソース
#[derive(Resource)]
pub struct HokoraResource {
    /// 現在選択中のメニュー項目 (0=様子を見る, 1=扉を開ける, 2=出る)
    pub selected_item: usize,
    /// 現在のフェーズ
    pub phase: HokoraMenuPhase,
    /// ワープ先の座標（もう一方の祠）
    pub warp_destination: Option<(usize, usize)>,
    /// 最寄り祠のインデックス（0始まり、必要月のかけら数の算出に使用）
    pub hokora_index: usize,
    /// ワープ済みフラグ（メッセージ後にフィールドへ遷移するため）
    pub warped: bool,
}

const HOKORA_LABELS: [&str; 3] = ["様子を見る", "扉を開ける", "出る"];

impl CommandMenu for HokoraResource {
    fn menu_labels(&self) -> Vec<String> {
        HOKORA_LABELS.iter().map(|s| (*s).to_string()).collect()
    }

    fn selected(&self) -> usize {
        self.selected_item
    }

    fn set_selected(&mut self, index: usize) {
        self.selected_item = index;
    }

    fn is_active(&self) -> bool {
        matches!(self.phase, HokoraMenuPhase::MenuSelect)
    }
}

impl SceneMenu for HokoraResource {
    fn show_main_menu(&self) -> bool {
        true
    }

    fn current_message(&self) -> Option<&str> {
        match &self.phase {
            HokoraMenuPhase::ShowMessage { message } => Some(message),
            _ => None,
        }
    }
}

pub fn setup_hokora_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    hokora_positions: Res<HokoraPositions>,
    player_query: Query<&TilePosition, With<Player>>,
) {
    // 最寄りの祠を特定し、対応するワープ先とインデックスを取得
    let (warp_destination, hokora_index) = if let Ok(pos) = player_query.single() {
        let idx = hokora_positions.nearest(pos.x, pos.y);
        let dest = hokora_positions.warp_destination(idx);
        (dest, idx)
    } else {
        (None, 0)
    };

    commands.insert_resource(HokoraResource {
        selected_item: 0,
        phase: HokoraMenuPhase::MenuSelect,
        warp_destination,
        hokora_index,
        warped: false,
    });

    let root = menu_style::spawn_menu_scene(
        &mut commands,
        &asset_server,
        "ほこらに ついた",
        &HOKORA_LABELS,
        HOKORA_LABELS.len(),
        HokoraSceneRoot,
    );
    menu_style::spawn_message_area(&mut commands, root, &asset_server);
}

pub fn cleanup_hokora_scene(
    mut commands: Commands,
    query: Query<Entity, With<HokoraSceneRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<HokoraResource>();
}
