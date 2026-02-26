use spell::SpellKind;
use party::CombatStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    Slime,
    Bat,
    Goblin,
    Wolf,
    Ghost,
    Scorpion,
    Skeleton,
    Lizardman,
    Golem,
    Demon,
    Dragon,
    Wraith,
    DarkLord,
}

impl EnemyKind {
    pub fn name(self) -> &'static str {
        match self {
            EnemyKind::Slime => "スライム",
            EnemyKind::Bat => "コウモリ",
            EnemyKind::Goblin => "ゴブリン",
            EnemyKind::Wolf => "おおかみ",
            EnemyKind::Ghost => "ゴースト",
            EnemyKind::Scorpion => "サソリ",
            EnemyKind::Skeleton => "スケルトン",
            EnemyKind::Lizardman => "リザードマン",
            EnemyKind::Golem => "ゴーレム",
            EnemyKind::Demon => "デーモン",
            EnemyKind::Dragon => "ドラゴン",
            EnemyKind::Wraith => "レイス",
            EnemyKind::DarkLord => "まおう",
        }
    }

    pub fn sprite_path(self) -> &'static str {
        match self {
            EnemyKind::Slime => "enemies/slime.png",
            EnemyKind::Bat => "enemies/bat.png",
            EnemyKind::Goblin => "enemies/goblin.png",
            EnemyKind::Wolf => "enemies/wolf.png",
            EnemyKind::Ghost => "enemies/ghost.png",
            EnemyKind::Scorpion => "enemies/scorpion.png",
            EnemyKind::Skeleton => "enemies/skeleton.png",
            EnemyKind::Lizardman => "enemies/lizardman.png",
            EnemyKind::Golem => "enemies/golem.png",
            EnemyKind::Demon => "enemies/demon.png",
            EnemyKind::Dragon => "enemies/dragon.png",
            EnemyKind::Wraith => "enemies/wraith.png",
            EnemyKind::DarkLord => "enemies/dark_lord.png",
        }
    }

    /// Tier 1 の基本経験値
    pub fn base_exp_reward(self) -> u32 {
        match self {
            EnemyKind::Slime => 3,
            EnemyKind::Bat => 4,
            EnemyKind::Goblin => 6,
            EnemyKind::Wolf => 8,
            EnemyKind::Scorpion => 7,
            EnemyKind::Skeleton => 10,
            EnemyKind::Ghost => 10,
            EnemyKind::Lizardman => 12,
            EnemyKind::Golem => 15,
            EnemyKind::Demon => 18,
            EnemyKind::Dragon => 25,
            EnemyKind::Wraith => 20,
            EnemyKind::DarkLord => 100,
        }
    }

    /// 後方互換: Tier 1 の経験値
    pub fn exp_reward(self) -> u32 {
        self.base_exp_reward()
    }

    /// 使用可能な呪文テーブル
    pub fn spells(self) -> &'static [SpellKind] {
        match self {
            EnemyKind::Ghost => &[SpellKind::Fire1, SpellKind::Drain1, SpellKind::Sleep1],
            EnemyKind::Demon => &[SpellKind::Fire1, SpellKind::Blaze1],
            EnemyKind::Wraith => &[SpellKind::Fire2, SpellKind::Blaze1, SpellKind::Drain2, SpellKind::Sleepall1],
            EnemyKind::Dragon => &[SpellKind::Blaze2],
            EnemyKind::DarkLord => &[SpellKind::Blaze2, SpellKind::Fire2, SpellKind::Heal2, SpellKind::Siphon2, SpellKind::Poisonall1],
            _ => &[],
        }
    }

    /// Tier 1 の基本ステータス (max_hp, attack, defense, speed, max_mp)
    fn base_stats(self) -> (i32, i32, i32, i32, i32) {
        match self {
            EnemyKind::Slime => (8, 2, 1, 3, 0),
            EnemyKind::Bat => (6, 3, 0, 6, 0),
            EnemyKind::Goblin => (15, 5, 2, 3, 0),
            EnemyKind::Wolf => (12, 7, 1, 5, 0),
            EnemyKind::Scorpion => (14, 6, 3, 4, 0),
            EnemyKind::Skeleton => (18, 8, 4, 3, 0),
            EnemyKind::Ghost => (20, 4, 3, 2, 8),
            EnemyKind::Lizardman => (22, 9, 5, 4, 0),
            EnemyKind::Golem => (35, 10, 8, 1, 0),
            EnemyKind::Demon => (28, 12, 6, 5, 15),
            EnemyKind::Dragon => (40, 15, 10, 3, 20),
            EnemyKind::Wraith => (30, 11, 7, 6, 18),
            EnemyKind::DarkLord => (200, 25, 15, 8, 50),
        }
    }
}

