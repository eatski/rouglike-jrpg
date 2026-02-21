use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use world::exploration::TileVisibility;
use terrain::{Terrain, MAP_HEIGHT, MAP_WIDTH};

use movement_ui::{ActiveMap, MAP_PIXEL_WIDTH};

use crate::map_mode::{ExplorationData, MapModeState};

/// ミニマップスプライトを識別するコンポーネント
#[derive(Component)]
pub struct MinimapSprite;

/// ミニマップテクスチャのハンドルを保持するリソース
#[derive(Resource)]
pub struct MinimapTexture {
    pub handle: Handle<Image>,
}

/// 地形タイプから色を取得
fn terrain_to_color(terrain: Terrain) -> [u8; 4] {
    match terrain {
        Terrain::Sea => [64, 64, 200, 255],        // 青
        Terrain::Plains => [100, 200, 100, 255],   // 緑
        Terrain::Forest => [34, 139, 34, 255],     // 濃い緑
        Terrain::Mountain => [139, 137, 137, 255], // グレー
        Terrain::Town => [200, 160, 60, 255],      // 金色
        Terrain::Cave => [100, 80, 60, 255],       // 暗い茶色
        Terrain::CaveWall => [60, 50, 40, 255],    // 暗い茶色
        Terrain::CaveFloor => [140, 120, 90, 255], // 薄い茶色
        Terrain::WarpZone => [180, 100, 200, 255], // 紫
        Terrain::Ladder => [200, 180, 60, 255],    // 黄色
        Terrain::Hokora => [190, 45, 40, 255],     // 赤（鳥居の色）
        Terrain::BossCave => [120, 40, 100, 255],  // 紫（ボス洞窟入口）
        Terrain::BossCaveWall => [50, 20, 55, 255], // 暗い紫
        Terrain::BossCaveFloor => [100, 65, 105, 255], // 紫系床
        Terrain::DarkPlains => [90, 70, 100, 255],  // 暗い紫緑（禍々しい平地）
        Terrain::DarkForest => [50, 30, 60, 255],   // 暗い紫（禍々しい森）
    }
}

/// 探索状態に応じて色を調整
fn apply_visibility(base_color: [u8; 4], visibility: TileVisibility) -> [u8; 4] {
    match visibility {
        TileVisibility::Visible => base_color,
        TileVisibility::Explored => {
            // 暗め（50%）
            [
                (base_color[0] as u16 * 50 / 100) as u8,
                (base_color[1] as u16 * 50 / 100) as u8,
                (base_color[2] as u16 * 50 / 100) as u8,
                255,
            ]
        }
        TileVisibility::Unexplored => [0, 0, 0, 255], // 黒
    }
}

/// ミニマップテクスチャを初期化するシステム
pub fn init_minimap_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    active_map: Res<ActiveMap>,
    exploration_data: Res<ExplorationData>,
) {
    // テクスチャデータを生成（RGBA8形式）
    let mut data = vec![0u8; MAP_WIDTH * MAP_HEIGHT * 4];

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let terrain = active_map.grid[y][x];
            let visibility = exploration_data
                .map
                .get(x, y)
                .unwrap_or(TileVisibility::Unexplored);

            let base_color = terrain_to_color(terrain);
            let final_color = apply_visibility(base_color, visibility);

            // Y座標を反転（テクスチャは上から下、ゲームは下から上）
            let tex_y = MAP_HEIGHT - 1 - y;
            let idx = (tex_y * MAP_WIDTH + x) * 4;
            data[idx] = final_color[0];
            data[idx + 1] = final_color[1];
            data[idx + 2] = final_color[2];
            data[idx + 3] = final_color[3];
        }
    }

    // Imageを作成
    let image = Image::new(
        Extent3d {
            width: MAP_WIDTH as u32,
            height: MAP_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let handle = images.add(image);

    // ミニマップスプライトを生成（初期は非表示）
    let scale = MAP_PIXEL_WIDTH / MAP_WIDTH as f32;

    commands.spawn((
        MinimapSprite,
        Sprite::from_image(handle.clone()),
        Transform::from_xyz(0.0, 0.0, 10.0).with_scale(Vec3::splat(scale)),
        Visibility::Hidden,
    ));

    commands.insert_resource(MinimapTexture { handle });
}

/// 探索状態変更時にミニマップテクスチャを更新するシステム
pub fn update_minimap_texture_system(
    exploration_data: Res<ExplorationData>,
    active_map: Res<ActiveMap>,
    minimap_texture: Res<MinimapTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    // ExplorationDataが変更された時のみ更新
    if !exploration_data.is_changed() {
        return;
    }

    let Some(image) = images.get_mut(&minimap_texture.handle) else {
        return;
    };

    let Some(ref mut data) = image.data else {
        return;
    };

    // テクスチャデータを更新
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let terrain = active_map.grid[y][x];
            let visibility = exploration_data
                .map
                .get(x, y)
                .unwrap_or(TileVisibility::Unexplored);

            let base_color = terrain_to_color(terrain);
            let final_color = apply_visibility(base_color, visibility);

            let tex_y = MAP_HEIGHT - 1 - y;
            let idx = (tex_y * MAP_WIDTH + x) * 4;
            data[idx] = final_color[0];
            data[idx + 1] = final_color[1];
            data[idx + 2] = final_color[2];
            data[idx + 3] = final_color[3];
        }
    }
}

/// マップモード切り替え時にミニマップの表示/非表示を切り替えるシステム
pub fn toggle_minimap_visibility_system(
    map_mode_state: Res<MapModeState>,
    mut minimap_query: Query<&mut Visibility, With<MinimapSprite>>,
) {
    if !map_mode_state.is_changed() {
        return;
    }

    for mut visibility in minimap_query.iter_mut() {
        *visibility = if map_mode_state.enabled {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
