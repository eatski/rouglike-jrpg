use bevy::prelude::*;

use super::scene::{BattlePhase, BattleResource};

/// 敵HP表示テキストのマーカー
#[derive(Component)]
pub struct EnemyHpText;

/// メッセージ表示テキストのマーカー
#[derive(Component)]
pub struct MessageText;

/// プレイヤーHP表示テキストのマーカー
#[derive(Component)]
pub struct PlayerHpText;

/// コマンドカーソルのマーカー（indexでどのコマンドかを識別）
#[derive(Component)]
pub struct CommandCursor {
    pub index: usize,
}

/// 戦闘画面の表示を更新するシステム
#[allow(clippy::type_complexity)]
pub fn battle_display_system(
    battle_res: Res<BattleResource>,
    mut enemy_hp_query: Query<&mut Text, (With<EnemyHpText>, Without<MessageText>, Without<PlayerHpText>, Without<CommandCursor>)>,
    mut message_query: Query<&mut Text, (With<MessageText>, Without<EnemyHpText>, Without<PlayerHpText>, Without<CommandCursor>)>,
    mut player_hp_query: Query<&mut Text, (With<PlayerHpText>, Without<EnemyHpText>, Without<MessageText>, Without<CommandCursor>)>,
    mut command_query: Query<(&CommandCursor, &mut Text), (Without<EnemyHpText>, Without<MessageText>, Without<PlayerHpText>)>,
) {
    // 敵HP更新
    for mut text in &mut enemy_hp_query {
        let enemy = &battle_res.state.enemy;
        **text = format!(
            "{} HP: {}/{}",
            enemy.kind.name(),
            enemy.stats.hp,
            enemy.stats.max_hp
        );
    }

    // プレイヤーHP更新
    for mut text in &mut player_hp_query {
        **text = format!(
            "HP: {}/{}",
            battle_res.state.player.hp, battle_res.state.player.max_hp
        );
    }

    // メッセージ更新
    for mut text in &mut message_query {
        match &battle_res.phase {
            BattlePhase::CommandSelect => {
                **text = "コマンド？".to_string();
            }
            BattlePhase::ShowMessage { messages, index } => {
                if let Some(msg) = messages.get(*index) {
                    **text = msg.clone();
                }
            }
            BattlePhase::BattleOver { message } => {
                **text = message.clone();
            }
        }
    }

    // コマンドカーソル更新
    let commands = ["たたかう", "にげる"];
    for (cursor, mut text) in &mut command_query {
        let prefix = if cursor.index == battle_res.selected_command {
            "> "
        } else {
            "  "
        };
        **text = format!("{}{}", prefix, commands[cursor.index]);
    }
}
