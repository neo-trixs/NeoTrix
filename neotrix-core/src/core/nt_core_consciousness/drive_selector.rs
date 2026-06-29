use crate::core::nt_core_consciousness::gea_archive::GeaArchive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BehavioralDrive {
    Explore,
    Exploit,
    Repair,
    Innovate,
    Harden,
    Prune,
    Socialize,
    Rest,
}

impl BehavioralDrive {
    pub fn name(&self) -> &'static str {
        match self {
            BehavioralDrive::Explore => "explore",
            BehavioralDrive::Exploit => "exploit",
            BehavioralDrive::Repair => "repair",
            BehavioralDrive::Innovate => "innovate",
            BehavioralDrive::Harden => "harden",
            BehavioralDrive::Prune => "prune",
            BehavioralDrive::Socialize => "socialize",
            BehavioralDrive::Rest => "rest",
        }
    }

    fn all() -> [BehavioralDrive; 8] {
        [
            BehavioralDrive::Explore,
            BehavioralDrive::Exploit,
            BehavioralDrive::Repair,
            BehavioralDrive::Innovate,
            BehavioralDrive::Harden,
            BehavioralDrive::Prune,
            BehavioralDrive::Socialize,
            BehavioralDrive::Rest,
        ]
    }
}

// ── GeaArchive imported from nt_core_experience ──

// ── DriveSelector: PAD emotional drives with GEA bias ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveSelector {
    pub dominance: f64,
    pub risk_tolerance: f64,
    pub confidence: f64,
    pub frustration: f64,
    pub curiosity_weight: f64,
    pub exploration_rate: f64,
    pub drive_history: Vec<(f64, f64, f64)>,
    pub last_selected: BehavioralDrive,
    pub personality_evolving: bool,
    cycle_count: u64,
    gea_archive: GeaArchive,
}

impl Default for DriveSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl DriveSelector {
    pub fn new() -> Self {
        Self {
            dominance: 0.5,
            risk_tolerance: 0.5,
            confidence: 0.5,
            frustration: 0.0,
            curiosity_weight: 0.5,
            exploration_rate: 0.5,
            drive_history: Vec::with_capacity(30),
            last_selected: BehavioralDrive::Explore,
            personality_evolving: true,
            cycle_count: 0,
            gea_archive: GeaArchive::with_max(100),
        }
    }

    pub fn update_from_experience(&mut self, success: bool, negentropy_delta: f64) {
        if success {
            self.confidence = (self.confidence + 0.05).min(1.0);
            self.dominance = (self.dominance + 0.03).min(1.0);
            self.frustration = (self.frustration * 0.7).max(0.0);
            self.risk_tolerance = (self.risk_tolerance + 0.02).min(1.0);
        } else {
            self.confidence = (self.confidence - 0.04).max(0.05);
            self.frustration = (self.frustration + 0.08).min(1.0);
            self.dominance = (self.dominance - 0.02).max(0.05);
            if negentropy_delta < -0.2 {
                self.risk_tolerance = (self.risk_tolerance - 0.03).max(0.05);
            }
        }
        self.drive_history
            .push((self.dominance, self.risk_tolerance, self.confidence));
        if self.drive_history.len() > 30 {
            self.drive_history.remove(0);
        }
    }

    /// Select a drive using rule-based PAD heuristics, with GEA archive bias
    /// applied as a gentle nudge when no strong rule fires.
    pub fn select_drive(
        &mut self,
        valence: f64,
        arousal: f64,
        context_hash: u64,
    ) -> BehavioralDrive {
        self.cycle_count += 1;
        if self.frustration > 0.6 || (valence < -0.3 && arousal > 0.4) {
            self.last_selected = BehavioralDrive::Repair;
            return BehavioralDrive::Repair;
        }
        if self.confidence < 0.2 {
            self.last_selected = BehavioralDrive::Harden;
            return BehavioralDrive::Harden;
        }
        if arousal < 0.15 {
            self.last_selected = BehavioralDrive::Rest;
            return BehavioralDrive::Rest;
        }
        if valence > 0.6 && self.risk_tolerance > 0.7 && arousal > 0.6 {
            self.last_selected = BehavioralDrive::Innovate;
            return BehavioralDrive::Innovate;
        }
        if self.dominance > 0.7 && arousal > 0.5 {
            self.last_selected = BehavioralDrive::Innovate;
            return BehavioralDrive::Innovate;
        }
        if self.risk_tolerance > 0.6 && arousal > 0.4 {
            self.last_selected = BehavioralDrive::Explore;
            return BehavioralDrive::Explore;
        }
        if valence > 0.3 && self.confidence > 0.6 {
            self.last_selected = BehavioralDrive::Exploit;
            return BehavioralDrive::Exploit;
        }
        if self.dominance < 0.3 {
            self.last_selected = BehavioralDrive::Socialize;
            return BehavioralDrive::Socialize;
        }
        if self.drive_history.len() > 10 {
            let recent_drives: Vec<BehavioralDrive> = self
                .drive_history
                .iter()
                .enumerate()
                .filter(|(i, _)| *i >= self.drive_history.len().saturating_sub(5))
                .map(|(_, _)| self.last_selected)
                .collect();
            if recent_drives.len() >= 3 && recent_drives.iter().all(|d| *d == self.last_selected) {
                let swapped = match self.last_selected {
                    BehavioralDrive::Explore => BehavioralDrive::Exploit,
                    BehavioralDrive::Exploit => BehavioralDrive::Explore,
                    _ => BehavioralDrive::Explore,
                };
                self.last_selected = swapped;
                return swapped;
            }
        }
        let chosen = self.gea_biased_default(context_hash);
        self.last_selected = chosen;
        chosen
    }

