use serde::{Serialize, Deserialize};
use crate::foundation::simulation_bus::Season;

/// Season enum for the day/night cycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CycleSeason {
    Spring,
    Summer,
    Fall,
    Winter,
}

impl From<Season> for CycleSeason {
    fn from(s: Season) -> Self {
        match s {
            Season::Spring => CycleSeason::Spring,
            Season::Summer => CycleSeason::Summer,
            Season::Autumn => CycleSeason::Fall,
            Season::Winter => CycleSeason::Winter,
        }
    }
}

/// Weather modifier affecting the day/night cycle
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WeatherModifier {
    /// Cloud cover 0.0 (clear) to 1.0 (overcast) - dims daylight
    pub cloud_cover: f32,
    /// Fog density 0.0 (none) to 1.0 (dense) - reduces visibility
    pub fog_density: f32,
}

impl Default for WeatherModifier {
    fn default() -> Self {
        Self {
            cloud_cover: 0.0,
            fog_density: 0.0,
        }
    }
}

/// Continuous day/night cycle with smooth transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayNightCycle {
    /// Time of day in hours (0.0 - 24.0)
    pub time_of_day: f32,
    /// Current season
    pub season: CycleSeason,
    /// Weather modifier affecting light/energy
    pub weather: WeatherModifier,
    /// Base hours per day (default 24)
    hours_per_day: f32,
    /// Dawn start hour
    dawn_start: f32,
    /// Dawn end hour
    dawn_end: f32,
    /// Dusk start hour
    dusk_start: f32,
    /// Dusk end hour
    dusk_end: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self::new()
    }
}

impl DayNightCycle {
    pub fn new() -> Self {
        Self {
            time_of_day: 6.0,
            season: CycleSeason::Spring,
            weather: WeatherModifier::default(),
            hours_per_day: 24.0,
            dawn_start: 5.0,
            dawn_end: 7.0,
            dusk_start: 18.0,
            dusk_end: 20.0,
        }
    }

    pub fn with_time(mut self, time: f32) -> Self {
        self.time_of_day = time % self.hours_per_day;
        self
    }

    pub fn with_season(mut self, season: CycleSeason) -> Self {
        self.season = season;
        self
    }

    /// Advance time by dt hours
    pub fn tick(&mut self, dt: f32) {
        self.time_of_day += dt;
        while self.time_of_day >= self.hours_per_day {
            self.time_of_day -= self.hours_per_day;
        }
    }

    /// Check if it is currently night
    pub fn is_night(&self) -> bool {
        self.time_of_day < self.dawn_start || self.time_of_day >= self.dusk_end
    }

    /// Check if it is currently dawn
    pub fn is_dawn(&self) -> bool {
        self.time_of_day >= self.dawn_start && self.time_of_day < self.dawn_end
    }

    /// Check if it is currently dusk
    pub fn is_dusk(&self) -> bool {
        self.time_of_day >= self.dusk_start && self.time_of_day < self.dusk_end
    }

    /// Check if it is currently day
    pub fn is_day(&self) -> bool {
        self.time_of_day >= self.dawn_end && self.time_of_day < self.dusk_start
    }

    /// Get the current phase as a fraction (0.0 midnight, 0.5 noon)
    pub fn solar_phase(&self) -> f32 {
        self.time_of_day / self.hours_per_day
    }

    /// Get normalized sun position (0.0 = midnight, 1.0 = noon)
    pub fn sun_position(&self) -> f32 {
        let phase = self.solar_phase();
        // Sinusoidal: peaks at noon (0.5)
        (phase * std::f32::consts::PI).sin().max(0.0)
    }

    /// Visibility modifier (0.3 at night, 1.0 at peak day)
    pub fn visibility_modifier(&self) -> f32 {
        let base = self.sun_position();

        // Dawn/dusk softening
        let transition = if self.is_dawn() {
            let t = (self.time_of_day - self.dawn_start) / (self.dawn_end - self.dawn_start);
            t * 0.5 + 0.2
        } else if self.is_dusk() {
            let t = (self.time_of_day - self.dusk_start) / (self.dusk_end - self.dusk_start);
            (1.0 - t) * 0.5 + 0.2
        } else {
            base
        };

        let day_factor = transition.max(0.3);
        let cloud_factor = 1.0 - self.weather.cloud_cover * 0.3;
        let fog_factor = 1.0 - self.weather.fog_density * 0.6;

        (day_factor * cloud_factor * fog_factor).clamp(0.1, 1.0)
    }

    /// Energy modifier (agents more active during day, sleepy at night)
    pub fn energy_modifier(&self) -> f32 {
        let base = if self.is_night() {
            0.5
        } else if self.is_dawn() || self.is_dusk() {
            0.75
        } else {
            1.0
        };

        // Season affects energy
        let season_mod: f32 = match self.season {
            CycleSeason::Spring => 1.1,
            CycleSeason::Summer => 1.0,
            CycleSeason::Fall => 0.9,
            CycleSeason::Winter => 0.7,
        };

        (base * season_mod).clamp(0.3, 1.2)
    }

    /// Perception modifier (how well agents can perceive their environment)
    pub fn perception_modifier(&self) -> f32 {
        let vis = self.visibility_modifier();
        // Perception is slightly better than pure visibility at dawn/dusk
        if self.is_dawn() || self.is_dusk() {
            (vis + 0.1).min(1.0)
        } else {
            vis
        }
    }

