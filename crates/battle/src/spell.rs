#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellKind {
    // 単体攻撃
    Zola,
    Zolaga,
    // 全体攻撃
    Neld,
    Neldora,
    // 単体回復
    Luna,
    Lunarm,
    // 全体回復
    Panam,
    Panamuda,
    // 味方単体DEF↑
    Garde,
    Gardeon,
    // 味方全体DEF↑
    Felza,
    Felzark,
    // 味方単体ATK↑
    Bolga,
    Bolgarda,
    // 味方全体ATK↑
    Zekta,
    Zektanam,
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
            SpellKind::Zola => "ゾラ",
            SpellKind::Zolaga => "ゾラーガ",
            SpellKind::Neld => "ネルド",
            SpellKind::Neldora => "ネルドーラ",
            SpellKind::Luna => "ルナ",
            SpellKind::Lunarm => "ルナーム",
            SpellKind::Panam => "パナム",
            SpellKind::Panamuda => "パナムーダ",
            SpellKind::Garde => "ガルデ",
            SpellKind::Gardeon => "ガルデオン",
            SpellKind::Felza => "フェルザ",
            SpellKind::Felzark => "フェルザーク",
            SpellKind::Bolga => "ボルガ",
            SpellKind::Bolgarda => "ボルガーダ",
            SpellKind::Zekta => "ゼクタ",
            SpellKind::Zektanam => "ゼクタナム",
        }
    }

    pub fn mp_cost(self) -> i32 {
        match self {
            SpellKind::Zola => 3,
            SpellKind::Zolaga => 7,
            SpellKind::Neld => 5,
            SpellKind::Neldora => 10,
            SpellKind::Luna => 3,
            SpellKind::Lunarm => 7,
            SpellKind::Panam => 6,
            SpellKind::Panamuda => 12,
            SpellKind::Garde => 3,
            SpellKind::Gardeon => 6,
            SpellKind::Felza => 6,
            SpellKind::Felzark => 10,
            SpellKind::Bolga => 3,
            SpellKind::Bolgarda => 6,
            SpellKind::Zekta => 6,
            SpellKind::Zektanam => 10,
        }
    }

    pub fn power(self) -> i32 {
        match self {
            SpellKind::Zola => 12,
            SpellKind::Zolaga => 25,
            SpellKind::Neld => 8,
            SpellKind::Neldora => 18,
            SpellKind::Luna => 15,
            SpellKind::Lunarm => 40,
            SpellKind::Panam => 10,
            SpellKind::Panamuda => 25,
            SpellKind::Garde => 3,
            SpellKind::Gardeon => 6,
            SpellKind::Felza => 2,
            SpellKind::Felzark => 4,
            SpellKind::Bolga => 3,
            SpellKind::Bolgarda => 6,
            SpellKind::Zekta => 2,
            SpellKind::Zektanam => 4,
        }
    }

    pub fn target_type(self) -> SpellTarget {
        match self {
            SpellKind::Zola | SpellKind::Zolaga => SpellTarget::SingleEnemy,
            SpellKind::Neld | SpellKind::Neldora => SpellTarget::AllEnemies,
            SpellKind::Luna | SpellKind::Lunarm => SpellTarget::SingleAlly,
            SpellKind::Panam | SpellKind::Panamuda => SpellTarget::AllAllies,
            SpellKind::Garde | SpellKind::Gardeon => SpellTarget::SingleAlly,
            SpellKind::Felza | SpellKind::Felzark => SpellTarget::AllAllies,
            SpellKind::Bolga | SpellKind::Bolgarda => SpellTarget::SingleAlly,
            SpellKind::Zekta | SpellKind::Zektanam => SpellTarget::AllAllies,
        }
    }

    pub fn effect(self) -> SpellEffect {
        match self {
            SpellKind::Zola | SpellKind::Zolaga | SpellKind::Neld | SpellKind::Neldora => {
                SpellEffect::Damage
            }
            SpellKind::Luna | SpellKind::Lunarm | SpellKind::Panam | SpellKind::Panamuda => {
                SpellEffect::Heal
            }
            SpellKind::Bolga | SpellKind::Bolgarda | SpellKind::Zekta | SpellKind::Zektanam => {
                SpellEffect::AttackBuff
            }
            SpellKind::Garde | SpellKind::Gardeon | SpellKind::Felza | SpellKind::Felzark => {
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
        SpellKind::Zola,
        SpellKind::Zolaga,
        SpellKind::Neld,
        SpellKind::Neldora,
        SpellKind::Luna,
        SpellKind::Lunarm,
        SpellKind::Panam,
        SpellKind::Panamuda,
        SpellKind::Garde,
        SpellKind::Gardeon,
        SpellKind::Felza,
        SpellKind::Felzark,
        SpellKind::Bolga,
        SpellKind::Bolgarda,
        SpellKind::Zekta,
        SpellKind::Zektanam,
    ]
}

/// キャラ別の呪文習得テーブル: (必要レベル, 呪文) のペア
pub fn spell_learn_table(kind: party::PartyMemberKind) -> &'static [(u32, SpellKind)] {
    use party::PartyMemberKind;
    match kind {
        PartyMemberKind::Marcille => &[
            (1, SpellKind::Zola),
            (3, SpellKind::Neld),
            (5, SpellKind::Zolaga),
            (7, SpellKind::Neldora),
        ],
        PartyMemberKind::Falin => &[
            (1, SpellKind::Luna),
            (3, SpellKind::Panam),
            (5, SpellKind::Lunarm),
            (7, SpellKind::Gardeon),
            (9, SpellKind::Panamuda),
            (10, SpellKind::Felzark),
        ],
        PartyMemberKind::Rinsha => &[
            (1, SpellKind::Zola),
            (3, SpellKind::Luna),
            (5, SpellKind::Bolga),
            (7, SpellKind::Bolgarda),
            (9, SpellKind::Zektanam),
        ],
        PartyMemberKind::Kabru => &[
            (3, SpellKind::Luna),
            (5, SpellKind::Garde),
            (7, SpellKind::Felza),
            (9, SpellKind::Zekta),
        ],
        PartyMemberKind::Laios => &[
            (1, SpellKind::Luna),
            (1, SpellKind::Bolga),
            (1, SpellKind::Zola),
            (1, SpellKind::Zolaga),
            (1, SpellKind::Neld),
            (1, SpellKind::Neldora),
            (1, SpellKind::Lunarm),
            (1, SpellKind::Panam),
            (1, SpellKind::Panamuda),
            (1, SpellKind::Garde),
            (1, SpellKind::Gardeon),
            (1, SpellKind::Felza),
            (1, SpellKind::Felzark),
            (1, SpellKind::Bolgarda),
            (1, SpellKind::Zekta),
            (1, SpellKind::Zektanam),
        ],
        PartyMemberKind::Izutsumi => &[
            (5, SpellKind::Zola),
            (8, SpellKind::Bolga),
        ],
        PartyMemberKind::Senshi => &[
            (4, SpellKind::Garde),
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
    fn marcille_learns_zola_at_level_1() {
        let spells = available_spells(party::PartyMemberKind::Marcille, 1);
        assert_eq!(spells, vec![SpellKind::Zola]);
    }

    #[test]
    fn marcille_learns_neld_at_level_3() {
        let spells = available_spells(party::PartyMemberKind::Marcille, 3);
        assert_eq!(spells, vec![SpellKind::Zola, SpellKind::Neld]);
    }

    #[test]
    fn marcille_learns_zolaga_at_level_5() {
        let spells = available_spells(party::PartyMemberKind::Marcille, 5);
        assert_eq!(
            spells,
            vec![SpellKind::Zola, SpellKind::Neld, SpellKind::Zolaga]
        );
    }

    #[test]
    fn falin_learns_luna_at_level_1() {
        let spells = available_spells(party::PartyMemberKind::Falin, 1);
        assert_eq!(spells, vec![SpellKind::Luna]);
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
            vec![SpellKind::Zola]
        );
        assert_eq!(
            spells_learned_at_level(party::PartyMemberKind::Marcille, 3),
            vec![SpellKind::Neld]
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
    fn senshi_learns_garde_at_level_4() {
        let spells = available_spells(party::PartyMemberKind::Senshi, 4);
        assert_eq!(spells, vec![SpellKind::Garde]);
    }

    #[test]
    fn no_spell_characters() {
        assert!(available_spells(party::PartyMemberKind::Chilchuck, 99).is_empty());
        assert!(available_spells(party::PartyMemberKind::Shuro, 99).is_empty());
        assert!(available_spells(party::PartyMemberKind::Namari, 99).is_empty());
    }
}
