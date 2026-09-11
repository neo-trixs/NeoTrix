pub struct StatsPanel {
    pub x: f32,
    pub y: f32,
}

impl StatsPanel {
    pub fn new(x: f32, y: f32) -> Self {
        StatsPanel { x, y }
    }

    pub fn format_stats(&self, pop: usize, fitness: f32, trades: u64, tick: u64) -> String {
        format!("Pop:{} Fit:{:.2} Trades:{} Tick:{}", pop, fitness, trades, tick)
    }
}
