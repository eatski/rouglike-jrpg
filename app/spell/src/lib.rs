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

/// 状態異常呪文の付与判定
/// success_rate: 0~100の成功率、random_factor: 0.0~1.0の乱数
pub fn calculate_ailment_success(success_rate: i32, random_factor: f32) -> bool {
    random_factor * 100.0 < success_rate as f32
}

/// 毒の固定ダメージ
pub const POISON_DAMAGE: i32 = 3;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellEffect {
    Damage,
    Heal,
    AttackBuff,
    Block,
    MpDrain,
    Ailment,
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
            SpellKind::Drain1 => 4,
            SpellKind::Drain2 => 8,
            SpellKind::Siphon1 => 6,
            SpellKind::Siphon2 => 10,
            SpellKind::Sleep1 => 4,
            SpellKind::Sleepall1 => 8,
            SpellKind::Poison1 => 3,
            SpellKind::Poisonall1 => 6,
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
            SpellKind::Shield1 => 10,
            SpellKind::Shield2 => 20,
            SpellKind::Barrier1 => 6,
            SpellKind::Barrier2 => 12,
            SpellKind::Boost1 => 3,
            SpellKind::Boost2 => 6,
            SpellKind::Rally1 => 2,
            SpellKind::Rally2 => 4,
            SpellKind::Drain1 => 8,
            SpellKind::Drain2 => 18,
            SpellKind::Siphon1 => 5,
            SpellKind::Siphon2 => 12,
            SpellKind::Sleep1 => 70,
            SpellKind::Sleepall1 => 50,
            SpellKind::Poison1 => 80,
            SpellKind::Poisonall1 => 60,
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
                SpellEffect::Block
            }
            SpellKind::Drain1 | SpellKind::Drain2 | SpellKind::Siphon1 | SpellKind::Siphon2 => {
                SpellEffect::MpDrain
            }
            SpellKind::Sleep1
            | SpellKind::Sleepall1
            | SpellKind::Poison1
            | SpellKind::Poisonall1 => SpellEffect::Ailment,
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

    /// フィールドで使用可能か（回復呪文のみtrue）
    pub fn is_usable_in_field(self) -> bool {
        self.effect() == SpellEffect::Heal
    }

    /// 状態異常呪文かどうか
    pub fn is_ailment(self) -> bool {
        self.effect() == SpellEffect::Ailment
    }
}

/// 呪文ダメージ = (power - defense/4) × random_factor、最小1
pub fn calculate_spell_damage(power: i32, defense: i32, random_factor: f32) -> i32 {
    let base = power as f32 - defense as f32 / 4.0;
    let damage = (base * random_factor).round() as i32;
    damage.max(1)
}

/// MP減少量 = power × random_factor、最小1
pub fn calculate_mp_drain(power: i32, random_factor: f32) -> i32 {
    let amount = (power as f32 * random_factor).round() as i32;
    amount.max(1)
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
        SpellKind::Drain1,
        SpellKind::Drain2,
        SpellKind::Siphon1,
        SpellKind::Siphon2,
        SpellKind::Sleep1,
        SpellKind::Sleepall1,
        SpellKind::Poison1,
        SpellKind::Poisonall1,
    ]
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
    fn all_spells_returns_24() {
        assert_eq!(all_spells().len(), 24);
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
                SpellEffect::AttackBuff | SpellEffect::Block => {
                    assert!(!spell.is_offensive());
                    assert!(!spell.is_usable_in_field());
                }
                SpellEffect::MpDrain => {
                    assert!(spell.is_offensive());
                    assert!(!spell.is_usable_in_field());
                }
                SpellEffect::Ailment => {
                    assert!(spell.is_offensive());
                    assert!(!spell.is_usable_in_field());
                    assert!(spell.ailment().is_some());
                }
            }
        }
    }

    #[test]
    fn mp_drain_basic() {
        let amount = calculate_mp_drain(8, 1.0);
        assert_eq!(amount, 8);
    }

    #[test]
    fn mp_drain_with_random() {
        let low = calculate_mp_drain(8, 0.8);
        let high = calculate_mp_drain(8, 1.2);
        assert_eq!(low, 6); // 8 * 0.8 = 6.4 → 6
        assert_eq!(high, 10); // 8 * 1.2 = 9.6 → 10
    }

    #[test]
    fn mp_drain_minimum_is_one() {
        let amount = calculate_mp_drain(1, 0.1);
        assert_eq!(amount, 1);
    }

}
