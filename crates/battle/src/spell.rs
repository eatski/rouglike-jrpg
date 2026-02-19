#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellKind {
    Fire,
    Blaze,
    Heal,
    FullHeal,
}

impl SpellKind {
    pub fn name(self) -> &'static str {
        match self {
            SpellKind::Fire => "ファイヤ",
            SpellKind::Blaze => "ブレイズ",
            SpellKind::Heal => "ヒール",
            SpellKind::FullHeal => "フルヒール",
        }
    }

    pub fn mp_cost(self) -> i32 {
        match self {
            SpellKind::Fire => 3,
            SpellKind::Blaze => 7,
            SpellKind::Heal => 4,
            SpellKind::FullHeal => 8,
        }
    }

    pub fn power(self) -> i32 {
        match self {
            SpellKind::Fire => 12,
            SpellKind::Blaze => 25,
            SpellKind::Heal => 15,
            SpellKind::FullHeal => 40,
        }
    }

    pub fn is_offensive(self) -> bool {
        match self {
            SpellKind::Fire | SpellKind::Blaze => true,
            SpellKind::Heal | SpellKind::FullHeal => false,
        }
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
        SpellKind::Fire,
        SpellKind::Blaze,
        SpellKind::Heal,
        SpellKind::FullHeal,
    ]
}

/// クラス別の呪文習得テーブル: (必要レベル, 呪文) のペア
pub fn spell_learn_table(kind: party::PartyMemberKind) -> &'static [(u32, SpellKind)] {
    use party::PartyMemberKind;
    match kind {
        PartyMemberKind::Hero => &[(3, SpellKind::Heal)],
        PartyMemberKind::Mage => &[(1, SpellKind::Fire), (5, SpellKind::Blaze)],
        PartyMemberKind::Priest => &[(1, SpellKind::Heal), (4, SpellKind::FullHeal)],
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
    fn fire_properties() {
        assert_eq!(SpellKind::Fire.name(), "ファイヤ");
        assert_eq!(SpellKind::Fire.mp_cost(), 3);
        assert_eq!(SpellKind::Fire.power(), 12);
        assert!(SpellKind::Fire.is_offensive());
    }

    #[test]
    fn heal_properties() {
        assert_eq!(SpellKind::Heal.name(), "ヒール");
        assert_eq!(SpellKind::Heal.mp_cost(), 4);
        assert_eq!(SpellKind::Heal.power(), 15);
        assert!(!SpellKind::Heal.is_offensive());
    }

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
    fn blaze_properties() {
        assert_eq!(SpellKind::Blaze.name(), "ブレイズ");
        assert_eq!(SpellKind::Blaze.mp_cost(), 7);
        assert_eq!(SpellKind::Blaze.power(), 25);
        assert!(SpellKind::Blaze.is_offensive());
    }

    #[test]
    fn fullheal_properties() {
        assert_eq!(SpellKind::FullHeal.name(), "フルヒール");
        assert_eq!(SpellKind::FullHeal.mp_cost(), 8);
        assert_eq!(SpellKind::FullHeal.power(), 40);
        assert!(!SpellKind::FullHeal.is_offensive());
    }

    #[test]
    fn all_spells_returns_all() {
        let spells = all_spells();
        assert_eq!(spells.len(), 4);
        assert_eq!(spells[0], SpellKind::Fire);
        assert_eq!(spells[1], SpellKind::Blaze);
        assert_eq!(spells[2], SpellKind::Heal);
        assert_eq!(spells[3], SpellKind::FullHeal);
    }

    #[test]
    fn hero_has_no_spells_before_level_3() {
        assert!(available_spells(party::PartyMemberKind::Hero, 1).is_empty());
        assert!(available_spells(party::PartyMemberKind::Hero, 2).is_empty());
    }

    #[test]
    fn hero_learns_heal_at_level_3() {
        let spells = available_spells(party::PartyMemberKind::Hero, 3);
        assert_eq!(spells, vec![SpellKind::Heal]);
    }

    #[test]
    fn mage_learns_fire_at_level_1() {
        let spells = available_spells(party::PartyMemberKind::Mage, 1);
        assert_eq!(spells, vec![SpellKind::Fire]);
    }

    #[test]
    fn mage_learns_blaze_at_level_5() {
        let spells = available_spells(party::PartyMemberKind::Mage, 5);
        assert_eq!(spells, vec![SpellKind::Fire, SpellKind::Blaze]);
    }

    #[test]
    fn mage_no_blaze_at_level_4() {
        let spells = available_spells(party::PartyMemberKind::Mage, 4);
        assert_eq!(spells, vec![SpellKind::Fire]);
    }

    #[test]
    fn priest_learns_heal_at_level_1() {
        let spells = available_spells(party::PartyMemberKind::Priest, 1);
        assert_eq!(spells, vec![SpellKind::Heal]);
    }

    #[test]
    fn priest_learns_fullheal_at_level_4() {
        let spells = available_spells(party::PartyMemberKind::Priest, 4);
        assert_eq!(spells, vec![SpellKind::Heal, SpellKind::FullHeal]);
    }

    #[test]
    fn spells_learned_at_specific_level() {
        assert_eq!(
            spells_learned_at_level(party::PartyMemberKind::Mage, 1),
            vec![SpellKind::Fire]
        );
        assert_eq!(
            spells_learned_at_level(party::PartyMemberKind::Mage, 5),
            vec![SpellKind::Blaze]
        );
        assert!(spells_learned_at_level(party::PartyMemberKind::Mage, 3).is_empty());
    }
}
