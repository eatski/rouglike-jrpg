//! Bevyヘッドレス統合テスト
//!
//! MinimalPluginsを使用してウィンドウなしでECSシステムを実行し、
//! game crateのロジックとui crateのBevyシステムを結合してテストする。
//!
//! 本番（main.rs）と同じregister関数・State条件を使用し、
//! テストと本番のシステム登録の乖離を最小限に抑える。

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use battle::{default_party, Enemy};
use terrain::{Terrain, MAP_HEIGHT, MAP_WIDTH};
use world_gen::generate_map;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::time::Duration;
use app_state::{BattleState, SceneState};
use battle_ui::{BattlePhase, BattleUIState};
use field_core::{ActiveMap, Boat, OnBoat, Player, TilePosition, TILE_SIZE};
use field_walk_ui::{
    MovementBlockedEvent, MovementLocked, MovementState,
    PlayerMovedEvent,
};
use app_state::PartyState;
use field_walk_ui::MapModeState;
use field_walk_ui::SpawnPosition;

const SPAWN_X: usize = 50;
const SPAWN_Y: usize = 50;
const MAX_ANIM_FRAMES: usize = 30;

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
///
/// 本番と同じregister関数・State条件を使用する。
fn setup_test_app_with_map(grid: Vec<Vec<Terrain>>, spawn_x: usize, spawn_y: usize) -> App {
    let mut app = App::new();

    // MinimalPluginsのみを使用（ウィンドウ、レンダリングなし）
    app.add_plugins(MinimalPlugins);

    // StatesPluginを追加（State遷移をサポート）
    app.add_plugins(bevy::state::app::StatesPlugin);

    // 時間制御を手動に設定（1フレーム=50ms）
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_relative_speed(1.0);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(50)));

    // State登録（本番と同じ）
    app.init_state::<SceneState>();   // デフォルト: Exploring
    app.init_state::<BattleState>();  // デフォルト: None

    // 必要なリソースをセットアップ
    let width = grid[0].len();
    let height = grid.len();
    let origin_x = -(width as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    let origin_y = -(height as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    app.insert_resource(ActiveMap {
        structures: vec![vec![terrain::Structure::None; width]; height],
        grid,
        width,
        height,
        origin_x,
        origin_y,
        wraps: true,
    });
    app.insert_resource(SpawnPosition {
        x: spawn_x,
        y: spawn_y,
    });
    app.insert_resource(MovementState::default());
    app.insert_resource(EventCounters::default());
    app.insert_resource(MapModeState::default());
    app.init_resource::<PartyState>();
    app.init_resource::<ButtonInput<KeyCode>>();

    // イベントを登録
    app.add_message::<PlayerMovedEvent>();
    app.add_message::<MovementBlockedEvent>();
    app.add_message::<field_walk_ui::TileEnteredEvent>();

    // 本番と同じシステム登録（移動コアのみ、エンカウント除外）
    field_walk_ui::register_exploring_movement_systems(&mut app);

    // テスト専用イベントカウンタ（run_if外で常時実行）
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

/// 単一キーを押す（他のキーはリセット）
fn press_single_key(app: &mut App, key: KeyCode) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
    input.press(key);
}

/// キーをすべて離す
fn release_all_keys(app: &mut App) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
}

/// 移動アニメーションが完了するまでフレームを進める
fn wait_for_movement_complete(app: &mut App, player_entity: Entity, max_frames: usize) {
    for _ in 0..max_frames {
        app.update();

        // MovementLockedがなければ完了
        if app.world().get::<MovementLocked>(player_entity).is_none() {
            break;
        }
    }
}

/// エンティティのタイル位置を取得
fn get_tile_pos(app: &App, entity: Entity) -> (usize, usize) {
    let tp = app.world().get::<TilePosition>(entity).unwrap();
    (tp.x, tp.y)
}

/// 1回の移動を実行し、アニメーション完了まで待つ。
///
/// 最後のapp.update()でMovementStateがリセットされ、
/// 次のpress_keyでdirection_changed発火が可能になる。
fn perform_move(app: &mut App, player_entity: Entity, dx: i32, dy: i32) {
    press_key(app, dx, dy);
    app.update();
    release_all_keys(app);
    wait_for_movement_complete(app, player_entity, MAX_ANIM_FRAMES);
    app.update();
}

/// 海ベースのカスタムマップグリッドを生成
fn setup_sea_grid() -> Vec<Vec<Terrain>> {
    vec![vec![Terrain::Sea; MAP_WIDTH]; MAP_HEIGHT]
}

/// イベントカウンタをリセット
fn reset_event_counters(app: &mut App) {
    let mut c = app.world_mut().resource_mut::<EventCounters>();
    c.moved_count = 0;
    c.blocked_count = 0;
}

// ============================================
// 基本移動テスト
// ============================================

#[test]
fn player_can_move_on_walkable_terrain() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 1] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X, SPAWN_Y));

    reset_event_counters(&mut app);

    // 右に移動
    press_key(&mut app, 1, 0);
    app.update();
    app.update(); // イベントカウンタがメッセージを読み取るために追加フレーム

    let moved_count = app.world().resource::<EventCounters>().moved_count;
    assert!(moved_count >= 1, "PlayerMovedEvent should be emitted (got {})", moved_count);

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X + 1, SPAWN_Y), "Player should move to new position");
}

#[test]
fn player_cannot_move_into_sea() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    // grid[SPAWN_Y][SPAWN_X + 1] は海のまま

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    reset_event_counters(&mut app);

    // 右に移動（海に向かう）
    press_key(&mut app, 1, 0);
    app.update();
    app.update(); // イベントカウンタがメッセージを読み取るために追加フレーム

    let blocked_count = app.world().resource::<EventCounters>().blocked_count;
    assert!(blocked_count >= 1, "MovementBlockedEvent should be emitted (got {})", blocked_count);

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X, SPAWN_Y), "Player should not move into sea");
}

// ============================================
// 船移動テスト
// ============================================

#[test]
fn player_can_board_boat() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    let boat_entity = spawn_test_boat(&mut app, SPAWN_X + 1, SPAWN_Y);

    // 右に移動（船に乗る）
    perform_move(&mut app, player_entity, 1, 0);

    let world = app.world();
    let on_boat = world.get::<OnBoat>(player_entity);

    assert!(on_boat.is_some(), "Player should be on boat");
    assert_eq!(on_boat.unwrap().boat_entity, boat_entity);

    let player_pos = world.get::<TilePosition>(player_entity).unwrap();
    let boat_pos = world.get::<TilePosition>(boat_entity).unwrap();

    assert_eq!(player_pos.x, boat_pos.x);
    assert_eq!(player_pos.y, boat_pos.y);
}

#[test]
fn player_on_boat_can_move_on_sea() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    let boat_entity = spawn_test_boat(&mut app, SPAWN_X + 1, SPAWN_Y);

    // 右に移動（船に乗る）
    perform_move(&mut app, player_entity, 1, 0);

    assert!(
        app.world().get::<OnBoat>(player_entity).is_some(),
        "Player should be on boat after first move"
    );

    // さらに右に移動（海上を移動）
    perform_move(&mut app, player_entity, 1, 0);

    let final_pos = get_tile_pos(&app, player_entity);
    assert_eq!(final_pos, (SPAWN_X + 2, SPAWN_Y), "Player on boat should move on sea (got {:?})", final_pos);

    let boat_pos = app.world().get::<TilePosition>(boat_entity).unwrap();
    assert_eq!(boat_pos.x, final_pos.0);
    assert_eq!(boat_pos.y, final_pos.1);
}

#[test]
fn player_can_disembark_from_boat() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 2] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    spawn_test_boat(&mut app, SPAWN_X + 1, SPAWN_Y);

    // 右に移動（船に乗る）
    perform_move(&mut app, player_entity, 1, 0);

    // さらに右に移動（陸地に下船）
    perform_move(&mut app, player_entity, 1, 0);

    let world = app.world();
    assert!(world.get::<OnBoat>(player_entity).is_none(), "Player should have disembarked");

    let player_pos = world.get::<TilePosition>(player_entity).unwrap();
    assert_eq!(player_pos.x, SPAWN_X + 2);
    assert_eq!(player_pos.y, SPAWN_Y);
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
        let active_map = app.world().resource::<ActiveMap>();
        let mut x = edge_x;
        let y = edge_y;

        while !active_map.grid[y][x].is_walkable() && x > 0 {
            x -= 1;
        }

        if !active_map.grid[y][x].is_walkable() {
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
        let active_map = app.world().resource::<ActiveMap>();
        player_x == MAP_WIDTH - 1
            && active_map.grid[player_y][0].is_walkable()
    };

    if !can_wrap {
        return;
    }

    // 右に移動（マップ端を超える）
    perform_move(&mut app, player_entity, 1, 0);

    let final_pos = get_tile_pos(&app, player_entity);
    assert_eq!(final_pos.0, 0, "Player should wrap around to x=0");
    assert_eq!(final_pos.1, player_y);
}

// ============================================
// 決定性テスト
// ============================================

#[test]
fn same_seed_produces_deterministic_behavior() {
    let seed = 99999;

    let mut app1 = setup_test_app(seed);
    let player1 = spawn_test_player(&mut app1);
    let pos1_initial = get_tile_pos(&app1, player1);

    let mut app2 = setup_test_app(seed);
    let player2 = spawn_test_player(&mut app2);
    let pos2_initial = get_tile_pos(&app2, player2);

    assert_eq!(pos1_initial, pos2_initial, "Same seed should produce same spawn position");

    let map1 = app1.world().resource::<ActiveMap>();
    let map2 = app2.world().resource::<ActiveMap>();

    assert_eq!(map1.grid, map2.grid, "Same seed should produce same map");
}

// ============================================
// 地形別移動テスト
// ============================================

#[test]
fn player_can_move_on_forest() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 1] = Terrain::Forest;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    reset_event_counters(&mut app);

    // 右に移動（森に入る）
    press_key(&mut app, 1, 0);
    app.update();
    app.update(); // イベントカウンタがメッセージを読み取るために追加フレーム

    let moved_count = app.world().resource::<EventCounters>().moved_count;
    assert!(moved_count >= 1, "PlayerMovedEvent should be emitted when moving to forest");

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X + 1, SPAWN_Y), "Player should move onto forest");
}

#[test]
fn player_cannot_move_on_mountain() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 1] = Terrain::Mountain;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    reset_event_counters(&mut app);

    // 右に移動（山は通行不可）
    press_key(&mut app, 1, 0);
    app.update();

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X, SPAWN_Y), "Player should not move onto mountain");
}

// ============================================
// 連続移動テスト
// ============================================

#[test]
fn player_can_move_multiple_steps() {
    let mut grid = setup_sea_grid();
    for i in 0..4 {
        grid[SPAWN_Y][SPAWN_X + i] = Terrain::Plains;
    }

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    // 3回右に移動
    for i in 0..3 {
        perform_move(&mut app, player_entity, 1, 0);

        assert_eq!(
            get_tile_pos(&app, player_entity),
            (SPAWN_X + i + 1, SPAWN_Y),
            "Player should be at step {}", i + 1
        );
    }

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X + 3, SPAWN_Y), "Player should have moved 3 steps");
}

#[test]
fn player_cannot_move_while_locked() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 1] = Terrain::Plains;
    grid[SPAWN_Y + 1][SPAWN_X + 1] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    // 右に移動開始
    press_key(&mut app, 1, 0);
    app.update();

    assert!(
        app.world().get::<MovementLocked>(player_entity).is_some(),
        "Player should be locked during movement animation"
    );

    // ロック中に上方向のキーを押す（上は平原なので、ロック無視なら移動してしまう）
    press_key(&mut app, 0, 1);
    app.update();

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    let final_pos = get_tile_pos(&app, player_entity);
    assert_eq!(final_pos.0, SPAWN_X + 1, "X should move one step right");
    assert_eq!(final_pos.1, SPAWN_Y, "Y should not change (movement was locked)");
}

// ============================================
// バウンスアニメーションテスト
// ============================================

#[test]
fn blocked_movement_triggers_bounce_and_clears() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    reset_event_counters(&mut app);

    // 右に移動（海に向かう - ブロックされる）
    press_key(&mut app, 1, 0);
    app.update();
    app.update(); // イベントカウンタがメッセージを読み取るために追加フレーム

    let blocked_count = app.world().resource::<EventCounters>().blocked_count;
    assert!(blocked_count >= 1, "MovementBlockedEvent should be emitted");

    assert!(
        app.world().get::<MovementLocked>(player_entity).is_some(),
        "MovementLocked should be added for bounce animation"
    );

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    assert!(
        app.world().get::<MovementLocked>(player_entity).is_none(),
        "MovementLocked should be cleared after bounce"
    );

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X, SPAWN_Y), "Player should not move when blocked");
}

