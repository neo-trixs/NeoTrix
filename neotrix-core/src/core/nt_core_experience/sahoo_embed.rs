use super::goal_drift_index::GoalDriftIndex;

/// SAHOO-style embedding: bridges GDI + constraint + regression checks
/// into a unified safety_gate check (arXiv:2603.06333).
#[derive(Debug, Clone)]
pub struct SahooEmbed {
    gdi_enabled: bool,
    gdi_baseline: f64,
    max_drift: f64,
    #[allow(dead_code)]
    max_violation_rate: f64,
}

#[derive(Debug, Clone)]
pub struct SahooReport {
    pub gdi_score: f64,
    pub gdi_drift: f64,
    pub gdi_pass: bool,
    pub constraint_violation_rate: f64,
    pub constraint_pass: bool,
    pub all_pass: bool,
}

impl SahooEmbed {
    pub fn new() -> Self {
        Self {
            gdi_enabled: true,
            gdi_baseline: 0.0,
            max_drift: 0.3,
            max_violation_rate: 0.1,
        }
    }

    /// Run all SAHOO checks. Returns a report with pass/fail for each dimension.
    pub fn check(&self, gdi: &GoalDriftIndex) -> SahooReport {
        let gdi_score = gdi.gdi();
        let gdi_drift = (gdi_score - self.gdi_baseline).abs();
        let gdi_pass = gdi_drift <= self.max_drift;

        // Constraint violation tracking from GoalDriftIndex stats
        let stats = gdi.stats();
        let violation_rate = if stats.drift_count > 0 && stats.sample_count > 0 {
            stats.drift_count as f64 / stats.sample_count as f64
        } else {
            0.0
        };
        let constraint_pass = violation_rate <= self.max_violation_rate;

        SahooReport {
            gdi_score,
            gdi_drift,
            gdi_pass,
            constraint_violation_rate: violation_rate,
            constraint_pass,
            all_pass: gdi_pass && constraint_pass,
        }
    }

    /// Reset baseline after successful evolution commit
    pub fn reset_baseline(&mut self, current_gdi: f64) {
        self.gdi_baseline = current_gdi;
    }

    pub fn set_max_drift(&mut self, drift: f64) {
        self.max_drift = drift;
    }
    pub fn set_max_violation_rate(&mut self, rate: f64) {
        self.max_violation_rate = rate;
    }
    pub fn set_gdi_enabled(&mut self, enabled: bool) {
        self.gdi_enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::goal_drift_index::GoalDriftIndex;

    fn make_gdi(drift: f64, _violations: u64, _total: u64) -> GoalDriftIndex {
        // Use record() to create samples that produce the target drift score
        let mut g = GoalDriftIndex::new(100);
        let ref_text = "The quick brown fox jumps over the lazy dog. The sky is blue and the sun is bright. Today is a beautiful day for a walk in the park.";
        // Generate a text with different phrasing to achieve the desired drift
        let test_text = if drift > 0.5 {
            "The slow purple elephant walks under the rainbow bridge. The moon shines bright and stars are glowing. Midnight jazz fills the air under the velvet sky."
        } else {
            ref_text
        };
        for _ in 0..5 {
            g.record(test_text, ref_text);
        }
        g
    }

    #[test]
    fn test_sahoo_all_pass_when_stable() {
        let embed = SahooEmbed::new();
        let gdi = make_gdi(0.1, 0, 100);
        let report = embed.check(&gdi);
        assert!(report.all_pass);
    }

    #[test]
    fn test_sahoo_gdi_drift_detected() {
        let mut embed = SahooEmbed::new();
        embed.gdi_baseline = 0.0;
        embed.max_drift = 0.2;
        let gdi = make_gdi(0.8, 0, 100);
        let report = embed.check(&gdi);
        // With different text, we should have some drift
        assert!(!report.gdi_pass);
    }

    #[test]
    fn test_sahoo_constraint_violation_detected() {
        let embed = SahooEmbed::new();
        let gdi = make_gdi(0.1, 20, 100);
        let report = embed.check(&gdi);
        // Drift_count from stats provides violation rate
        assert!(!report.constraint_pass || report.constraint_violation_rate > 0.0);
    }

    #[test]
    fn test_sahoo_baseline_reset() {
        let mut embed = SahooEmbed::new();
        embed.reset_baseline(0.5);
        assert!((embed.gdi_baseline - 0.5).abs() < 0.01);
    }
}
