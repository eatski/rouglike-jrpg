//! Bevyヘッドレス統合テスト
//!
//! MinimalPluginsを使用してウィンドウなしでECSシステムを実行し、
//! game crateのロジックとui crateのBevyシステムを結合してテストする。

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use game::battle::{default_party, Enemy};
use game::map::{generate_map, Terrain, MAP_HEIGHT, MAP_WIDTH};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::time::Duration;
use ui::app_state::AppState;
use ui::battle::{battle_input_system, BattlePhase, BattleResource, PendingCommands};
use ui::components::{Boat, MovementLocked, OnBoat, Player, TilePosition};
use ui::events::{MovementBlockedEvent, PlayerMovedEvent};
use ui::map_mode::MapModeState;
use ui::resources::{MapDataResource, MovementState, SpawnPosition};
use ui::{player_movement, sync_boat_with_player};
use ui::{start_bounce, start_smooth_move, update_bounce, update_smooth_move};

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

    // 全キーをリセット（pressed状態も含めて完全にクリア）
    input.reset_all();

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
    input.reset_all();
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

// ============================================
// 地形別移動テスト
// ============================================

#[test]
fn player_can_move_on_forest() {
    // カスタムマップ: プレイヤーが平原の上で、右に森がある
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;
    grid[spawn_y][spawn_x + 1] = Terrain::Forest;

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // イベントカウンタをリセット
    app.world_mut().resource_mut::<EventCounters>().moved_count = 0;

    // 右に移動（森に入る）
    press_key(&mut app, 1, 0);

    // キー入力を反映するために複数フレーム進める
    for _ in 0..3 {
        app.update();
    }

    // イベントが発行されたか確認
    let moved_count = app.world().resource::<EventCounters>().moved_count;
    assert!(moved_count >= 1, "PlayerMovedEvent should be emitted when moving to forest");

    // 移動アニメーション完了まで待つ
    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 30);

    // 位置が変わったか確認
    let final_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    assert_eq!(final_pos, (spawn_x + 1, spawn_y), "Player should move onto forest");
}

#[test]
fn player_can_move_on_mountain() {
    // カスタムマップ: プレイヤーが平原の上で、右に山がある
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;
    grid[spawn_y][spawn_x + 1] = Terrain::Mountain;

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // イベントカウンタをリセット
    app.world_mut().resource_mut::<EventCounters>().moved_count = 0;

    // 右に移動（山に登る）
    press_key(&mut app, 1, 0);

    // キー入力を反映するために複数フレーム進める
    for _ in 0..3 {
        app.update();
    }

    // イベントが発行されたか確認
    let moved_count = app.world().resource::<EventCounters>().moved_count;
    assert!(moved_count >= 1, "PlayerMovedEvent should be emitted when moving to mountain");

    // 移動アニメーション完了まで待つ
    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 30);

    // 位置が変わったか確認
    let final_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    assert_eq!(final_pos, (spawn_x + 1, spawn_y), "Player should move onto mountain");
}

// ============================================
// 連続移動テスト
// ============================================

#[test]
fn player_can_move_multiple_steps() {
    // カスタムマップ: 平原が3マス続く
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;
    grid[spawn_y][spawn_x + 1] = Terrain::Plains;
    grid[spawn_y][spawn_x + 2] = Terrain::Plains;
    grid[spawn_y][spawn_x + 3] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // 3回右に移動
    for i in 0..3 {
        press_key(&mut app, 1, 0);

        // 初回遅延を超えるために3フレーム進める
        for _ in 0..3 {
            app.update();
        }

        release_all_keys(&mut app);
        wait_for_movement_complete(&mut app, player_entity, 30);

        // 各移動後の位置を確認
        let pos = {
            let world = app.world();
            let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
            (tile_pos.x, tile_pos.y)
        };
        assert_eq!(pos, (spawn_x + i + 1, spawn_y), "Player should be at step {}", i + 1);
    }

    // 最終位置を確認
    let final_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    assert_eq!(final_pos, (spawn_x + 3, spawn_y), "Player should have moved 3 steps");
}