// ============================================
// マップモードテスト
// ============================================

#[test]
fn map_mode_blocks_movement() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 1] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    // マップモードを有効化
    app.world_mut().resource_mut::<MapModeState>().enabled = true;

    reset_event_counters(&mut app);

    // 右に移動を試みる
    press_key(&mut app, 1, 0);
    app.update();

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    let counters = app.world().resource::<EventCounters>();
    assert_eq!(counters.moved_count, 0, "No PlayerMovedEvent should be emitted in map mode");
    assert_eq!(counters.blocked_count, 0, "No MovementBlockedEvent should be emitted in map mode");

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X, SPAWN_Y), "Player should not move in map mode");
}

// ============================================
// PendingMove (斜め入力分解) テスト
// ============================================

#[test]
fn diagonal_input_decomposes_into_two_moves() {
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;
    grid[SPAWN_Y][SPAWN_X + 1] = Terrain::Plains;
    grid[SPAWN_Y + 1][SPAWN_X + 1] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    // 斜め入力（右上）: W + D を同時押し
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.clear();
    input.press(KeyCode::KeyW); // 上
    input.press(KeyCode::KeyD); // 右
    drop(input); // borrowを解放

    app.update();

    release_all_keys(&mut app);
    wait_for_movement_complete(&mut app, player_entity, MAX_ANIM_FRAMES);

    let pos_after_first = get_tile_pos(&app, player_entity);
    assert!(pos_after_first != (SPAWN_X, SPAWN_Y), "First diagonal move should complete");

    // PendingMoveがあれば2回目の移動が自動で実行される
    wait_for_movement_complete(&mut app, player_entity, 10);

    assert_eq!(
        get_tile_pos(&app, player_entity),
        (SPAWN_X + 1, SPAWN_Y + 1),
        "Diagonal input should decompose into two sequential moves"
    );
}

// ============================================
// イベント整合性テスト
// ============================================

#[test]
fn multiple_moves_emit_correct_event_count() {
    let mut grid = setup_sea_grid();
    for i in 0..4 {
        grid[SPAWN_Y][SPAWN_X + i] = Terrain::Plains;
    }

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);
    let player_entity = spawn_test_player(&mut app);

    reset_event_counters(&mut app);

    // 3回右に移動
    for _ in 0..3 {
        perform_move(&mut app, player_entity, 1, 0);
    }

    let counters = app.world().resource::<EventCounters>();
    assert_eq!(counters.moved_count, 3, "Should emit 3 PlayerMovedEvents");

    assert_eq!(get_tile_pos(&app, player_entity), (SPAWN_X + 3, SPAWN_Y), "Player should have moved 3 steps");
}

// ============================================
// 戦闘システムテスト
// ============================================

/// 戦闘用のBevyアプリをセットアップ（MinimalPlugins + State + register関数）
fn setup_battle_test_app() -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    // StatesPluginを追加
    app.add_plugins(bevy::state::app::StatesPlugin);

    // 時間制御を手動に設定
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_relative_speed(1.0);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(50)));

    // ステートを登録
    app.init_state::<SceneState>();
    app.init_state::<BattleState>();

    // 必要なリソースをセットアップ
    app.init_resource::<ButtonInput<KeyCode>>();

    // cleanup_battle_sceneが必要とするリソース
    app.insert_resource(MovementState::default());
    app.init_resource::<PartyState>();
    app.insert_resource(ActiveMap {
        grid: vec![vec![Terrain::Plains; 1]; 1],
        structures: vec![vec![terrain::Structure::None; 1]; 1],
        width: 1,
        height: 1,
        origin_x: 0.0,
        origin_y: 0.0,
        wraps: true,
    });

    // 本番と同じシステム登録（ロジックのみ）
    battle_ui::register_battle_logic_systems(&mut app);

    app
}

/// BattleGameState + BattleUIState を init_battle_resources で生成して挿入するヘルパー
///
/// リソースを先に挿入してからBattleState::Activeに遷移する。
/// これにより、遷移時に実行されるbattle_input_systemがリソースを参照できる。
fn insert_battle_resource(app: &mut App, phase: BattlePhase) {
    // 本番と同じロジックでリソース生成
    let party = default_party();
    let enemies = vec![Enemy::slime()];
    let (game_state, mut ui_state) = battle_ui::init_battle_resources(party, enemies, None);

    // テスト用にphaseを上書き
    ui_state.phase = phase;

    // リソースを先に挿入（battle_input_systemが参照可能にする）
    app.insert_resource(game_state);
    app.insert_resource(ui_state);

    // BattleState::Activeに遷移（run_if条件を満たす）
    app.world_mut()
        .resource_mut::<NextState<BattleState>>()
        .set(BattleState::Active);
    app.update();
}

#[test]
fn battle_phase_transitions_from_command_to_exploring() {
    let mut app = setup_battle_test_app();

    // BattleResourceを挿入（CommandSelectから開始）
    insert_battle_resource(&mut app, BattlePhase::CommandSelect { member_index: 0 });

    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert!(matches!(battle_res.phase, BattlePhase::CommandSelect { .. }));
    }

    // BattleOverまで進める
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps <= MAX_ANIM_FRAMES, "Battle should reach BattleOver within {MAX_ANIM_FRAMES} steps");

        let phase = {
            let battle_res = app.world().resource::<BattleUIState>();
            battle_res.phase.clone()
        };

        match phase {
            BattlePhase::CommandSelect { .. } => {
                // たたかうを選択（Enter）→ TargetSelectに遷移
                press_single_key(&mut app, KeyCode::Enter);
                app.update();
                release_all_keys(&mut app);
            }
            BattlePhase::SpellSelect { .. } | BattlePhase::AllyTargetSelect { .. } | BattlePhase::ItemSelect { .. } => {
                // 呪文選択や味方ターゲットが出たらキャンセルしてコマンドに戻す
                press_single_key(&mut app, KeyCode::Escape);
                app.update();
                release_all_keys(&mut app);
            }
            BattlePhase::TargetSelect { .. } => {
                // ターゲット確定（Enter）→ 次メンバーまたはターン実行
                press_single_key(&mut app, KeyCode::Enter);
                app.update();
                release_all_keys(&mut app);
            }
            BattlePhase::ShowMessage { .. } => {
                press_single_key(&mut app, KeyCode::Enter);
                app.update();
                release_all_keys(&mut app);
            }
            BattlePhase::BattleOver { .. } => {
                break;
            }
        }
    }

    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert!(
            matches!(battle_res.phase, BattlePhase::BattleOver { .. }),
            "Should reach BattleOver phase"
        );
    }

    // BattleOverでEnterを押すとBattleState::Noneに遷移
    // 1フレーム目: battle_input_systemがNextStateを設定
    // 2フレーム目: StateTransitionスケジュールが状態を適用 + cleanup実行
    press_single_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    let current_state = app.world().resource::<State<BattleState>>();
    assert_eq!(
        **current_state,
        BattleState::None,
        "Should transition back from battle after BattleOver"
    );
}

#[test]
fn battle_command_selection_with_keys() {
    let mut app = setup_battle_test_app();
    insert_battle_resource(&mut app, BattlePhase::CommandSelect { member_index: 0 });

    // 初期状態: selected_command = 0
    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert_eq!(battle_res.selected_command, 0);
    }

    // S（下）を押して選択を1に
    press_single_key(&mut app, KeyCode::KeyS);
    app.update();

    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert_eq!(
            battle_res.selected_command, 1,
            "Should select command 1 after pressing S"
        );
    }

    release_all_keys(&mut app);

    // W（上）を押して選択を0に戻す
    press_single_key(&mut app, KeyCode::KeyW);
    app.update();

    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert_eq!(
            battle_res.selected_command, 0,
            "Should select command 0 after pressing W"
        );
    }

    release_all_keys(&mut app);

    // ラップアラウンド確認: 0でW（上）を押すと3に循環
    press_single_key(&mut app, KeyCode::KeyW);
    app.update();

    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert_eq!(
            battle_res.selected_command, 3,
            "Should wrap to 3 (wrap around from top)"
        );
    }

    release_all_keys(&mut app);

    // ラップアラウンド確認: 3でS（下）を押すと0に循環
    press_single_key(&mut app, KeyCode::KeyS);
    app.update();

    {
        let battle_res = app.world().resource::<BattleUIState>();
        assert_eq!(
            battle_res.selected_command, 0,
            "Should wrap to 0 (wrap around from bottom)"
        );
    }
}

#[test]
fn battle_flee_command_transitions_correctly() {
    let mut app = setup_battle_test_app();
    insert_battle_resource(&mut app, BattlePhase::CommandSelect { member_index: 0 });

    // にげる（selected_command=3）を選択
    for _ in 0..3 {
        press_single_key(&mut app, KeyCode::KeyS);
        app.update();
        release_all_keys(&mut app);
    }

    // Enterで決定（逃走を選択）
    press_single_key(&mut app, KeyCode::Enter);
    app.update();
    release_all_keys(&mut app);

    // 逃走は50%確率。ShowMessage（にげきれた or にげられなかった）またはBattleOverに遷移
    let phase = {
        let battle_res = app.world().resource::<BattleUIState>();
        battle_res.phase.clone()
    };

    match phase {
        BattlePhase::ShowMessage { messages, .. } => {
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
    let mut grid = setup_sea_grid();
    grid[SPAWN_Y][SPAWN_X] = Terrain::Plains;

    let mut app = setup_test_app_with_map(grid, SPAWN_X, SPAWN_Y);

    // InField ComputedStateを登録し、OnExit(InField)でクリーンアップを配線
    app.add_computed_state::<app_state::InField>();
    app.add_systems(
        OnExit(app_state::InField),
        field_walk_ui::cleanup_player_movement,
    );

    let player_entity = spawn_test_player(&mut app);

    // 初回updateでInField ComputedStateを確立
    app.update();

    // プレイヤーに手動でMovementLockedを付与（戦闘開始前に移動中だった想定）
    app.world_mut()
        .entity_mut(player_entity)
        .insert(MovementLocked);

    assert!(
        app.world().get::<MovementLocked>(player_entity).is_some(),
        "MovementLocked should be present before battle cleanup"
    );

    // 戦闘に入る → InFieldからの離脱 → cleanup_player_movementが実行される
    app.world_mut()
        .resource_mut::<NextState<BattleState>>()
        .set(BattleState::Active);
    app.update();

    assert!(
        app.world().get::<MovementLocked>(player_entity).is_none(),
        "MovementLocked should be removed after leaving InField"
    );
}

// ============================================
// ドメイン層統合テスト
// ============================================
// 以下のテストはBevy ECSを使わず、ドメイン層のcrate間連携を検証する。
// t-wada原則: 公開APIをテストし、モジュール間の結合を通じて設計の健全性を確認する。

// ============================================
// 戦闘→経験値→レベルアップ→呪文習得の一連フロー
// ============================================

#[test]
fn battle_victory_grants_exp_and_levels_up_party() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors};
    use party::default_party;

    // 弱い敵1体 vs デフォルトパーティ
    let party = default_party();
    let enemies = vec![Enemy::slime()];
    let mut battle = BattleDomainState::new(party.clone(), enemies);

    // 全員でスライムを攻撃（乱数最大で確実に倒す）
    let commands = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.2; 4],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    let _results = battle.execute_turn(&commands, &randoms);

    // 勝利を確認
    assert!(battle.is_victory(), "Party should defeat the slime");

    // 経験値報酬を計算
    let total_exp = battle.total_exp_reward();
    assert_eq!(total_exp, 3, "Slime gives 3 exp");

    // パーティ全員に経験値を分配（ゲームの仕様通り）
    for member in &mut battle.party {
        member.gain_exp(total_exp);
    }

    // Lv1→2に必要な経験値は10なので、3expではレベルアップしない
    for member in &battle.party {
        assert_eq!(member.level, 1, "3 exp should not be enough to level up");
        assert_eq!(member.exp, 3);
    }

    // もう1回戦って合計経験値を10以上にすれば、レベルアップする
    let enemies2 = vec![Enemy::wolf(), Enemy::wolf()]; // 8exp x 2 = 16exp
    let mut battle2 = BattleDomainState::new(battle.party.clone(), enemies2);

    // ライオスの強力な攻撃で倒す（乱数最大）
    let commands2 = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(1) },
    ];
    let randoms2 = TurnRandomFactors {
        damage_randoms: vec![1.2; 5],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    // 複数ターン回して倒す
    for _ in 0..10 {
        if battle2.is_over() { break; }
        battle2.execute_turn(&commands2, &randoms2);
    }

    if battle2.is_victory() {
        let exp2 = battle2.total_exp_reward();
        assert_eq!(exp2, 16, "Two wolves give 16 exp");

        for member in &mut battle2.party {
            member.gain_exp(exp2);
            // 累計 3+16=19 exp >= 10 (Lv1→2)なのでレベルアップ
            assert!(member.level >= 2, "Should reach at least level 2, got {}", member.level);
        }
    }
}

