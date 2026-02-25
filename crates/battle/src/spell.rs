#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    // 味方単体DEF↑
    Shield1,
    Shield2,
    // 味方全体DEF↑
    Barrier1,
    Barrier2,
    // 味方単体ATK↑
    Boost1,
    Boost2,
    // 味方全体ATK↑
    Rally1,
    Rally2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellTarget {
    SingleEnemy,
    AllEnemies,
    SingleAlly,
    AllAllies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellEffect {
    Damage,
    Heal,
    AttackBuff,
    DefenseBuff,
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
        }
    }

    pub fn mp_cost(self) -> i32 {
        match self {
            SpellKind::Fire1 => 3,
            SpellKind::Fire2 => 7,
            SpellKind::Blaze1 => 5,
            SpellKind::Blaze2 => 10,
            SpellKind::Heal1 => 3,
            SpellKind::Heal2 => 7,
            SpellKind::Healall1 => 6,
            SpellKind::Healall2 => 12,
            SpellKind::Shield1 => 3,
            SpellKind::Shield2 => 6,
            SpellKind::Barrier1 => 6,
            SpellKind::Barrier2 => 10,
            SpellKind::Boost1 => 3,
            SpellKind::Boost2 => 6,
            SpellKind::Rally1 => 6,
            SpellKind::Rally2 => 10,
        }
    }

    pub fn power(self) -> i32 {
        match self {
            SpellKind::Fire1 => 12,
            SpellKind::Fire2 => 25,
            SpellKind::Blaze1 => 8,
            SpellKind::Blaze2 => 18,
            SpellKind::Heal1 => 15,
            SpellKind::Heal2 => 40,
            SpellKind::Healall1 => 10,
            SpellKind::Healall2 => 25,
            SpellKind::Shield1 => 3,
            SpellKind::Shield2 => 6,
            SpellKind::Barrier1 => 2,
            SpellKind::Barrier2 => 4,
            SpellKind::Boost1 => 3,
            SpellKind::Boost2 => 6,
            SpellKind::Rally1 => 2,
            SpellKind::Rally2 => 4,
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
        }
    }

    pub fn effect(self) -> SpellEffect {
        match self {
            SpellKind::Fire1 | SpellKind::Fire2 | SpellKind::Blaze1 | SpellKind::Blaze2 => {
                SpellEffect::Damage
            }
            SpellKind::Heal1 | SpellKind::Heal2 | SpellKind::Healall1 | SpellKind::Healall2 => {
                SpellEffect::Heal
            }
            SpellKind::Boost1 | SpellKind::Boost2 | SpellKind::Rally1 | SpellKind::Rally2 => {
                SpellEffect::AttackBuff
            }
            SpellKind::Shield1 | SpellKind::Shield2 | SpellKind::Barrier1 | SpellKind::Barrier2 => {
                SpellEffect::DefenseBuff
            }
        }
    }

    /// 後方互換: target_type()から導出
    pub fn is_offensive(self) -> bool {
        matches!(
            self.target_type(),
            SpellTarget::SingleEnemy | SpellTarget::AllEnemies
        )
    }

    /// フィールドで使用可能か（回復呪文のみtrue）
    pub fn is_usable_in_field(self) -> bool {
        self.effect() == SpellEffect::Heal
    }
}

/// 呪文ダメージ = (power - defense/4) × random_factor、最小1
pub fn calculate_spell_damage(power: i32, defense: i32, random_factor: f32) -> i32 {
    let base = power as f32 - defense as f32 / 4.0;
    let damage = (base * random_factor).round() as i32;
    damage.max(1)
}

/// 回復量 = power × random_factor
pub fn calculate_heal_amount(power: i32, random_factor: f32) -> i32 {
    let amount = (power as f32 * random_factor).round() as i32;
    amount.max(1)
}

/// 全呪文リストを返す
pub fn all_spells() -> Vec<SpellKind> {
    vec![
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
    ]
}

/// キャラ別の呪文習得テーブル: (必要レベル, 呪文) のペア
pub fn spell_learn_table(kind: party::PartyMemberKind) -> &'static [(u32, SpellKind)] {
    use party::PartyMemberKind;
    match kind {
        PartyMemberKind::Marcille => &[
            (1, SpellKind::Fire1),
            (3, SpellKind::Blaze1),
            (5, SpellKind::Fire2),
            (7, SpellKind::Blaze2),
        ],
        PartyMemberKind::Falin => &[
            (1, SpellKind::Heal1),
            (3, SpellKind::Healall1),
            (5, SpellKind::Heal2),
            (7, SpellKind::Shield2),
            (9, SpellKind::Healall2),
            (10, SpellKind::Barrier2),
        ],
        PartyMemberKind::Rinsha => &[
            (1, SpellKind::Fire1),
            (3, SpellKind::Heal1),
            (5, SpellKind::Boost1),
            (7, SpellKind::Boost2),
            (9, SpellKind::Rally2),
        ],
        PartyMemberKind::Kabru => &[
            (3, SpellKind::Heal1),
            (5, SpellKind::Shield1),
            (7, SpellKind::Barrier1),
            (9, SpellKind::Rally1),
        ],
        PartyMemberKind::Laios => &[
            (1, SpellKind::Heal1),
            (1, SpellKind::Boost1),
            (1, SpellKind::Fire1),
            (1, SpellKind::Fire2),
            (1, SpellKind::Blaze1),
            (1, SpellKind::Blaze2),
            (1, SpellKind::Heal2),
            (1, SpellKind::Healall1),
            (1, SpellKind::Healall2),
            (1, SpellKind::Shield1),
            (1, SpellKind::Shield2),
            (1, SpellKind::Barrier1),
            (1, SpellKind::Barrier2),
            (1, SpellKind::Boost2),
            (1, SpellKind::Rally1),
            (1, SpellKind::Rally2),
        ],
        PartyMemberKind::Izutsumi => &[
            (5, SpellKind::Fire1),
            (8, SpellKind::Boost1),
        ],
        PartyMemberKind::Senshi => &[
            (4, SpellKind::Shield1),
        ],
        PartyMemberKind::Chilchuck
        | PartyMemberKind::Shuro
        | PartyMemberKind::Namari => &[],
    }
}

