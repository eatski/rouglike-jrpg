use party::{Inventory, ItemKind, PartyMember, PartyMemberKind, WeaponKind};
use terrain::{Terrain, MAP_HEIGHT, MAP_WIDTH};

#[derive(Debug, PartialEq, Eq)]
pub enum BuyResult {
    Success { remaining_gold: u32 },
    InsufficientGold,
    InventoryFull,
}

/// アイテムを購入する
pub fn buy_item(item: ItemKind, gold: u32, inventory: &mut Inventory) -> BuyResult {
    let price = item.price();
    if gold < price {
        return BuyResult::InsufficientGold;
    }
    if !inventory.can_add(1) {
        return BuyResult::InventoryFull;
    }
    inventory.add(item, 1);
    BuyResult::Success {
        remaining_gold: gold - price,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BuyWeaponResult {
    Success { remaining_gold: u32 },
    InsufficientGold,
}

/// 武器を購入して即装備する
pub fn buy_weapon(weapon: WeaponKind, gold: u32, member: &mut PartyMember) -> BuyWeaponResult {
    let price = weapon.price();
    if gold < price {
        return BuyWeaponResult::InsufficientGold;
    }
    member.equipment.equip_weapon(weapon);
    BuyWeaponResult::Success {
        remaining_gold: gold - price,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SellResult {
    Success { earned_gold: u32 },
    CannotSell,
    NotOwned,
}

/// アイテムを売却する
pub fn sell_item(item: ItemKind, inventory: &mut Inventory) -> SellResult {
    let sell_price = item.sell_price();
    if sell_price == 0 {
        return SellResult::CannotSell;
    }
    if !inventory.remove_item(item) {
        return SellResult::NotOwned;
    }
    SellResult::Success {
        earned_gold: sell_price,
    }
}

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
        (_, 1, _, true) => "きた",
        (_, -1, _, true) => "みなみ",
        (1, _, true, _) => "ひがし",
        (-1, _, true, _) => "にし",
        (1, 1, _, _) => "ほくとう",
        (-1, 1, _, _) => "ほくせい",
        (1, -1, _, _) => "なんとう",
        (-1, -1, _, _) => "なんせい",
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

/// 仲間候補との初対面セリフ
pub fn candidate_first_dialogue(kind: PartyMemberKind) -> String {
    match kind {
        PartyMemberKind::Chilchuck => {
            "おれは チルチャック。わなの かいじょと\nかぎあけが とくいだ。\nつぎの まちで まっているぜ。".to_string()
        }
        PartyMemberKind::Marcille => {
            "わたしは マルシル！\nまほうの ちからで たすけて あげる！\nつぎの まちで まっているわ。".to_string()
        }
        PartyMemberKind::Senshi => {
            "わしは センシ。りょうりなら\nまかせておけ。\nつぎの まちで まっておる。".to_string()
        }
        PartyMemberKind::Falin => {
            "わたしは ファリン。\nかいふくまほうで みんなを\nたすけたいの。つぎの まちで まってるね。".to_string()
        }
        PartyMemberKind::Izutsumi => {
            "……イヅツミだ。\nべつに いっしょに いきたい\nわけじゃないけど。つぎの まちにいる。".to_string()
        }
        PartyMemberKind::Shuro => {
            "シュローと もうす。\nこの けんで おやくに たてよう。\nつぎの まちで おまちしている。".to_string()
        }
        PartyMemberKind::Namari => {
            "ナマリだ。ちからには じしんが ある。\nつぎの まちで まっている。".to_string()
        }
        PartyMemberKind::Kabru => {
            "ぼくは カブルー。\nいろいろ かんがえるのが とくいさ。\nつぎの まちで まっているよ。".to_string()
        }
        PartyMemberKind::Rinsha => {
            "リンシャよ。けんも まほうも\nつかえるわ。つぎの まちで まってる。".to_string()
        }
        PartyMemberKind::Laios => {
            let name = kind.name();
            format!("わたしは {name}。\nいっしょに たびを しませんか？\nつぎの まちで まっています。")
        }
    }
}

/// 仲間候補の加入セリフ
pub fn candidate_join_dialogue(kind: PartyMemberKind) -> String {
    match kind {
        PartyMemberKind::Chilchuck => {
            "チルチャックが なかまに くわわった！\n「たのむから むちゃは するなよ」".to_string()
        }
        PartyMemberKind::Marcille => {
            "マルシルが なかまに くわわった！\n「まかせて！ わたしの まほうで\nふっとばすから！」".to_string()
        }
        PartyMemberKind::Senshi => {
            "センシが なかまに くわわった！\n「さて、きょうの しょくざいは\nなにが あるかな」".to_string()
        }
        PartyMemberKind::Falin => {
            "ファリンが なかまに くわわった！\n「みんなの けがは わたしが\nなおすからね」".to_string()
        }
        PartyMemberKind::Izutsumi => {
            "イヅツミが なかまに くわわった！\n「……かってに ついていくだけだ」".to_string()
        }
        PartyMemberKind::Shuro => {
            "シュローが なかまに くわわった！\n「この けんに かけて、\nおまもりいたす」".to_string()
        }
        PartyMemberKind::Namari => {
            "ナマリが なかまに くわわった！\n「よろしく たのむ」".to_string()
        }
        PartyMemberKind::Kabru => {
            "カブルーが なかまに くわわった！\n「きみたちの パーティ、\nきょうみぶかいね」".to_string()
        }
        PartyMemberKind::Rinsha => {
            "リンシャが なかまに くわわった！\n「ぜんりょくで サポートするわ」".to_string()
        }
        PartyMemberKind::Laios => {
            let name = kind.name();
            format!("{name}が なかまに くわわった！")
        }
    }
}

/// グリッド内で指定テラインへの最短方向を求める
fn find_nearest_terrain(
    grid: &[Vec<Terrain>],
    town_x: usize,
    town_y: usize,
    target: Terrain,
) -> Option<(i32, i32, i32)> {
    let mut best_dist = i32::MAX;
    let mut best_dx = 0i32;
    let mut best_dy = 0i32;

    for (y, row) in grid.iter().enumerate() {
        for (x, terrain) in row.iter().enumerate() {
            if *terrain == target {
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
        None
    } else {
        Some((best_dx, best_dy, best_dist))
    }
}

/// 街の位置から最寄りの洞窟の方角を教える台詞を生成する
pub fn cave_hint_dialogue(grid: &[Vec<Terrain>], town_x: usize, town_y: usize) -> String {
    let Some((dx, dy, dist)) = find_nearest_terrain(grid, town_x, town_y, Terrain::Cave) else {
        return townsperson_dialogue().to_string();
    };

    let dir = direction_label(dx, dy);
    let modifier = distance_modifier(dist);
    format!("{modifier} {dir}のほうに\nどうくつが あるらしいぞ")
}

/// 街の位置から最寄りの祠の方角を教える台詞を生成する
pub fn hokora_hint_dialogue(grid: &[Vec<Terrain>], town_x: usize, town_y: usize) -> String {
    let Some((dx, dy, dist)) = find_nearest_terrain(grid, town_x, town_y, Terrain::Hokora) else {
        return townsperson_dialogue().to_string();
    };

    let dir = direction_label(dx, dy);
    let modifier = distance_modifier(dist);
    format!("{modifier} {dir}のほうに\nふるい ほこらが あるそうだ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use party::default_party;

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

    /// テスト用の小さなグリッドを作成するヘルパー
    fn make_grid(width: usize, height: usize) -> Vec<Vec<Terrain>> {
        vec![vec![Terrain::Plains; width]; height]
    }

    #[test]
    fn cave_hint_dialogue_cave_to_the_north() {
        let mut grid = make_grid(MAP_WIDTH, MAP_HEIGHT);
        // 街(75, 75)の北(y=85)に洞窟を配置（tile_yが大きい=北）
        grid[85][75] = Terrain::Cave;
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
        // 街(75, 75)の南(y=65)に洞窟を配置（tile_yが小さい=南）
        grid[65][75] = Terrain::Cave;
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
    fn hokora_hint_dialogue_to_the_east() {
        let mut grid = make_grid(MAP_WIDTH, MAP_HEIGHT);
        grid[75][90] = Terrain::Hokora;
        let dialogue = hokora_hint_dialogue(&grid, 75, 75);
        assert!(
            dialogue.contains("ひがし"),
            "東の祠が検出されるべき: {dialogue}"
        );
        assert!(dialogue.contains("ほこら"));
    }

    #[test]
    fn hokora_hint_dialogue_no_hokora_fallback() {
        let grid = make_grid(MAP_WIDTH, MAP_HEIGHT);
        let dialogue = hokora_hint_dialogue(&grid, 75, 75);
        assert_eq!(dialogue, townsperson_dialogue());
    }

    #[test]
    fn hokora_hint_dialogue_wraparound() {
        let mut grid = make_grid(MAP_WIDTH, MAP_HEIGHT);
        // 街(5, 5)からマップ端の反対側(x=145)に祠を配置
        grid[5][MAP_WIDTH - 5] = Terrain::Hokora;
        let dialogue = hokora_hint_dialogue(&grid, 5, 5);
        assert!(
            dialogue.contains("すぐ ちかくの"),
            "ラップアラウンドで近い祠が選ばれるべき: {dialogue}"
        );
    }

    #[test]
    fn buy_item_success() {
        let mut inv = Inventory::new();
        let result = buy_item(ItemKind::Herb, 100, &mut inv);
        assert_eq!(result, BuyResult::Success { remaining_gold: 92 });
        assert_eq!(inv.count(ItemKind::Herb), 1);
    }

    #[test]
    fn buy_item_insufficient_gold() {
        let mut inv = Inventory::new();
        let result = buy_item(ItemKind::Herb, 5, &mut inv);
        assert_eq!(result, BuyResult::InsufficientGold);
        assert_eq!(inv.count(ItemKind::Herb), 0);
    }

    #[test]
    fn buy_item_exact_gold() {
        let mut inv = Inventory::new();
        let result = buy_item(ItemKind::Herb, 8, &mut inv);
        assert_eq!(result, BuyResult::Success { remaining_gold: 0 });
        assert_eq!(inv.count(ItemKind::Herb), 1);
    }

    #[test]
    fn buy_item_inventory_full() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::Herb, 6); // 容量いっぱい
        let result = buy_item(ItemKind::Herb, 100, &mut inv);
        assert_eq!(result, BuyResult::InventoryFull);
        assert_eq!(inv.count(ItemKind::Herb), 6);
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
        assert_eq!(direction_label(0, 10), "きた");
        assert_eq!(direction_label(0, -10), "みなみ");
        assert_eq!(direction_label(10, 0), "ひがし");
        assert_eq!(direction_label(-10, 0), "にし");
    }

    #[test]
    fn direction_label_diagonal() {
        assert_eq!(direction_label(5, 5), "ほくとう");
        assert_eq!(direction_label(-5, 5), "ほくせい");
        assert_eq!(direction_label(5, -5), "なんとう");
        assert_eq!(direction_label(-5, -5), "なんせい");
    }

    #[test]
    fn buy_weapon_success() {
        use party::WeaponKind;
        let mut member = PartyMember::laios();
        let result = buy_weapon(WeaponKind::WoodenSword, 100, &mut member);
        assert_eq!(
            result,
            BuyWeaponResult::Success {
                remaining_gold: 90
            }
        );
        assert_eq!(member.equipment.weapon, Some(WeaponKind::WoodenSword));
    }

    #[test]
    fn buy_weapon_insufficient_gold() {
        use party::WeaponKind;
        let mut member = PartyMember::laios();
        let result = buy_weapon(WeaponKind::IronSword, 10, &mut member);
        assert_eq!(result, BuyWeaponResult::InsufficientGold);
        assert_eq!(member.equipment.weapon, None);
    }

    #[test]
    fn buy_weapon_replaces_existing() {
        use party::WeaponKind;
        let mut member = PartyMember::laios();
        member.equipment.equip_weapon(WeaponKind::WoodenSword);
        let result = buy_weapon(WeaponKind::IronSword, 100, &mut member);
        assert_eq!(
            result,
            BuyWeaponResult::Success {
                remaining_gold: 50
            }
        );
        assert_eq!(member.equipment.weapon, Some(WeaponKind::IronSword));
    }
}
