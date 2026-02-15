/// ゲーム内時間のカウント
pub struct TimeCount {
    count: u32,
}

impl TimeCount {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn count(&self) -> u32 {
        self.count
    }
}

impl Default for TimeCount {
    fn default() -> Self {
        Self::new()
    }
}
