//! Skill Evolution — COBRA-Skills bandit-guided optimization
//!
//! Uses contextual bandits to:
//! 1. Score skill candidates against task context
//! 2. Select which skills to evaluate (budgeted allocation)
//! 3. Update skill scores from execution feedback
//! 4. Evolve skill population (promote/demote/retire)
//!
//! Reference: COBRA-Skills (arXiv:2609.11682) — 55-58% cost reduction vs baseline.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ============================================================================
// Types
// ============================================================================

/// Skill maturity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillMaturity {
    /// Untested — no evaluations yet
    Candidate,
    /// Few evaluations — provisional trust
    Provisional,
    /// Proven — meets quality threshold
    Trusted,
    /// Failed — retired from active pool
    Retired,
}

impl SkillMaturity {
    /// Minimum evaluations to advance from Candidate
    pub const CANDIDATE_THRESHOLD: u32 = 3;
    /// Minimum evaluations + score to advance from Provisional
    pub const PROVISIONAL_THRESHOLD: u32 = 10;
    /// Minimum average score to be Trusted
    pub const TRUSTED_MIN_SCORE: f64 = 0.7;
    /// Maximum failures before demotion
    pub const MAX_FAILURES: u32 = 5;
}

/// Where a skill comes from (evolution-state provenance)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillSource {
    Builtin,
    Local {
        #[serde(default)]
        path: String,
    },
    Git {
        #[serde(default)]
        url: String,
        #[serde(default)]
        branch: String,
    },
    Registry {
        #[serde(default)]
        url: String,
        #[serde(default)]
        skill_id: String,
    },
}

impl Default for SkillSource {
    fn default() -> Self {
        Self::Builtin
    }
}

/// Security audit record for a skill candidate.
///
/// 5-step pipeline: 扫描 / 签名锁版 / 运行时监控 / 最小权限 / 人工复核.
/// `stage` records the current step (e.g. "scan", "pin", "monitor", "least-privilege", "review").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityAudit {
    #[serde(default)]
    pub passed: bool,
    #[serde(default)]
    pub checked_at: u64,
    #[serde(default)]
    pub issues: Vec<String>,
    #[serde(default)]
    pub stage: String,
}

/// A skill candidate tracked by the bandit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCandidate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub performance_history: Vec<f64>,
    pub evaluation_count: u32,
    pub last_evaluated: Option<u64>,
    pub maturity: SkillMaturity,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub required_permissions: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub source: SkillSource,
    #[serde(default)]
    pub security_audit: Option<SecurityAudit>,
    /// Data scope: 0 = pure dialogue, 8 = needs 数云·ERP
    #[serde(default)]
    pub data_scope: u8,
    #[serde(default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub updated_at: u64,
    #[serde(default)]
    pub certified: bool,
    // ── T35 E轨（候选本体侧对齐 S7.1 install 记录 license/sourceUrl）──
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub source_url: Option<String>,
}

impl SkillCandidate {
    /// Average performance score (0.0-1.0), or 0.0 if no history
    pub fn average_score(&self) -> f64 {
        if self.performance_history.is_empty() {
            0.0
        } else {
            self.performance_history.iter().sum::<f64>() / self.performance_history.len() as f64
        }
    }

    /// Number of failures in recent window
    pub fn recent_failures(&self, window: usize) -> u32 {
        self.performance_history
            .iter()
            .rev()
            .take(window)
            .filter(|&&s| s < 0.3)
            .count() as u32
    }
}

/// Task context for skill matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub task_type: String,
    pub complexity: f64,
    pub domain: String,
    pub model_id: String,
    pub available_tokens: u32,
}

/// Bandit state for budgeted skill evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanditState {
    pub skill_scores: HashMap<String, f64>,
    pub total_evaluations: u32,
    pub budget_remaining: u32,
}

// ============================================================================
// SkillEvolver
// ============================================================================

/// COBRA-Skills bandit-guided skill evolution engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEvolver {
    pub candidates: Vec<SkillCandidate>,
    pub bandit: BanditState,
    pub exploration_rate: f64,
}