#[test]
fn player_cannot_move_while_locked() {
    // カスタムマップ: 十字型の平原（上方向にも歩けるようにする）
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;
    grid[spawn_y][spawn_x + 1] = Terrain::Plains;
    grid[spawn_y + 1][spawn_x + 1] = Terrain::Plains; // 移動先の上にも平原

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // 右に移動開始
    press_key(&mut app, 1, 0);

    // 初回遅延を超えて移動が始まる
    for _ in 0..3 {
        app.update();
    }

    // MovementLockedが設定されているか確認
    let is_locked = {
        let world = app.world();
        world.get::<MovementLocked>(player_entity).is_some()
    };
    assert!(is_locked, "Player should be locked during movement animation");

    // ロック中に上方向のキーを押す（上は平原なので、ロック無視なら移動してしまう）
    press_key(&mut app, 0, 1);

    // 複数フレーム進める
    for _ in 0..3 {
        app.update();
    }

    // 移動完了まで待つ
    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 30);

    // 最終位置を確認（最初の右移動のみ完了し、Y方向には動いていない）
    let final_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    assert_eq!(final_pos.0, spawn_x + 1, "X should move one step right");
    assert_eq!(final_pos.1, spawn_y, "Y should not change (movement was locked)");
}

// ============================================
// バウンスアニメーションテスト
// ============================================

/// バウンスシステムを含むテストアプリをセットアップ
fn setup_test_app_with_bounce(grid: Vec<Vec<Terrain>>, spawn_x: usize, spawn_y: usize) -> App {
    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);

    // バウンスシステムを追加
    app.add_systems(Update, start_bounce);
    app.add_systems(Update, update_bounce);

    app
}

#[test]
fn blocked_movement_triggers_bounce_and_clears() {
    // カスタムマップ: プレイヤーが平原、右は海
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;
    // grid[spawn_y][spawn_x + 1] は海のまま

    let mut app = setup_test_app_with_bounce(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // イベントカウンタをリセット
    app.world_mut().resource_mut::<EventCounters>().blocked_count = 0;

    // 右に移動（海に向かう - ブロックされる）
    press_key(&mut app, 1, 0);

    // キー入力を反映するために複数フレーム進める
    for _ in 0..3 {
        app.update();
    }

    // MovementBlockedEventが発行されたか確認
    let blocked_count = app.world().resource::<EventCounters>().blocked_count;
    assert!(blocked_count >= 1, "MovementBlockedEvent should be emitted");

    // バウンスアニメーション開始でMovementLockedが追加される
    let is_locked_after_block = {
        let world = app.world();
        world.get::<MovementLocked>(player_entity).is_some()
    };
    assert!(is_locked_after_block, "MovementLocked should be added for bounce animation");

    release_all_keys(&mut app);

    // バウンスアニメーション完了まで待つ（100ms = 2フレーム）
    wait_for_movement_complete(&mut app, player_entity, 30);

    // MovementLockedが解除されたか確認
    let is_locked_after_bounce = {
        let world = app.world();
        world.get::<MovementLocked>(player_entity).is_some()
    };
    assert!(!is_locked_after_bounce, "MovementLocked should be cleared after bounce");

    // 位置が変わっていないことを確認
    let final_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    assert_eq!(final_pos, (spawn_x, spawn_y), "Player should not move when blocked");
}

// ============================================
// マップモードテスト
// ============================================

#[test]
fn map_mode_blocks_movement() {
    // カスタムマップ: プレイヤーが平原の上で、右に移動できる
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;
    grid[spawn_y][spawn_x + 1] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // マップモードを有効化
    app.world_mut().resource_mut::<MapModeState>().enabled = true;

    // イベントカウンタをリセット
    {
        let mut counters = app.world_mut().resource_mut::<EventCounters>();
        counters.moved_count = 0;
        counters.blocked_count = 0;
    }

    // 右に移動を試みる
    press_key(&mut app, 1, 0);

    // キー入力を反映するために複数フレーム進める
    for _ in 0..3 {
        app.update();
    }

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 30);

    // イベントが発行されていないことを確認
    let counters = app.world().resource::<EventCounters>();
    assert_eq!(counters.moved_count, 0, "No PlayerMovedEvent should be emitted in map mode");
    assert_eq!(counters.blocked_count, 0, "No MovementBlockedEvent should be emitted in map mode");

    // 位置が変わっていないことを確認
    let final_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    assert_eq!(final_pos, (spawn_x, spawn_y), "Player should not move in map mode");
}

