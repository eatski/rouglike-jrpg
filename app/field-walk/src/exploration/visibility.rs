/// タイルの可視状態を表すenum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileVisibility {
    #[default]
    Unexplored, // 未探索
    Explored,   // 探索済み（一度視界に入った）
    Visible,    // 現在視界内
}

/// 視界範囲（中心からのタイル数）
/// VIEW_RADIUS = 4 で 9x9 タイルの視界範囲（中心 + 各方向4タイル）
pub const VIEW_RADIUS: usize = 4;

/// 中心座標と半径から視界内のタイル座標を計算
///
/// # Arguments
/// * `center_x` - 中心のx座標
/// * `center_y` - 中心のy座標
/// * `view_radius` - 視界半径
/// * `map_width` - マップの幅（トーラスラップ用）
/// * `map_height` - マップの高さ（トーラスラップ用）
///
/// # Returns
/// 視界内のタイル座標のベクター
pub fn calculate_visible_tiles(
    center_x: usize,
    center_y: usize,
    view_radius: usize,
    map_width: usize,
    map_height: usize,
) -> Vec<(usize, usize)> {
    let mut visible_tiles = Vec::new();

    let radius = view_radius as isize;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            // トーラスマップのラップアラウンド処理
            let x = terrain::coordinates::wrap_coordinate(center_x, dx as i32, map_width);
            let y = terrain::coordinates::wrap_coordinate(center_y, dy as i32, map_height);

            visible_tiles.push((x, y));
        }
    }

    visible_tiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_visible_tiles_basic() {
        // 5x5マップの中心 (2, 2) から半径1の視界
        let tiles = calculate_visible_tiles(2, 2, 1, 5, 5);

        // 3x3 = 9タイル
        assert_eq!(tiles.len(), 9);

        // 中心タイルが含まれているか確認
        assert!(tiles.contains(&(2, 2)));

        // 上下左右が含まれているか確認
        assert!(tiles.contains(&(2, 1))); // 上
        assert!(tiles.contains(&(2, 3))); // 下
        assert!(tiles.contains(&(1, 2))); // 左
        assert!(tiles.contains(&(3, 2))); // 右

        // 対角が含まれているか確認
        assert!(tiles.contains(&(1, 1))); // 左上
        assert!(tiles.contains(&(3, 1))); // 右上
        assert!(tiles.contains(&(1, 3))); // 左下
        assert!(tiles.contains(&(3, 3))); // 右下
    }

    #[test]
    fn test_calculate_visible_tiles_view_radius_4() {
        // VIEW_RADIUS = 4 で 9x9 タイルの視界
        let tiles = calculate_visible_tiles(10, 10, VIEW_RADIUS, 100, 100);

        // (VIEW_RADIUS * 2 + 1) ^ 2 = 9 * 9 = 81タイル
        assert_eq!(tiles.len(), 81);

        // 最も遠いタイルが含まれているか確認
        assert!(tiles.contains(&(6, 6)));   // 左上端
        assert!(tiles.contains(&(14, 6)));  // 右上端
        assert!(tiles.contains(&(6, 14)));  // 左下端
        assert!(tiles.contains(&(14, 14))); // 右下端
    }

    #[test]
    fn test_calculate_visible_tiles_wrap_left_edge() {
        // 左端 (0, 5) から半径1の視界 (10x10マップ)
        let tiles = calculate_visible_tiles(0, 5, 1, 10, 10);

        // 3x3 = 9タイル
        assert_eq!(tiles.len(), 9);

        // 左にラップして x=9 が含まれるか確認
        assert!(tiles.contains(&(9, 4))); // 左上（ラップ）
        assert!(tiles.contains(&(9, 5))); // 左（ラップ）
        assert!(tiles.contains(&(9, 6))); // 左下（ラップ）
    }

    #[test]
    fn test_calculate_visible_tiles_wrap_right_edge() {
        // 右端 (9, 5) から半径1の視界 (10x10マップ)
        let tiles = calculate_visible_tiles(9, 5, 1, 10, 10);

        // 3x3 = 9タイル
        assert_eq!(tiles.len(), 9);

        // 右にラップして x=0 が含まれるか確認
        assert!(tiles.contains(&(0, 4))); // 右上（ラップ）
        assert!(tiles.contains(&(0, 5))); // 右（ラップ）
        assert!(tiles.contains(&(0, 6))); // 右下（ラップ）
    }

    #[test]
    fn test_calculate_visible_tiles_wrap_top_edge() {
        // 上端 (5, 0) から半径1の視界 (10x10マップ)
        let tiles = calculate_visible_tiles(5, 0, 1, 10, 10);

        // 3x3 = 9タイル
        assert_eq!(tiles.len(), 9);

        // 上にラップして y=9 が含まれるか確認
        assert!(tiles.contains(&(4, 9))); // 左上（ラップ）
        assert!(tiles.contains(&(5, 9))); // 上（ラップ）
        assert!(tiles.contains(&(6, 9))); // 右上（ラップ）
    }

    #[test]
    fn test_calculate_visible_tiles_wrap_bottom_edge() {
        // 下端 (5, 9) から半径1の視界 (10x10マップ)
        let tiles = calculate_visible_tiles(5, 9, 1, 10, 10);

        // 3x3 = 9タイル
        assert_eq!(tiles.len(), 9);

        // 下にラップして y=0 が含まれるか確認
        assert!(tiles.contains(&(4, 0))); // 左下（ラップ）
        assert!(tiles.contains(&(5, 0))); // 下（ラップ）
        assert!(tiles.contains(&(6, 0))); // 右下（ラップ）
    }

    #[test]
    fn test_calculate_visible_tiles_wrap_corner() {
        // 左上角 (0, 0) から半径1の視界 (10x10マップ)
        let tiles = calculate_visible_tiles(0, 0, 1, 10, 10);

        // 3x3 = 9タイル
        assert_eq!(tiles.len(), 9);

        // 4つのコーナーすべてでラップが発生するケース
        assert!(tiles.contains(&(9, 9))); // 左上角（両方向ラップ）
        assert!(tiles.contains(&(0, 9))); // 上（y方向ラップ）
        assert!(tiles.contains(&(1, 9))); // 右上（y方向ラップ）
        assert!(tiles.contains(&(9, 0))); // 左（x方向ラップ）
        assert!(tiles.contains(&(9, 1))); // 左下（x方向ラップ）
    }

}