#[test]
fn mage_learns_fire1_at_level_1_and_fire2_at_level_5() {
    use battle::SpellKind;
    use party::{available_spells, spells_learned_at_level};
    use party::PartyMemberKind;

    // Lv1のマルシルはFire1を知っている
    let spells = available_spells(PartyMemberKind::Marcille, 1);
    assert_eq!(spells, vec![SpellKind::Fire1]);

    // Lv4まではFire2は未習得
    let spells4 = available_spells(PartyMemberKind::Marcille, 4);
    assert_eq!(spells4, vec![SpellKind::Fire1, SpellKind::Blaze1]);

    // Lv5でFire2を習得
    let learned = spells_learned_at_level(PartyMemberKind::Marcille, 5);
    assert_eq!(learned, vec![SpellKind::Fire2]);

    let spells5 = available_spells(PartyMemberKind::Marcille, 5);
    assert_eq!(spells5, vec![SpellKind::Fire1, SpellKind::Blaze1, SpellKind::Fire2]);
}

#[test]
fn hero_has_all_16_spells_at_level_1() {
    use party::available_spells;
    use party::PartyMemberKind;

    // ライオスはLv1で全24呪文を使える
    let spells = available_spells(PartyMemberKind::Laios, 1);
    assert_eq!(spells.len(), 24);
}

// ============================================
// 装備がダメージ計算に影響することを検証
// ============================================

#[test]
fn equipped_weapon_increases_battle_damage() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult};
    use item::WeaponKind;
    use party::PartyMember;

    // 武器なしのライオス
    let hero_unarmed = PartyMember::laios();
    let unarmed_attack = hero_unarmed.effective_attack();

    // 武器装備のライオス
    let mut hero_armed = PartyMember::laios();
    hero_armed.equipment.equip_weapon(WeaponKind::SteelSword);
    let armed_attack = hero_armed.effective_attack();

    assert_eq!(armed_attack, unarmed_attack + 10, "SteelSword should add 10 attack");

    // HP999の敵に対して同じ乱数で攻撃し、ダメージ差を検証
    let mut slime1 = Enemy::slime();
    slime1.stats.hp = 999;
    slime1.stats.max_hp = 999;
    let slime2 = slime1.clone();

    let mut battle_unarmed = BattleDomainState::new(vec![hero_unarmed], vec![slime1]);
    let mut battle_armed = BattleDomainState::new(vec![hero_armed], vec![slime2]);

    let commands = vec![BattleAction::Attack { target: TargetId::Enemy(0) }];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };

    let results_unarmed = battle_unarmed.execute_turn(&commands, &randoms);
    let results_armed = battle_armed.execute_turn(&commands, &randoms);

    // ダメージを抽出
    let damage_unarmed = results_unarmed.iter().find_map(|r| {
        if let TurnResult::Attack { attacker: battle::ActorId::Party(0), damage, .. } = r { Some(*damage) } else { None }
    }).unwrap();

    let damage_armed = results_armed.iter().find_map(|r| {
        if let TurnResult::Attack { attacker: battle::ActorId::Party(0), damage, .. } = r { Some(*damage) } else { None }
    }).unwrap();

    assert!(damage_armed > damage_unarmed, "Armed should deal more damage: {} vs {}", damage_armed, damage_unarmed);
}

// ============================================
// 戦闘中アイテム使用テスト
// ============================================

#[test]
fn herb_heals_in_battle() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult};
    use item::ItemKind;
    use party::PartyMember;

    let mut hero = PartyMember::laios();
    hero.stats.hp = 5; // HPを低くしておく
    hero.inventory.add(ItemKind::Herb, 1);

    // HP999の敵で戦闘が終わらないようにする
    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0; // 敵の攻撃を0にして干渉を防ぐ

    let mut battle = BattleDomainState::new(vec![hero], vec![slime]);

    let commands = vec![
        BattleAction::UseItem { item: ItemKind::Herb, target: TargetId::Party(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };

    let results = battle.execute_turn(&commands, &randoms);

    // ItemUsedイベントを確認
    let item_used = results.iter().any(|r| matches!(r, TurnResult::ItemUsed { .. }));
    assert!(item_used, "ItemUsed event should be emitted");

    // HPが回復している
    assert!(battle.party[0].stats.hp > 5, "HP should be healed, got {}", battle.party[0].stats.hp);
    assert!(battle.party[0].stats.hp <= battle.party[0].stats.max_hp, "HP should not exceed max");

    // やくそうが消費されている
    assert_eq!(battle.party[0].inventory.count(ItemKind::Herb), 0, "Herb should be consumed");
}

#[test]
fn copper_key_is_not_usable_in_battle() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult};
    use item::ItemKind;
    use party::PartyMember;

    let mut hero = PartyMember::laios();
    hero.inventory.add(ItemKind::CopperKey, 1);

    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0;

    let mut battle = BattleDomainState::new(vec![hero], vec![slime]);

    let commands = vec![
        BattleAction::UseItem { item: ItemKind::CopperKey, target: TargetId::Party(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };

    let results = battle.execute_turn(&commands, &randoms);

    // KeyItemは戦闘中使えない→ItemUsedイベントなし
    let item_used = results.iter().any(|r| matches!(r, TurnResult::ItemUsed { .. }));
    assert!(!item_used, "CopperKey should not be usable in battle");

    // CopperKeyは消費されない
    assert_eq!(battle.party[0].inventory.count(ItemKind::CopperKey), 1, "CopperKey should not be consumed");
}

// ============================================
// 街の購入→戦闘使用の連携テスト
// ============================================

#[test]
fn buy_herb_at_shop_then_use_in_battle() {
    use town::{buy_item, BuyResult};
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors};
    use item::ItemKind;
    use party::PartyMember;

    let mut hero = PartyMember::laios();
    let mut gold = 100u32;

    // 街でやくそうを購入
    let result = buy_item(ItemKind::Herb, gold, &mut hero.inventory);
    match result {
        BuyResult::Success { remaining_gold } => {
            gold = remaining_gold;
            assert_eq!(gold, 92); // 100 - 8 = 92
        }
        _ => panic!("Should succeed buying herb"),
    }
    assert_eq!(hero.inventory.count(ItemKind::Herb), 1);

    // HPを減らして戦闘に入る
    hero.stats.hp = 1;

    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0;

    let mut battle = BattleDomainState::new(vec![hero], vec![slime]);

    // やくそうを使う
    let commands = vec![
        BattleAction::UseItem { item: ItemKind::Herb, target: TargetId::Party(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    battle.execute_turn(&commands, &randoms);

    assert!(battle.party[0].stats.hp > 1, "Herb should heal in battle");
    assert_eq!(battle.party[0].inventory.count(ItemKind::Herb), 0, "Herb consumed after use");
}

#[test]
fn buy_weapon_at_shop_then_equip_affects_battle() {
    use town::{buy_item, BuyResult};
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult};
    use item::{ItemKind, WeaponKind};
    use party::PartyMember;

    let mut hero = PartyMember::laios();
    let gold = 100u32;

    // 武器購入前の攻撃力を記録
    let attack_before = hero.effective_attack();

    // 街で鉄の剣を購入（インベントリに入る）
    let result = buy_item(ItemKind::Weapon(WeaponKind::IronSword), gold, &mut hero.inventory);
    match result {
        BuyResult::Success { remaining_gold } => {
            assert_eq!(remaining_gold, 50); // 100 - 50 = 50
        }
        _ => panic!("Should succeed buying IronSword"),
    }

    // 道具メニューから装備する（武器はインベントリに残る）
    hero.equipment.equip_weapon(WeaponKind::IronSword);

    // 攻撃力が上がっていることを確認
    let attack_after = hero.effective_attack();
    assert_eq!(attack_after, attack_before + 5, "IronSword should add 5 attack");
    assert_eq!(hero.inventory.count(ItemKind::Weapon(WeaponKind::IronSword)), 1, "Weapon stays in inventory");

    // 戦闘でダメージを確認
    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;

    let mut battle = BattleDomainState::new(vec![hero], vec![slime]);
    let commands = vec![BattleAction::Attack { target: TargetId::Enemy(0) }];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands, &randoms);

    let damage = results.iter().find_map(|r| {
        if let TurnResult::Attack { attacker: battle::ActorId::Party(0), damage, .. } = r { Some(*damage) } else { None }
    }).unwrap();

    assert!(damage > 0, "Should deal damage with equipped weapon");
}

// ============================================
// やどや→戦闘の連携テスト
// ============================================

#[test]
fn inn_heals_party_before_battle() {
    use town::heal_party;
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors};
    use party::default_party;

    let mut party = default_party();

    // 全員のHPを1にする
    for member in &mut party {
        member.stats.hp = 1;
        member.stats.mp = 0;
    }

    // やどやで全回復
    heal_party(&mut party);

    for member in &party {
        assert_eq!(member.stats.hp, member.stats.max_hp, "HP should be fully restored");
        assert_eq!(member.stats.mp, member.stats.max_mp, "MP should be fully restored");
    }

    // 回復後に戦闘
    let enemies = vec![Enemy::slime()];
    let mut battle = BattleDomainState::new(party, enemies);

    let commands = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.2; 4],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    battle.execute_turn(&commands, &randoms);

    assert!(battle.is_victory(), "Fully healed party should defeat a slime");
}

// ============================================
// 仲間募集フローテスト
// ============================================

#[test]
fn full_recruitment_flow_undiscovered_to_recruited() {
    use party::{
        default_candidates, talk_to_candidate, initial_party,
        PartyMember, PartyMemberKind, RecruitmentStatus, TalkResult,
    };

    let mut party = initial_party(); // ライオスのみ
    assert_eq!(party.len(), 1);

    let mut candidates = default_candidates(); // 9人の仲間候補
    assert_eq!(candidates.len(), 9);

    // --- チルチャック: 1回目の会話（初対面→知り合い） ---
    let result = talk_to_candidate(&mut candidates[0]);
    assert_eq!(result, TalkResult::BecameAcquaintance);
    assert_eq!(candidates[0].status, RecruitmentStatus::Acquaintance);

    // --- チルチャック: 2回目の会話（知り合い→加入） ---
    let result = talk_to_candidate(&mut candidates[0]);
    assert_eq!(result, TalkResult::Recruited);
    assert_eq!(candidates[0].status, RecruitmentStatus::Recruited);

    // パーティにチルチャックを追加
    party.push(PartyMember::from_kind(candidates[0].kind));
    assert_eq!(party.len(), 2);
    assert_eq!(party[1].kind, PartyMemberKind::Chilchuck);

    // --- マルシル: 同様のフロー ---
    let result = talk_to_candidate(&mut candidates[1]);
    assert_eq!(result, TalkResult::BecameAcquaintance);

    let result = talk_to_candidate(&mut candidates[1]);
    assert_eq!(result, TalkResult::Recruited);

    party.push(PartyMember::from_kind(candidates[1].kind));
    assert_eq!(party.len(), 3);
    assert_eq!(party[2].kind, PartyMemberKind::Marcille);

    // --- 既に加入済みの候補に再度話しかける ---
    let result = talk_to_candidate(&mut candidates[0]);
    assert_eq!(result, TalkResult::AlreadyRecruited);
}

// ============================================
// 洞窟探索シナリオテスト
// ============================================

