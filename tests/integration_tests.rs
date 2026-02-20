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
use world::map::generate_map;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::time::Duration;
use app_state::{BattleState, SceneState};
use battle_ui::{BattlePhase, BattleUIState};
use movement_ui::{
    Boat, MovementBlockedEvent, MovementLocked, OnBoat, Player,
    PlayerMovedEvent, TilePosition,
};
use app_state::PartyState;
use movement_ui::{ActiveMap, MovementState, TILE_SIZE};
use world_ui::{MapModeState, SpawnPosition};

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
        grid,
        width,
        height,
        origin_x,
        origin_y,
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
    app.add_message::<movement_ui::PlayerArrivedEvent>();
    app.add_message::<movement_ui::TileEnteredEvent>();

    // 本番と同じシステム登録（移動コアのみ、エンカウント除外）
    world_ui::register_exploring_movement_systems(&mut app);

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
        width: 1,
        height: 1,
        origin_x: 0.0,
        origin_y: 0.0,
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

    // cleanup_battle_sceneシステムを追加（OnExit(BattleState::Active)で実行される）
    app.add_systems(OnExit(BattleState::Active), battle_ui::cleanup_battle_scene);

    let player_entity = spawn_test_player(&mut app);

    // プレイヤーに手動でMovementLockedを付与（戦闘開始前に移動中だった想定）
    app.world_mut()
        .entity_mut(player_entity)
        .insert(MovementLocked);

    assert!(
        app.world().get::<MovementLocked>(player_entity).is_some(),
        "MovementLocked should be present before battle cleanup"
    );

    // 戦闘に入る
    app.world_mut()
        .resource_mut::<NextState<BattleState>>()
        .set(BattleState::Active);
    app.update();

    // BattleResourceを挿入（戦闘シーンセットアップの代わり）
    let party = default_party();
    let enemies = vec![Enemy::slime()];
    let (game_state, mut ui_state) = battle_ui::init_battle_resources(party, enemies, None);
    ui_state.phase = BattlePhase::CommandSelect { member_index: 0 };
    app.insert_resource(game_state);
    app.insert_resource(ui_state);

    // 戦闘終了（BattleState::Noneに戻る）→ cleanup_battle_sceneが実行される
    app.world_mut()
        .resource_mut::<NextState<BattleState>>()
        .set(BattleState::None);
    app.update();

    assert!(
        app.world().get::<MovementLocked>(player_entity).is_none(),
        "MovementLocked should be removed after battle cleanup"
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
        flee_random: 1.0,
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

    // 勇者の強力な攻撃で倒す（乱数最大）
    let commands2 = vec![
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(0) },
        BattleAction::Attack { target: TargetId::Enemy(1) },
    ];
    let randoms2 = TurnRandomFactors {
        damage_randoms: vec![1.2; 5],
        flee_random: 1.0,
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
fn mage_learns_fire_at_level_1_and_blaze_at_level_5() {
    use battle::SpellKind;
    use battle::spell::{available_spells, spells_learned_at_level};
    use party::PartyMemberKind;

    // Lv1のMageはFireを知っている
    let spells = available_spells(PartyMemberKind::Mage, 1);
    assert_eq!(spells, vec![SpellKind::Fire]);

    // Lv4まではBlazeは未習得
    let spells4 = available_spells(PartyMemberKind::Mage, 4);
    assert_eq!(spells4, vec![SpellKind::Fire]);

    // Lv5でBlazeを習得
    let learned = spells_learned_at_level(PartyMemberKind::Mage, 5);
    assert_eq!(learned, vec![SpellKind::Blaze]);

    let spells5 = available_spells(PartyMemberKind::Mage, 5);
    assert_eq!(spells5, vec![SpellKind::Fire, SpellKind::Blaze]);
}

#[test]
fn hero_learns_heal_at_level_3() {
    use battle::SpellKind;
    use battle::spell::{available_spells, spells_learned_at_level};
    use party::PartyMemberKind;

    assert!(available_spells(PartyMemberKind::Hero, 1).is_empty());
    assert!(available_spells(PartyMemberKind::Hero, 2).is_empty());

    let learned = spells_learned_at_level(PartyMemberKind::Hero, 3);
    assert_eq!(learned, vec![SpellKind::Heal]);

    let spells = available_spells(PartyMemberKind::Hero, 3);
    assert_eq!(spells, vec![SpellKind::Heal]);
}

// ============================================
// 装備がダメージ計算に影響することを検証
// ============================================

#[test]
fn equipped_weapon_increases_battle_damage() {
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult};
    use party::{PartyMember, WeaponKind};

    // 武器なしの勇者
    let hero_unarmed = PartyMember::hero();
    let unarmed_attack = hero_unarmed.effective_attack();

    // 武器装備の勇者
    let mut hero_armed = PartyMember::hero();
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
        flee_random: 1.0,
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
    use party::{PartyMember, ItemKind};

    let mut hero = PartyMember::hero();
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
        flee_random: 1.0,
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
    use party::{PartyMember, ItemKind};

    let mut hero = PartyMember::hero();
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
        flee_random: 1.0,
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
    use party::{PartyMember, ItemKind};

    let mut hero = PartyMember::hero();
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
        flee_random: 1.0,
    };
    battle.execute_turn(&commands, &randoms);

    assert!(battle.party[0].stats.hp > 1, "Herb should heal in battle");
    assert_eq!(battle.party[0].inventory.count(ItemKind::Herb), 0, "Herb consumed after use");
}

