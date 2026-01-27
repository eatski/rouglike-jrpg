# Bevy 0.18 パターン集

## インポート

```rust
use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;
use bevy::window::{Window, WindowResolution};
```

## Message API（Bevy 0.17以降）

Bevy 0.17で`Event`/`EventWriter`/`EventReader`は`Message`/`MessageWriter`/`MessageReader`に名称変更された。

### メッセージ定義

```rust
#[derive(Message)]
pub struct PlayerMovedEvent {
    pub entity: Entity,
    pub direction: (i32, i32),
}
```

### メッセージ送信

```rust
fn movement(mut events: MessageWriter<PlayerMovedEvent>) {
    events.write(PlayerMovedEvent {
        entity,
        direction: (dx, dy)
    });
}
```

### メッセージ受信

```rust
fn handle_movement(mut events: MessageReader<PlayerMovedEvent>) {
    for event in events.read() {
        // event.entity, event.direction を使用
    }
}
```

### Appへの登録

```rust
App::new()
    .add_message::<PlayerMovedEvent>()
    .add_message::<MovementBlockedEvent>()
```

## カメラ設定

### 固定表示範囲のOrthographicカメラ

```rust
commands.spawn((
    Camera2d,
    Projection::from(OrthographicProjection {
        scaling_mode: ScalingMode::Fixed {
            width: 28.0,  // ワールド単位
            height: 28.0,
        },
        ..OrthographicProjection::default_2d()
    }),
));
```

**注意**: Bevy 0.18では`Camera2d`と`OrthographicProjection`を直接タプルにできない。`Projection::from()`でラップする必要がある。

### ウィンドウサイズ固定・リサイズ無効

```rust
App::new()
    .add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())  // ピクセルアート用
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(448, 448),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    )
```

## Query API

Bevy 0.18では`get_single()`/`get_single_mut()`が`single()`/`single_mut()`に変更され、`Result`を返す。

```rust
// 単一エンティティ取得
let Ok(transform) = query.single() else { return; };
let Ok(mut transform) = query.single_mut() else { return; };
```

## システムの実行順序

```rust
// chain()で順序を保証
.add_systems(Startup, (system_a, system_b, system_c).chain())
.add_systems(Update, (movement, camera_follow).chain())
```

## キーボード入力

```rust
fn movement(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::KeyW) { /* ... */ }
    if keyboard.just_pressed(KeyCode::KeyS) { /* ... */ }
    if keyboard.just_pressed(KeyCode::KeyA) { /* ... */ }
    if keyboard.just_pressed(KeyCode::KeyD) { /* ... */ }
}
```

## トーラスラップ（端で反対側に出る）

```rust
let new_x = ((x as i32 + dx).rem_euclid(MAP_WIDTH as i32)) as usize;
let new_y = ((y as i32 + dy).rem_euclid(MAP_HEIGHT as i32)) as usize;
```

## カメラ追従

```rust
fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player) = player_query.single() else { return; };
    let Ok(mut camera) = camera_query.single_mut() else { return; };

    camera.translation.x = player.translation.x;
    camera.translation.y = player.translation.y;
}
```

## リソースの受け渡し（Startup間）

```rust
#[derive(Resource)]
struct SpawnPosition { x: usize, y: usize }

// システムAでリソース登録
fn system_a(mut commands: Commands) {
    commands.insert_resource(SpawnPosition { x: 10, y: 20 });
}

// システムBでリソース使用（chain()で順序保証）
fn system_b(spawn_pos: Res<SpawnPosition>) {
    // spawn_pos.x, spawn_pos.y を使用
}
```