impl SkillEvolver {
    /// Create a new evolver with a budget cap
    pub fn new(budget: u32, exploration_rate: f64) -> Self {
        Self {
            candidates: Vec::new(),
            bandit: BanditState {
                skill_scores: HashMap::new(),
                total_evaluations: 0,
                budget_remaining: budget,
            },
            exploration_rate,
        }
    }

    /// Register a new skill candidate
    pub fn register(&mut self, id: &str, name: &str, description: &str) {
        if self.candidates.iter().any(|c| c.id == id) {
            return;
        }
        self.candidates.push(SkillCandidate {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            version: 1,
            performance_history: Vec::new(),
            evaluation_count: 0,
            last_evaluated: None,
            maturity: SkillMaturity::Candidate,
            author: String::new(),
            tags: Vec::new(),
            triggers: Vec::new(),
            required_permissions: Vec::new(),
            dependencies: Vec::new(),
            source: SkillSource::Builtin,
            security_audit: None,
            data_scope: 0,
            external_id: None,
            updated_at: 0,
            certified: false,
            license: String::new(),
            source_url: None,
        });
        self.bandit
            .skill_scores
            .insert(id.to_string(), 0.5);
    }

    /// UCB1 score for a candidate
    ///
    /// `score = mean_reward + exploration_rate * sqrt(ln(total_evaluations) / (n + 1))`
    fn ucb1_score(&self, candidate: &SkillCandidate) -> f64 {
        let n = candidate.evaluation_count as f64;
        let total = self.bandit.total_evaluations as f64;
        if n == 0.0 {
            return f64::INFINITY;
        }
        let mean = candidate.average_score();
        let exploration = self.exploration_rate * (total.ln() / (n + 1.0)).sqrt();
        mean + exploration
    }

