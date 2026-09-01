use serde::{Deserialize, Serialize};

/// #12 — Peak Window: optimal performance window.
/// Tracks circadian-like energy/resource cycles to identify the optimal
/// time windows for high-cost cognitive operations.
/// Biological analog: peak performance windows in athletic/cognitive training.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakWindow {
    /// Internal clock value [0, 1) representing position in cycle
    pub phase: f64,
    /// Energy/resource level [0, 1]
    pub energy: f64,
    /// Duration of a full cycle in iterations
    pub cycle_length: f64,
    /// Current estimated performance capacity [0, 1]
    pub capacity: f64,
    /// Whether currently in a peak window
    pub in_peak: bool,
    /// Time until next peak (in iterations)
    pub ticks_to_peak: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakSchedule {
    /// Energy peak phase (where in [0,1) energy is maximized)
    pub peak_phase: f64,
    /// Width of the peak window as fraction of cycle
    pub peak_width: f64,
    /// Base energy recovery rate
    pub recovery_rate: f64,
    /// Energy consumption rate under load
    pub consumption_rate: f64,
    /// Minimum resting energy
    pub min_energy: f64,
}

impl Default for PeakSchedule {
    fn default() -> Self {
        Self {
            peak_phase: 0.25,
            peak_width: 0.15,
            recovery_rate: 0.05,
            consumption_rate: 0.08,
            min_energy: 0.15,
        }
    }
}

impl PeakWindow {
    pub fn new(phase: f64, cycle_length: f64) -> Self {
        let schedule = PeakSchedule::default();
        let dist = (phase - schedule.peak_phase).abs().min(1.0 - (phase - schedule.peak_phase).abs());
        let in_peak = dist < schedule.peak_width / 2.0;
        let capacity = Self::compute_capacity(phase, &schedule);
        let ticks_to_peak = Self::ticks_to_next_peak(phase, &schedule, cycle_length);
        Self {
            phase,
            energy: 0.8,
            cycle_length,
            capacity,
            in_peak,
            ticks_to_peak,
        }
    }

    fn compute_capacity(phase: f64, schedule: &PeakSchedule) -> f64 {
        let dist = (phase - schedule.peak_phase).abs().min(1.0 - (phase - schedule.peak_phase).abs());
        let raw = (-dist * 20.0).exp();
        raw.max(0.1).min(1.0)
    }

    fn ticks_to_next_peak(phase: f64, schedule: &PeakSchedule, cycle_length: f64) -> f64 {
        let mut next = schedule.peak_phase - phase;
        if next < 0.0 {
            next += 1.0;
        }
        next * cycle_length
    }

    /// Advance the internal clock by `delta` iterations, updating energy and capacity.
    pub fn advance(&mut self, delta: f64, under_load: bool) {
        self.phase = (self.phase + delta / self.cycle_length).fract();
        if self.phase < 0.0 {
            self.phase += 1.0;
        }
        let schedule = PeakSchedule::default();
        if under_load {
            self.energy = (self.energy - schedule.consumption_rate * delta).max(schedule.min_energy);
        } else {
            self.energy = (self.energy + schedule.recovery_rate * delta).min(1.0);
        }
        let dist = (self.phase - schedule.peak_phase).abs()
            .min(1.0 - (self.phase - schedule.peak_phase).abs());
        self.in_peak = dist < schedule.peak_width / 2.0;
        self.capacity = Self::compute_capacity(self.phase, &schedule) * self.energy;
        self.ticks_to_peak = Self::ticks_to_next_peak(self.phase, &schedule, self.cycle_length);
    }

    /// Whether it is advisable to start a high-cost operation now.
    pub fn should_execute_high_cost(&self, cost: f64) -> bool {
        self.capacity >= cost
    }

    /// Recommend optimal delay for a high-cost operation (in iterations).
    pub fn recommend_delay(&self, cost: f64) -> f64 {
        if self.should_execute_high_cost(cost) {
            return 0.0;
        }
        let _schedule = PeakSchedule::default();
        // Compute how many ticks until capacity >= cost
        let mut probe = self.clone();
        let mut ticks = 0.0;
        while ticks < self.cycle_length * 2.0 {
            probe.advance(1.0, false);
            ticks += 1.0;
            if probe.capacity >= cost {
                return ticks;
            }
        }
        self.ticks_to_peak
    }

    pub fn capacity(&self) -> f64 { self.capacity }
    pub fn energy(&self) -> f64 { self.energy }
    pub fn in_peak(&self) -> bool { self.in_peak }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peak_at_default_phase() {
        let pw = PeakWindow::new(0.25, 100.0);
        assert!(pw.in_peak);
        assert!(pw.capacity > 0.8);
    }

    #[test]
    fn test_off_peak_low_capacity() {
        let pw = PeakWindow::new(0.75, 100.0);
        assert!(!pw.in_peak);
        assert!(pw.capacity < 0.5);
    }

    #[test]
    fn test_advance_moves_phase() {
        let mut pw = PeakWindow::new(0.0, 100.0);
        pw.advance(50.0, false);
        assert!(pw.phase > 0.45 && pw.phase < 0.55);
    }

    #[test]
    fn test_energy_drops_under_load() {
        let mut pw = PeakWindow::new(0.5, 100.0);
        let initial = pw.energy;
        pw.advance(10.0, true);
        assert!(pw.energy < initial);
    }

    #[test]
    fn test_recommend_delay_returns_zero_when_ready() {
        let pw = PeakWindow::new(0.25, 100.0);
        assert_eq!(pw.recommend_delay(0.3), 0.0);
    }

    #[test]
    fn test_recommend_delay_positive_when_not_ready() {
        let pw = PeakWindow::new(0.75, 100.0);
        let delay = pw.recommend_delay(0.8);
        assert!(delay > 0.0);
    }
}
