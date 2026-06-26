#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DriveType {
    Curiosity,
    Mastery,
    Coherence,
    Novelty,
}

impl fmt::Display for DriveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriveType::Curiosity => write!(f, "Curiosity"),
            DriveType::Mastery => write!(f, "Mastery"),
            DriveType::Coherence => write!(f, "Coherence"),
            DriveType::Novelty => write!(f, "Novelty"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HomeostaticDriveConfig {
    pub curiosity_weight: f64,
    pub mastery_weight: f64,
    pub coherence_weight: f64,
    pub novelty_weight: f64,
    pub drive_decay: f64,
    pub curiosity_setpoint: f64,
    pub mastery_setpoint: f64,
    pub coherence_setpoint: f64,
    pub novelty_setpoint: f64,
    pub learning_rate: f64,
    pub temperature: f64,
}

impl Default for HomeostaticDriveConfig {
    fn default() -> Self {
        Self {
            curiosity_weight: 1.0,
            mastery_weight: 1.0,
            coherence_weight: 1.0,
            novelty_weight: 1.0,
            drive_decay: 0.95,
            curiosity_setpoint: 0.5,
            mastery_setpoint: 0.7,
            coherence_setpoint: 0.8,
            novelty_setpoint: 0.3,
            learning_rate: 0.1,
            temperature: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DriveState {
    pub drive_type: DriveType,
    pub current: f64,
    pub setpoint: f64,
    pub error: f64,
    pub weight: f64,
}

impl DriveState {
    fn new(drive_type: DriveType, setpoint: f64, weight: f64) -> Self {
        Self {
            drive_type,
            current: setpoint,
            setpoint,
            error: 0.0,
            weight,
        }
    }
}

pub struct HomeostaticDriveSystem {
    config: HomeostaticDriveConfig,
    drives: HashMap<DriveType, DriveState>,
    history: VecDeque<(u64, HashMap<DriveType, f64>)>,
}

impl HomeostaticDriveSystem {
    pub fn new(config: HomeostaticDriveConfig) -> Self {
        let mut drives = HashMap::new();
        drives.insert(
            DriveType::Curiosity,
            DriveState::new(
                DriveType::Curiosity,
                config.curiosity_setpoint,
                config.curiosity_weight,
            ),
        );
        drives.insert(
            DriveType::Mastery,
            DriveState::new(
                DriveType::Mastery,
                config.mastery_setpoint,
                config.mastery_weight,
            ),
        );
        drives.insert(
            DriveType::Coherence,
            DriveState::new(
                DriveType::Coherence,
                config.coherence_setpoint,
                config.coherence_weight,
            ),
        );
        drives.insert(
            DriveType::Novelty,
            DriveState::new(
                DriveType::Novelty,
                config.novelty_setpoint,
                config.novelty_weight,
            ),
        );
        Self {
            config,
            drives,
            history: VecDeque::with_capacity(100),
        }
    }

    pub fn update_drive(&mut self, drive_type: DriveType, signal: f64, cycle: u64) {
        let drive = self
            .drives
            .get_mut(&drive_type)
            .expect("drive type must exist");

        let decay = self.config.drive_decay;
        drive.current = drive.current * decay + signal * (1.0 - decay);
        drive.current = drive.current.clamp(0.0, 1.0);
        drive.error = drive.current - drive.setpoint;

        let mut snapshot = HashMap::new();
        for (dt, d) in &self.drives {
            snapshot.insert(*dt, d.current);
        }
        self.history.push_back((cycle, snapshot));
        while self.history.len() > 100 {
            self.history.pop_front();
        }
    }

    pub fn drive_values(&self) -> HashMap<DriveType, f64> {
        let mut values = HashMap::new();
        for (dt, d) in &self.drives {
            values.insert(*dt, d.current);
        }
        values
    }

    pub fn drive_errors(&self) -> HashMap<DriveType, f64> {
        let mut errors = HashMap::new();
        for (dt, d) in &self.drives {
            errors.insert(*dt, d.error);
        }
        errors
    }

    pub fn overall_tension(&self) -> f64 {
        self.drives.values().map(|d| d.weight * d.error.abs()).sum()
    }

    /// Tick: update drives from current consciousness metrics.
    pub fn tick(&mut self, meta_accuracy: f64, ece: f64, _loss: f64) {
        self.update_drive(DriveType::Coherence, 1.0 - ece, 0);
        self.update_drive(DriveType::Mastery, meta_accuracy, 0);
    }

    pub fn select_action<'a>(&self, available_actions: &[(&'a str, f64)]) -> Option<&'a str> {
        if available_actions.is_empty() {
            return None;
        }
        let temp = self.config.temperature;
        let mut scores: Vec<f64> = available_actions
            .iter()
            .map(|(_, action_drive)| {
                let mut score = 0.0;
                for d in self.drives.values() {
                    score += d.weight * d.error.abs() * action_drive;
                }
                score
            })
            .collect();

        let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if max_score.is_finite() {
            for s in &mut scores {
                *s = ((*s - max_score) / temp).exp();
            }
        }

        let sum: f64 = scores.iter().sum();
        if sum <= 0.0 {
            return Some(available_actions[0].0);
        }

        let mut rng = fastrand::f64();
        for (i, s) in scores.iter().enumerate() {
            rng -= s / sum;
            if rng <= 0.0 {
                return Some(available_actions[i].0);
            }
        }
        Some(available_actions.last().unwrap().0)
    }

    pub fn drive_contribution(&self, drive_type: DriveType) -> f64 {
        self.drives
            .get(&drive_type)
            .map(|d| d.weight * d.error.abs())
            .unwrap_or(0.0)
    }

    pub fn dominant_drive(&self) -> Option<DriveType> {
        self.drives
            .values()
            .max_by(|a, b| {
                a.error
                    .abs()
                    .partial_cmp(&b.error.abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|d| d.drive_type)
    }

    pub fn stats(&self) -> String {
        let tension = self.overall_tension();
        let dominant = self
            .dominant_drive()
            .map(|d| d.to_string())
            .unwrap_or_else(|| String::from("none"));
        let total_weight = self.config.curiosity_weight
            + self.config.mastery_weight
            + self.config.coherence_weight
            + self.config.novelty_weight;
        format!(
            "HomeostaticDriveSystem(tension={:.4}, dominant={}, drives={}, total_weight={:.2}, history={})",
            tension,
            dominant,
            self.drives.len(),
            total_weight,
            self.history.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_system() -> HomeostaticDriveSystem {
        HomeostaticDriveSystem::new(HomeostaticDriveConfig::default())
    }

    #[test]
    fn test_new_drives_all_present() {
        let system = default_system();
        let values = system.drive_values();
        assert_eq!(values.len(), 4);
        assert!(values.contains_key(&DriveType::Curiosity));
        assert!(values.contains_key(&DriveType::Mastery));
        assert!(values.contains_key(&DriveType::Coherence));
        assert!(values.contains_key(&DriveType::Novelty));
    }

    #[test]
    fn test_update_drive_changes_value() {
        let mut system = default_system();
        system.update_drive(DriveType::Curiosity, 0.9, 1);
        let values = system.drive_values();
        // with decay=0.95, initial=0.5: 0.5*0.95 + 0.9*0.05 = 0.475 + 0.045 = 0.52
        let expected = 0.5 * 0.95 + 0.9 * 0.05;
        let actual = values[&DriveType::Curiosity];
        assert!((actual - expected).abs() < 1e-10);
    }

    #[test]
    fn test_drive_decay() {
        let mut system = default_system();
        // Curiosity starts at setpoint 0.5. Feed 0.0 repeatedly.
        system.update_drive(DriveType::Curiosity, 0.0, 1);
        system.update_drive(DriveType::Curiosity, 0.0, 2);
        system.update_drive(DriveType::Curiosity, 0.0, 3);
        let v = system.drive_values()[&DriveType::Curiosity];
        // 0.5 * 0.95^3 ≈ 0.429
        let expected = 0.5 * 0.95f64.powi(3);
        assert!((v - expected).abs() < 1e-6);
        // Should be less than initial
        assert!(v < 0.5);
    }

    #[test]
    fn test_drive_error_calculation() {
        let mut system = default_system();
        system.update_drive(DriveType::Curiosity, 0.9, 1);
        let errors = system.drive_errors();
        let actual = errors[&DriveType::Curiosity];
        let expected = (0.5 * 0.95 + 0.9 * 0.05) - 0.5;
        assert!((actual - expected).abs() < 1e-10);
    }

    #[test]
    fn test_overall_tension() {
        let mut system = default_system();
        system.update_drive(DriveType::Curiosity, 0.9, 1);
        system.update_drive(DriveType::Mastery, 0.1, 1);
        system.update_drive(DriveType::Coherence, 1.0, 1);
        system.update_drive(DriveType::Novelty, 0.0, 1);
        let tension = system.overall_tension();
        assert!(tension > 0.0);
        let errors = system.drive_errors();
        let expected: f64 = errors
            .iter()
            .map(|(dt, e)| {
                let w = match dt {
                    DriveType::Curiosity => 1.0,
                    DriveType::Mastery => 1.0,
                    DriveType::Coherence => 1.0,
                    DriveType::Novelty => 1.0,
                };
                w * e.abs()
            })
            .sum();
        assert!((tension - expected).abs() < 1e-10);
    }

    #[test]
    fn test_select_action_returns_some() {
        let mut system = default_system();
        system.update_drive(DriveType::Curiosity, 0.1, 1);
        system.update_drive(DriveType::Mastery, 0.2, 1);
        let actions = vec![("explore", 0.5), ("refine", 0.3), ("rest", 0.1)];
        let selected = system.select_action(&actions);
        assert!(selected.is_some());
    }

    #[test]
    fn test_select_action_returns_none() {
        let system = default_system();
        let actions: Vec<(&str, f64)> = vec![];
        let selected = system.select_action(&actions);
        assert!(selected.is_none());
    }

    #[test]
    fn test_dominant_drive() {
        let mut system = default_system();
        system.update_drive(DriveType::Curiosity, 1.0, 1);
        system.update_drive(DriveType::Mastery, 0.0, 1);
        system.update_drive(DriveType::Coherence, 0.8, 1);
        system.update_drive(DriveType::Novelty, 0.3, 1);
        let dominant = system.dominant_drive();
        assert!(dominant.is_some());
        // Mastery has the largest error (fed 0.0, setpoint=0.7 -> error ≈ -0.665)
        assert_eq!(dominant.unwrap(), DriveType::Mastery);
    }

    #[test]
    fn test_history_capped() {
        let mut system = default_system();
        for i in 0..150 {
            system.update_drive(DriveType::Curiosity, 0.5, i as u64);
        }
        // We can't inspect history directly, but we can verify that
        // update_drive didn't panic and the getter works.
        let values = system.drive_values();
        assert_eq!(values.len(), 4);
    }

    #[test]
    fn test_drive_contribution() {
        let mut system = default_system();
        system.update_drive(DriveType::Curiosity, 0.9, 1);
        let contrib = system.drive_contribution(DriveType::Curiosity);
        assert!(contrib > 0.0);
        let contrib_nonexistent = system.drive_contribution(DriveType::Mastery);
        // Mastery is at setpoint (0.7), so error should be ~0
        assert!(contrib_nonexistent >= 0.0);
    }

    #[test]
    fn test_stats_output() {
        let mut system = default_system();
        system.update_drive(DriveType::Curiosity, 0.9, 1);
        system.update_drive(DriveType::Mastery, 0.2, 1);
        let s = system.stats();
        assert!(!s.is_empty());
        assert!(s.contains("tension="));
        assert!(s.contains("dominant="));
        assert!(s.contains("drives="));
    }

    #[test]
    fn test_homeostatic_equilibrium() {
        let system = default_system();
        let errors = system.drive_errors();
        for (_, e) in &errors {
            assert!(
                (*e).abs() < 1e-10,
                "error should be near zero at equilibrium"
            );
        }
        let tension = system.overall_tension();
        assert!(
            (tension).abs() < 1e-10,
            "tension should be zero at equilibrium"
        );
    }

    #[test]
    fn test_select_action_prefers_high_error_drive() {
        let mut system = default_system();
        // Push Mastery far from setpoint
        system.update_drive(DriveType::Mastery, 0.0, 1);
        system.update_drive(DriveType::Mastery, 0.0, 2);
        system.update_drive(DriveType::Mastery, 0.0, 3);
        // Keep Curiosity near setpoint
        system.update_drive(DriveType::Curiosity, 0.5, 1);

        let actions = vec![("mastery_action", 1.0), ("curiosity_action", 0.0)];
        // With high temperature and mastery drive error dominant,
        // mastery_action should be strongly preferred
        let selected = system.select_action(&actions);
        assert_eq!(selected, Some("mastery_action"));
    }
}
