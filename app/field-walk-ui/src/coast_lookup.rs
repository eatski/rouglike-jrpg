/// 海岸線オートタイル用ルックアップテーブル
///
/// 8隣接ビットマスクから47種類の海岸タイルへのマッピングを提供する。
///
/// ビットマスク定数: 8方位
pub const N: u8 = 1;
pub const NE: u8 = 2;
pub const E: u8 = 4;
pub const SE: u8 = 8;
pub const S: u8 = 16;
pub const SW: u8 = 32;
pub const W: u8 = 64;
pub const NW: u8 = 128;

/// 対角ビットを正規化する
///
/// 対角方位は隣接する2つのカーディナル方位が両方とも陸の場合のみ有効
fn normalize_mask(mask: u8) -> u8 {
    let mut result = mask & (N | E | S | W);
    if mask & NE != 0 && mask & N != 0 && mask & E != 0 {
        result |= NE;
    }
    if mask & SE != 0 && mask & S != 0 && mask & E != 0 {
        result |= SE;
    }
    if mask & SW != 0 && mask & S != 0 && mask & W != 0 {
        result |= SW;
    }
    if mask & NW != 0 && mask & N != 0 && mask & W != 0 {
        result |= NW;
    }
    result
}

/// ルックアップテーブルを構築する
///
/// 返り値: (lookup, tile_count)
/// - lookup[256]: 任意のマスクから正規化タイルインデックスへ
/// - tile_count: ユニークタイル数(47)
pub fn build_lookup_table() -> ([u8; 256], usize) {
    let mut lookup = [0u8; 256];
    let mut unique_masks: Vec<u8> = Vec::new();

    for raw in 0..=255u8 {
        let normalized = normalize_mask(raw);
        let index = if let Some(pos) = unique_masks.iter().position(|&m| m == normalized) {
            pos
        } else {
            let pos = unique_masks.len();
            unique_masks.push(normalized);
            pos
        };
        lookup[raw as usize] = index as u8;
    }

    let count = unique_masks.len();
    (lookup, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_tile_count_is_47() {
        let (_, count) = build_lookup_table();
        assert_eq!(count, 47);
    }

    #[test]
    fn mask_0_maps_to_index_0() {
        let (lookup, _) = build_lookup_table();
        assert_eq!(lookup[0], 0);
    }
}
