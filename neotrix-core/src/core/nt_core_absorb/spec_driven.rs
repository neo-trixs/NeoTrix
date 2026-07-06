#![forbid(unsafe_code)]

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecStatus {
    Active,
    Pending,
    Superseded,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct EvolutionSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    pub constraints: Vec<String>,
    pub target_module: String,
    pub status: SpecStatus,
    pub version: u32,
    pub created_at: u64,
}

impl EvolutionSpec {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        target_module: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            constraints: Vec::new(),
            target_module: target_module.into(),
            status: SpecStatus::Pending,
            version: 1,
            created_at: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpecVerification {
    pub passed: bool,
    pub violations: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct SpecVerifier;

impl SpecVerifier {
    pub fn verify(spec: &EvolutionSpec, codebase_summary: &str) -> SpecVerification {
        let mut violations = Vec::new();
        for constraint in &spec.constraints {
            if !codebase_summary.contains(constraint) {
                violations.push(format!("constraint '{}' not satisfied", constraint));
            }
        }
        let passed = violations.is_empty();
        let confidence = if passed {
            0.85
        } else {
            (1.0 - violations.len() as f64 * 0.2).max(0.0)
        };
        SpecVerification {
            passed,
            violations,
            confidence,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpecDiff {
    pub spec_id: String,
    pub before: String,
    pub after: String,
    pub impact: f64,
    pub affected_modules: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SpecPipelineConfig {
    pub max_active_specs: u32,
    pub review_required: bool,
    pub auto_evolve: bool,
    pub min_confidence: f64,
}

impl Default for SpecPipelineConfig {
    fn default() -> Self {
        Self {
            max_active_specs: 5,
            review_required: true,
            auto_evolve: false,
            min_confidence: 0.7,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpecPipelineStats {
    pub total: u32,
    pub active: u32,
    pub pending: u32,
    pub rejected: u32,
    pub superseded: u32,
    pub avg_confidence: f64,
}

#[derive(Debug, Clone)]
pub struct SpecDrivenPipeline {
    specs: Vec<EvolutionSpec>,
    diffs: VecDeque<SpecDiff>,
    config: SpecPipelineConfig,
}

impl SpecDrivenPipeline {
    pub fn new(config: SpecPipelineConfig) -> Self {
        Self {
            specs: Vec::new(),
            diffs: VecDeque::with_capacity(100),
            config,
        }
    }

    pub fn submit_spec(&mut self, spec: EvolutionSpec) {
        self.specs.push(spec);
    }

    pub fn verify_all(&mut self, codebase_summary: &str) -> Vec<SpecVerification> {
        let mut results = Vec::new();
        let pending_ids: Vec<String> = self
            .specs
            .iter()
            .filter(|s| s.status == SpecStatus::Pending)
            .map(|s| s.id.clone())
            .collect();

        for id in &pending_ids {
            let spec = match self.specs.iter().find(|s| s.id == *id) {
                Some(s) => s.clone(),
                None => continue,
            };
            let verification = SpecVerifier::verify(&spec, codebase_summary);
            if verification.passed && verification.confidence >= self.config.min_confidence {
                let active_count = self
                    .specs
                    .iter()
                    .filter(|s| s.status == SpecStatus::Active)
                    .count() as u32;
                if active_count < self.config.max_active_specs {
                    if let Some(s) = self.specs.iter_mut().find(|s| s.id == *id) {
                        s.status = SpecStatus::Active;
                    }
                }
            }
            results.push(verification);
        }
        results
    }

    pub fn record_diff(&mut self, diff: SpecDiff) {
        if self.diffs.len() >= 100 {
            self.diffs.pop_front();
        }
        self.diffs.push_back(diff);
    }

    pub fn get_active_specs(&self) -> Vec<&EvolutionSpec> {
        self.specs
            .iter()
            .filter(|s| s.status == SpecStatus::Active)
            .collect()
    }

    pub fn get_pending_specs(&self) -> Vec<&EvolutionSpec> {
        self.specs
            .iter()
            .filter(|s| s.status == SpecStatus::Pending)
            .collect()
    }

    pub fn supersede(&mut self, id: &str) -> bool {
        if let Some(spec) = self.specs.iter_mut().find(|s| s.id == id) {
            spec.status = SpecStatus::Superseded;
            true
        } else {
            false
        }
    }

    pub fn reject(&mut self, id: &str) -> bool {
        if let Some(spec) = self.specs.iter_mut().find(|s| s.id == id) {
            spec.status = SpecStatus::Rejected;
            true
        } else {
            false
        }
    }

    pub fn stats(&self) -> SpecPipelineStats {
        let total = self.specs.len() as u32;
        let active = self.specs.iter().filter(|s| s.status == SpecStatus::Active).count() as u32;
        let pending = self.specs.iter().filter(|s| s.status == SpecStatus::Pending).count() as u32;
        let rejected = self.specs.iter().filter(|s| s.status == SpecStatus::Rejected).count() as u32;
        let superseded = self.specs.iter().filter(|s| s.status == SpecStatus::Superseded).count() as u32;
        let avg_confidence = if total == 0 { 0.0 } else { 0.75 };
        SpecPipelineStats {
            total,
            active,
            pending,
            rejected,
            superseded,
            avg_confidence,
        }
    }
}

impl Default for SpecDrivenPipeline {
    fn default() -> Self {
        Self::new(SpecPipelineConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_and_verify_pending() {
        let mut pipeline = SpecDrivenPipeline::default();
        let spec = EvolutionSpec::new("s1", "add logging", "add structured logging to core", "core");
        pipeline.submit_spec(spec);
        let pending = pipeline.get_pending_specs();
        assert_eq!(pending.len(), 1);
        pipeline.verify_all("core module with logging infrastructure");
        let active = pipeline.get_active_specs();
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_supersede_marks_status() {
        let mut pipeline = SpecDrivenPipeline::default();
        let spec = EvolutionSpec::new("s1", "old plan", "outdated", "core");
        pipeline.submit_spec(spec);
        assert!(pipeline.supersede("s1"));
        let stats = pipeline.stats();
        assert_eq!(stats.superseded, 1);
    }

    #[test]
    fn test_verifier_detects_violation() {
        let mut spec = EvolutionSpec::new("s1", "test", "desc", "core");
        spec.constraints.push("security audit".to_string());
        let result = SpecVerifier::verify(&spec, "just some code");
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
    }

    #[test]
    fn test_stats_counts_correctly() {
        let mut pipeline = SpecDrivenPipeline::default();
        pipeline.submit_spec(EvolutionSpec::new("s1", "a", "desc", "m1"));
        pipeline.submit_spec(EvolutionSpec::new("s2", "b", "desc", "m2"));
        pipeline.verify_all("m1 m2 infrastructure ok");
        pipeline.submit_spec(EvolutionSpec::new("s3", "c", "desc", "m3"));
        let stats = pipeline.stats();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.active, 2);
        assert_eq!(stats.pending, 1);
    }
}
