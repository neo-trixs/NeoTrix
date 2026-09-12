use crate::core::{Resource, UniversalWorld};
use crate::core::scheduler::UniversalSystem;
use super::time::GameTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherType {
    Clear,
    Rain,
    Storm,
    Snow,
    GreenRain,
}

impl WeatherType {
    pub fn name(&self) -> &str {
        match self {
            WeatherType::Clear => "Clear",
            WeatherType::Rain => "Rain",
            WeatherType::Storm => "Storm",
            WeatherType::Snow => "Snow",
            WeatherType::GreenRain => "Green Rain",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            WeatherType::Clear => "*",
            WeatherType::Rain => "~",
            WeatherType::Storm => "!",
            WeatherType::Snow => ".",
            WeatherType::GreenRain => "%",
        }
    }

    pub fn growth_modifier(&self) -> f32 {
        match self {
            WeatherType::Clear => 1.0,
            WeatherType::Rain => 1.3,
            WeatherType::Storm => 0.8,
            WeatherType::Snow => 0.0,
            WeatherType::GreenRain => 1.5,
        }
    }

    pub fn energy_modifier(&self) -> f32 {
        match self {
            WeatherType::Clear => 1.0,
            WeatherType::Rain => 0.9,
            WeatherType::Storm => 0.7,
            WeatherType::Snow => 0.8,
            WeatherType::GreenRain => 1.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Weather {
    pub current: WeatherType,
    pub forecast: [WeatherType; 3],
    pub wind_strength: f32,
    pub days_rained: u32,
}

impl Weather {
    pub fn new() -> Self {
        Self {
            current: WeatherType::Clear,
            forecast: [WeatherType::Clear; 3],
            wind_strength: 0.0,
            days_rained: 0,
        }
    }

    pub fn tomorrow(&self) -> WeatherType {
        self.forecast[0]
    }

    pub fn advance_day(&mut self) {
        self.current = self.forecast[0];
        self.forecast[0] = self.forecast[1];
        self.forecast[1] = self.forecast[2];
        self.forecast[2] = WeatherType::Clear;

        if self.current == WeatherType::Rain || self.current == WeatherType::Storm {
            self.days_rained += 1;
        }

        self.wind_strength = match self.current {
            WeatherType::Storm => 0.7 + rand_factor() * 0.3,
            WeatherType::Rain => 0.2 + rand_factor() * 0.3,
            WeatherType::Snow => 0.1 + rand_factor() * 0.2,
            _ => rand_factor() * 0.2,
        };
    }

    pub fn is_raining(&self) -> bool {
        matches!(self.current, WeatherType::Rain | WeatherType::Storm | WeatherType::GreenRain)
    }

    pub fn waters_crops(&self) -> bool {
        self.is_raining()
    }
}

impl Default for Weather {
    fn default() -> Self { Self::new() }
}

impl Resource for Weather {}

pub struct WeatherSystem;

impl UniversalSystem for WeatherSystem {
    fn name(&self) -> &str {
        "WeatherSystem"
    }

    fn update(&mut self, world: &mut UniversalWorld, _dt: f32) {
        let time_day = world.get_resource::<GameTime>().map(|t| t.day);
        if let Some(day) = time_day {
            if let Some(weather) = world.get_resource_mut::<Weather>() {
                let time_hash = day.wrapping_mul(2654435761);
                let idx = (time_hash % 4) as usize;
                weather.forecast[2] = match idx {
                    0 => WeatherType::Clear,
                    1 => WeatherType::Rain,
                    2 => WeatherType::Storm,
                    _ => WeatherType::Snow,
                };
            }
        }
    }
}

fn rand_factor() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos % 1000) as f32 / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_advance() {
        let mut weather = Weather::new();
        weather.forecast = [WeatherType::Rain, WeatherType::Clear, WeatherType::Storm];
        weather.advance_day();
        assert_eq!(weather.current, WeatherType::Rain);
        assert!(weather.is_raining());
    }

    #[test]
    fn test_weather_modifiers() {
        assert!((WeatherType::Rain.growth_modifier() - 1.3).abs() < 0.01);
        assert!((WeatherType::Snow.growth_modifier()).abs() < 0.01);
    }
}