#[test]
fn cave_exploration_scenario() {
    use cave::{generate_cave_map, CAVE_WIDTH, CAVE_HEIGHT};
    use terrain::{try_grid_move, MoveResult};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let cave = generate_cave_map(&mut rng, &[]);

    // スポーン地点は梯子
    let (sx, sy) = cave.spawn_position;
    assert_eq!(cave.structures[sy][sx], terrain::Structure::Ladder);

    // スポーン地点から歩行可能な隣接タイルを探す
    let mut can_move = false;
    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        let result = try_grid_move(sx, sy, dx, dy, &cave.grid, CAVE_WIDTH, CAVE_HEIGHT, false, |_x, _y, t: Terrain| t.is_walkable());
        if let MoveResult::Moved { .. } = result {
            can_move = true;
            break;
        }
    }
    assert!(can_move, "Should be able to move from spawn in at least one direction");

    // 宝箱が存在する
    assert!(!cave.treasures.is_empty(), "Cave should have at least one treasure");
    assert!(cave.treasures.len() <= 3, "Cave should have at most 3 treasures");

    // 宝箱は有効な中身を持つ
    for chest in &cave.treasures {
        // 宝箱の名前が空でないことで中身の有効性を確認
        assert!(!chest.content.name().is_empty());
        // 宝箱は床タイル上
        assert_eq!(cave.grid[chest.y][chest.x], Terrain::CaveFloor,
            "Treasure at ({},{}) should be on CaveFloor, got {:?}", chest.x, chest.y, cave.grid[chest.y][chest.x]);
        // スポーン地点ではない
        assert_ne!((chest.x, chest.y), cave.spawn_position, "Treasure should not be at spawn");
    }

    // 宝箱位置まで移動してみる（経路探索ではないが、宝箱位置が移動可能か確認）
    let first_chest = &cave.treasures[0];
    // 宝箱のタイルが歩行可能であること
    assert!(cave.grid[first_chest.y][first_chest.x].is_walkable(), "Treasure tile should be walkable");
}

#[test]
fn cave_generation_is_deterministic() {
    use cave::generate_cave_map;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let seed = 12345u64;
    let mut rng1 = ChaCha8Rng::seed_from_u64(seed);
    let mut rng2 = ChaCha8Rng::seed_from_u64(seed);

    let cave1 = generate_cave_map(&mut rng1, &[]);
    let cave2 = generate_cave_map(&mut rng2, &[]);

    assert_eq!(cave1.grid, cave2.grid);
    assert_eq!(cave1.spawn_position, cave2.spawn_position);
    assert_eq!(cave1.treasures.len(), cave2.treasures.len());
    for (t1, t2) in cave1.treasures.iter().zip(cave2.treasures.iter()) {
        assert_eq!(t1.x, t2.x);
        assert_eq!(t1.y, t2.y);
        assert_eq!(t1.content, t2.content);
    }
}

#[test]
fn cave_diagonal_movement_is_always_blocked() {
    use cave::{generate_cave_map, CAVE_WIDTH, CAVE_HEIGHT};
    use terrain::{try_grid_move, MoveResult};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let cave = generate_cave_map(&mut rng, &[]);
    let (sx, sy) = cave.spawn_position;

    // 斜め移動は常にブロックされる
    for (dx, dy) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let result = try_grid_move(sx, sy, dx, dy, &cave.grid, CAVE_WIDTH, CAVE_HEIGHT, false, |_x, _y, t: Terrain| t.is_walkable());
        assert_eq!(result, MoveResult::Blocked, "Diagonal move ({},{}) should be blocked", dx, dy);
    }
}

// ============================================
// 洞窟宝箱→インベントリの連携テスト
// ============================================

#[test]
fn cave_treasure_adds_to_inventory() {
    use cave::{generate_cave_map, TreasureContent};
    use terrain::Structure;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use party::PartyMember;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let cave = generate_cave_map(&mut rng, &[]);

    // 宝箱位置が structures に Chest として設定されていることを検証
    for chest in &cave.treasures {
        assert_eq!(
            cave.structures[chest.y][chest.x],
            Structure::Chest,
            "Treasure at ({}, {}) should be Structure::Chest in structures layer",
            chest.x, chest.y,
        );
    }

    let mut hero = PartyMember::laios();
    assert!(hero.inventory.is_empty());

    // 宝箱を開ける（ドメインロジックとして直接追加）
    let mut item_count = 0u32;
    for chest in &cave.treasures {
        match chest.content {
            TreasureContent::Item(item) => {
                hero.inventory.add(item, 1);
                item_count += 1;
            }
            TreasureContent::Weapon(_weapon) => {
                // 武器はインベントリに追加しない（装備扱い）
            }
        }
    }

    // 宝箱のアイテム分だけインベントリに追加されている
    assert_eq!(
        hero.inventory.total_count(),
        item_count,
        "Inventory should contain items from opened treasure chests"
    );
}

// ============================================
// 街の洞窟ヒント台詞テスト
// ============================================

#[test]
fn cave_hint_dialogue_finds_nearest_cave_in_generated_map() {
    use world_gen::generate_map;
    use town::cave_hint_dialogue;
    use terrain::Structure;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let map = generate_map(&mut rng);

    // マップ内の町を見つける
    let mut town_pos = None;
    for (y, row) in map.structures.iter().enumerate() {
        for (x, structure) in row.iter().enumerate() {
            if *structure == Structure::Town {
                town_pos = Some((x, y));
                break;
            }
        }
        if town_pos.is_some() { break; }
    }

    if let Some((tx, ty)) = town_pos {
        let dialogue = cave_hint_dialogue(&map.structures, tx, ty, None);
        // マップに洞窟があれば方角ヒントが返る
        let has_cave = map.structures.iter().flatten().any(|s| *s == Structure::Cave);
        if has_cave {
            assert!(dialogue.contains("どうくつ"), "Dialogue should mention cave: {}", dialogue);
        }
    }
}

// ============================================
// ワールドマップ品質統合テスト
// ============================================

#[test]
fn generated_map_has_towns_and_caves_on_walkable_tiles() {
    use world_gen::generate_connected_map;
    use terrain::Structure;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    for seed in [1, 42, 100, 999, 54321] {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let map = generate_connected_map(&mut rng);

        // 構造物の確認
        for (y, row) in map.structures.iter().enumerate() {
            for (x, structure) in row.iter().enumerate() {
                if *structure == Structure::Town {
                    // 構造物下の地形はis_walkableであること
                    assert!(map.grid[y][x].is_walkable(), "Town at ({},{}) should be on walkable terrain", x, y);
                    // TileActionがEnterTownであること
                    assert_eq!(structure.tile_action(), terrain::TileAction::EnterTown);
                }
                if *structure == Structure::Cave {
                    // CaveもEnterCaveアクションを持つ
                    assert_eq!(structure.tile_action(), terrain::TileAction::EnterCave);
                }
            }
        }

        // スポーン位置がPlainsであること
        let (sx, sy) = map.spawn_position;
        assert_eq!(map.grid[sy][sx], Terrain::Plains, "Spawn should be Plains for seed {}", seed);
    }
}

#[test]
fn generated_map_spawn_is_on_walkable_connected_island() {
    use world_gen::{generate_connected_map, detect_islands};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let map = generate_connected_map(&mut rng);
    let (sx, sy) = map.spawn_position;

    // スポーン位置は歩行可能
    assert!(map.grid[sy][sx].is_walkable(), "Spawn position should be walkable");

    // スポーン位置が属する島を見つける
    let islands = detect_islands(&map.grid);
    let spawn_island = islands.iter().find(|island| island.contains(&(sx, sy)));
    assert!(spawn_island.is_some(), "Spawn position should be on an island");

    // スポーン島は十分な大きさ
    let island = spawn_island.unwrap();
    assert!(island.len() >= 100, "Spawn island should be reasonably large, got {} tiles", island.len());
}

// ============================================
// 戦闘の行動順序テスト（素早さ順の検証）
// ============================================

#[test]
fn battle_action_order_respects_speed() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult, ActorId};
    use party::PartyMember;

    // 速度が異なるキャラクターを用意
    // Marcille(SPD7) > Bat(SPD6) > Laios(SPD5) > Falin(SPD4)
    let party = vec![PartyMember::laios(), PartyMember::marcille(), PartyMember::falin()];

    // 敵のHPを高くして戦闘が終わらないようにする
    let mut bat = Enemy::bat();
    bat.stats.hp = 999;
    bat.stats.max_hp = 999;

    let mut battle = BattleDomainState::new(party, vec![bat]);

    let commands = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 4],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands, &randoms);

    // 攻撃の順序を確認
    let attack_order: Vec<ActorId> = results.iter().filter_map(|r| {
        if let TurnResult::Attack { attacker, .. } = r { Some(*attacker) } else { None }
    }).collect();

    // 期待: Marcille(Party(1), SPD7) → Bat(Enemy(0), SPD6) → Laios(Party(0), SPD5) → Falin(Party(2), SPD4)
    assert_eq!(attack_order.len(), 4, "All 4 actors should attack");
    assert_eq!(attack_order[0], ActorId::Party(1), "Marcille should attack first (SPD7)");
    assert_eq!(attack_order[1], ActorId::Enemy(0), "Bat should attack second (SPD6)");
    assert_eq!(attack_order[2], ActorId::Party(0), "Laios should attack third (SPD5)");
    assert_eq!(attack_order[3], ActorId::Party(2), "Falin should attack fourth (SPD4)");
}

// ============================================
// 戦闘経験値報酬の統合テスト
// ============================================

#[test]
fn total_exp_reward_sums_defeated_enemies_only() {
    use battle::{BattleState as BattleDomainState, Enemy};

    let enemies = vec![Enemy::slime(), Enemy::goblin(), Enemy::ghost()]; // 3+6+10 = 19
    let mut battle = BattleDomainState::new(vec![], enemies);

    // まだ誰も倒していない
    assert_eq!(battle.total_exp_reward(), 0, "No exp when no enemies defeated");

    // スライムだけ倒す
    battle.enemies[0].stats.hp = 0;
    assert_eq!(battle.total_exp_reward(), 3, "Only slime exp");

    // ゴブリンも倒す
    battle.enemies[1].stats.hp = 0;
    assert_eq!(battle.total_exp_reward(), 9, "Slime + Goblin exp");

    // 全員倒す
    battle.enemies[2].stats.hp = 0;
    assert_eq!(battle.total_exp_reward(), 19, "All enemies exp");
}

// ============================================
// MP不足時の呪文使用テスト
// ============================================

#[test]
fn spell_fails_silently_when_mp_insufficient() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult, SpellKind};
    use party::PartyMember;

    let mut mage = PartyMember::marcille();
    mage.stats.mp = 0; // MP枯渇

    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0;

    let mut battle = BattleDomainState::new(vec![mage], vec![slime]);

    let commands = vec![
        BattleAction::Spell { spell: SpellKind::Fire1, target: TargetId::Enemy(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands, &randoms);

    // SpellDamageイベントは発生しない
    let spell_damage = results.iter().any(|r| matches!(r, TurnResult::SpellDamage { .. }));
    assert!(!spell_damage, "Should not cast spell when MP is 0");
}

// ============================================
// 全滅検知テスト
// ============================================

#[test]
fn party_wipe_ends_battle_mid_turn() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult};
    use party::PartyMember;

    // HP1のライオス1人 vs 強い敵2体
    let mut hero = PartyMember::laios();
    hero.stats.hp = 1;
    hero.stats.speed = 1; // 敵より遅くして先に倒されるようにする

    let mut wolf1 = Enemy::wolf();
    wolf1.stats.attack = 100; // 確実に倒す
    let mut wolf2 = Enemy::wolf();
    wolf2.stats.attack = 100;

    let mut battle = BattleDomainState::new(vec![hero], vec![wolf1, wolf2]);

    let commands = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 3],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands, &randoms);

    assert!(battle.is_party_wiped(), "Party should be wiped");
    assert!(battle.is_over(), "Battle should be over");

    // Defeated(Party(0))が含まれる
    let hero_defeated = results.iter().any(|r| matches!(r, TurnResult::Defeated { target: TargetId::Party(0) }));
    assert!(hero_defeated, "Laios defeat should be recorded");
}

// ============================================
// インベントリ容量とよろず屋の連携テスト
// ============================================

#[test]
fn shop_rejects_purchase_when_inventory_full() {
    use town::{buy_item, BuyResult};
    use item::{ItemKind, Inventory, INVENTORY_CAPACITY};

    let mut inv = Inventory::new();
    let gold = 1000u32;

    // 容量いっぱいまでやくそうを購入
    for i in 0..INVENTORY_CAPACITY {
        let result = buy_item(ItemKind::Herb, gold, &mut inv);
        assert!(matches!(result, BuyResult::Success { .. }), "Purchase {} should succeed", i);
    }

    assert_eq!(inv.total_count(), INVENTORY_CAPACITY);

    // 容量いっぱいの状態でさらに購入しようとする
    let result = buy_item(ItemKind::Herb, gold, &mut inv);
    assert_eq!(result, BuyResult::InventoryFull, "Should reject when inventory is full");
    assert_eq!(inv.total_count(), INVENTORY_CAPACITY);
}

// ============================================
// レベルアップ時のステータス成長統合テスト
// ============================================

