use bevy::prelude::*;

use app_state::{FieldMenuOpen, InField, PartyState};
use hud_ui::command_menu::{
    self, CommandMenu, CommandMenuItem, CommandMenuScrollDown, CommandMenuScrollUp,
};
use input_ui::InputSystemSet;
use party::available_spells;
use party::{ItemEffect, ItemKind};
use spell::{calculate_heal_amount, SpellKind, SpellTarget};

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
    cached_labels: Vec<String>,
    disabled_indices: Vec<usize>,
}

impl FieldMenuState {
    pub fn new(phase: FieldMenuPhase, party_state: &PartyState) -> Self {
        let mut s = Self {
            phase,
            cached_labels: Vec::new(),
            disabled_indices: Vec::new(),
        };
        s.rebuild_cache(party_state);
        s
    }

    fn set_phase(&mut self, phase: FieldMenuPhase, party_state: &PartyState) {
        self.phase = phase;
        self.rebuild_cache(party_state);
    }

    fn rebuild_cache(&mut self, party_state: &PartyState) {
        self.cached_labels.clear();
        self.disabled_indices.clear();
        match &self.phase {
            FieldMenuPhase::TopMenu { .. } => {
                self.cached_labels = vec!["じゅもん".to_string(), "どうぐ".to_string()];
            }
            FieldMenuPhase::CasterSelect { candidates, .. } => {
                for &idx in candidates {
                    let m = &party_state.members[idx];
                    self.cached_labels.push(format!(
                        "{} HP:{}/{} MP:{}/{}",
                        m.kind.name(),
                        m.stats.hp,
                        m.stats.max_hp,
                        m.stats.mp,
                        m.stats.max_mp,
                    ));
                }
            }
            FieldMenuPhase::SpellSelect { caster, spells, .. } => {
                let caster_mp = party_state.members[*caster].stats.mp;
                for (i, &spell) in spells.iter().enumerate() {
                    self.cached_labels
                        .push(format!("{} ({})", spell.name(), spell.mp_cost()));
                    if caster_mp < spell.mp_cost() {
                        self.disabled_indices.push(i);
                    }
                }
            }
            FieldMenuPhase::MemberSelect { candidates, .. } => {
                for &idx in candidates {
                    let m = &party_state.members[idx];
                    self.cached_labels.push(format!(
                        "{} HP:{}/{}",
                        m.kind.name(),
                        m.stats.hp,
                        m.stats.max_hp,
                    ));
                }
            }
            FieldMenuPhase::ItemSelect { member, items, .. } => {
                let member_data = &party_state.members[*member];
                for &item in items {
                    let count = member_data.inventory.count(item);
                    if let Some(w) = item.as_weapon() {
                        let equipped = member_data.equipment.weapon == Some(w);
                        let equip_mark = if equipped { "E " } else { "" };
                        self.cached_labels.push(format!(
                            "{}{} ATK+{} x{}",
                            equip_mark,
                            item.name(),
                            w.attack_bonus(),
                            count
                        ));
                    } else {
                        self.cached_labels
                            .push(format!("{} x{}", item.name(), count));
                    }
                }
            }
            FieldMenuPhase::TargetSelect { candidates, .. } => {
                for &idx in candidates {
                    let m = &party_state.members[idx];
                    self.cached_labels.push(format!(
                        "{} HP:{}/{}",
                        m.kind.name(),
                        m.stats.hp,
                        m.stats.max_hp,
                    ));
                }
            }
            FieldMenuPhase::ShowMessage { .. } => {}
        }
    }
}

impl CommandMenu for FieldMenuState {
    fn menu_labels(&self) -> Vec<String> {
        self.cached_labels.clone()
    }

