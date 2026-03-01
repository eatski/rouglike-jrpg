use bevy::prelude::*;

// ─── CommandMenu trait ───

/// コマンドメニューの共通インターフェース（基盤Layer 1）
pub trait CommandMenu: Resource {
    /// メニューの選択肢ラベル一覧
    fn menu_labels(&self) -> Vec<String>;
    /// 現在の選択インデックス
    fn selected(&self) -> usize;
    fn set_selected(&mut self, index: usize);
    /// 入力受付中か
    fn is_active(&self) -> bool;
    /// 表示可能なアイテム数（None=スクロールなし＝全件表示）
    fn visible_items(&self) -> Option<usize> {
        None
    }
    /// 指定インデックスが無効（グレーアウト）か
    fn is_disabled(&self, _index: usize) -> bool {
        false
    }
}

// ─── 色定数・ヘルパー ───

pub const DISABLED_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);

pub fn menu_item_color(is_disabled: bool) -> TextColor {
    if is_disabled {
        TextColor(DISABLED_COLOR)
    } else {
        TextColor(Color::WHITE)
    }
}

pub fn menu_item_text(label: &str, is_selected: bool) -> String {
    let prefix = if is_selected { "> " } else { "  " };
    format!("{}{}", prefix, label)
}

pub fn scroll_offset(cursor: usize, total: usize, visible: usize) -> usize {
    if total <= visible {
        return 0;
    }
    let half = visible / 2;
    cursor.saturating_sub(half).min(total - visible)
}

pub fn set_item_visible(vis: &mut Visibility, node: &mut Node, show: bool) {
    *vis = if show {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    node.display = if show {
        Display::DEFAULT
    } else {
        Display::None
    };
}

// ─── 共通コンポーネント ───

#[derive(Component)]
pub struct CommandMenuItem {
    pub index: usize,
}

#[derive(Component)]
pub struct CommandMenuScrollUp;

#[derive(Component)]
pub struct CommandMenuScrollDown;

// ─── 表示システム ───

#[allow(clippy::type_complexity)]
pub fn command_menu_display_system<T: CommandMenu>(
    menu: Option<Res<T>>,
    mut item_query: Query<
        (
            &CommandMenuItem,
            &mut Text,
            &mut TextColor,
            &mut Visibility,
            &mut Node,
        ),
        (Without<CommandMenuScrollUp>, Without<CommandMenuScrollDown>),
    >,
    mut scroll_up_query: Query<
        (&mut Visibility, &mut Node),
        (
            With<CommandMenuScrollUp>,
            Without<CommandMenuItem>,
            Without<CommandMenuScrollDown>,
        ),
    >,
    mut scroll_down_query: Query<
        (&mut Visibility, &mut Node),
        (
            With<CommandMenuScrollDown>,
            Without<CommandMenuItem>,
            Without<CommandMenuScrollUp>,
        ),
    >,
) {
    let Some(menu) = menu else { return };
    let labels = menu.menu_labels();
    let selected = menu.selected();
    let total = labels.len();
    let visible = menu.visible_items().unwrap_or(total);
    let offset = scroll_offset(selected, total, visible);

    for (item, mut text, mut color, mut vis, mut node) in &mut item_query {
        let data_index = offset + item.index;
        if item.index < visible && data_index < total {
            let is_selected = data_index == selected;
            **text = menu_item_text(&labels[data_index], is_selected);
            *color = menu_item_color(menu.is_disabled(data_index));
            set_item_visible(&mut vis, &mut node, true);
        } else {
            set_item_visible(&mut vis, &mut node, false);
        }
    }

    // スクロールインジケータ
    let has_scroll = total > visible;
    for (mut vis, mut node) in &mut scroll_up_query {
        set_item_visible(&mut vis, &mut node, has_scroll && offset > 0);
    }
    for (mut vis, mut node) in &mut scroll_down_query {
        set_item_visible(
            &mut vis,
            &mut node,
            has_scroll && offset + visible < total,
        );
    }
}

// ─── 入力ヘルパー ───

/// メニューのカーソル移動（上下キー、ラップあり）
pub fn handle_menu_navigation<T: CommandMenu>(keyboard: &ButtonInput<KeyCode>, menu: &mut T) {
    use input_ui::{is_down_just_pressed, is_up_just_pressed};

    if !menu.is_active() {
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
        menu.set_selected(if current < count - 1 {
            current + 1
        } else {
            0
        });
    }
}
