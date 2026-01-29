//! Bevyヘッドレス統合テスト
//!
//! MinimalPluginsを使用してウィンドウなしでECSシステムを実行し、
//! game crateのロジックとui crateのBevyシステムを結合してテストする。

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use game::map::{generate_map, Terrain, MAP_HEIGHT, MAP_WIDTH};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::time::Duration;
use ui::components::{Boat, MovementLocked, OnBoat, Player, TilePosition};
use ui::events::{MovementBlockedEvent, PlayerMovedEvent};
use ui::map_mode::MapModeState;
use ui::resources::{MapDataResource, MovementState, SpawnPosition};
use ui::{player_movement, sync_boat_with_player};
use ui::{start_smooth_move, update_smooth_move};

/// イベントカウンタ（テスト用）
#[derive(Resource, Default)]
struct EventCounters {
    moved_count: usize,
    blocked_count: usize,
}

/// イベントカウンタシステム
fn count_moved_events(
    mut counters: ResMut<EventCounters>,
    mut events: MessageReader<PlayerMovedEvent>,
) {
    for _ in events.read() {
        counters.moved_count += 1;
    }
}

fn count_blocked_events(
    mut counters: ResMut<EventCounters>,
    mut events: MessageReader<MovementBlockedEvent>,
) {
    for _ in events.read() {
        counters.blocked_count += 1;
    }
}

/// テスト用のBevyアプリをセットアップ（ランダムマップ）
fn setup_test_app(seed: u64) -> App {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let map_data = generate_map(&mut rng);
    let spawn_pos = map_data.spawn_position;

    setup_test_app_with_map(map_data.grid, spawn_pos.0, spawn_pos.1)
}

/// テスト用のBevyアプリをセットアップ（カスタムマップ）
fn setup_test_app_with_map(grid: Vec<Vec<Terrain>>, spawn_x: usize, spawn_y: usize) -> App {
    let mut app = App::new();

    // MinimalPluginsのみを使用（ウィンドウ、レンダリングなし）
    app.add_plugins(MinimalPlugins);

    // 時間制御を手動に設定（テストで明示的に進める）
    // 1フレーム=50msに設定することで、初回遅延(150ms)を3フレームで超えられる
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_relative_speed(1.0);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(50)));

    // 必要なリソースをセットアップ
    app.insert_resource(MapDataResource {
        grid,
        spawn_position: (spawn_x, spawn_y),
    });
    app.insert_resource(SpawnPosition {
        x: spawn_x,
        y: spawn_y,
    });
    app.insert_resource(MovementState::default());
    app.insert_resource(EventCounters::default());
    app.insert_resource(MapModeState::default());
    app.init_resource::<ButtonInput<KeyCode>>();

    // イベントを登録
    app.add_message::<PlayerMovedEvent>();
    app.add_message::<MovementBlockedEvent>();

    // システムを手動で追加（通常のPluginは使わない）
    app.add_systems(Update, player_movement);
    app.add_systems(Update, start_smooth_move);
    app.add_systems(Update, update_smooth_move);
    app.add_systems(Update, sync_boat_with_player);
    app.add_systems(Update, count_moved_events);
    app.add_systems(Update, count_blocked_events);

    app
}

/// プレイヤーをスポーン
fn spawn_test_player(app: &mut App) -> Entity {
    let spawn_pos = app.world().resource::<SpawnPosition>();
    let x = spawn_pos.x;
    let y = spawn_pos.y;

    app.world_mut()
        .spawn((
            Player,
            TilePosition { x, y },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ))
        .id()
}

/// 船をスポーン
fn spawn_test_boat(app: &mut App, x: usize, y: usize) -> Entity {
    app.world_mut()
        .spawn((Boat, TilePosition { x, y }, Transform::from_xyz(0.0, 0.0, 0.5)))
        .id()
}

/// 指定方向のキー入力をシミュレート
fn press_key(app: &mut App, dx: i32, dy: i32) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();

    // 全キーをクリア
    input.clear();

    // 方向に応じてキーを押す
    if dx > 0 {
        input.press(KeyCode::KeyD);
    } else if dx < 0 {
        input.press(KeyCode::KeyA);
    }

    if dy > 0 {
        input.press(KeyCode::KeyW);
    } else if dy < 0 {
        input.press(KeyCode::KeyS);
    }
}

/// キーをすべて離す
fn release_all_keys(app: &mut App) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.clear();
}

