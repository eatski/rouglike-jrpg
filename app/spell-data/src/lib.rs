pub use spell::{Ailment, SpellEffect, SpellEntry, SpellTarget};
pub use spell::{ailment_success, heal_amount, mp_drain_amount, spell_damage};
pub use spell::{DEFENSE_DIVISOR, POISON_DAMAGE};

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

impl SpellKind {
    pub const fn name(self) -> &'static str {
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

    pub const fn target_type(self) -> SpellTarget {
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
    pub const fn ailment(self) -> Option<Ailment> {
        match self {
            SpellKind::Sleep1 | SpellKind::Sleepall1 => Some(Ailment::Sleep),
            SpellKind::Poison1 | SpellKind::Poisonall1 => Some(Ailment::Poison),
            _ => None,
        }
    }

    /// target_type()から導出
    pub const fn is_offensive(self) -> bool {
        matches!(
            self.target_type(),
            SpellTarget::SingleEnemy | SpellTarget::AllEnemies
        )
    }

    pub const fn mp_cost(self) -> i32 {
        self.mp_cost_value()
    }

    pub const fn effect(self) -> SpellEffect {
        self.effect_value()
    }

    pub const fn entry(self) -> SpellEntry {
        SpellEntry {
            name: self.name(),
            mp_cost: self.mp_cost_value(),
            effect: self.effect_value(),
            target_type: self.target_type(),
            ailment: self.ailment(),
        }
    }

    const fn mp_cost_value(self) -> i32 {
        self.effect_and_cost().0
    }

    const fn effect_value(self) -> SpellEffect {
        self.effect_and_cost().1
    }

    const fn effect_and_cost(self) -> (i32, SpellEffect) {
        use SpellEffect::*;
        match self {
            // 単体攻撃
            SpellKind::Fire1 => (3, Damage { base_damage: 12 }),
            SpellKind::Fire2 => (7, Damage { base_damage: 25 }),
            // 全体攻撃
            SpellKind::Blaze1 => (5, Damage { base_damage: 8 }),
            SpellKind::Blaze2 => (10, Damage { base_damage: 18 }),
            // 単体回復
            SpellKind::Heal1 => (3, Heal { base_heal: 15 }),
            SpellKind::Heal2 => (7, Heal { base_heal: 40 }),
            // 全体回復
            SpellKind::Healall1 => (6, Heal { base_heal: 10 }),
            SpellKind::Healall2 => (12, Heal { base_heal: 25 }),
            // 味方単体ブロック
            SpellKind::Shield1 => (3, Block { amount: 10 }),
            SpellKind::Shield2 => (6, Block { amount: 20 }),
            // 味方全体ブロック
            SpellKind::Barrier1 => (6, Block { amount: 6 }),
            SpellKind::Barrier2 => (10, Block { amount: 12 }),
            // 味方単体ATK↑
            SpellKind::Boost1 => (3, AttackBuff { amount: 3 }),
            SpellKind::Boost2 => (6, AttackBuff { amount: 6 }),
            // 味方全体ATK↑
            SpellKind::Rally1 => (6, AttackBuff { amount: 2 }),
            SpellKind::Rally2 => (10, AttackBuff { amount: 4 }),
            // 単体MP減少
            SpellKind::Drain1 => (4, MpDrain { base_drain: 8 }),
            SpellKind::Drain2 => (8, MpDrain { base_drain: 18 }),
            // 全体MP減少
            SpellKind::Siphon1 => (6, MpDrain { base_drain: 5 }),
            SpellKind::Siphon2 => (10, MpDrain { base_drain: 12 }),
            // 単体眠り
            SpellKind::Sleep1 => (4, Ailment { success_rate: 70 }),
            // 全体眠り
            SpellKind::Sleepall1 => (8, Ailment { success_rate: 50 }),
            // 単体毒
            SpellKind::Poison1 => (3, Ailment { success_rate: 80 }),
            // 全体毒
            SpellKind::Poisonall1 => (6, Ailment { success_rate: 60 }),
        }
    }
}

pub static ALL_SPELLS: &[SpellKind] = &[
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
pub fn all_spells() -> &'static [SpellKind] {
    ALL_SPELLS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell_damage_basic() {
        // base_damage=12, defense=4, divisor=4.0 → base = 12 - 1 = 11
        let damage = spell_damage(12, 4, DEFENSE_DIVISOR, 1.0);
        assert_eq!(damage, 11);
    }

    #[test]
    fn spell_damage_with_random() {
        let low = spell_damage(12, 4, DEFENSE_DIVISOR, 0.8);
        let high = spell_damage(12, 4, DEFENSE_DIVISOR, 1.2);
        assert_eq!(low, 9); // 11 * 0.8 = 8.8 → 9
        assert_eq!(high, 13); // 11 * 1.2 = 13.2 → 13
    }

    #[test]
    fn spell_damage_minimum_is_one() {
        let damage = spell_damage(1, 100, DEFENSE_DIVISOR, 0.8);
        assert_eq!(damage, 1);
    }

    #[test]
    fn heal_amount_basic() {
        let amount = heal_amount(15, 1.0);
        assert_eq!(amount, 15);
    }

    #[test]
    fn heal_amount_with_random() {
        let low = heal_amount(15, 0.8);
        let high = heal_amount(15, 1.2);
        assert_eq!(low, 12);
        assert_eq!(high, 18);
    }

    #[test]
    fn all_spells_returns_24() {
        assert_eq!(all_spells().len(), 24);
    }

    #[test]
    fn target_type_and_effect_consistency() {
        for &spell in all_spells() {
            match spell.effect() {
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
        let amount = mp_drain_amount(8, 1.0);
        assert_eq!(amount, 8);
    }

    #[test]
    fn mp_drain_with_random() {
        let low = mp_drain_amount(8, 0.8);
        let high = mp_drain_amount(8, 1.2);
        assert_eq!(low, 6);
        assert_eq!(high, 10);
    }

    #[test]
    fn mp_drain_minimum_is_one() {
        let amount = mp_drain_amount(1, 0.1);
        assert_eq!(amount, 1);
    }
}