#[test]
fn buy_weapon_at_shop_then_equip_affects_battle() {
    use town::{buy_weapon, BuyWeaponResult};
    use battle::{BattleAction, BattleState as BattleDomainState, Enemy, TargetId, TurnRandomFactors, TurnResult};
    use party::{PartyMember, WeaponKind};

    let mut hero = PartyMember::hero();
    let gold = 100u32;

    // 武器購入前の攻撃力を記録
    let attack_before = hero.effective_attack();

    // 街で鉄の剣を購入
    let result = buy_weapon(WeaponKind::IronSword, gold, &mut hero);
    match result {
        BuyWeaponResult::Success { remaining_gold } => {
            assert_eq!(remaining_gold, 50); // 100 - 50 = 50
        }
        _ => panic!("Should succeed buying IronSword"),
    }

    // 攻撃力が上がっていることを確認
    let attack_after = hero.effective_attack();
    assert_eq!(attack_after, attack_before + 5, "IronSword should add 5 attack");

    // 戦闘でダメージを確認
    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;

    let mut battle = BattleDomainState::new(vec![hero], vec![slime]);
    let commands = vec![BattleAction::Attack { target: TargetId::Enemy(0) }];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0,
    };
    let results = battle.execute_turn(&commands, &randoms);

    let damage = results.iter().find_map(|r| {
        if let TurnResult::Attack { attacker: battle::ActorId::Party(0), damage, .. } = r { Some(*damage) } else { None }
    }).unwrap();

    // attack_after(13) - defense(1)/2 = 12.5 → round(12.5 * 1.0) = 13 (or 12)
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
        flee_random: 1.0,
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

    let mut party = initial_party(); // 勇者のみ
    assert_eq!(party.len(), 1);

    let mut candidates = default_candidates(); // Mage, Priest
    assert_eq!(candidates.len(), 2);

    // --- 魔法使い: 1回目の会話（初対面→知り合い） ---
    let result = talk_to_candidate(&mut candidates[0]);
    assert_eq!(result, TalkResult::BecameAcquaintance);
    assert_eq!(candidates[0].status, RecruitmentStatus::Acquaintance);

    // --- 魔法使い: 2回目の会話（知り合い→加入） ---
    let result = talk_to_candidate(&mut candidates[0]);
    assert_eq!(result, TalkResult::Recruited);
    assert_eq!(candidates[0].status, RecruitmentStatus::Recruited);

    // パーティに魔法使いを追加
    party.push(PartyMember::from_kind(candidates[0].kind));
    assert_eq!(party.len(), 2);
    assert_eq!(party[1].kind, PartyMemberKind::Mage);

    // --- 僧侶: 同様のフロー ---
    let result = talk_to_candidate(&mut candidates[1]);
    assert_eq!(result, TalkResult::BecameAcquaintance);

    let result = talk_to_candidate(&mut candidates[1]);
    assert_eq!(result, TalkResult::Recruited);

    party.push(PartyMember::from_kind(candidates[1].kind));
    assert_eq!(party.len(), 3);
    assert_eq!(party[2].kind, PartyMemberKind::Priest);

    // --- 既に加入済みの候補に再度話しかける ---
    let result = talk_to_candidate(&mut candidates[0]);
    assert_eq!(result, TalkResult::AlreadyRecruited);
}

