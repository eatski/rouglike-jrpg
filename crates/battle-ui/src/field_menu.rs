use bevy::prelude::*;

use party::available_spells;
use spell::{calculate_heal_amount, SpellKind, SpellTarget};
use app_state::{FieldMenuOpen, PartyState};
use party::{ItemEffect, ItemKind};

/// ターゲット選択の文脈（呪文 or アイテム）
#[derive(Debug, Clone)]
pub enum TargetContext {
    Spell {
        caster: usize,
        spells: Vec<SpellKind>,
        spell_cursor: usize,
    },
    Item {
        member: usize,
        items: Vec<ItemKind>,
        item_cursor: usize,
    },
}

/// フィールドメニューのフェーズ（各フェーズが自身のデータを保持）
#[derive(Debug, Clone)]
pub enum FieldMenuPhase {
    /// トップメニュー（じゅもん/どうぐ）
    TopMenu { cursor: usize },
    /// キャスター選択（呪文フロー）
    CasterSelect { candidates: Vec<usize>, cursor: usize },
    /// 呪文選択
    SpellSelect { caster: usize, spells: Vec<SpellKind>, cursor: usize },
    /// メンバー選択（アイテムフロー）
    MemberSelect { candidates: Vec<usize>, cursor: usize },
    /// アイテム選択
    ItemSelect { member: usize, items: Vec<ItemKind>, cursor: usize },
    /// ターゲット選択（共用）
    TargetSelect { candidates: Vec<usize>, cursor: usize, context: TargetContext },
    /// メッセージ表示
    ShowMessage { message: String },
}

/// フィールドメニューの状態
#[derive(Resource)]
pub struct FieldMenuState {
    pub phase: FieldMenuPhase,
}

/// フィールドメニューのUIルートマーカー
#[derive(Component)]
pub struct FieldMenuRoot;

/// フィールドメニューのタイトルテキスト
#[derive(Component)]
pub struct FieldMenuTitle;

/// フィールドメニューの選択肢アイテム
#[derive(Component)]
pub struct FieldMenuItem {
    index: usize,
}

const MAX_MENU_ITEMS: usize = 16;
const VISIBLE_ITEMS: usize = 6;
const SELECTED_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);
const UNSELECTED_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const DISABLED_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);

/// フィールドメニューのスクロール上インジケータ
#[derive(Component)]
pub struct FieldMenuScrollUp;

/// フィールドメニューのスクロール下インジケータ
#[derive(Component)]
pub struct FieldMenuScrollDown;

fn scroll_offset(cursor: usize, total: usize, visible: usize) -> usize {
    if total <= visible {
        return 0;
    }
    let half = visible / 2;
    cursor.saturating_sub(half).min(total - visible)
}

/// メニューUIをスポーンする
fn spawn_menu_ui(commands: &mut Commands, asset_server: &AssetServer) {
    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Bold.ttf");
    let panel_bg = Color::srgba(0.1, 0.1, 0.15, 0.92);
    let border_color = Color::srgb(0.4, 0.4, 0.5);

    commands
        .spawn((
            FieldMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
            GlobalZIndex(200),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(260.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        row_gap: Val::Px(6.0),
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(border_color),
                ))
                .with_children(|menu_box| {
                    // タイトル
                    menu_box.spawn((
                        FieldMenuTitle,
                        Text::new(""),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                    ));

                    // ▲ スクロールインジケータ
                    menu_box.spawn((
                        FieldMenuScrollUp,
                        Text::new("  ▲"),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(UNSELECTED_COLOR),
                        Visibility::Hidden,
                    ));

                    // メニューアイテム
                    for i in 0..MAX_MENU_ITEMS {
                        menu_box.spawn((
                            FieldMenuItem { index: i },
                            Text::new(""),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(UNSELECTED_COLOR),
                            Visibility::Hidden,
                        ));
                    }

                    // ▼ スクロールインジケータ
                    menu_box.spawn((
                        FieldMenuScrollDown,
                        Text::new("  ▼"),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(UNSELECTED_COLOR),
                        Visibility::Hidden,
                    ));
                });
        });
}

