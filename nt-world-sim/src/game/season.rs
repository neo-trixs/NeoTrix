use super::time::Season;

pub struct SeasonEffects {
    growth_modifier: f32,
    spawn_modifier: f32,
    #[allow(dead_code)]
    weather_weights: [f32; 4],
}

impl SeasonEffects {
    pub fn for_season(season: Season) -> Self {
        match season {
            Season::Clarity => Self {
                growth_modifier: 1.0, spawn_modifier: 1.0,
                weather_weights: [0.6, 0.25, 0.1, 0.05],
            },
            Season::Flow => Self {
                growth_modifier: 1.2, spawn_modifier: 1.3,
                weather_weights: [0.5, 0.2, 0.2, 0.1],
            },
            Season::Reflection => Self {
                growth_modifier: 0.8, spawn_modifier: 0.9,
                weather_weights: [0.4, 0.3, 0.2, 0.1],
            },
            Season::Stillness => Self {
                growth_modifier: 0.0, spawn_modifier: 0.5,
                weather_weights: [0.3, 0.1, 0.1, 0.5],
            },
        }
    }
    
    pub fn growth_modifier(&self) -> f32 { self.growth_modifier }
    pub fn spawn_modifier(&self) -> f32 { self.spawn_modifier }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_season_effects() {
        let clarity = SeasonEffects::for_season(Season::Clarity);
        assert!((clarity.growth_modifier() - 1.0).abs() < 0.01);
        
        let flow = SeasonEffects::for_season(Season::Flow);
        assert!((flow.growth_modifier() - 1.2).abs() < 0.01);
    }
}
