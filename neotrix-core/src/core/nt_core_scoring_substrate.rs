use crate::core::nt_core_self_test::{SelfTest, SelfTestResult};

pub struct ScoringSubstrate {
    threshold: f64,
}

impl ScoringSubstrate {
    pub fn new() -> Self {
        Self { threshold: 0.5 }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn threshold(&self) -> f64 {
        self.threshold
    }
}

impl Default for ScoringSubstrate {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfTest for ScoringSubstrate {
    fn name(&self) -> &'static str {
        "ScoringSubstrate"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        if self.threshold <= 0.0 || self.threshold > 1.0 {
            return Err(vec![format!("invalid threshold: {}", self.threshold)]);
        }
        Ok(())
    }
}