// ============================================
// 洞窟探索シナリオテスト
// ============================================

#[test]
fn cave_exploration_scenario() {
    use cave::{generate_cave_map, try_cave_move, CaveMoveResult, CAVE_WIDTH, CAVE_HEIGHT};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let cave = generate_cave_map(&mut rng);

    // スポーン地点は梯子
    let (sx, sy) = cave.spawn_position;
    assert_eq!(cave.grid[sy][sx], Terrain::Ladder);

    // スポーン地点から歩行可能な隣接タイルを探す
    let mut can_move = false;
    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        let result = try_cave_move(sx, sy, dx, dy, &cave.grid, CAVE_WIDTH, CAVE_HEIGHT);
        if let CaveMoveResult::Moved { .. } = result {
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

    let cave1 = generate_cave_map(&mut rng1);
    let cave2 = generate_cave_map(&mut rng2);

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
    use cave::{generate_cave_map, try_cave_move, CaveMoveResult, CAVE_WIDTH, CAVE_HEIGHT};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let cave = generate_cave_map(&mut rng);
    let (sx, sy) = cave.spawn_position;

    // 斜め移動は常にブロックされる
    for (dx, dy) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let result = try_cave_move(sx, sy, dx, dy, &cave.grid, CAVE_WIDTH, CAVE_HEIGHT);
        assert_eq!(result, CaveMoveResult::Blocked, "Diagonal move ({},{}) should be blocked", dx, dy);
    }
}

// ============================================
// 洞窟宝箱→インベントリの連携テスト
// ============================================

