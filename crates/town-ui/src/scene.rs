use bevy::prelude::*;

use party::{shop_items, shop_weapons, ItemKind, WeaponKind, INVENTORY_CAPACITY};
use app_state::{PartyState, TavernBounties};
use field_core::{Player, TilePosition};
/// 町メニューのコマンド
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TownCommand {
    Inn,
    Shop,
    Tavern,
    SellBounty(ItemKind),
    Leave,
}

impl TownCommand {
    pub fn label(&self) -> String {
        match self {
            TownCommand::Inn => "やどや".to_string(),
            TownCommand::Shop => "よろず屋".to_string(),
            TownCommand::Tavern => "居酒屋".to_string(),
            TownCommand::SellBounty(item) => format!("{}をうる", item.name()),
            TownCommand::Leave => "街を出る".to_string(),
        }
    }
}

/// デフォルトの町メニューコマンドを構築する
pub fn build_town_commands(bounty: Option<ItemKind>) -> Vec<TownCommand> {
    let mut cmds = vec![
        TownCommand::Inn,
        TownCommand::Shop,
        TownCommand::Tavern,
    ];
    if let Some(item) = bounty {
        cmds.push(TownCommand::SellBounty(item));
    }
    cmds.push(TownCommand::Leave);
    cmds
}

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
    /// よろず屋 — かう/うる選択
    ShopModeSelect { selected: usize },
    /// よろず屋 — 売却キャラ選択
    SellCharacterSelect { selected: usize },
    /// よろず屋 — 売却アイテム選択
    SellItemSelect { member_index: usize, selected: usize },
    /// 買い取り依頼 — キャラクター選択
    BountyCharacterSelect { item: ItemKind, selected: usize },
    /// 買い取り依頼 — 結果メッセージ
    BountyMessage { message: String },
}

/// 町の状態管理リソース
#[derive(Resource)]
pub struct TownResource {
    /// 現在選択中のメニュー項目
    pub selected_item: usize,
    /// 現在のフェーズ
    pub phase: TownMenuPhase,
    /// 動的メニューコマンド一覧
    pub commands: Vec<TownCommand>,
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

/// ショップパネル内のメニュー項目最大数（購入・売却で共用）
const SHOP_PANEL_MAX_ITEMS: usize = 7;

/// メインメニューの最大項目数（基本4 + 買い取り依頼1）
const TOWN_MENU_MAX_ITEMS: usize = 5;

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

/// 町シーンの初期状態設定（リソースとして注入）
#[derive(Resource)]
pub struct TownSceneConfig {
    /// 初期フェーズ
    pub initial_phase: TownMenuPhase,
    /// 初期選択位置
    pub selected_item: usize,
}

pub fn setup_town_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    party_state: Res<PartyState>,
    tavern_bounties: Res<TavernBounties>,
    player_query: Query<&TilePosition, With<Player>>,
) {
    let bounty_item = player_query.single().ok().and_then(|pos| {
        tavern_bounties.active.get(&(pos.x, pos.y)).copied()
    });
    setup_town_scene_inner(&mut commands, &asset_server, &party_state, TownMenuPhase::MenuSelect, 0, bounty_item);
}

/// TownSceneConfigリソースから設定を読んでシーンを構築するシステム
pub fn setup_town_scene_with_config(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    party_state: Res<PartyState>,
    config: Res<TownSceneConfig>,
    tavern_bounties: Res<TavernBounties>,
    player_query: Query<&TilePosition, With<Player>>,
) {
    let phase = config.initial_phase.clone();
    let selected = config.selected_item;
    commands.remove_resource::<TownSceneConfig>();
    let bounty_item = player_query.single().ok().and_then(|pos| {
        tavern_bounties.active.get(&(pos.x, pos.y)).copied()
    });
    setup_town_scene_inner(&mut commands, &asset_server, &party_state, phase, selected, bounty_item);
}