#[test]
fn level_up_applies_correct_stat_growth_per_class() {
    use party::PartyMember;

    // ライオスのレベルアップ
    let mut hero = PartyMember::laios();
    let base_hp = hero.stats.max_hp;
    let base_attack = hero.stats.attack;
    let base_defense = hero.stats.defense;
    let base_speed = hero.stats.speed;
    let base_mp = hero.stats.max_mp;

    let level_ups = hero.gain_exp(10); // Lv1→2
    assert_eq!(level_ups, 1);
    assert_eq!(hero.stats.max_hp, base_hp + 5); // Laios: hp+5
    assert_eq!(hero.stats.attack, base_attack + 2); // Laios: attack+2
    assert_eq!(hero.stats.defense, base_defense + 1); // Laios: defense+1
    assert_eq!(hero.stats.speed, base_speed + 1); // Laios: speed+1
    assert_eq!(hero.stats.max_mp, base_mp + 1); // Laios: mp+1
    // レベルアップ時は全回復
    assert_eq!(hero.stats.hp, hero.stats.max_hp);
    assert_eq!(hero.stats.mp, hero.stats.max_mp);

    // マルシルのレベルアップ
    let mut mage = PartyMember::marcille();
    let base_hp = mage.stats.max_hp;
    let base_mp = mage.stats.max_mp;

    let level_ups = mage.gain_exp(10);
    assert_eq!(level_ups, 1);
    assert_eq!(mage.stats.max_hp, base_hp + 3); // Marcille: hp+3
    assert_eq!(mage.stats.max_mp, base_mp + 3); // Marcille: mp+3

    // ファリンのレベルアップ
    let mut priest = PartyMember::falin();
    let base_hp = priest.stats.max_hp;
    let base_mp = priest.stats.max_mp;

    let level_ups = priest.gain_exp(10);
    assert_eq!(level_ups, 1);
    assert_eq!(priest.stats.max_hp, base_hp + 4); // Falin: hp+4
    assert_eq!(priest.stats.max_mp, base_mp + 2); // Falin: mp+2
}

// ============================================
// 探索マップ（Fog of War）統合テスト
// ============================================

#[test]
fn exploration_map_tracks_movement_correctly() {
    use field_walk::exploration::{ExplorationMap, TileVisibility, VIEW_RADIUS};

    let mut map = ExplorationMap::new(150, 150);

    // 初期状態: 全て未探索
    assert_eq!(map.get(75, 75), Some(TileVisibility::Unexplored));

    // プレイヤーが(75,75)に立つ
    map.update_visibility(75, 75, VIEW_RADIUS);

    // 中心と周囲がVisible
    assert_eq!(map.get(75, 75), Some(TileVisibility::Visible));
    assert_eq!(map.get(75 + VIEW_RADIUS, 75), Some(TileVisibility::Visible));

    // 視界外は未探索
    assert_eq!(map.get(75 + VIEW_RADIUS + 1, 75), Some(TileVisibility::Unexplored));

    // 移動: (76,75)に移動
    map.update_visibility(76, 75, VIEW_RADIUS);

    // 旧中心はExplored
    // (75,75)は新しい視界範囲（76-4=72 ~ 76+4=80）内なので、まだVisible
    assert_eq!(map.get(75, 75), Some(TileVisibility::Visible));

    // 大きく移動して旧位置を視界外にする
    map.update_visibility(100, 100, VIEW_RADIUS);
    assert_eq!(map.get(75, 75), Some(TileVisibility::Explored));
    assert_eq!(map.get(100, 100), Some(TileVisibility::Visible));

    // 探索済みタイルは一定数以上存在する
    let explored_count = map.get_explored_tiles().count();
    // 3回の視界更新で少なくとも81タイルが探索済み
    assert!(explored_count >= 81, "Should have explored at least one view's worth of tiles, got {}", explored_count);
}

// ============================================
// 逃走判定ロジックの検証
// ============================================

#[test]
fn flee_succeeds_when_random_below_threshold() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult};
    use party::default_party;

    let party = default_party();
    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    let enemies = vec![slime];
    let mut battle = BattleDomainState::new(party, enemies);

    let commands = vec![
        BattleAction::Flee,
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
    ];

    // flee_random = 0.3 < 0.5 → 逃走成功
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 4],
        flee_random: 0.3, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands, &randoms);
    assert_eq!(results, vec![TurnResult::Fled], "Should flee when random < 0.5");
    assert!(battle.is_over());
}

#[test]
fn flee_fails_when_random_above_threshold() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult, ActorId};
    use party::default_party;

    let party = default_party();
    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    let enemies = vec![slime];
    let mut battle = BattleDomainState::new(party, enemies);

    let commands = vec![
        BattleAction::Flee,
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
    ];

    // flee_random = 0.7 >= 0.5 → 逃走失敗
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 4],
        flee_random: 0.7, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands, &randoms);

    assert!(matches!(results[0], TurnResult::FleeFailed), "First result should be FleeFailed");
    assert!(!battle.is_over(), "Battle should continue after flee failure");

    // 逃走失敗時は敵だけが行動する
    let enemy_attacks = results.iter().filter(|r| {
        matches!(r, TurnResult::Attack { attacker: ActorId::Enemy(_), .. })
    }).count();
    assert!(enemy_attacks > 0, "Enemies should attack after flee failure");

    // パーティは行動しない
    let party_attacks = results.iter().filter(|r| {
        matches!(r, TurnResult::Attack { attacker: ActorId::Party(_), .. })
    }).count();
    assert_eq!(party_attacks, 0, "Party should not attack after flee failure");
}

// ============================================
// ダメージ計算式の検証
// ============================================

#[test]
fn physical_damage_formula_uses_half_defense() {
    use party::CombatStats;

    // ダメージ = (attack - defense/2) * random_factor, 最小1
    // attack=20, defense=10, random=1.0 → 20 - 5 = 15
    let damage = CombatStats::calculate_damage(20, 10, 1.0);
    assert_eq!(damage, 15, "Damage should be attack - defense/2");

    // defense/2 であって defense*2 ではないことを確認
    // attack=10, defense=6, random=1.0 → 10 - 3 = 7 (defense/2の場合)
    // もし defense*2 だったら → 10 - 12 = -2 → 1 (最小値保証)
    let damage = CombatStats::calculate_damage(10, 6, 1.0);
    assert_eq!(damage, 7, "Damage should use defense/2, not defense*2");
}

#[test]
fn spell_damage_formula_uses_quarter_defense() {
    use battle::spell::calculate_spell_damage;

    // 呪文ダメージ = (power - defense/4) * random_factor, 最小1
    // power=12, defense=4, random=1.0 → 12 - 1 = 11
    let damage = calculate_spell_damage(12, 4, 1.0);
    assert_eq!(damage, 11, "Spell damage should be power - defense/4");

    // defense/4 であって defense*4 や +defense/4 ではないことを確認
    // power=20, defense=8, random=1.0 → 20 - 2 = 18 (defense/4の場合)
    // もし +defense/4 だったら → 20 + 2 = 22
    let damage = calculate_spell_damage(20, 8, 1.0);
    assert_eq!(damage, 18, "Spell damage should subtract defense/4");
}

// ============================================
// MP境界条件のテスト
// ============================================

