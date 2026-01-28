# アーキテクチャパターン集

## ゲームロジックとUIのcrate分離

### 基本原則

```
crates/game/（純粋Rust）     crates/ui/（Bevy依存）
├── 何が起きるか             ├── どう見せるか
├── ルールの判定             ├── アニメーション
├── 純粋関数                 ├── Component/Resource/Message
└── タイル座標               └── ワールド座標

Bevy非依存 ←───────────────→ Bevy依存
（他エンジンでも再利用可能）
```

### 依存関係

```
rouglike-jrpg (binary)
    ├── game (rand のみ依存)
    └── ui (bevy + game に依存)
```

### なぜcrate分離するか

1. **テスト容易性** - game crateは純粋Rust、Bevy無しで高速テスト
2. **変更の局所化** - 見た目の変更がロジックに影響しない
3. **再利用性** - 他のゲームエンジン（macroquad等）でも再利用可能
4. **コンパイル時間** - gameの変更時にBevyの再コンパイル不要
5. **責務の明確化** - Bevy型がgameに混入しない

### 分離の判断基準

| game/ に置くもの | ui/ に置くもの |
|---|---|
| タイル座標 (usize) | ワールド座標 (f32) |
| 移動可否の判定 | 移動アニメーション |
| 当たり判定 | 衝突エフェクト |
| ダメージ計算 | ダメージ表示 |
| ターン管理 | ターン遷移演出 |

**判断のコツ**: 「画面がなくても意味があるか？」→ Yes なら game/

## crate間通信パターン

### パターン: 純粋関数 + ui側Message

game crateは純粋関数で判定のみ、ui crateがBevy統合を担当。

```rust
// game/movement/player.rs - 純粋関数（Bevy非依存）
pub fn try_move(x: usize, y: usize, dx: i32, dy: i32, grid: &[Vec<Terrain>]) -> MoveResult {
    // ロジックのみ、Bevy型なし
}

pub enum MoveResult {
    Moved { new_x: usize, new_y: usize },
    Blocked,
}

// ui/events.rs - Bevy Message定義
#[derive(Message)]
pub struct MovementBlockedEvent {
    pub entity: Entity,
    pub direction: Direction,
}

// ui/player_input.rs - 純粋関数を呼び出してMessageを発行
fn player_movement(..., mut blocked_events: MessageWriter<MovementBlockedEvent>) {
    match game::movement::try_move(x, y, dx, dy, &grid) {
        MoveResult::Blocked => {
            blocked_events.write(MovementBlockedEvent { entity, direction });
        }
        MoveResult::Moved { new_x, new_y } => { ... }
    }
}

// ui/bounce.rs - Messageを受け取ってアニメーション
fn start_bounce(mut events: MessageReader<MovementBlockedEvent>) {
    for event in events.read() {
        commands.entity(event.entity).insert(Bounce { ... });
    }
}
```

### パターン: マーカーコンポーネント

アニメーション中は入力無効などの制御。**ui crateで完結**。

```rust
// ui/components.rs - ui側で定義
#[derive(Component)]
pub struct MovementLocked;

// ui/bounce.rs - アニメーション開始時に追加
commands.entity(entity).insert(MovementLocked);

// ui/bounce.rs - アニメーション終了時に削除
commands.entity(entity).remove::<MovementLocked>();

// ui/player_input.rs - 入力システムで判定
fn player_movement(query: Query<..., Option<&MovementLocked>>) {
    if locked.is_some() { return; }
}
```

## 座標系の変換

### タイル座標 → ワールド座標

```rust
// ui/constants.rs
pub const TILE_SIZE: f32 = 4.0;
pub const MAP_PIXEL_WIDTH: f32 = MAP_WIDTH as f32 * TILE_SIZE;

// 変換関数
fn tile_to_world(tile_x: usize, tile_y: usize) -> Vec2 {
    let origin_x = -MAP_PIXEL_WIDTH / 2.0 + TILE_SIZE / 2.0;
    let origin_y = -MAP_PIXEL_HEIGHT / 2.0 + TILE_SIZE / 2.0;
    Vec2::new(
        origin_x + tile_x as f32 * TILE_SIZE,
        origin_y + tile_y as f32 * TILE_SIZE,
    )
}
```

## 新機能追加時のチェックリスト

1. **ゲームルールか見た目か判断**
   - ルール → `crates/game/` に純粋関数として実装
   - 見た目 → `crates/ui/` に実装
   - 両方 → ロジックを`game/`、Bevy統合を`ui/`に分離

2. **Bevy型を使うか確認**
   - `Entity`, `Component`, `Resource`, `Message`, `Timer` → **ui/ のみ**
   - game/ には入れない

3. **定数の配置を決定**
   - ルールに関わる数値 → `crates/game/` 内
   - 見た目に関わる数値 → `crates/ui/src/constants.rs`

4. **依存方向を確認**
   - `ui/` は `game/` に依存してOK
   - `game/` は `ui/` に依存してはいけない
   - `game/` は `bevy` に依存してはいけない

## パフォーマンスチューニング

### 移動入力の推奨値

| パラメータ | 値 | 説明 |
|-----------|-----|------|
| リピート間隔 | 60ms | 連続移動時の1歩あたりの時間（約16.7歩/秒） |
| 初回遅延 | 150ms | キー押し続け時、リピート開始までの待機時間 |

### バウンスアニメーションの推奨値

| パラメータ | 値 | 説明 |
|-----------|-----|------|
| 持続時間 | 100ms | これより短いとアニメーションが見えない |
| 距離 | 0.3タイル | 視認性とのバランス |

### 注意点

- `Timer`の持続時間を短くしすぎると、アニメーションが視認できなくなる
- `normalize()` より `normalize_or_zero()` を使用（ゼロベクトル対策）