    /// Danger modifier (more dangerous at night)
    pub fn danger_modifier(&self) -> f32 {
        if self.is_night() {
            1.5
        } else if self.is_dawn() || self.is_dusk() {
            1.2
        } else {
            1.0
        }
    }

    /// Social activity modifier (agents more social during day)
    pub fn social_modifier(&self) -> f32 {
        if self.is_night() {
            0.3
        } else if self.is_dawn() || self.is_dusk() {
            0.6
        } else {
            1.0
        }
    }

    /// Get a display string for the current time
    pub fn display_time(&self) -> String {
        let hours = self.time_of_day as u32;
        let minutes = ((self.time_of_day - hours as f32) * 60.0) as u32;
        let phase = if self.is_night() { "Night" }
        else if self.is_dawn() { "Dawn" }
        else if self.is_dusk() { "Dusk" }
        else { "Day" };
        format!("{:02}:{:02} ({})", hours, minutes, phase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_default() {
        let cycle = DayNightCycle::new();
        assert_eq!(cycle.time_of_day, 6.0);
        assert_eq!(cycle.season, CycleSeason::Spring);
    }

    #[test]
    fn tick_advances_time() {
        let mut cycle = DayNightCycle::new();
        cycle.tick(1.0);
        assert!((cycle.time_of_day - 7.0).abs() < 0.001);
    }

    #[test]
    fn tick_wraps_around() {
        let mut cycle = DayNightCycle::new().with_time(23.5);
        cycle.tick(1.0);
        assert!(cycle.time_of_day < 1.0);
    }

    #[test]
    fn night_detection() {
        let cycle = DayNightCycle::new().with_time(2.0);
        assert!(cycle.is_night());
        assert!(!cycle.is_day());
    }

    #[test]
    fn dawn_detection() {
        let cycle = DayNightCycle::new().with_time(6.0);
        assert!(cycle.is_dawn());
        assert!(!cycle.is_night());
        assert!(!cycle.is_day());
    }

    #[test]
    fn day_detection() {
        let cycle = DayNightCycle::new().with_time(12.0);
        assert!(cycle.is_day());
        assert!(!cycle.is_night());
        assert!(!cycle.is_dawn());
    }

    #[test]
    fn dusk_detection() {
        let cycle = DayNightCycle::new().with_time(19.0);
        assert!(cycle.is_dusk());
        assert!(!cycle.is_night());
        assert!(!cycle.is_day());
    }

    #[test]
    fn visibility_at_night() {
        let cycle = DayNightCycle::new().with_time(0.0);
        assert!(cycle.visibility_modifier() < 0.5);
    }

    #[test]
    fn visibility_at_noon() {
        let cycle = DayNightCycle::new().with_time(12.0);
        assert!(cycle.visibility_modifier() > 0.9);
    }

    #[test]
    fn visibility_reduced_by_fog() {
        let mut cycle = DayNightCycle::new().with_time(12.0);
        let clear_vis = cycle.visibility_modifier();
        cycle.weather.fog_density = 0.8;
        let foggy_vis = cycle.visibility_modifier();
        assert!(foggy_vis < clear_vis);
    }

    #[test]
    fn energy_modifier_night_low() {
        let cycle = DayNightCycle::new().with_time(0.0);
        assert!(cycle.energy_modifier() < 0.7);
    }

    #[test]
    fn energy_modifier_day_high() {
        let cycle = DayNightCycle::new().with_time(12.0);
        assert!(cycle.energy_modifier() >= 1.0);
    }

    #[test]
    fn danger_at_night() {
        let cycle = DayNightCycle::new().with_time(0.0);
        assert!(cycle.danger_modifier() > 1.0);
    }

    #[test]
    fn danger_at_day() {
        let cycle = DayNightCycle::new().with_time(12.0);
        assert!(cycle.danger_modifier() == 1.0);
    }

    #[test]
    fn social_modifier_night_low() {
        let cycle = DayNightCycle::new().with_time(0.0);
        assert!(cycle.social_modifier() < 0.5);
    }

    #[test]
    fn solar_phase() {
        let cycle = DayNightCycle::new().with_time(12.0);
        assert!((cycle.solar_phase() - 0.5).abs() < 0.001);
    }

    #[test]
    fn sun_position_peaks_at_noon() {
        let cycle = DayNightCycle::new().with_time(12.0);
        assert!(cycle.sun_position() > 0.9);
    }

    #[test]
    fn sun_position_zero_at_midnight() {
        let cycle = DayNightCycle::new().with_time(0.0);
        assert!(cycle.sun_position() < 0.01);
    }

    #[test]
    fn season_affects_energy() {
        let mut winter = DayNightCycle::new().with_time(12.0);
        winter.season = CycleSeason::Winter;
        let summer = DayNightCycle::new().with_time(12.0);
        assert!(winter.energy_modifier() < summer.energy_modifier());
    }

    #[test]
    fn display_time_format() {
        let cycle = DayNightCycle::new().with_time(14.30);
        let display = cycle.display_time();
        assert!(display.contains("14"));
        assert!(display.contains("Day"));
    }
}
