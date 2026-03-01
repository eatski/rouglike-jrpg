use bevy::prelude::*;

use party::{shop_items, shop_weapons, ItemKind, PartyMemberKind, WeaponKind, BAG_CAPACITY, INVENTORY_CAPACITY};
use app_state::{PartyState, RecruitmentMap, TavernBounties};
use field_core::{Player, TilePosition};
use hud_ui::menu_style::{self, SceneMenu, PANEL_BG, PANEL_BORDER, SELECTED_COLOR, UNSELECTED_COLOR, FONT_PATH};

/// 町メニューのコマンド
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TownCommand {
    Inn,
    Shop,
    Tavern,
    SellBounty(ItemKind),
    HireCompanion(PartyMemberKind),
    Leave,
}

impl TownCommand {
    pub fn label(&self) -> String {
        match self {
            TownCommand::Inn => "やどや".to_string(),
            TownCommand::Shop => "よろず屋".to_string(),
            TownCommand::Tavern => "居酒屋".to_string(),
            TownCommand::SellBounty(item) => format!("{}をうる", item.name()),
            TownCommand::HireCompanion(kind) => format!("{}をやとう", kind.name()),
            TownCommand::Leave => "街を出る".to_string(),
        }
    }
}

/// デフォルトの町メニューコマンドを構築する
pub fn build_town_commands(bounty: Option<ItemKind>, hire_candidates: &[PartyMemberKind]) -> Vec<TownCommand> {
    let mut cmds = vec![
        TownCommand::Inn,
        TownCommand::Shop,
        TownCommand::Tavern,
    ];
    if let Some(item) = bounty {
        cmds.push(TownCommand::SellBounty(item));
    }
    for &kind in hire_candidates {
        cmds.push(TownCommand::HireCompanion(kind));
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

impl SceneMenu for TownResource {
    fn menu_labels(&self) -> Vec<String> {
        self.commands.iter().map(|c| c.label()).collect()
    }

    fn selected(&self) -> usize {
        self.selected_item
    }

    fn set_selected(&mut self, index: usize) {
        self.selected_item = index;
    }

    fn is_in_main_menu(&self) -> bool {
        matches!(self.phase, TownMenuPhase::MenuSelect)
    }

    fn show_main_menu(&self) -> bool {
        !matches!(
            self.phase,
            TownMenuPhase::ShopSelect { .. }
                | TownMenuPhase::ShopModeSelect { .. }
                | TownMenuPhase::SellItemSelect { .. }
                | TownMenuPhase::ShopCharacterSelect { .. }
                | TownMenuPhase::SellCharacterSelect { .. }
                | TownMenuPhase::BountyCharacterSelect { .. }
                | TownMenuPhase::ShopMessage { .. }
                | TownMenuPhase::BountyMessage { .. }
        )
    }

    fn current_message(&self) -> Option<&str> {
        match &self.phase {
            TownMenuPhase::ShowMessage { message }
            | TownMenuPhase::ShopMessage { message }
            | TownMenuPhase::RecruitMessage { message }
            | TownMenuPhase::BountyMessage { message } => Some(message),
            _ => None,
        }
    }
}

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

/// ショップパネル内のメニュー項目最大数（購入・売却で共用）
const SHOP_PANEL_MAX_ITEMS: usize = 7;

/// メインメニューの最大項目数（基本4 + 買い取り依頼1 + 雇用1）
const TOWN_MENU_MAX_ITEMS: usize = 6;

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
    recruitment_map: Res<RecruitmentMap>,
    player_query: Query<&TilePosition, With<Player>>,
) {
    let town_pos = player_query.single().ok().map(|pos| (pos.x, pos.y));
    let bounty_item = town_pos.and_then(|tp| tavern_bounties.active.get(&tp).copied());
    let hire_candidates = collect_hire_candidates(town_pos, &recruitment_map, &party_state);
    setup_town_scene_inner(&mut commands, &asset_server, &party_state, TownMenuPhase::MenuSelect, 0, bounty_item, &hire_candidates);
}

/// TownSceneConfigリソースから設定を読んでシーンを構築するシステム
pub fn setup_town_scene_with_config(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    party_state: Res<PartyState>,
    config: Res<TownSceneConfig>,
    tavern_bounties: Res<TavernBounties>,
    recruitment_map: Res<RecruitmentMap>,
    player_query: Query<&TilePosition, With<Player>>,
) {
    let phase = config.initial_phase.clone();
    let selected = config.selected_item;
    commands.remove_resource::<TownSceneConfig>();
    let town_pos = player_query.single().ok().map(|pos| (pos.x, pos.y));
    let bounty_item = town_pos.and_then(|tp| tavern_bounties.active.get(&tp).copied());
    let hire_candidates = collect_hire_candidates(town_pos, &recruitment_map, &party_state);
    setup_town_scene_inner(&mut commands, &asset_server, &party_state, phase, selected, bounty_item, &hire_candidates);
}

/// 雇用可能なキャラを収集する
fn collect_hire_candidates(
    town_pos: Option<(usize, usize)>,
    recruitment_map: &RecruitmentMap,
    party_state: &PartyState,
) -> Vec<PartyMemberKind> {
    let Some(tp) = town_pos else { return Vec::new() };
    recruitment_map
        .hire_available
        .get(&tp)
        .map(|&idx| party_state.candidates[idx].kind)
        .into_iter()
        .collect()
}

fn setup_town_scene_inner(
    commands: &mut Commands,
    asset_server: &AssetServer,
    party_state: &PartyState,
    initial_phase: TownMenuPhase,
    selected_item: usize,
    bounty_item: Option<ItemKind>,
    hire_candidates: &[PartyMemberKind],
) {
    let town_commands = build_town_commands(bounty_item, hire_candidates);
    let initial_labels: Vec<String> = town_commands.iter().map(|c| c.label()).collect();
    let initial_label_refs: Vec<&str> = initial_labels.iter().map(|s| s.as_str()).collect();

    commands.insert_resource(TownResource {
        selected_item,
        phase: initial_phase,
        commands: town_commands,
    });

    let root = menu_style::spawn_menu_scene(
        commands,
        asset_server,
        "まちに ついた",
        &initial_label_refs,
        TOWN_MENU_MAX_ITEMS,
        TownSceneRoot,
    );

    // 町固有パネル（ショップ、キャラクター選択）を追加
    let font: Handle<Font> = asset_server.load(FONT_PATH);
    commands.entity(root).with_children(|parent| {
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
                BackgroundColor(PANEL_BG),
                BorderColor::all(PANEL_BORDER),
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
                BackgroundColor(PANEL_BG),
                BorderColor::all(PANEL_BORDER),
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
                // ふくろエントリ
                let bag_index = party_state.members.len();
                let bag_label = format!(
                    "  ふくろ  {}/{}",
                    party_state.bag.total_count(),
                    BAG_CAPACITY,
                );
                char_panel.spawn((
                    ShopCharacterMenuItem { index: bag_index },
                    Text::new(bag_label),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(UNSELECTED_COLOR),
                ));
            });
    });

    // メッセージエリアを最後に追加
    menu_style::spawn_message_area(commands, root, asset_server);
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