/// メニューUIをdespawnする
fn despawn_menu_ui(commands: &mut Commands, query: &Query<Entity, With<FieldMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 生存メンバーのインデックス一覧を返す
fn alive_member_indices(party_state: &PartyState) -> Vec<usize> {
    party_state
        .members
        .iter()
        .enumerate()
        .filter(|(_, m)| m.stats.is_alive())
        .map(|(i, _)| i)
        .collect()
}

fn close_menu(
    commands: &mut Commands,
    root_query: &Query<Entity, With<FieldMenuRoot>>,
) {
    despawn_menu_ui(commands, root_query);
    commands.remove_resource::<FieldMenuState>();
    commands.remove_resource::<FieldMenuOpen>();
}

/// フィールドメニューの入力処理システム
#[allow(clippy::too_many_arguments)]
pub fn field_menu_input_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    menu_open: Option<Res<FieldMenuOpen>>,
    mut party_state: ResMut<PartyState>,
    state: Option<ResMut<FieldMenuState>>,
    root_query: Query<Entity, With<FieldMenuRoot>>,
) {
    if menu_open.is_none() {
        // メニュー非表示: 確認キーで開く
        if input_ui::is_menu_just_pressed(&keyboard) {
            let alive = alive_member_indices(&party_state);
            if alive.is_empty() {
                return;
            }

            commands.insert_resource(FieldMenuState {
                phase: FieldMenuPhase::TopMenu { cursor: 0 },
            });

            spawn_menu_ui(&mut commands, &asset_server);
            commands.insert_resource(FieldMenuOpen);
        }
        return;
    }

    let Some(mut state) = state else { return };

    match state.phase.clone() {
        FieldMenuPhase::TopMenu { cursor } => {
            handle_top_menu(&keyboard, &mut state, &party_state, &mut commands, &root_query, cursor);
        }
        FieldMenuPhase::CasterSelect { candidates, cursor } => {
            handle_caster_select(&keyboard, &mut state, &party_state, candidates, cursor);
        }
        FieldMenuPhase::SpellSelect { caster, spells, cursor } => {
            handle_spell_select(&keyboard, &mut state, &mut party_state, caster, spells, cursor);
        }
        FieldMenuPhase::MemberSelect { candidates, cursor } => {
            handle_member_select(&keyboard, &mut state, &party_state, candidates, cursor);
        }
        FieldMenuPhase::ItemSelect { member, items, cursor } => {
            handle_item_select(&keyboard, &mut state, &mut party_state, member, items, cursor);
        }
        FieldMenuPhase::TargetSelect { candidates, cursor, context } => {
            handle_target_select(
                &keyboard, &mut state, &mut party_state,
                candidates, cursor, context,
            );
        }
        FieldMenuPhase::ShowMessage { .. } => {
            if input_ui::is_confirm_just_pressed(&keyboard) {
                state.phase = FieldMenuPhase::TopMenu { cursor: 0 };
            }
            if input_ui::is_cancel_just_pressed(&keyboard) {
                close_menu(&mut commands, &root_query);
            }
        }
    }
}

fn handle_top_menu(
    keyboard: &ButtonInput<KeyCode>,
    state: &mut FieldMenuState,
    party_state: &PartyState,
    commands: &mut Commands,
    root_query: &Query<Entity, With<FieldMenuRoot>>,
    mut cursor: usize,
) {
    if input_ui::is_up_just_pressed(keyboard) && cursor > 0 {
        cursor -= 1;
    }
    if input_ui::is_down_just_pressed(keyboard) && cursor < 1 {
        cursor += 1;
    }
    state.phase = FieldMenuPhase::TopMenu { cursor };

    if input_ui::is_cancel_just_pressed(keyboard) {
        close_menu(commands, root_query);
        return;
    }

    if input_ui::is_confirm_just_pressed(keyboard) {
        if cursor == 0 {
            // じゅもん → CasterSelect
            let candidates = alive_member_indices(party_state);
            state.phase = FieldMenuPhase::CasterSelect { candidates, cursor: 0 };
        } else {
            // どうぐ → MemberSelect
            let candidates = alive_member_indices(party_state);
            state.phase = FieldMenuPhase::MemberSelect { candidates, cursor: 0 };
        }
    }
}

fn handle_caster_select(
    keyboard: &ButtonInput<KeyCode>,
    state: &mut FieldMenuState,
    party_state: &PartyState,
    candidates: Vec<usize>,
    mut cursor: usize,
) {
    let count = candidates.len();
    if input_ui::is_up_just_pressed(keyboard) && cursor > 0 {
        cursor -= 1;
    }
    if input_ui::is_down_just_pressed(keyboard) && cursor < count - 1 {
        cursor += 1;
    }
    state.phase = FieldMenuPhase::CasterSelect { candidates: candidates.clone(), cursor };

    if input_ui::is_cancel_just_pressed(keyboard) {
        state.phase = FieldMenuPhase::TopMenu { cursor: 0 };
        return;
    }

    if input_ui::is_confirm_just_pressed(keyboard) {
        let member_idx = candidates[cursor];
        let spells = available_spells(party_state.members[member_idx].kind, party_state.members[member_idx].level);
        if spells.is_empty() {
            state.phase = FieldMenuPhase::ShowMessage {
                message: "じゅもんを おぼえていない".to_string(),
            };
        } else {
            state.phase = FieldMenuPhase::SpellSelect {
                caster: member_idx,
                spells,
                cursor: 0,
            };
        }
    }
}

