/// 呪文の効果（データ付き）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpellEffect {
    Damage { base_damage: i32 },
    Heal { base_heal: i32 },
    AttackBuff { amount: i32 },
    Block { amount: i32 },
    MpDrain { base_drain: i32 },
    Ailment { success_rate: i32 },
}

/// 1呪文のデータ
#[derive(Debug, Clone, Copy)]
pub struct SpellEntry {
    pub mp_cost: i32,
    pub effect: SpellEffect,
}

/// 全呪文パラメータテーブル
#[derive(Debug, Clone)]
pub struct SpellParamTable {
    entries: Vec<SpellEntry>,
    pub poison_damage: i32,
    pub defense_divisor: f32,
}

impl SpellParamTable {
    /// 全呪文を網羅する関数からテーブルを構築（match網羅性でコンパイル時検出）
    pub fn from_fn(f: impl Fn(SpellKind) -> SpellEntry, poison_damage: i32, defense_divisor: f32) -> Self {
        let spells = all_spells();
        let entries: Vec<SpellEntry> = spells.iter().map(|&s| f(s)).collect();
        Self { entries, poison_damage, defense_divisor }
    }

    pub fn mp_cost(&self, spell: SpellKind) -> i32 {
        self.entries[spell_index(spell)].mp_cost
    }

    pub fn effect(&self, spell: SpellKind) -> &SpellEffect {
        &self.entries[spell_index(spell)].effect
    }

    /// 呪文ダメージ = (base_damage - defense/defense_divisor) × random_factor、最小1
    pub fn spell_damage(&self, base_damage: i32, defense: i32, random_factor: f32) -> i32 {
        let base = base_damage as f32 - defense as f32 / self.defense_divisor;
        let damage = (base * random_factor).round() as i32;
        damage.max(1)
    }

    /// 回復量 = base_heal × random_factor、最小1
    pub fn heal_amount(base_heal: i32, random_factor: f32) -> i32 {
        let amount = (base_heal as f32 * random_factor).round() as i32;
        amount.max(1)
    }

    /// MP減少量 = base_drain × random_factor、最小1
    pub fn mp_drain_amount(base_drain: i32, random_factor: f32) -> i32 {
        let amount = (base_drain as f32 * random_factor).round() as i32;
        amount.max(1)
    }

    /// 状態異常の成功判定
    pub fn ailment_success(success_rate: i32, random_factor: f32) -> bool {
        random_factor * 100.0 < success_rate as f32
    }

    pub fn poison_damage(&self) -> i32 {
        self.poison_damage
    }
}

/// SpellKind → all_spells() 内のインデックス
fn spell_index(spell: SpellKind) -> usize {
    ALL_SPELLS.iter().position(|&s| s == spell).expect("unknown spell")
}

/// 状態異常の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ailment {
    /// 眠り: 行動不能、攻撃を受けると解除
    Sleep,
    /// 毒: ターン終了時に固定ダメージ
    Poison,
}

