use super::enemy::Enemy;
use super::stats::{player_stats, CombatStats};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleAction {
    Attack,
    Flee,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnResult {
    /// プレイヤーが敵にダメージを与えた
    PlayerAttack { damage: i32 },
    /// 敵がプレイヤーにダメージを与えた
    EnemyAttack { damage: i32 },
    /// 敵を倒した
    EnemyDefeated,
    /// プレイヤーが倒された
    PlayerDefeated,
    /// 逃走成功
    Fled,
}

#[derive(Debug, Clone)]
pub struct BattleState {
    pub player: CombatStats,
    pub enemy: Enemy,
    pub turn_log: Vec<TurnResult>,
}

impl BattleState {
    pub fn new(enemy: Enemy) -> Self {
        Self {
            player: player_stats(),
            enemy,
            turn_log: Vec::new(),
        }
    }

    /// プレイヤーのターンを実行し、結果を返す
    ///
    /// `random_factor`: 0.8~1.2のダメージ乱数
    pub fn execute_player_turn(&mut self, action: BattleAction, random_factor: f32) -> Vec<TurnResult> {
        let mut results = Vec::new();

        match action {
            BattleAction::Flee => {
                results.push(TurnResult::Fled);
                self.turn_log.extend(results.clone());
                return results;
            }
            BattleAction::Attack => {
                let damage = CombatStats::calculate_damage(
                    self.player.attack,
                    self.enemy.stats.defense,
                    random_factor,
                );
                self.enemy.stats.take_damage(damage);
                results.push(TurnResult::PlayerAttack { damage });

                if !self.enemy.stats.is_alive() {
                    results.push(TurnResult::EnemyDefeated);
                    self.turn_log.extend(results.clone());
                    return results;
                }
            }
        }

        self.turn_log.extend(results.clone());
        results
    }

    /// 敵のターンを実行し、結果を返す
    pub fn execute_enemy_turn(&mut self, random_factor: f32) -> Vec<TurnResult> {
        let mut results = Vec::new();

        let damage = CombatStats::calculate_damage(
            self.enemy.stats.attack,
            self.player.defense,
            random_factor,
        );
        self.player.take_damage(damage);
        results.push(TurnResult::EnemyAttack { damage });

        if !self.player.is_alive() {
            results.push(TurnResult::PlayerDefeated);
        }

        self.turn_log.extend(results.clone());
        results
    }

    pub fn is_over(&self) -> bool {
        !self.player.is_alive()
            || !self.enemy.stats.is_alive()
            || self
                .turn_log
                .iter()
                .any(|r| matches!(r, TurnResult::Fled))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_battle() -> BattleState {
        BattleState::new(Enemy::slime())
    }

    #[test]
    fn new_battle_has_correct_initial_state() {
        let battle = test_battle();
        assert_eq!(battle.player.max_hp, 30);
        assert_eq!(battle.enemy.stats.max_hp, 10);
        assert!(battle.turn_log.is_empty());
    }

    #[test]
    fn player_attack_damages_enemy() {
        let mut battle = test_battle();
        let results = battle.execute_player_turn(BattleAction::Attack, 1.0);

        assert!(matches!(results[0], TurnResult::PlayerAttack { .. }));
        assert!(battle.enemy.stats.hp < battle.enemy.stats.max_hp);
    }

    #[test]
    fn enemy_attack_damages_player() {
        let mut battle = test_battle();
        let results = battle.execute_enemy_turn(1.0);

        assert!(matches!(results[0], TurnResult::EnemyAttack { .. }));
        assert!(battle.player.hp < battle.player.max_hp);
    }

    #[test]
    fn flee_ends_battle() {
        let mut battle = test_battle();
        let results = battle.execute_player_turn(BattleAction::Flee, 1.0);

        assert_eq!(results, vec![TurnResult::Fled]);
        assert!(battle.is_over());
    }

    #[test]
    fn enemy_defeated_when_hp_zero() {
        let mut battle = test_battle();
        // 繰り返し攻撃してスライムを倒す
        loop {
            let results = battle.execute_player_turn(BattleAction::Attack, 1.2);
            if results.contains(&TurnResult::EnemyDefeated) {
                break;
            }
            if battle.is_over() {
                break;
            }
            battle.execute_enemy_turn(1.0);
        }
        assert!(!battle.enemy.stats.is_alive());
        assert!(battle.is_over());
    }

    #[test]
    fn battle_not_over_initially() {
        let battle = test_battle();
        assert!(!battle.is_over());
    }

    #[test]
    fn damage_calculation_uses_stats() {
        let mut battle = test_battle();
        // プレイヤー: 攻撃8, スライム: 防御1
        // base = 8 - 0.5 = 7.5, * 1.0 = 7.5 → 8 (round)
        let results = battle.execute_player_turn(BattleAction::Attack, 1.0);
        if let TurnResult::PlayerAttack { damage } = results[0] {
            assert_eq!(damage, 8); // 7.5 rounded = 8
        } else {
            panic!("Expected PlayerAttack");
        }
    }
}
