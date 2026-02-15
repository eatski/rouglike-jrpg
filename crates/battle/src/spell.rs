#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellKind {
    Fire,
    Heal,
}

impl SpellKind {
    pub fn name(self) -> &'static str {
        match self {
            SpellKind::Fire => "ファイヤ",
            SpellKind::Heal => "ヒール",
        }
    }

    pub fn mp_cost(self) -> i32 {
        match self {
            SpellKind::Fire => 3,
            SpellKind::Heal => 4,
        }
    }

    pub fn power(self) -> i32 {
        match self {
            SpellKind::Fire => 12,
            SpellKind::Heal => 15,
        }
    }

    pub fn is_offensive(self) -> bool {
        match self {
            SpellKind::Fire => true,
            SpellKind::Heal => false,
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
    vec![SpellKind::Fire, SpellKind::Heal]
}

/// クラス別に使用可能な呪文リストを返す
pub fn available_spells(kind: party::PartyMemberKind) -> Vec<SpellKind> {
    use party::PartyMemberKind;
    match kind {
        PartyMemberKind::Hero => vec![],
        PartyMemberKind::Mage => vec![SpellKind::Fire],
        PartyMemberKind::Priest => vec![SpellKind::Heal],
    }
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
    fn all_spells_returns_both() {
        let spells = all_spells();
        assert_eq!(spells.len(), 2);
        assert_eq!(spells[0], SpellKind::Fire);
        assert_eq!(spells[1], SpellKind::Heal);
    }

    #[test]
    fn hero_has_no_spells() {
        let spells = available_spells(party::PartyMemberKind::Hero);
        assert!(spells.is_empty());
    }

    #[test]
    fn mage_has_fire() {
        let spells = available_spells(party::PartyMemberKind::Mage);
        assert_eq!(spells, vec![SpellKind::Fire]);
    }

    #[test]
    fn priest_has_heal() {
        let spells = available_spells(party::PartyMemberKind::Priest);
        assert_eq!(spells, vec![SpellKind::Heal]);
    }
}
