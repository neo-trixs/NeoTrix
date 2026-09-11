pub struct TimeControl {
    pub speed: f32,
    pub paused: bool,
    pub tick_count: u64,
}

impl TimeControl {
    pub fn new() -> Self {
        TimeControl {
            speed: 1.0,
            paused: false,
            tick_count: 0,
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(0.1, 10.0);
    }

    pub fn advance(&mut self) -> u64 {
        if !self.paused {
            self.tick_count += 1;
        }
        self.tick_count
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }
}
