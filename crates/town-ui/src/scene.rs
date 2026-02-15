use bevy::prelude::*;

use movement_ui::{Bounce, MovementLocked, PendingMove, Player, SmoothMove, TilePosition};
use party::{shop_items, shop_weapons, ItemKind, WeaponKind, INVENTORY_CAPACITY};
use shared_ui::{ActiveMap, MovementState, PartyState};

/// よろず屋の商品（アイテムと武器を統合）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopGoods {
    Item(ItemKind),
    Weapon(WeaponKind),
}

impl ShopGoods {
    pub fn name(self) -> &'static str {
        match self {
            ShopGoods::Item(item) => item.name(),
            ShopGoods::Weapon(weapon) => weapon.name(),
        }
    }

    pub fn price(self) -> u32 {
        match self {
            ShopGoods::Item(item) => item.price(),
            ShopGoods::Weapon(weapon) => weapon.price(),
        }
    }
}

/// よろず屋で購入可能な商品一覧
pub fn shop_goods() -> Vec<ShopGoods> {
    let mut goods: Vec<ShopGoods> = shop_items().into_iter().map(ShopGoods::Item).collect();
    goods.extend(shop_weapons().into_iter().map(ShopGoods::Weapon));
    goods
}

/// 町シーンのルートUIエンティティを識別するマーカー
#[derive(Component)]
pub struct TownSceneRoot;

/// 町メニューのフェーズ
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TownMenuPhase {
    /// メニュー選択中
    MenuSelect,
    /// メッセージ表示中
    ShowMessage { message: String },
    /// よろず屋 — 商品選択中
    ShopSelect { selected: usize },
    /// よろず屋 — キャラクター選択中
    ShopCharacterSelect { goods: ShopGoods, selected: usize },
    /// よろず屋 — 購入結果メッセージ表示中
    ShopMessage { message: String },
    /// 仲間候補との会話メッセージ表示中
    RecruitMessage { message: String },
}

/// 町の状態管理リソース
#[derive(Resource)]
pub struct TownResource {
    /// 現在選択中のメニュー項目 (0=やどや, 1=よろず屋, 2=話を聞く, 3=街を出る)
    pub selected_item: usize,
    /// 現在のフェーズ
    pub phase: TownMenuPhase,
}

/// 町メニュー項目のマーカー
#[derive(Component)]
pub struct TownMenuItem {
    pub index: usize,
}

/// 町メッセージテキストのマーカー
#[derive(Component)]
pub struct TownMessageText;

/// メッセージエリアの親ノードのマーカー
#[derive(Component)]
pub struct TownMessageArea;

/// メインメニューパネルのマーカー
#[derive(Component)]
pub struct TownMainMenu;

/// ショップパネルのルートマーカー
#[derive(Component)]
pub struct ShopMenuRoot;

/// ショップメニュー項目のマーカー
#[derive(Component)]
pub struct ShopMenuItem {
    pub index: usize,
}

/// ショップゴールド表示のマーカー
#[derive(Component)]
pub struct ShopGoldText;

/// キャラクター選択パネルのルートマーカー
#[derive(Component)]
pub struct ShopCharacterPanel;

/// キャラクター選択メニュー項目のマーカー
#[derive(Component)]
pub struct ShopCharacterMenuItem {
    pub index: usize,
}

const SELECTED_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);
const UNSELECTED_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);

fn format_goods_label(prefix: &str, goods: &ShopGoods) -> String {
    match goods {
        ShopGoods::Item(item) => {
            format!("{}{}  {}G", prefix, item.name(), item.price())
        }
        ShopGoods::Weapon(weapon) => {
            format!(
                "{}{}  {}G  ATK+{}",
                prefix,
                weapon.name(),
                weapon.price(),
                weapon.attack_bonus(),
            )
        }
    }
}