#[test]
fn cave_treasure_adds_to_inventory() {
    use cave::{generate_cave_map, TreasureContent};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use party::PartyMember;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let cave = generate_cave_map(&mut rng);

    let mut hero = PartyMember::hero();
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
    use world::map::generate_map;
    use town::cave_hint_dialogue;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let map = generate_map(&mut rng);

    // マップ内の町を見つける
    let mut town_pos = None;
    for (y, row) in map.grid.iter().enumerate() {
        for (x, terrain) in row.iter().enumerate() {
            if *terrain == Terrain::Town {
                town_pos = Some((x, y));
                break;
            }
        }
        if town_pos.is_some() { break; }
    }

    if let Some((tx, ty)) = town_pos {
        let dialogue = cave_hint_dialogue(&map.grid, tx, ty);
        // マップに洞窟があれば方角ヒントが返る
        let has_cave = map.grid.iter().flatten().any(|t| *t == Terrain::Cave);
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
    use world::map::generate_connected_map;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    for seed in [1, 42, 100, 999, 54321] {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let map = generate_connected_map(&mut rng);

        // 町の確認
        for (y, row) in map.grid.iter().enumerate() {
            for (x, terrain) in row.iter().enumerate() {
                if *terrain == Terrain::Town {
                    // Townタイルはis_walkableであること
                    assert!(terrain.is_walkable(), "Town at ({},{}) should be walkable", x, y);
                    // TileActionがEnterTownであること
                    assert_eq!(terrain.tile_action(), terrain::TileAction::EnterTown);
                }
                if *terrain == Terrain::Cave {
                    // CaveタイルもEnterCaveアクションを持つ
                    assert_eq!(terrain.tile_action(), terrain::TileAction::EnterCave);
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
    use world::map::{generate_connected_map, detect_islands};
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
    // Mage(SPD7) > Bat(SPD6) > Hero(SPD5) > Priest(SPD4)
    let party = vec![PartyMember::hero(), PartyMember::mage(), PartyMember::priest()];

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
        flee_random: 1.0,
    };
    let results = battle.execute_turn(&commands, &randoms);

    // 攻撃の順序を確認
    let attack_order: Vec<ActorId> = results.iter().filter_map(|r| {
        if let TurnResult::Attack { attacker, .. } = r { Some(*attacker) } else { None }
    }).collect();

    // 期待: Mage(Party(1), SPD7) → Bat(Enemy(0), SPD6) → Hero(Party(0), SPD5) → Priest(Party(2), SPD4)
    assert_eq!(attack_order.len(), 4, "All 4 actors should attack");
    assert_eq!(attack_order[0], ActorId::Party(1), "Mage should attack first (SPD7)");
    assert_eq!(attack_order[1], ActorId::Enemy(0), "Bat should attack second (SPD6)");
    assert_eq!(attack_order[2], ActorId::Party(0), "Hero should attack third (SPD5)");
    assert_eq!(attack_order[3], ActorId::Party(2), "Priest should attack fourth (SPD4)");
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

    let mut mage = PartyMember::mage();
    mage.stats.mp = 0; // MP枯渇

    let mut slime = Enemy::slime();
    slime.stats.hp = 999;
    slime.stats.max_hp = 999;
    slime.stats.attack = 0;

    let mut battle = BattleDomainState::new(vec![mage], vec![slime]);

    let commands = vec![
        BattleAction::Spell { spell: SpellKind::Fire, target: TargetId::Enemy(0) },
    ];
    let randoms = TurnRandomFactors {
        damage_randoms: vec![1.0; 2],
        flee_random: 1.0,
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

    // HP1の勇者1人 vs 強い敵2体
    let mut hero = PartyMember::hero();
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
        flee_random: 1.0,
    };
    let results = battle.execute_turn(&commands, &randoms);

    assert!(battle.is_party_wiped(), "Party should be wiped");
    assert!(battle.is_over(), "Battle should be over");

    // Defeated(Party(0))が含まれる
    let hero_defeated = results.iter().any(|r| matches!(r, TurnResult::Defeated { target: TargetId::Party(0) }));
    assert!(hero_defeated, "Hero defeat should be recorded");
}

// ============================================
// インベントリ容量とよろず屋の連携テスト
// ============================================

#[test]
fn shop_rejects_purchase_when_inventory_full() {
    use town::{buy_item, BuyResult};
    use party::{ItemKind, Inventory, INVENTORY_CAPACITY};

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

    // 勇者のレベルアップ
    let mut hero = PartyMember::hero();
    let base_hp = hero.stats.max_hp;
    let base_attack = hero.stats.attack;
    let base_defense = hero.stats.defense;
    let base_speed = hero.stats.speed;
    let base_mp = hero.stats.max_mp;

    let level_ups = hero.gain_exp(10); // Lv1→2
    assert_eq!(level_ups, 1);
    assert_eq!(hero.stats.max_hp, base_hp + 5); // Hero: hp+5
    assert_eq!(hero.stats.attack, base_attack + 2); // Hero: attack+2
    assert_eq!(hero.stats.defense, base_defense + 1); // Hero: defense+1
    assert_eq!(hero.stats.speed, base_speed + 1); // Hero: speed+1
    assert_eq!(hero.stats.max_mp, base_mp + 1); // Hero: mp+1
    // レベルアップ時は全回復
    assert_eq!(hero.stats.hp, hero.stats.max_hp);
    assert_eq!(hero.stats.mp, hero.stats.max_mp);

    // 魔法使いのレベルアップ
    let mut mage = PartyMember::mage();
    let base_hp = mage.stats.max_hp;
    let base_mp = mage.stats.max_mp;

    let level_ups = mage.gain_exp(10);
    assert_eq!(level_ups, 1);
    assert_eq!(mage.stats.max_hp, base_hp + 3); // Mage: hp+3
    assert_eq!(mage.stats.max_mp, base_mp + 3); // Mage: mp+3

    // 僧侶のレベルアップ
    let mut priest = PartyMember::priest();
    let base_hp = priest.stats.max_hp;
    let base_mp = priest.stats.max_mp;

    let level_ups = priest.gain_exp(10);
    assert_eq!(level_ups, 1);
    assert_eq!(priest.stats.max_hp, base_hp + 4); // Priest: hp+4
    assert_eq!(priest.stats.max_mp, base_mp + 2); // Priest: mp+2
}

// ============================================
// 探索マップ（Fog of War）統合テスト
// ============================================

#[test]
fn exploration_map_tracks_movement_correctly() {
    use world::exploration::{ExplorationMap, TileVisibility, VIEW_RADIUS};

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
