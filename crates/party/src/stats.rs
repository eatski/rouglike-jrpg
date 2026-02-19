#[derive(Debug, Clone)]
pub struct CombatStats {
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
}

impl CombatStats {
    pub fn new(max_hp: i32, attack: i32, defense: i32, speed: i32, max_mp: i32) -> Self {
        Self {
            hp: max_hp,
            max_hp,
            mp: max_mp,
            max_mp,
            attack,
            defense,
            speed,
        }
    }

    /// MP消費。足りなければfalseを返し消費しない
    pub fn use_mp(&mut self, cost: i32) -> bool {
        if self.mp >= cost {
            self.mp -= cost;
            true
        } else {
            false
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// ダメージ = (攻撃 - 防御/2) × 乱数(0.8~1.2)、最小1
    pub fn calculate_damage(attacker_attack: i32, defender_defense: i32, random_factor: f32) -> i32 {
        let base = attacker_attack as f32 - defender_defense as f32 / 2.0;
        let damage = (base * random_factor).round() as i32;
        damage.max(1)
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.hp = (self.hp - damage).max(0);
    }

    /// ステータス成長を適用する
    pub fn apply_growth(&mut self, growth: &StatGrowth) {
        self.max_hp += growth.hp;
        self.hp = self.max_hp; // レベルアップ時は全回復
        self.max_mp += growth.mp;
        self.mp = self.max_mp;
        self.attack += growth.attack;
        self.defense += growth.defense;
        self.speed += growth.speed;
    }
}

/// レベルアップ時のステータス成長値
#[derive(Debug, Clone)]
pub struct StatGrowth {
    pub hp: i32,
    pub mp: i32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stats_have_full_hp() {
        let stats = CombatStats::new(30, 8, 3, 5, 10);
        assert_eq!(stats.hp, 30);
        assert_eq!(stats.max_hp, 30);
        assert_eq!(stats.mp, 10);
        assert_eq!(stats.max_mp, 10);
    }

    #[test]
    fn is_alive_returns_true_when_hp_positive() {
        let stats = CombatStats::new(10, 5, 2, 3, 0);
        assert!(stats.is_alive());
    }

    #[test]
    fn is_alive_returns_false_when_hp_zero() {
        let mut stats = CombatStats::new(10, 5, 2, 3, 0);
        stats.hp = 0;
        assert!(!stats.is_alive());
    }

    #[test]
    fn calculate_damage_minimum_is_one() {
        let damage = CombatStats::calculate_damage(1, 100, 1.0);
        assert_eq!(damage, 1);
    }

    #[test]
    fn calculate_damage_with_random_factor() {
        let damage_low = CombatStats::calculate_damage(8, 2, 0.8);
        let damage_high = CombatStats::calculate_damage(8, 2, 1.2);
        assert_eq!(damage_low, 6);
        assert_eq!(damage_high, 8);
    }

    #[test]
    fn take_damage_reduces_hp() {
        let mut stats = CombatStats::new(30, 8, 3, 5, 0);
        stats.take_damage(10);
        assert_eq!(stats.hp, 20);
    }

    #[test]
    fn take_damage_does_not_go_below_zero() {
        let mut stats = CombatStats::new(10, 5, 2, 3, 0);
        stats.take_damage(20);
        assert_eq!(stats.hp, 0);
    }

    #[test]
    fn use_mp_success() {
        let mut stats = CombatStats::new(10, 5, 2, 3, 10);
        assert!(stats.use_mp(3));
        assert_eq!(stats.mp, 7);
    }

    #[test]
    fn use_mp_insufficient() {
        let mut stats = CombatStats::new(10, 5, 2, 3, 2);
        assert!(!stats.use_mp(3));
        assert_eq!(stats.mp, 2);
    }
}
