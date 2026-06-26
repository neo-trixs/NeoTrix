use super::ConsciousnessIntegration;

impl ConsciousnessIntegration {
    /// AVSAD tick: run adversarial visual detection on recent images in sensory buffer.
    /// Frequency: cycle%10 in core.rs.
    pub fn handle_avsad_tick(&mut self) -> String {
        let detector = match self.avsad.as_mut() {
            Some(d) => d,
            None => return "avsad:unwired".to_string(),
        };

        let vision = match self.vision.as_ref() {
            Some(v) => v,
            None => return "avsad:no_vision".to_string(),
        };

        let available = vision.is_available();
        let stats = format!(
            "avsad:checks={}_flagged={}_rate={:.4}_vision={}",
            detector.total_checks,
            detector.flagged_count,
            detector.detection_rate(),
            if available { "ready" } else { "unavailable" }
        );

        // Only run actual analysis when vision has recent input
        if !available {
            return stats;
        }

        let recent_count = self.vsa_buffer.len().min(5);
        let mut flagged = 0u64;
        for i in 0..recent_count {
            if let Some(vsa) = self.vsa_buffer.back().map(|b| b.as_slice()) {
                let (_score, is_adv, _details) = detector.analyze(vsa);
                if is_adv {
                    flagged += 1;
                }
            }
            // Simulate reading from vsa_buffer at different positions
            if i > 0 && self.vsa_buffer.len() > i {
                let idx = self.vsa_buffer.len() - 1 - i;
                if let Some(vsa) = self.vsa_buffer.get(idx) {
                    let (_score, is_adv, _details) = detector.analyze(vsa);
                    if is_adv {
                        flagged += 1;
                    }
                }
            }
        }

        if flagged > 0 {
            log::warn!(
                "[avsad] flagged {} adversarial samples in recent buffer (checks={})",
                flagged,
                detector.total_checks
            );
        }

        format!("{}|flagged_samples={}", stats, flagged)
    }

    /// Reset AVSAD detector statistics.
    pub fn handle_avsad_reset_tick(&mut self) -> String {
        match self.avsad.as_mut() {
            Some(d) => {
                d.total_checks = 0;
                d.flagged_count = 0;
                d.score_history.clear();
                "avsad:reset".to_string()
            }
            None => "avsad:unwired".to_string(),
        }
    }
}
