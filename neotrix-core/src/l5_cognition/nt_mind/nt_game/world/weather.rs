#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherType {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Snow,
    Fog,
    Wind,
}

#[derive(Debug, Clone)]
pub struct WeatherState {
    pub current: WeatherType,
    pub intensity: f64,
    pub duration_remaining: f64,
    pub transition_chance: f64,
}

impl WeatherState {
    pub fn new() -> Self {
        Self {
            current: WeatherType::Clear,
            intensity: 1.0,
            duration_remaining: 60.0,
            transition_chance: 0.01,
        }
    }
    pub fn tick(&mut self, dt: f64, season: &super::season::Season) {
        self.duration_remaining -= dt;
        if self.duration_remaining <= 0.0 {
            self.transition(season);
        }
    }
    fn transition(&mut self, season: &super::season::Season) {
        self.current = match season {
            super::season::Season::Spring => WeatherType::Rain,
            super::season::Season::Summer => WeatherType::Clear,
            super::season::Season::Autumn => WeatherType::Cloudy,
            super::season::Season::Winter => WeatherType::Snow,
        };
        self.duration_remaining = 120.0;
        self.intensity = 0.5 + (self.current as u32 as f64 * 0.1);
    }
    pub fn visibility_modifier(&self) -> f64 {
        match self.current {
            WeatherType::Clear => 1.0,
            WeatherType::Cloudy => 0.9,
            WeatherType::Rain => 0.7,
            WeatherType::Storm => 0.5,
            WeatherType::Snow => 0.6,
            WeatherType::Fog => 0.3,
            WeatherType::Wind => 0.85,
        }
    }
    pub fn movement_modifier(&self) -> f64 {
        match self.current {
            WeatherType::Clear => 1.0,
            WeatherType::Rain => 0.85,
            WeatherType::Storm => 0.6,
            WeatherType::Snow => 0.7,
            WeatherType::Fog => 0.9,
            WeatherType::Wind => 0.8,
            WeatherType::Cloudy => 1.0,
        }
    }
}

impl Default for WeatherState {
    fn default() -> Self {
        Self::new()
    }
}