    fn selected(&self) -> usize {
        match &self.phase {
            FieldMenuPhase::TopMenu { cursor }
            | FieldMenuPhase::CasterSelect { cursor, .. }
            | FieldMenuPhase::SpellSelect { cursor, .. }
            | FieldMenuPhase::MemberSelect { cursor, .. }
            | FieldMenuPhase::ItemSelect { cursor, .. }
            | FieldMenuPhase::TargetSelect { cursor, .. } => *cursor,
            FieldMenuPhase::ShowMessage { .. } => 0,
        }
    }

    fn set_selected(&mut self, index: usize) {
        match &mut self.phase {
            FieldMenuPhase::TopMenu { cursor }
            | FieldMenuPhase::CasterSelect { cursor, .. }
            | FieldMenuPhase::SpellSelect { cursor, .. }
            | FieldMenuPhase::MemberSelect { cursor, .. }
            | FieldMenuPhase::ItemSelect { cursor, .. }
            | FieldMenuPhase::TargetSelect { cursor, .. } => *cursor = index,
            FieldMenuPhase::ShowMessage { .. } => {}
        }
    }

    fn is_active(&self) -> bool {
        !matches!(self.phase, FieldMenuPhase::ShowMessage { .. })
    }

    fn visible_items(&self) -> Option<usize> {
        match &self.phase {
            FieldMenuPhase::SpellSelect { .. } | FieldMenuPhase::ItemSelect { .. } => {
                Some(VISIBLE_ITEMS)
            }
            _ => None,
        }
    }

    fn is_disabled(&self, index: usize) -> bool {
        self.disabled_indices.contains(&index)
    }
}

/// フィールドメニューのUIルートマーカー
#[derive(Component)]
pub struct FieldMenuRoot;

/// フィールドメニューのタイトルテキスト
#[derive(Component)]
pub struct FieldMenuTitle;

const MAX_MENU_ITEMS: usize = 16;
const VISIBLE_ITEMS: usize = 6;

pub struct FieldMenuPlugin;