fn handle_spell_select(
    keyboard: &ButtonInput<KeyCode>,
    state: &mut FieldMenuState,
    party_state: &mut PartyState,
    caster: usize,
    spells: Vec<SpellKind>,
    mut cursor: usize,
) {
    let count = spells.len();
    if input_ui::is_up_just_pressed(keyboard) && cursor > 0 {
        cursor -= 1;
    }
    if input_ui::is_down_just_pressed(keyboard) && cursor < count - 1 {
        cursor += 1;
    }
    state.phase = FieldMenuPhase::SpellSelect { caster, spells: spells.clone(), cursor };

    if input_ui::is_cancel_just_pressed(keyboard) {
        let candidates = alive_member_indices(party_state);
        state.phase = FieldMenuPhase::CasterSelect { candidates, cursor: 0 };
        return;
    }

    if input_ui::is_confirm_just_pressed(keyboard) {
        let spell = spells[cursor];
        let caster_member = &party_state.members[caster];

        if caster_member.stats.mp < spell.mp_cost() {
            return;
        }

        if !spell.is_usable_in_field() {
            state.phase = FieldMenuPhase::ShowMessage {
                message: "フィールドでは つかえない".to_string(),
            };
        } else if spell.target_type() == SpellTarget::AllAllies {
            // 全体回復: ターゲット選択スキップ、全味方に一括実行
            execute_aoe_heal(state, party_state, caster, spell);
        } else {
            // 単体回復: ターゲット選択へ
            let target_candidates = alive_member_indices(party_state);
            state.phase = FieldMenuPhase::TargetSelect {
                candidates: target_candidates,
                cursor: 0,
                context: TargetContext::Spell {
                    caster,
                    spells,
                    spell_cursor: cursor,
                },
            };
        }
    }
}

fn handle_member_select(
    keyboard: &ButtonInput<KeyCode>,
    state: &mut FieldMenuState,
    party_state: &PartyState,
    candidates: Vec<usize>,
    mut cursor: usize,
) {
    let count = candidates.len();
    if input_ui::is_up_just_pressed(keyboard) && cursor > 0 {
        cursor -= 1;
    }
    if input_ui::is_down_just_pressed(keyboard) && cursor < count - 1 {
        cursor += 1;
    }
    state.phase = FieldMenuPhase::MemberSelect { candidates: candidates.clone(), cursor };

    if input_ui::is_cancel_just_pressed(keyboard) {
        state.phase = FieldMenuPhase::TopMenu { cursor: 0 };
        return;
    }

    if input_ui::is_confirm_just_pressed(keyboard) {
        let member_idx = candidates[cursor];
        let items = party_state.members[member_idx].inventory.owned_items();
        if items.is_empty() {
            state.phase = FieldMenuPhase::ShowMessage {
                message: "もちものが ない".to_string(),
            };
        } else {
            state.phase = FieldMenuPhase::ItemSelect {
                member: member_idx,
                items,
                cursor: 0,
            };
        }
    }
}

