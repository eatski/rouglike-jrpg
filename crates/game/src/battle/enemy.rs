use super::stats::CombatStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            stats: CombatStats::new(10, 3, 1),
        }
    }
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
    }

    #[test]
    fn slime_name() {
        assert_eq!(EnemyKind::Slime.name(), "スライム");
    }
}
