use spell::{SpellEffect::*, SpellEntry, SpellKind, SpellParamTable};

pub fn spell_param_table() -> SpellParamTable {
    SpellParamTable::from_fn(|spell| match spell {
        // 単体攻撃
        SpellKind::Fire1 => SpellEntry { mp_cost: 3, effect: Damage { base_damage: 12 } },
        SpellKind::Fire2 => SpellEntry { mp_cost: 7, effect: Damage { base_damage: 25 } },
        // 全体攻撃
        SpellKind::Blaze1 => SpellEntry { mp_cost: 5, effect: Damage { base_damage: 8 } },
        SpellKind::Blaze2 => SpellEntry { mp_cost: 10, effect: Damage { base_damage: 18 } },
        // 単体回復
        SpellKind::Heal1 => SpellEntry { mp_cost: 3, effect: Heal { base_heal: 15 } },
        SpellKind::Heal2 => SpellEntry { mp_cost: 7, effect: Heal { base_heal: 40 } },
        // 全体回復
        SpellKind::Healall1 => SpellEntry { mp_cost: 6, effect: Heal { base_heal: 10 } },
        SpellKind::Healall2 => SpellEntry { mp_cost: 12, effect: Heal { base_heal: 25 } },
        // 味方単体ブロック
        SpellKind::Shield1 => SpellEntry { mp_cost: 3, effect: Block { amount: 10 } },
        SpellKind::Shield2 => SpellEntry { mp_cost: 6, effect: Block { amount: 20 } },
        // 味方全体ブロック
        SpellKind::Barrier1 => SpellEntry { mp_cost: 6, effect: Block { amount: 6 } },
        SpellKind::Barrier2 => SpellEntry { mp_cost: 10, effect: Block { amount: 12 } },
        // 味方単体ATK↑
        SpellKind::Boost1 => SpellEntry { mp_cost: 3, effect: AttackBuff { amount: 3 } },
        SpellKind::Boost2 => SpellEntry { mp_cost: 6, effect: AttackBuff { amount: 6 } },
        // 味方全体ATK↑
        SpellKind::Rally1 => SpellEntry { mp_cost: 6, effect: AttackBuff { amount: 2 } },
        SpellKind::Rally2 => SpellEntry { mp_cost: 10, effect: AttackBuff { amount: 4 } },
        // 単体MP減少
        SpellKind::Drain1 => SpellEntry { mp_cost: 4, effect: MpDrain { base_drain: 8 } },
        SpellKind::Drain2 => SpellEntry { mp_cost: 8, effect: MpDrain { base_drain: 18 } },
        // 全体MP減少
        SpellKind::Siphon1 => SpellEntry { mp_cost: 6, effect: MpDrain { base_drain: 5 } },
        SpellKind::Siphon2 => SpellEntry { mp_cost: 10, effect: MpDrain { base_drain: 12 } },
        // 単体眠り
        SpellKind::Sleep1 => SpellEntry { mp_cost: 4, effect: Ailment { success_rate: 70 } },
        // 全体眠り
        SpellKind::Sleepall1 => SpellEntry { mp_cost: 8, effect: Ailment { success_rate: 50 } },
        // 単体毒
        SpellKind::Poison1 => SpellEntry { mp_cost: 3, effect: Ailment { success_rate: 80 } },
        // 全体毒
        SpellKind::Poisonall1 => SpellEntry { mp_cost: 6, effect: Ailment { success_rate: 60 } },
    }, 3, 4.0)
}