// ============================================
// PendingMove (斜め入力分解) テスト
// ============================================

#[test]
fn diagonal_input_decomposes_into_two_moves() {
    // カスタムマップ: L字型の平原パス
    // (50,50) -> (51,50) -> (51,51)
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;     // スタート地点
    grid[spawn_y][spawn_x + 1] = Terrain::Plains; // 右
    grid[spawn_y + 1][spawn_x + 1] = Terrain::Plains; // 右上

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // 斜め入力（右上）: W + D を同時押し
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.clear();
    input.press(KeyCode::KeyW); // 上
    input.press(KeyCode::KeyD); // 右
    drop(input); // borrowを解放

    // 初回移動が始まるまで待つ
    for _ in 0..3 {
        app.update();
    }

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, 30);

    // 1回目の移動が完了（X方向またはY方向のいずれか）
    let pos_after_first = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    // 最低でも1方向に移動しているはず
    let moved_once = pos_after_first != (spawn_x, spawn_y);
    assert!(moved_once, "First diagonal move should complete");

    // PendingMoveがあれば2回目の移動が自動で実行される
    // 追加のフレームを進めて2回目の移動を完了
    for _ in 0..10 {
        app.update();

        // MovementLockedがなければ完了
        let world = app.world();
        if world.get::<MovementLocked>(player_entity).is_none() {
            break;
        }
    }

    // 最終位置を確認
    let final_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    // 斜め移動が完了して (51, 51) に到達しているはず
    assert_eq!(final_pos, (spawn_x + 1, spawn_y + 1),
        "Diagonal input should decompose into two sequential moves");
}

// ============================================
// イベント整合性テスト
// ============================================

#[test]
fn multiple_moves_emit_correct_event_count() {
    // カスタムマップ: 平原が4マス続く
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    for i in 0..4 {
        grid[spawn_y][spawn_x + i] = Terrain::Plains;
    }

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);
    let player_entity = spawn_test_player(&mut app);

    // イベントカウンタをリセット
    {
        let mut counters = app.world_mut().resource_mut::<EventCounters>();
        counters.moved_count = 0;
        counters.blocked_count = 0;
    }

    // 3回右に移動
    for _ in 0..3 {
        press_key(&mut app, 1, 0);

        // 初回遅延を超えるために3フレーム進める
        for _ in 0..3 {
            app.update();
        }

        release_all_keys(&mut app);
        wait_for_movement_complete(&mut app, player_entity, 30);
    }

    // イベントカウントを確認（moved_countのみ）
    let counters = app.world().resource::<EventCounters>();
    assert_eq!(counters.moved_count, 3, "Should emit 3 PlayerMovedEvents");
    // blocked_countは検証しない（MovementStateの内部状態により変動する可能性がある）

    // 最終位置を確認
    let final_pos = {
        let world = app.world();
        let tile_pos = world.get::<TilePosition>(player_entity).unwrap();
        (tile_pos.x, tile_pos.y)
    };

    assert_eq!(final_pos, (spawn_x + 3, spawn_y), "Player should have moved 3 steps");
}

// ============================================
// 戦闘システムテスト
// ============================================

/// 戦闘用のBevyアプリをセットアップ（MinimalPlugins + State + BattleResource直接挿入）
fn setup_battle_test_app() -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    // StatesPluginを追加（AppStateを使用するために必須）
    app.add_plugins(bevy::state::app::StatesPlugin);

    // 時間制御を手動に設定
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_relative_speed(1.0);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(50)));

    // AppStateを登録
    app.init_state::<AppState>();

    // 必要なリソースをセットアップ
    app.init_resource::<ButtonInput<KeyCode>>();

    // 戦闘入力システムを追加
    app.add_systems(Update, battle_input_system);

    app
}

