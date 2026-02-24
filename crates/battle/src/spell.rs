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

/// キャラ別の呪文習得テーブル: (必要レベル, 呪文) のペア
pub fn spell_learn_table(kind: party::PartyMemberKind) -> &'static [(u32, SpellKind)] {
    use party::PartyMemberKind;
    match kind {
        PartyMemberKind::Laios => &[(3, SpellKind::Heal)],
        PartyMemberKind::Marcille => &[(1, SpellKind::Fire), (5, SpellKind::Blaze)],
        PartyMemberKind::Falin => &[(1, SpellKind::Heal), (4, SpellKind::FullHeal)],
        PartyMemberKind::Izutsumi => &[(5, SpellKind::Fire)],
        PartyMemberKind::Kabru => &[(4, SpellKind::Heal)],
        PartyMemberKind::Rinsha => &[(1, SpellKind::Fire), (3, SpellKind::Heal)],
        PartyMemberKind::Chilchuck
        | PartyMemberKind::Senshi
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
    fn laios_has_no_spells_before_level_3() {
        assert!(available_spells(party::PartyMemberKind::Laios, 1).is_empty());
        assert!(available_spells(party::PartyMemberKind::Laios, 2).is_empty());
    }

    #[test]
    fn laios_learns_heal_at_level_3() {
        let spells = available_spells(party::PartyMemberKind::Laios, 3);
        assert_eq!(spells, vec![SpellKind::Heal]);
    }

    #[test]
    fn marcille_learns_fire_at_level_1() {
        let spells = available_spells(party::PartyMemberKind::Marcille, 1);
        assert_eq!(spells, vec![SpellKind::Fire]);
    }

    #[test]
    fn marcille_learns_blaze_at_level_5() {
        let spells = available_spells(party::PartyMemberKind::Marcille, 5);
        assert_eq!(spells, vec![SpellKind::Fire, SpellKind::Blaze]);
    }

    #[test]
    fn marcille_no_blaze_at_level_4() {
        let spells = available_spells(party::PartyMemberKind::Marcille, 4);
        assert_eq!(spells, vec![SpellKind::Fire]);
    }

    #[test]
    fn falin_learns_heal_at_level_1() {
        let spells = available_spells(party::PartyMemberKind::Falin, 1);
        assert_eq!(spells, vec![SpellKind::Heal]);
    }

    #[test]
    fn falin_learns_fullheal_at_level_4() {
        let spells = available_spells(party::PartyMemberKind::Falin, 4);
        assert_eq!(spells, vec![SpellKind::Heal, SpellKind::FullHeal]);
    }

    #[test]
    fn spells_learned_at_specific_level() {
        assert_eq!(
            spells_learned_at_level(party::PartyMemberKind::Marcille, 1),
            vec![SpellKind::Fire]
        );
        assert_eq!(
            spells_learned_at_level(party::PartyMemberKind::Marcille, 5),
            vec![SpellKind::Blaze]
        );
        assert!(spells_learned_at_level(party::PartyMemberKind::Marcille, 3).is_empty());
    }
}
