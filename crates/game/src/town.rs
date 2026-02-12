use crate::battle::PartyMember;
use crate::map::{Terrain, MAP_HEIGHT, MAP_WIDTH};

/// パーティ全員のHP/MPを全回復する
pub fn heal_party(party: &mut [PartyMember]) {
    for member in party.iter_mut() {
        member.stats.hp = member.stats.max_hp;
        member.stats.mp = member.stats.max_mp;
    }
}

/// NPCの台詞を返す（フォールバック用）
pub fn townsperson_dialogue() -> &'static str {
    "このさきに まおうの しろが あるらしいぞ"
}

/// トーラスマップ上の符号付き最短距離を計算する
fn torus_delta(from: usize, to: usize, size: usize) -> i32 {
    let raw = to as i32 - from as i32;
    let half = size as i32 / 2;
    if raw > half {
        raw - size as i32
    } else if raw < -half {
        raw + size as i32
    } else {
        raw
    }
}

/// 方角文字列を返す
fn direction_label(dx: i32, dy: i32) -> &'static str {
    let ax = dx.unsigned_abs();
    let ay = dy.unsigned_abs();

    // 主軸と副軸を判定（差が2倍以上なら単一方角）
    let mostly_x = ax > ay * 2;
    let mostly_y = ay > ax * 2;

    match (dx.signum(), dy.signum(), mostly_x, mostly_y) {
        (_, -1, _, true) => "きた",
        (_, 1, _, true) => "みなみ",
        (1, _, true, _) => "ひがし",
        (-1, _, true, _) => "にし",
        (1, -1, _, _) => "ほくとう",
        (-1, -1, _, _) => "ほくせい",
        (1, 1, _, _) => "なんとう",
        (-1, 1, _, _) => "なんせい",
        _ => "ちかく",
    }
}

/// 距離感を表す修飾語を返す
fn distance_modifier(distance: i32) -> &'static str {
    if distance <= 15 {
        "すぐ ちかくの"
    } else if distance <= 40 {
        "すこし あるいた さきの"
    } else {
        "はるか とおくの"
    }
}

/// 街の位置から最寄りの洞窟の方角を教える台詞を生成する
pub fn cave_hint_dialogue(grid: &[Vec<Terrain>], town_x: usize, town_y: usize) -> String {
    let mut best_dist = i32::MAX;
    let mut best_dx = 0i32;
    let mut best_dy = 0i32;

    for (y, row) in grid.iter().enumerate() {
        for (x, terrain) in row.iter().enumerate() {
            if *terrain == Terrain::Cave {
                let dx = torus_delta(town_x, x, MAP_WIDTH);
                let dy = torus_delta(town_y, y, MAP_HEIGHT);
                let dist = dx.abs() + dy.abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_dx = dx;
                    best_dy = dy;
                }
            }
        }
    }

    if best_dist == i32::MAX {
        return townsperson_dialogue().to_string();
    }

    let dir = direction_label(best_dx, best_dy);
    let modifier = distance_modifier(best_dist);
    format!("{modifier} {dir}のほうに\nどうくつが あるらしいぞ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::default_party;

    #[test]
    fn heal_party_restores_full_hp_mp() {
        let mut party = default_party();
        // ダメージを与える
        party[0].stats.hp = 1;
        party[0].stats.mp = 0;
        party[1].stats.hp = 5;

        heal_party(&mut party);

        for member in &party {
            assert_eq!(member.stats.hp, member.stats.max_hp);
            assert_eq!(member.stats.mp, member.stats.max_mp);
        }
    }

    #[test]
    fn townsperson_dialogue_returns_non_empty() {
        let dialogue = townsperson_dialogue();
        assert!(!dialogue.is_empty());
    }

    /// テスト用の小さなグリッドを作成するヘルパー
    fn make_grid(width: usize, height: usize) -> Vec<Vec<Terrain>> {
        vec![vec![Terrain::Plains; width]; height]
    }

    #[test]
    fn cave_hint_dialogue_cave_to_the_north() {
        let mut grid = make_grid(MAP_WIDTH, MAP_HEIGHT);
        // 街(75, 75)の北(y=65)に洞窟を配置
        grid[65][75] = Terrain::Cave;
        let dialogue = cave_hint_dialogue(&grid, 75, 75);
        assert!(
            dialogue.contains("きた"),
            "北の洞窟が検出されるべき: {dialogue}"
        );
        assert!(dialogue.contains("どうくつ"));
    }

    #[test]
    fn cave_hint_dialogue_cave_to_the_south() {
        let mut grid = make_grid(MAP_WIDTH, MAP_HEIGHT);
        // 街(75, 75)の南(y=85)に洞窟を配置
        grid[85][75] = Terrain::Cave;
        let dialogue = cave_hint_dialogue(&grid, 75, 75);
        assert!(
            dialogue.contains("みなみ"),
            "南の洞窟が検出されるべき: {dialogue}"
        );
    }

    #[test]
    fn cave_hint_dialogue_no_cave_fallback() {
        let grid = make_grid(MAP_WIDTH, MAP_HEIGHT);
        let dialogue = cave_hint_dialogue(&grid, 75, 75);
        assert_eq!(dialogue, townsperson_dialogue());
    }

    #[test]
    fn cave_hint_dialogue_wraparound_finds_closer_cave() {
        let mut grid = make_grid(MAP_WIDTH, MAP_HEIGHT);
        // 街(5, 5) — マップ端の反対側(y=145)に洞窟を配置
        // トーラスラップで距離は 10 (5+5) = 近い
        grid[MAP_HEIGHT - 5][5] = Terrain::Cave;
        // 遠い洞窟(y=80)も配置
        grid[80][5] = Terrain::Cave;

        let dialogue = cave_hint_dialogue(&grid, 5, 5);
        // ラップアラウンドでy=145は南方向（5→145 = +140だがラップで-10）
        // つまり北が近い
        assert!(
            dialogue.contains("すぐ ちかくの"),
            "ラップアラウンドで近い洞窟が選ばれるべき: {dialogue}"
        );
    }

    #[test]
    fn cave_hint_dialogue_distance_modifiers() {
        let mut grid = make_grid(MAP_WIDTH, MAP_HEIGHT);
        // 遠い洞窟（距離50以上）
        grid[30][75] = Terrain::Cave;
        let dialogue = cave_hint_dialogue(&grid, 75, 75);
        assert!(
            dialogue.contains("はるか とおくの"),
            "遠い洞窟: {dialogue}"
        );
    }

    #[test]
    fn torus_delta_direct_path() {
        assert_eq!(torus_delta(10, 20, 150), 10);
        assert_eq!(torus_delta(20, 10, 150), -10);
    }

    #[test]
    fn torus_delta_wraparound_path() {
        // 0→145: 直接は+145、ラップは-5（近い）
        assert_eq!(torus_delta(0, 145, 150), -5);
        // 145→0: 直接は-145、ラップは+5（近い）
        assert_eq!(torus_delta(145, 0, 150), 5);
    }

    #[test]
    fn direction_label_cardinal() {
        assert_eq!(direction_label(0, -10), "きた");
        assert_eq!(direction_label(0, 10), "みなみ");
        assert_eq!(direction_label(10, 0), "ひがし");
        assert_eq!(direction_label(-10, 0), "にし");
    }

    #[test]
    fn direction_label_diagonal() {
        assert_eq!(direction_label(5, -5), "ほくとう");
        assert_eq!(direction_label(-5, -5), "ほくせい");
        assert_eq!(direction_label(5, 5), "なんとう");
        assert_eq!(direction_label(-5, 5), "なんせい");
    }
}