/// 町固有の表示を更新するシステム（ショップ・キャラ選択パネル等）
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn town_extra_display_system(
    town_res: Res<TownResource>,
    party_state: Res<PartyState>,
    mut shop_root_query: Query<&mut Node, (With<ShopMenuRoot>, Without<ShopCharacterPanel>, Without<ShopMenuItem>)>,
    mut shop_item_query: Query<
        (&ShopMenuItem, &mut Text, &mut TextColor, &mut Node),
        (Without<ShopCharacterMenuItem>, Without<ShopGoldText>, Without<ShopMenuRoot>, Without<ShopCharacterPanel>),
    >,
    mut gold_query: Query<&mut Text, (With<ShopGoldText>, Without<ShopMenuItem>, Without<ShopCharacterMenuItem>)>,
    mut char_panel_query: Query<&mut Node, (With<ShopCharacterPanel>, Without<ShopMenuRoot>, Without<ShopMenuItem>)>,
    mut char_item_query: Query<
        (&ShopCharacterMenuItem, &mut Text, &mut TextColor),
        Without<ShopMenuItem>,
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

    // ショップパネル表示/非表示
    for mut node in &mut shop_root_query {
        menu_style::set_panel_visible(&mut node, in_shop_panel);
    }

    // キャラクター選択パネル表示/非表示
    for mut node in &mut char_panel_query {
        menu_style::set_panel_visible(&mut node, in_char_select);
    }

    // キャラクター選択メニュー項目の更新（購入）
    if let TownMenuPhase::ShopCharacterSelect { selected, .. } = &town_res.phase {
        for (char_item, mut text, mut color) in &mut char_item_query {
            let is_selected = char_item.index == *selected;
            let prefix = if is_selected { "> " } else { "  " };
            if char_item.index < party_state.members.len() {
                let member = &party_state.members[char_item.index];
                let detail = format!("{}/{}", member.inventory.total_count(), INVENTORY_CAPACITY);
                **text = format!("{}{}  {}", prefix, member.kind.name(), detail);
                *color = menu_style::menu_item_color(is_selected);
            } else if char_item.index == party_state.members.len() {
                **text = format!("{}ふくろ  {}/{}", prefix, party_state.bag.total_count(), BAG_CAPACITY);
                *color = menu_style::menu_item_color(is_selected);
            }
        }
    }

    // キャラクター選択メニュー項目の更新（買い取り依頼）
    if let TownMenuPhase::BountyCharacterSelect { item, selected } = &town_res.phase {
        for (char_item, mut text, mut color) in &mut char_item_query {
            let is_selected = char_item.index == *selected;
            let prefix = if is_selected { "> " } else { "  " };
            if char_item.index < party_state.members.len() {
                let member = &party_state.members[char_item.index];
                let count = member.inventory.count(*item);
                **text = format!("{}{} ({}個)", prefix, member.kind.name(), count);
                *color = menu_style::menu_item_color(is_selected);
            } else if char_item.index == party_state.members.len() {
                let count = party_state.bag.count(*item);
                **text = format!("{}ふくろ ({}個)", prefix, count);
                *color = menu_style::menu_item_color(is_selected);
            }
        }
    }

    // キャラクター選択メニュー項目の更新（売却）
    if let TownMenuPhase::SellCharacterSelect { selected } = &town_res.phase {
        for (char_item, mut text, mut color) in &mut char_item_query {
            let is_selected = char_item.index == *selected;
            let prefix = if is_selected { "> " } else { "  " };
            if char_item.index < party_state.members.len() {
                let member = &party_state.members[char_item.index];
                let equipped_weapon = member.equipment.weapon;
                let sellable_count: u32 = member
                    .inventory
                    .owned_items()
                    .iter()
                    .filter(|i| i.sell_price() > 0)
                    .map(|i| {
                        let cnt = member.inventory.count(*i);
                        if let Some(w) = i.as_weapon()
                            && equipped_weapon == Some(w)
                        {
                            return cnt.saturating_sub(1);
                        }
                        cnt
                    })
                    .sum();
                **text = format!("{}{}  売却可: {}個", prefix, member.kind.name(), sellable_count);
                *color = menu_style::menu_item_color(is_selected);
            } else if char_item.index == party_state.members.len() {
                let sellable_count: u32 = party_state.bag
                    .owned_items()
                    .iter()
                    .filter(|i| i.sell_price() > 0)
                    .map(|i| party_state.bag.count(*i))
                    .sum();
                **text = format!("{}ふくろ  売却可: {}個", prefix, sellable_count);
                *color = menu_style::menu_item_color(is_selected);
            }
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
                    *color = menu_style::menu_item_color(is_selected);
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
                    *color = menu_style::menu_item_color(is_selected);
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
            let is_bag = *member_index == party_state.members.len();
            let (inventory, equipped_weapon) = if is_bag {
                (&party_state.bag, None)
            } else {
                let member = &party_state.members[*member_index];
                (&member.inventory, member.equipment.weapon)
            };
            let sellable_items: Vec<_> = inventory
                .owned_items()
                .into_iter()
                .filter(|i| {
                    if i.sell_price() == 0 { return false; }
                    if let Some(w) = i.as_weapon()
                        && equipped_weapon == Some(w)
                    {
                        return inventory.count(*i) > 1;
                    }
                    true
                })
                .collect();
            for (shop_item, mut text, mut color, mut node) in &mut shop_item_query {
                if shop_item.index < sellable_items.len() {
                    let item = sellable_items[shop_item.index];
                    let count = inventory.count(item);
                    let is_selected = shop_item.index == *selected;
                    let prefix = if is_selected { "> " } else { "  " };
                    if let Some(w) = item.as_weapon() {
                        let is_equipped = equipped_weapon == Some(w);
                        let equip_mark = if is_equipped { "E " } else { "" };
                        **text = format!(
                            "{}{}{} x{}  {}G",
                            prefix,
                            equip_mark,
                            item.name(),
                            count,
                            item.sell_price()
                        );
                    } else {
                        **text = format!(
                            "{}{} x{}  {}G",
                            prefix,
                            item.name(),
                            count,
                            item.sell_price()
                        );
                    }
                    *color = menu_style::menu_item_color(is_selected);
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
}
