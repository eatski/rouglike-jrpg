use bevy::prelude::*;

/// シーン（場所）の状態
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SceneState {
    #[default]
    Exploring,
    Town,
    Cave,
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
            | (SceneState::Cave, BattleState::None) => Some(InField),
            _ => None,
        }
    }
}
