use bevy::prelude::*;

use time::TimeCount;

/// ゲーム内時間カウンターのBevyリソース
#[derive(Resource)]
pub struct TimeCounter {
    pub time_count: TimeCount,
}

impl Default for TimeCounter {
    fn default() -> Self {
        Self {
            time_count: TimeCount::new(),
        }
    }
}
