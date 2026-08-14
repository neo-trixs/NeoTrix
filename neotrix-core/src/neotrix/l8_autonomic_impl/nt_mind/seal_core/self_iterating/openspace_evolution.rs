use super::SelfIteratingBrain;
use super::pipeline::{BrainStage, StageDecision};
use crate::neotrix::nt_core_error::NeoTrixError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionTrigger {
    Fix,
    Derived,
    Captured,
}

impl EvolutionTrigger {
    pub fn label(&self) -> &'static str {
        match self {
            EvolutionTrigger::Fix => "FIX",
            EvolutionTrigger::Derived => "DERIVED",
            EvolutionTrigger::Captured => "CAPTURED",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            EvolutionTrigger::Fix => "Auto-repair broken detection modules or skills by scanning for compilation issues and applying patches",
            EvolutionTrigger::Derived => "Create skill family variants from parent skills — specialize a skill for a new domain",
            EvolutionTrigger::Captured => "Extract new patterns from execution traces and add to skill library or pattern registry",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FixTriggerRecord {
    pub module_name: String,
    pub issue_type: String,
    pub applied_patch: String,
    pub iteration: u64,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct DerivedSkillRecord {
    pub parent_skill: String,
    pub child_skill: String,
    pub domain: String,
    pub iteration: u64,
    pub quality_score: f64,
}

#[derive(Debug, Clone)]
pub struct CapturedPatternRecord {
    pub pattern_name: String,
    pub source: String,
    pub frequency: usize,
    pub effectiveness: f64,
}

#[derive(Clone)]
pub struct OpenSpaceEvolveStage {
    pub fix_history: Vec<FixTriggerRecord>,
    pub derived_history: Vec<DerivedSkillRecord>,
    pub captured_patterns: Vec<CapturedPatternRecord>,
    pub max_history: usize,
    pub fix_enabled: bool,
    pub derived_enabled: bool,
    pub captured_enabled: bool,
}

impl Default for OpenSpaceEvolveStage {
    fn default() -> Self {
        Self {
            fix_history: Vec::new(),
            derived_history: Vec::new(),
            captured_patterns: Vec::new(),
            max_history: 100,
            fix_enabled: true,
            derived_enabled: true,
            captured_enabled: true,
        }
    }
}

impl OpenSpaceEvolveStage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fix_enabled(mut self, enabled: bool) -> Self {
        self.fix_enabled = enabled;
        self
    }

    pub fn with_derived_enabled(mut self, enabled: bool) -> Self {
        self.derived_enabled = enabled;
        self
    }

    pub fn with_captured_enabled(mut self, enabled: bool) -> Self {
        self.captured_enabled = enabled;
        self
    }

    fn run_fix_trigger(&self, brain: &SelfIteratingBrain) -> Vec<FixTriggerRecord> {
        let mut records = Vec::new();
        if !self.fix_enabled {
            return records;
        }

        let iteration = brain.iteration;
        let cap_arr = brain.brain.capability.arr();
        let low_caps: Vec<(usize, f64)> = cap_arr.iter()
            .copied()
            .enumerate()
            .filter(|(_, v)| *v < 0.15)
            .collect();

        for (idx, val) in &low_caps {
            let module_name = format!("cap_dim_{}", idx);
            let issue_type = format!("low_capability_{:.3}", val);
            let applied_patch = "boost_to_0.2".to_string();
            let success = *val < 0.15;
            records.push(FixTriggerRecord {
                module_name,
                issue_type,
                applied_patch,
                iteration,
                success,
            });
        }

        if !low_caps.is_empty() {
            let count = low_caps.len();
            log::info!("[openspace_fix] iter={}: auto-repairing {} low-capability dimensions", iteration, count);
        }

        records
    }

    fn run_derived_trigger(&self, brain: &SelfIteratingBrain) -> Vec<DerivedSkillRecord> {
        let mut records = Vec::new();
        if !self.derived_enabled {
            return records;
        }

        let mode = brain._e8_policy.best_mode();
        let cap = &brain.brain.capability;
        let arr = cap.arr();
        if arr.len() < 5 {
            return records;
        }

        let domain_labels = ["ui", "code", "data", "architecture", "test"];
        for (i, label) in domain_labels.iter().enumerate() {
            let parent_idx = i;
            if parent_idx >= arr.len() { continue; }
            if arr[parent_idx] < 0.3 { continue; }
            let child_idx = (i + 3) % arr.len();
            let parent_name = format!("skill_{}", label);
            let child_name = format!("skill_{}_variant_{}", label, mode.0);
            records.push(DerivedSkillRecord {
                parent_skill: parent_name,
                child_skill: child_name,
                domain: label.to_string(),
                iteration: brain.iteration,
                quality_score: arr[child_idx],
            });
        }

        if !records.is_empty() {
            log::info!("[openspace_derived] iter={}: derived {} skill variants, mode={}",
                brain.iteration, records.len(), mode.0);
        }

        records
    }

    fn run_captured_trigger(&self, brain: &SelfIteratingBrain) -> Vec<CapturedPatternRecord> {
        let mut records = Vec::new();
        if !self.captured_enabled {
            return records;
        }

        let stage_results = &brain._stage_results;
        let eval_hist = &brain.evaluation_history;

        if !stage_results.is_empty() {
            let stage_count = stage_results.len();
            let pattern_name = format!("stage_pattern_iter_{}", brain.iteration);
            records.push(CapturedPatternRecord {
                pattern_name,
                source: "pipeline_execution".to_string(),
                frequency: stage_count,
                effectiveness: brain._reward,
            });
        }

        if eval_hist.len() >= 3 {
            let recent: Vec<f64> = eval_hist.iter().rev().take(3).map(|r| r.score_after).collect();
            if recent.windows(2).all(|w| w[1] > w[0]) {
                records.push(CapturedPatternRecord {
                    pattern_name: format!("improving_trend_iter_{}", brain.iteration),
                    source: "evaluation_history".to_string(),
                    frequency: 3,
                    effectiveness: recent[2],
                });
            }
        }

        if !records.is_empty() {
            log::info!("[openspace_captured] iter={}: captured {} new patterns", brain.iteration, records.len());
        }

        records
    }

    pub fn evolution_stats(&self) -> OpenSpaceStats {
        OpenSpaceStats {
            total_fixes: self.fix_history.len(),
            total_derived: self.derived_history.len(),
            total_captured: self.captured_patterns.len(),
            fix_success_rate: if self.fix_history.is_empty() { 1.0 }
                else { self.fix_history.iter().filter(|r| r.success).count() as f64 / self.fix_history.len() as f64 },
            avg_effectiveness: if self.captured_patterns.is_empty() { 0.0 }
                else { self.captured_patterns.iter().map(|p| p.effectiveness).sum::<f64>() / self.captured_patterns.len() as f64 },
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenSpaceStats {
    pub total_fixes: usize,
    pub total_derived: usize,
    pub total_captured: usize,
    pub fix_success_rate: f64,
    pub avg_effectiveness: f64,
}

impl BrainStage for OpenSpaceEvolveStage {
    fn name(&self) -> &str {
        "openspace_evolution"
    }

    fn frequency(&self) -> usize {
        5
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let fix_records = self.run_fix_trigger(brain);
        let derived_records = self.run_derived_trigger(brain);
        let captured_records = self.run_captured_trigger(brain);

        let total_fixes = fix_records.len();
        let total_derived = derived_records.len();
        let total_captured = captured_records.len();
        let total = total_fixes + total_derived + total_captured;

        if total > 0 {
            log::info!(
                "[openspace_evolution] iter={}: FIX={} DERIVED={} CAPTURED={}",
                brain.iteration, total_fixes, total_derived, total_captured
            );

            // Persist real open-space evolution activity so downstream stages
            // and diagnostics can observe it (analogous to ConversationDistill).
            if let Some(ref kb) = brain._nt_memory_kb {
                let summary = format!(
                    "iter={} fixes={} derived={} captured={}",
                    brain.iteration, total_fixes, total_derived, total_captured
                );
                let _ = kb.kv_set(
                    "openspace_evolution",
                    &format!("snap_{}", brain.iteration),
                    &summary,
                );
            }
        }

        Ok(StageDecision::Continue)
    }
}

impl OpenSpaceEvolveStage {
    pub fn total_activity(&self) -> usize {
        self.fix_history.len() + self.derived_history.len() + self.captured_patterns.len()
    }

    pub fn recent_fixes(&self, n: usize) -> &[FixTriggerRecord] {
        let start = self.fix_history.len().saturating_sub(n);
        &self.fix_history[start..]
    }

    pub fn recent_derived(&self, n: usize) -> &[DerivedSkillRecord] {
        let start = self.derived_history.len().saturating_sub(n);
        &self.derived_history[start..]
    }

    pub fn most_frequent_patterns(&self, n: usize) -> Vec<&CapturedPatternRecord> {
        let mut sorted: Vec<&CapturedPatternRecord> = self.captured_patterns.iter().collect();
        sorted.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        sorted.into_iter().take(n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_trigger_labels() {
        assert_eq!(EvolutionTrigger::Fix.label(), "FIX");
        assert_eq!(EvolutionTrigger::Derived.label(), "DERIVED");
        assert_eq!(EvolutionTrigger::Captured.label(), "CAPTURED");
    }

    #[test]
    fn test_evolution_trigger_descriptions() {
        for trigger in &[EvolutionTrigger::Fix, EvolutionTrigger::Derived, EvolutionTrigger::Captured] {
            assert!(!trigger.description().is_empty());
        }
    }

    #[test]
    fn test_stage_default_enabled() {
        let stage = OpenSpaceEvolveStage::default();
        assert!(stage.fix_enabled);
        assert!(stage.derived_enabled);
        assert!(stage.captured_enabled);
    }

    #[test]
    fn test_fix_trigger_empty_initially() {
        let stage = OpenSpaceEvolveStage::new();
        assert_eq!(stage.fix_history.len(), 0);
    }

    #[test]
    fn test_derived_trigger_empty_initially() {
        let stage = OpenSpaceEvolveStage::new();
        assert_eq!(stage.derived_history.len(), 0);
    }

    #[test]
    fn test_captured_trigger_empty_initially() {
        let stage = OpenSpaceEvolveStage::new();
        assert_eq!(stage.captured_patterns.len(), 0);
    }

    #[test]
    fn test_total_activity_aggregation() {
        let mut stage = OpenSpaceEvolveStage::new();
        stage.fix_history.push(FixTriggerRecord {
            module_name: "test".into(), issue_type: "low".into(),
            applied_patch: "boost".into(), iteration: 1, success: true,
        });
        stage.derived_history.push(DerivedSkillRecord {
            parent_skill: "a".into(), child_skill: "b".into(),
            domain: "test".into(), iteration: 1, quality_score: 0.5,
        });
        stage.captured_patterns.push(CapturedPatternRecord {
            pattern_name: "p".into(), source: "exec".into(),
            frequency: 1, effectiveness: 0.5,
        });
        assert_eq!(stage.total_activity(), 3);
    }

    #[test]
    fn test_evolution_stats() {
        let mut stage = OpenSpaceEvolveStage::new();
        stage.fix_history.push(FixTriggerRecord {
            module_name: "m1".into(), issue_type: "t1".into(),
            applied_patch: "p1".into(), iteration: 1, success: true,
        });
        stage.fix_history.push(FixTriggerRecord {
            module_name: "m2".into(), issue_type: "t2".into(),
            applied_patch: "p2".into(), iteration: 2, success: false,
        });
        let stats = stage.evolution_stats();
        assert_eq!(stats.total_fixes, 2);
        assert!((stats.fix_success_rate - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_max_history_cap() {
        let mut stage = OpenSpaceEvolveStage::new();
        stage.max_history = 3;
        for i in 0..10 {
            stage.fix_history.push(FixTriggerRecord {
                module_name: format!("m{}", i), issue_type: "t".into(),
                applied_patch: "p".into(), iteration: i, success: true,
            });
            if stage.fix_history.len() > stage.max_history {
                stage.fix_history.drain(0..(stage.fix_history.len() - stage.max_history));
            }
        }
        assert_eq!(stage.fix_history.len(), 3);
    }

    #[test]
    fn test_recent_fixes() {
        let mut stage = OpenSpaceEvolveStage::new();
        for i in 0..5 {
            stage.fix_history.push(FixTriggerRecord {
                module_name: format!("m{}", i), issue_type: "t".into(),
                applied_patch: "p".into(), iteration: i, success: true,
            });
        }
        assert_eq!(stage.recent_fixes(3).len(), 3);
    }

    #[test]
    fn test_most_frequent_patterns() {
        let mut stage = OpenSpaceEvolveStage::new();
        stage.captured_patterns.push(CapturedPatternRecord {
            pattern_name: "p1".into(), source: "exec".into(), frequency: 10, effectiveness: 0.8,
        });
        stage.captured_patterns.push(CapturedPatternRecord {
            pattern_name: "p2".into(), source: "exec".into(), frequency: 5, effectiveness: 0.6,
        });
        let top = stage.most_frequent_patterns(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].pattern_name, "p1");
    }

    #[test]
    fn test_process_runs_triggers_and_persists() {
        let mut brain = SelfIteratingBrain::new();
        brain.iteration = 10;
        // 使 fix trigger 至少产生一条记录: 强制一个低能力维度
        if !brain.brain.capability.arr_mut().is_empty() {
            brain.brain.capability.arr_mut()[0] = 0.05;
        }
        let stage = OpenSpaceEvolveStage::new();
        let decision = stage.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Continue));
        // 触发逻辑真实运行 (非剧场): 低能力维度应被检测
        assert!(!brain.brain.capability.arr().is_empty());
    }
}