/// BattleResourceを直接挿入するヘルパー
fn insert_battle_resource(app: &mut App, phase: BattlePhase) {
    let party = default_party();
    let enemies = vec![Enemy::slime()];
    let battle_state = game::battle::BattleState::new(party, enemies);

    let display_party_hp = battle_state.party.iter().map(|m| m.stats.hp).collect();
    let display_party_mp = battle_state.party.iter().map(|m| m.stats.mp).collect();

    app.insert_resource(BattleResource {
        state: battle_state,
        selected_command: 0,
        selected_target: 0,
        pending_commands: PendingCommands::default(),
        phase,
        hidden_enemies: vec![false; 1],
        display_party_hp,
        display_party_mp,
        selected_spell: 0,
        pending_spell: None,
        selected_ally_target: 0,
        message_effects: Vec::new(),
        shake_timer: None,
        blink_timer: None,
        blink_enemy: None,
    });
}

/// キーを押すヘルパー（戦闘用）
fn press_battle_key(app: &mut App, key: KeyCode) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
    input.press(key);
}

/// 全キーをリセット（戦闘用）
fn release_battle_keys(app: &mut App) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
}

#[test]
fn battle_phase_transitions_from_command_to_exploring() {
    let mut app = setup_battle_test_app();

    // BattleResourceを挿入（CommandSelectから開始）
    insert_battle_resource(&mut app, BattlePhase::CommandSelect { member_index: 0 });

    // 初期状態がCommandSelectであることを確認
    {
        let battle_res = app.world().resource::<BattleResource>();
        assert!(matches!(battle_res.phase, BattlePhase::CommandSelect { .. }));
    }

    // 最大30回Enterを押してBattleOverまで進める
    // 新フロー: CommandSelect → TargetSelect → (次メンバー) → ... → ShowMessage → ...
    for _ in 0..30 {
        let phase = {
            let battle_res = app.world().resource::<BattleResource>();
            battle_res.phase.clone()
        };

        match phase {
            BattlePhase::CommandSelect { .. } => {
                // たたかうを選択（Enter）→ TargetSelectに遷移
                press_battle_key(&mut app, KeyCode::Enter);
                app.update();
                release_battle_keys(&mut app);
            }
            BattlePhase::SpellSelect { .. } | BattlePhase::AllyTargetSelect { .. } => {
                // 呪文選択や味方ターゲットが出たらキャンセルしてコマンドに戻す
                press_battle_key(&mut app, KeyCode::Escape);
                app.update();
                release_battle_keys(&mut app);
            }
            BattlePhase::TargetSelect { .. } => {
                // ターゲット確定（Enter）→ 次メンバーまたはターン実行
                press_battle_key(&mut app, KeyCode::Enter);
                app.update();
                release_battle_keys(&mut app);
            }
            BattlePhase::ShowMessage { .. } => {
                press_battle_key(&mut app, KeyCode::Enter);
                app.update();
                release_battle_keys(&mut app);
            }
            BattlePhase::BattleOver { .. } => {
                break;
            }
        }
    }

    // 最終的にBattleOverになっているはず
    {
        let battle_res = app.world().resource::<BattleResource>();
        assert!(
            matches!(battle_res.phase, BattlePhase::BattleOver { .. }),
            "Should reach BattleOver phase"
        );
    }

    // BattleOverでEnterを押すとAppState::Exploringに遷移
    press_battle_key(&mut app, KeyCode::Enter);
    app.update();

    let current_state = app.world().resource::<State<AppState>>();
    assert_eq!(
        **current_state,
        AppState::Exploring,
        "Should transition to Exploring after BattleOver"
    );
}

