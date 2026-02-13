//! リモート制御用のコマンド定義とパース（Bevy非依存）

/// リモートコマンドの種類
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteCommand {
    /// キー入力のシミュレート
    KeyPress(RemoteKey),
    /// 状態ダンプ要求
    QueryState,
    /// Nフレーム待機
    Wait(u32),
    /// 入力間隔の設定（フレーム数）
    SetInputInterval(u32),
    /// アプリケーション終了
    Quit,
}

/// リモートで送信可能なキー
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteKey {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
    MapToggle,
}

/// 1行のJSONコマンドをパースする
///
/// フォーマット例:
/// - `{"cmd":"key","key":"up"}`
/// - `{"cmd":"wait","frames":30}`
/// - `{"cmd":"query_state"}`
pub fn parse_command(line: &str) -> Result<RemoteCommand, String> {
    let line = line.trim();
    if line.is_empty() {
        return Err("empty line".to_string());
    }

    // 簡易JSONパーサ（serde_json依存を避ける）
    let obj = parse_json_object(line)?;

    let cmd = obj
        .get("cmd")
        .ok_or_else(|| "missing 'cmd' field".to_string())?
        .as_str();

    match cmd {
        "key" => {
            let key_str = obj
                .get("key")
                .ok_or_else(|| "missing 'key' field".to_string())?
                .as_str();
            let key = parse_remote_key(key_str)?;
            Ok(RemoteCommand::KeyPress(key))
        }
        "query_state" => Ok(RemoteCommand::QueryState),
        "wait" => {
            let frames_str = obj
                .get("frames")
                .ok_or_else(|| "missing 'frames' field".to_string())?
                .as_str();
            let frames: u32 = frames_str
                .parse()
                .map_err(|_| format!("invalid frames value: {}", frames_str))?;
            Ok(RemoteCommand::Wait(frames))
        }
        "set_input_interval" => {
            let frames_str = obj
                .get("frames")
                .ok_or_else(|| "missing 'frames' field".to_string())?
                .as_str();
            let frames: u32 = frames_str
                .parse()
                .map_err(|_| format!("invalid frames value: {}", frames_str))?;
            Ok(RemoteCommand::SetInputInterval(frames))
        }
        "quit" => Ok(RemoteCommand::Quit),
        _ => Err(format!("unknown command: {}", cmd)),
    }
}

fn parse_remote_key(s: &str) -> Result<RemoteKey, String> {
    match s {
        "up" => Ok(RemoteKey::Up),
        "down" => Ok(RemoteKey::Down),
        "left" => Ok(RemoteKey::Left),
        "right" => Ok(RemoteKey::Right),
        "confirm" | "enter" | "z" => Ok(RemoteKey::Confirm),
        "cancel" | "escape" | "x" => Ok(RemoteKey::Cancel),
        "map" | "m" => Ok(RemoteKey::MapToggle),
        _ => Err(format!("unknown key: {}", s)),
    }
}

/// 簡易的なJSON値
#[derive(Debug, Clone)]
enum JsonValue {
    Str(String),
    Num(String),
    Null,
}

impl JsonValue {
    fn as_str(&self) -> &str {
        match self {
            JsonValue::Str(s) => s,
            JsonValue::Num(s) => s,
            JsonValue::Null => "",
        }
    }
}

