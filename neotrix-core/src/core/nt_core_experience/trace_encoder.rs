use std::collections::VecDeque;

/// A point-in-time calibration snapshot.
#[derive(Debug, Clone)]
pub struct CalibrationSnapshot {
    pub cycle: u64,
    pub ece: f64,
    pub meta_accuracy: f64,
    pub composite_loss: f64,
    pub neuromodulator_arousal: f64,
}

/// A discrete intervention event recorded during self-evolution.
#[derive(Debug, Clone)]
pub struct InterventionEvent {
    pub cycle: u64,
    pub kind: String,
    pub description: String,
    pub success: bool,
}

/// A structured trace combining snapshots and events over a time window,
/// with linear-trend estimates for ECE and meta-accuracy.
#[derive(Debug, Clone)]
pub struct StructuredTrace {
    pub snapshot: CalibrationSnapshot,
    pub events: Vec<InterventionEvent>,
    pub window_start: u64,
    pub window_end: u64,
    pub trend_ece: f64,
    pub trend_meta: f64,
}

#[derive(Debug, Clone)]
pub struct TraceEncoderConfig {
    pub max_window: usize,
}

impl Default for TraceEncoderConfig {
    fn default() -> Self {
        Self { max_window: 50 }
    }
}

/// Encoder that converts raw calibration snapshots and intervention logs
/// into structured traces for GEPA-style diagnosis.
#[derive(Debug, Clone)]
pub struct TraceEncoder {
    buffer: VecDeque<CalibrationSnapshot>,
    event_log: VecDeque<InterventionEvent>,
    max_window: usize,
}

impl TraceEncoder {
    pub fn new() -> Self {
        Self::with_config(TraceEncoderConfig::default())
    }

    pub fn with_config(config: TraceEncoderConfig) -> Self {
        Self {
            buffer: VecDeque::with_capacity(config.max_window),
            event_log: VecDeque::with_capacity(config.max_window),
            max_window: config.max_window,
        }
    }

    /// Record a calibration snapshot, trimming to max_window.
    pub fn record_snapshot(
        &mut self,
        cycle: u64,
        ece: f64,
        meta_accuracy: f64,
        composite_loss: f64,
        neuromodulator_arousal: f64,
    ) {
        if self.buffer.len() >= self.max_window {
            self.buffer.pop_front();
        }
        self.buffer.push_back(CalibrationSnapshot {
            cycle,
            ece,
            meta_accuracy,
            composite_loss,
            neuromodulator_arousal,
        });
    }

    /// Record an intervention event, trimming to max_window.
    pub fn record_event(&mut self, cycle: u64, kind: String, description: String, success: bool) {
        if self.event_log.len() >= self.max_window {
            self.event_log.pop_front();
        }
        self.event_log.push_back(InterventionEvent {
            cycle,
            kind,
            description,
            success,
        });
    }

    /// Build a structured trace over the most recent snapshots and events
    /// within the given window size.
    ///
    /// Returns `None` when no snapshots are available.
    /// Trend is computed as the slope over the window:
    ///   (last_value - first_value) / window_size
    pub fn encode_window(&self, cycle: u64, window_size: usize) -> Option<StructuredTrace> {
        if self.buffer.is_empty() {
            return None;
        }

        let actual_window = window_size.min(self.max_window);

        // Collect snapshots within the window
        let threshold = cycle.saturating_sub(actual_window as u64);
        let snapshots: Vec<&CalibrationSnapshot> = self
            .buffer
            .iter()
            .filter(|s| s.cycle >= threshold && s.cycle <= cycle)
            .collect();

        if snapshots.is_empty() {
            return None;
        }

        // Collect events within the same window
        let events: Vec<InterventionEvent> = self
            .event_log
            .iter()
            .filter(|e| e.cycle >= threshold && e.cycle <= cycle)
            .cloned()
            .collect();

        // Compute trends as simple slopes
        let n = actual_window as f64;
        let trend_ece = if snapshots.len() >= 2 {
            (snapshots.last().unwrap().ece - snapshots.first().unwrap().ece) / n
        } else {
            0.0
        };
        let trend_meta = if snapshots.len() >= 2 {
            (snapshots.last().unwrap().meta_accuracy - snapshots.first().unwrap().meta_accuracy) / n
        } else {
            0.0
        };

        Some(StructuredTrace {
            snapshot: (*snapshots.last().unwrap()).clone(),
            events,
            window_start: threshold,
            window_end: cycle,
            trend_ece,
            trend_meta,
        })
    }

    /// Clear all recorded snapshots and events.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.event_log.clear();
    }

    /// Human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "TraceEncoder: snapshots={} events={} window={}",
            self.buffer.len(),
            self.event_log.len(),
            self.max_window,
        )
    }

    // -- accessors for testing --

    pub fn snapshot_count(&self) -> usize {
        self.buffer.len()
    }

    pub fn event_count(&self) -> usize {
        self.event_log.len()
    }
}

