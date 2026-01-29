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
| 島検出（Flood Fill） | 船スプライトの生成・配置 |
| 船移動ロジック | 乗り降りの視覚演出 |
| 当たり判定 | 衝突エフェクト |
| ダメージ計算 | ダメージ表示 |
| ターン管理 | ターン遷移演出 |

**判断のコツ**: 「画面がなくても意味があるか？」→ Yes なら game/

### 実例: 船移動システム

```rust
// game/movement/boat.rs - 船移動の判定（純粋関数）
pub fn try_move_on_boat(
    current_x: usize, current_y: usize,
    dx: i32, dy: i32,
    grid: &[Vec<Terrain>],
) -> MoveResult {
    let new_x = ((current_x as i32 + dx).rem_euclid(MAP_WIDTH as i32)) as usize;
    let new_y = ((current_y as i32 + dy).rem_euclid(MAP_HEIGHT as i32)) as usize;

    if grid[new_y][new_x] == Terrain::Sea {
        MoveResult::Moved { new_x, new_y }
    } else {
        MoveResult::Blocked
    }
}

// ui/player_input.rs - Bevy統合
fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut TilePosition, Option<&OnBoat>)>,
    grid: Res<TerrainGrid>,
) {
    let is_on_boat = on_boat.is_some();
    let move_result = if is_on_boat {
        game::movement::boat::try_move_on_boat(pos.x, pos.y, dx, dy, &grid.0)
    } else {
        game::movement::player::try_move(pos.x, pos.y, dx, dy, &grid.0)
    };
    // Bevy Componentの更新処理...
}
```

**ポイント**:
- 船移動の判定ロジックは`game/`（Bevy非依存）
- 状態管理（`OnBoat`コンポーネント）は`ui/`（Bevy依存）
- 自動乗り降り判定は`game/`、演出は`ui/`に分離

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

## Flood Fillパターン（マップ解析）

島検出など、連結領域の探索に使用。

```rust
// game/map/islands.rs - Flood Fillで島を検出
fn detect_islands(grid: &[Vec<Terrain>]) -> Vec<Vec<(usize, usize)>> {
    let mut visited = vec![vec![false; MAP_WIDTH]; MAP_HEIGHT];
    let mut islands = Vec::new();

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            if !visited[y][x] && grid[y][x] != Terrain::Sea {
                let island = flood_fill(x, y, grid, &mut visited);
                islands.push(island);
            }
        }
    }
    islands
}

fn flood_fill(
    start_x: usize,
    start_y: usize,
    grid: &[Vec<Terrain>],
    visited: &mut [Vec<bool>],
) -> Vec<(usize, usize)> {
    let mut island = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back((start_x, start_y));
    visited[start_y][start_x] = true;

    while let Some((x, y)) = queue.pop_front() {
        island.push((x, y));
        // 4近傍を探索（ラップアラウンド対応）
        for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            let nx = (x as i32 + dx).rem_euclid(MAP_WIDTH as i32) as usize;
            let ny = (y as i32 + dy).rem_euclid(MAP_HEIGHT as i32) as usize;

            if !visited[ny][nx] && grid[ny][nx] != Terrain::Sea {
                visited[ny][nx] = true;
                queue.push_back((nx, ny));
            }
        }
    }
    island
}
```

### 応用例

- **船スポーン位置の計算**: 各島の外郭（海に隣接する陸地）を検出し、その隣の海タイルに船を配置
- **バイオーム検出**: 同じ地形が連結している領域を抽出
- **到達可能判定**: プレイヤーが特定の場所に行けるか判定

## 探索システム（Fog of War）パターン

プレイヤーが訪れた場所を記録し、視界外のエリアを暗くする。

### アーキテクチャ

```
crates/game/exploration/        crates/ui/map_mode.rs
├── TileVisibility enum         ├── ExplorationData (Resource)
│   ├── Unexplored              ├── OriginalColor (Component)
│   ├── Explored                ├── init_exploration_system
│   └── Visible                 ├── update_exploration_system
├── ExplorationMap              ├── apply_map_mode_fog_system
└── calculate_visible_tiles()   └── restore_tile_colors_system

Bevy非依存 ←───────────────────→ Bevy依存
（視界判定ロジック）               （色変更・UI制御）
```

### 実装例

```rust
// game/exploration/visibility.rs - 視界判定（純粋関数）
pub enum TileVisibility {
    Unexplored,  // 未探索（黒）
    Explored,    // 探索済み（暗め）
    Visible,     // 視界内（通常）
}

pub fn calculate_visible_tiles(
    center_x: usize,
    center_y: usize,
    radius: usize,
    map_width: usize,
    map_height: usize,
) -> Vec<(usize, usize)> {
    // トーラスマップ対応の視界計算
    // チェビシェフ距離で判定（正方形視界）
}

// game/exploration/map.rs - 探索状態管理
pub struct ExplorationMap {
    tiles: Vec<Vec<TileVisibility>>,
}

impl ExplorationMap {
    pub fn update_visibility(&mut self, x: usize, y: usize, radius: usize) {
        let visible = calculate_visible_tiles(x, y, radius, width, height);
        // Unexplored → Explored → Visible への状態遷移
    }
}

// ui/map_mode.rs - Bevy統合
#[derive(Resource)]
pub struct ExplorationData {
    pub map: ExplorationMap,
}

fn update_exploration_system(
    mut exploration_data: ResMut<ExplorationData>,
    player_query: Query<&TilePosition, With<Player>>,
    mut moved_events: MessageReader<PlayerMovedEvent>,
) {
    for _event in moved_events.read() {
        if let Ok(tile_pos) = player_query.single() {
            exploration_data.map.update_visibility(tile_pos.x, tile_pos.y, VIEW_RADIUS);
        }
    }
}

fn apply_map_mode_fog_system(
    exploration_data: Res<ExplorationData>,
    mut tile_query: Query<(&TilePosition, &mut Sprite, Option<&OriginalColor>), With<MapTile>>,
) {
    // TileVisibilityに応じて色を変更
    // Visible: 元の色、Explored: 暗め（0.5倍）、Unexplored: 黒
}
```

### マップモード（全体表示）との連携

```rust
// ui/map_mode.rs - Mキーでマップモードをトグル
#[derive(Resource, Default)]
pub struct MapModeState {
    pub enabled: bool,
}

fn toggle_map_mode_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut map_mode_state: ResMut<MapModeState>,
    mut camera_query: Query<&mut Projection, With<Camera2d>>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        map_mode_state.enabled = !map_mode_state.enabled;
        // カメラズームを切り替え（VISIBLE_SIZE ↔ MAP_PIXEL_WIDTH）
    }
}

// ui/player_input.rs - マップモード中は移動無効化
fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    map_mode_state: Res<MapModeState>,
) {
    if map_mode_state.enabled {
        return; // マップモード中は入力を無視
    }
}
```

### ポイント

- **視界判定ロジックはgame/**: トーラス対応、チェビシェフ距離計算など
- **色変更・カメラ制御はui/**: Sprite色、Projection、入力無効化など
- **状態遷移**: Unexplored → Explored（一度訪れる）、Explored → Visible（視界内）
- **元の色を保存**: OriginalColorコンポーネントで復元可能に

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
