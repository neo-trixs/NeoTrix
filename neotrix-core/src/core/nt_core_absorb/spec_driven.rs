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

    /// 增量合并提交 (dot-skill append→delta→merge 吸收):
    /// 同 id 已有规格时, 不静默覆盖 — 保留既有结论作为基线, 新提交作为 delta
    /// 追加合并 (描述/约束增量并集, 版本递增, 旧版本标记 Superseded 留证据链)。
    /// 返回 (合并结果, 是否新建)。新建返回 (Some(id), true); 已存在并合并返回
    /// (Some(id), false); 目标已被 Rejected 时返回 Err (拒绝复活, 须显式重建)。
    pub fn submit_spec_incremental(&mut self, spec: EvolutionSpec) -> Result<(String, bool), String> {
        if spec.id.is_empty() {
            return Err("spec id must not be empty".to_string());
        }
        if let Some(existing) = self.specs.iter().find(|s| s.id == spec.id) {
            if existing.status == SpecStatus::Rejected {
                return Err(format!(
                    "spec '{}' is Rejected (explicitly closed); use a new id to reopen",
                    spec.id
                ));
            }
            // 提取既有基线 (克隆, 避免借用冲突)
            let existing_base = existing.clone();
            // 旧版本归档为 Superseded (证据链保留)。
            if let Some(prev) = self.specs.iter_mut().find(|s| s.id == spec.id) {
                prev.status = SpecStatus::Superseded;
            }
            // delta 合并: 约束取并集 (既有 + 新增), 保留旧版创建时间。
            let mut merged_constraints = existing_base.constraints.clone();
            for c in &spec.constraints {
                if !merged_constraints.contains(c) {
                    merged_constraints.push(c.clone());
                }
            }
            let mut merged = spec.clone();
            merged.constraints = merged_constraints;
            merged.status = SpecStatus::Pending; // 需要重新验证
            merged.version = existing_base.version + 1;
            merged.created_at = existing_base.created_at;
            let id = merged.id.clone();
            self.specs.push(merged);
            return Ok((id, false));
        }
        let id = spec.id.clone();
        self.specs.push(spec);
        Ok((id, true))
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

    #[test]
    fn test_incremental_merge_keeps_baseline_and_delta() {
        // 增量合并 (dot-skill append→delta→merge 吸收): 同 id 第二次提交不覆盖
        // 既有结论, 而是保留旧版本为 Superseded + 新版本约束并集。
        let mut pipeline = SpecDrivenPipeline::default();
        let mut spec1 = EvolutionSpec::new("evo-1", "absorb", "baseline", "core");
        spec1.constraints.push("compiles".to_string());
        let (id, created) = pipeline.submit_spec_incremental(spec1).unwrap();
        assert_eq!(id, "evo-1");
        assert!(created);

        // 第二次提交: delta 合并
        let mut spec2 = EvolutionSpec::new("evo-1", "absorb", "updated delta", "core");
        spec2.constraints.push("tests pass".to_string());
        let (id2, created2) = pipeline.submit_spec_incremental(spec2).unwrap();
        assert_eq!(id2, "evo-1");
        assert!(!created2, "existing spec merges, not new");

        // 旧版本归档 Superseded (证据链)
        assert_eq!(pipeline.specs.iter().filter(|s| s.status == SpecStatus::Superseded).count(), 1);
        // 最新版本约束 = 并集 (compiles + tests pass)
        let latest = pipeline.specs.iter().filter(|s| s.status == SpecStatus::Pending).last().unwrap();
        assert_eq!(latest.version, 2);
        assert!(latest.constraints.contains(&"compiles".to_string()));
        assert!(latest.constraints.contains(&"tests pass".to_string()));
    }

    #[test]
    fn test_incremental_rejected_cannot_reopen() {
        let mut pipeline = SpecDrivenPipeline::default();
        let spec = EvolutionSpec::new("dead", "x", "desc", "m");
        pipeline.submit_spec(spec);
        assert!(pipeline.reject("dead"));
        let reopen = EvolutionSpec::new("dead", "y", "reopen attempt", "m");
        let err = pipeline.submit_spec_incremental(reopen).unwrap_err();
        assert!(err.contains("Rejected"), "rejected spec cannot reopen via merge");
    }

    #[test]
    fn test_incremental_empty_id_rejected() {
        let mut pipeline = SpecDrivenPipeline::default();
        let spec = EvolutionSpec::new("", "x", "desc", "m");
        assert!(pipeline.submit_spec_incremental(spec).is_err());
    }
}
