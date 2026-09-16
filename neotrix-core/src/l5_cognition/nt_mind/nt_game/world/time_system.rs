#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    Dawn,
    Morning,
    Noon,
    Afternoon,
    Dusk,
    Night,
}

#[derive(Debug, Clone)]
pub struct GameClock {
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub speed: f64,
    pub paused: bool,
}

impl GameClock {
    pub fn new() -> Self {
        Self {
            day: 1,
            hour: 8,
            minute: 0,
            speed: 1.0,
            paused: false,
        }
    }
    pub fn tick(&mut self, dt: f64) {
        if self.paused {
            return;
        }
        let minutes = dt * self.speed * 10.0;
        self.minute += minutes as u32;
        while self.minute >= 60 {
            self.minute -= 60;
            self.hour += 1;
        }
        while self.hour >= 24 {
            self.hour -= 24;
            self.day += 1;
        }
    }
    pub fn time_of_day(&self) -> TimeOfDay {
        match self.hour {
            5..=6 => TimeOfDay::Dawn,
            7..=10 => TimeOfDay::Morning,
            11..=13 => TimeOfDay::Noon,
            14..=17 => TimeOfDay::Afternoon,
            18..=20 => TimeOfDay::Dusk,
            _ => TimeOfDay::Night,
        }
    }
    pub fn is_daytime(&self) -> bool {
        self.hour >= 6 && self.hour < 20
    }
    pub fn format(&self) -> String {
        format!("Day {} {:02}:{:02}", self.day, self.hour, self.minute)
    }
    pub fn pause(&mut self) {
        self.paused = true;
    }
    pub fn resume(&mut self) {
        self.paused = false;
    }
    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed;
    }
}
impl Default for GameClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_progression() {
        let mut c = GameClock::new();
        c.tick(60.0);
        assert_eq!(c.hour, 9);
        assert_eq!(c.minute, 0);
    }

    #[test]
    fn test_day_wrap() {
        let mut c = GameClock::new();
        c.hour = 23;
        c.tick(120.0);
        assert_eq!(c.day, 2);
    }

    #[test]
    fn test_time_of_day() {
        let mut c = GameClock::new();
        assert_eq!(c.time_of_day(), TimeOfDay::Morning);
        c.hour = 12;
        assert_eq!(c.time_of_day(), TimeOfDay::Noon);
    }
}