fn setup_town_scene_inner(
    commands: &mut Commands,
    asset_server: &AssetServer,
    party_state: &PartyState,
    initial_phase: TownMenuPhase,
    selected_item: usize,
    bounty_item: Option<ItemKind>,
) {
    let town_commands = build_town_commands(bounty_item);
    commands.insert_resource(TownResource {
        selected_item,
        phase: initial_phase,
        commands: town_commands,
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
                    let base_labels = ["やどや", "よろず屋", "居酒屋", "街を出る"];
                    for i in 0..TOWN_MENU_MAX_ITEMS {
                        let (label, display) = if i < base_labels.len() {
                            let prefix = if i == 0 { "> " } else { "  " };
                            (format!("{}{}", prefix, base_labels[i]), Display::Flex)
                        } else {
                            (String::new(), Display::None)
                        };
                        let color = if i == 0 {
                            SELECTED_COLOR
                        } else {
                            UNSELECTED_COLOR
                        };
                        menu.spawn((
                            TownMenuItem { index: i },
                            Text::new(label),
                            TextFont {
                                font: font.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(color),
                            Node {
                                display,
                                ..default()
                            },
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

                    // メニュー項目（購入・売却で共用）
                    for i in 0..SHOP_PANEL_MAX_ITEMS {
                        let display = if i < shop_goods().len() {
                            Display::Flex
                        } else {
                            Display::None
                        };
                        let (label, color) = if i < shop_goods().len() {
                            let prefix = if i == 0 { "> " } else { "  " };
                            let label = format_goods_label(prefix, &shop_goods()[i]);
                            let color = if i == 0 { SELECTED_COLOR } else { UNSELECTED_COLOR };
                            (label, color)
                        } else {
                            (String::new(), UNSELECTED_COLOR)
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
                            Node {
                                display,
                                ..default()
                            },
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
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TownResource>();
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
        (&TownMenuItem, &mut Text, &mut TextColor, &mut Node),
        (Without<TownMessageText>, Without<TownMessageArea>, Without<ShopMenuItem>, Without<ShopCharacterMenuItem>, Without<TownMainMenu>, Without<ShopMenuRoot>, Without<ShopCharacterPanel>),
    >,
    mut shop_item_query: Query<
        (&ShopMenuItem, &mut Text, &mut TextColor, &mut Node),
        (Without<TownMenuItem>, Without<TownMessageText>, Without<TownMessageArea>, Without<ShopCharacterMenuItem>, Without<ShopGoldText>, Without<TownMainMenu>, Without<ShopMenuRoot>, Without<ShopCharacterPanel>),
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
    let in_shop_panel = matches!(
        &town_res.phase,
        TownMenuPhase::ShopSelect { .. }
            | TownMenuPhase::ShopModeSelect { .. }
            | TownMenuPhase::SellItemSelect { .. }
    );
    let in_char_select = matches!(
        &town_res.phase,
        TownMenuPhase::ShopCharacterSelect { .. }
            | TownMenuPhase::SellCharacterSelect { .. }
            | TownMenuPhase::BountyCharacterSelect { .. }
    );
    let in_shop_message = matches!(
        &town_res.phase,
        TownMenuPhase::ShopMessage { .. } | TownMenuPhase::BountyMessage { .. }
    );

    // メインメニュー表示/非表示
    for mut node in &mut main_menu_query {
        set_panel_visible(&mut node, !in_shop_panel && !in_char_select && !in_shop_message);
    }

    // ショップパネル表示/非表示
    for mut node in &mut shop_root_query {
        set_panel_visible(&mut node, in_shop_panel);
    }

    // キャラクター選択パネル表示/非表示
    for mut node in &mut char_panel_query {
        set_panel_visible(&mut node, in_char_select);
    }

    // キャラクター選択メニュー項目の更新（購入）
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

    // キャラクター選択メニュー項目の更新（買い取り依頼）
    if let TownMenuPhase::BountyCharacterSelect { item, selected } = &town_res.phase {
        for (char_item, mut text, mut color) in &mut char_item_query {
            if char_item.index < party_state.members.len() {
                let is_selected = char_item.index == *selected;
                let prefix = if is_selected { "> " } else { "  " };
                let member = &party_state.members[char_item.index];
                let count = member.inventory.count(*item);
                **text = format!("{}{} ({}個)", prefix, member.kind.name(), count);
                *color = if is_selected {
                    TextColor(SELECTED_COLOR)
                } else {
                    TextColor(UNSELECTED_COLOR)
                };
            }
        }
    }

    // キャラクター選択メニュー項目の更新（売却）
    if let TownMenuPhase::SellCharacterSelect { selected } = &town_res.phase {
        for (char_item, mut text, mut color) in &mut char_item_query {
            if char_item.index < party_state.members.len() {
                let is_selected = char_item.index == *selected;
                let prefix = if is_selected { "> " } else { "  " };
                let member = &party_state.members[char_item.index];
                let sellable_count: u32 = member
                    .inventory
                    .owned_items()
                    .iter()
                    .filter(|i| i.sell_price() > 0)
                    .map(|i| member.inventory.count(*i))
                    .sum();
                **text = format!("{}{}  売却可: {}個", prefix, member.kind.name(), sellable_count);
                *color = if is_selected {
                    TextColor(SELECTED_COLOR)
                } else {
                    TextColor(UNSELECTED_COLOR)
                };
            }
        }
    }

    // メインメニュー項目の更新（commands ベースで動的）
    for (item, mut text, mut color, mut node) in &mut menu_query {
        if item.index < town_res.commands.len() {
            let is_selected = item.index == town_res.selected_item;
            let prefix = if is_selected { "> " } else { "  " };
            **text = format!("{}{}", prefix, town_res.commands[item.index].label());
            *color = if is_selected {
                TextColor(SELECTED_COLOR)
            } else {
                TextColor(UNSELECTED_COLOR)
            };
            node.display = Display::Flex;
        } else {
            **text = String::new();
            node.display = Display::None;
        }
    }

    // ショップパネル項目の更新
    match &town_res.phase {
        TownMenuPhase::ShopSelect { selected } => {
            let goods_list = shop_goods();
            for (shop_item, mut text, mut color, mut node) in &mut shop_item_query {
                if shop_item.index < goods_list.len() {
                    let is_selected = shop_item.index == *selected;
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format_goods_label(prefix, &goods_list[shop_item.index]);
                    *color = if is_selected {
                        TextColor(SELECTED_COLOR)
                    } else {
                        TextColor(UNSELECTED_COLOR)
                    };
                    node.display = Display::Flex;
                } else {
                    **text = String::new();
                    node.display = Display::None;
                }
            }
        }
        TownMenuPhase::ShopModeSelect { selected } => {
            let labels = ["かう", "うる"];
            for (shop_item, mut text, mut color, mut node) in &mut shop_item_query {
                if shop_item.index < labels.len() {
                    let is_selected = shop_item.index == *selected;
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format!("{}{}", prefix, labels[shop_item.index]);
                    *color = if is_selected {
                        TextColor(SELECTED_COLOR)
                    } else {
                        TextColor(UNSELECTED_COLOR)
                    };
                    node.display = Display::Flex;
                } else {
                    **text = String::new();
                    node.display = Display::None;
                }
            }
        }
        TownMenuPhase::SellItemSelect {
            member_index,
            selected,
        } => {
            let sellable_items: Vec<_> = if *member_index < party_state.members.len() {
                party_state.members[*member_index]
                    .inventory
                    .owned_items()
                    .into_iter()
                    .filter(|i| i.sell_price() > 0)
                    .collect()
            } else {
                Vec::new()
            };
            for (shop_item, mut text, mut color, mut node) in &mut shop_item_query {
                if shop_item.index < sellable_items.len() {
                    let item = sellable_items[shop_item.index];
                    let count = party_state.members[*member_index].inventory.count(item);
                    let is_selected = shop_item.index == *selected;
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format!(
                        "{}{} x{}  {}G",
                        prefix,
                        item.name(),
                        count,
                        item.sell_price()
                    );
                    *color = if is_selected {
                        TextColor(SELECTED_COLOR)
                    } else {
                        TextColor(UNSELECTED_COLOR)
                    };
                    node.display = Display::Flex;
                } else {
                    **text = String::new();
                    node.display = Display::None;
                }
            }
        }
        _ => {}
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
            | TownMenuPhase::BountyMessage { .. }
    );

    for mut node in &mut message_area_query {
        set_panel_visible(&mut node, show_message);
    }

    for mut text in &mut message_query {
        match &town_res.phase {
            TownMenuPhase::ShowMessage { message }
            | TownMenuPhase::ShopMessage { message }
            | TownMenuPhase::RecruitMessage { message }
            | TownMenuPhase::BountyMessage { message } => {
                **text = message.clone();
            }
            _ => {}
        }
    }
}
