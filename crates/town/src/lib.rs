use party::{Inventory, ItemKind, PartyMember, PartyMemberKind, WeaponKind};
use terrain::{Structure, MAP_HEIGHT, MAP_WIDTH};

/// 宿屋の宿泊料金
pub const INN_PRICE: u32 = 20;

/// 居酒屋の利用料金
pub const TAVERN_PRICE: u32 = 5;

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
pub enum SellResult {
    Success { earned_gold: u32 },
    CannotSell,
    NotOwned,
}

/// アイテムを売却する
///
/// `equipped_weapon` が `Some` かつ該当武器が1本のみの場合、売却不可。
pub fn sell_item(item: ItemKind, inventory: &mut Inventory, equipped_weapon: Option<WeaponKind>) -> SellResult {
    let sell_price = item.sell_price();
    if sell_price == 0 {
        return SellResult::CannotSell;
    }
    // 装備中の武器が1本のみの場合は売却不可
    if let Some(w) = item.as_weapon()
        && equipped_weapon == Some(w)
        && inventory.count(item) <= 1
    {
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
            "おれは チルチャック。わなの かいじょと\nかぎあけが とくいだ。\n200G はらえば てを かしてやるぜ。".to_string()
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
            "ナマリだ。ちからには じしんが ある。\n200G はらえば なかまに\nなってやるぞ。".to_string()
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

/// 雇用成功時のセリフ
pub fn hire_success_dialogue(kind: PartyMemberKind) -> String {
    match kind {
        PartyMemberKind::Chilchuck => {
            "チルチャックが なかまに くわわった！\n「まあ きんの きれめが\nえんの きれめだけどな」".to_string()
        }
        PartyMemberKind::Namari => {
            "ナマリが なかまに くわわった！\n「きんの きれめが えんの きれめだ。\nよろしく たのむ」".to_string()
        }
        _ => candidate_join_dialogue(kind),
    }
}

/// 同じ大陸内で仲間候補がいる街の方角を教える台詞を生成する
///
/// `candidate_towns` は (街x, 街y, 候補のkind) のリスト。
/// 最も近い街を選び、方角と仲間の名前を含む台詞を返す。
/// 候補がいなければ None を返す。
pub fn companion_hint_dialogue(
    town_x: usize,
    town_y: usize,
    candidate_towns: &[(usize, usize, PartyMemberKind)],
) -> Option<String> {
    if candidate_towns.is_empty() {
        return None;
    }

    let mut best_dist = i32::MAX;
    let mut best_dx = 0i32;
    let mut best_dy = 0i32;
    let mut best_kind = candidate_towns[0].2;

    for &(cx, cy, kind) in candidate_towns {
        let dx = torus_delta(town_x, cx, MAP_WIDTH);
        let dy = torus_delta(town_y, cy, MAP_HEIGHT);
        let dist = dx.abs() + dy.abs();
        if dist < best_dist {
            best_dist = dist;
            best_dx = dx;
            best_dy = dy;
            best_kind = kind;
        }
    }

    let dir = direction_label(best_dx, best_dy);
    let modifier = distance_modifier(best_dist);
    let name = best_kind.name();
    Some(format!(
        "{modifier} {dir}の まちに\n{name}という つわものが いるらしいぞ"
    ))
}

/// 構造物グリッド内で指定構造物への最短方向を求める
///
/// `continent_filter` が `Some((continent_map, town_continent_id))` の場合、
/// ターゲットタイルの大陸IDが `town_continent_id` と一致するもののみ探索する。
fn find_nearest_structure(
    structures: &[Vec<Structure>],
    town_x: usize,
    town_y: usize,
    target: Structure,
    continent_filter: Option<(&[Vec<Option<u8>>], u8)>,
) -> Option<(i32, i32, i32)> {
    let mut best_dist = i32::MAX;
    let mut best_dx = 0i32;
    let mut best_dy = 0i32;

    for (y, row) in structures.iter().enumerate() {
        for (x, structure) in row.iter().enumerate() {
            if *structure == target {
                // 大陸フィルタが有効な場合、同じ大陸のもののみ対象とする
                if let Some((continent_map, town_cid)) = continent_filter {
                    let tile_cid = continent_map
                        .get(y)
                        .and_then(|r| r.get(x))
                        .copied()
                        .flatten();
                    if tile_cid != Some(town_cid) {
                        continue;
                    }
                }
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
///
/// `continent_filter` が `Some` なら同じ大陸の洞窟のみ対象。
pub fn cave_hint_dialogue(
    structures: &[Vec<Structure>],
    town_x: usize,
    town_y: usize,
    continent_filter: Option<(&[Vec<Option<u8>>], u8)>,
) -> String {
    let Some((dx, dy, dist)) =
        find_nearest_structure(structures, town_x, town_y, Structure::Cave, continent_filter)
    else {
        return townsperson_dialogue().to_string();
    };

    let dir = direction_label(dx, dy);
    let modifier = distance_modifier(dist);
    format!("{modifier} {dir}のほうに\nどうくつが あるらしいぞ")
}

/// 街の位置から最寄りの祠の方角を教える台詞を生成する
///
/// `continent_filter` が `Some` なら同じ大陸の祠のみ対象。
pub fn hokora_hint_dialogue(
    structures: &[Vec<Structure>],
    town_x: usize,
    town_y: usize,
    continent_filter: Option<(&[Vec<Option<u8>>], u8)>,
) -> String {
    let Some((dx, dy, dist)) =
        find_nearest_structure(structures, town_x, town_y, Structure::Hokora, continent_filter)
    else {
        return townsperson_dialogue().to_string();
    };

    let dir = direction_label(dx, dy);
    let modifier = distance_modifier(dist);
    format!("{modifier} {dir}のほうに\nふるい ほこらが あるそうだ")
}

/// 買い取り対象の素材アイテム一覧（安い順）
pub fn bounty_eligible_items() -> [ItemKind; 4] {
    [
        ItemKind::MagicStone,
        ItemKind::SilverOre,
        ItemKind::AncientCoin,
        ItemKind::DragonScale,
    ]
}

/// 街座標から決定論的に買い取り対象アイテムを決定する
pub fn tavern_bounty_item(town_x: usize, town_y: usize) -> ItemKind {
    let items = bounty_eligible_items();
    let hash = town_x.wrapping_mul(31).wrapping_add(town_y.wrapping_mul(97));
    items[hash % items.len()]
}

/// 買い取り価格（売価×3）
pub fn bounty_buy_price(item: ItemKind) -> u32 {
    item.sell_price() * 3
}

/// 買い取り依頼の台詞（持っていない場合）
pub fn bounty_offer_dialogue(item: ItemKind) -> String {
    let price = bounty_buy_price(item);
    format!(
        "{}G はらった。\n「{}を もってきてくれ。\n{}Gで かいとるぞ」",
        TAVERN_PRICE,
        item.name(),
        price,
    )
}

/// 買い取り依頼の台詞（持っている場合）
pub fn bounty_has_item_dialogue(item: ItemKind) -> String {
    let price = bounty_buy_price(item);
    format!(
        "{}G はらった。\n「おっ {}を もっているのか！\n{}Gで かいとるぞ！」",
        TAVERN_PRICE,
        item.name(),
        price,
    )
}

/// 買い取り依頼で売却完了時のメッセージ
pub fn bounty_sold_dialogue(item: ItemKind) -> String {
    let price = bounty_buy_price(item);
    format!("{}を {}Gで かいとった！", item.name(), price)
}

/// 買い取り依頼でアイテムを売却する（売価×3）
pub fn sell_bounty_item(item: ItemKind, inventory: &mut Inventory) -> SellResult {
    if !inventory.remove_item(item) {
        return SellResult::NotOwned;
    }
    SellResult::Success {
        earned_gold: bounty_buy_price(item),
    }
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

    /// テスト用の構造物グリッドを作成するヘルパー
    fn make_structures(width: usize, height: usize) -> Vec<Vec<Structure>> {
        vec![vec![Structure::None; width]; height]
    }

    #[test]
    fn cave_hint_dialogue_cave_to_the_north() {
        let mut structures = make_structures(MAP_WIDTH, MAP_HEIGHT);
        // 街(75, 75)の北(y=85)に洞窟を配置（tile_yが大きい=北）
        structures[85][75] = Structure::Cave;
        let dialogue = cave_hint_dialogue(&structures, 75, 75, None);
        assert!(
            dialogue.contains("きた"),
            "北の洞窟が検出されるべき: {dialogue}"
        );
        assert!(dialogue.contains("どうくつ"));
    }

    #[test]
    fn cave_hint_dialogue_cave_to_the_south() {
        let mut structures = make_structures(MAP_WIDTH, MAP_HEIGHT);
        // 街(75, 75)の南(y=65)に洞窟を配置（tile_yが小さい=南）
        structures[65][75] = Structure::Cave;
        let dialogue = cave_hint_dialogue(&structures, 75, 75, None);
        assert!(
            dialogue.contains("みなみ"),
            "南の洞窟が検出されるべき: {dialogue}"
        );
    }

    #[test]
    fn cave_hint_dialogue_no_cave_fallback() {
        let structures = make_structures(MAP_WIDTH, MAP_HEIGHT);
        let dialogue = cave_hint_dialogue(&structures, 75, 75, None);
        assert_eq!(dialogue, townsperson_dialogue());
    }

    #[test]
    fn cave_hint_dialogue_wraparound_finds_closer_cave() {
        let mut structures = make_structures(MAP_WIDTH, MAP_HEIGHT);
        // 街(5, 5) — マップ端の反対側(y=145)に洞窟を配置
        // トーラスラップで距離は 10 (5+5) = 近い
        structures[MAP_HEIGHT - 5][5] = Structure::Cave;
        // 遠い洞窟(y=80)も配置
        structures[80][5] = Structure::Cave;

        let dialogue = cave_hint_dialogue(&structures, 5, 5, None);
        // ラップアラウンドでy=145は南方向（5→145 = +140だがラップで-10）
        // つまり北が近い
        assert!(
            dialogue.contains("すぐ ちかくの"),
            "ラップアラウンドで近い洞窟が選ばれるべき: {dialogue}"
        );
    }

    #[test]
    fn cave_hint_dialogue_distance_modifiers() {
        let mut structures = make_structures(MAP_WIDTH, MAP_HEIGHT);
        // 遠い洞窟（距離50以上）
        structures[30][75] = Structure::Cave;
        let dialogue = cave_hint_dialogue(&structures, 75, 75, None);
        assert!(
            dialogue.contains("はるか とおくの"),
            "遠い洞窟: {dialogue}"
        );
    }

    #[test]
    fn hokora_hint_dialogue_to_the_east() {
        let mut structures = make_structures(MAP_WIDTH, MAP_HEIGHT);
        structures[75][90] = Structure::Hokora;
        let dialogue = hokora_hint_dialogue(&structures, 75, 75, None);
        assert!(
            dialogue.contains("ひがし"),
            "東の祠が検出されるべき: {dialogue}"
        );
        assert!(dialogue.contains("ほこら"));
    }

    #[test]
    fn hokora_hint_dialogue_no_hokora_fallback() {
        let structures = make_structures(MAP_WIDTH, MAP_HEIGHT);
        let dialogue = hokora_hint_dialogue(&structures, 75, 75, None);
        assert_eq!(dialogue, townsperson_dialogue());
    }

    #[test]
    fn hokora_hint_dialogue_wraparound() {
        let mut structures = make_structures(MAP_WIDTH, MAP_HEIGHT);
        // 街(5, 5)からマップ端の反対側(x=145)に祠を配置
        structures[5][MAP_WIDTH - 5] = Structure::Hokora;
        let dialogue = hokora_hint_dialogue(&structures, 5, 5, None);
        assert!(
            dialogue.contains("すぐ ちかくの"),
            "ラップアラウンドで近い祠が選ばれるべき: {dialogue}"
        );
    }

    /// 大陸フィルタ用のcontinent_mapを作成するヘルパー
    fn make_continent_map(width: usize, height: usize) -> Vec<Vec<Option<u8>>> {
        vec![vec![None; width]; height]
    }

    #[test]
    fn cave_hint_continent_filter_same_continent() {
        let mut structures = make_structures(MAP_WIDTH, MAP_HEIGHT);
        let mut cmap = make_continent_map(MAP_WIDTH, MAP_HEIGHT);

        // 街(75, 75) = 大陸1、近い洞窟(75, 85)も大陸1
        cmap[75][75] = Some(1);
        structures[85][75] = Structure::Cave;
        cmap[85][75] = Some(1);

        let dialogue = cave_hint_dialogue(&structures, 75, 75, Some((&cmap, 1)));
        assert!(
            dialogue.contains("どうくつ"),
            "同じ大陸の洞窟が見つかるべき: {dialogue}"
        );
    }

    #[test]
    fn cave_hint_continent_filter_different_continent_excluded() {
        let mut structures = make_structures(MAP_WIDTH, MAP_HEIGHT);
        let mut cmap = make_continent_map(MAP_WIDTH, MAP_HEIGHT);

        // 街(75, 75) = 大陸1、洞窟(75, 85) = 大陸2 → 除外
        cmap[75][75] = Some(1);
        structures[85][75] = Structure::Cave;
        cmap[85][75] = Some(2);

        let dialogue = cave_hint_dialogue(&structures, 75, 75, Some((&cmap, 1)));
        assert_eq!(
            dialogue,
            townsperson_dialogue(),
            "別大陸の洞窟は除外されるべき: {dialogue}"
        );
    }

    #[test]
    fn cave_hint_continent_filter_picks_same_continent_over_closer() {
        let mut structures = make_structures(MAP_WIDTH, MAP_HEIGHT);
        let mut cmap = make_continent_map(MAP_WIDTH, MAP_HEIGHT);

        // 街(75, 75) = 大陸1
        cmap[75][75] = Some(1);
        // 近い洞窟(75, 80) = 大陸2 → 除外
        structures[80][75] = Structure::Cave;
        cmap[80][75] = Some(2);
        // 遠い洞窟(75, 30) = 大陸1 → 選ばれる
        structures[30][75] = Structure::Cave;
        cmap[30][75] = Some(1);

        let dialogue = cave_hint_dialogue(&structures, 75, 75, Some((&cmap, 1)));
        assert!(
            dialogue.contains("どうくつ"),
            "同じ大陸の遠い洞窟が選ばれるべき: {dialogue}"
        );
        assert!(
            dialogue.contains("はるか とおくの"),
            "遠い洞窟なので距離修飾語が正しいべき: {dialogue}"
        );
    }

    #[test]
    fn hokora_hint_continent_filter_different_continent_excluded() {
        let mut structures = make_structures(MAP_WIDTH, MAP_HEIGHT);
        let mut cmap = make_continent_map(MAP_WIDTH, MAP_HEIGHT);

        // 街(75, 75) = 大陸1、祠(90, 75) = 大陸2 → 除外
        cmap[75][75] = Some(1);
        structures[75][90] = Structure::Hokora;
        cmap[75][90] = Some(2);

        let dialogue = hokora_hint_dialogue(&structures, 75, 75, Some((&cmap, 1)));
        assert_eq!(
            dialogue,
            townsperson_dialogue(),
            "別大陸の祠は除外されるべき: {dialogue}"
        );
    }

    #[test]
    fn companion_hint_dialogue_nearest_town() {
        // 街(75, 75)の北東に仲間候補がいる街がある
        let candidates = vec![(85, 85, PartyMemberKind::Chilchuck)];
        let result = companion_hint_dialogue(75, 75, &candidates);
        assert!(result.is_some());
        let dialogue = result.unwrap();
        assert!(
            dialogue.contains("ほくとう"),
            "北東の仲間が検出されるべき: {dialogue}"
        );
        assert!(dialogue.contains("チルチャック"));
        assert!(dialogue.contains("つわもの"));
    }

    #[test]
    fn companion_hint_dialogue_picks_closest() {
        // 近い候補と遠い候補がある場合、近い方を選ぶ
        let candidates = vec![
            (120, 75, PartyMemberKind::Marcille),   // 遠い (東45)
            (80, 75, PartyMemberKind::Chilchuck),    // 近い (東5)
        ];
        let result = companion_hint_dialogue(75, 75, &candidates);
        let dialogue = result.unwrap();
        assert!(
            dialogue.contains("チルチャック"),
            "近い方の仲間が選ばれるべき: {dialogue}"
        );
    }

    #[test]
    fn companion_hint_dialogue_empty_returns_none() {
        let result = companion_hint_dialogue(75, 75, &[]);
        assert!(result.is_none());
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
    fn buy_weapon_item_success() {
        use party::{Inventory, WeaponKind};
        let mut inv = Inventory::new();
        let result = buy_item(ItemKind::Weapon(WeaponKind::WoodenSword), 100, &mut inv);
        assert_eq!(
            result,
            BuyResult::Success {
                remaining_gold: 90
            }
        );
        assert_eq!(inv.count(ItemKind::Weapon(WeaponKind::WoodenSword)), 1);
    }

    #[test]
    fn buy_weapon_item_insufficient_gold() {
        use party::{Inventory, WeaponKind};
        let mut inv = Inventory::new();
        let result = buy_item(ItemKind::Weapon(WeaponKind::IronSword), 10, &mut inv);
        assert_eq!(result, BuyResult::InsufficientGold);
        assert_eq!(inv.count(ItemKind::Weapon(WeaponKind::IronSword)), 0);
    }

    #[test]
    fn buy_weapon_item_inventory_full() {
        use party::{Inventory, WeaponKind};
        let mut inv = Inventory::new();
        inv.add(ItemKind::Herb, 6); // 容量いっぱい
        let result = buy_item(ItemKind::Weapon(WeaponKind::IronSword), 100, &mut inv);
        assert_eq!(result, BuyResult::InventoryFull);
    }

    #[test]
    fn bounty_buy_price_is_triple_sell_price() {
        assert_eq!(bounty_buy_price(ItemKind::MagicStone), 90);
        assert_eq!(bounty_buy_price(ItemKind::SilverOre), 180);
        assert_eq!(bounty_buy_price(ItemKind::AncientCoin), 360);
        assert_eq!(bounty_buy_price(ItemKind::DragonScale), 750);
    }

    #[test]
    fn tavern_bounty_item_deterministic() {
        // 同じ座標なら常に同じアイテム
        let item1 = tavern_bounty_item(10, 20);
        let item2 = tavern_bounty_item(10, 20);
        assert_eq!(item1, item2);
        // bounty_eligible_items に含まれる
        assert!(bounty_eligible_items().contains(&item1));
    }

    #[test]
    fn sell_bounty_item_success() {
        let mut inv = Inventory::new();
        inv.add(ItemKind::MagicStone, 1);
        let result = sell_bounty_item(ItemKind::MagicStone, &mut inv);
        assert_eq!(result, SellResult::Success { earned_gold: 90 });
        assert_eq!(inv.count(ItemKind::MagicStone), 0);
    }

    #[test]
    fn sell_bounty_item_not_owned() {
        let mut inv = Inventory::new();
        let result = sell_bounty_item(ItemKind::MagicStone, &mut inv);
        assert_eq!(result, SellResult::NotOwned);
    }
}
