mod resources;

use bevy::prelude::*;

pub use resources::{
    BossBattlePending, BossDefeated, ContinentCavePositions, ContinentMap, EncounterZone,
    FieldMenuOpen, HeardTavernHints, HokoraPositions, OpenedChests, PartyState, RecruitmentMap,
    TavernBounties, TavernHintKind,
};

pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .init_state::<BattleState>()
            .add_computed_state::<InField>()
            .init_resource::<PartyState>()
            .init_resource::<OpenedChests>()
            .init_resource::<TavernBounties>();
    }
}

/// シーン（場所）の状態
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SceneState {
    #[default]
    Exploring,
    Town,
    Cave,
    BossCave,
    Hokora,
}

/// 戦闘オーバーレイの状態
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BattleState {
    #[default]
    None,
    Active,
}

/// フィールド上にいる状態（Exploring or Cave、かつ戦闘中でない）
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InField;

impl ComputedStates for InField {
    type SourceStates = (SceneState, BattleState);

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            (SceneState::Exploring, BattleState::None)
            | (SceneState::Cave, BattleState::None)
            | (SceneState::BossCave, BattleState::None) => Some(InField),
            _ => None,
        }
    }
}