impl Default for TraceEncoder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_snapshot() {
        let mut enc = TraceEncoder::new();
        enc.record_snapshot(1, 0.10, 0.90, 0.30, 0.60);
        enc.record_snapshot(2, 0.12, 0.88, 0.32, 0.62);
        enc.record_snapshot(3, 0.15, 0.85, 0.35, 0.64);
        assert_eq!(enc.snapshot_count(), 3);
    }

    #[test]
    fn test_record_event() {
        let mut enc = TraceEncoder::new();
        enc.record_event(1, "calibration".into(), "ECE spike detected".into(), true);
        enc.record_event(2, "mutation".into(), "Tuned learning rate".into(), true);
        enc.record_event(3, "mutation".into(), "Rolled back change".into(), false);
        assert_eq!(enc.event_count(), 3);
    }

    #[test]
    fn test_encode_window_empty() {
        let enc = TraceEncoder::new();
        assert!(enc.encode_window(10, 10).is_none());
    }

    #[test]
    fn test_encode_window_basic() {
        let mut enc = TraceEncoder::new();
        for i in 0..5 {
            let c = i as u64;
            enc.record_snapshot(
                c,
                0.10 + c as f64 * 0.02,
                0.90 - c as f64 * 0.01,
                0.30,
                0.60,
            );
        }
        enc.record_event(1, "calibration".into(), "spike".into(), true);
        enc.record_event(2, "mutation".into(), "tune".into(), true);
        enc.record_event(3, "mutation".into(), "rollback".into(), false);

        let trace = enc.encode_window(5, 10);
        assert!(trace.is_some());
        let t = trace.unwrap();
        assert_eq!(t.snapshot.cycle, 4);
        assert_eq!(t.events.len(), 3);
        // trend_ece = (0.18 - 0.10) / 10.0 = 0.008
        assert!((t.trend_ece - 0.008).abs() < 1e-10);
        // trend_meta = (0.85 - 0.90) / 10.0 = -0.005
        assert!((t.trend_meta - (-0.005)).abs() < 1e-10);
    }

    #[test]
    fn test_encode_window_trend() {
        let mut enc = TraceEncoder::new();
        for i in 0..5 {
            let c = i as u64;
            enc.record_snapshot(c, 0.10 + c as f64 * 0.10, 0.90, 0.30, 0.60);
        }
        let trace = enc.encode_window(5, 10);
        assert!(trace.is_some());
        assert!(trace.unwrap().trend_ece > 0.0);
    }

    #[test]
    fn test_encode_window_trend_negative() {
        let mut enc = TraceEncoder::new();
        for i in 0..5 {
            let c = i as u64;
            enc.record_snapshot(c, 0.50 - c as f64 * 0.10, 0.90, 0.30, 0.60);
        }
        let trace = enc.encode_window(5, 10);
        assert!(trace.is_some());
        assert!(trace.unwrap().trend_ece < 0.0);
    }

    #[test]
    fn test_clear() {
        let mut enc = TraceEncoder::new();
        enc.record_snapshot(1, 0.1, 0.9, 0.3, 0.6);
        enc.record_event(1, "test".into(), "desc".into(), true);
        assert_eq!(enc.snapshot_count(), 1);
        assert_eq!(enc.event_count(), 1);
        enc.clear();
        assert_eq!(enc.snapshot_count(), 0);
        assert_eq!(enc.event_count(), 0);
    }

    #[test]
    fn test_summary() {
        let mut enc = TraceEncoder::new();
        enc.record_snapshot(1, 0.1, 0.9, 0.3, 0.6);
        enc.record_snapshot(2, 0.2, 0.8, 0.4, 0.7);
        enc.record_event(1, "test".into(), "event".into(), true);
        let s = enc.summary();
        assert!(s.contains("TraceEncoder:"));
        assert!(s.contains("snapshots=2"));
        assert!(s.contains("events=1"));
        assert!(s.contains("window=50"));
    }

    #[test]
    fn test_max_window_trim() {
        let mut enc = TraceEncoder::with_config(TraceEncoderConfig { max_window: 3 });
        for i in 0..5 {
            let c = i as u64;
            enc.record_snapshot(c, 0.1, 0.9, 0.3, 0.6);
            enc.record_event(c, "ev".into(), "desc".into(), true);
        }
        assert_eq!(enc.snapshot_count(), 3);
        assert_eq!(enc.event_count(), 3);
        // The oldest two should have been trimmed; the remaining are cycles 2,3,4
        let trace = enc.encode_window(4, 5);
        assert!(trace.is_some());
        assert_eq!(trace.unwrap().snapshot.cycle, 4);
    }
}
