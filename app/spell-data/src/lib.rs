pub use spell::{Ailment, SpellEffect, SpellEntry, SpellTarget};
pub use spell::{ailment_success, heal_amount, mp_drain_amount, spell_damage};
pub use spell::{DEFENSE_DIVISOR, POISON_DAMAGE};

use SpellEffect::*;
use SpellTarget::*;

// 単体攻撃
pub const FIRE1: SpellEntry = SpellEntry { name: "Fire1", mp_cost: 3, effect: Damage { base_damage: 12 }, target_type: SingleEnemy, ailment: None };
pub const FIRE2: SpellEntry = SpellEntry { name: "Fire2", mp_cost: 7, effect: Damage { base_damage: 25 }, target_type: SingleEnemy, ailment: None };
// 全体攻撃
pub const BLAZE1: SpellEntry = SpellEntry { name: "Blaze1", mp_cost: 5, effect: Damage { base_damage: 8 }, target_type: AllEnemies, ailment: None };
pub const BLAZE2: SpellEntry = SpellEntry { name: "Blaze2", mp_cost: 10, effect: Damage { base_damage: 18 }, target_type: AllEnemies, ailment: None };
// 単体回復
pub const HEAL1: SpellEntry = SpellEntry { name: "Heal1", mp_cost: 3, effect: Heal { base_heal: 15 }, target_type: SingleAlly, ailment: None };
pub const HEAL2: SpellEntry = SpellEntry { name: "Heal2", mp_cost: 7, effect: Heal { base_heal: 40 }, target_type: SingleAlly, ailment: None };
// 全体回復
pub const HEALALL1: SpellEntry = SpellEntry { name: "Healall1", mp_cost: 6, effect: Heal { base_heal: 10 }, target_type: AllAllies, ailment: None };
pub const HEALALL2: SpellEntry = SpellEntry { name: "Healall2", mp_cost: 12, effect: Heal { base_heal: 25 }, target_type: AllAllies, ailment: None };
// 味方単体ブロック
pub const SHIELD1: SpellEntry = SpellEntry { name: "Shield1", mp_cost: 3, effect: Block { amount: 10 }, target_type: SingleAlly, ailment: None };
pub const SHIELD2: SpellEntry = SpellEntry { name: "Shield2", mp_cost: 6, effect: Block { amount: 20 }, target_type: SingleAlly, ailment: None };
// 味方全体ブロック
pub const BARRIER1: SpellEntry = SpellEntry { name: "Barrier1", mp_cost: 6, effect: Block { amount: 6 }, target_type: AllAllies, ailment: None };
pub const BARRIER2: SpellEntry = SpellEntry { name: "Barrier2", mp_cost: 10, effect: Block { amount: 12 }, target_type: AllAllies, ailment: None };
// 味方単体ATK↑
pub const BOOST1: SpellEntry = SpellEntry { name: "Boost1", mp_cost: 3, effect: AttackBuff { amount: 3 }, target_type: SingleAlly, ailment: None };
pub const BOOST2: SpellEntry = SpellEntry { name: "Boost2", mp_cost: 6, effect: AttackBuff { amount: 6 }, target_type: SingleAlly, ailment: None };
// 味方全体ATK↑
pub const RALLY1: SpellEntry = SpellEntry { name: "Rally1", mp_cost: 6, effect: AttackBuff { amount: 2 }, target_type: AllAllies, ailment: None };
pub const RALLY2: SpellEntry = SpellEntry { name: "Rally2", mp_cost: 10, effect: AttackBuff { amount: 4 }, target_type: AllAllies, ailment: None };
// 単体MP減少
pub const DRAIN1: SpellEntry = SpellEntry { name: "Drain1", mp_cost: 4, effect: MpDrain { base_drain: 8 }, target_type: SingleEnemy, ailment: None };
pub const DRAIN2: SpellEntry = SpellEntry { name: "Drain2", mp_cost: 8, effect: MpDrain { base_drain: 18 }, target_type: SingleEnemy, ailment: None };
// 全体MP減少
pub const SIPHON1: SpellEntry = SpellEntry { name: "Siphon1", mp_cost: 6, effect: MpDrain { base_drain: 5 }, target_type: AllEnemies, ailment: None };
pub const SIPHON2: SpellEntry = SpellEntry { name: "Siphon2", mp_cost: 10, effect: MpDrain { base_drain: 12 }, target_type: AllEnemies, ailment: None };
// 単体眠り
pub const SLEEP1: SpellEntry = SpellEntry { name: "Sleep1", mp_cost: 4, effect: SpellEffect::Ailment { success_rate: 70 }, target_type: SingleEnemy, ailment: Some(spell::Ailment::Sleep) };
// 全体眠り
pub const SLEEPALL1: SpellEntry = SpellEntry { name: "Sleepall1", mp_cost: 8, effect: SpellEffect::Ailment { success_rate: 50 }, target_type: AllEnemies, ailment: Some(spell::Ailment::Sleep) };
// 単体毒
pub const POISON1: SpellEntry = SpellEntry { name: "Poison1", mp_cost: 3, effect: SpellEffect::Ailment { success_rate: 80 }, target_type: SingleEnemy, ailment: Some(spell::Ailment::Poison) };
// 全体毒
pub const POISONALL1: SpellEntry = SpellEntry { name: "Poisonall1", mp_cost: 6, effect: SpellEffect::Ailment { success_rate: 60 }, target_type: AllEnemies, ailment: Some(spell::Ailment::Poison) };

pub static ALL_SPELLS: &[SpellEntry] = &[
    FIRE1, FIRE2, BLAZE1, BLAZE2,
    HEAL1, HEAL2, HEALALL1, HEALALL2,
    SHIELD1, SHIELD2, BARRIER1, BARRIER2,
    BOOST1, BOOST2, RALLY1, RALLY2,
    DRAIN1, DRAIN2, SIPHON1, SIPHON2,
    SLEEP1, SLEEPALL1, POISON1, POISONALL1,
];

/// 全呪文リストを返す
pub fn all_spells() -> &'static [SpellEntry] {
    ALL_SPELLS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell_damage_basic() {
        let damage = spell_damage(12, 4, DEFENSE_DIVISOR, 1.0);
        assert_eq!(damage, 11);
    }

    #[test]
    fn spell_damage_with_random() {
        let low = spell_damage(12, 4, DEFENSE_DIVISOR, 0.8);
        let high = spell_damage(12, 4, DEFENSE_DIVISOR, 1.2);
        assert_eq!(low, 9);
        assert_eq!(high, 13);
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
        for spell in all_spells() {
            match spell.effect {
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
                    assert!(spell.ailment.is_some());
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
