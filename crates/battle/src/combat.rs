use crate::enemy::Enemy;
use spell::{calculate_ailment_success, calculate_heal_amount, calculate_mp_drain, calculate_spell_damage, Ailment, SpellEffect, SpellKind, SpellTarget, POISON_DAMAGE};
use party::{CombatStats, ItemEffect, ItemKind, PartyMember};

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
    Spell { spell: SpellKind, target: TargetId },
    UseItem { item: ItemKind, target: TargetId },
    Flee,
}

/// バフ1種の状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuffState {
    pub amount: i32,
    pub remaining_turns: u32,
}

/// 1アクターのバフ群
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActorBuffs {
    pub attack_up: Option<BuffState>,
    pub defense_up: Option<BuffState>,
}

pub const BUFF_DURATION: u32 = 5;

/// 1アクターの状態異常
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActorAilments {
    pub sleep: bool,
    pub poison: bool,
}

impl ActorAilments {
    pub fn has_any(&self) -> bool {
        self.sleep || self.poison
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnResult {
    Attack {
        attacker: ActorId,
        target: TargetId,
        damage: i32,
    },
    SpellDamage {
        caster: ActorId,
        spell: SpellKind,
        target: TargetId,
        damage: i32,
    },
    Healed {
        caster: ActorId,
        spell: SpellKind,
        target: TargetId,
        amount: i32,
    },
    Buffed {
        caster: ActorId,
        spell: SpellKind,
        target: TargetId,
        amount: i32,
    },
    BuffExpired {
        target: TargetId,
        stat: BuffStat,
    },
    ItemUsed {
        user: ActorId,
        item: ItemKind,
        target: TargetId,
        amount: i32,
    },
    MpDrained {
        caster: ActorId,
        spell: SpellKind,
        target: TargetId,
        amount: i32,
    },
    Defeated {
        target: TargetId,
    },
    AilmentInflicted {
        caster: ActorId,
        spell: SpellKind,
        target: TargetId,
        ailment: Ailment,
    },
    AilmentResisted {
        caster: ActorId,
        spell: SpellKind,
        target: TargetId,
    },
    Sleeping {
        actor: ActorId,
    },
    PoisonDamage {
        target: TargetId,
        damage: i32,
    },
    AilmentCured {
        target: TargetId,
        ailment: Ailment,
    },
    Fled,
    FleeFailed,
}

/// バフが適用されるステータスの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffStat {
    Attack,
    Defense,
}

/// ターン実行に必要な乱数群
pub struct TurnRandomFactors {
    /// 各アクターのダメージ乱数(0.8~1.2)。indexはaction_order内の順番
    pub damage_randoms: Vec<f32>,
    /// 逃走判定用の乱数(0.0~1.0)。0.5未満で成功
    pub flee_random: f32,
    /// 敵ごとの呪文使用判定用乱数(0.0~1.0)。0.5未満で呪文使用
    pub spell_randoms: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct BattleState {
    pub party: Vec<PartyMember>,
    pub enemies: Vec<Enemy>,
    pub turn_log: Vec<TurnResult>,
    pub party_buffs: Vec<ActorBuffs>,
    pub party_ailments: Vec<ActorAilments>,
    pub enemy_ailments: Vec<ActorAilments>,
}

impl BattleState {
    pub fn new(party: Vec<PartyMember>, enemies: Vec<Enemy>) -> Self {
        let party_count = party.len();
        let enemy_count = enemies.len();
        Self {
            party,
            enemies,
            turn_log: Vec::new(),
            party_buffs: vec![ActorBuffs::default(); party_count],
            party_ailments: vec![ActorAilments::default(); party_count],
            enemy_ailments: vec![ActorAilments::default(); enemy_count],
        }
    }

    /// パーティメンバーの実効攻撃力（バフ込み）
    pub fn effective_attack_with_buff(&self, party_idx: usize) -> i32 {
        let base = self.party[party_idx].effective_attack();
        let buff_amount = self.party_buffs[party_idx]
            .attack_up
            .map(|b| b.amount)
            .unwrap_or(0);
        base + buff_amount
    }

    /// パーティメンバーの実効防御力（バフ込み）
    pub fn effective_defense_with_buff(&self, party_idx: usize) -> i32 {
        let base = self.party[party_idx].stats.defense;
        let buff_amount = self.party_buffs[party_idx]
            .defense_up
            .map(|b| b.amount)
            .unwrap_or(0);
        base + buff_amount
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
                    let spell_random = random_factors
                        .spell_randoms
                        .get(ei)
                        .copied()
                        .unwrap_or(1.0);
                    results.extend(self.execute_enemy_action(ei, random, spell_random));
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
                    if self.party_ailments[pi].sleep {
                        results.push(TurnResult::Sleeping {
                            actor: ActorId::Party(pi),
                        });
                        continue;
                    }
                    match party_commands.get(pi) {
                        Some(BattleAction::Attack { target }) => {
                            let actual_target = self.retarget_enemy(*target);
                            if let Some(actual_target) = actual_target {
                                results
                                    .extend(self.execute_party_attack(pi, actual_target, random));
                            }
                        }
                        Some(BattleAction::Spell { spell, target }) => {
                            results.extend(self.execute_spell(pi, *spell, *target, random));
                        }
                        Some(BattleAction::UseItem { item, target }) => {
                            results.extend(self.execute_item(pi, *item, *target, random));
                        }
                        _ => {}
                    }
                }
                ActorId::Enemy(ei) => {
                    let ei = *ei;
                    if !self.enemies[ei].stats.is_alive() {
                        continue;
                    }
                    if self.enemy_ailments[ei].sleep {
                        results.push(TurnResult::Sleeping {
                            actor: ActorId::Enemy(ei),
                        });
                        continue;
                    }
                    let spell_random = random_factors
                        .spell_randoms
                        .get(ei)
                        .copied()
                        .unwrap_or(1.0);
                    results.extend(self.execute_enemy_action(ei, random, spell_random));
                }
            }
        }

        // ターン終了時: バフのtick
        results.extend(self.tick_buffs());

        // ターン終了時: 毒ダメージ
        results.extend(self.tick_poison());

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

