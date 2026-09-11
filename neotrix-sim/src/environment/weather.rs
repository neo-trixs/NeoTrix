use serde::{Serialize, Deserialize};

/// Weather state types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherType {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Snow,
    Fog,
}

impl WeatherType {
    /// Transition probabilities from this weather to others [Clear, Cloudy, Rain, Storm, Snow, Fog]
    fn transition_weights(&self) -> [f32; 6] {
        match self {
            WeatherType::Clear => [0.70, 0.20, 0.05, 0.00, 0.00, 0.05],
            WeatherType::Cloudy => [0.15, 0.50, 0.20, 0.05, 0.05, 0.05],
            WeatherType::Rain => [0.05, 0.20, 0.45, 0.15, 0.05, 0.10],
            WeatherType::Storm => [0.05, 0.15, 0.30, 0.35, 0.05, 0.10],
            WeatherType::Snow => [0.10, 0.20, 0.05, 0.05, 0.50, 0.10],
            WeatherType::Fog => [0.30, 0.20, 0.10, 0.05, 0.05, 0.30],
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            WeatherType::Clear => "Clear",
            WeatherType::Cloudy => "Cloudy",
            WeatherType::Rain => "Rain",
            WeatherType::Storm => "Storm",
            WeatherType::Snow => "Snow",
            WeatherType::Fog => "Fog",
        }
    }
}

/// Configuration for weather transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    /// Base transition chance per tick (0.0-1.0)
    pub transition_chance: f32,
    /// Minimum duration (ticks) before weather can change
    pub min_duration: u64,
    /// Maximum duration before forced change
    pub max_duration: u64,
    /// Season bias: [spring_w, summer_w, fall_w, winter_w] for snow
    pub season_snow_bias: [f32; 4],
    /// Season bias: [spring_w, summer_w, fall_w, winter_w] for fog
    pub season_fog_bias: [f32; 4],
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            transition_chance: 0.02,
            min_duration: 50,
            max_duration: 500,
            season_snow_bias: [0.05, 0.0, 0.1, 0.4],
            season_fog_bias: [0.15, 0.05, 0.2, 0.1],
        }
    }
}

/// Weather system with probabilistic transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSystem {
    pub current: WeatherType,
    config: WeatherConfig,
    ticks_in_current: u64,
    /// Simple PRNG state (xorshift32)
    rng_state: u32,
}

impl Default for WeatherSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl WeatherSystem {
    pub fn new() -> Self {
        Self {
            current: WeatherType::Clear,
            config: WeatherConfig::default(),
            ticks_in_current: 0,
            rng_state: 12345,
        }
    }

