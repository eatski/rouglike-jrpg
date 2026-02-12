use bevy::prelude::*;

use game::Direction;

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

/// プレイヤーがタイルに到着したときに発行されるメッセージ
/// SmoothMoveアニメーション完了時に発火する（洞窟内専用）
#[derive(Message)]
pub struct PlayerArrivedEvent {
    pub entity: Entity,
}

/// プレイヤーが能動的な移動でタイルに到着したときに発行されるメッセージ
/// フィールドのSmoothMove完了時のみ発火する（テレポートでは発火しない）
#[derive(Message)]
pub struct TileEnteredEvent {
    pub entity: Entity,
}
