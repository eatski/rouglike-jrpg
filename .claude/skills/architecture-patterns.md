# アーキテクチャパターン集

## ゲームロジックとUIの分離

### 基本原則

```
game/（ロジック層）        ui/（プレゼンテーション層）
├── 何が起きるか           ├── どう見せるか
├── ルールの判定           ├── アニメーション
├── 状態の管理             ├── 描画
└── タイル座標             └── ワールド座標

        ↓ Message ↓
    疎結合な一方向通信
```

### なぜ分離するか

1. **テスト容易性** - ゲームロジックをUI無しでテスト可能
2. **変更の局所化** - 見た目の変更がロジックに影響しない
3. **再利用性** - 同じロジックに異なるUIを適用可能
4. **責務の明確化** - 「何をするか」と「どう見せるか」を混同しない

### 分離の判断基準

| game/ に置くもの | ui/ に置くもの |
|---|---|
| タイル座標 (usize) | ワールド座標 (f32) |
| 移動可否の判定 | 移動アニメーション |
| 当たり判定 | 衝突エフェクト |
| ダメージ計算 | ダメージ表示 |
| ターン管理 | ターン遷移演出 |

**判断のコツ**: 「画面がなくても意味があるか？」→ Yes なら game/

## Message による層間通信

### パターン: イベント駆動

```rust
// game/movement/events.rs
#[derive(Message)]
pub struct MovementBlockedEvent {
    pub entity: Entity,
    pub direction: (i32, i32),
}

// game/movement/player.rs
fn player_movement(
    mut blocked_events: MessageWriter<MovementBlockedEvent>,
) {
    if cannot_move {
        blocked_events.write(MovementBlockedEvent { entity, direction });
    }
}

// ui/bounce.rs
fn start_bounce(
    mut commands: Commands,
    mut events: MessageReader<MovementBlockedEvent>,
) {
    for event in events.read() {
        commands.entity(event.entity).insert(BounceAnimation { ... });
    }
}
```

### パターン: マーカーコンポーネント

UIがゲームロジックに影響を与える場合（例: アニメーション中は入力無効）

```rust
// game/ で定義（ロジックが参照するため）
#[derive(Component)]
pub struct MovementLocked;

// ui/ で追加・削除
commands.entity(entity).insert(MovementLocked);
commands.entity(entity).remove::<MovementLocked>();

// game/ で判定
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
   - ルール → `game/` に実装
   - 見た目 → `ui/` に実装
   - 両方 → ロジックを`game/`、表示を`ui/`に分離

2. **層間通信が必要か確認**
   - game → ui: Message を定義
   - ui → game: マーカーコンポーネント or 別 Message

3. **定数の配置を決定**
   - ルールに関わる数値 → `game/` 内
   - 見た目に関わる数値 → `ui/constants.rs`

4. **依存方向を確認**
   - `ui/` は `game/` に依存してOK
   - `game/` は `ui/` に依存してはいけない