impl Ailment {
    pub fn name(self) -> &'static str {
        match self {
            Ailment::Sleep => "ねむり",
            Ailment::Poison => "どく",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpellKind {
    // 単体攻撃
    Fire1,
    Fire2,
    // 全体攻撃
    Blaze1,
    Blaze2,
    // 単体回復
    Heal1,
    Heal2,
    // 全体回復
    Healall1,
    Healall2,
    // 味方単体ブロック付与
    Shield1,
    Shield2,
    // 味方全体ブロック付与
    Barrier1,
    Barrier2,
    // 味方単体ATK↑
    Boost1,
    Boost2,
    // 味方全体ATK↑
    Rally1,
    Rally2,
    // 単体MP減少
    Drain1,
    Drain2,
    // 全体MP減少
    Siphon1,
    Siphon2,
    // 単体眠り
    Sleep1,
    // 全体眠り
    Sleepall1,
    // 単体毒
    Poison1,
    // 全体毒
    Poisonall1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellTarget {
    SingleEnemy,
    AllEnemies,
    SingleAlly,
    AllAllies,
}

impl SpellKind {
    pub fn name(self) -> &'static str {
        match self {
            SpellKind::Fire1 => "Fire1",
            SpellKind::Fire2 => "Fire2",
            SpellKind::Blaze1 => "Blaze1",
            SpellKind::Blaze2 => "Blaze2",
            SpellKind::Heal1 => "Heal1",
            SpellKind::Heal2 => "Heal2",
            SpellKind::Healall1 => "Healall1",
            SpellKind::Healall2 => "Healall2",
            SpellKind::Shield1 => "Shield1",
            SpellKind::Shield2 => "Shield2",
            SpellKind::Barrier1 => "Barrier1",
            SpellKind::Barrier2 => "Barrier2",
            SpellKind::Boost1 => "Boost1",
            SpellKind::Boost2 => "Boost2",
            SpellKind::Rally1 => "Rally1",
            SpellKind::Rally2 => "Rally2",
            SpellKind::Drain1 => "Drain1",
            SpellKind::Drain2 => "Drain2",
            SpellKind::Siphon1 => "Siphon1",
            SpellKind::Siphon2 => "Siphon2",
            SpellKind::Sleep1 => "Sleep1",
            SpellKind::Sleepall1 => "Sleepall1",
            SpellKind::Poison1 => "Poison1",
            SpellKind::Poisonall1 => "Poisonall1",
        }
    }

    pub fn target_type(self) -> SpellTarget {
        match self {
            SpellKind::Fire1 | SpellKind::Fire2 => SpellTarget::SingleEnemy,
            SpellKind::Blaze1 | SpellKind::Blaze2 => SpellTarget::AllEnemies,
            SpellKind::Heal1 | SpellKind::Heal2 => SpellTarget::SingleAlly,
            SpellKind::Healall1 | SpellKind::Healall2 => SpellTarget::AllAllies,
            SpellKind::Shield1 | SpellKind::Shield2 => SpellTarget::SingleAlly,
            SpellKind::Barrier1 | SpellKind::Barrier2 => SpellTarget::AllAllies,
            SpellKind::Boost1 | SpellKind::Boost2 => SpellTarget::SingleAlly,
            SpellKind::Rally1 | SpellKind::Rally2 => SpellTarget::AllAllies,
            SpellKind::Drain1 | SpellKind::Drain2 => SpellTarget::SingleEnemy,
            SpellKind::Siphon1 | SpellKind::Siphon2 => SpellTarget::AllEnemies,
            SpellKind::Sleep1 | SpellKind::Poison1 => SpellTarget::SingleEnemy,
            SpellKind::Sleepall1 | SpellKind::Poisonall1 => SpellTarget::AllEnemies,
        }
    }

    /// 状態異常呪文が付与するAilmentを返す
    pub fn ailment(self) -> Option<Ailment> {
        match self {
            SpellKind::Sleep1 | SpellKind::Sleepall1 => Some(Ailment::Sleep),
            SpellKind::Poison1 | SpellKind::Poisonall1 => Some(Ailment::Poison),
            _ => None,
        }
    }

    /// 後方互換: target_type()から導出
    pub fn is_offensive(self) -> bool {
        matches!(
            self.target_type(),
            SpellTarget::SingleEnemy | SpellTarget::AllEnemies
        )
    }
}

static ALL_SPELLS: &[SpellKind] = &[
    SpellKind::Fire1,
    SpellKind::Fire2,
    SpellKind::Blaze1,
    SpellKind::Blaze2,
    SpellKind::Heal1,
    SpellKind::Heal2,
    SpellKind::Healall1,
    SpellKind::Healall2,
    SpellKind::Shield1,
    SpellKind::Shield2,
    SpellKind::Barrier1,
    SpellKind::Barrier2,
    SpellKind::Boost1,
    SpellKind::Boost2,
    SpellKind::Rally1,
    SpellKind::Rally2,
    SpellKind::Drain1,
    SpellKind::Drain2,
    SpellKind::Siphon1,
    SpellKind::Siphon2,
    SpellKind::Sleep1,
    SpellKind::Sleepall1,
    SpellKind::Poison1,
    SpellKind::Poisonall1,
];

/// 全呪文リストを返す
pub fn all_spells() -> Vec<SpellKind> {
    ALL_SPELLS.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_table() -> SpellParamTable {
        use SpellEffect::*;
        SpellParamTable::from_fn(|spell| match spell {
            SpellKind::Fire1 => SpellEntry { mp_cost: 3, effect: Damage { base_damage: 12 } },
            SpellKind::Fire2 => SpellEntry { mp_cost: 7, effect: Damage { base_damage: 25 } },
            SpellKind::Blaze1 => SpellEntry { mp_cost: 5, effect: Damage { base_damage: 8 } },
            SpellKind::Blaze2 => SpellEntry { mp_cost: 10, effect: Damage { base_damage: 18 } },
            SpellKind::Heal1 => SpellEntry { mp_cost: 3, effect: Heal { base_heal: 15 } },
            SpellKind::Heal2 => SpellEntry { mp_cost: 7, effect: Heal { base_heal: 40 } },
            SpellKind::Healall1 => SpellEntry { mp_cost: 6, effect: Heal { base_heal: 10 } },
            SpellKind::Healall2 => SpellEntry { mp_cost: 12, effect: Heal { base_heal: 25 } },
            SpellKind::Shield1 => SpellEntry { mp_cost: 3, effect: Block { amount: 10 } },
            SpellKind::Shield2 => SpellEntry { mp_cost: 6, effect: Block { amount: 20 } },
            SpellKind::Barrier1 => SpellEntry { mp_cost: 6, effect: Block { amount: 6 } },
            SpellKind::Barrier2 => SpellEntry { mp_cost: 10, effect: Block { amount: 12 } },
            SpellKind::Boost1 => SpellEntry { mp_cost: 3, effect: AttackBuff { amount: 3 } },
            SpellKind::Boost2 => SpellEntry { mp_cost: 6, effect: AttackBuff { amount: 6 } },
            SpellKind::Rally1 => SpellEntry { mp_cost: 6, effect: AttackBuff { amount: 2 } },
            SpellKind::Rally2 => SpellEntry { mp_cost: 10, effect: AttackBuff { amount: 4 } },
            SpellKind::Drain1 => SpellEntry { mp_cost: 4, effect: MpDrain { base_drain: 8 } },
            SpellKind::Drain2 => SpellEntry { mp_cost: 8, effect: MpDrain { base_drain: 18 } },
            SpellKind::Siphon1 => SpellEntry { mp_cost: 6, effect: MpDrain { base_drain: 5 } },
            SpellKind::Siphon2 => SpellEntry { mp_cost: 10, effect: MpDrain { base_drain: 12 } },
            SpellKind::Sleep1 => SpellEntry { mp_cost: 4, effect: Ailment { success_rate: 70 } },
            SpellKind::Sleepall1 => SpellEntry { mp_cost: 8, effect: Ailment { success_rate: 50 } },
            SpellKind::Poison1 => SpellEntry { mp_cost: 3, effect: Ailment { success_rate: 80 } },
            SpellKind::Poisonall1 => SpellEntry { mp_cost: 6, effect: Ailment { success_rate: 60 } },
        }, 3, 4.0)
    }

    #[test]
    fn spell_damage_basic() {
        let table = test_table();
        // base_damage=12, defense=4 → base = 12 - 1 = 11
        let damage = table.spell_damage(12, 4, 1.0);
        assert_eq!(damage, 11);
    }

    #[test]
    fn spell_damage_with_random() {
        let table = test_table();
        let low = table.spell_damage(12, 4, 0.8);
        let high = table.spell_damage(12, 4, 1.2);
        assert_eq!(low, 9); // 11 * 0.8 = 8.8 → 9
        assert_eq!(high, 13); // 11 * 1.2 = 13.2 → 13
    }

    #[test]
    fn spell_damage_minimum_is_one() {
        let table = test_table();
        let damage = table.spell_damage(1, 100, 0.8);
        assert_eq!(damage, 1);
    }

    #[test]
    fn heal_amount_basic() {
        let amount = SpellParamTable::heal_amount(15, 1.0);
        assert_eq!(amount, 15);
    }

    #[test]
    fn heal_amount_with_random() {
        let low = SpellParamTable::heal_amount(15, 0.8);
        let high = SpellParamTable::heal_amount(15, 1.2);
        assert_eq!(low, 12); // 15 * 0.8 = 12
        assert_eq!(high, 18); // 15 * 1.2 = 18
    }

    #[test]
    fn all_spells_returns_24() {
        assert_eq!(all_spells().len(), 24);
    }

    #[test]
    fn target_type_and_effect_consistency() {
        let table = test_table();
        for spell in all_spells() {
            match table.effect(spell) {
                SpellEffect::Damage { .. } => {
                    assert!(spell.is_offensive());
                }
                SpellEffect::Heal { .. } => {
                    assert!(!spell.is_offensive());
                }
                SpellEffect::AttackBuff { .. } | SpellEffect::Block { .. } => {
                    assert!(!spell.is_offensive());
                }
                SpellEffect::MpDrain { .. } => {
                    assert!(spell.is_offensive());
                }
                SpellEffect::Ailment { .. } => {
                    assert!(spell.is_offensive());
                    assert!(spell.ailment().is_some());
                }
            }
        }
    }

    #[test]
    fn mp_drain_basic() {
        let amount = SpellParamTable::mp_drain_amount(8, 1.0);
        assert_eq!(amount, 8);
    }

    #[test]
    fn mp_drain_with_random() {
        let low = SpellParamTable::mp_drain_amount(8, 0.8);
        let high = SpellParamTable::mp_drain_amount(8, 1.2);
        assert_eq!(low, 6); // 8 * 0.8 = 6.4 → 6
        assert_eq!(high, 10); // 8 * 1.2 = 9.6 → 10
    }

    #[test]
    fn mp_drain_minimum_is_one() {
        let amount = SpellParamTable::mp_drain_amount(1, 0.1);
        assert_eq!(amount, 1);
    }
}
