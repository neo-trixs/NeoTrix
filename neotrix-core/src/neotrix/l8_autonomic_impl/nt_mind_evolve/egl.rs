use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct EglSnapshot {
    pub pass_rate: f64,
    pub timestamp: std::time::Instant,
    pub iteration: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EglStatus {
    Improving { avg: f64, current: f64, diff: f64 },
    Stable { avg: f64, current: f64, diff: f64 },
    Regressing { avg: f64, current: f64, diff: f64 },
}

impl EglStatus {
    pub fn avg(&self) -> f64 {
        match self {
            EglStatus::Improving { avg, .. }
            | EglStatus::Stable { avg, .. }
            | EglStatus::Regressing { avg, .. } => *avg,
        }
    }

    pub fn current(&self) -> f64 {
        match self {
            EglStatus::Improving { current, .. }
            | EglStatus::Stable { current, .. }
            | EglStatus::Regressing { current, .. } => *current,
        }
    }

    pub fn diff(&self) -> f64 {
        match self {
            EglStatus::Improving { diff, .. }
            | EglStatus::Stable { diff, .. }
            | EglStatus::Regressing { diff, .. } => *diff,
        }
    }

    pub fn is_regressing(&self) -> bool {
        matches!(self, EglStatus::Regressing { .. })
    }

    pub fn is_improving(&self) -> bool {
        matches!(self, EglStatus::Improving { .. })
    }
}

#[derive(Debug, Clone)]
pub struct EglTracker {
    pub history: VecDeque<EglSnapshot>,
    pub window_size: usize,
    pub regression_threshold: f64,
    pub iteration: u64,
}

impl Default for EglTracker {
    fn default() -> Self {
        Self {
            history: VecDeque::with_capacity(10),
            window_size: 10,
            regression_threshold: -0.05,
            iteration: 0,
        }
    }
}

impl EglTracker {
    pub fn new(window_size: usize, regression_threshold: f64) -> Self {
        Self {
            history: VecDeque::with_capacity(window_size),
            window_size,
            regression_threshold,
            iteration: 0,
        }
    }

    pub fn track(&mut self, pass_rate: f64) -> EglStatus {
        self.iteration += 1;
        let snapshot = EglSnapshot {
            pass_rate,
            timestamp: std::time::Instant::now(),
            iteration: self.iteration,
        };
        self.history.push_back(snapshot);

        while self.history.len() > self.window_size {
            self.history.pop_front();
        }

        let avg = self.average();
        let diff = pass_rate - avg;

        if pass_rate < avg + self.regression_threshold {
            EglStatus::Regressing { avg, current: pass_rate, diff }
        } else if pass_rate > avg + 0.05 {
            EglStatus::Improving { avg, current: pass_rate, diff }
        } else {
            EglStatus::Stable { avg, current: pass_rate, diff }
        }
    }

    pub fn average(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.history.iter().map(|s| s.pass_rate).sum();
        sum / self.history.len() as f64
    }

    pub fn is_converged(&self, egl_window: usize, egl_threshold: f64) -> bool {
        if self.history.len() < egl_window {
            return false;
        }
        let recent: Vec<&EglSnapshot> = self.history.iter().rev().take(egl_window).collect();
        if recent.len() < 2 {
            return false;
        }
        for pair in recent.windows(2) {
            let diff = (pair[0].pass_rate - pair[1].pass_rate).abs();
            if diff >= egl_threshold {
                return false;
            }
        }
        true
    }

    pub fn reset(&mut self) {
        self.history.clear();
        self.iteration = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_egl_tracker_new() {
        let tracker = EglTracker::new(5, -0.05);
        assert_eq!(tracker.window_size, 5);
        assert!((tracker.regression_threshold - (-0.05)).abs() < 1e-6);
        assert_eq!(tracker.iteration, 0);
        assert!(tracker.history.is_empty());
    }

    #[test]
    fn test_egl_tracker_default() {
        let tracker = EglTracker::default();
        assert_eq!(tracker.window_size, 10);
    }

    #[test]
    fn test_egl_tracker_improving() {
        let mut tracker = EglTracker::new(5, -0.05);
        tracker.track(0.5);
        let status = tracker.track(0.7);
        assert!(status.is_improving());
    }

    #[test]
    fn test_egl_tracker_regressing() {
        let mut tracker = EglTracker::new(5, -0.05);
        tracker.track(0.8);
        tracker.track(0.7);
        let status = tracker.track(0.5);
        assert!(status.is_regressing());
    }

    #[test]
    fn test_egl_tracker_stable() {
        let mut tracker = EglTracker::new(5, -0.05);
        tracker.track(0.5);
        tracker.track(0.51);
        let status = tracker.track(0.52);
        assert!(matches!(status, EglStatus::Stable { .. }));
    }

    #[test]
    fn test_egl_tracker_average() {
        let mut tracker = EglTracker::new(10, -0.05);
        tracker.track(0.6);
        tracker.track(0.7);
        tracker.track(0.8);
        let avg = tracker.average();
        assert!((avg - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_egl_tracker_average_empty() {
        let tracker = EglTracker::new(5, -0.05);
        assert_eq!(tracker.average(), 0.0);
    }

    #[test]
    fn test_egl_tracker_window_eviction() {
        let mut tracker = EglTracker::new(3, -0.05);
        tracker.track(0.1);
        tracker.track(0.2);
        tracker.track(0.3);
        assert_eq!(tracker.history.len(), 3);
        tracker.track(0.4);
        assert_eq!(tracker.history.len(), 3);
    }

    #[test]
    fn test_egl_not_converged_insufficient_data() {
        let tracker = EglTracker::new(10, -0.05);
        assert!(!tracker.is_converged(5, 0.01));
    }

    #[test]
    fn test_egl_converged() {
        let mut tracker = EglTracker::new(10, -0.05);
        for _ in 0..5 {
            tracker.track(0.75);
        }
        assert!(tracker.is_converged(3, 0.01));
    }

    #[test]
    fn test_egl_not_converged_still_changing() {
        let mut tracker = EglTracker::new(10, -0.05);
        tracker.track(0.7);
        tracker.track(0.75);
        tracker.track(0.72);
        assert!(!tracker.is_converged(3, 0.01));
    }

    #[test]
    fn test_egl_status_accessors() {
        let status = EglStatus::Improving { avg: 0.5, current: 0.7, diff: 0.2 };
        assert!((status.avg() - 0.5).abs() < 1e-6);
        assert!((status.current() - 0.7).abs() < 1e-6);
        assert!((status.diff() - 0.2).abs() < 1e-6);

        let reg = EglStatus::Regressing { avg: 0.7, current: 0.5, diff: -0.2 };
        assert!(reg.is_regressing());
        assert!(!reg.is_improving());
    }

    #[test]
    fn test_egl_tracker_reset() {
        let mut tracker = EglTracker::new(5, -0.05);
        tracker.track(0.5);
        tracker.track(0.6);
        assert_eq!(tracker.iteration, 2);
        tracker.reset();
        assert!(tracker.history.is_empty());
        assert_eq!(tracker.iteration, 0);
    }
}
