use super::enemy::Enemy;
use super::party::PartyMember;
use super::stats::CombatStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorId {
    Party(usize),
    Enemy(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetId {
    Enemy(usize),
    Party(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleAction {
    Attack { target: TargetId },
    Flee,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnResult {
    Attack {
        attacker: ActorId,
        target: TargetId,
        damage: i32,
    },
    Defeated {
        target: TargetId,
    },
    Fled,
    FleeFailed,
}

/// ターン実行に必要な乱数群
pub struct TurnRandomFactors {
    /// 各アクターのダメージ乱数(0.8~1.2)。indexはaction_order内の順番
    pub damage_randoms: Vec<f32>,
    /// 逃走判定用の乱数(0.0~1.0)。0.5未満で成功
    pub flee_random: f32,
}

#[derive(Debug, Clone)]
pub struct BattleState {
    pub party: Vec<PartyMember>,
    pub enemies: Vec<Enemy>,
    pub turn_log: Vec<TurnResult>,
}

impl BattleState {
    pub fn new(party: Vec<PartyMember>, enemies: Vec<Enemy>) -> Self {
        Self {
            party,
            enemies,
            turn_log: Vec::new(),
        }
    }

    /// パーティ全員分のコマンドを受け取り、素早さ順で一括実行
    ///
    /// `party_commands`: パーティメンバーのインデックス順のアクション
    /// `random_factors`: 乱数群
    pub fn execute_turn(
        &mut self,
        party_commands: &[BattleAction],
        random_factors: &TurnRandomFactors,
    ) -> Vec<TurnResult> {
        // 逃走チェック: 誰かがFleeを選んでいたら逃走判定
        let has_flee = party_commands.iter().any(|a| matches!(a, BattleAction::Flee));
        if has_flee {
            if random_factors.flee_random < 0.5 {
                let result = vec![TurnResult::Fled];
                self.turn_log.extend(result.clone());
                return result;
            } else {
                // 逃走失敗: 敵だけ行動
                let mut results = vec![TurnResult::FleeFailed];
                for (random_idx, ei) in self.alive_enemy_indices().into_iter().enumerate() {
                    let random = random_factors
                        .damage_randoms
                        .get(random_idx)
                        .copied()
                        .unwrap_or(1.0);
                    results.extend(self.execute_enemy_attack(ei, random));
                }
                self.turn_log.extend(results.clone());
                return results;
            }
        }

        // 素早さ順の行動順序を決定
        let action_order = self.build_action_order(party_commands);

        let mut results = Vec::new();

        for (random_idx, actor) in action_order.iter().enumerate() {
            let random = random_factors
                .damage_randoms
                .get(random_idx)
                .copied()
                .unwrap_or(1.0);

            match actor {
                ActorId::Party(pi) => {
                    let pi = *pi;
                    if !self.party[pi].stats.is_alive() {
                        continue;
                    }
                    if let Some(BattleAction::Attack { target }) = party_commands.get(pi) {
                        let actual_target = self.retarget_enemy(*target);
                        if let Some(actual_target) = actual_target {
                            results.extend(self.execute_party_attack(pi, actual_target, random));
                        }
                    }
                }
                ActorId::Enemy(ei) => {
                    let ei = *ei;
                    if !self.enemies[ei].stats.is_alive() {
                        continue;
                    }
                    results.extend(self.execute_enemy_attack(ei, random));
                }
            }

            // 戦闘終了チェック
            if self.is_over() {
                break;
            }
        }

        self.turn_log.extend(results.clone());
        results
    }

    /// 素早さ順の行動順序を構築
    fn build_action_order(&self, party_commands: &[BattleAction]) -> Vec<ActorId> {
        let mut actors: Vec<(ActorId, i32)> = Vec::new();

        for (i, member) in self.party.iter().enumerate() {
            if member.stats.is_alive() && i < party_commands.len() {
                actors.push((ActorId::Party(i), member.stats.speed));
            }
        }
        for (i, enemy) in self.enemies.iter().enumerate() {
            if enemy.stats.is_alive() {
                actors.push((ActorId::Enemy(i), enemy.stats.speed));
            }
        }

        // 素早さ降順（同速ならパーティ優先）
        actors.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| {
                let a_order = match a.0 {
                    ActorId::Party(_) => 0,
                    ActorId::Enemy(_) => 1,
                };
                let b_order = match b.0 {
                    ActorId::Party(_) => 0,
                    ActorId::Enemy(_) => 1,
                };
                a_order.cmp(&b_order)
            })
        });

        actors.into_iter().map(|(id, _)| id).collect()
    }

    /// ターゲットの敵が既に倒されていたら最初の生存敵にリターゲット
    fn retarget_enemy(&self, target: TargetId) -> Option<TargetId> {
        if let TargetId::Enemy(ei) = target {
            if self.enemies[ei].stats.is_alive() {
                return Some(target);
            }
            // リターゲット: 最初の生存敵
            self.alive_enemy_indices()
                .first()
                .map(|&i| TargetId::Enemy(i))
        } else {
            Some(target)
        }
    }

    /// パーティメンバーが敵を攻撃
    fn execute_party_attack(
        &mut self,
        party_idx: usize,
        target: TargetId,
        random_factor: f32,
    ) -> Vec<TurnResult> {
        let mut results = Vec::new();
        if let TargetId::Enemy(ei) = target {
            let damage = CombatStats::calculate_damage(
                self.party[party_idx].stats.attack,
                self.enemies[ei].stats.defense,
                random_factor,
            );
            self.enemies[ei].stats.take_damage(damage);
            results.push(TurnResult::Attack {
                attacker: ActorId::Party(party_idx),
                target,
                damage,
            });
            if !self.enemies[ei].stats.is_alive() {
                results.push(TurnResult::Defeated { target });
            }
        }
        results
    }

    /// 敵がランダムなパーティメンバーを攻撃（最初の生存メンバーをターゲット）
    fn execute_enemy_attack(&mut self, enemy_idx: usize, random_factor: f32) -> Vec<TurnResult> {
        let mut results = Vec::new();
        let alive_party = self.alive_party_indices();
        if alive_party.is_empty() {
            return results;
        }
        // 簡易: 最初の生存パーティメンバーを攻撃
        let target_idx = alive_party[0];
        let damage = CombatStats::calculate_damage(
            self.enemies[enemy_idx].stats.attack,
            self.party[target_idx].stats.defense,
            random_factor,
        );
        self.party[target_idx].stats.take_damage(damage);
        let target = TargetId::Party(target_idx);
        results.push(TurnResult::Attack {
            attacker: ActorId::Enemy(enemy_idx),
            target,
            damage,
        });
        if !self.party[target_idx].stats.is_alive() {
            results.push(TurnResult::Defeated { target });
        }
        results
    }

    pub fn is_over(&self) -> bool {
        self.is_victory()
            || self.is_party_wiped()
            || self.turn_log.iter().any(|r| matches!(r, TurnResult::Fled))
    }

    pub fn is_victory(&self) -> bool {
        self.enemies.iter().all(|e| !e.stats.is_alive())
    }

    pub fn is_party_wiped(&self) -> bool {
        self.party.iter().all(|p| !p.stats.is_alive())
    }

    pub fn alive_enemy_indices(&self) -> Vec<usize> {
        self.enemies
            .iter()
            .enumerate()
            .filter(|(_, e)| e.stats.is_alive())
            .map(|(i, _)| i)
            .collect()
    }

    pub fn alive_party_indices(&self) -> Vec<usize> {
        self.party
            .iter()
            .enumerate()
            .filter(|(_, p)| p.stats.is_alive())
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::enemy::Enemy;
    use crate::battle::party::{default_party, PartyMember};

    fn make_random(damage_randoms: Vec<f32>, flee_random: f32) -> TurnRandomFactors {
        TurnRandomFactors {
            damage_randoms,
            flee_random,
        }
    }

    #[test]
    fn basic_3v2_turn() {
        let party = default_party();
        let enemies = vec![Enemy::slime(), Enemy::slime()];
        let mut battle = BattleState::new(party, enemies);

        let commands = vec![
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Attack {
                target: TargetId::Enemy(1),
            },
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
        ];
        let randoms = make_random(vec![1.0; 5], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        // 全員が行動しているはず
        let attack_count = results
            .iter()
            .filter(|r| matches!(r, TurnResult::Attack { .. }))
            .count();
        assert!(attack_count >= 3); // パーティ3人 + 生存敵
    }

    #[test]
    fn speed_ordering() {
        let party = default_party();
        // Mage(SPD7) > Hero(SPD5) > Priest(SPD4) > Slime(SPD3)
        let enemies = vec![Enemy::slime()];
        let mut battle = BattleState::new(party, enemies);

        let commands = vec![
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
        ];
        let randoms = make_random(vec![1.0; 4], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        // 最初の攻撃は魔法使い(Party(1))のはず
        let first_attack = results.iter().find(|r| matches!(r, TurnResult::Attack { .. }));
        assert!(matches!(
            first_attack,
            Some(TurnResult::Attack {
                attacker: ActorId::Party(1),
                ..
            })
        ));
    }

    #[test]
    fn retarget_when_enemy_already_defeated() {
        let party = vec![PartyMember::hero(), PartyMember::mage()];
        let enemies = vec![Enemy::slime(), Enemy::slime()];
        let mut battle = BattleState::new(party, enemies);

        // 敵0を事前に倒す
        battle.enemies[0].stats.hp = 0;

        let commands = vec![
            BattleAction::Attack {
                target: TargetId::Enemy(0), // 倒されている → リターゲット
            },
            BattleAction::Attack {
                target: TargetId::Enemy(1),
            },
        ];
        let randoms = make_random(vec![1.0; 3], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        // 勇者の攻撃は敵1にリターゲットされているはず
        for result in &results {
            if let TurnResult::Attack {
                attacker: ActorId::Party(0),
                target,
                ..
            } = result
            {
                assert_eq!(*target, TargetId::Enemy(1));
            }
        }
    }

    #[test]
    fn flee_success() {
        let party = default_party();
        let enemies = vec![Enemy::slime()];
        let mut battle = BattleState::new(party, enemies);

        let commands = vec![
            BattleAction::Flee,
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
        ];
        let randoms = make_random(vec![1.0], 0.3); // 0.3 < 0.5 → 成功
        let results = battle.execute_turn(&commands, &randoms);

        assert_eq!(results, vec![TurnResult::Fled]);
        assert!(battle.is_over());
    }

    #[test]
    fn flee_failure() {
        let party = default_party();
        let enemies = vec![Enemy::slime()];
        let mut battle = BattleState::new(party, enemies);

        let commands = vec![
            BattleAction::Flee,
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
        ];
        let randoms = make_random(vec![1.0], 0.7); // 0.7 >= 0.5 → 失敗
        let results = battle.execute_turn(&commands, &randoms);

        assert!(matches!(results[0], TurnResult::FleeFailed));
        // 敵だけ行動する
        let enemy_attacks = results
            .iter()
            .filter(|r| {
                matches!(
                    r,
                    TurnResult::Attack {
                        attacker: ActorId::Enemy(_),
                        ..
                    }
                )
            })
            .count();
        assert!(enemy_attacks > 0);
        assert!(!battle.is_over());
    }

    #[test]
    fn victory_detection() {
        let party = default_party();
        let enemies = vec![Enemy::slime()];
        let mut battle = BattleState::new(party, enemies);

        // 敵を倒す
        battle.enemies[0].stats.hp = 0;
        assert!(battle.is_victory());
        assert!(battle.is_over());
        assert!(!battle.is_party_wiped());
    }

    #[test]
    fn party_wipe_detection() {
        let party = default_party();
        let enemies = vec![Enemy::slime()];
        let mut battle = BattleState::new(party, enemies);

        // 全員倒す
        for member in &mut battle.party {
            member.stats.hp = 0;
        }
        assert!(battle.is_party_wiped());
        assert!(battle.is_over());
        assert!(!battle.is_victory());
    }
}
