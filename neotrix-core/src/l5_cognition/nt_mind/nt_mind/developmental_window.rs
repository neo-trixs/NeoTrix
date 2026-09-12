use serde::{Deserialize, Serialize};

/// #11 — Developmental Window: critical period _plasticity.
/// Models time-limited windows where the system has heightened _plasticity
/// for acquiring new skills or adapting to environmental changes.
/// Biological analog: critical periods in neurodevelopment (e.g., language acquisition).

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentalWindow {
    /// Current developmental stage index
    pub stage: u32,
    /// Total iteration count of the system
    pub age_iters: u64,
    /// Plasticity multiplier during active window [0, 1]
    pub _plasticity: f64,
    /// Whether currently in a critical window
    pub in_window: bool,
    /// How many windows have been completed
    pub windows_completed: u32,
    /// Learning _acceleration factor during window (e.g., 3.0 = 3x)
    pub _acceleration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentalSchedule {
    /// Each entry: (start_iter, end_iter, _plasticity, _acceleration)
    pub windows: Vec<(u64, u64, f64, f64)>,
}

impl Default for DevelopmentalSchedule {
    fn default() -> Self {
        Self {
            windows: vec![
                (0, 100, 1.0, 3.0),      // Infancy: maximal _plasticity
                (100, 500, 0.8, 2.0),     // Early development
                (500, 2000, 0.6, 1.5),    // Juvenile
                (2000, 5000, 0.4, 1.2),   // Adolescent
                (5000, 10000, 0.25, 1.0), // Adult
                (10000, 50000, 0.15, 0.8), // Mature
            ],
        }
    }
}

impl DevelopmentalWindow {
    pub fn new(age_iters: u64) -> Self {
        let schedule = DevelopmentalSchedule::default();
        let mut stage = 0;
        let mut in_window = false;
        let mut _plasticity = 0.0;
        let mut _acceleration = 1.0;
        for (i, (start, end, pl, acc)) in schedule.windows.iter().enumerate() {
            if age_iters >= *start && age_iters < *end {
                stage = i as u32;
                in_window = true;
                _plasticity = *pl;
                _acceleration = *acc;
                break;
            }
        }
        Self {
            stage,
            age_iters,
            _plasticity,
            in_window,
            windows_completed: 0,
            _acceleration,
        }
    }

    /// Advance age by `delta` iterations.
    pub fn advance(&mut self, delta: u64) {
        self.age_iters += delta;
        let schedule = DevelopmentalSchedule::default();
        for (i, (start, end, pl, acc)) in schedule.windows.iter().enumerate() {
            if self.age_iters >= *start && self.age_iters < *end {
                let prev_stage = self.stage;
                self.stage = i as u32;
                self._plasticity = *pl;
                self._acceleration = *acc;
                self.in_window = true;
                if self.stage > prev_stage {
                    self.windows_completed += 1;
                }
                return;
            }
        }
        self.in_window = false;
        self._plasticity = 0.05;
        self._acceleration = 0.5;
    }

    /// Assess if a new skill can be acquired at current stage.
    pub(crate) fn _can_acquire_skill(&self, skill_complexity: f64) -> bool {
        if !self.in_window {
            return false;
        }
        skill_complexity <= self._plasticity * 2.0
    }

    /// Modulate learning rate based on developmental _plasticity.
    pub(crate) fn _modulated_rate(&self, base_rate: f64) -> f64 {
        base_rate * (1.0 + self._plasticity * (self._acceleration - 1.0))
    }

    pub(crate) fn _is_in_window(&self) -> bool { self.in_window }
    pub(crate) fn _plasticity(&self) -> f64 { self.plasticity }
    pub(crate) fn _acceleration(&self) -> f64 { self.acceleration }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newborn_has_max_plasticity() {
        let dw = DevelopmentalWindow::new(0);
        assert!(dw._is_in_window());
        assert!(dw._plasticity >= 0.9);
        assert!(dw._acceleration >= 2.5);
    }

    #[test]
    fn test_mature_age_low_plasticity() {
        let dw = DevelopmentalWindow::new(20000);
        assert!(dw._is_in_window());
        assert!(dw._plasticity <= 0.2);
    }

    #[test]
    fn test_post_schedule_no_window() {
        let dw = DevelopmentalWindow::new(100000);
        assert!(!dw._is_in_window());
    }

    #[test]
    fn test_advance_moves_through_stages() {
        let mut dw = DevelopmentalWindow::new(0);
        assert_eq!(dw.stage, 0);
        dw.advance(200);
        assert!(dw.stage >= 1, "Should advance past infancy");
    }

    #[test]
    fn test_can_acquire_simple_skill_in_early_stage() {
        let dw = DevelopmentalWindow::new(50);
        assert!(dw._can_acquire_skill(0.5));
    }

    #[test]
    fn test_cannot_acquire_complex_skill_in_late_stage() {
        let dw = DevelopmentalWindow::new(10000);
        assert!(!dw._can_acquire_skill(1.5));
    }
}
