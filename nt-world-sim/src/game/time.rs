use crate::core::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Season {
    Clarity,   // Spring
    Flow,      // Summer
    Reflection,// Fall
    Stillness, // Winter
}

impl Season {
    pub fn index(&self) -> u32 {
        match self {
            Season::Clarity => 0,
            Season::Flow => 1,
            Season::Reflection => 2,
            Season::Stillness => 3,
        }
    }
    
    pub fn name(&self) -> &str {
        match self {
            Season::Clarity => "Clarity",
            Season::Flow => "Flow",
            Season::Reflection => "Reflection",
            Season::Stillness => "Stillness",
        }
    }
    
    pub fn next(&self) -> Season {
        match self {
            Season::Clarity => Season::Flow,
            Season::Flow => Season::Reflection,
            Season::Reflection => Season::Stillness,
            Season::Stillness => Season::Clarity,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameTime {
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub season: Season,
    pub year: u32,
    pub days_played: u32,
}

impl GameTime {
    pub fn new() -> Self {
        Self {
            day: 1, hour: 6, minute: 0,
            season: Season::Clarity,
            year: 1, days_played: 0,
        }
    }
    
    pub fn advance_minute(&mut self, minutes: u32) {
        self.minute += minutes;
        while self.minute >= 60 {
            self.minute -= 60;
            self.hour += 1;
        }
        while self.hour >= 24 {
            self.hour -= 24;
            self.advance_day();
        }
    }
    
    pub fn advance_day(&mut self) {
        self.day += 1;
        self.days_played += 1;
        if self.day > 28 {
            self.day = 1;
            self.season = self.season.next();
            if self.season == Season::Clarity {
                self.year += 1;
            }
        }
    }
    
    pub fn time_string(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }
    
    pub fn date_string(&self) -> String {
        format!("{} Day {}, Year {}", self.season.name(), self.day, self.year)
    }
    
    pub fn is_night(&self) -> bool {
        self.hour >= 20 || self.hour < 6
    }
    
    pub fn day_of_week(&self) -> &str {
        let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        days[(self.days_played % 7) as usize]
    }
}

impl Resource for GameTime {}

impl Default for GameTime {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_advance() {
        let mut time = GameTime::new();
        assert_eq!(time.hour, 6);
        
        time.advance_minute(30);
        assert_eq!(time.hour, 6);
        assert_eq!(time.minute, 30);
        
        time.advance_minute(45);
        assert_eq!(time.hour, 7);
        assert_eq!(time.minute, 15);
    }
    
    #[test]
    fn test_day_advance() {
        let mut time = GameTime::new();
        time.advance_day();
        assert_eq!(time.day, 2);
        assert_eq!(time.season, Season::Clarity);
    }
    
    #[test]
    fn test_season_change() {
        let mut time = GameTime::new();
        time.day = 28;
        time.advance_day();
        assert_eq!(time.day, 1);
        assert_eq!(time.season, Season::Flow);
    }
}
