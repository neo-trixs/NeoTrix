// SimTime - Independent simulation clock with day/night, seasons, calendar
//
// Design rationale: OpenLife proved that budget-based metabolism creates real
// survival pressure. SimTime creates temporal pressure: agents must act within
// time constraints, seasons affect resource availability, day/night affects
// perception range and safety.

use serde::{Serialize, Deserialize};
use std::fmt;

use super::simulation_bus::{SimEvent, EventPriority, Season};

/// Extended simulation time with rich calendar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimClock {
    pub current: super::simulation_bus::SimTime,
    pub ticks_per_hour: u64,
    pub hours_per_day: u8,
    pub days_per_season: u32,
    pub seasons_per_year: u8,
    pub total_ticks: u64,
}

impl Default for SimClock {
    fn default() -> Self {
        Self {
            current: super::simulation_bus::SimTime {
                tick: 0,
                day: 1,
                hour: 6,
                minute: 0,
                season: Season::Spring,
            },
            ticks_per_hour: 100,
            hours_per_day: 24,
            days_per_season: 30,
            seasons_per_year: 4,
            total_ticks: 0,
        }
    }
}

impl SimClock {
    pub fn new(ticks_per_hour: u64) -> Self {
        Self {
            ticks_per_hour,
            ..Default::default()
        }
    }

    /// Advance one tick, returns events that occurred
    pub fn tick(&mut self) -> Vec<(SimEvent, EventPriority)> {
        let mut events = Vec::new();
        self.total_ticks += 1;
        self.current.tick = self.total_ticks;

        let ticks_per_minute = self.ticks_per_hour / 60;
        if ticks_per_minute > 0 && self.total_ticks % ticks_per_minute == 0 {
            self.current.minute += 1;
            if self.current.minute >= 60 {
                self.current.minute = 0;
                let _old_hour = self.current.hour;
                self.current.hour += 1;

                events.push((
                    SimEvent::TimeAdvanced { time: self.current },
                    EventPriority::Low,
                ));

                if self.current.hour >= self.hours_per_day {
                    self.current.hour = 0;
                    self.current.day += 1;

                    if self.current.day > self.days_per_season {
                        self.current.day = 1;
                        self.current.season = self.next_season();

                        events.push((
                            SimEvent::SeasonChanged { season: self.current.season },
                            EventPriority::High,
                        ));
                    }
                }
            }
        }

        events
    }

    /// Get day phase for perception/behavior modifiers
    pub fn day_phase(&self) -> DayPhase {
        match self.current.hour {
            0..=4 => DayPhase::LateNight,
            5..=7 => DayPhase::Dawn,
            8..=11 => DayPhase::Morning,
            12..=13 => DayPhase::Noon,
            14..=17 => DayPhase::Afternoon,
            18..=20 => DayPhase::Dusk,
            21..=23 => DayPhase::Night,
            _ => DayPhase::Night, // impossible with valid time, but satisfy compiler
        }
    }

    /// Perception modifier based on time (0.0 = blind, 1.0 = full)
    pub fn perception_modifier(&self) -> f32 {
        match self.day_phase() {
            DayPhase::Dawn | DayPhase::Dusk => 0.7,
            DayPhase::Morning | DayPhase::Afternoon => 1.0,
            DayPhase::Noon => 0.9,
            DayPhase::Night => 0.4,
            DayPhase::LateNight => 0.2,
        }
    }

    /// Energy regeneration modifier based on season
    pub fn season_energy_modifier(&self) -> f32 {
        match self.current.season {
            Season::Spring => 1.1,
            Season::Summer => 1.0,
            Season::Autumn => 0.9,
            Season::Winter => 0.7,
        }
    }

    /// Resource scarcity modifier based on season
    pub fn season_resource_modifier(&self) -> f32 {
        match self.current.season {
            Season::Spring => 1.2,
            Season::Summer => 1.0,
            Season::Autumn => 0.8,
            Season::Winter => 0.5,
        }
    }

    fn next_season(&self) -> Season {
        match self.current.season {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }

    /// Format as human-readable string
    pub fn display(&self) -> String {
        format!(
            "Day {} {:02}:{:02} ({:?}) [tick {}]",
            self.current.day, self.current.hour, self.current.minute,
            self.current.season, self.total_ticks
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayPhase {
    Dawn,
    Morning,
    Noon,
    Afternoon,
    Dusk,
    Night,
    LateNight,
}

impl fmt::Display for DayPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DayPhase::Dawn => write!(f, "Dawn"),
            DayPhase::Morning => write!(f, "Morning"),
            DayPhase::Noon => write!(f, "Noon"),
            DayPhase::Afternoon => write!(f, "Afternoon"),
            DayPhase::Dusk => write!(f, "Dusk"),
            DayPhase::Night => write!(f, "Night"),
            DayPhase::LateNight => write!(f, "Late Night"),
        }
    }
}

/// World time modifiers that affect simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeModifiers {
    pub perception: f32,
    pub energy_regen: f32,
    pub resource_availability: f32,
    pub danger_level: f32,
    pub social_activity: f32,
}

impl TimeModifiers {
    pub fn from_clock(clock: &SimClock) -> Self {
        let night_factor = match clock.day_phase() {
            DayPhase::Night | DayPhase::LateNight => 1.0,
            DayPhase::Dawn | DayPhase::Dusk => 0.5,
            _ => 0.0,
        };

        Self {
            perception: clock.perception_modifier(),
            energy_regen: clock.season_energy_modifier(),
            resource_availability: clock.season_resource_modifier(),
            danger_level: night_factor * 0.8,
            social_activity: 1.0 - night_factor * 0.7,
        }
    }
}