    pub fn with_config(mut self, config: WeatherConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_seed(mut self, seed: u32) -> Self {
        self.rng_state = seed.max(1);
        self
    }

    /// Simple xorshift32 PRNG
    fn next_f32(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        (self.rng_state as f32) / (u32::MAX as f32)
    }

    /// Advance weather by one tick
    pub fn tick(&mut self, season_bias: Option<[f32; 4]>) {
        self.ticks_in_current += 1;

        if self.ticks_in_current < self.config.min_duration {
            return;
        }

        let roll = self.next_f32();
        let mut chance = self.config.transition_chance;

        // Increase chance if we've been stuck too long
        if self.ticks_in_current > self.config.max_duration {
            chance = 1.0;
        } else if self.ticks_in_current > self.config.max_duration / 2 {
            let excess = (self.ticks_in_current - self.config.max_duration / 2) as f32
                / (self.config.max_duration / 2) as f32;
            chance += excess * 0.3;
        }

        if roll < chance {
            self.transition(season_bias);
        }
    }

    /// Force a weather transition
    fn transition(&mut self, season_bias: Option<[f32; 4]>) {
        let mut weights = self.current.transition_weights();

        // Apply season biases
        if let Some(bias) = season_bias {
            // Winter increases snow, autumn/spring increase fog
            weights[4] += bias[3] * 0.2; // snow
            weights[5] += (bias[2] + bias[1]) * 0.1; // fog
        }

        // Normalize weights
        let sum: f32 = weights.iter().sum();
        if sum <= 0.0 {
            return;
        }

        let roll = self.next_f32() * sum;
        let mut cumulative = 0.0;
        let all_types = [
            WeatherType::Clear,
            WeatherType::Cloudy,
            WeatherType::Rain,
            WeatherType::Storm,
            WeatherType::Snow,
            WeatherType::Fog,
        ];

        for (i, &w) in weights.iter().enumerate() {
            cumulative += w;
            if roll <= cumulative {
                self.current = all_types[i];
                self.ticks_in_current = 0;
                return;
            }
        }
    }

    /// Force a specific weather type
    pub fn set_weather(&mut self, weather: WeatherType) {
        self.current = weather;
        self.ticks_in_current = 0;
    }

    /// Movement speed modifier (1.0 = no penalty)
    pub fn movement_modifier(&self) -> f32 {
        match self.current {
            WeatherType::Clear => 1.0,
            WeatherType::Cloudy => 0.95,
            WeatherType::Rain => 0.7,
            WeatherType::Storm => 0.4,
            WeatherType::Snow => 0.5,
            WeatherType::Fog => 0.85,
        }
    }

    /// Danger multiplier from weather
    pub fn danger_modifier(&self) -> f32 {
        match self.current {
            WeatherType::Clear => 1.0,
            WeatherType::Cloudy => 1.0,
            WeatherType::Rain => 1.1,
            WeatherType::Storm => 1.5,
            WeatherType::Snow => 1.2,
            WeatherType::Fog => 1.2,
        }
    }

    /// Effective visibility range in world units
    pub fn visibility_range(&self) -> f32 {
        match self.current {
            WeatherType::Clear => 15.0,
            WeatherType::Cloudy => 12.0,
            WeatherType::Rain => 8.0,
            WeatherType::Storm => 5.0,
            WeatherType::Snow => 6.0,
            WeatherType::Fog => 3.0,
        }
    }

    /// Visibility multiplier (0.0-1.0)
    pub fn visibility_modifier(&self) -> f32 {
        self.visibility_range() / 15.0
    }

    /// Energy drain modifier (storm drains more)
    pub fn energy_drain_modifier(&self) -> f32 {
        match self.current {
            WeatherType::Clear => 1.0,
            WeatherType::Cloudy => 1.0,
            WeatherType::Rain => 1.1,
            WeatherType::Storm => 1.4,
            WeatherType::Snow => 1.2,
            WeatherType::Fog => 1.05,
        }
    }

    /// Duration in current weather
    pub fn ticks_in_current(&self) -> u64 {
        self.ticks_in_current
    }

    pub fn display(&self) -> String {
        format!("{} ({} ticks)", self.current.as_str(), self.ticks_in_current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_default() {
        let ws = WeatherSystem::new();
        assert_eq!(ws.current, WeatherType::Clear);
        assert_eq!(ws.ticks_in_current, 0);
    }

    #[test]
    fn tick_increments_duration() {
        let mut ws = WeatherSystem::new();
        ws.tick(None);
        assert_eq!(ws.ticks_in_current, 1);
        ws.tick(None);
        assert_eq!(ws.ticks_in_current, 2);
    }

    #[test]
    fn no_transition_before_min_duration() {
        let mut ws = WeatherSystem::new()
            .with_config(WeatherConfig {
                transition_chance: 1.0, // would force transition
                min_duration: 10,
                ..Default::default()
            });
        for _ in 0..9 {
            ws.tick(None);
        }
        assert_eq!(ws.current, WeatherType::Clear);
    }

    #[test]
    fn transition_can_happen_after_min() {
        let mut ws = WeatherSystem::new()
            .with_seed(42)
            .with_config(WeatherConfig {
                transition_chance: 0.99,
                min_duration: 1,
                ..Default::default()
            });
        // Run many ticks; weather should eventually change
        let mut changed = false;
        for _ in 0..200 {
            ws.tick(None);
            if ws.current != WeatherType::Clear {
                changed = true;
                break;
            }
        }
        assert!(changed, "Weather should have transitioned within 200 ticks");
    }

    #[test]
    fn movement_modifier_rain() {
        let ws = WeatherSystem {
            current: WeatherType::Rain,
            ..WeatherSystem::new()
        };
        assert!((ws.movement_modifier() - 0.7).abs() < 0.01);
    }

    #[test]
    fn movement_modifier_storm() {
        let ws = WeatherSystem {
            current: WeatherType::Storm,
            ..WeatherSystem::new()
        };
        assert!((ws.movement_modifier() - 0.4).abs() < 0.01);
    }

    #[test]
    fn movement_modifier_snow() {
        let ws = WeatherSystem {
            current: WeatherType::Snow,
            ..WeatherSystem::new()
        };
        assert!((ws.movement_modifier() - 0.5).abs() < 0.01);
    }

    #[test]
    fn danger_storm_high() {
        let ws = WeatherSystem {
            current: WeatherType::Storm,
            ..WeatherSystem::new()
        };
        assert!(ws.danger_modifier() > 1.3);
    }

    #[test]
    fn danger_fog_moderate() {
        let ws = WeatherSystem {
            current: WeatherType::Fog,
            ..WeatherSystem::new()
        };
        assert!(ws.danger_modifier() > 1.0);
        assert!(ws.danger_modifier() < 1.5);
    }

    #[test]
    fn visibility_range_fog_low() {
        let ws = WeatherSystem {
            current: WeatherType::Fog,
            ..WeatherSystem::new()
        };
        assert!(ws.visibility_range() < 5.0);
    }

    #[test]
    fn visibility_range_clear_high() {
        let ws = WeatherSystem {
            current: WeatherType::Clear,
            ..WeatherSystem::new()
        };
        assert!(ws.visibility_range() >= 15.0);
    }

    #[test]
    fn energy_drain_storm() {
        let ws = WeatherSystem {
            current: WeatherType::Storm,
            ..WeatherSystem::new()
        };
        assert!(ws.energy_drain_modifier() > 1.3);
    }

    #[test]
    fn set_weather_resets_duration() {
        let mut ws = WeatherSystem::new();
        ws.tick(None);
        ws.tick(None);
        assert_eq!(ws.ticks_in_current, 2);
        ws.set_weather(WeatherType::Storm);
        assert_eq!(ws.current, WeatherType::Storm);
        assert_eq!(ws.ticks_in_current, 0);
    }

    #[test]
    fn forced_change_at_max_duration() {
        let mut ws = WeatherSystem::new()
            .with_config(WeatherConfig {
                transition_chance: 0.0,
                min_duration: 0,
                max_duration: 5,
                ..Default::default()
            });
        for _ in 0..5 {
            ws.tick(None);
        }
        // After max_duration, transition_chance becomes 1.0
        // So next tick should force a change
        let before = ws.current;
        ws.tick(None);
        // It might still be Clear if roll lands on Clear, but the transition was attempted
        assert!(ws.ticks_in_current <= 1);
    }

    #[test]
    fn display_format() {
        let ws = WeatherSystem {
            current: WeatherType::Rain,
            ticks_in_current: 42,
            ..WeatherSystem::new()
        };
        let d = ws.display();
        assert!(d.contains("Rain"));
        assert!(d.contains("42"));
    }
}