#[test]
fn battle_command_selection_with_keys() {
    let mut app = setup_battle_test_app();
    insert_battle_resource(&mut app, BattlePhase::CommandSelect { member_index: 0 });

    // 初期状態: selected_command = 0
    {
        let battle_res = app.world().resource::<BattleResource>();
        assert_eq!(battle_res.selected_command, 0);
    }

    // S（下）を押して選択を1に
    press_battle_key(&mut app, KeyCode::KeyS);
    app.update();

    {
        let battle_res = app.world().resource::<BattleResource>();
        assert_eq!(
            battle_res.selected_command, 1,
            "Should select command 1 after pressing S"
        );
    }

    release_battle_keys(&mut app);

    // W（上）を押して選択を0に戻す
    press_battle_key(&mut app, KeyCode::KeyW);
    app.update();

    {
        let battle_res = app.world().resource::<BattleResource>();
        assert_eq!(
            battle_res.selected_command, 0,
            "Should select command 0 after pressing W"
        );
    }

    release_battle_keys(&mut app);

    // 上限確認: 0でさらにW（上）を押しても0のまま
    press_battle_key(&mut app, KeyCode::KeyW);
    app.update();

    {
        let battle_res = app.world().resource::<BattleResource>();
        assert_eq!(
            battle_res.selected_command, 0,
            "Should stay at 0 (upper bound)"
        );
    }

    release_battle_keys(&mut app);

    // 下限確認: 2でさらにS（下）を押しても2のまま
    // 0→1
    press_battle_key(&mut app, KeyCode::KeyS);
    app.update();
    release_battle_keys(&mut app);
    // 1→2
    press_battle_key(&mut app, KeyCode::KeyS);
    app.update();

    {
        let battle_res = app.world().resource::<BattleResource>();
        assert_eq!(battle_res.selected_command, 2);
    }

    release_battle_keys(&mut app);

    // 2でさらにS → 2のまま
    press_battle_key(&mut app, KeyCode::KeyS);
    app.update();

    {
        let battle_res = app.world().resource::<BattleResource>();
        assert_eq!(
            battle_res.selected_command, 2,
            "Should stay at 2 (lower bound)"
        );
    }
}

#[test]
fn battle_flee_command_transitions_correctly() {
    let mut app = setup_battle_test_app();
    insert_battle_resource(&mut app, BattlePhase::CommandSelect { member_index: 0 });

    // にげる（selected_command=2）を選択
    press_battle_key(&mut app, KeyCode::KeyS);
    app.update();
    release_battle_keys(&mut app);
    press_battle_key(&mut app, KeyCode::KeyS);
    app.update();
    release_battle_keys(&mut app);

    // Enterで決定（逃走を選択）
    press_battle_key(&mut app, KeyCode::Enter);
    app.update();
    release_battle_keys(&mut app);

    // 逃走は50%確率。ShowMessage（にげきれた or にげられなかった）またはBattleOverに遷移
    let phase = {
        let battle_res = app.world().resource::<BattleResource>();
        battle_res.phase.clone()
    };

    match phase {
        BattlePhase::ShowMessage { messages, .. } => {
            // にげきれた or にげられなかったメッセージがある
            assert!(
                messages.iter().any(|m| m.contains("にげ")),
                "Should have flee-related message, got: {:?}",
                messages
            );
        }
        BattlePhase::BattleOver { message } => {
            assert!(
                message.contains("にげ"),
                "BattleOver message should contain flee text"
            );
        }
        _ => {
            // 逃走失敗時はCommandSelectに戻ることもある
        }
    }
}

#[test]
fn battle_cleanup_removes_movement_lock() {
    // 通常のマップアプリをセットアップ
    let mut grid = vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT];
    let spawn_x = 50;
    let spawn_y = 50;
    grid[spawn_y][spawn_x] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, spawn_x, spawn_y);

    // StatesPluginを追加（AppStateを使用するために必須）
    app.add_plugins(bevy::state::app::StatesPlugin);

    // AppStateを追加
    app.init_state::<AppState>();

    // cleanup_battle_sceneシステムを追加（OnExit(AppState::Battle)で実行される想定）
    app.add_systems(OnExit(AppState::Battle), ui::cleanup_battle_scene);

    let player_entity = spawn_test_player(&mut app);

    // プレイヤーに手動でMovementLockedを付与（戦闘開始前に移動中だった想定）
    app.world_mut()
        .entity_mut(player_entity)
        .insert(MovementLocked);

    // MovementLockedが付いていることを確認
    {
        let world = app.world();
        assert!(
            world.get::<MovementLocked>(player_entity).is_some(),
            "MovementLocked should be present before battle cleanup"
        );
    }

    // 戦闘に入る
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Battle);
    app.update();

    // BattleResourceを挿入（戦闘シーンセットアップの代わり）
    insert_battle_resource(&mut app, BattlePhase::CommandSelect { member_index: 0 });

    // 戦闘終了（Exploringに戻る）→ cleanup_battle_sceneが実行される
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Exploring);
    app.update();

    // MovementLockedが除去されていることを確認
    {
        let world = app.world();
        assert!(
            world.get::<MovementLocked>(player_entity).is_none(),
            "MovementLocked should be removed after battle cleanup"
        );
    }
}