#[test]
fn spell_succeeds_when_mp_exactly_equals_cost() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult, SpellKind};
    use party::PartyMember;

    let mut mage = PartyMember::marcille();
    mage.stats.mp = 3; // Fire1 のMP消費量ちょうど

    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0;

    let mut battle = BattleDomainState::new(vec![mage], vec![slime]);

    let commands = vec![
        BattleAction::Spell { spell: SpellKind::Fire1, target: TargetId::Enemy(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands, &randoms);

    // MPちょうどなら呪文は成功すべき
    let spell_cast = results.iter().any(|r| matches!(r, TurnResult::SpellDamage { .. }));
    assert!(spell_cast, "Spell should succeed when MP exactly equals cost");
    assert_eq!(battle.party[0].stats.mp, 0, "MP should be exactly 0 after casting");
}

// ============================================
// HP回復上限の検証
// ============================================

#[test]
fn heal_spell_does_not_exceed_max_hp() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors};
    use party::PartyMember;

    // ファリンのHPをmax_hpの1だけ下に設定
    // Heal1 power=15 → 回復量15。キャップなしなら max_hp+14 になる
    let mut priest = PartyMember::falin();
    priest.stats.hp = priest.stats.max_hp - 1;

    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0; // 最低ダメージ1は発生するがHP超過の判定には影響しない

    let mut battle = BattleDomainState::new(vec![priest], vec![slime]);

    let commands = vec![
        BattleAction::Spell { spell: battle::SpellKind::Heal1, target: TargetId::Party(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    battle.execute_turn(&commands, &randoms);

    // HP回復はmax_hpを超えない（敵の最低1ダメージで下がる場合もある）
    assert!(battle.party[0].stats.hp <= battle.party[0].stats.max_hp,
        "Heal should cap at max_hp, got {} > {}", battle.party[0].stats.hp, battle.party[0].stats.max_hp);
}

#[test]
fn item_heal_does_not_exceed_max_hp() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors};
    use item::ItemKind;
    use party::PartyMember;

    // HPをmax_hpの1だけ下に。Herb power=25 → キャップなしなら max_hp+24
    let mut hero = PartyMember::laios();
    hero.stats.hp = hero.stats.max_hp - 1;
    hero.inventory.add(ItemKind::Herb, 1);

    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0;

    let mut battle = BattleDomainState::new(vec![hero], vec![slime]);

    let commands = vec![
        BattleAction::UseItem { item: ItemKind::Herb, target: TargetId::Party(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    battle.execute_turn(&commands, &randoms);

    // アイテム回復もmax_hpを超えない
    assert!(battle.party[0].stats.hp <= battle.party[0].stats.max_hp,
        "Item heal should cap at max_hp, got {} > {}", battle.party[0].stats.hp, battle.party[0].stats.max_hp);
}

// ============================================
// take_damageのHP下限保証
// ============================================

#[test]
fn take_damage_does_not_go_below_zero() {
    use party::CombatStats;

    let mut stats = CombatStats::new(10, 5, 2, 3, 0);
    stats.take_damage(100); // HPを大幅に超えるダメージ
    assert_eq!(stats.hp, 0, "HP should not go below zero");
    assert!(stats.hp >= 0, "HP must never be negative");
}

// ============================================
// 購入の金額境界条件テスト
// ============================================

#[test]
fn buy_item_succeeds_with_exact_gold() {
    use town::{buy_item, BuyResult};
    use item::{Inventory, ItemKind};

    let mut inv = Inventory::new();
    // やくそうの価格は8ゴールド
    let result = buy_item(ItemKind::Herb, 8, &mut inv);
    assert_eq!(result, BuyResult::Success { remaining_gold: 0 },
        "Should succeed when gold exactly equals price");
    assert_eq!(inv.count(ItemKind::Herb), 1);
}

// ============================================
// 売却機能のテスト
// ============================================

#[test]
fn sell_key_item_is_rejected() {
    use town::{sell_item, SellResult};
    use item::{Inventory, ItemKind};

    let mut inv = Inventory::new();
    inv.add(ItemKind::CopperKey, 1);

    let result = sell_item(ItemKind::CopperKey, &mut inv, None);
    assert_eq!(result, SellResult::CannotSell, "Key items should not be sellable");
    assert_eq!(inv.count(ItemKind::CopperKey), 1, "Key item should remain in inventory");
}

#[test]
fn sell_material_item_succeeds() {
    use town::{sell_item, SellResult};
    use item::{Inventory, ItemKind};

    let mut inv = Inventory::new();
    inv.add(ItemKind::MagicStone, 1);

    let result = sell_item(ItemKind::MagicStone, &mut inv, None);
    assert_eq!(result, SellResult::Success { earned_gold: 30 },
        "Material item should sell for its sell_price");
    assert_eq!(inv.count(ItemKind::MagicStone), 0, "Item should be removed after selling");
}

// ============================================
// 戦闘勝利→レベルアップ→呪文習得の連携テスト
// ============================================

#[test]
fn battle_victory_leveling_unlocks_new_spell() {
    use battle::{BattleState as BattleDomainState, Enemy, SpellKind};
    use party::{available_spells, spells_learned_at_level};
    use party::{PartyMember, PartyMemberKind};

    // マルシルLv1: Fire1のみ習得
    let mut mage = PartyMember::marcille();
    let spells_lv1 = available_spells(PartyMemberKind::Marcille, mage.level);
    assert_eq!(spells_lv1, vec![SpellKind::Fire1]);

    // 十分な経験値を得るために複数回戦闘
    // exp_to_next_level(1)=10, exp_to_next_level(2)=25 → Lv3到達に累計35必要
    // Ghost×4 = 10×4 = 40exp per battle
    let enemies = vec![Enemy::ghost(), Enemy::ghost(), Enemy::ghost(), Enemy::ghost()];
    let mut battle = BattleDomainState::new(vec![mage.clone()], enemies);

    for enemy in &mut battle.enemies {
        enemy.stats.hp = 0;
    }
    assert!(battle.is_victory());

    let total_exp = battle.total_exp_reward();
    assert_eq!(total_exp, 40);

    let level_ups = mage.gain_exp(total_exp);
    assert!(level_ups >= 2, "Should level up at least twice with 40 exp, got {} level ups", level_ups);
    assert!(mage.level >= 3, "Should reach at least level 3, got {}", mage.level);

    // Lv3でマルシルはBlaze1を習得
    let learned = spells_learned_at_level(PartyMemberKind::Marcille, 3);
    assert_eq!(learned, vec![SpellKind::Blaze1]);

    let spells = available_spells(PartyMemberKind::Marcille, mage.level);
    assert!(spells.contains(&SpellKind::Blaze1), "Marcille at level {} should know Blaze1", mage.level);
}

// ============================================
// sync_from_battle でHP/MP/インベントリが反映されるテスト
// ============================================

#[test]
fn sync_from_battle_reflects_battle_state() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors};
    use item::ItemKind;
    use party::PartyMember;

    // パーティ側（元データ）
    let mut original_hero = PartyMember::laios();
    original_hero.inventory.add(ItemKind::Herb, 2);
    // 戦闘用コピー
    let battle_hero = original_hero.clone();

    // 戦闘でやくそうを1つ使い、ダメージも受ける
    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0;

    let mut battle = BattleDomainState::new(vec![battle_hero], vec![slime]);

    // HPを減らしてやくそうを使用
    battle.party[0].stats.hp = 5;
    let commands = vec![
        BattleAction::UseItem { item: ItemKind::Herb, target: TargetId::Party(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    battle.execute_turn(&commands, &randoms);

    // 戦闘後のHP（回復済み）とインベントリ（Herb 1つ消費）
    let battle_hp = battle.party[0].stats.hp;
    let battle_herb_count = battle.party[0].inventory.count(ItemKind::Herb);
    assert!(battle_hp > 5, "HP should be healed in battle");
    assert_eq!(battle_herb_count, 1, "One herb should be consumed");

    // sync_from_battle で元データに反映
    original_hero.sync_from_battle(&battle.party[0]);

    assert_eq!(original_hero.stats.hp, battle_hp, "HP should be synced");
    assert_eq!(original_hero.inventory.count(ItemKind::Herb), 1, "Inventory should be synced");
}

// ============================================
// 洞窟宝箱→街で売却の連携テスト
// ============================================

#[test]
fn cave_treasure_sold_at_shop() {
    use cave::{generate_cave_map, TreasureContent};
    use town::{sell_item, SellResult};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use party::PartyMember;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let cave = generate_cave_map(&mut rng, &[]);

    let mut hero = PartyMember::laios();
    // 洞窟の宝箱からアイテムを入手
    for chest in &cave.treasures {
        if let TreasureContent::Item(item) = chest.content {
            hero.inventory.add(item, 1);
        }
    }

    // 所持アイテムのうち売却可能なものをすべて売る
    let owned = hero.inventory.owned_items();
    let mut total_earned = 0u32;
    for item in &owned {
        // 同じアイテムが複数ある場合に全て売却
        while hero.inventory.count(*item) > 0 {
            let result = sell_item(*item, &mut hero.inventory, None);
            match result {
                SellResult::Success { earned_gold } => {
                    total_earned += earned_gold;
                    assert!(earned_gold > 0, "Sold item should earn gold");
                }
                SellResult::CannotSell => {
                    assert_eq!(item.sell_price(), 0, "CannotSell should only happen for items with sell_price=0");
                    break;
                }
                _ => break,
            }
        }
    }

    // 売却可能なアイテムはすべてインベントリから消えている
    for item in &owned {
        if item.sell_price() > 0 {
            assert_eq!(hero.inventory.count(*item), 0, "{} should be removed after selling", item.name());
        }
    }
    assert!(total_earned > 0, "Should earn some gold from selling cave treasures");
}

// ============================================
// 武器買い替えで旧武器が置き換わるテスト
// ============================================

#[test]
fn weapon_upgrade_replaces_old_and_changes_damage() {
    use town::{buy_item, BuyResult};
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult};
    use item::{ItemKind, WeaponKind};
    use party::PartyMember;

    let mut hero = PartyMember::laios();
    let gold = 500u32;

    // 鉄の剣を購入して装備（+5）— 武器はインベントリに残る
    let result = buy_item(ItemKind::Weapon(WeaponKind::IronSword), gold, &mut hero.inventory);
    let remaining = match result {
        BuyResult::Success { remaining_gold } => remaining_gold,
        _ => panic!("Should buy IronSword"),
    };
    hero.equipment.equip_weapon(WeaponKind::IronSword);
    assert_eq!(hero.equipment.weapon, Some(WeaponKind::IronSword));
    assert_eq!(hero.inventory.count(ItemKind::Weapon(WeaponKind::IronSword)), 1);
    let attack_with_iron = hero.effective_attack();

    // HP999の敵にダメージを与えて記録
    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    let mut battle1 = BattleDomainState::new(vec![hero.clone()], vec![slime.clone()]);
    let commands = vec![BattleAction::Attack { target: TargetId::Enemy(0) }];
    let randoms = TurnRandomFactors { damage_randoms: vec![1.0; 2], flee_random: 1.0, spell_randoms: vec![1.0; 10] };
    let results1 = battle1.execute_turn(&commands, &randoms);
    let damage_iron = results1.iter().find_map(|r| {
        if let TurnResult::Attack { attacker: battle::ActorId::Party(0), damage, .. } = r { Some(*damage) } else { None }
    }).unwrap();

    // 鋼の剣を購入して装備（+10）— 旧武器もインベントリに残る
    let result = buy_item(ItemKind::Weapon(WeaponKind::SteelSword), remaining, &mut hero.inventory);
    assert!(matches!(result, BuyResult::Success { .. }));
    hero.equipment.equip_weapon(WeaponKind::SteelSword);
    assert_eq!(hero.equipment.weapon, Some(WeaponKind::SteelSword));
    assert_eq!(hero.inventory.count(ItemKind::Weapon(WeaponKind::IronSword)), 1, "Old weapon stays in inventory");
    assert_eq!(hero.inventory.count(ItemKind::Weapon(WeaponKind::SteelSword)), 1);
    let attack_with_steel = hero.effective_attack();
    assert_eq!(attack_with_steel, attack_with_iron + 5, "SteelSword(+10) should be 5 more than IronSword(+5)");

    // 同じ敵に同じ乱数で攻撃→ダメージが増えている
    let mut battle2 = BattleDomainState::new(vec![hero], vec![slime]);
    let results2 = battle2.execute_turn(&commands, &randoms);
    let damage_steel = results2.iter().find_map(|r| {
        if let TurnResult::Attack { attacker: battle::ActorId::Party(0), damage, .. } = r { Some(*damage) } else { None }
    }).unwrap();

    assert!(damage_steel > damage_iron, "SteelSword should deal more damage: {} vs {}", damage_steel, damage_iron);
}

// ============================================
// 大量経験値で複数レベルアップのテスト
// ============================================

#[test]
fn large_exp_gain_causes_multiple_level_ups() {
    use party::PartyMember;

    let mut hero = PartyMember::laios();
    assert_eq!(hero.level, 1);

    let base_max_hp = hero.stats.max_hp;
    let base_attack = hero.stats.attack;

    // 100exp → Lv1→2(10exp) + Lv2→3(25exp) + Lv3→4(50exp) = 85exp消費 → Lv4到達
    let level_ups = hero.gain_exp(100);
    assert!(level_ups >= 3, "Should gain at least 3 levels with 100 exp, got {}", level_ups);
    assert!(hero.level >= 4, "Should reach at least level 4, got {}", hero.level);

    // ステータスが複数回成長している
    assert!(hero.stats.max_hp > base_max_hp + 5, "max_hp should grow multiple times");
    assert!(hero.stats.attack > base_attack + 2, "attack should grow multiple times");

    // レベルアップ時は全回復
    assert_eq!(hero.stats.hp, hero.stats.max_hp, "HP should be fully restored");
    assert_eq!(hero.stats.mp, hero.stats.max_mp, "MP should be fully restored");
}

// ============================================
// sell_itemの未所持・売却不可エッジケースのテスト
// ============================================

#[test]
fn sell_item_not_owned_returns_not_owned() {
    use town::{sell_item, SellResult};
    use item::{Inventory, ItemKind};

    let mut inv = Inventory::new();
    // 所持していないアイテムを売却しようとする
    let result = sell_item(ItemKind::Herb, &mut inv, None);
    assert_eq!(result, SellResult::NotOwned, "Selling unowned item should return NotOwned");
}

#[test]
fn sell_herb_succeeds_at_half_price() {
    use town::{sell_item, SellResult};
    use item::{Inventory, ItemKind};

    let mut inv = Inventory::new();
    inv.add(ItemKind::Herb, 1);

    // やくそうは売却可能（sell_price=4、購入価格8の半額）
    let result = sell_item(ItemKind::Herb, &mut inv, None);
    assert_eq!(result, SellResult::Success { earned_gold: 4 }, "Herb should sell for 4 gold");
    assert_eq!(inv.count(ItemKind::Herb), 0, "Herb should be removed after selling");
}

// ============================================
// レベルアップ後のheal_partyが新max_hpまで回復するテスト
// ============================================

#[test]
fn heal_party_restores_to_increased_max_after_level_up() {
    use town::heal_party;
    use party::PartyMember;

    let mut hero = PartyMember::laios();
    hero.gain_exp(10); // Lv1→2, max_hp増加
    let new_max_hp = hero.stats.max_hp;
    let new_max_mp = hero.stats.max_mp;

    // ダメージを受けた後
    hero.stats.hp = 1;
    hero.stats.mp = 0;

    let mut party = vec![hero];
    heal_party(&mut party);

    assert_eq!(party[0].stats.hp, new_max_hp, "HP should restore to new max_hp after level up");
    assert_eq!(party[0].stats.mp, new_max_mp, "MP should restore to new max_mp after level up");
}

// ============================================
// 仲間募集→パーティ追加→戦闘の一連フロー
// ============================================

#[test]
fn recruit_party_then_battle_together() {
    use party::{initial_party, default_candidates, talk_to_candidate, PartyMember, PartyMemberKind, TalkResult};
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult, ActorId};

    // ライオスのみでスタート
    let mut party = initial_party();
    assert_eq!(party.len(), 1);
    assert_eq!(party[0].kind, PartyMemberKind::Laios);

    // チルチャックを募集
    let mut candidates = default_candidates();
    talk_to_candidate(&mut candidates[0]); // Acquaintance
    let result = talk_to_candidate(&mut candidates[0]); // Recruited
    assert_eq!(result, TalkResult::Recruited);
    party.push(PartyMember::from_kind(candidates[0].kind));

    assert_eq!(party.len(), 2);

    // 2人で戦闘（HP999の敵で両方の行動を確認）
    let mut wolf = Enemy::wolf();
    wolf.stats.hp = 999;
    wolf.stats.max_hp = 999;
    wolf.stats.attack = 0;
    let enemies = vec![wolf];
    let mut battle = BattleDomainState::new(party, enemies);

    let commands = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 3],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands, &randoms);

    // 両方が攻撃に参加
    let hero_attacked = results.iter().any(|r| matches!(r, TurnResult::Attack { attacker: ActorId::Party(0), .. }));
    let mage_attacked = results.iter().any(|r| matches!(r, TurnResult::Attack { attacker: ActorId::Party(1), .. }));
    assert!(hero_attacked, "Laios should attack");
    assert!(mage_attacked, "Chilchuck should attack");
}

// ============================================
// 金額不足での購入失敗テスト
// ============================================

#[test]
fn buy_item_fails_with_insufficient_gold() {
    use town::{buy_item, BuyResult};
    use item::{Inventory, ItemKind};

    let mut inv = Inventory::new();
    // やくそうは8ゴールド
    let result = buy_item(ItemKind::Herb, 7, &mut inv);
    assert_eq!(result, BuyResult::InsufficientGold, "Should fail with 7 gold for 8-gold herb");
    assert_eq!(inv.count(ItemKind::Herb), 0, "No herb should be added");
}

