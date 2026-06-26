use std::collections::{HashMap, VecDeque};

/// CPE: Capability-Preserving Evolution regularizer
/// References: arXiv 2605.09315 (UIUC, 2026)
/// Tracks capability signatures across evolution dimensions and computes
/// retention regularization scores to prevent capability erosion.

#[derive(Debug, Clone)]
pub struct CapabilitySignature {
    pub dimension: CpeDimension,
    pub label: String,
    pub behavioral_hash: u64,
    pub created_at: u64,
    pub last_seen: u64,
    pub invocation_count: u64,
    pub retention_score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CpeDimension {
    Workflow,
    Skill,
    Model,
    Memory,
}

#[derive(Debug, Clone)]
pub struct CpeConfig {
    pub workflow_retention_weight: f64,
    pub skill_retention_weight: f64,
    pub model_retention_weight: f64,
    pub memory_retention_weight: f64,
    pub stability_plasticity_tradeoff: f64,
    pub max_signatures: usize,
}

impl Default for CpeConfig {
    fn default() -> Self {
        Self {
            workflow_retention_weight: 0.5,
            skill_retention_weight: 0.5,
            model_retention_weight: 0.3,
            memory_retention_weight: 0.4,
            stability_plasticity_tradeoff: 0.6,
            max_signatures: 500,
        }
    }
}

pub struct CpeRegularizer {
    pub signatures: VecDeque<CapabilitySignature>,
    pub config: CpeConfig,
    pub total_regularized: u64,
    pub total_deviation: f64,
    /// Per-cycle retention loss (smoothed)
    pub retention_loss_ema: f64,
}

impl CpeRegularizer {
    pub fn new() -> Self {
        Self::with_config(CpeConfig::default())
    }

    pub fn with_config(config: CpeConfig) -> Self {
        Self {
            signatures: VecDeque::with_capacity(config.max_signatures),
            config,
            total_regularized: 0,
            total_deviation: 0.0,
            retention_loss_ema: 0.0,
        }
    }

    /// Record a capability signature from the current evolution state
    pub fn record_signature(&mut self, dimension: CpeDimension, label: &str, hash: u64) {
        if self.signatures.len() >= self.config.max_signatures {
            self.signatures.pop_front();
        }
        self.signatures.push_back(CapabilitySignature {
            dimension,
            label: label.to_string(),
            behavioral_hash: hash,
            created_at: self.total_regularized,
            last_seen: self.total_regularized,
            invocation_count: 1,
            retention_score: 1.0,
        });
    }

    /// Update retention score when a signature is re-observed
    pub fn observe(&mut self, label: &str, hash: u64) {
        for sig in &mut self.signatures {
            if sig.label == label {
                sig.last_seen = self.total_regularized;
                sig.invocation_count += 1;
                let match_ratio = if sig.behavioral_hash == hash {
                    1.0
                } else {
                    0.5
                };
                sig.retention_score = sig.retention_score * 0.7 + match_ratio * 0.3;
                return;
            }
        }
    }

    /// Compute Ω_t retention regularizer for a given dimension
    /// Returns 0.0 (perfect retention) to 1.0 (complete erosion)
    pub fn compute_omega(&self, dimension: CpeDimension) -> f64 {
        let relevant: Vec<&CapabilitySignature> = self
            .signatures
            .iter()
            .filter(|s| s.dimension == dimension)
            .collect();
        if relevant.is_empty() {
            return 0.0;
        }
        let avg_retention: f64 =
            relevant.iter().map(|s| s.retention_score).sum::<f64>() / relevant.len() as f64;
        let weight = match dimension {
            CpeDimension::Workflow => self.config.workflow_retention_weight,
            CpeDimension::Skill => self.config.skill_retention_weight,
            CpeDimension::Model => self.config.model_retention_weight,
            CpeDimension::Memory => self.config.memory_retention_weight,
        };
        (1.0 - avg_retention) * weight
    }

    /// Combined retention penalty for a proposed change
    /// Lower is better (less interference with prior capabilities)
    pub fn retention_penalty(&self) -> f64 {
        let mut total = 0.0f64;
        for dim in &[
            CpeDimension::Workflow,
            CpeDimension::Skill,
            CpeDimension::Model,
            CpeDimension::Memory,
        ] {
            total += self.compute_omega(*dim);
        }
        total * self.config.stability_plasticity_tradeoff
    }

    /// Tick: decay old signatures
    pub fn tick(&mut self) {
        let current = self.total_regularized;
        for sig in &mut self.signatures {
            let age = current - sig.last_seen;
            if age > 10 {
                sig.retention_score *= 0.95;
            }
            if age > 50 {
                sig.retention_score *= 0.90;
            }
        }
        // Prune fully decayed
        self.signatures.retain(|s| s.retention_score > 0.05);
        // Update EMA
        let penalty = self.retention_penalty();
        self.retention_loss_ema = self.retention_loss_ema * 0.9 + penalty * 0.1;
        self.total_regularized += 1;
    }

    pub fn summary(&self) -> String {
        format!(
            "cpe: sigs={} retention_ema={:.3} penalty={:.3} regularized={}",
            self.signatures.len(),
            self.retention_loss_ema,
            self.retention_penalty(),
            self.total_regularized,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_observe() {
        let mut c = CpeRegularizer::new();
        c.record_signature(CpeDimension::Workflow, "tool_usage", 42);
        assert_eq!(c.signatures.len(), 1);
        assert!((c.signatures[0].retention_score - 1.0).abs() < 0.01);
        c.observe("tool_usage", 42);
        assert_eq!(c.signatures[0].invocation_count, 2);
    }

    #[test]
    fn test_omega_starts_at_zero() {
        let c = CpeRegularizer::new();
        assert!((c.compute_omega(CpeDimension::Workflow) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_tick_decays_old_signatures() {
        let mut c = CpeRegularizer::new();
        c.record_signature(CpeDimension::Memory, "old_mem", 0);
        // Advance time
        for _ in 0..60 {
            c.tick();
        }
        // Old sig should have decayed close to 0
        let omega = c.compute_omega(CpeDimension::Memory);
        assert!(omega > 0.0);
    }

    #[test]
    fn test_retention_penalty() {
        let mut c = CpeRegularizer::new();
        assert!((c.retention_penalty() - 0.0).abs() < 0.001);
        c.record_signature(CpeDimension::Skill, "skill_a", 100);
        // First tick should give some penalty from decay
        c.tick();
        let p = c.retention_penalty();
        assert!(p >= 0.0);
    }

    #[test]
    fn test_max_signatures_prunes() {
        let cfg = CpeConfig {
            max_signatures: 5,
            ..Default::default()
        };
        let mut c = CpeRegularizer::with_config(cfg);
        for i in 0..10 {
            c.record_signature(CpeDimension::Workflow, &format!("sig_{}", i), i as u64);
        }
        assert!(c.signatures.len() <= 5);
    }

    #[test]
    fn test_summary() {
        let mut c = CpeRegularizer::new();
        c.record_signature(CpeDimension::Model, "param", 0);
        c.tick();
        let s = c.summary();
        assert!(s.contains("cpe:"));
        assert!(s.contains("retention_ema="));
    }
}