fn tier_multiplier(tier: u8) -> f32 {
    match tier {
        1 => 1.0,
        2 => 1.5,
        3 => 2.0,
        _ => 1.0,
    }
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub tier: u8,
    pub stats: CombatStats,
}

impl Enemy {
    /// 種類と段階を指定して敵を生成
    pub fn new(kind: EnemyKind, tier: u8) -> Self {
        let (hp, atk, def, spd, mp) = kind.base_stats();
        let m = tier_multiplier(tier);
        Self {
            kind,
            tier,
            stats: CombatStats::new(
                (hp as f32 * m).round() as i32,
                (atk as f32 * m).round() as i32,
                (def as f32 * m).round() as i32,
                (spd as f32 * m).round() as i32,
                (mp as f32 * m).round() as i32,
            ),
        }
    }

    /// 段階を考慮した経験値
    pub fn exp_reward(&self) -> u32 {
        let base = self.kind.base_exp_reward();
        match self.tier {
            1 => base,
            2 => base * 3 / 2,
            3 => base * 2,
            _ => base,
        }
    }

    /// 段階付き表示名
    pub fn display_name(&self) -> String {
        let base = self.kind.name();
        match self.tier {
            2 => format!("{}・強", base),
            3 => format!("{}・凶", base),
            _ => base.to_string(),
        }
    }

    // 後方互換コンストラクタ (Tier 1)
    pub fn slime() -> Self {
        Self::new(EnemyKind::Slime, 1)
    }
    pub fn bat() -> Self {
        Self::new(EnemyKind::Bat, 1)
    }
    pub fn goblin() -> Self {
        Self::new(EnemyKind::Goblin, 1)
    }
    pub fn wolf() -> Self {
        Self::new(EnemyKind::Wolf, 1)
    }
    pub fn ghost() -> Self {
        Self::new(EnemyKind::Ghost, 1)
    }
    pub fn dark_lord() -> Self {
        Self::new(EnemyKind::DarkLord, 1)
    }
}

// ── エンカウントテーブル ──────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct EncounterEntry {
    pub kind: EnemyKind,
    pub tier: u8,
    pub weight: u8,
}

const fn e(kind: EnemyKind, tier: u8, weight: u8) -> EncounterEntry {
    EncounterEntry { kind, tier, weight }
}

// フィールド（大陸別） — 各大陸5〜7種の敵が出現、隣接大陸と重複あり
static FIELD_CONTINENT_0: [EncounterEntry; 3] = [
    e(EnemyKind::Slime, 1, 5),
    e(EnemyKind::Bat, 1, 4),
    e(EnemyKind::Goblin, 1, 2),
];

static FIELD_CONTINENT_1: [EncounterEntry; 8] = [
    e(EnemyKind::Slime, 2, 2),
    e(EnemyKind::Bat, 1, 2),
    e(EnemyKind::Bat, 2, 2),
    e(EnemyKind::Goblin, 1, 4),
    e(EnemyKind::Goblin, 2, 2),
    e(EnemyKind::Scorpion, 1, 4),
    e(EnemyKind::Scorpion, 2, 1),
    e(EnemyKind::Wolf, 1, 1),
];

static FIELD_CONTINENT_2: [EncounterEntry; 9] = [
    e(EnemyKind::Bat, 2, 1),
    e(EnemyKind::Goblin, 2, 2),
    e(EnemyKind::Scorpion, 1, 2),
    e(EnemyKind::Scorpion, 2, 2),
    e(EnemyKind::Wolf, 1, 4),
    e(EnemyKind::Wolf, 2, 2),
    e(EnemyKind::Skeleton, 1, 4),
    e(EnemyKind::Skeleton, 2, 1),
    e(EnemyKind::Ghost, 1, 2),
];

static FIELD_CONTINENT_3: [EncounterEntry; 9] = [
    e(EnemyKind::Scorpion, 2, 1),
    e(EnemyKind::Wolf, 2, 2),
    e(EnemyKind::Skeleton, 2, 3),
    e(EnemyKind::Skeleton, 3, 1),
    e(EnemyKind::Ghost, 1, 3),
    e(EnemyKind::Ghost, 2, 2),
    e(EnemyKind::Lizardman, 1, 4),
    e(EnemyKind::Lizardman, 2, 1),
    e(EnemyKind::Golem, 1, 1),
];