impl Plugin for FieldMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                field_menu_input_system
                    .in_set(InputSystemSet::FieldMenuInput)
                    .after(InputSystemSet::MessageInput),
                command_menu::command_menu_display_system::<FieldMenuState>,
                field_menu_title_system,
            )
                .chain()
                .run_if(in_state(InField)),
        );
    }
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
                        CommandMenuScrollUp,
                        Text::new("  ▲"),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Visibility::Hidden,
                    ));

                    // メニューアイテム
                    for i in 0..MAX_MENU_ITEMS {
                        menu_box.spawn((
                            CommandMenuItem { index: i },
                            Text::new(""),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Visibility::Hidden,
                        ));
                    }

                    // ▼ スクロールインジケータ
                    menu_box.spawn((
                        CommandMenuScrollDown,
                        Text::new("  ▼"),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
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

fn close_menu(commands: &mut Commands, root_query: &Query<Entity, With<FieldMenuRoot>>) {
    despawn_menu_ui(commands, root_query);
    commands.remove_resource::<FieldMenuState>();
    commands.remove_resource::<FieldMenuOpen>();
}

/// フィールドメニューの入力処理システム
#[allow(clippy::too_many_arguments)]
fn field_menu_input_system(
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

            commands.insert_resource(FieldMenuState::new(
                FieldMenuPhase::TopMenu { cursor: 0 },
                &party_state,
            ));

            spawn_menu_ui(&mut commands, &asset_server);
            commands.insert_resource(FieldMenuOpen);
        }
        return;
    }

    let Some(mut state) = state else { return };

    match state.phase.clone() {
        FieldMenuPhase::TopMenu { cursor } => {
            handle_top_menu(
                &keyboard,
                &mut state,
                &party_state,
                &mut commands,
                &root_query,
                cursor,
            );
        }
        FieldMenuPhase::CasterSelect { candidates, cursor } => {
            handle_caster_select(&keyboard, &mut state, &party_state, candidates, cursor);
        }
        FieldMenuPhase::SpellSelect {
            caster,
            spells,
            cursor,
        } => {
            handle_spell_select(
                &keyboard,
                &mut state,
                &mut party_state,
                caster,
                spells,
                cursor,
            );
        }
        FieldMenuPhase::MemberSelect { candidates, cursor } => {
            handle_member_select(&keyboard, &mut state, &party_state, candidates, cursor);
        }
        FieldMenuPhase::ItemSelect {
            member,
            items,
            cursor,
        } => {
            handle_item_select(
                &keyboard,
                &mut state,
                &mut party_state,
                member,
                items,
                cursor,
            );
        }
        FieldMenuPhase::TargetSelect {
            candidates,
            cursor,
            context,
        } => {
            handle_target_select(
                &keyboard,
                &mut state,
                &mut party_state,
                candidates,
                cursor,
                context,
            );
        }
        FieldMenuPhase::ShowMessage { .. } => {
            if input_ui::is_confirm_just_pressed(&keyboard) {
                state.set_phase(FieldMenuPhase::TopMenu { cursor: 0 }, &party_state);
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
            state.set_phase(
                FieldMenuPhase::CasterSelect {
                    candidates,
                    cursor: 0,
                },
                party_state,
            );
        } else {
            // どうぐ → MemberSelect
            let candidates = alive_member_indices(party_state);
            state.set_phase(
                FieldMenuPhase::MemberSelect {
                    candidates,
                    cursor: 0,
                },
                party_state,
            );
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
    state.phase = FieldMenuPhase::CasterSelect {
        candidates: candidates.clone(),
        cursor,
    };

    if input_ui::is_cancel_just_pressed(keyboard) {
        state.set_phase(FieldMenuPhase::TopMenu { cursor: 0 }, party_state);
        return;
    }

    if input_ui::is_confirm_just_pressed(keyboard) {
        let member_idx = candidates[cursor];
        let spells = available_spells(
            party_state.members[member_idx].kind,
            party_state.members[member_idx].level,
        );
        if spells.is_empty() {
            state.set_phase(
                FieldMenuPhase::ShowMessage {
                    message: "じゅもんを おぼえていない".to_string(),
                },
                party_state,
            );
        } else {
            state.set_phase(
                FieldMenuPhase::SpellSelect {
                    caster: member_idx,
                    spells,
                    cursor: 0,
                },
                party_state,
            );
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
    state.phase = FieldMenuPhase::SpellSelect {
        caster,
        spells: spells.clone(),
        cursor,
    };

    if input_ui::is_cancel_just_pressed(keyboard) {
        let candidates = alive_member_indices(party_state);
        state.set_phase(
            FieldMenuPhase::CasterSelect {
                candidates,
                cursor: 0,
            },
            party_state,
        );
        return;
    }

    if input_ui::is_confirm_just_pressed(keyboard) {
        let spell = spells[cursor];
        let caster_member = &party_state.members[caster];

        if caster_member.stats.mp < spell.mp_cost() {
            return;
        }

        if !spell.is_usable_in_field() {
            state.set_phase(
                FieldMenuPhase::ShowMessage {
                    message: "フィールドでは つかえない".to_string(),
                },
                party_state,
            );
        } else if spell.target_type() == SpellTarget::AllAllies {
            // 全体回復: ターゲット選択スキップ、全味方に一括実行
            execute_aoe_heal(state, party_state, caster, spell);
        } else {
            // 単体回復: ターゲット選択へ
            let target_candidates = alive_member_indices(party_state);
            state.set_phase(
                FieldMenuPhase::TargetSelect {
                    candidates: target_candidates,
                    cursor: 0,
                    context: TargetContext::Spell {
                        caster,
                        spells,
                        spell_cursor: cursor,
                    },
                },
                party_state,
            );
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
    state.phase = FieldMenuPhase::MemberSelect {
        candidates: candidates.clone(),
        cursor,
    };

    if input_ui::is_cancel_just_pressed(keyboard) {
        state.set_phase(FieldMenuPhase::TopMenu { cursor: 0 }, party_state);
        return;
    }

    if input_ui::is_confirm_just_pressed(keyboard) {
        let member_idx = candidates[cursor];
        let items = party_state.members[member_idx].inventory.owned_items();
        if items.is_empty() {
            state.set_phase(
                FieldMenuPhase::ShowMessage {
                    message: "もちものが ない".to_string(),
                },
                party_state,
            );
        } else {
            state.set_phase(
                FieldMenuPhase::ItemSelect {
                    member: member_idx,
                    items,
                    cursor: 0,
                },
                party_state,
            );
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
    state.phase = FieldMenuPhase::ItemSelect {
        member,
        items: items.clone(),
        cursor,
    };

    if input_ui::is_cancel_just_pressed(keyboard) {
        let candidates = alive_member_indices(party_state);
        state.set_phase(
            FieldMenuPhase::MemberSelect {
                candidates,
                cursor: 0,
            },
            party_state,
        );
        return;
    }

    if input_ui::is_confirm_just_pressed(keyboard) {
        let item = items[cursor];
        match item.effect() {
            ItemEffect::Heal { .. } => {
                let target_candidates = alive_member_indices(party_state);
                state.set_phase(
                    FieldMenuPhase::TargetSelect {
                        candidates: target_candidates,
                        cursor: 0,
                        context: TargetContext::Item {
                            member,
                            items,
                            item_cursor: cursor,
                        },
                    },
                    party_state,
                );
            }
            ItemEffect::Equip => {
                let weapon = item.as_weapon().expect("Equip effect must be Weapon");
                let member_ref = &mut party_state.members[member];
                let member_name = member_ref.kind.name();
                let old_weapon = member_ref.equipment.equip_weapon(weapon);
                let mut msg = format!("{}は {}を そうびした！", member_name, weapon.name());
                if let Some(old_w) = old_weapon {
                    msg.push_str(&format!("\n{}を はずした", old_w.name()));
                }
                state.set_phase(
                    FieldMenuPhase::ShowMessage { message: msg },
                    party_state,
                );
            }
            ItemEffect::KeyItem | ItemEffect::Material => {
                let member_name = party_state.members[member].kind.name();
                state.set_phase(
                    FieldMenuPhase::ShowMessage {
                        message: format!(
                            "{}は {}を しらべた。\n{}",
                            member_name,
                            item.name(),
                            item.description()
                        ),
                    },
                    party_state,
                );
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

    state.set_phase(
        FieldMenuPhase::ShowMessage {
            message: lines.join("\n"),
        },
        party_state,
    );
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
            TargetContext::Item {
                member,
                items,
                item_cursor,
            } => {
                state.set_phase(
                    FieldMenuPhase::ItemSelect {
                        member,
                        items,
                        cursor: item_cursor,
                    },
                    party_state,
                );
            }
            TargetContext::Spell {
                caster,
                spells,
                spell_cursor,
            } => {
                state.set_phase(
                    FieldMenuPhase::SpellSelect {
                        caster,
                        spells,
                        cursor: spell_cursor,
                    },
                    party_state,
                );
            }
        }
        return;
    }

    if input_ui::is_confirm_just_pressed(keyboard) {
        let target_idx = candidates[cursor];

        let message = match context {
            TargetContext::Item {
                member,
                ref items,
                item_cursor,
            } => {
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
            TargetContext::Spell {
                caster,
                ref spells,
                spell_cursor,
            } => {
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
        state.set_phase(
            FieldMenuPhase::ShowMessage { message },
            party_state,
        );
    }
}

/// フィールドメニューのタイトル・メッセージ表示システム
fn field_menu_title_system(
    state: Option<Res<FieldMenuState>>,
    party_state: Res<PartyState>,
    mut title_query: Query<&mut Text, (With<FieldMenuTitle>, Without<CommandMenuItem>)>,
) {
    let Some(state) = state else { return };

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
}