/// 簡易JSONオブジェクトパーサ（フラットなstring/number値のみ対応）
fn parse_json_object(s: &str) -> Result<std::collections::HashMap<String, JsonValue>, String> {
    let s = s.trim();
    if !s.starts_with('{') || !s.ends_with('}') {
        return Err("expected JSON object".to_string());
    }

    let inner = &s[1..s.len() - 1];
    let mut map = std::collections::HashMap::new();
    let mut chars = inner.chars().peekable();

    loop {
        skip_whitespace(&mut chars);
        if chars.peek().is_none() {
            break;
        }

        // キー
        let key = parse_json_string(&mut chars)?;
        skip_whitespace(&mut chars);

        // コロン
        if chars.next() != Some(':') {
            return Err("expected ':'".to_string());
        }
        skip_whitespace(&mut chars);

        // 値
        let value = match chars.peek() {
            Some('"') => JsonValue::Str(parse_json_string(&mut chars)?),
            Some('n') => {
                // null
                for expected in ['n', 'u', 'l', 'l'] {
                    if chars.next() != Some(expected) {
                        return Err("invalid null value".to_string());
                    }
                }
                JsonValue::Null
            }
            Some(c) if c.is_ascii_digit() => {
                let mut num = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                JsonValue::Num(num)
            }
            _ => return Err("unexpected value".to_string()),
        };

        map.insert(key, value);

        skip_whitespace(&mut chars);
        if chars.peek() == Some(&',') {
            chars.next();
        }
    }

    Ok(map)
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn parse_json_string(
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<String, String> {
    if chars.next() != Some('"') {
        return Err("expected '\"'".to_string());
    }

    let mut s = String::new();
    loop {
        match chars.next() {
            Some('"') => return Ok(s),
            Some('\\') => match chars.next() {
                Some(c) => s.push(c),
                None => return Err("unexpected end of string escape".to_string()),
            },
            Some(c) => s.push(c),
            None => return Err("unexpected end of string".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_commands() {
        assert_eq!(
            parse_command(r#"{"cmd":"key","key":"up"}"#).unwrap(),
            RemoteCommand::KeyPress(RemoteKey::Up)
        );
        assert_eq!(
            parse_command(r#"{"cmd":"key","key":"down"}"#).unwrap(),
            RemoteCommand::KeyPress(RemoteKey::Down)
        );
        assert_eq!(
            parse_command(r#"{"cmd":"key","key":"left"}"#).unwrap(),
            RemoteCommand::KeyPress(RemoteKey::Left)
        );
        assert_eq!(
            parse_command(r#"{"cmd":"key","key":"right"}"#).unwrap(),
            RemoteCommand::KeyPress(RemoteKey::Right)
        );
        assert_eq!(
            parse_command(r#"{"cmd":"key","key":"confirm"}"#).unwrap(),
            RemoteCommand::KeyPress(RemoteKey::Confirm)
        );
        assert_eq!(
            parse_command(r#"{"cmd":"key","key":"enter"}"#).unwrap(),
            RemoteCommand::KeyPress(RemoteKey::Confirm)
        );
        assert_eq!(
            parse_command(r#"{"cmd":"key","key":"cancel"}"#).unwrap(),
            RemoteCommand::KeyPress(RemoteKey::Cancel)
        );
        assert_eq!(
            parse_command(r#"{"cmd":"key","key":"map"}"#).unwrap(),
            RemoteCommand::KeyPress(RemoteKey::MapToggle)
        );
    }

    #[test]
    fn test_parse_wait() {
        assert_eq!(
            parse_command(r#"{"cmd":"wait","frames":30}"#).unwrap(),
            RemoteCommand::Wait(30)
        );
    }

    #[test]
    fn test_parse_query_state() {
        assert_eq!(
            parse_command(r#"{"cmd":"query_state"}"#).unwrap(),
            RemoteCommand::QueryState
        );
    }

    #[test]
    fn test_parse_set_input_interval() {
        assert_eq!(
            parse_command(r#"{"cmd":"set_input_interval","frames":8}"#).unwrap(),
            RemoteCommand::SetInputInterval(8)
        );
        assert_eq!(
            parse_command(r#"{"cmd":"set_input_interval","frames":0}"#).unwrap(),
            RemoteCommand::SetInputInterval(0)
        );
    }

    #[test]
    fn test_parse_quit() {
        assert_eq!(
            parse_command(r#"{"cmd":"quit"}"#).unwrap(),
            RemoteCommand::Quit
        );
    }

    #[test]
    fn test_parse_errors() {
        assert!(parse_command("").is_err());
        assert!(parse_command("not json").is_err());
        assert!(parse_command(r#"{"cmd":"unknown"}"#).is_err());
        assert!(parse_command(r#"{"cmd":"key"}"#).is_err()); // missing key
        assert!(parse_command(r#"{"cmd":"key","key":"invalid"}"#).is_err());
        assert!(parse_command(r#"{"cmd":"wait"}"#).is_err()); // missing frames
    }

    #[test]
    fn test_parse_with_whitespace() {
        assert_eq!(
            parse_command(r#"  {"cmd": "key", "key": "up"}  "#).unwrap(),
            RemoteCommand::KeyPress(RemoteKey::Up)
        );
    }
}