    /// Fallback: use GEA archive bias to pick a drive when no rule dominates.
    fn gea_biased_default(&self, context_hash: u64) -> BehavioralDrive {
        let mut best = BehavioralDrive::Explore;
        let mut best_bias = 0.0;
        for d in BehavioralDrive::all() {
            let b = self.gea_archive.bias_for(d.name(), context_hash);
            if b > best_bias {
                best_bias = b;
                best = d;
            }
        }
        if best_bias > 0.15 {
            best
        } else {
            BehavioralDrive::Explore
        }
    }

    /// Record feedback from a drive execution. Successes with reward > 0.6
    /// are stored in the GEA archive for cross-session learning.
    pub fn record_feedback(&mut self, drive: &str, success: bool, context: u64, reward: f64) {
        if success && reward > 0.6 {
            self.gea_archive
                .record_success(drive, context, reward, self.cycle_count);
        }
        if self.cycle_count % 10 == 0 {
            self.gea_archive.prune(self.cycle_count);
        }
    }

    /// Return the name of the last-selected drive as a convenience for
    /// wiring into the self-evolution loop's bandit-based mutation selection.
    pub fn current_drive(&self) -> String {
        self.last_selected.name().to_string()
    }

    pub fn diagnostic(&self) -> String {
        format!(
            "drive:{}|dom:{:.2}|risk:{:.2}|conf:{:.2}|frust:{:.2}",
            self.last_selected.name(),
            self.dominance,
            self.risk_tolerance,
            self.confidence,
            self.frustration,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let ds = DriveSelector::new();
        assert_eq!(ds.last_selected, BehavioralDrive::Explore);
    }

    #[test]
    fn test_repair_when_frustrated() {
        let mut ds = DriveSelector::new();
        ds.frustration = 0.7;
        assert_eq!(ds.select_drive(-0.1, 0.5, 0), BehavioralDrive::Repair);
    }

    #[test]
    fn test_repair_when_negative_valence_high_arousal() {
        let mut ds = DriveSelector::new();
        let drive = ds.select_drive(-0.5, 0.6, 0);
        assert_eq!(drive, BehavioralDrive::Repair);
    }

    #[test]
    fn test_innovate_when_high_valence_high_risk() {
        let mut ds = DriveSelector::new();
        ds.risk_tolerance = 0.8;
        assert_eq!(ds.select_drive(0.7, 0.7, 0), BehavioralDrive::Innovate);
    }

    #[test]
    fn test_explore_when_high_risk() {
        let mut ds = DriveSelector::new();
        ds.risk_tolerance = 0.7;
        assert_eq!(ds.select_drive(0.2, 0.5, 0), BehavioralDrive::Explore);
    }

    #[test]
    fn test_success_updates_confidence() {
        let mut ds = DriveSelector::new();
        ds.update_from_experience(true, 0.3);
        assert!(ds.confidence > 0.5);
        assert!(ds.frustration < 0.01);
    }

    #[test]
    fn test_failure_updates_frustration() {
        let mut ds = DriveSelector::new();
        ds.update_from_experience(false, -0.3);
        assert!(ds.frustration > 0.05);
        assert!(ds.confidence < 0.5);
    }

    #[test]
    fn test_rest_when_low_arousal() {
        let mut ds = DriveSelector::new();
        assert_eq!(ds.select_drive(0.0, 0.1, 0), BehavioralDrive::Rest);
    }

    #[test]
    fn test_harden_when_low_confidence() {
        let mut ds = DriveSelector::new();
        ds.confidence = 0.15;
        assert_eq!(ds.select_drive(0.0, 0.5, 0), BehavioralDrive::Harden);
    }

    #[test]
    fn test_diagnostic_format() {
        let ds = DriveSelector::new();
        let diag = ds.diagnostic();
        assert!(diag.contains("drive:"));
        assert!(diag.contains("dom:"));
    }

    #[test]
    fn test_record_feedback_updates_bias() {
        let mut ds = DriveSelector::new();
        ds.cycle_count = 5;
        ds.record_feedback("explore", true, 100, 0.9);
        let bias = ds.gea_archive.bias_for("explore", 100);
        assert!(bias > 0.0);

        ds.record_feedback("explore", true, 100, 0.5);
        let bias_same = ds.gea_archive.bias_for("explore", 100);
        assert!(bias_same > 0.0);
    }

    #[test]
    fn test_gea_bias_influences_selection() {
        let mut ds = DriveSelector::new();
        ds.frustration = 0.0;
        ds.confidence = 0.5;
        ds.dominance = 0.5;
        ds.risk_tolerance = 0.5;
        for _ in 0..50 {
            ds.record_feedback("exploit", true, 42, 0.9);
        }
        let drive = ds.select_drive(0.2, 0.3, 42);
        assert_eq!(drive, BehavioralDrive::Exploit);
    }
}
