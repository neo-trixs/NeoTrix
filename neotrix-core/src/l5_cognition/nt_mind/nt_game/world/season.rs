#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn next(&self) -> Season {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }
    pub fn crop_modifier(&self) -> f64 {
        match self {
            Season::Spring => 1.0,
            Season::Summer => 1.2,
            Season::Autumn => 0.8,
            Season::Winter => 0.3,
        }
    }
    pub fn temperature_modifier(&self) -> f64 {
        match self {
            Season::Spring => 15.0,
            Season::Summer => 30.0,
            Season::Autumn => 10.0,
            Season::Winter => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SeasonState {
    pub current: Season,
    pub days_in_season: u32,
    pub season_length: u32,
}

impl SeasonState {
    pub fn new(season_length: u32) -> Self {
        Self {
            current: Season::Spring,
            days_in_season: 0,
            season_length,
        }
    }
    pub fn advance_day(&mut self) -> bool {
        self.days_in_season += 1;
        if self.days_in_season >= self.season_length {
            self.days_in_season = 0;
            self.current = self.current.next();
            true
        } else {
            false
        }
    }
    pub fn progress(&self) -> f64 {
        self.days_in_season as f64 / self.season_length as f64
    }
}

impl Default for SeasonState {
    fn default() -> Self {
        Self::new(28)
    }
}