    /// 呪文実行
    fn execute_spell(
        &mut self,
        caster_idx: usize,
        spell: SpellKind,
        target: TargetId,
        random_factor: f32,
    ) -> Vec<TurnResult> {
        let mut results = Vec::new();

        // MP消費
        if !self.party[caster_idx].stats.use_mp(spell.mp_cost()) {
            return results;
        }

        match spell.effect() {
            SpellEffect::Damage => {
                match spell.target_type() {
                    SpellTarget::SingleEnemy => {
                        let actual_target = self.retarget_enemy(target);
                        if let Some(TargetId::Enemy(ei)) = actual_target {
                            let damage = calculate_spell_damage(
                                spell.power(),
                                self.enemies[ei].stats.defense,
                                random_factor,
                            );
                            self.enemies[ei].stats.take_damage(damage);
                            results.push(TurnResult::SpellDamage {
                                caster: ActorId::Party(caster_idx),
                                spell,
                                target: TargetId::Enemy(ei),
                                damage,
                            });
                            if !self.enemies[ei].stats.is_alive() {
                                results.push(TurnResult::Defeated {
                                    target: TargetId::Enemy(ei),
                                });
                            }
                        }
                    }
                    SpellTarget::AllEnemies => {
                        for ei in self.alive_enemy_indices() {
                            let damage = calculate_spell_damage(
                                spell.power(),
                                self.enemies[ei].stats.defense,
                                random_factor,
                            );
                            self.enemies[ei].stats.take_damage(damage);
                            results.push(TurnResult::SpellDamage {
                                caster: ActorId::Party(caster_idx),
                                spell,
                                target: TargetId::Enemy(ei),
                                damage,
                            });
                            if !self.enemies[ei].stats.is_alive() {
                                results.push(TurnResult::Defeated {
                                    target: TargetId::Enemy(ei),
                                });
                            }
                        }
                    }
                    _ => {} // Damage spells don't target allies
                }
            }
            SpellEffect::Heal => {
                match spell.target_type() {
                    SpellTarget::SingleAlly => {
                        let actual_target = self.retarget_ally(target);
                        if let Some(TargetId::Party(pi)) = actual_target {
                            let amount = calculate_heal_amount(spell.power(), random_factor);
                            let member = &mut self.party[pi];
                            member.stats.hp = (member.stats.hp + amount).min(member.stats.max_hp);
                            results.push(TurnResult::Healed {
                                caster: ActorId::Party(caster_idx),
                                spell,
                                target: TargetId::Party(pi),
                                amount,
                            });
                        }
                    }
                    SpellTarget::AllAllies => {
                        for pi in self.alive_party_indices() {
                            let amount = calculate_heal_amount(spell.power(), random_factor);
                            let member = &mut self.party[pi];
                            member.stats.hp = (member.stats.hp + amount).min(member.stats.max_hp);
                            results.push(TurnResult::Healed {
                                caster: ActorId::Party(caster_idx),
                                spell,
                                target: TargetId::Party(pi),
                                amount,
                            });
                        }
                    }
                    _ => {} // Heal spells don't target enemies
                }
            }
            SpellEffect::AttackBuff => {
                match spell.target_type() {
                    SpellTarget::SingleAlly => {
                        let actual_target = self.retarget_ally(target);
                        if let Some(TargetId::Party(pi)) = actual_target {
                            self.party_buffs[pi].attack_up = Some(BuffState {
                                amount: spell.power(),
                                remaining_turns: BUFF_DURATION,
                            });
                            results.push(TurnResult::Buffed {
                                caster: ActorId::Party(caster_idx),
                                spell,
                                target: TargetId::Party(pi),
                                amount: spell.power(),
                            });
                        }
                    }
                    SpellTarget::AllAllies => {
                        for pi in self.alive_party_indices() {
                            self.party_buffs[pi].attack_up = Some(BuffState {
                                amount: spell.power(),
                                remaining_turns: BUFF_DURATION,
                            });
                            results.push(TurnResult::Buffed {
                                caster: ActorId::Party(caster_idx),
                                spell,
                                target: TargetId::Party(pi),
                                amount: spell.power(),
                            });
                        }
                    }
                    _ => {}
                }
            }
            SpellEffect::DefenseBuff => {
                match spell.target_type() {
                    SpellTarget::SingleAlly => {
                        let actual_target = self.retarget_ally(target);
                        if let Some(TargetId::Party(pi)) = actual_target {
                            self.party_buffs[pi].defense_up = Some(BuffState {
                                amount: spell.power(),
                                remaining_turns: BUFF_DURATION,
                            });
                            results.push(TurnResult::Buffed {
                                caster: ActorId::Party(caster_idx),
                                spell,
                                target: TargetId::Party(pi),
                                amount: spell.power(),
                            });
                        }
                    }
                    SpellTarget::AllAllies => {
                        for pi in self.alive_party_indices() {
                            self.party_buffs[pi].defense_up = Some(BuffState {
                                amount: spell.power(),
                                remaining_turns: BUFF_DURATION,
                            });
                            results.push(TurnResult::Buffed {
                                caster: ActorId::Party(caster_idx),
                                spell,
                                target: TargetId::Party(pi),
                                amount: spell.power(),
                            });
                        }
                    }
                    _ => {}
                }
            }
            SpellEffect::MpDrain => {
                match spell.target_type() {
                    SpellTarget::SingleEnemy => {
                        let actual_target = self.retarget_enemy(target);
                        if let Some(TargetId::Enemy(ei)) = actual_target {
                            let amount = calculate_mp_drain(spell.power(), random_factor);
                            self.enemies[ei].stats.drain_mp(amount);
                            results.push(TurnResult::MpDrained {
                                caster: ActorId::Party(caster_idx),
                                spell,
                                target: TargetId::Enemy(ei),
                                amount,
                            });
                        }
                    }
                    SpellTarget::AllEnemies => {
                        for ei in self.alive_enemy_indices() {
                            let amount = calculate_mp_drain(spell.power(), random_factor);
                            self.enemies[ei].stats.drain_mp(amount);
                            results.push(TurnResult::MpDrained {
                                caster: ActorId::Party(caster_idx),
                                spell,
                                target: TargetId::Enemy(ei),
                                amount,
                            });
                        }
                    }
                    _ => {}
                }
            }
            SpellEffect::Ailment => {
                let ailment = spell.ailment().expect("Ailment spell must have ailment");
                let success_rate = spell.power();
                match spell.target_type() {
                    SpellTarget::SingleEnemy => {
                        let actual_target = self.retarget_enemy(target);
                        if let Some(TargetId::Enemy(ei)) = actual_target {
                            if calculate_ailment_success(success_rate, random_factor) {
                                self.apply_ailment_to_enemy(ei, ailment);
                                results.push(TurnResult::AilmentInflicted {
                                    caster: ActorId::Party(caster_idx),
                                    spell,
                                    target: TargetId::Enemy(ei),
                                    ailment,
                                });
                            } else {
                                results.push(TurnResult::AilmentResisted {
                                    caster: ActorId::Party(caster_idx),
                                    spell,
                                    target: TargetId::Enemy(ei),
                                });
                            }
                        }
                    }
                    SpellTarget::AllEnemies => {
                        for ei in self.alive_enemy_indices() {
                            if calculate_ailment_success(success_rate, random_factor) {
                                self.apply_ailment_to_enemy(ei, ailment);
                                results.push(TurnResult::AilmentInflicted {
                                    caster: ActorId::Party(caster_idx),
                                    spell,
                                    target: TargetId::Enemy(ei),
                                    ailment,
                                });
                            } else {
                                results.push(TurnResult::AilmentResisted {
                                    caster: ActorId::Party(caster_idx),
                                    spell,
                                    target: TargetId::Enemy(ei),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        results
    }

    /// アイテム使用
    fn execute_item(
        &mut self,
        user_idx: usize,
        item: ItemKind,
        target: TargetId,
        random_factor: f32,
    ) -> Vec<TurnResult> {
        let mut results = Vec::new();

        match item.effect() {
            ItemEffect::Heal { power } => {
                if !self.party[user_idx].inventory.use_item(item) {
                    return results;
                }

                let actual_target = self.retarget_ally(target);
                if let Some(TargetId::Party(pi)) = actual_target {
                    let amount = calculate_heal_amount(power, random_factor);
                    let member = &mut self.party[pi];
                    member.stats.hp = (member.stats.hp + amount).min(member.stats.max_hp);
                    results.push(TurnResult::ItemUsed {
                        user: ActorId::Party(user_idx),
                        item,
                        target: TargetId::Party(pi),
                        amount,
                    });
                }
            }
            ItemEffect::KeyItem | ItemEffect::Material | ItemEffect::Equip => {}

        }

        results
    }

    /// ターゲットの味方が既に倒されていたら最初の生存味方にリターゲット
    fn retarget_ally(&self, target: TargetId) -> Option<TargetId> {
        if let TargetId::Party(pi) = target {
            if self.party[pi].stats.is_alive() {
                return Some(target);
            }
            self.alive_party_indices()
                .first()
                .map(|&i| TargetId::Party(i))
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
                self.effective_attack_with_buff(party_idx),
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
            } else if let Some(cure) = self.wake_up_if_sleeping(&target) {
                results.push(cure);
            }
        }
        results
    }

    /// 敵の行動選択: 呪文使用可能なら50%で呪文、それ以外は物理攻撃
    fn execute_enemy_action(
        &mut self,
        enemy_idx: usize,
        random_factor: f32,
        spell_random: f32,
    ) -> Vec<TurnResult> {
        let spells = self.enemies[enemy_idx].kind.spells();
        if !spells.is_empty() && spell_random < 0.5 {
            // 使用可能な呪文からMP足りるものを選択
            let usable: Vec<SpellKind> = spells
                .iter()
                .filter(|s| self.enemies[enemy_idx].stats.mp >= s.mp_cost())
                .copied()
                .collect();
            if let Some(&spell) = usable.first() {
                return self.execute_enemy_spell(enemy_idx, spell, random_factor);
            }
        }
        self.execute_enemy_attack(enemy_idx, random_factor)
    }

    /// 敵の呪文実行（ターゲット方向を逆転: Damage→パーティ, Heal→自身）
    fn execute_enemy_spell(
        &mut self,
        enemy_idx: usize,
        spell: SpellKind,
        random_factor: f32,
    ) -> Vec<TurnResult> {
        let mut results = Vec::new();

        // MP消費
        if !self.enemies[enemy_idx].stats.use_mp(spell.mp_cost()) {
            return self.execute_enemy_attack(enemy_idx, random_factor);
        }

        match spell.effect() {
            SpellEffect::Damage => {
                match spell.target_type() {
                    SpellTarget::SingleEnemy => {
                        // 敵から見て「敵」= パーティメンバー → 先頭の生存メンバーを攻撃
                        let alive_party = self.alive_party_indices();
                        if let Some(&pi) = alive_party.first() {
                            let damage = calculate_spell_damage(
                                spell.power(),
                                self.effective_defense_with_buff(pi),
                                random_factor,
                            );
                            self.party[pi].stats.take_damage(damage);
                            results.push(TurnResult::SpellDamage {
                                caster: ActorId::Enemy(enemy_idx),
                                spell,
                                target: TargetId::Party(pi),
                                damage,
                            });
                            if !self.party[pi].stats.is_alive() {
                                results.push(TurnResult::Defeated {
                                    target: TargetId::Party(pi),
                                });
                            }
                        }
                    }
                    SpellTarget::AllEnemies => {
                        // 敵から見て「全体敵」= パーティ全員
                        for pi in self.alive_party_indices() {
                            let damage = calculate_spell_damage(
                                spell.power(),
                                self.effective_defense_with_buff(pi),
                                random_factor,
                            );
                            self.party[pi].stats.take_damage(damage);
                            results.push(TurnResult::SpellDamage {
                                caster: ActorId::Enemy(enemy_idx),
                                spell,
                                target: TargetId::Party(pi),
                                damage,
                            });
                            if !self.party[pi].stats.is_alive() {
                                results.push(TurnResult::Defeated {
                                    target: TargetId::Party(pi),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            SpellEffect::Heal => {
                // 自身を回復
                let amount = calculate_heal_amount(spell.power(), random_factor);
                let enemy = &mut self.enemies[enemy_idx];
                enemy.stats.hp = (enemy.stats.hp + amount).min(enemy.stats.max_hp);
                results.push(TurnResult::Healed {
                    caster: ActorId::Enemy(enemy_idx),
                    spell,
                    target: TargetId::Enemy(enemy_idx),
                    amount,
                });
            }
            SpellEffect::MpDrain => {
                match spell.target_type() {
                    SpellTarget::SingleEnemy => {
                        // 敵から見て「敵」= パーティメンバー → 先頭の生存メンバーのMP減少
                        let alive_party = self.alive_party_indices();
                        if let Some(&pi) = alive_party.first() {
                            let amount = calculate_mp_drain(spell.power(), random_factor);
                            self.party[pi].stats.drain_mp(amount);
                            results.push(TurnResult::MpDrained {
                                caster: ActorId::Enemy(enemy_idx),
                                spell,
                                target: TargetId::Party(pi),
                                amount,
                            });
                        }
                    }
                    SpellTarget::AllEnemies => {
                        // 敵から見て「全体敵」= パーティ全員のMP減少
                        for pi in self.alive_party_indices() {
                            let amount = calculate_mp_drain(spell.power(), random_factor);
                            self.party[pi].stats.drain_mp(amount);
                            results.push(TurnResult::MpDrained {
                                caster: ActorId::Enemy(enemy_idx),
                                spell,
                                target: TargetId::Party(pi),
                                amount,
                            });
                        }
                    }
                    _ => {}
                }
            }
            SpellEffect::Ailment => {
                let ailment = spell.ailment().expect("Ailment spell must have ailment");
                let success_rate = spell.power();
                match spell.target_type() {
                    SpellTarget::SingleEnemy => {
                        let alive_party = self.alive_party_indices();
                        if let Some(&pi) = alive_party.first() {
                            if calculate_ailment_success(success_rate, random_factor) {
                                self.apply_ailment_to_party(pi, ailment);
                                results.push(TurnResult::AilmentInflicted {
                                    caster: ActorId::Enemy(enemy_idx),
                                    spell,
                                    target: TargetId::Party(pi),
                                    ailment,
                                });
                            } else {
                                results.push(TurnResult::AilmentResisted {
                                    caster: ActorId::Enemy(enemy_idx),
                                    spell,
                                    target: TargetId::Party(pi),
                                });
                            }
                        }
                    }
                    SpellTarget::AllEnemies => {
                        for pi in self.alive_party_indices() {
                            if calculate_ailment_success(success_rate, random_factor) {
                                self.apply_ailment_to_party(pi, ailment);
                                results.push(TurnResult::AilmentInflicted {
                                    caster: ActorId::Enemy(enemy_idx),
                                    spell,
                                    target: TargetId::Party(pi),
                                    ailment,
                                });
                            } else {
                                results.push(TurnResult::AilmentResisted {
                                    caster: ActorId::Enemy(enemy_idx),
                                    spell,
                                    target: TargetId::Party(pi),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            // バフ呪文は敵には未実装
            _ => {}
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
            self.effective_defense_with_buff(target_idx),
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
        } else if let Some(cure) = self.wake_up_if_sleeping(&target) {
            results.push(cure);
        }
        results
    }

    /// 敵に状態異常を付与
    fn apply_ailment_to_enemy(&mut self, ei: usize, ailment: Ailment) {
        match ailment {
            Ailment::Sleep => self.enemy_ailments[ei].sleep = true,
            Ailment::Poison => self.enemy_ailments[ei].poison = true,
        }
    }

    /// 味方に状態異常を付与
    fn apply_ailment_to_party(&mut self, pi: usize, ailment: Ailment) {
        match ailment {
            Ailment::Sleep => self.party_ailments[pi].sleep = true,
            Ailment::Poison => self.party_ailments[pi].poison = true,
        }
    }

    /// ターン終了時の毒ダメージ処理
    fn tick_poison(&mut self) -> Vec<TurnResult> {
        let mut results = Vec::new();

        for pi in 0..self.party.len() {
            if self.party[pi].stats.is_alive() && self.party_ailments[pi].poison {
                self.party[pi].stats.take_damage(POISON_DAMAGE);
                results.push(TurnResult::PoisonDamage {
                    target: TargetId::Party(pi),
                    damage: POISON_DAMAGE,
                });
                if !self.party[pi].stats.is_alive() {
                    results.push(TurnResult::Defeated {
                        target: TargetId::Party(pi),
                    });
                }
            }
        }

        for ei in 0..self.enemies.len() {
            if self.enemies[ei].stats.is_alive() && self.enemy_ailments[ei].poison {
                self.enemies[ei].stats.take_damage(POISON_DAMAGE);
                results.push(TurnResult::PoisonDamage {
                    target: TargetId::Enemy(ei),
                    damage: POISON_DAMAGE,
                });
                if !self.enemies[ei].stats.is_alive() {
                    results.push(TurnResult::Defeated {
                        target: TargetId::Enemy(ei),
                    });
                }
            }
        }

        results
    }

    /// 攻撃を受けた対象の眠りを解除
    fn wake_up_if_sleeping(&mut self, target: &TargetId) -> Option<TurnResult> {
        match target {
            TargetId::Enemy(ei) => {
                if self.enemy_ailments[*ei].sleep {
                    self.enemy_ailments[*ei].sleep = false;
                    Some(TurnResult::AilmentCured {
                        target: *target,
                        ailment: Ailment::Sleep,
                    })
                } else {
                    None
                }
            }
            TargetId::Party(pi) => {
                if self.party_ailments[*pi].sleep {
                    self.party_ailments[*pi].sleep = false;
                    Some(TurnResult::AilmentCured {
                        target: *target,
                        ailment: Ailment::Sleep,
                    })
                } else {
                    None
                }
            }
        }
    }

    /// ターン終了時にバフの残りターンをデクリメント、0になったバフを除去
    fn tick_buffs(&mut self) -> Vec<TurnResult> {
        let mut results = Vec::new();

        for pi in 0..self.party_buffs.len() {
            if let Some(ref mut buff) = self.party_buffs[pi].attack_up {
                buff.remaining_turns = buff.remaining_turns.saturating_sub(1);
                if buff.remaining_turns == 0 {
                    self.party_buffs[pi].attack_up = None;
                    results.push(TurnResult::BuffExpired {
                        target: TargetId::Party(pi),
                        stat: BuffStat::Attack,
                    });
                }
            }
            if let Some(ref mut buff) = self.party_buffs[pi].defense_up {
                buff.remaining_turns = buff.remaining_turns.saturating_sub(1);
                if buff.remaining_turns == 0 {
                    self.party_buffs[pi].defense_up = None;
                    results.push(TurnResult::BuffExpired {
                        target: TargetId::Party(pi),
                        stat: BuffStat::Defense,
                    });
                }
            }
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

    /// 倒した敵の合計経験値を計算（段階補正込み）
    pub fn total_exp_reward(&self) -> u32 {
        self.enemies
            .iter()
            .filter(|e| !e.stats.is_alive())
            .map(|e| e.exp_reward())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemy::{Enemy, EnemyKind};
    use party::{default_party, PartyMember};

    fn make_random(damage_randoms: Vec<f32>, flee_random: f32) -> TurnRandomFactors {
        TurnRandomFactors {
            damage_randoms,
            flee_random,
            spell_randoms: vec![1.0; 10], // 1.0 = 呪文不使用（既存テスト互換）
        }
    }

    fn make_random_with_spells(
        damage_randoms: Vec<f32>,
        flee_random: f32,
        spell_randoms: Vec<f32>,
    ) -> TurnRandomFactors {
        TurnRandomFactors {
            damage_randoms,
            flee_random,
            spell_randoms,
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
        // Marcille(SPD7) > Laios(SPD5) > Falin(SPD4) > Slime(SPD3)
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

        // 最初の攻撃はマルシル(Party(1))のはず
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
        let party = vec![PartyMember::laios(), PartyMember::marcille()];
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

        // ライオスの攻撃は敵1にリターゲットされているはず
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

    #[test]
    fn zola_spell_damages_enemy() {
        let party = default_party();
        let enemies = vec![Enemy::slime()];
        let mut battle = BattleState::new(party, enemies);

        let mage_mp_before = battle.party[1].stats.mp;
        let commands = vec![
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Spell {
                spell: SpellKind::Fire1,
                target: TargetId::Enemy(0),
            },
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
        ];
        let randoms = make_random(vec![1.0; 4], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        // SpellDamageが含まれる
        let spell_results: Vec<_> = results
            .iter()
            .filter(|r| matches!(r, TurnResult::SpellDamage { .. }))
            .collect();
        assert_eq!(spell_results.len(), 1);

        // MP消費
        assert_eq!(battle.party[1].stats.mp, mage_mp_before - 3);
    }

    #[test]
    fn luna_spell_restores_hp() {
        let party = default_party();
        // HP999のスライムで戦闘が終わらないようにする
        let mut slime = Enemy::slime();
        slime.stats.hp = 999;
        slime.stats.max_hp = 999;
        let enemies = vec![slime];
        let mut battle = BattleState::new(party, enemies);

        // ライオスのHPを減らす
        battle.party[0].stats.hp = 10;

        let commands = vec![
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Spell {
                spell: SpellKind::Heal1,
                target: TargetId::Party(0),
            },
        ];
        let randoms = make_random(vec![1.0; 4], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        // Healedが含まれる
        let heal_results: Vec<_> = results
            .iter()
            .filter(|r| matches!(r, TurnResult::Healed { .. }))
            .collect();
        assert_eq!(heal_results.len(), 1);

        // HPが回復している（上限を超えない）
        assert!(battle.party[0].stats.hp > 10);
        assert!(battle.party[0].stats.hp <= battle.party[0].stats.max_hp);
    }

    #[test]
    fn heal_retargets_to_alive_ally() {
        let party = default_party();
        let enemies = vec![Enemy::slime()];
        let mut battle = BattleState::new(party, enemies);

        // ライオスを倒す
        battle.party[0].stats.hp = 0;

        let commands = vec![
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Spell {
                spell: SpellKind::Heal1,
                target: TargetId::Party(0), // 倒されている → リターゲット
            },
        ];
        let randoms = make_random(vec![1.0; 3], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        // リターゲットされて生存味方(Party(1))に回復
        for result in &results {
            if let TurnResult::Healed { target, .. } = result {
                assert_eq!(*target, TargetId::Party(1));
            }
        }
    }

    #[test]
    fn dead_hero_priest_still_attacks() {
        let party = default_party();
        let mut slime = Enemy::slime();
        slime.stats.hp = 999;
        slime.stats.max_hp = 999;
        let enemies = vec![slime];
        let mut battle = BattleState::new(party, enemies);

        // ライオスを倒す
        battle.party[0].stats.hp = 0;

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
        let randoms = make_random(vec![1.0; 3], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        let mage_attacks = results
            .iter()
            .filter(|r| {
                matches!(
                    r,
                    TurnResult::Attack {
                        attacker: ActorId::Party(1),
                        ..
                    }
                )
            })
            .count();
        let priest_attacks = results
            .iter()
            .filter(|r| {
                matches!(
                    r,
                    TurnResult::Attack {
                        attacker: ActorId::Party(2),
                        ..
                    }
                )
            })
            .count();

        assert_eq!(mage_attacks, 1, "マルシルは1回攻撃するはず");
        assert_eq!(priest_attacks, 1, "ファリンは1回攻撃するはず");

        let hero_attacks = results
            .iter()
            .filter(|r| {
                matches!(
                    r,
                    TurnResult::Attack {
                        attacker: ActorId::Party(0),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(hero_attacks, 0, "死亡したライオスは攻撃しないはず");
    }

    #[test]
    fn neld_aoe_damages_all_enemies() {
        let party = vec![PartyMember::marcille()];
        let enemies = vec![Enemy::slime(), Enemy::slime(), Enemy::slime()];
        let mut battle = BattleState::new(party, enemies);

        let commands = vec![BattleAction::Spell {
            spell: SpellKind::Blaze1,
            target: TargetId::Enemy(0), // AoEなのでダミー
        }];
        let randoms = make_random(vec![1.0; 4], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        let spell_hits: Vec<_> = results
            .iter()
            .filter(|r| matches!(r, TurnResult::SpellDamage { .. }))
            .collect();
        assert_eq!(spell_hits.len(), 3, "全3体の敵にヒットするはず");
    }

    #[test]
    fn panam_aoe_heals_all_allies() {
        // ファリンLv3以上でPanamを習得
        let mut falin = PartyMember::falin();
        falin.level = 3;
        let mut laios = PartyMember::laios();
        laios.stats.hp = 5;
        let mut marcille = PartyMember::marcille();
        marcille.stats.hp = 5;
        falin.stats.hp = 5;

        let party = vec![laios, marcille, falin];
        let mut slime = Enemy::slime();
        slime.stats.hp = 999;
        slime.stats.max_hp = 999;
        slime.stats.attack = 0;
        let enemies = vec![slime];
        let mut battle = BattleState::new(party, enemies);

        let commands = vec![
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Spell {
                spell: SpellKind::Healall1,
                target: TargetId::Party(0), // AoEなのでダミー
            },
        ];
        let randoms = make_random(vec![1.0; 4], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        let heal_hits: Vec<_> = results
            .iter()
            .filter(|r| matches!(r, TurnResult::Healed { .. }))
            .collect();
        assert_eq!(heal_hits.len(), 3, "全3人の味方が回復するはず");
    }

    #[test]
    fn bolga_buff_increases_attack() {
        let mut rinsha = PartyMember::rinsha();
        rinsha.level = 5; // Bolga習得
        let laios = PartyMember::laios();

        let party = vec![laios, rinsha];
        let mut slime = Enemy::slime();
        slime.stats.hp = 999;
        slime.stats.max_hp = 999;
        slime.stats.attack = 0;
        let enemies = vec![slime];
        let mut battle = BattleState::new(party, enemies);

        let base_attack = battle.effective_attack_with_buff(0);

        let commands = vec![
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Spell {
                spell: SpellKind::Boost1,
                target: TargetId::Party(0),
            },
        ];
        let randoms = make_random(vec![1.0; 3], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        // Buffedイベントが発生
        let buffed = results
            .iter()
            .any(|r| matches!(r, TurnResult::Buffed { .. }));
        assert!(buffed, "Buffedイベントが発生するはず");

        // 攻撃力が上昇
        let buffed_attack = battle.effective_attack_with_buff(0);
        assert_eq!(buffed_attack, base_attack + 3, "ATK+3のはず");
    }

    #[test]
    fn garde_buff_reduces_damage_taken() {
        let mut senshi = PartyMember::senshi();
        senshi.level = 4; // Garde習得
        let laios = PartyMember::laios();

        let party = vec![laios, senshi];
        let mut wolf = Enemy::wolf();
        wolf.stats.attack = 20;
        wolf.stats.hp = 999;
        wolf.stats.max_hp = 999;
        let enemies = vec![wolf];
        let mut battle = BattleState::new(party, enemies);

        let base_defense = battle.effective_defense_with_buff(0);

        let commands = vec![
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
            BattleAction::Spell {
                spell: SpellKind::Shield1,
                target: TargetId::Party(0),
            },
        ];
        let randoms = make_random(vec![1.0; 3], 0.0);
        battle.execute_turn(&commands, &randoms);

        let buffed_defense = battle.effective_defense_with_buff(0);
        assert_eq!(buffed_defense, base_defense + 3, "DEF+3のはず");
    }

    #[test]
    fn buff_expires_after_5_turns() {
        let mut rinsha = PartyMember::rinsha();
        rinsha.level = 5;
        let party = vec![rinsha];
        let mut slime = Enemy::slime();
        slime.stats.hp = 999;
        slime.stats.max_hp = 999;
        slime.stats.attack = 0;
        let enemies = vec![slime];
        let mut battle = BattleState::new(party, enemies);

        // ターン1: バフ付与
        let commands = vec![BattleAction::Spell {
            spell: SpellKind::Boost1,
            target: TargetId::Party(0),
        }];
        let randoms = make_random(vec![1.0; 2], 0.0);
        battle.execute_turn(&commands, &randoms);
        assert!(battle.party_buffs[0].attack_up.is_some());

        // ターン2~5: バフ持続
        for _ in 0..4 {
            let commands = vec![BattleAction::Attack {
                target: TargetId::Enemy(0),
            }];
            let randoms = make_random(vec![1.0; 2], 0.0);
            let results = battle.execute_turn(&commands, &randoms);
            // 最後のターンでBuffExpiredが出る
            if battle.party_buffs[0].attack_up.is_none() {
                let expired = results
                    .iter()
                    .any(|r| matches!(r, TurnResult::BuffExpired { .. }));
                assert!(expired, "BuffExpiredが発生するはず");
            }
        }

        assert!(
            battle.party_buffs[0].attack_up.is_none(),
            "5ターン後にバフが消失するはず"
        );
    }

    #[test]
    fn buff_overwrite_resets_duration() {
        let mut rinsha = PartyMember::rinsha();
        rinsha.level = 7; // Bolgarda(ATK+6)も習得
        rinsha.stats.mp = 99;
        rinsha.stats.max_mp = 99;
        let party = vec![rinsha];
        let mut slime = Enemy::slime();
        slime.stats.hp = 999;
        slime.stats.max_hp = 999;
        slime.stats.attack = 0;
        let enemies = vec![slime];
        let mut battle = BattleState::new(party, enemies);

        // Bolga(ATK+3)を付与
        let commands = vec![BattleAction::Spell {
            spell: SpellKind::Boost1,
            target: TargetId::Party(0),
        }];
        let randoms = make_random(vec![1.0; 2], 0.0);
        battle.execute_turn(&commands, &randoms);
        assert_eq!(battle.party_buffs[0].attack_up.unwrap().amount, 3);

        // 3ターン経過
        for _ in 0..3 {
            let commands = vec![BattleAction::Attack {
                target: TargetId::Enemy(0),
            }];
            let randoms = make_random(vec![1.0; 2], 0.0);
            battle.execute_turn(&commands, &randoms);
        }
        // remaining_turns = 5 - 1(付与ターン) - 3 = 1
        assert!(battle.party_buffs[0].attack_up.is_some());

        // Bolgarda(ATK+6)で上書き → 持続5にリセット
        let commands = vec![BattleAction::Spell {
            spell: SpellKind::Boost2,
            target: TargetId::Party(0),
        }];
        let randoms = make_random(vec![1.0; 2], 0.0);
        battle.execute_turn(&commands, &randoms);
        let buff = battle.party_buffs[0].attack_up.unwrap();
        assert_eq!(buff.amount, 6, "上書き後はATK+6");
        // tick_buffsで1減るので remaining_turns = 5 - 1 = 4
        assert_eq!(buff.remaining_turns, 4, "上書き後の持続ターンは4(5-1tick)");
    }

    #[test]
    fn enemy_spell_damages_party() {
        let party = vec![PartyMember::laios()];
        let ghost = Enemy::ghost();
        let enemies = vec![ghost];
        let mut battle = BattleState::new(party, enemies);

        let hp_before = battle.party[0].stats.hp;

        let commands = vec![BattleAction::Attack {
            target: TargetId::Enemy(0),
        }];
        // spell_randoms[0] = 0.0 → 呪文使用
        let randoms = make_random_with_spells(vec![1.0; 2], 0.0, vec![0.0]);
        let results = battle.execute_turn(&commands, &randoms);

        let spell_hits: Vec<_> = results
            .iter()
            .filter(|r| matches!(r, TurnResult::SpellDamage { caster: ActorId::Enemy(0), .. }))
            .collect();
        assert!(!spell_hits.is_empty(), "ゴーストがFire1を唱えるはず");
        assert!(battle.party[0].stats.hp < hp_before, "パーティにダメージが入るはず");
    }

    #[test]
    fn enemy_falls_back_to_attack_when_mp_empty() {
        let party = vec![PartyMember::laios()];
        let mut ghost = Enemy::ghost();
        ghost.stats.mp = 0; // MP枯渇
        let enemies = vec![ghost];
        let mut battle = BattleState::new(party, enemies);

        let commands = vec![BattleAction::Attack {
            target: TargetId::Enemy(0),
        }];
        // spell_randoms[0] = 0.0 → 呪文使用を試みるがMP不足でフォールバック
        let randoms = make_random_with_spells(vec![1.0; 2], 0.0, vec![0.0]);
        let results = battle.execute_turn(&commands, &randoms);

        let enemy_attacks = results
            .iter()
            .filter(|r| matches!(r, TurnResult::Attack { attacker: ActorId::Enemy(0), .. }))
            .count();
        assert!(enemy_attacks > 0, "MP不足時は物理攻撃にフォールバックするはず");
    }

    #[test]
    fn enemy_heal_spell_restores_hp() {
        let party = vec![PartyMember::laios()];
        let mut dark_lord = Enemy::dark_lord();
        dark_lord.stats.hp = 50; // HPを減らす
        let enemies = vec![dark_lord];
        let mut battle = BattleState::new(party, enemies);

        // DarkLordの呪文: [Blaze2, Fire2, Heal2] — 最初のMP足りるものを使う
        // Heal2(MP7)を使わせるために、Blaze2(MP10)とFire2(MP7)のMPを消費させる
        battle.enemies[0].stats.mp = 7; // Blaze2(10)は不可、Fire2(7)は可だが先にくる
        // Fire2が先に選ばれるので、Fire2のMPも使えないようにする
        // DarkLordの呪文テーブル: [Blaze2, Fire2, Heal2]
        // Blaze2 cost=10 > 7 → skip, Fire2 cost=7 <= 7 → 選択される（Damage呪文）
        // Heal2を使わせるには、Fire2もスキップさせる必要がある
        battle.enemies[0].stats.mp = 50; // フルMPにして、手動でHeal呪文をテスト

        // 直接execute_enemy_spellをテストする代わりに、呪文テーブルの仕組みを確認
        // DarkLordの呪文テーブル: [Blaze2, Fire2, Heal2]
        // usable.first()はBlaze2を返す → Damage呪文
        // Heal呪文のテストは直接実行
        let hp_before = battle.enemies[0].stats.hp;
        let results = battle.execute_enemy_spell(0, SpellKind::Heal2, 1.0);
        assert!(
            results.iter().any(|r| matches!(r, TurnResult::Healed { .. })),
            "Heal2で回復イベントが発生するはず"
        );
        assert!(battle.enemies[0].stats.hp > hp_before, "敵のHPが回復するはず");
    }

    #[test]
    fn enemy_aoe_spell_hits_all_party() {
        let party = vec![PartyMember::laios(), PartyMember::marcille(), PartyMember::falin()];
        let dragon = Enemy::new(EnemyKind::Dragon, 1);
        let enemies = vec![dragon];
        let mut battle = BattleState::new(party, enemies);

        let commands = vec![
            BattleAction::Attack { target: TargetId::Enemy(0) },
            BattleAction::Attack { target: TargetId::Enemy(0) },
            BattleAction::Attack { target: TargetId::Enemy(0) },
        ];
        // spell_randoms[0] = 0.0 → 呪文使用（Blaze2 = AllEnemies → パーティ全員）
        let randoms = make_random_with_spells(vec![1.0; 4], 0.0, vec![0.0]);
        let results = battle.execute_turn(&commands, &randoms);

        let spell_hits: Vec<_> = results
            .iter()
            .filter(|r| matches!(r, TurnResult::SpellDamage { caster: ActorId::Enemy(0), .. }))
            .collect();
        assert_eq!(spell_hits.len(), 3, "Blaze2でパーティ全員にヒットするはず");
    }

    #[test]
    fn sleeping_actor_skips_turn() {
        let party = vec![PartyMember::laios()];
        let mut slime = Enemy::slime();
        slime.stats.hp = 999;
        slime.stats.max_hp = 999;
        slime.stats.attack = 0;
        let enemies = vec![slime];
        let mut battle = BattleState::new(party, enemies);

        // ライオスを眠り状態にする
        battle.party_ailments[0].sleep = true;

        let commands = vec![BattleAction::Attack {
            target: TargetId::Enemy(0),
        }];
        let randoms = make_random(vec![1.0; 2], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        let sleeping = results
            .iter()
            .any(|r| matches!(r, TurnResult::Sleeping { actor: ActorId::Party(0) }));
        assert!(sleeping, "眠り状態のアクターはSleepingが出るはず");

        let attacks = results
            .iter()
            .filter(|r| matches!(r, TurnResult::Attack { attacker: ActorId::Party(0), .. }))
            .count();
        assert_eq!(attacks, 0, "眠り状態のアクターは攻撃しないはず");
    }

    #[test]
    fn sleeping_enemy_skips_turn() {
        let party = vec![PartyMember::laios()];
        let mut slime = Enemy::slime();
        slime.stats.hp = 999;
        slime.stats.max_hp = 999;
        slime.stats.speed = 99; // 敵が先に行動するようにする
        let enemies = vec![slime];
        let mut battle = BattleState::new(party, enemies);

        // 敵を眠り状態にする
        battle.enemy_ailments[0].sleep = true;

        let commands = vec![BattleAction::Attack {
            target: TargetId::Enemy(0),
        }];
        let randoms = make_random(vec![1.0; 2], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        let sleeping = results
            .iter()
            .any(|r| matches!(r, TurnResult::Sleeping { actor: ActorId::Enemy(0) }));
        assert!(sleeping, "眠り状態の敵はSleepingが出るはず");

        let enemy_attacks = results
            .iter()
            .filter(|r| matches!(r, TurnResult::Attack { attacker: ActorId::Enemy(0), .. }))
            .count();
        assert_eq!(enemy_attacks, 0, "眠り状態の敵は攻撃しないはず");
    }

    #[test]
    fn poison_damage_at_turn_end() {
        let party = vec![PartyMember::laios()];
        let mut slime = Enemy::slime();
        slime.stats.hp = 999;
        slime.stats.max_hp = 999;
        slime.stats.attack = 0;
        let enemies = vec![slime];
        let mut battle = BattleState::new(party, enemies);

        // ライオスを毒状態にする
        battle.party_ailments[0].poison = true;

        let commands = vec![BattleAction::Attack {
            target: TargetId::Enemy(0),
        }];
        let randoms = make_random(vec![1.0; 2], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        let poison_dmg = results
            .iter()
            .any(|r| matches!(r, TurnResult::PoisonDamage { target: TargetId::Party(0), .. }));
        assert!(poison_dmg, "毒状態でPoisonDamageが発生するはず");

        // 敵攻撃（最低1ダメージ）＋毒ダメージ分HPが減っていること
        let hp_after = battle.party[0].stats.hp;
        let max_hp = battle.party[0].stats.max_hp;
        assert!(hp_after <= max_hp - POISON_DAMAGE, "毒ダメージ分以上HPが減るはず");
    }

    #[test]
    fn poison_damage_on_enemy() {
        let party = vec![PartyMember::laios()];
        let mut slime = Enemy::slime();
        slime.stats.hp = 999;
        slime.stats.max_hp = 999;
        slime.stats.attack = 0;
        let enemies = vec![slime];
        let mut battle = BattleState::new(party, enemies);

        // 敵を毒状態にする
        battle.enemy_ailments[0].poison = true;

        let commands = vec![BattleAction::Attack {
            target: TargetId::Enemy(0),
        }];
        let randoms = make_random(vec![1.0; 2], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        let poison_dmg = results
            .iter()
            .any(|r| matches!(r, TurnResult::PoisonDamage { target: TargetId::Enemy(0), .. }));
        assert!(poison_dmg, "毒状態の敵にPoisonDamageが発生するはず");
    }

    #[test]
    fn attack_wakes_up_sleeping_enemy() {
        let party = vec![PartyMember::laios()];
        let mut slime = Enemy::slime();
        slime.stats.hp = 999;
        slime.stats.max_hp = 999;
        slime.stats.attack = 0;
        let enemies = vec![slime];
        let mut battle = BattleState::new(party, enemies);

        // 敵を眠り状態にする
        battle.enemy_ailments[0].sleep = true;

        let commands = vec![BattleAction::Attack {
            target: TargetId::Enemy(0),
        }];
        let randoms = make_random(vec![1.0; 2], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        let cured = results.iter().any(|r| {
            matches!(
                r,
                TurnResult::AilmentCured {
                    target: TargetId::Enemy(0),
                    ailment: Ailment::Sleep,
                }
            )
        });
        assert!(cured, "攻撃で眠りが解除されるはず");
        assert!(!battle.enemy_ailments[0].sleep, "眠りフラグが解除されるはず");
    }

    #[test]
    fn ailment_spell_success_and_failure() {
        let mut marcille = PartyMember::marcille();
        marcille.level = 8;
        marcille.stats.mp = 99;
        marcille.stats.max_mp = 99;
        let party = vec![marcille.clone(), marcille];
        let mut slime = Enemy::slime();
        slime.stats.hp = 999;
        slime.stats.max_hp = 999;
        slime.stats.attack = 0;
        let enemies = vec![slime];
        let mut battle = BattleState::new(party, enemies);

        // Sleep1 power=70 → random_factor < 0.7 で成功
        // Party(0): 成功ケース (random=0.5 → 0.5*100=50 < 70 → 成功)
        let commands = vec![
            BattleAction::Spell {
                spell: SpellKind::Sleep1,
                target: TargetId::Enemy(0),
            },
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
        ];
        let randoms = make_random(vec![0.5; 3], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        let inflicted = results.iter().any(|r| {
            matches!(
                r,
                TurnResult::AilmentInflicted {
                    ailment: Ailment::Sleep,
                    ..
                }
            )
        });
        assert!(inflicted, "成功率内なら状態異常が付与されるはず");

        // リセット
        battle.enemy_ailments[0].sleep = false;

        // 失敗ケース (random=0.9 → 0.9*100=90 >= 70 → 失敗)
        let commands = vec![
            BattleAction::Spell {
                spell: SpellKind::Sleep1,
                target: TargetId::Enemy(0),
            },
            BattleAction::Attack {
                target: TargetId::Enemy(0),
            },
        ];
        let randoms = make_random(vec![0.9; 3], 0.0);
        let results = battle.execute_turn(&commands, &randoms);

        let resisted = results
            .iter()
            .any(|r| matches!(r, TurnResult::AilmentResisted { .. }));
        assert!(resisted, "成功率外なら状態異常が抵抗されるはず");
        assert!(!battle.enemy_ailments[0].sleep, "抵抗時は眠りフラグがfalseのまま");
    }
}