/// クラスとレベルに応じた使用可能な呪文リストを返す
pub fn available_spells(kind: party::PartyMemberKind, level: u32) -> Vec<SpellKind> {
    spell_learn_table(kind)
        .iter()
        .filter(|(req_level, _)| level >= *req_level)
        .map(|(_, spell)| *spell)
        .collect()
}

/// 指定レベルで新たに習得する呪文を返す
pub fn spells_learned_at_level(kind: party::PartyMemberKind, level: u32) -> Vec<SpellKind> {
    spell_learn_table(kind)
        .iter()
        .filter(|(req_level, _)| *req_level == level)
        .map(|(_, spell)| *spell)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell_damage_basic() {
        // power=12, defense=4 → base = 12 - 1 = 11
        let damage = calculate_spell_damage(12, 4, 1.0);
        assert_eq!(damage, 11);
    }

    #[test]
    fn spell_damage_with_random() {
        let low = calculate_spell_damage(12, 4, 0.8);
        let high = calculate_spell_damage(12, 4, 1.2);
        assert_eq!(low, 9); // 11 * 0.8 = 8.8 → 9
        assert_eq!(high, 13); // 11 * 1.2 = 13.2 → 13
    }

    #[test]
    fn spell_damage_minimum_is_one() {
        let damage = calculate_spell_damage(1, 100, 0.8);
        assert_eq!(damage, 1);
    }

    #[test]
    fn heal_amount_basic() {
        let amount = calculate_heal_amount(15, 1.0);
        assert_eq!(amount, 15);
    }

    #[test]
    fn heal_amount_with_random() {
        let low = calculate_heal_amount(15, 0.8);
        let high = calculate_heal_amount(15, 1.2);
        assert_eq!(low, 12); // 15 * 0.8 = 12
        assert_eq!(high, 18); // 15 * 1.2 = 18
    }

    #[test]
    fn laios_has_all_16_spells_at_level_1() {
        let spells = available_spells(party::PartyMemberKind::Laios, 1);
        assert_eq!(spells.len(), 16);
    }

    #[test]
    fn marcille_learns_fire1_at_level_1() {
        let spells = available_spells(party::PartyMemberKind::Marcille, 1);
        assert_eq!(spells, vec![SpellKind::Fire1]);
    }

    #[test]
    fn marcille_learns_blaze1_at_level_3() {
        let spells = available_spells(party::PartyMemberKind::Marcille, 3);
        assert_eq!(spells, vec![SpellKind::Fire1, SpellKind::Blaze1]);
    }

    #[test]
    fn marcille_learns_fire2_at_level_5() {
        let spells = available_spells(party::PartyMemberKind::Marcille, 5);
        assert_eq!(
            spells,
            vec![SpellKind::Fire1, SpellKind::Blaze1, SpellKind::Fire2]
        );
    }

    #[test]
    fn falin_learns_heal1_at_level_1() {
        let spells = available_spells(party::PartyMemberKind::Falin, 1);
        assert_eq!(spells, vec![SpellKind::Heal1]);
    }

    #[test]
    fn falin_max_spells() {
        let spells = available_spells(party::PartyMemberKind::Falin, 10);
        assert_eq!(spells.len(), 6);
    }

    #[test]
    fn spells_learned_at_specific_level() {
        assert_eq!(
            spells_learned_at_level(party::PartyMemberKind::Marcille, 1),
            vec![SpellKind::Fire1]
        );
        assert_eq!(
            spells_learned_at_level(party::PartyMemberKind::Marcille, 3),
            vec![SpellKind::Blaze1]
        );
        assert!(spells_learned_at_level(party::PartyMemberKind::Marcille, 2).is_empty());
    }

    #[test]
    fn all_spells_returns_16() {
        assert_eq!(all_spells().len(), 16);
    }

    #[test]
    fn target_type_and_effect_consistency() {
        for spell in all_spells() {
            match spell.effect() {
                SpellEffect::Damage => {
                    assert!(spell.is_offensive());
                    assert!(!spell.is_usable_in_field());
                }
                SpellEffect::Heal => {
                    assert!(!spell.is_offensive());
                    assert!(spell.is_usable_in_field());
                }
                SpellEffect::AttackBuff | SpellEffect::DefenseBuff => {
                    assert!(!spell.is_offensive());
                    assert!(!spell.is_usable_in_field());
                }
            }
        }
    }

    #[test]
    fn senshi_learns_shield1_at_level_4() {
        let spells = available_spells(party::PartyMemberKind::Senshi, 4);
        assert_eq!(spells, vec![SpellKind::Shield1]);
    }

    #[test]
    fn no_spell_characters() {
        assert!(available_spells(party::PartyMemberKind::Chilchuck, 99).is_empty());
        assert!(available_spells(party::PartyMemberKind::Shuro, 99).is_empty());
        assert!(available_spells(party::PartyMemberKind::Namari, 99).is_empty());
    }
}
