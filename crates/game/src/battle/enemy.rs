use super::stats::CombatStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    Slime,
}

impl EnemyKind {
    pub fn name(self) -> &'static str {
        match self {
            EnemyKind::Slime => "スライム",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub stats: CombatStats,
}

impl Enemy {
    pub fn slime() -> Self {
        Self {
            kind: EnemyKind::Slime,
            stats: CombatStats::new(10, 3, 1, 3, 0),
        }
    }
}

/// 乱数値(0.0~1.0)に基づいて1〜4匹のスライムを生成
pub fn generate_enemy_group(random_value: f32) -> Vec<Enemy> {
    let count = match random_value {
        v if v < 0.3 => 1,
        v if v < 0.6 => 2,
        v if v < 0.85 => 3,
        _ => 4,
    };
    (0..count).map(|_| Enemy::slime()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slime_has_correct_stats() {
        let slime = Enemy::slime();
        assert_eq!(slime.kind, EnemyKind::Slime);
        assert_eq!(slime.stats.max_hp, 10);
        assert_eq!(slime.stats.attack, 3);
        assert_eq!(slime.stats.defense, 1);
        assert_eq!(slime.stats.speed, 3);
    }

    #[test]
    fn slime_name() {
        assert_eq!(EnemyKind::Slime.name(), "スライム");
    }

    #[test]
    fn generate_enemy_group_returns_1_to_4() {
        assert_eq!(generate_enemy_group(0.0).len(), 1);
        assert_eq!(generate_enemy_group(0.29).len(), 1);
        assert_eq!(generate_enemy_group(0.3).len(), 2);
        assert_eq!(generate_enemy_group(0.6).len(), 3);
        assert_eq!(generate_enemy_group(0.85).len(), 4);
        assert_eq!(generate_enemy_group(1.0).len(), 4);
    }
}