fn handle_item_select(
    keyboard: &ButtonInput<KeyCode>,
    state: &mut FieldMenuState,
    party_state: &mut PartyState,
    member: usize,
    items: Vec<ItemKind>,
    mut cursor: usize,
) {
    let count = items.len();
    if input_ui::is_up_just_pressed(keyboard) && cursor > 0 {
        cursor -= 1;
    }
    if input_ui::is_down_just_pressed(keyboard) && cursor < count - 1 {
        cursor += 1;
    }
    state.phase = FieldMenuPhase::ItemSelect { member, items: items.clone(), cursor };

    if input_ui::is_cancel_just_pressed(keyboard) {
        let candidates = alive_member_indices(party_state);
        state.phase = FieldMenuPhase::MemberSelect { candidates, cursor: 0 };
        return;
    }

    if input_ui::is_confirm_just_pressed(keyboard) {
        let item = items[cursor];
        match item.effect() {
            ItemEffect::Heal { .. } => {
                let target_candidates = alive_member_indices(party_state);
                state.phase = FieldMenuPhase::TargetSelect {
                    candidates: target_candidates,
                    cursor: 0,
                    context: TargetContext::Item {
                        member,
                        items,
                        item_cursor: cursor,
                    },
                };
            }
            ItemEffect::Equip => {
                let weapon = item.as_weapon().expect("Equip effect must be Weapon");
                let member_ref = &mut party_state.members[member];
                let member_name = member_ref.kind.name();

                // インベントリから新武器を取り出す
                member_ref.inventory.remove_item(item);

                // 装備中の武器を外してインベントリに戻す
                let old_weapon = member_ref.equipment.unequip_weapon();
                if let Some(old_w) = old_weapon {
                    member_ref.inventory.add(ItemKind::Weapon(old_w), 1);
                }

                // 新武器を装備
                member_ref.equipment.equip_weapon(weapon);

                let mut msg = format!("{}は {}を そうびした！", member_name, weapon.name());
                if let Some(old_w) = old_weapon {
                    msg.push_str(&format!("\n{}を はずした", old_w.name()));
                }
                state.phase = FieldMenuPhase::ShowMessage { message: msg };
            }
            ItemEffect::KeyItem | ItemEffect::Material => {
                let member_name = party_state.members[member].kind.name();
                state.phase = FieldMenuPhase::ShowMessage {
                    message: format!(
                        "{}は {}を しらべた。\n{}",
                        member_name,
                        item.name(),
                        item.description()
                    ),
                };
            }
        }
    }
}

/// 全体回復呪文をフィールドで実行
fn execute_aoe_heal(
    state: &mut FieldMenuState,
    party_state: &mut PartyState,
    caster: usize,
    spell: SpellKind,
) {
    let consumed = party_state.members[caster].stats.use_mp(spell.mp_cost());
    if !consumed {
        return;
    }

    let caster_name = party_state.members[caster].kind.name().to_string();
    let alive = alive_member_indices(party_state);
    let mut lines = vec![format!("{}は {}を となえた！", caster_name, spell.name())];

    for &pi in &alive {
        let random_factor = 0.8 + rand::random::<f32>() * 0.4;
        let amount = calculate_heal_amount(spell.power(), random_factor);
        let target = &mut party_state.members[pi];
        target.stats.hp = (target.stats.hp + amount).min(target.stats.max_hp);
        let target_name = party_state.members[pi].kind.name();
        lines.push(format!("{}の HPが {}かいふく！", target_name, amount));
    }

    state.phase = FieldMenuPhase::ShowMessage {
        message: lines.join("\n"),
    };
}

fn handle_target_select(
    keyboard: &ButtonInput<KeyCode>,
    state: &mut FieldMenuState,
    party_state: &mut PartyState,
    candidates: Vec<usize>,
    mut cursor: usize,
    context: TargetContext,
) {
    let count = candidates.len();
    if input_ui::is_up_just_pressed(keyboard) && cursor > 0 {
        cursor -= 1;
    }
    if input_ui::is_down_just_pressed(keyboard) && cursor < count - 1 {
        cursor += 1;
    }
    state.phase = FieldMenuPhase::TargetSelect {
        candidates: candidates.clone(),
        cursor,
        context: context.clone(),
    };

    if input_ui::is_cancel_just_pressed(keyboard) {
        match context {
            TargetContext::Item { member, items, item_cursor } => {
                state.phase = FieldMenuPhase::ItemSelect { member, items, cursor: item_cursor };
            }
            TargetContext::Spell { caster, spells, spell_cursor } => {
                state.phase = FieldMenuPhase::SpellSelect { caster, spells, cursor: spell_cursor };
            }
        }
        return;
    }

    if input_ui::is_confirm_just_pressed(keyboard) {
        let target_idx = candidates[cursor];

        let message = match context {
            TargetContext::Item { member, ref items, item_cursor } => {
                let item = items[item_cursor];
                if let ItemEffect::Heal { power } = item.effect() {
                    let used = party_state.members[member].inventory.use_item(item);
                    if !used {
                        return;
                    }

                    let random_factor = 0.8 + rand::random::<f32>() * 0.4;
                    let amount = calculate_heal_amount(power, random_factor);

                    let target = &mut party_state.members[target_idx];
                    target.stats.hp = (target.stats.hp + amount).min(target.stats.max_hp);

                    let member_name = party_state.members[member].kind.name();
                    let target_name = party_state.members[target_idx].kind.name();
                    format!(
                        "{}は {}を つかった！\n{}の HPが {}かいふく！",
                        member_name,
                        item.name(),
                        target_name,
                        amount
                    )
                } else {
                    return;
                }
            }
            TargetContext::Spell { caster, ref spells, spell_cursor } => {
                let spell = spells[spell_cursor];

                let consumed = party_state.members[caster].stats.use_mp(spell.mp_cost());
                if !consumed {
                    return;
                }

                let random_factor = 0.8 + rand::random::<f32>() * 0.4;
                let amount = calculate_heal_amount(spell.power(), random_factor);

                let target = &mut party_state.members[target_idx];
                target.stats.hp = (target.stats.hp + amount).min(target.stats.max_hp);

                let caster_name = party_state.members[caster].kind.name();
                let target_name = party_state.members[target_idx].kind.name();
                format!(
                    "{}は {}を となえた！\n{}の HPが {}かいふく！",
                    caster_name,
                    spell.name(),
                    target_name,
                    amount
                )
            }
        };
        state.phase = FieldMenuPhase::ShowMessage { message };
    }
}

