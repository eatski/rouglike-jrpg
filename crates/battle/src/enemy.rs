use party::CombatStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    Slime,
    Bat,
    Goblin,
    Wolf,
    Ghost,
    DarkLord,
}

impl EnemyKind {
    pub fn name(self) -> &'static str {
        match self {
            EnemyKind::Slime => "スライム",
            EnemyKind::Bat => "コウモリ",
            EnemyKind::Goblin => "ゴブリン",
            EnemyKind::Wolf => "おおかみ",
            EnemyKind::Ghost => "ゴースト",
            EnemyKind::DarkLord => "まおう",
        }
    }

    pub fn sprite_path(self) -> &'static str {
        match self {
            EnemyKind::Slime => "enemies/slime.png",
            EnemyKind::Bat => "enemies/bat.png",
            EnemyKind::Goblin => "enemies/goblin.png",
            EnemyKind::Wolf => "enemies/wolf.png",
            EnemyKind::Ghost => "enemies/ghost.png",
            EnemyKind::DarkLord => "enemies/dark_lord.png",
        }
    }

    /// 倒した時に得られる経験値
    pub fn exp_reward(self) -> u32 {
        match self {
            EnemyKind::Slime => 3,
            EnemyKind::Bat => 4,
            EnemyKind::Goblin => 6,
            EnemyKind::Wolf => 8,
            EnemyKind::Ghost => 10,
            EnemyKind::DarkLord => 100,
        }
    }
}

const ALL_ENEMY_KINDS: [EnemyKind; 5] = [
    EnemyKind::Slime,
    EnemyKind::Bat,
    EnemyKind::Goblin,
    EnemyKind::Wolf,
    EnemyKind::Ghost,
];

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

    pub fn bat() -> Self {
        Self {
            kind: EnemyKind::Bat,
            stats: CombatStats::new(8, 4, 0, 6, 0),
        }
    }

    pub fn goblin() -> Self {
        Self {
            kind: EnemyKind::Goblin,
            stats: CombatStats::new(15, 5, 2, 3, 0),
        }
    }

    pub fn wolf() -> Self {
        Self {
            kind: EnemyKind::Wolf,
            stats: CombatStats::new(12, 7, 1, 5, 0),
        }
    }

    pub fn ghost() -> Self {
        Self {
            kind: EnemyKind::Ghost,
            stats: CombatStats::new(20, 4, 3, 2, 0),
        }
    }

    pub fn dark_lord() -> Self {
        Self {
            kind: EnemyKind::DarkLord,
            stats: CombatStats::new(200, 25, 15, 8, 50),
        }
    }

    fn from_kind(kind: EnemyKind) -> Self {
        match kind {
            EnemyKind::Slime => Self::slime(),
            EnemyKind::Bat => Self::bat(),
            EnemyKind::Goblin => Self::goblin(),
            EnemyKind::Wolf => Self::wolf(),
            EnemyKind::Ghost => Self::ghost(),
            EnemyKind::DarkLord => Self::dark_lord(),
        }
    }
}

/// 2つの乱数値(0.0~1.0)に基づいて敵グループを生成
/// - `count_random`: 匹数決定用（1〜4匹）
/// - `kind_random`: 敵の種類決定用
pub fn generate_enemy_group(count_random: f32, kind_random: f32) -> Vec<Enemy> {
    let count = match count_random {
        v if v < 0.3 => 1,
        v if v < 0.6 => 2,
        v if v < 0.85 => 3,
        _ => 4,
    };

    let kind_index = (kind_random * ALL_ENEMY_KINDS.len() as f32).min(ALL_ENEMY_KINDS.len() as f32 - 1.0) as usize;
    let kind = ALL_ENEMY_KINDS[kind_index];

    (0..count).map(|_| Enemy::from_kind(kind)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_enemy_group_returns_1_to_4() {
        assert_eq!(generate_enemy_group(0.0, 0.0).len(), 1);
        assert_eq!(generate_enemy_group(0.29, 0.0).len(), 1);
        assert_eq!(generate_enemy_group(0.3, 0.0).len(), 2);
        assert_eq!(generate_enemy_group(0.6, 0.0).len(), 3);
        assert_eq!(generate_enemy_group(0.85, 0.0).len(), 4);
        assert_eq!(generate_enemy_group(1.0, 0.0).len(), 4);
    }

    #[test]
    fn generate_enemy_group_selects_kind() {
        // kind_random=0.0 -> Slime (index 0)
        let group = generate_enemy_group(0.0, 0.0);
        assert_eq!(group[0].kind, EnemyKind::Slime);

        // kind_random=0.5 -> Goblin (index 2)
        let group = generate_enemy_group(0.0, 0.5);
        assert_eq!(group[0].kind, EnemyKind::Goblin);

        // kind_random=0.99 -> Ghost (index 4)
        let group = generate_enemy_group(0.0, 0.99);
        assert_eq!(group[0].kind, EnemyKind::Ghost);
    }
}
