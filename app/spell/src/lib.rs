/// 呪文の効果（データ付き）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellEffect {
    Damage { base_damage: i32 },
    Heal { base_heal: i32 },
    AttackBuff { amount: i32 },
    Block { amount: i32 },
    MpDrain { base_drain: i32 },
    Ailment { success_rate: i32 },
}

/// 1呪文のデータ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellEntry {
    pub name: &'static str,
    pub mp_cost: i32,
    pub effect: SpellEffect,
    pub target_type: SpellTarget,
    pub ailment: Option<Ailment>,
}

pub const DEFENSE_DIVISOR: f32 = 4.0;
pub const POISON_DAMAGE: i32 = 3;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellTarget {
    SingleEnemy,
    AllEnemies,
    SingleAlly,
    AllAllies,
}

/// 呪文ダメージ = (base_damage - defense/defense_divisor) × random_factor、最小1
pub fn spell_damage(base_damage: i32, defense: i32, defense_divisor: f32, random_factor: f32) -> i32 {
    let base = base_damage as f32 - defense as f32 / defense_divisor;
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