pub fn setup_town_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    party_state: Res<PartyState>,
) {
    commands.insert_resource(TownResource {
        selected_item: 0,
        phase: TownMenuPhase::MenuSelect,
    });

    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Bold.ttf");
    let panel_bg = Color::srgba(0.1, 0.1, 0.15, 0.85);
    let border_color = Color::srgb(0.4, 0.4, 0.5);

    commands
        .spawn((
            TownSceneRoot,
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
                Text::new("まちに ついた"),
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
            parent
                .spawn((
                    TownMainMenu,
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(24.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(border_color),
                ))
                .with_children(|menu| {
                    let items = ["> やどや", "  よろず屋", "  話を聞く", "  街を出る"];
                    for (i, label) in items.iter().enumerate() {
                        let color = if i == 0 {
                            SELECTED_COLOR
                        } else {
                            UNSELECTED_COLOR
                        };
                        menu.spawn((
                            TownMenuItem { index: i },
                            Text::new(*label),
                            TextFont {
                                font: font.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(color),
                        ));
                    }
                });

            // よろず屋パネル（初期は非表示）
            parent
                .spawn((
                    ShopMenuRoot,
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(24.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        row_gap: Val::Px(8.0),
                        min_width: Val::Px(300.0),
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(border_color),
                ))
                .with_children(|shop| {
                    // ゴールド表示
                    shop.spawn((
                        ShopGoldText,
                        Text::new(format!("所持金: {}G", party_state.gold)),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.3)),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));

                    // 商品一覧（アイテム＋武器）
                    for (i, goods) in shop_goods().iter().enumerate() {
                        let prefix = if i == 0 { "> " } else { "  " };
                        let label = format_goods_label(prefix, goods);
                        let color = if i == 0 {
                            SELECTED_COLOR
                        } else {
                            UNSELECTED_COLOR
                        };
                        shop.spawn((
                            ShopMenuItem { index: i },
                            Text::new(label),
                            TextFont {
                                font: font.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(color),
                        ));
                    }
                });

            // キャラクター選択パネル（初期は非表示）
            parent
                .spawn((
                    ShopCharacterPanel,
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(24.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        row_gap: Val::Px(8.0),
                        min_width: Val::Px(300.0),
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(border_color),
                ))
                .with_children(|char_panel| {
                    for (i, member) in party_state.members.iter().enumerate() {
                        let label = format!(
                            "{}{}  {}/{}",
                            if i == 0 { "> " } else { "  " },
                            member.kind.name(),
                            member.inventory.total_count(),
                            INVENTORY_CAPACITY,
                        );
                        let color = if i == 0 {
                            SELECTED_COLOR
                        } else {
                            UNSELECTED_COLOR
                        };
                        char_panel.spawn((
                            ShopCharacterMenuItem { index: i },
                            Text::new(label),
                            TextFont {
                                font: font.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(color),
                        ));
                    }
                });

            // メッセージエリア（初期は非表示）
            parent
                .spawn((
                    TownMessageArea,
                    Node {
                        margin: UiRect::top(Val::Px(24.0)),
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        min_width: Val::Px(300.0),
                        justify_content: JustifyContent::Center,
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(border_color),
                ))
                .with_children(|area| {
                    area.spawn((
                        TownMessageText,
                        Text::new(""),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn cleanup_town_scene(
    mut commands: Commands,
    query: Query<Entity, With<TownSceneRoot>>,
    mut player_query: Query<(Entity, &TilePosition, &mut Transform), With<Player>>,
    mut move_state: ResMut<MovementState>,
    active_map: Res<ActiveMap>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TownResource>();

    // プレイヤーの移動関連コンポーネントをクリーンアップ
    if let Ok((entity, tile_pos, mut transform)) = player_query.single_mut() {
        commands
            .entity(entity)
            .remove::<MovementLocked>()
            .remove::<SmoothMove>()
            .remove::<PendingMove>()
            .remove::<Bounce>();

        let (world_x, world_y) = active_map.to_world(tile_pos.x, tile_pos.y);
        transform.translation.x = world_x;
        transform.translation.y = world_y;
    }

    *move_state = MovementState::default();
}

/// パネルの表示/非表示を切り替える（Display::Noneでレイアウトからも除外）
fn set_panel_visible(node: &mut Node, show: bool) {
    node.display = if show { Display::Flex } else { Display::None };
}

/// 町メニューの表示を更新するシステム
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn town_display_system(
    town_res: Res<TownResource>,
    party_state: Res<PartyState>,
    mut main_menu_query: Query<&mut Node, (With<TownMainMenu>, Without<ShopMenuRoot>, Without<TownMessageArea>, Without<ShopCharacterPanel>)>,
    mut shop_root_query: Query<&mut Node, (With<ShopMenuRoot>, Without<TownMainMenu>, Without<TownMessageArea>, Without<ShopCharacterPanel>)>,
    mut menu_query: Query<
        (&TownMenuItem, &mut Text, &mut TextColor),
        (Without<TownMessageText>, Without<TownMessageArea>, Without<ShopMenuItem>, Without<ShopCharacterMenuItem>),
    >,
    mut shop_item_query: Query<
        (&ShopMenuItem, &mut Text, &mut TextColor),
        (Without<TownMenuItem>, Without<TownMessageText>, Without<TownMessageArea>, Without<ShopCharacterMenuItem>),
    >,
    mut gold_query: Query<&mut Text, (With<ShopGoldText>, Without<TownMenuItem>, Without<ShopMenuItem>, Without<TownMessageText>, Without<TownMessageArea>, Without<ShopCharacterMenuItem>)>,
    mut message_query: Query<&mut Text, (With<TownMessageText>, Without<TownMessageArea>, Without<TownMenuItem>, Without<ShopMenuItem>, Without<ShopGoldText>, Without<ShopCharacterMenuItem>)>,
    mut message_area_query: Query<&mut Node, (With<TownMessageArea>, Without<TownMainMenu>, Without<ShopMenuRoot>, Without<ShopCharacterPanel>)>,
    mut char_panel_query: Query<&mut Node, (With<ShopCharacterPanel>, Without<TownMainMenu>, Without<ShopMenuRoot>, Without<TownMessageArea>)>,
    mut char_item_query: Query<
        (&ShopCharacterMenuItem, &mut Text, &mut TextColor),
        (Without<TownMenuItem>, Without<ShopMenuItem>, Without<TownMessageText>, Without<ShopGoldText>),
    >,
) {
    let in_shop_select = matches!(&town_res.phase, TownMenuPhase::ShopSelect { .. });
    let in_char_select = matches!(&town_res.phase, TownMenuPhase::ShopCharacterSelect { .. });
    let in_shop_message = matches!(&town_res.phase, TownMenuPhase::ShopMessage { .. });

    // メインメニュー表示/非表示
    for mut node in &mut main_menu_query {
        set_panel_visible(&mut node, !in_shop_select && !in_char_select && !in_shop_message);
    }

    // ショップパネル表示/非表示（メッセージ表示中は隠す）
    for mut node in &mut shop_root_query {
        set_panel_visible(&mut node, in_shop_select);
    }

    // キャラクター選択パネル表示/非表示
    for mut node in &mut char_panel_query {
        set_panel_visible(&mut node, in_char_select);
    }

    // キャラクター選択メニュー項目の更新
    if let TownMenuPhase::ShopCharacterSelect { goods, selected } = &town_res.phase {
        for (char_item, mut text, mut color) in &mut char_item_query {
            if char_item.index < party_state.members.len() {
                let is_selected = char_item.index == *selected;
                let prefix = if is_selected { "> " } else { "  " };
                let member = &party_state.members[char_item.index];
                let detail = match goods {
                    ShopGoods::Item(_) => {
                        format!("{}/{}", member.inventory.total_count(), INVENTORY_CAPACITY)
                    }
                    ShopGoods::Weapon(_) => {
                        let weapon_name = member.equipment.weapon.map_or("なし", |w| w.name());
                        format!("[{}]", weapon_name)
                    }
                };
                **text = format!("{}{}  {}", prefix, member.kind.name(), detail);
                *color = if is_selected {
                    TextColor(SELECTED_COLOR)
                } else {
                    TextColor(UNSELECTED_COLOR)
                };
            }
        }
    }

    // メインメニュー項目の更新
    let labels = ["やどや", "よろず屋", "話を聞く", "街を出る"];
    for (item, mut text, mut color) in &mut menu_query {
        if item.index < labels.len() {
            let is_selected = item.index == town_res.selected_item;
            let prefix = if is_selected { "> " } else { "  " };
            **text = format!("{}{}", prefix, labels[item.index]);
            *color = if is_selected {
                TextColor(SELECTED_COLOR)
            } else {
                TextColor(UNSELECTED_COLOR)
            };
        }
    }

    // ショップ商品の更新
    if let TownMenuPhase::ShopSelect { selected } = &town_res.phase {
        let goods_list = shop_goods();
        for (shop_item, mut text, mut color) in &mut shop_item_query {
            if shop_item.index < goods_list.len() {
                let is_selected = shop_item.index == *selected;
                let prefix = if is_selected { "> " } else { "  " };
                **text = format_goods_label(prefix, &goods_list[shop_item.index]);
                *color = if is_selected {
                    TextColor(SELECTED_COLOR)
                } else {
                    TextColor(UNSELECTED_COLOR)
                };
            }
        }
    }

    // ゴールド表示更新
    for mut text in &mut gold_query {
        **text = format!("所持金: {}G", party_state.gold);
    }

    // メッセージエリアの表示/非表示
    let show_message = matches!(
        &town_res.phase,
        TownMenuPhase::ShowMessage { .. }
            | TownMenuPhase::ShopMessage { .. }
            | TownMenuPhase::RecruitMessage { .. }
    );

    for mut node in &mut message_area_query {
        set_panel_visible(&mut node, show_message);
    }

    for mut text in &mut message_query {
        match &town_res.phase {
            TownMenuPhase::ShowMessage { message }
            | TownMenuPhase::ShopMessage { message }
            | TownMenuPhase::RecruitMessage { message } => {
                **text = message.clone();
            }
            _ => {}
        }
    }
}