static FIELD_CONTINENT_4: [EncounterEntry; 10] = [
    e(EnemyKind::Skeleton, 2, 1),
    e(EnemyKind::Skeleton, 3, 1),
    e(EnemyKind::Ghost, 2, 2),
    e(EnemyKind::Lizardman, 1, 2),
    e(EnemyKind::Lizardman, 2, 3),
    e(EnemyKind::Golem, 1, 3),
    e(EnemyKind::Golem, 2, 1),
    e(EnemyKind::Wraith, 1, 2),
    e(EnemyKind::Demon, 1, 1),
    e(EnemyKind::Dragon, 1, 1),
];

static FIELD_CONTINENT_5: [EncounterEntry; 9] = [
    e(EnemyKind::Lizardman, 2, 1),
    e(EnemyKind::Golem, 2, 2),
    e(EnemyKind::Demon, 1, 3),
    e(EnemyKind::Demon, 2, 2),
    e(EnemyKind::Dragon, 1, 3),
    e(EnemyKind::Dragon, 2, 1),
    e(EnemyKind::Wraith, 1, 3),
    e(EnemyKind::Wraith, 2, 2),
    e(EnemyKind::Ghost, 2, 1),
];

static FIELD_CONTINENT_6: [EncounterEntry; 8] = [
    e(EnemyKind::Golem, 2, 1),
    e(EnemyKind::Golem, 3, 1),
    e(EnemyKind::Demon, 2, 3),
    e(EnemyKind::Demon, 3, 2),
    e(EnemyKind::Dragon, 1, 2),
    e(EnemyKind::Dragon, 2, 2),
    e(EnemyKind::Wraith, 2, 3),
    e(EnemyKind::Wraith, 3, 2),
];

// 洞窟（大陸別 — フィールドより少し強い、種類も多め）
static CAVE_CONTINENT_0: [EncounterEntry; 6] = [
    e(EnemyKind::Slime, 1, 3),
    e(EnemyKind::Slime, 2, 2),
    e(EnemyKind::Bat, 1, 3),
    e(EnemyKind::Bat, 2, 2),
    e(EnemyKind::Goblin, 1, 3),
    e(EnemyKind::Goblin, 2, 1),
];

static CAVE_CONTINENT_1: [EncounterEntry; 8] = [
    e(EnemyKind::Bat, 2, 1),
    e(EnemyKind::Bat, 3, 1),
    e(EnemyKind::Goblin, 2, 3),
    e(EnemyKind::Goblin, 3, 1),
    e(EnemyKind::Scorpion, 2, 3),
    e(EnemyKind::Scorpion, 3, 1),
    e(EnemyKind::Wolf, 1, 2),
    e(EnemyKind::Skeleton, 1, 1),
];

static CAVE_CONTINENT_2: [EncounterEntry; 8] = [
    e(EnemyKind::Goblin, 3, 1),
    e(EnemyKind::Scorpion, 2, 1),
    e(EnemyKind::Scorpion, 3, 1),
    e(EnemyKind::Wolf, 2, 3),
    e(EnemyKind::Wolf, 3, 1),
    e(EnemyKind::Skeleton, 1, 2),
    e(EnemyKind::Skeleton, 2, 2),
    e(EnemyKind::Ghost, 1, 2),
];

static CAVE_CONTINENT_3: [EncounterEntry; 8] = [
    e(EnemyKind::Wolf, 3, 1),
    e(EnemyKind::Skeleton, 2, 2),
    e(EnemyKind::Skeleton, 3, 1),
    e(EnemyKind::Ghost, 2, 3),
    e(EnemyKind::Ghost, 3, 1),
    e(EnemyKind::Lizardman, 1, 2),
    e(EnemyKind::Lizardman, 2, 2),
    e(EnemyKind::Golem, 1, 1),
];

static CAVE_CONTINENT_4: [EncounterEntry; 8] = [
    e(EnemyKind::Ghost, 3, 1),
    e(EnemyKind::Lizardman, 2, 3),
    e(EnemyKind::Lizardman, 3, 1),
    e(EnemyKind::Golem, 1, 2),
    e(EnemyKind::Golem, 2, 2),
    e(EnemyKind::Wraith, 1, 2),
    e(EnemyKind::Wraith, 2, 1),
    e(EnemyKind::Demon, 1, 1),
];

static CAVE_CONTINENT_5: [EncounterEntry; 8] = [
    e(EnemyKind::Golem, 2, 1),
    e(EnemyKind::Golem, 3, 1),
    e(EnemyKind::Demon, 2, 3),
    e(EnemyKind::Demon, 3, 1),
    e(EnemyKind::Dragon, 1, 2),
    e(EnemyKind::Dragon, 2, 1),
    e(EnemyKind::Wraith, 2, 3),
    e(EnemyKind::Wraith, 3, 1),
];

