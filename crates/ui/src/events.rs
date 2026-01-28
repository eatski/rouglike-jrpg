use bevy::prelude::*;

use game::movement::Direction;

/// 移動がブロックされたときに発行されるメッセージ
#[derive(Message)]
pub struct MovementBlockedEvent {
    pub entity: Entity,
    pub direction: Direction,
}

/// プレイヤーが移動したときに発行されるメッセージ
#[derive(Message)]
pub struct PlayerMovedEvent {
    pub entity: Entity,
    pub direction: Direction,
}
