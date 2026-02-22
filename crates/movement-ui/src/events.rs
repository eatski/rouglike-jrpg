use bevy::prelude::*;

use terrain::Direction;

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

/// プレイヤーが能動的な移動でタイルに到着したときに発行されるメッセージ
/// フィールドのSmoothMove完了時のみ発火する（テレポートでは発火しない）
#[derive(Message)]
pub struct TileEnteredEvent {
    pub entity: Entity,
}