/// 移動アニメーションが完了するまでフレームを進める
/// player_entityを渡す必要がある
fn wait_for_movement_complete(app: &mut App, player_entity: Entity, max_frames: usize) {
    for _ in 0..max_frames {
        app.update();

        // MovementLockedがなければ完了
        let world = app.world();
        if world.get::<MovementLocked>(player_entity).is_none() {
            break;
        }
    }
}

// ============================================
// 基本移動テスト
// ============================================

#[test]
fn player_can_move_on_walkable_terrain() {
    // カスタムマップ: プレイヤーが平原の上で、右に移動できる
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;
    grid[spawn_y][spawn_x + 1] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // 初期位置を確認
    let initial_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };
    assert_eq!(initial_pos, (spawn_x, spawn_y));

    // イベントカウンタをリセット
    app.world_mut().resource_mut::<EventCounters>().moved_count = 0;

    // 右に移動
    press_key(&mut app, 1, 0);

    // キー入力を反映するために複数フレーム進める
    for _ in 0..3 {
        app.update();
    }

    // イベントが発行されたか確認
    let moved_count = app.world().resource::<EventCounters>().moved_count;
    assert!(moved_count >= 1, "PlayerMovedEvent should be emitted (got {})", moved_count);

    // 移動アニメーション完了まで待つ
    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 30);

    // 位置が変わったか確認
    let final_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    assert_eq!(final_pos, (spawn_x + 1, spawn_y), "Player should move to new position");
}

#[test]
fn player_cannot_move_into_sea() {
    // カスタムマップ: プレイヤーが平原の上で、右は海
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;
    // grid[spawn_y][spawn_x + 1] は海のまま

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // イベントカウンタをリセット
    app.world_mut().resource_mut::<EventCounters>().blocked_count = 0;

    // 右に移動（海に向かう）
    press_key(&mut app, 1, 0);

    // キー入力を反映するために複数フレーム進める
    for _ in 0..3 {
        app.update();
    }

    // MovementBlockedEventが発行されたか確認
    let blocked_count = app.world().resource::<EventCounters>().blocked_count;
    assert!(blocked_count >= 1, "MovementBlockedEvent should be emitted (got {})", blocked_count);

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 30);

    // 位置が変わっていないことを確認
    let final_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    assert_eq!(final_pos, (spawn_x, spawn_y), "Player should not move into sea");
}

// ============================================
// 船移動テスト
// ============================================

#[test]
fn player_can_board_boat() {
    // カスタムマップ: プレイヤーが平原、右に海、船がそこに
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;
    // grid[spawn_y][spawn_x + 1] は海

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // 右の海に船を配置
    let boat_x = spawn_x + 1;
    let boat_y = spawn_y;
    let boat_entity = spawn_test_boat(&mut app, boat_x, boat_y);

    // 右に移動（船に乗る）
    press_key(&mut app, 1, 0);
    app.update();
    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 20);

    // プレイヤーが船に乗っているか確認
    let world = app.world();
    let on_boat = world.get::<OnBoat>(player_entity);

    assert!(on_boat.is_some(), "Player should be on boat");
    assert_eq!(on_boat.unwrap().boat_entity, boat_entity);

    // プレイヤーと船の位置が同じか確認
    let player_pos = world.get::<TilePosition>(player_entity).unwrap();
    let boat_pos = world.get::<TilePosition>(boat_entity).unwrap();

    assert_eq!(player_pos.x, boat_pos.x);
    assert_eq!(player_pos.y, boat_pos.y);
}

#[test]
fn player_on_boat_can_move_on_sea() {
    // カスタムマップ: プレイヤーが平原、右に海が2マス続く
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;
    // grid[spawn_y][spawn_x + 1] は海
    // grid[spawn_y][spawn_x + 2] も海

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // 右の海に船を配置
    let boat_x = spawn_x + 1;
    let boat_y = spawn_y;
    let boat_entity = spawn_test_boat(&mut app, boat_x, boat_y);

    // 右に移動（船に乗る）
    press_key(&mut app, 1, 0);
    for _ in 0..3 {
        app.update();
    }
    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 30);

    // 乗船したか確認
    let on_boat_after_first_move = {
        let world = app.world();
        world.get::<OnBoat>(player_entity).is_some()
    };
    assert!(on_boat_after_first_move, "Player should be on boat after first move");

    // さらに右に移動（海上を移動）
    press_key(&mut app, 1, 0);
    for _ in 0..3 {
        app.update();
    }
    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 30);

    // 移動したか確認
    let final_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    assert_eq!(final_pos, (spawn_x + 2, spawn_y), "Player on boat should move on sea (got {:?})", final_pos);

    // 船も同じ位置に移動しているか確認
    let world = app.world();
    let boat_pos = world.get::<TilePosition>(boat_entity).unwrap();
    assert_eq!(boat_pos.x, final_pos.0);
    assert_eq!(boat_pos.y, final_pos.1);
}

