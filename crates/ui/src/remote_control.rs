use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use game::remote::{parse_command, RemoteCommand, RemoteKey};

use crate::app_state::AppState;
use crate::components::{Player, TilePosition};

/// リモート制御モードの設定リソース
#[derive(Resource)]
pub struct RemoteControlMode {
    /// コマンドファイルパス
    pub command_file: PathBuf,
    /// レスポンスファイルパス
    pub response_file: PathBuf,
    /// 処理済みコマンド行数
    processed_lines: usize,
    /// 待機カウンター
    wait_frames: u32,
    /// スクショ要求キュー
    screenshot_requests: Vec<Option<String>>,
    /// フレームカウンタ
    frame_count: u64,
}

impl Default for RemoteControlMode {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoteControlMode {
    pub fn new() -> Self {
        Self {
            command_file: PathBuf::from("remote/commands.jsonl"),
            response_file: PathBuf::from("remote/response.jsonl"),
            processed_lines: 0,
            wait_frames: 0,
            screenshot_requests: Vec::new(),
            frame_count: 0,
        }
    }

    fn append_response(&self, line: &str) {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.response_file)
        {
            let _ = writeln!(f, "{}", line);
        }
    }
}

/// 仮想入力バッファ（1フレーム分の入力をバッファリング）
#[derive(Resource, Default)]
pub struct VirtualInput {
    /// 今フレームで「just_pressed」として扱うキー
    pub just_pressed: Vec<RemoteKey>,
    /// 今フレームで「pressed（押しっぱなし）」として扱うキー
    pub pressed: Vec<RemoteKey>,
}

/// コマンドファイルを読み取り VirtualInput に変換するシステム
pub fn read_remote_commands(mut remote: ResMut<RemoteControlMode>, mut vi: ResMut<VirtualInput>) {
    remote.frame_count += 1;

    // 待機中は後続コマンドを処理しない
    if remote.wait_frames > 0 {
        remote.wait_frames -= 1;
        return;
    }

    // ファイル読み取り（未処理行がある場合も含めて毎フレーム確認）
    let content = match std::fs::read_to_string(&remote.command_file) {
        Ok(c) => c,
        Err(_) => return,
    };

    let lines: Vec<&str> = content.lines().collect();
    if remote.processed_lines >= lines.len() {
        return;
    }
    let new_lines = &lines[remote.processed_lines..];

    // 1フレームに1コマンドのみ処理
    if let Some(line) = new_lines.first() {
        remote.processed_lines += 1;
        let frame = remote.frame_count;

        match parse_command(line) {
            Ok(RemoteCommand::KeyPress(key)) => {
                vi.just_pressed.push(key);
                vi.pressed.push(key);
                remote.append_response(&format!(
                    r#"{{"event":"command_processed","cmd":"key","frame":{}}}"#,
                    frame
                ));
            }
            Ok(RemoteCommand::Screenshot { filename }) => {
                remote.screenshot_requests.push(filename);
                // レスポンスはスクショ撮影時に出力
            }
            Ok(RemoteCommand::Wait(frames)) => {
                remote.wait_frames = frames;
                remote.append_response(&format!(
                    r#"{{"event":"command_processed","cmd":"wait","frames":{},"frame":{}}}"#,
                    frames, frame
                ));
            }
            Ok(RemoteCommand::QueryState) => {
                // 状態クエリは別のシステムで処理
                remote.append_response(&format!(
                    r#"{{"event":"command_processed","cmd":"query_state","frame":{}}}"#,
                    frame
                ));
            }
            Err(e) => {
                warn!("Remote command parse error: {} (line: {})", e, line);
            }
        }
    }
}

/// フレーム終了後に VirtualInput をクリアするシステム
pub fn clear_virtual_input(mut vi: ResMut<VirtualInput>) {
    vi.just_pressed.clear();
    vi.pressed.clear();
}

/// リモートスクリーンショット撮影システム
pub fn remote_screenshot_system(mut commands: Commands, mut remote: ResMut<RemoteControlMode>) {
    let requests: Vec<Option<String>> = remote.screenshot_requests.drain(..).collect();
    for filename in requests {
        std::fs::create_dir_all("screenshots").ok();
        let path = filename.unwrap_or_else(|| "screenshots/latest.png".to_string());
        let frame = remote.frame_count;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));
        remote.append_response(&format!(
            r#"{{"event":"screenshot_saved","path":"{}","frame":{}}}"#,
            path, frame
        ));
        info!("Remote screenshot: {}", path);
    }
}

/// ゲーム状態をレスポンスファイルに書き出すシステム
pub fn write_game_state_log(
    remote: Res<RemoteControlMode>,
    app_state: Res<State<AppState>>,
    player_query: Query<&TilePosition, With<Player>>,
) {
    // QueryStateコマンドのレスポンスとして状態を出力
    // （毎フレームではなく、query_stateコマンド受信時のみ出力するため、
    //   read_remote_commandsで処理済み。ここではフレーム開始時の ready イベントのみ）
    if remote.frame_count == 1 {
        let (px, py) = player_query
            .single()
            .map(|p| (p.x, p.y))
            .unwrap_or((0, 0));
        let state_name = format!("{:?}", app_state.get());
        remote.append_response(&format!(
            r#"{{"event":"ready","app_state":"{}","player_x":{},"player_y":{},"frame":{}}}"#,
            state_name, px, py, remote.frame_count
        ));
    }
}