/// フィールドメニューの表示更新システム
#[allow(clippy::type_complexity)]
pub fn field_menu_display_system(
    state: Option<Res<FieldMenuState>>,
    party_state: Res<PartyState>,
    mut title_query: Query<&mut Text, (With<FieldMenuTitle>, Without<FieldMenuItem>, Without<FieldMenuScrollUp>, Without<FieldMenuScrollDown>)>,
    mut item_query: Query<
        (&FieldMenuItem, &mut Text, &mut TextColor, &mut Visibility, &mut Node),
        (Without<FieldMenuTitle>, Without<FieldMenuScrollUp>, Without<FieldMenuScrollDown>),
    >,
    mut scroll_up_query: Query<(&mut Visibility, &mut Node), (With<FieldMenuScrollUp>, Without<FieldMenuTitle>, Without<FieldMenuItem>, Without<FieldMenuScrollDown>)>,
    mut scroll_down_query: Query<(&mut Visibility, &mut Node), (With<FieldMenuScrollDown>, Without<FieldMenuTitle>, Without<FieldMenuItem>, Without<FieldMenuScrollUp>)>,
) {
    let Some(state) = state else { return };

    // タイトル更新
    for mut text in &mut title_query {
        match &state.phase {
            FieldMenuPhase::TopMenu { .. } => {
                **text = String::new();
            }
            FieldMenuPhase::CasterSelect { .. } => {
                **text = "だれが じゅもんを つかう？".to_string();
            }
            FieldMenuPhase::SpellSelect { caster, .. } => {
                let name = party_state.members[*caster].kind.name();
                **text = format!("{}の じゅもん", name);
            }
            FieldMenuPhase::MemberSelect { .. } => {
                **text = "だれの どうぐを つかう？".to_string();
            }
            FieldMenuPhase::ItemSelect { member, .. } => {
                let name = party_state.members[*member].kind.name();
                **text = format!("{}の もちもの", name);
            }
            FieldMenuPhase::TargetSelect { .. } => {
                **text = "だれに つかう？".to_string();
            }
            FieldMenuPhase::ShowMessage { message } => {
                **text = message.clone();
            }
        }
    }

    // メニューアイテム更新
    for (item, mut text, mut color, mut vis, mut node) in &mut item_query {
        match &state.phase {
            FieldMenuPhase::TopMenu { cursor } => {
                let labels = ["じゅもん", "どうぐ"];
                if item.index < labels.len() {
                    let is_selected = item.index == *cursor;
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format!("{}{}", prefix, labels[item.index]);
                    *color = if is_selected {
                        TextColor(SELECTED_COLOR)
                    } else {
                        TextColor(UNSELECTED_COLOR)
                    };
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            FieldMenuPhase::CasterSelect { candidates, cursor } => {
                if item.index < candidates.len() {
                    let member_idx = candidates[item.index];
                    let member = &party_state.members[member_idx];
                    let is_selected = item.index == *cursor;
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format!(
                        "{}{} HP:{}/{} MP:{}/{}",
                        prefix,
                        member.kind.name(),
                        member.stats.hp,
                        member.stats.max_hp,
                        member.stats.mp,
                        member.stats.max_mp,
                    );
                    *color = if is_selected {
                        TextColor(SELECTED_COLOR)
                    } else {
                        TextColor(UNSELECTED_COLOR)
                    };
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            FieldMenuPhase::SpellSelect { caster, spells, cursor } => {
                let offset = scroll_offset(*cursor, spells.len(), VISIBLE_ITEMS);
                let data_index = offset + item.index;
                if item.index < VISIBLE_ITEMS && data_index < spells.len() {
                    let spell = spells[data_index];
                    let is_selected = data_index == *cursor;
                    let caster_mp = party_state.members[*caster].stats.mp;
                    let can_use = caster_mp >= spell.mp_cost();
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format!("{}{} ({})", prefix, spell.name(), spell.mp_cost());
                    *color = if !can_use {
                        TextColor(DISABLED_COLOR)
                    } else if is_selected {
                        TextColor(SELECTED_COLOR)
                    } else {
                        TextColor(UNSELECTED_COLOR)
                    };
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            FieldMenuPhase::MemberSelect { candidates, cursor } => {
                if item.index < candidates.len() {
                    let member_idx = candidates[item.index];
                    let member = &party_state.members[member_idx];
                    let is_selected = item.index == *cursor;
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format!(
                        "{}{} HP:{}/{}",
                        prefix,
                        member.kind.name(),
                        member.stats.hp,
                        member.stats.max_hp,
                    );
                    *color = if is_selected {
                        TextColor(SELECTED_COLOR)
                    } else {
                        TextColor(UNSELECTED_COLOR)
                    };
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            FieldMenuPhase::ItemSelect { member, items, cursor } => {
                let offset = scroll_offset(*cursor, items.len(), VISIBLE_ITEMS);
                let data_index = offset + item.index;
                if item.index < VISIBLE_ITEMS && data_index < items.len() {
                    let item_kind = items[data_index];
                    let count = party_state.members[*member]
                        .inventory
                        .count(item_kind);
                    let is_selected = data_index == *cursor;
                    let prefix = if is_selected { "> " } else { "  " };
                    if let Some(w) = item_kind.as_weapon() {
                        **text = format!("{}{} ATK+{} x{}", prefix, item_kind.name(), w.attack_bonus(), count);
                    } else {
                        **text = format!("{}{} x{}", prefix, item_kind.name(), count);
                    }
                    *color = if is_selected {
                        TextColor(SELECTED_COLOR)
                    } else {
                        TextColor(UNSELECTED_COLOR)
                    };
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            FieldMenuPhase::TargetSelect { candidates, cursor, .. } => {
                if item.index < candidates.len() {
                    let member_idx = candidates[item.index];
                    let member = &party_state.members[member_idx];
                    let is_selected = item.index == *cursor;
                    let prefix = if is_selected { "> " } else { "  " };
                    **text = format!(
                        "{}{} HP:{}/{}",
                        prefix,
                        member.kind.name(),
                        member.stats.hp,
                        member.stats.max_hp,
                    );
                    *color = if is_selected {
                        TextColor(SELECTED_COLOR)
                    } else {
                        TextColor(UNSELECTED_COLOR)
                    };
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            FieldMenuPhase::ShowMessage { .. } => {
                *vis = Visibility::Hidden;
            }
        }
        // Display::NoneでレイアウトからもHiddenアイテムを除外
        node.display = if *vis == Visibility::Hidden { Display::None } else { Display::DEFAULT };
    }

    // スクロールインジケータ更新
    let (scroll_total, scroll_cursor) = match &state.phase {
        FieldMenuPhase::SpellSelect { spells, cursor, .. } => (spells.len(), *cursor),
        FieldMenuPhase::ItemSelect { items, cursor, .. } => (items.len(), *cursor),
        _ => (0, 0),
    };

    if scroll_total > VISIBLE_ITEMS {
        let offset = scroll_offset(scroll_cursor, scroll_total, VISIBLE_ITEMS);
        for (mut vis, mut node) in &mut scroll_up_query {
            *vis = if offset > 0 { Visibility::Inherited } else { Visibility::Hidden };
            node.display = if *vis == Visibility::Hidden { Display::None } else { Display::DEFAULT };
        }
        for (mut vis, mut node) in &mut scroll_down_query {
            *vis = if offset + VISIBLE_ITEMS < scroll_total { Visibility::Inherited } else { Visibility::Hidden };
            node.display = if *vis == Visibility::Hidden { Display::None } else { Display::DEFAULT };
        }
    } else {
        for (mut vis, mut node) in &mut scroll_up_query {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }
        for (mut vis, mut node) in &mut scroll_down_query {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }
    }
}