    /// Select top-k skills to evaluate under budget using UCB1 ranking
    pub fn select_skills_to_evaluate(&self, budget: u32) -> Vec<String> {
        let mut scored: Vec<(String, f64)> = self
            .candidates
            .iter()
            .filter(|c| c.maturity != SkillMaturity::Retired)
            .map(|c| (c.id.clone(), self.ucb1_score(c)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(budget as usize)
            .map(|(id, _)| id)
            .collect()
    }

    /// Record an evaluation outcome and update bandit state.
    ///
    /// NOTE: `success` is currently recorded only via `score` (callers fold
    /// success into the score); the flag is kept for a future success-weighted
    /// update — see blueprint V3 A26.
    pub fn record_outcome(&mut self, skill_id: &str, _success: bool, score: f64) {
        if self.bandit.budget_remaining == 0 {
            return;
        }

        let score = score.clamp(0.0, 1.0);

        if let Some(candidate) = self.candidates.iter_mut().find(|c| c.id == skill_id) {
            candidate.performance_history.push(score);
            candidate.evaluation_count += 1;
            candidate.last_evaluated = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
        }

        // Update bandit score (running average)
        let entry = self.bandit.skill_scores.entry(skill_id.to_string()).or_insert(0.0);
        let n = self.bandit.total_evaluations as f64;
        if n > 0.0 {
            *entry = (*entry * n + score) / (n + 1.0);
        } else {
            *entry = score;
        }

        self.bandit.total_evaluations += 1;
        self.bandit.budget_remaining = self.bandit.budget_remaining.saturating_sub(1);
    }

    /// Promote a skill to the next maturity level
    pub fn promote(&mut self, skill_id: &str) -> bool {
        if let Some(candidate) = self.candidates.iter_mut().find(|c| c.id == skill_id) {
            candidate.maturity = match candidate.maturity {
                SkillMaturity::Candidate => SkillMaturity::Provisional,
                SkillMaturity::Provisional => SkillMaturity::Trusted,
                SkillMaturity::Trusted => return false,
                SkillMaturity::Retired => SkillMaturity::Candidate,
            };
            return true;
        }
        false
    }

    /// Demote a skill to the previous maturity level
    pub fn demote(&mut self, skill_id: &str) -> bool {
        if let Some(candidate) = self.candidates.iter_mut().find(|c| c.id == skill_id) {
            candidate.maturity = match candidate.maturity {
                SkillMaturity::Candidate => return false,
                SkillMaturity::Provisional => SkillMaturity::Candidate,
                SkillMaturity::Trusted => SkillMaturity::Provisional,
                SkillMaturity::Retired => return false,
            };
            return true;
        }
        false
    }

    /// Retire a skill — remove from active pool
    pub fn retire(&mut self, skill_id: &str) -> bool {
        if let Some(candidate) = self.candidates.iter_mut().find(|c| c.id == skill_id) {
            if candidate.maturity == SkillMaturity::Retired {
                return false;
            }
            candidate.maturity = SkillMaturity::Retired;
            return true;
        }
        false
    }

    /// Auto-evolve: promote/demote based on performance history
    pub fn auto_evolve(&mut self) -> Vec<(String, String)> {
        let mut changes = Vec::new();
        let ids: Vec<String> = self.candidates.iter().map(|c| c.id.clone()).collect();

        for id in &ids {
            if let Some(c) = self.candidates.iter().find(|c| c.id == *id) {
                match c.maturity {
                    SkillMaturity::Candidate if c.evaluation_count >= SkillMaturity::CANDIDATE_THRESHOLD => {
                        let id_clone = id.clone();
                        self.promote(&id_clone);
                        changes.push((id_clone, "promoted to Provisional".to_string()));
                    }
                    SkillMaturity::Provisional if c.evaluation_count >= SkillMaturity::PROVISIONAL_THRESHOLD
                        && c.average_score() >= SkillMaturity::TRUSTED_MIN_SCORE =>
                    {
                        let id_clone = id.clone();
                        self.promote(&id_clone);
                        changes.push((id_clone, "promoted to Trusted".to_string()));
                    }
                    SkillMaturity::Trusted if c.recent_failures(10) >= SkillMaturity::MAX_FAILURES => {
                        let id_clone = id.clone();
                        self.demote(&id_clone);
                        changes.push((id_clone, "demoted to Provisional (too many failures)".to_string()));
                    }
                    _ => {}
                }
            }
        }
        changes
    }

    /// Get ranked skill recommendations for a task context
    ///
    /// Returns (skill_id, confidence) pairs sorted by confidence descending.
    /// Confidence = bandit_score * maturity_weight.
    pub fn get_recommendations(&self, _task_context: &TaskContext) -> Vec<(String, f64)> {
        let maturity_weight = |m: &SkillMaturity| match m {
            SkillMaturity::Trusted => 1.0,
            SkillMaturity::Provisional => 0.6,
            SkillMaturity::Candidate => 0.3,
            SkillMaturity::Retired => 0.0,
        };

        let mut recs: Vec<(String, f64)> = self
            .candidates
            .iter()
            .filter(|c| c.maturity != SkillMaturity::Retired)
            .map(|c| {
                let score = self.bandit.skill_scores.get(&c.id).copied().unwrap_or(0.0);
                let weight = maturity_weight(&c.maturity);
                (c.id.clone(), score * weight)
            })
            .collect();

        recs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        recs
    }

    /// Summary stats
    pub fn stats(&self) -> EvolverStats {
        let by_maturity = |m: SkillMaturity| -> usize {
            self.candidates.iter().filter(|c| c.maturity == m).count()
        };
        EvolverStats {
            total_candidates: self.candidates.len(),
            candidate_count: by_maturity(SkillMaturity::Candidate),
            provisional_count: by_maturity(SkillMaturity::Provisional),
            trusted_count: by_maturity(SkillMaturity::Trusted),
            retired_count: by_maturity(SkillMaturity::Retired),
            total_evaluations: self.bandit.total_evaluations,
            budget_remaining: self.bandit.budget_remaining,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolverStats {
    pub total_candidates: usize,
    pub candidate_count: usize,
    pub provisional_count: usize,
    pub trusted_count: usize,
    pub retired_count: usize,
    pub total_evaluations: u32,
    pub budget_remaining: u32,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_evolver() -> SkillEvolver {
        let mut ev = SkillEvolver::new(100, 1.41);
        ev.register("s1", "Refactor", "Code refactoring skill");
        ev.register("s2", "Debug", "Debugging skill");
        ev.register("s3", "Test", "Test writing skill");
        ev
    }

    #[test]
    fn test_register_dedup() {
        let mut ev = SkillEvolver::new(10, 1.0);
        ev.register("a", "A", "desc");
        ev.register("a", "A2", "desc2");
        assert_eq!(ev.candidates.len(), 1);
        assert_eq!(ev.candidates[0].name, "A");
    }

    #[test]
    fn test_ucb1_infinite_for_unvisited() {
        let ev = make_evolver();
        let c = &ev.candidates[0];
        assert_eq!(ev.ucb1_score(c), f64::INFINITY);
    }

    #[test]
    fn test_select_all_unvisited() {
        let ev = make_evolver();
        let selected = ev.select_skills_to_evaluate(3);
        assert_eq!(selected.len(), 3);
    }

    #[test]
    fn test_select_respects_budget() {
        let ev = make_evolver();
        let selected = ev.select_skills_to_evaluate(2);
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn test_record_outcome_updates_stats() {
        let mut ev = make_evolver();
        ev.record_outcome("s1", true, 0.9);
        assert_eq!(ev.bandit.total_evaluations, 1);
        assert_eq!(ev.bandit.budget_remaining, 99);
        assert_eq!(ev.candidates[0].evaluation_count, 1);
        assert_eq!(ev.candidates[0].performance_history, vec![0.9]);
    }

    #[test]
    fn test_record_outcome_clamps_score() {
        let mut ev = make_evolver();
        ev.record_outcome("s1", true, 1.5);
        assert_eq!(ev.candidates[0].performance_history, vec![1.0]);
        ev.record_outcome("s1", false, -0.5);
        assert_eq!(ev.candidates[0].performance_history, vec![1.0, 0.0]);
    }

    #[test]
    fn test_budget_exhausted() {
        let mut ev = SkillEvolver::new(2, 1.0);
        ev.register("a", "A", "d");
        ev.record_outcome("a", true, 0.8);
        ev.record_outcome("a", true, 0.9);
        assert_eq!(ev.bandit.budget_remaining, 0);
        // Further evaluations are ignored
        ev.record_outcome("a", true, 0.7);
        assert_eq!(ev.bandit.total_evaluations, 2);
    }

    #[test]
    fn test_promote_demote_cycle() {
        let mut ev = make_evolver();
        assert_eq!(ev.candidates[0].maturity, SkillMaturity::Candidate);
        assert!(ev.promote("s1"));
        assert_eq!(ev.candidates[0].maturity, SkillMaturity::Provisional);
        assert!(ev.promote("s1"));
        assert_eq!(ev.candidates[0].maturity, SkillMaturity::Trusted);
        assert!(!ev.promote("s1")); // already Trusted
        assert!(ev.demote("s1"));
        assert_eq!(ev.candidates[0].maturity, SkillMaturity::Provisional);
        assert!(ev.demote("s1"));
        assert_eq!(ev.candidates[0].maturity, SkillMaturity::Candidate);
        assert!(!ev.demote("s1")); // already Candidate
    }

    #[test]
    fn test_retire() {
        let mut ev = make_evolver();
        assert!(ev.retire("s1"));
        assert_eq!(ev.candidates[0].maturity, SkillMaturity::Retired);
        assert!(!ev.retire("s1")); // already retired
        // Retired skills excluded from selection
        let selected = ev.select_skills_to_evaluate(10);
        assert!(!selected.contains(&"s1".to_string()));
    }

    #[test]
    fn test_retire_and_promote复活() {
        let mut ev = make_evolver();
        ev.retire("s1");
        assert!(ev.promote("s1")); // Retired → Candidate
        assert_eq!(ev.candidates[0].maturity, SkillMaturity::Candidate);
    }

    #[test]
    fn test_auto_evolve_candidate_to_provisional() {
        let mut ev = make_evolver();
        for _ in 0..3 {
            ev.record_outcome("s1", true, 0.8);
        }
        let changes = ev.auto_evolve();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].0, "s1");
        assert_eq!(ev.candidates[0].maturity, SkillMaturity::Provisional);
    }

    #[test]
    fn test_auto_evolve_provisional_to_trusted() {
        let mut ev = make_evolver();
        // Promote to Provisional first
        for _ in 0..3 {
            ev.record_outcome("s1", true, 0.9);
        }
        ev.auto_evolve();
        assert_eq!(ev.candidates[0].maturity, SkillMaturity::Provisional);
        // Need 10 evals + avg >= 0.7
        for _ in 0..7 {
            ev.record_outcome("s1", true, 0.9);
        }
        ev.auto_evolve();
        assert_eq!(ev.candidates[0].maturity, SkillMaturity::Trusted);
    }

    #[test]
    fn test_get_recommendations_ranking() {
        let mut ev = make_evolver();
        // Give s1 good score, s2 mediocre
        ev.record_outcome("s1", true, 0.95);
        ev.record_outcome("s2", true, 0.4);
        ev.promote("s1"); // Candidate → Provisional (weight 0.6)

        let ctx = TaskContext {
            task_type: "refactor".into(),
            complexity: 0.5,
            domain: "code".into(),
            model_id: "test".into(),
            available_tokens: 1000,
        };
        let recs = ev.get_recommendations(&ctx);
        assert_eq!(recs.len(), 3);
        // s1 should be first (high score * provisional weight)
        assert_eq!(recs[0].0, "s1");
    }

    #[test]
    fn test_stats() {
        let mut ev = make_evolver();
        ev.retire("s3");
        ev.record_outcome("s1", true, 0.8);
        let stats = ev.stats();
        assert_eq!(stats.total_candidates, 3);
        assert_eq!(stats.retired_count, 1);
        assert_eq!(stats.total_evaluations, 1);
        assert_eq!(stats.budget_remaining, 99);
    }

    #[test]
    fn test_average_score_empty() {
        let c = SkillCandidate {
            id: "x".into(),
            name: "X".into(),
            description: "".into(),
            version: 1,
            performance_history: vec![],
            evaluation_count: 0,
            last_evaluated: None,
            maturity: SkillMaturity::Candidate,
            author: String::new(),
            tags: Vec::new(),
            triggers: Vec::new(),
            required_permissions: Vec::new(),
            dependencies: Vec::new(),
            source: SkillSource::Builtin,
            security_audit: None,
            data_scope: 0,
            external_id: None,
            updated_at: 0,
            certified: false,
            license: String::new(),
            source_url: None,
        };
        assert_eq!(c.average_score(), 0.0);
    }

    #[test]
    fn test_average_score() {
        let c = SkillCandidate {
            id: "x".into(),
            name: "X".into(),
            description: "".into(),
            version: 1,
            performance_history: vec![0.6, 0.8, 1.0],
            evaluation_count: 3,
            last_evaluated: None,
            maturity: SkillMaturity::Candidate,
            author: String::new(),
            tags: Vec::new(),
            triggers: Vec::new(),
            required_permissions: Vec::new(),
            dependencies: Vec::new(),
            source: SkillSource::Builtin,
            security_audit: None,
            data_scope: 0,
            external_id: None,
            updated_at: 0,
            certified: false,
            license: String::new(),
            source_url: None,
        };
        assert!((c.average_score() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_recent_failures() {
        let c = SkillCandidate {
            id: "x".into(),
            name: "X".into(),
            description: "".into(),
            version: 1,
            performance_history: vec![0.9, 0.1, 0.8, 0.2, 0.95],
            evaluation_count: 5,
            last_evaluated: None,
            maturity: SkillMaturity::Candidate,
            author: String::new(),
            tags: Vec::new(),
            triggers: Vec::new(),
            required_permissions: Vec::new(),
            dependencies: Vec::new(),
            source: SkillSource::Builtin,
            security_audit: None,
            data_scope: 0,
            external_id: None,
            updated_at: 0,
            certified: false,
            license: String::new(),
            source_url: None,
        };
        // Window of 3: [0.8, 0.2, 0.95] → 1 failure
        assert_eq!(c.recent_failures(3), 1);
        // Window of 5: [0.9, 0.1, 0.8, 0.2, 0.95] → 2 failures
        assert_eq!(c.recent_failures(5), 2);
    }

    #[test]
    fn test_new_fields_default_and_serde_backward_compat() {
        // Defaults via register()
        let ev = make_evolver();
        let c = &ev.candidates[0];
        assert!(c.author.is_empty());
        assert!(c.tags.is_empty());
        assert!(c.triggers.is_empty());
        assert!(c.required_permissions.is_empty());
        assert!(c.dependencies.is_empty());
        assert_eq!(c.source, SkillSource::Builtin);
        assert!(c.security_audit.is_none());
        assert_eq!(c.data_scope, 0);
        assert!(c.external_id.is_none());
        assert_eq!(c.updated_at, 0);
        assert!(!c.certified);

        // Old snapshot without the new keys must still parse (serde defaults)
        let old_json = r#"{"id":"x","name":"X","description":"","version":1,"performance_history":[],"evaluation_count":0,"last_evaluated":null,"maturity":"candidate"}"#;
        let parsed: SkillCandidate = match serde_json::from_str(old_json) {
            Ok(v) => v,
            Err(e) => {
                assert!(false, "old snapshot must parse: {e}");
                return;
            }
        };
        assert!(parsed.author.is_empty());
        assert!(parsed.tags.is_empty());
        assert_eq!(parsed.source, SkillSource::Builtin);
        assert!(parsed.security_audit.is_none());
        assert_eq!(parsed.data_scope, 0);
        assert!(parsed.external_id.is_none());
        assert!(!parsed.certified);

        // Serde round-trip preserves the new fields
        let value = match serde_json::to_value(&parsed) {
            Ok(v) => v,
            Err(e) => {
                assert!(false, "serialize must succeed: {e}");
                return;
            }
        };
        let back: SkillCandidate = match serde_json::from_value(value) {
            Ok(v) => v,
            Err(e) => {
                assert!(false, "round-trip must parse: {e}");
                return;
            }
        };
        assert_eq!(back.id, parsed.id);
        assert_eq!(back.source, parsed.source);
        assert_eq!(back.data_scope, parsed.data_scope);
    }

    #[test]
    fn test_t35_license_source_url_serde() {
        // register 默认
        let ev = make_evolver();
        let c = &ev.candidates[0];
        assert!(c.license.is_empty());
        assert!(c.source_url.is_none());
        // 旧快照（无 license/source_url）兼容
        let old_json = r#"{"id":"y","name":"Y","description":"","version":1,"performance_history":[],"evaluation_count":0,"last_evaluated":null,"maturity":"candidate"}"#;
        let parsed: SkillCandidate = match serde_json::from_str(old_json) {
            Ok(v) => v,
            Err(e) => {
                assert!(false, "old snapshot must parse: {e}");
                return;
            }
        };
        assert!(parsed.license.is_empty());
        assert!(parsed.source_url.is_none());
        // 非默认往返
        let mut full = parsed;
        full.license = "MIT".to_string();
        full.source_url = Some("https://example.com/skill.zip".to_string());
        let value = match serde_json::to_value(&full) {
            Ok(v) => v,
            Err(e) => {
                assert!(false, "serialize must succeed: {e}");
                return;
            }
        };
        let back: SkillCandidate = match serde_json::from_value(value) {
            Ok(v) => v,
            Err(e) => {
                assert!(false, "round-trip must parse: {e}");
                return;
            }
        };
        assert_eq!(back.license, "MIT");
        assert_eq!(
            back.source_url,
            Some("https://example.com/skill.zip".to_string())
        );
    }
}