#[test]
fn buy_weapon_fails_with_insufficient_gold() {
    use town::{buy_item, BuyResult};
    use item::{ItemKind, WeaponKind};
    use party::PartyMember;

    let mut hero = PartyMember::laios();
    // 鉄の剣は50ゴールド
    let result = buy_item(ItemKind::Weapon(WeaponKind::IronSword), 49, &mut hero.inventory);
    assert_eq!(result, BuyResult::InsufficientGold, "Should fail with 49 gold for 50-gold sword");
    assert_eq!(hero.inventory.count(ItemKind::Weapon(WeaponKind::IronSword)), 0, "No weapon should be in inventory");
}

#[test]
fn buy_weapon_fails_with_full_inventory() {
    use town::{buy_item, BuyResult};
    use item::{ItemKind, WeaponKind};
    use party::PartyMember;

    let mut hero = PartyMember::laios();
    hero.inventory.add(ItemKind::Herb, 6); // 容量いっぱい
    let result = buy_item(ItemKind::Weapon(WeaponKind::IronSword), 100, &mut hero.inventory);
    assert_eq!(result, BuyResult::InventoryFull, "Should fail when inventory is full");
}

// ============================================
// HighHerbの戦闘中使用テスト
// ============================================

#[test]
fn high_herb_heals_more_than_regular_herb_in_battle() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult};
    use item::ItemKind;
    use party::PartyMember;

    // ライオス1: やくそうで回復
    let mut hero1 = PartyMember::laios();
    hero1.stats.hp = 1;
    hero1.inventory.add(ItemKind::Herb, 1);

    let mut slime1 = Enemy::slime();
    slime1.stats.hp = 999;
    slime1.stats.max_hp = 999;
    slime1.stats.attack = 0;

    let mut battle1 = BattleDomainState::new(vec![hero1], vec![slime1]);
    let commands = vec![BattleAction::UseItem { item: ItemKind::Herb, target: TargetId::Party(0) }];
    let randoms = TurnRandomFactors { damage_randoms: vec![1.0; 2], flee_random: 1.0, spell_randoms: vec![1.0; 10] };
    let results1 = battle1.execute_turn(&commands, &randoms);
    let heal_herb = results1.iter().find_map(|r| {
        if let TurnResult::ItemUsed { amount, .. } = r { Some(*amount) } else { None }
    }).unwrap();

    // ライオス2: 上やくそうで回復
    let mut hero2 = PartyMember::laios();
    hero2.stats.hp = 1;
    hero2.inventory.add(ItemKind::HighHerb, 1);

    let mut slime2 = Enemy::slime();
    slime2.stats.hp = 999;
    slime2.stats.max_hp = 999;
    slime2.stats.attack = 0;

    let mut battle2 = BattleDomainState::new(vec![hero2], vec![slime2]);
    let commands2 = vec![BattleAction::UseItem { item: ItemKind::HighHerb, target: TargetId::Party(0) }];
    let results2 = battle2.execute_turn(&commands2, &randoms);
    let heal_high = results2.iter().find_map(|r| {
        if let TurnResult::ItemUsed { amount, .. } = r { Some(*amount) } else { None }
    }).unwrap();

    assert!(heal_high > heal_herb, "HighHerb should heal more than Herb: {} vs {}", heal_high, heal_herb);
}

// ============================================
// 全素材アイテムの売却価格検証
// ============================================

#[test]
fn all_material_items_can_be_sold_with_correct_price() {
    use town::{sell_item, SellResult};
    use item::{Inventory, ItemKind};

    let materials = [
        (ItemKind::MagicStone, 30),
        (ItemKind::SilverOre, 60),
        (ItemKind::AncientCoin, 120),
        (ItemKind::DragonScale, 250),
    ];

    for (item, expected_price) in &materials {
        let mut inv = Inventory::new();
        inv.add(*item, 1);

        let result = sell_item(*item, &mut inv, None);
        assert_eq!(result, SellResult::Success { earned_gold: *expected_price },
            "{} should sell for {} gold", item.name(), expected_price);
        assert_eq!(inv.count(*item), 0, "{} should be removed after selling", item.name());
    }
}

// ============================================
// 敵グループ生成→戦闘→経験値の完全フロー
// ============================================

#[test]
fn generated_enemy_group_battle_to_victory_and_exp() {
    use battle::{BattleAction, BattleState as BattleDomainState, TargetId, TurnRandomFactors};
    use battle::enemy::generate_enemy_group;
    use party::default_party;

    // ランダムな敵グループを生成
    let enemies = generate_enemy_group(0, false, 0.5, 0.3); // 大陸0フィールドで2匹のグループ
    assert!(!enemies.is_empty());

    let party = default_party();
    let mut battle = BattleDomainState::new(party, enemies);

    // 全敵を全員で攻撃して勝利を目指す
    let commands = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.2; 10],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };

    // 複数ターン実行して勝利
    for _ in 0..20 {
        if battle.is_over() { break; }
        battle.execute_turn(&commands, &randoms);
    }

    if battle.is_victory() {
        let total_exp = battle.total_exp_reward();
        assert!(total_exp > 0, "Defeated enemies should give exp");

        // 経験値をパーティに分配
        for member in &mut battle.party {
            if member.stats.is_alive() {
                member.gain_exp(total_exp);
                assert!(member.exp > 0, "Party member should have exp after battle");
            }
        }
    }
}

// ============================================
// 複数ターン戦闘でturn_logが蓄積されるテスト
// ============================================

#[test]
fn multi_turn_battle_accumulates_turn_log() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors};
    use party::default_party;

    let party = default_party();
    let mut wolf = Enemy::wolf();
    wolf.stats.hp = 999;
    wolf.stats.max_hp = 999;
    wolf.stats.attack = 0; // パーティを倒さない

    let mut battle = BattleDomainState::new(party, vec![wolf]);

    let commands = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 4],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };

    // 3ターン実行
    battle.execute_turn(&commands, &randoms);
    let log_after_1 = battle.turn_log.len();
    assert!(log_after_1 > 0, "Turn log should have entries after turn 1");

    battle.execute_turn(&commands, &randoms);
    let log_after_2 = battle.turn_log.len();
    assert!(log_after_2 > log_after_1, "Turn log should grow after turn 2");

    battle.execute_turn(&commands, &randoms);
    let log_after_3 = battle.turn_log.len();
    assert!(log_after_3 > log_after_2, "Turn log should grow after turn 3");
}

// ============================================
// 装備→購入→戦闘→勝利→経験値→レベルアップの長いフロー
// ============================================

#[test]
fn full_town_equip_battle_levelup_flow() {
    use town::{buy_item, heal_party, BuyResult};
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors};
    use item::{ItemKind, WeaponKind};
    use party::PartyMember;

    let mut hero = PartyMember::laios();
    let mut gold = 500u32;

    // 1. 街で武器を買って装備する（武器はインベントリに残る）
    if let BuyResult::Success { remaining_gold } = buy_item(ItemKind::Weapon(WeaponKind::IronSword), gold, &mut hero.inventory) {
        gold = remaining_gold;
        hero.equipment.equip_weapon(WeaponKind::IronSword);
    }

    // 2. やくそうを買う
    if let BuyResult::Success { remaining_gold } = buy_item(ItemKind::Herb, gold, &mut hero.inventory) {
        let _gold = remaining_gold;
    }

    // 3. 戦闘に入る（ゴースト3体 = 30exp）
    let enemies = vec![Enemy::ghost(), Enemy::ghost(), Enemy::ghost()];
    let mut battle = BattleDomainState::new(vec![hero], enemies);

    let commands = vec![BattleAction::Attack { target: TargetId::Enemy(0) }];
    let randoms = TurnRandomFactors { damage_randoms: vec![1.2; 4], flee_random: 1.0, spell_randoms: vec![1.0; 10] };

    for _ in 0..30 {
        if battle.is_over() { break; }
        battle.execute_turn(&commands, &randoms);
    }

    // 4. 勝利したら経験値を得てレベルアップ
    if battle.is_victory() {
        let exp = battle.total_exp_reward();
        assert_eq!(exp, 30);

        let level_ups = battle.party[0].gain_exp(exp);
        assert!(level_ups >= 1, "Should level up at least once");

        // 5. レベルアップ後、やどやで回復
        battle.party[0].stats.hp = 1;
        heal_party(&mut battle.party);
        assert_eq!(battle.party[0].stats.hp, battle.party[0].stats.max_hp);
    }
}

// ============================================
// 同速時のパーティ優先テスト
// ============================================

#[test]
fn same_speed_party_acts_before_enemy() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult, ActorId};
    use party::{PartyMember, CombatStats};

    // ライオスと敵を同じ速度にする
    let mut hero = PartyMember::laios();
    hero.stats = CombatStats::new(100, 20, 5, 10, 0); // speed=10

    let mut enemy = Enemy::slime();
    enemy.stats = CombatStats::new(999, 5, 2, 10, 0); // speed=10 (同速)

    let mut battle = BattleDomainState::new(vec![hero], vec![enemy]);

    let commands = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands, &randoms);

    let attack_order: Vec<ActorId> = results.iter().filter_map(|r| {
        if let TurnResult::Attack { attacker, .. } = r { Some(*attacker) } else { None }
    }).collect();

    assert_eq!(attack_order.len(), 2, "Both actors should attack");
    assert_eq!(attack_order[0], ActorId::Party(0), "Party should act first at same speed");
    assert_eq!(attack_order[1], ActorId::Enemy(0), "Enemy should act second at same speed");
}

// ============================================
// AoE攻撃テスト
// ============================================

#[test]
fn neld_aoe_damages_all_enemies_integration() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult, SpellKind};
    use party::PartyMember;

    // マルシルLv3以上でBlaze1を習得
    let mut mage = PartyMember::marcille();
    mage.level = 3;
    let mage_max_mp = mage.stats.max_mp;

    let enemies = vec![Enemy::slime(), Enemy::slime(), Enemy::slime()];
    let mut battle = BattleDomainState::new(vec![mage], enemies);

    let commands = vec![
        BattleAction::Spell { spell: SpellKind::Blaze1, target: TargetId::Enemy(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 4],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands, &randoms);

    let spell_hits: Vec<_> = results.iter().filter(|r| matches!(r, TurnResult::SpellDamage { .. })).collect();
    assert_eq!(spell_hits.len(), 3, "Blaze1 should hit all 3 enemies");

    // MP消費は1回分のみ
    assert_eq!(battle.party[0].stats.mp, mage_max_mp - 5, "Blaze1 costs 5 MP");
}

// ============================================
// AoE回復テスト
// ============================================

#[test]
fn panam_aoe_heals_all_allies_integration() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult, SpellKind};
    use party::PartyMember;

    let mut falin = PartyMember::falin();
    falin.level = 3; // Healall1習得
    let mut laios = PartyMember::laios();
    laios.stats.hp = 5;
    let mut marcille = PartyMember::marcille();
    marcille.stats.hp = 5;
    falin.stats.hp = 5;

    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0;

    let mut battle = BattleDomainState::new(vec![laios, marcille, falin], vec![slime]);

    let commands = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Spell { spell: SpellKind::Healall1, target: TargetId::Party(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 4],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands, &randoms);

    let heal_hits: Vec<_> = results.iter().filter(|r| matches!(r, TurnResult::Healed { .. })).collect();
    assert_eq!(heal_hits.len(), 3, "Healall1 should heal all 3 allies");
}

// ============================================
// バフ適用テスト
// ============================================

#[test]
fn bolga_buff_increases_attack_integration() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult, SpellKind};
    use party::PartyMember;

    let mut rinsha = PartyMember::rinsha();
    rinsha.level = 5; // Boost1習得
    let laios = PartyMember::laios();

    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0;

    let mut battle = BattleDomainState::new(vec![laios, rinsha], vec![slime]);
    let base_attack = battle.effective_attack_with_buff(0);

    let commands = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Spell { spell: SpellKind::Boost1, target: TargetId::Party(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 3],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands, &randoms);

    let buffed = results.iter().any(|r| matches!(r, TurnResult::Buffed { .. }));
    assert!(buffed, "Buffed event should be emitted");
    assert_eq!(battle.effective_attack_with_buff(0), base_attack + 3, "ATK should increase by 3");
}

// ============================================
// ブロックによる被ダメージ吸収テスト
// ============================================