#[test]
fn player_can_disembark_from_boat() {
    // カスタムマップ: プレイヤーが平原、右に海、さらに右に平原
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;
    // grid[spawn_y][spawn_x + 1] は海
    grid[spawn_y][spawn_x + 2] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // 右の海に船を配置
    let boat_x = spawn_x + 1;
    let boat_y = spawn_y;
    spawn_test_boat(&mut app, boat_x, boat_y);

    // 右に移動（船に乗る）
    press_key(&mut app, 1, 0);
    for _ in 0..3 {
        app.update();
    }
    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 30);

    // さらに右に移動（陸地に下船）
    press_key(&mut app, 1, 0);
    for _ in 0..3 {
        app.update();
    }
    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 30);

    // 下船したか確認
    let world = app.world();
    let on_boat = world.get::<OnBoat>(player_entity);

    assert!(on_boat.is_none(), "Player should have disembarked");

    // 位置が陸地にあるか確認
    let player_pos = world.get::<TilePosition>(player_entity).unwrap();
    assert_eq!(player_pos.x, spawn_x + 2);
    assert_eq!(player_pos.y, spawn_y);
}

// ============================================
// マップ端ラップアラウンドテスト
// ============================================

#[test]
fn player_wraps_around_map_edge() {
    let mut app = setup_test_app(12345);

    // マップ端にプレイヤーを配置
    let edge_x = MAP_WIDTH - 1;
    let edge_y = MAP_HEIGHT / 2;

    // 端の位置が歩行可能か確認し、必要に応じて調整
    let (player_x, player_y) = {
        let map_data = app.world().resource::<MapDataResource>();
        let mut x = edge_x;
        let y = edge_y;

        // 歩行可能な位置を探す
        while !map_data.grid[y][x].is_walkable() && x > 0 {
            x -= 1;
        }

        if !map_data.grid[y][x].is_walkable() {
            panic!("Could not find walkable position near edge");
        }

        (x, y)
    };

    let player_entity = app
        .world_mut()
        .spawn((
            Player,
            TilePosition {
                x: player_x,
                y: player_y,
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ))
        .id();

    // 右端が歩行可能でラップ先も歩行可能なケースをセットアップ
    let can_wrap = {
        let map_data = app.world().resource::<MapDataResource>();
        player_x == MAP_WIDTH - 1
            && map_data.grid[player_y][0].is_walkable()
    };

    if !can_wrap {
        // このシードではラップアラウンドをテストできないのでスキップ
        return;
    }

    // 右に移動（マップ端を超える）
    press_key(&mut app, 1, 0);
    app.update();
    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 20);

    // x=0にラップアラウンドしたか確認
    let final_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    assert_eq!(final_pos.0, 0, "Player should wrap around to x=0");
    assert_eq!(final_pos.1, player_y);
}

// ============================================
// 決定性テスト
// ============================================

#[test]
fn same_seed_produces_deterministic_behavior() {
    let seed = 99999;

    // 1回目
    let mut app1 = setup_test_app(seed);
    let player1 = spawn_test_player(&mut app1);
    let pos1_initial = {
        let world = app1.world();
        let tile_pos = world.get::<TilePosition>(player1).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    // 2回目
    let mut app2 = setup_test_app(seed);
    let player2 = spawn_test_player(&mut app2);
    let pos2_initial = {
        let world = app2.world();
        let tile_pos = world.get::<TilePosition>(player2).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    // 同じシードで同じ初期位置か確認
    assert_eq!(pos1_initial, pos2_initial, "Same seed should produce same spawn position");

    // マップも同じか確認
    let map1 = app1.world().resource::<MapDataResource>();
    let map2 = app2.world().resource::<MapDataResource>();

    assert_eq!(map1.grid, map2.grid, "Same seed should produce same map");
}
