#[derive(Debug, Clone, PartialEq)]
pub enum SimSpeed {
    Paused,
    Slow,
    Normal,
    Fast,
    Ultra,
}

impl SimSpeed {
    pub fn multiplier(&self) -> f32 {
        match self {
            SimSpeed::Paused => 0.0,
            SimSpeed::Slow => 0.5,
            SimSpeed::Normal => 1.0,
            SimSpeed::Fast => 2.0,
            SimSpeed::Ultra => 5.0,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SimSpeed::Paused => SimSpeed::Slow,
            SimSpeed::Slow => SimSpeed::Normal,
            SimSpeed::Normal => SimSpeed::Fast,
            SimSpeed::Fast => SimSpeed::Ultra,
            SimSpeed::Ultra => SimSpeed::Paused,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            SimSpeed::Paused => SimSpeed::Ultra,
            SimSpeed::Slow => SimSpeed::Paused,
            SimSpeed::Normal => SimSpeed::Slow,
            SimSpeed::Fast => SimSpeed::Normal,
            SimSpeed::Ultra => SimSpeed::Fast,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            SimSpeed::Paused => "⏸ Paused",
            SimSpeed::Slow => "▶▶ Slow",
            SimSpeed::Normal => "▶ Normal",
            SimSpeed::Fast => "▶▶▶ Fast",
            SimSpeed::Ultra => "⚡ Ultra",
        }
    }
}

pub struct TimeController {
    pub speed: SimSpeed,
    pub ticks_per_second: f32,
    pub accumulated: f32,
    pub total_ticks: u64,
    pub session_start: std::time::Instant,
}

impl TimeController {
    pub fn new() -> Self {
        Self {
            speed: SimSpeed::Normal,
            ticks_per_second: 20.0,
            accumulated: 0.0,
            total_ticks: 0,
            session_start: std::time::Instant::now(),
        }
    }

    pub fn update(&mut self, dt: f32) -> u32 {
        if self.speed == SimSpeed::Paused {
            return 0;
        }

        self.accumulated += dt * self.ticks_per_second * self.speed.multiplier();
        let ticks = self.accumulated as u32;
        self.accumulated -= ticks as f32;
        self.total_ticks += ticks as u64;
        ticks
    }

    pub fn set_speed(&mut self, speed: SimSpeed) {
        self.speed = speed;
    }

    pub fn toggle_pause(&mut self) {
        self.speed = if self.speed == SimSpeed::Paused {
            SimSpeed::Normal
        } else {
            SimSpeed::Paused
        };
    }

    pub fn speed_up(&mut self) {
        self.speed = self.speed.next();
    }

    pub fn slow_down(&mut self) {
        self.speed = self.speed.previous();
    }

    pub fn set_ticks_per_second(&mut self, tps: f32) {
        self.ticks_per_second = tps.max(1.0);
    }

    pub fn session_duration(&self) -> std::time::Duration {
        self.session_start.elapsed()
    }

    pub fn render_status(&self) -> String {
        format!(
            "Speed: {} | TPS: {:.0} | Total: {} | Session: {:?}",
            self.speed.label(),
            self.ticks_per_second,
            self.total_ticks,
            self.session_duration()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_multiplier() {
        assert_eq!(SimSpeed::Paused.multiplier(), 0.0);
        assert_eq!(SimSpeed::Normal.multiplier(), 1.0);
        assert_eq!(SimSpeed::Fast.multiplier(), 2.0);
    }

    #[test]
    fn test_speed_cycle() {
        assert_eq!(SimSpeed::Paused.next(), SimSpeed::Slow);
        assert_eq!(SimSpeed::Ultra.next(), SimSpeed::Paused);
        assert_eq!(SimSpeed::Normal.previous(), SimSpeed::Slow);
    }

    #[test]
    fn test_time_controller() {
        let mut ctrl = TimeController::new();
        let ticks = ctrl.update(0.1);
        assert!(ticks >= 0);
        assert_eq!(ctrl.speed, SimSpeed::Normal);
    }

    #[test]
    fn test_time_controller_paused() {
        let mut ctrl = TimeController::new();
        ctrl.set_speed(SimSpeed::Paused);
        let ticks = ctrl.update(1.0);
        assert_eq!(ticks, 0);
    }
}
