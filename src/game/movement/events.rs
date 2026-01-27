use bevy::prelude::*;

/// 移動がブロックされたときに発行されるメッセージ
#[derive(Message)]
pub struct MovementBlockedEvent {
    pub entity: Entity,
    pub direction: (i32, i32),
}

/// プレイヤーが移動したときに発行されるメッセージ
#[derive(Message)]
pub struct PlayerMovedEvent {
    pub entity: Entity,
    pub direction: (i32, i32),
}