/// 大陸IDとis_caveフラグからエンカウントテーブルを返す
pub fn encounter_table(continent_id: u8, is_cave: bool) -> &'static [EncounterEntry] {
    if is_cave {
        match continent_id {
            0 => &CAVE_CONTINENT_0,
            1 => &CAVE_CONTINENT_1,
            2 => &CAVE_CONTINENT_2,
            3 => &CAVE_CONTINENT_3,
            4 => &CAVE_CONTINENT_4,
            5 => &CAVE_CONTINENT_5,
            _ => &CAVE_CONTINENT_0,
        }
    } else {
        match continent_id {
            0 => &FIELD_CONTINENT_0,
            1 => &FIELD_CONTINENT_1,
            2 => &FIELD_CONTINENT_2,
            3 => &FIELD_CONTINENT_3,
            4 => &FIELD_CONTINENT_4,
            5 => &FIELD_CONTINENT_5,
            6 => &FIELD_CONTINENT_6,
            _ => &FIELD_CONTINENT_0,
        }
    }
}

/// テーブルから重み付きで1体選択するヘルパー
fn pick_from_table(table: &[EncounterEntry], random: f32) -> &EncounterEntry {
    let total_weight: u32 = table.iter().map(|e| e.weight as u32).sum();
    let selected = (random * total_weight as f32).min(total_weight as f32 - 1.0) as u32;

    let mut cumulative = 0u32;
    for entry in table {
        cumulative += entry.weight as u32;
        if selected < cumulative {
            return entry;
        }
    }
    &table[0]
}

