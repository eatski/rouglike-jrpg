use bevy::prelude::*;

// ─── 色定数 ───

pub const SELECTED_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);
pub const UNSELECTED_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);
pub const PANEL_BG: Color = Color::srgba(0.1, 0.1, 0.15, 0.85);
pub const PANEL_BORDER: Color = Color::srgb(0.4, 0.4, 0.5);
pub const FONT_PATH: &str = "fonts/NotoSansJP-Bold.ttf";

// ─── ヘルパー関数 ───

pub fn menu_item_color(is_selected: bool) -> TextColor {
    if is_selected {
        TextColor(SELECTED_COLOR)
    } else {
        TextColor(UNSELECTED_COLOR)
    }
}

pub fn menu_item_text(label: &str, is_selected: bool) -> String {
    let prefix = if is_selected { "> " } else { "  " };
    format!("{}{}", prefix, label)
}

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

/// メニューシーンの共通インターフェース
pub trait SceneMenu: Resource {
    /// メインメニューの選択肢ラベル一覧
    fn menu_labels(&self) -> Vec<String>;
    /// 現在の選択インデックス
    fn selected(&self) -> usize;
    fn set_selected(&mut self, index: usize);
    /// メインメニューで入力を受け付ける状態か
    fn is_in_main_menu(&self) -> bool;
    /// メインメニューパネルを表示するか
    fn show_main_menu(&self) -> bool;
    /// 表示中のメッセージ（Noneならメッセージ非表示）
    fn current_message(&self) -> Option<&str>;
}

// ─── 共通コンポーネント ───

#[derive(Component)]
pub struct SceneMenuItem {
    pub index: usize,
}

#[derive(Component)]
pub struct SceneMessageArea;

#[derive(Component)]
pub struct SceneMessageText;

#[derive(Component)]
pub struct SceneMainMenu;

// ─── ジェネリック表示システム ───

#[allow(clippy::type_complexity)]
pub fn menu_display_system<T: SceneMenu>(
    menu: Res<T>,
    mut menu_query: Query<
        (&SceneMenuItem, &mut Text, &mut TextColor, &mut Node),
        (Without<SceneMessageText>, Without<SceneMessageArea>, Without<SceneMainMenu>),
    >,
    mut main_menu_query: Query<
        &mut Node,
        (With<SceneMainMenu>, Without<SceneMenuItem>, Without<SceneMessageArea>),
    >,
    mut message_area_query: Query<
        &mut Node,
        (With<SceneMessageArea>, Without<SceneMainMenu>, Without<SceneMenuItem>),
    >,
    mut message_text_query: Query<
        &mut Text,
        (With<SceneMessageText>, Without<SceneMenuItem>),
    >,
) {
    let labels = menu.menu_labels();
    let selected = menu.selected();

    // メニュー項目の更新
    for (item, mut text, mut color, mut node) in &mut menu_query {
        if item.index < labels.len() {
            let is_selected = item.index == selected;
            **text = menu_item_text(&labels[item.index], is_selected);
            *color = menu_item_color(is_selected);
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

/// メインメニューのカーソル移動（上下キー、ラップあり）
pub fn handle_menu_navigation<T: SceneMenu>(keyboard: &ButtonInput<KeyCode>, menu: &mut T) {
    use input_ui::{is_down_just_pressed, is_up_just_pressed};

    if !menu.is_in_main_menu() {
        return;
    }

    let count = menu.menu_labels().len();
    if count == 0 {
        return;
    }

    let current = menu.selected();
    if is_up_just_pressed(keyboard) {
        menu.set_selected(if current > 0 { current - 1 } else { count - 1 });
    }
    if is_down_just_pressed(keyboard) {
        menu.set_selected(if current < count - 1 { current + 1 } else { 0 });
    }
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
                                menu_item_text(initial_labels[i], i == 0),
                                Display::Flex,
                            )
                        } else {
                            (String::new(), Display::None)
                        };
                        let color = menu_item_color(i == 0);
                        menu.spawn((
                            SceneMenuItem { index: i },
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
pub fn spawn_message_area(
    commands: &mut Commands,
    parent: Entity,
    asset_server: &AssetServer,
) {
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
