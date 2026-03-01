use bevy::prelude::*;

use crate::command_menu::{self, CommandMenu, CommandMenuItem};

// ─── 色定数（パネル用のみ残す） ───

pub const PANEL_BG: Color = Color::srgba(0.1, 0.1, 0.15, 0.85);
pub const PANEL_BORDER: Color = Color::srgb(0.4, 0.4, 0.5);
pub const FONT_PATH: &str = "fonts/NotoSansJP-Bold.ttf";

// ─── ヘルパー関数 ───

pub fn menu_panel_node() -> (Node, BackgroundColor, BorderColor) {
    (
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(24.0)),
            border: UiRect::all(Val::Px(2.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(PANEL_BG),
        BorderColor::all(PANEL_BORDER),
    )
}

pub fn message_area_node() -> (Node, BackgroundColor, BorderColor) {
    (
        Node {
            margin: UiRect::top(Val::Px(24.0)),
            padding: UiRect::all(Val::Px(16.0)),
            border: UiRect::all(Val::Px(2.0)),
            min_width: Val::Px(300.0),
            justify_content: JustifyContent::Center,
            display: Display::None,
            ..default()
        },
        BackgroundColor(PANEL_BG),
        BorderColor::all(PANEL_BORDER),
    )
}

pub fn set_panel_visible(node: &mut Node, show: bool) {
    node.display = if show { Display::Flex } else { Display::None };
}

// ─── trait 定義 ───

/// シーンメニューの共通インターフェース（CommandMenu の上位Layer 2）
/// メッセージエリア・メインメニュー表示制御を追加
pub trait SceneMenu: CommandMenu {
    /// メインメニューパネルを表示するか
    fn show_main_menu(&self) -> bool;
    /// 表示中のメッセージ（Noneならメッセージ非表示）
    fn current_message(&self) -> Option<&str>;
}

// ─── SceneMenu固有コンポーネント ───

#[derive(Component)]
pub struct SceneMessageArea;

#[derive(Component)]
pub struct SceneMessageText;

#[derive(Component)]
pub struct SceneMainMenu;

// ─── ジェネリック表示システム ───

/// SceneMenu用の表示システム
/// CommandMenu のメニュー項目表示＋SceneMenu固有のメッセージエリア・メインメニュー制御
#[allow(clippy::type_complexity)]
pub fn scene_menu_display_system<T: SceneMenu>(
    menu: Res<T>,
    mut menu_query: Query<
        (&CommandMenuItem, &mut Text, &mut TextColor, &mut Node),
        (
            Without<SceneMessageText>,
            Without<SceneMessageArea>,
            Without<SceneMainMenu>,
        ),
    >,
    mut main_menu_query: Query<
        &mut Node,
        (
            With<SceneMainMenu>,
            Without<CommandMenuItem>,
            Without<SceneMessageArea>,
        ),
    >,
    mut message_area_query: Query<
        &mut Node,
        (
            With<SceneMessageArea>,
            Without<SceneMainMenu>,
            Without<CommandMenuItem>,
        ),
    >,
    mut message_text_query: Query<&mut Text, (With<SceneMessageText>, Without<CommandMenuItem>)>,
) {
    let labels = menu.menu_labels();
    let selected = menu.selected();

    // メニュー項目の更新
    for (item, mut text, mut color, mut node) in &mut menu_query {
        if item.index < labels.len() {
            let is_selected = item.index == selected;
            **text = command_menu::menu_item_text(&labels[item.index], is_selected);
            *color = command_menu::menu_item_color(menu.is_disabled(item.index));
            node.display = Display::Flex;
        } else {
            **text = String::new();
            node.display = Display::None;
        }
    }

    // メインメニューパネルの表示/非表示
    let show = menu.show_main_menu();
    for mut node in &mut main_menu_query {
        set_panel_visible(&mut node, show);
    }

    // メッセージエリアの表示/非表示
    let message = menu.current_message();
    for mut node in &mut message_area_query {
        set_panel_visible(&mut node, message.is_some());
    }

    // メッセージテキストの更新
    if let Some(msg) = message {
        for mut text in &mut message_text_query {
            **text = msg.to_string();
        }
    }
}

// ─── 入力ヘルパー ───

/// SceneMenu用カーソル移動（CommandMenu::handle_menu_navigation に委譲）
pub fn handle_menu_navigation<T: SceneMenu>(keyboard: &ButtonInput<KeyCode>, menu: &mut T) {
    command_menu::handle_menu_navigation(keyboard, menu);
}

// ─── スポーンヘルパー ───

/// 標準メニューシーンUI（タイトル＋メインメニュー）をスポーン。
/// メッセージエリアやシーン固有パネルは spawn 後に追加し、
/// 最後に `spawn_message_area` を呼ぶ。
pub fn spawn_menu_scene(
    commands: &mut Commands,
    asset_server: &AssetServer,
    title: &str,
    initial_labels: &[&str],
    max_items: usize,
    root_marker: impl Bundle,
) -> Entity {
    let font: Handle<Font> = asset_server.load(FONT_PATH);

    commands
        .spawn((
            root_marker,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(100),
        ))
        .with_children(|parent| {
            // タイトル
            parent.spawn((
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(32.0)),
                    ..default()
                },
            ));

            // メインメニューパネル
            let (panel_node, panel_bg, panel_border) = menu_panel_node();
            parent
                .spawn((SceneMainMenu, panel_node, panel_bg, panel_border))
                .with_children(|menu| {
                    for i in 0..max_items {
                        let (label, display) = if i < initial_labels.len() {
                            (
                                command_menu::menu_item_text(initial_labels[i], i == 0),
                                Display::Flex,
                            )
                        } else {
                            (String::new(), Display::None)
                        };
                        let color = command_menu::menu_item_color(false);
                        menu.spawn((
                            CommandMenuItem { index: i },
                            Text::new(label),
                            TextFont {
                                font: font.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            color,
                            Node {
                                display,
                                ..default()
                            },
                        ));
                    }
                });
        })
        .id()
}

/// メッセージエリアを親エンティティの子として追加する。
/// `spawn_menu_scene` でルートを作成した後、シーン固有パネルを追加した後に呼ぶ。
pub fn spawn_message_area(commands: &mut Commands, parent: Entity, asset_server: &AssetServer) {
    let font: Handle<Font> = asset_server.load(FONT_PATH);
    let (msg_node, msg_bg, msg_border) = message_area_node();

    commands.entity(parent).with_children(|area_parent| {
        area_parent
            .spawn((SceneMessageArea, msg_node, msg_bg, msg_border))
            .with_children(|area| {
                area.spawn((
                    SceneMessageText,
                    Text::new(""),
                    TextFont {
                        font,
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
    });
}