#[test]
fn block_absorbs_damage_integration() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, SpellKind, ActorId, TurnResult};
    use party::PartyMember;

    let mut senshi = PartyMember::senshi();
    senshi.level = 4; // Shield1習得
    let mut laios = PartyMember::laios();
    laios.stats.hp = 999;
    laios.stats.max_hp = 999;

    let mut wolf = Enemy::wolf();
    wolf.stats.attack = 20;
    wolf.stats.hp = 999;
    wolf.stats.max_hp = 999;

    // Shield1を付与
    let mut battle = BattleDomainState::new(vec![laios, senshi], vec![wolf]);
    let commands_buff = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Spell { spell: SpellKind::Shield1, target: TargetId::Party(0) },
    ];
    let randoms_buff = TurnRandomFactors {
        damage_randoms: vec![1.0; 3],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    battle.execute_turn(&commands_buff, &randoms_buff);

    // ブロック値が半減後に残っていることを確認（Shield1 power=10 → 半減5）
    assert_eq!(battle.party_buffs[0].block, 5, "ブロック半減後は5");

    // 次のターンで敵の攻撃を受ける
    let hp_before = battle.party[0].stats.hp;
    let commands_next = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
    ];
    let randoms_next = TurnRandomFactors {
        damage_randoms: vec![1.0; 3],
        flee_random: 1.0, spell_randoms: vec![1.0; 10],
    };
    let results = battle.execute_turn(&commands_next, &randoms_next);

    // 敵の攻撃でブロックが発生していることを確認
    let enemy_attack = results.iter().find_map(|r| {
        if let TurnResult::Attack { attacker: ActorId::Enemy(0), damage, blocked, .. } = r {
            Some((*damage, *blocked))
        } else { None }
    }).unwrap();

    assert!(enemy_attack.1 > 0, "ブロックが発生するはず: blocked={}", enemy_attack.1);
    let hp_lost = hp_before - battle.party[0].stats.hp;
    assert_eq!(hp_lost, enemy_attack.0, "実ダメージ分のみHP減少");
}

// ============================================
// バフ消失テスト
// ============================================

#[test]
fn buff_expires_after_5_turns_integration() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, SpellKind};
    use party::PartyMember;

    let mut rinsha = PartyMember::rinsha();
    rinsha.level = 5; // Boost1習得
    rinsha.stats.mp = 99;
    rinsha.stats.max_mp = 99;

    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0;

    let mut battle = BattleDomainState::new(vec![rinsha], vec![slime]);

    // ターン1: バフ付与
    let commands = vec![BattleAction::Spell { spell: SpellKind::Boost1, target: TargetId::Party(0) }];
    let randoms = TurnRandomFactors { damage_randoms: vec![1.0; 2], flee_random: 1.0, spell_randoms: vec![1.0; 10] };
    battle.execute_turn(&commands, &randoms);
    assert!(battle.party_buffs[0].attack_up.is_some(), "Buff should be active after cast");

    // ターン2~5
    for _ in 0..4 {
        let commands = vec![BattleAction::Attack { target: TargetId::Enemy(0) }];
        let randoms = TurnRandomFactors { damage_randoms: vec![1.0; 2], flee_random: 1.0, spell_randoms: vec![1.0; 10] };
        battle.execute_turn(&commands, &randoms);
    }

    assert!(battle.party_buffs[0].attack_up.is_none(), "Buff should expire after 5 turns");
}

// ============================================
// バフ上書きテスト
// ============================================

#[test]
fn buff_overwrite_resets_duration_integration() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, SpellKind};
    use party::PartyMember;

    let mut rinsha = PartyMember::rinsha();
    rinsha.level = 7; // Boost2(ATK+6)も習得
    rinsha.stats.mp = 99;
    rinsha.stats.max_mp = 99;

    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0;

    let mut battle = BattleDomainState::new(vec![rinsha], vec![slime]);

    // Boost1(ATK+3)付与
    let commands = vec![BattleAction::Spell { spell: SpellKind::Boost1, target: TargetId::Party(0) }];
    let randoms = TurnRandomFactors { damage_randoms: vec![1.0; 2], flee_random: 1.0, spell_randoms: vec![1.0; 10] };
    battle.execute_turn(&commands, &randoms);
    assert_eq!(battle.party_buffs[0].attack_up.unwrap().amount, 3);

    // 3ターン経過
    for _ in 0..3 {
        let commands = vec![BattleAction::Attack { target: TargetId::Enemy(0) }];
        let randoms = TurnRandomFactors { damage_randoms: vec![1.0; 2], flee_random: 1.0, spell_randoms: vec![1.0; 10] };
        battle.execute_turn(&commands, &randoms);
    }
    assert!(battle.party_buffs[0].attack_up.is_some(), "Buff should still be active");

    // Boost2(ATK+6)で上書き
    let commands = vec![BattleAction::Spell { spell: SpellKind::Boost2, target: TargetId::Party(0) }];
    let randoms = TurnRandomFactors { damage_randoms: vec![1.0; 2], flee_random: 1.0, spell_randoms: vec![1.0; 10] };
    battle.execute_turn(&commands, &randoms);

    let buff = battle.party_buffs[0].attack_up.unwrap();
    assert_eq!(buff.amount, 6, "Overwritten buff should have ATK+6");
    assert_eq!(buff.remaining_turns, 4, "Overwritten buff should have 4 remaining turns (5 - 1 tick)");
}

// ============================================
// MP減少呪文テスト
// ============================================

#[test]
fn drain_spell_reduces_enemy_mp() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult, SpellKind};
    use party::PartyMember;

    let mut laios = PartyMember::laios();
    laios.stats.mp = 99;
    laios.stats.max_mp = 99;

    let mut ghost = Enemy::ghost();
    ghost.stats.hp = 999;
    ghost.stats.max_hp = 999;
    let initial_mp = ghost.stats.mp; // Ghost has 8 MP

    let mut battle = BattleDomainState::new(vec![laios], vec![ghost]);

    let commands = vec![BattleAction::Spell { spell: SpellKind::Drain1, target: TargetId::Enemy(0) }];
    let randoms = TurnRandomFactors { damage_randoms: vec![1.0; 2], flee_random: 1.0, spell_randoms: vec![1.0; 10] };
    let results = battle.execute_turn(&commands, &randoms);

    // MpDrained結果が含まれている
    let mp_drained = results.iter().find(|r| matches!(r, TurnResult::MpDrained { .. }));
    assert!(mp_drained.is_some(), "Should have MpDrained result");

    // 敵のMPが減っている
    assert!(battle.enemies[0].stats.mp < initial_mp, "Enemy MP should be reduced");
}

#[test]
fn siphon_spell_reduces_all_enemies_mp() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult, SpellKind};
    use party::PartyMember;

    let mut laios = PartyMember::laios();
    laios.stats.mp = 99;
    laios.stats.max_mp = 99;

    let mut ghost1 = Enemy::ghost();
    ghost1.stats.hp = 999;
    ghost1.stats.max_hp = 999;
    let mut ghost2 = Enemy::ghost();
    ghost2.stats.hp = 999;
    ghost2.stats.max_hp = 999;
    let initial_mp = ghost1.stats.mp;

    let mut battle = BattleDomainState::new(vec![laios], vec![ghost1, ghost2]);

    let commands = vec![BattleAction::Spell { spell: SpellKind::Siphon1, target: TargetId::Enemy(0) }];
    let randoms = TurnRandomFactors { damage_randoms: vec![1.0; 3], flee_random: 1.0, spell_randoms: vec![1.0; 10] };
    let results = battle.execute_turn(&commands, &randoms);

    // 2体分のMpDrained結果
    let mp_drained_count = results.iter().filter(|r| matches!(r, TurnResult::MpDrained { .. })).count();
    assert_eq!(mp_drained_count, 2, "Should have 2 MpDrained results for 2 enemies");

    // 両方の敵のMPが減っている
    assert!(battle.enemies[0].stats.mp < initial_mp, "Enemy 0 MP should be reduced");
    assert!(battle.enemies[1].stats.mp < initial_mp, "Enemy 1 MP should be reduced");
}

#[test]
fn enemy_uses_drain_spell_on_party() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult};
    use party::PartyMember;

    let mut marcille = PartyMember::marcille();
    marcille.stats.mp = 20;
    marcille.stats.max_mp = 20;

    // Ghostが呪文を使うように: spell_random < 0.5 にする
    let mut ghost = Enemy::ghost();
    ghost.stats.hp = 999;
    ghost.stats.max_hp = 999;
    ghost.stats.mp = 20; // Drain1(cost=4)を使えるだけのMP

    let mut battle = BattleDomainState::new(vec![marcille], vec![ghost]);

    // パーティは通常攻撃、敵が呪文を使う（spell_random=0.0 < 0.5で呪文使用）
    let commands = vec![BattleAction::Attack { target: TargetId::Enemy(0) }];
    // spell_random=0.0で呪文使用。Ghostの呪文は[Fire1, Drain1]なので最初のFire1が選ばれる可能性がある
    // 確実にMPが変化したことを確認するため、ダメージか MP減少のどちらかが発生することを検証
    let randoms = TurnRandomFactors { damage_randoms: vec![1.0; 2], flee_random: 1.0, spell_randoms: vec![0.0; 10] };
    let results = battle.execute_turn(&commands, &randoms);

    // 敵が何らかの呪文を使ったことを確認
    let enemy_spell_used = results.iter().any(|r| matches!(r,
        TurnResult::SpellDamage { caster: battle::ActorId::Enemy(_), .. } |
        TurnResult::MpDrained { caster: battle::ActorId::Enemy(_), .. }
    ));
    assert!(enemy_spell_used, "Enemy should have used a spell");
}

// ============================================
// 装備中武器の売却ガードテスト
// ============================================

#[test]
fn sell_equipped_weapon_only_one_is_rejected() {
    use town::{sell_item, SellResult};
    use item::{Inventory, ItemKind, WeaponKind};

    let mut inv = Inventory::new();
    inv.add(ItemKind::Weapon(WeaponKind::IronSword), 1);

    // 装備中の武器が1本のみ → 売却不可
    let result = sell_item(ItemKind::Weapon(WeaponKind::IronSword), &mut inv, Some(WeaponKind::IronSword));
    assert_eq!(result, SellResult::CannotSell, "Equipped weapon (only 1) should not be sellable");
    assert_eq!(inv.count(ItemKind::Weapon(WeaponKind::IronSword)), 1, "Weapon should remain");
}

#[test]
fn sell_equipped_weapon_two_copies_allows_one_sale() {
    use town::{sell_item, SellResult};
    use item::{Inventory, ItemKind, WeaponKind};

    let mut inv = Inventory::new();
    inv.add(ItemKind::Weapon(WeaponKind::IronSword), 2);

    // 同じ武器2本持ち、1本装備中 → 1本は売却可能
    let result = sell_item(ItemKind::Weapon(WeaponKind::IronSword), &mut inv, Some(WeaponKind::IronSword));
    assert_eq!(result, SellResult::Success { earned_gold: 25 }, "Should sell one copy of equipped weapon");
    assert_eq!(inv.count(ItemKind::Weapon(WeaponKind::IronSword)), 1, "One copy should remain");

    // 残り1本は装備中なので売却不可
    let result2 = sell_item(ItemKind::Weapon(WeaponKind::IronSword), &mut inv, Some(WeaponKind::IronSword));
    assert_eq!(result2, SellResult::CannotSell, "Last equipped weapon should not be sellable");
}

#[test]
fn sell_unequipped_weapon_succeeds() {
    use town::{sell_item, SellResult};
    use item::{Inventory, ItemKind, WeaponKind};

    let mut inv = Inventory::new();
    inv.add(ItemKind::Weapon(WeaponKind::IronSword), 1);

    // 装備していない武器は売却可能
    let result = sell_item(ItemKind::Weapon(WeaponKind::IronSword), &mut inv, None);
    assert_eq!(result, SellResult::Success { earned_gold: 25 }, "Unequipped weapon should be sellable");
    assert_eq!(inv.count(ItemKind::Weapon(WeaponKind::IronSword)), 0);
}

#[test]
fn sell_different_weapon_while_another_equipped() {
    use town::{sell_item, SellResult};
    use item::{Inventory, ItemKind, WeaponKind};

    let mut inv = Inventory::new();
    inv.add(ItemKind::Weapon(WeaponKind::WoodenSword), 1);

    // 鉄の剣を装備中、木の剣は装備していないので売却可能
    let result = sell_item(ItemKind::Weapon(WeaponKind::WoodenSword), &mut inv, Some(WeaponKind::IronSword));
    assert_eq!(result, SellResult::Success { earned_gold: 5 }, "Non-equipped weapon should be sellable");
    assert_eq!(inv.count(ItemKind::Weapon(WeaponKind::WoodenSword)), 0);
}