/// 大陸と場所に基づいて敵グループを生成
///
/// - `continent_id`: 大陸ID (0〜6)
/// - `is_cave`: 洞窟内かどうか
/// - `count_random`: 匹数決定用乱数 (0.0〜1.0)
/// - `kind_random`: 敵種類決定用乱数 (0.0〜1.0) — 1体目の選択に使用
pub fn generate_enemy_group(
    continent_id: u8,
    is_cave: bool,
    count_random: f32,
    kind_random: f32,
) -> Vec<Enemy> {
    let table = encounter_table(continent_id, is_cave);

    let max_count: usize = match continent_id {
        0 => 2,
        1 => 3,
        _ => 4,
    };

    let count = match count_random {
        v if v < 0.3 => 1,
        v if v < 0.6 => 2,
        v if v < 0.85 => 3,
        _ => 4,
    };
    let count = count.min(max_count);

    // 各敵を独立にテーブルから選択（1回の戦闘で複数種類が出る）
    (0..count)
        .map(|i| {
            // kind_randomを各敵でずらして異なる種類が選ばれやすくする
            let r = (kind_random + i as f32 * 0.37).fract();
            let chosen = pick_from_table(table, r);
            Enemy::new(chosen.kind, chosen.tier)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_enemy_group_returns_1_to_4() {
        // 大陸2以上は最大4体
        assert_eq!(generate_enemy_group(2, false, 0.0, 0.0).len(), 1);
        assert_eq!(generate_enemy_group(2, false, 0.29, 0.0).len(), 1);
        assert_eq!(generate_enemy_group(2, false, 0.3, 0.0).len(), 2);
        assert_eq!(generate_enemy_group(2, false, 0.6, 0.0).len(), 3);
        assert_eq!(generate_enemy_group(2, false, 0.85, 0.0).len(), 4);
        assert_eq!(generate_enemy_group(2, false, 1.0, 0.0).len(), 4);
    }

    #[test]
    fn continent_0_max_2_enemies() {
        // 大陸0は最大2体
        assert_eq!(generate_enemy_group(0, false, 0.0, 0.0).len(), 1);
        assert_eq!(generate_enemy_group(0, false, 0.3, 0.0).len(), 2);
        assert_eq!(generate_enemy_group(0, false, 0.85, 0.0).len(), 2);
        assert_eq!(generate_enemy_group(0, false, 1.0, 0.0).len(), 2);
    }

    #[test]
    fn continent_1_max_3_enemies() {
        // 大陸1は最大3体
        assert_eq!(generate_enemy_group(1, false, 0.6, 0.0).len(), 3);
        assert_eq!(generate_enemy_group(1, false, 0.85, 0.0).len(), 3);
        assert_eq!(generate_enemy_group(1, false, 1.0, 0.0).len(), 3);
    }

    #[test]
    fn generate_enemy_group_continent_0_field() {
        // kind_random=0.0 → 最初のエントリ: Slime T1
        let group = generate_enemy_group(0, false, 0.0, 0.0);
        assert_eq!(group[0].kind, EnemyKind::Slime);
        assert_eq!(group[0].tier, 1);
    }

    #[test]
    fn generate_enemy_group_continent_6_field() {
        // ボス大陸のフィールドにはGolem/Demon/Dragon/Wraithが出る
        let group = generate_enemy_group(6, false, 0.0, 0.0);
        assert!(matches!(
            group[0].kind,
            EnemyKind::Golem | EnemyKind::Demon | EnemyKind::Dragon | EnemyKind::Wraith
        ));
        assert!(group[0].tier >= 1);
    }

    #[test]
    fn cave_enemies_are_higher_tier() {
        // 大陸0洞窟: Slime/Bat/Goblin T1/T2
        let cave_table = encounter_table(0, true);
        // 洞窟テーブルの最低ティアが1以上であること
        assert!(cave_table.iter().all(|e| e.tier >= 1));
        // 洞窟にはT2以上のエントリが含まれること
        assert!(cave_table.iter().any(|e| e.tier >= 2));
    }

    #[test]
    fn mixed_enemy_types_in_group() {
        // 4体生成時に異なる種類が含まれることがある
        let group = generate_enemy_group(2, false, 0.99, 0.1);
        assert_eq!(group.len(), 4);
        // 大陸2は7種のテーブルなので、4体選べば複数種が出やすい
        // （kind_randomがずれるため異なるエントリが選ばれる）
    }

    #[test]
    fn each_continent_has_at_least_5_species() {
        use std::collections::HashSet;
        for c in 0..7 {
            let table = encounter_table(c, false);
            let species: HashSet<_> = table.iter().map(|e| e.kind).collect();
            assert!(
                species.len() >= 3,
                "大陸{}のフィールドは{}種のみ（最低3種必要）",
                c,
                species.len()
            );
        }
    }

    #[test]
    fn tier_scaling_applies_correctly() {
        let t1 = Enemy::new(EnemyKind::Slime, 1);
        let t2 = Enemy::new(EnemyKind::Slime, 2);
        let t3 = Enemy::new(EnemyKind::Slime, 3);

        assert_eq!(t1.stats.max_hp, 8);
        assert_eq!(t2.stats.max_hp, 12); // 8 * 1.5
        assert_eq!(t3.stats.max_hp, 16); // 8 * 2.0

        assert_eq!(t1.stats.attack, 2);
        assert_eq!(t2.stats.attack, 3); // 2 * 1.5 = 3
        assert_eq!(t3.stats.attack, 4); // 2 * 2.0
    }

    #[test]
    fn exp_reward_scales_with_tier() {
        let t1 = Enemy::new(EnemyKind::Slime, 1);
        let t2 = Enemy::new(EnemyKind::Slime, 2);
        let t3 = Enemy::new(EnemyKind::Slime, 3);

        assert_eq!(t1.exp_reward(), 3);
        assert_eq!(t2.exp_reward(), 4); // 3 * 1.5 = 4
        assert_eq!(t3.exp_reward(), 6); // 3 * 2
    }

    #[test]
    fn display_name_includes_tier_suffix() {
        let t1 = Enemy::new(EnemyKind::Goblin, 1);
        let t2 = Enemy::new(EnemyKind::Goblin, 2);
        let t3 = Enemy::new(EnemyKind::Goblin, 3);

        assert_eq!(t1.display_name(), "ゴブリン");
        assert_eq!(t2.display_name(), "ゴブリン・強");
        assert_eq!(t3.display_name(), "ゴブリン・凶");
    }

    #[test]
    fn backward_compat_constructors() {
        let s = Enemy::slime();
        assert_eq!(s.kind, EnemyKind::Slime);
        assert_eq!(s.tier, 1);
        assert_eq!(s.stats.max_hp, 8);

        let dl = Enemy::dark_lord();
        assert_eq!(dl.kind, EnemyKind::DarkLord);
        assert_eq!(dl.stats.max_hp, 200);
    }

    #[test]
    fn each_continent_has_unique_encounter_table() {
        // 全大陸のフィールドテーブルが異なることを確認
        for c in 0..7 {
            let table = encounter_table(c, false);
            assert!(!table.is_empty(), "大陸{}のフィールドテーブルが空", c);
        }
        for c in 0..6 {
            let table = encounter_table(c, true);
            assert!(!table.is_empty(), "大陸{}の洞窟テーブルが空", c);
        }
    }
}
