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
    /// P0-1: append-only 技能贡献账本 (Library Drift 处方).
    /// 只追加不修改; 旧快照无此键经 serde default 兼容为空账本.
    #[serde(default)]
    pub nt_ledger: Vec<NtContributionEntry>,
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
            nt_ledger: Vec::new(),
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
        self.bandit.skill_scores.insert(id.to_string(), 0.5);
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
        let entry = self
            .bandit
            .skill_scores
            .entry(skill_id.to_string())
            .or_insert(0.0);
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
                    SkillMaturity::Candidate
                        if c.evaluation_count >= SkillMaturity::CANDIDATE_THRESHOLD =>
                    {
                        let id_clone = id.clone();
                        self.promote(&id_clone);
                        changes.push((id_clone, "promoted to Provisional".to_string()));
                    }
                    SkillMaturity::Provisional
                        if c.evaluation_count >= SkillMaturity::PROVISIONAL_THRESHOLD
                            && c.average_score() >= SkillMaturity::TRUSTED_MIN_SCORE =>
                    {
                        let id_clone = id.clone();
                        self.promote(&id_clone);
                        changes.push((id_clone, "promoted to Trusted".to_string()));
                    }
                    SkillMaturity::Trusted
                        if c.recent_failures(10) >= SkillMaturity::MAX_FAILURES =>
                    {
                        let id_clone = id.clone();
                        self.demote(&id_clone);
                        changes.push((
                            id_clone,
                            "demoted to Provisional (too many failures)".to_string(),
                        ));
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
// P0-1: Contribution ledger + drift guardrails (Library Drift 处方, A)
// ============================================================================
//
// 背景: 自进化技能库的静默失败模式是 library drift — 无界累积 +
// 无 outcome 生命周期管理 → 检索退化 + 有害注入. 处方三件套:
// outcome-driven 退役 + 有界 active-cap + 归因遥测.
// 参考: arXiv:2605.19576 (Library Drift).
//
// 设计约束:
// - 账本 append-only: 只 push, 永不就地修改历史条目;
// - 账本是观测层, 不消耗 bandit budget (与 record_outcome 解耦);
// - 全部新字段 serde default, 旧快照反序列化兼容.

/// 单次技能注入的归因 verdict (Critic 判定).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum NtContributionVerdict {
    Helped,
    Hurt,
    Neutral,
    #[default]
    Inapplicable,
}

/// Append-only 贡献账本条目: 一次任务决策中某技能的一次归因记录.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtContributionEntry {
    #[serde(default)]
    pub skill_id: String,
    #[serde(default)]
    pub task_id: String,
    #[serde(default)]
    pub verdict: NtContributionVerdict,
    /// 贡献分 ∈ [-1.0, 1.0]: helped 为正, hurt 为负.
    #[serde(default)]
    pub contribution: f64,
    /// 归因模式标签 (如 "stale-retry", "wrong-scope"), 用于聚类.
    #[serde(default)]
    pub pattern_label: String,
    /// Critic 置信度 ∈ [0.0, 1.0].
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub timestamp: u64,
}

impl NtContributionEntry {
    /// 构造一条账本条目; contribution/confidence 按范围 clamp.
    pub fn nt_new(
        skill_id: &str,
        task_id: &str,
        verdict: NtContributionVerdict,
        contribution: f64,
        pattern_label: &str,
        confidence: f64,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            skill_id: skill_id.to_string(),
            task_id: task_id.to_string(),
            verdict,
            contribution: contribution.clamp(-1.0, 1.0),
            pattern_label: pattern_label.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            timestamp: now,
        }
    }
}

/// 账本统计快照 (trace-level 诊断信号).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NtLedgerStats {
    #[serde(default)]
    pub total_entries: usize,
    #[serde(default)]
    pub helped: usize,
    #[serde(default)]
    pub hurt: usize,
    #[serde(default)]
    pub neutral: usize,
    #[serde(default)]
    pub inapplicable: usize,
    /// 全账本平均贡献分.
    #[serde(default)]
    pub mean_contribution: f64,
    /// 路由器 engagement = 非 inapplicable 条目占比; 空账本返回 1.0 (无信号≠告警).
    #[serde(default)]
    pub engagement: f64,
    /// hurt 占比; 空账本返回 0.0.
    #[serde(default)]
    pub hurt_ratio: f64,
}

impl SkillEvolver {
    /// 证据下限: 累积 trial 数不足此值不得退役 (防 A4 式过早退役坍缩).
    pub const NT_EVIDENCE_FLOOR: u32 = 100;
    /// 退役阈值: trial 充足且经验贡献 ≤ -τ 时退役.
    pub const NT_RETIRE_TAU: f64 = 0.10;
    /// 有界 active-cap: 非退役技能数上限, 超出则按贡献分驱逐最低者.
    pub const NT_ACTIVE_CAP: usize = 50;
    /// engagement 告警线: 低于此值说明路由器已不信任技能库 (漂移先兆).
    pub const NT_ENGAGEMENT_WARN: f64 = 0.5;

    /// 追加一条归因记录 (append-only, 不消耗 budget).
    pub fn nt_record_contribution(&mut self, entry: NtContributionEntry) {
        self.nt_ledger.push(entry);
    }

    /// 取某技能的全部账本条目 (按追加顺序).
    pub fn nt_contributions_for(&self, skill_id: &str) -> Vec<&NtContributionEntry> {
        self.nt_ledger
            .iter()
            .filter(|e| e.skill_id == skill_id)
            .collect()
    }

    /// 某技能的经验贡献均值; 无条目返回 None (≠0, 避免把"无证据"当"零贡献").
    pub fn nt_mean_contribution(&self, skill_id: &str) -> Option<f64> {
        let entries = self.nt_contributions_for(skill_id);
        if entries.is_empty() {
            return None;
        }
        let sum: f64 = entries.iter().map(|e| e.contribution).sum();
        Some(sum / entries.len() as f64)
    }

    /// 是否满足退役条件: trial 数 ≥ 证据下限 且 经验贡献 ≤ -τ.
    pub fn nt_should_retire(&self, skill_id: &str) -> bool {
        let entries = self.nt_contributions_for(skill_id);
        if (entries.len() as u32) < Self::NT_EVIDENCE_FLOOR {
            return false;
        }
        let sum: f64 = entries.iter().map(|e| e.contribution).sum();
        sum / entries.len() as f64 <= -Self::NT_RETIRE_TAU
    }

    /// 账本统计快照: per-verdict 计数 + engagement + hurt_ratio.
    pub fn nt_ledger_stats(&self) -> NtLedgerStats {
        let mut stats = NtLedgerStats::default();
        stats.total_entries = self.nt_ledger.len();
        if self.nt_ledger.is_empty() {
            stats.engagement = 1.0;
            return stats;
        }
        let mut sum = 0.0;
        for e in &self.nt_ledger {
            sum += e.contribution;
            match e.verdict {
                NtContributionVerdict::Helped => stats.helped += 1,
                NtContributionVerdict::Hurt => stats.hurt += 1,
                NtContributionVerdict::Neutral => stats.neutral += 1,
                NtContributionVerdict::Inapplicable => stats.inapplicable += 1,
            }
        }
        let total = stats.total_entries as f64;
        stats.mean_contribution = sum / total;
        stats.engagement = (stats.helped + stats.hurt + stats.neutral) as f64 / total;
        stats.hurt_ratio = stats.hurt as f64 / total;
        stats
    }

    /// 路由器 engagement 是否跌破告警线 (漂移先兆; 空账本不告警).
    pub fn nt_engagement_warned(&self) -> bool {
        !self.nt_ledger.is_empty() && self.nt_ledger_stats().engagement < Self::NT_ENGAGEMENT_WARN
    }

    /// 治理执行: outcome-driven 退役 + 有界 active-cap 驱逐.
    ///
    /// 返回 (skill_id, 动作描述) 变更清单; 纯加法, 不改动现有 auto_evolve.
    pub fn nt_enforce_governance(&mut self) -> Vec<(String, String)> {
        let mut changes = Vec::new();
        // 1) 有害技能退役 (证据充足才动手).
        let ids: Vec<String> = self.candidates.iter().map(|c| c.id.clone()).collect();
        for id in &ids {
            if self.nt_should_retire(id) && self.retire(id) {
                changes.push((
                    id.clone(),
                    "retired by contribution ledger (harmful)".to_string(),
                ));
            }
        }
        // 2) 有界 active-cap: 非退役数超限则按贡献分驱逐最低者.
        loop {
            let mut active: Vec<(String, f64)> = self
                .candidates
                .iter()
                .filter(|c| c.maturity != SkillMaturity::Retired)
                .map(|c| {
                    let mean = self.nt_mean_contribution(&c.id).unwrap_or(0.0);
                    (c.id.clone(), mean)
                })
                .collect();
            if active.len() <= Self::NT_ACTIVE_CAP {
                break;
            }
            active.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            let victim = active.first().map(|(id, _)| id.clone());
            match victim {
                Some(id) => {
                    if self.retire(&id) {
                        changes.push((
                            id,
                            "evicted by active-cap (lowest contribution)".to_string(),
                        ));
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }
        changes
    }
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

    // ── P0-1 贡献账本 + 漂移护栏 ──────────────────────────────

    fn nt_entry(skill: &str, task: &str, v: NtContributionVerdict, c: f64) -> NtContributionEntry {
        NtContributionEntry::nt_new(skill, task, v, c, "unit-test", 0.9)
    }

    #[test]
    fn test_nt_ledger_append_only_and_mean() {
        let mut ev = make_evolver();
        assert!(ev.nt_mean_contribution("s1").is_none());
        ev.nt_record_contribution(nt_entry("s1", "t1", NtContributionVerdict::Helped, 0.6));
        ev.nt_record_contribution(nt_entry("s1", "t2", NtContributionVerdict::Hurt, -0.2));
        assert_eq!(ev.nt_ledger.len(), 2);
        assert_eq!(ev.nt_contributions_for("s1").len(), 2);
        assert!(ev.nt_contributions_for("s2").is_empty());
        match ev.nt_mean_contribution("s1") {
            Some(m) => assert!((m - 0.2).abs() < 1e-10),
            None => assert!(false, "s1 must have a mean"),
        }
        // 账本不消耗 budget
        assert_eq!(ev.bandit.total_evaluations, 0);
    }

    #[test]
    fn test_nt_should_retire_needs_evidence_floor() {
        let mut ev = make_evolver();
        // 证据不足: 全 hurt 也不退役 (防 A4 式坍缩)
        for i in 0..20 {
            let task = format!("t{i}");
            ev.nt_record_contribution(nt_entry("s1", &task, NtContributionVerdict::Hurt, -1.0));
        }
        assert!(!ev.nt_should_retire("s1"));
        // 补足到证据下限且均值 ≤ -τ → 退役
        for i in 20..100 {
            let task = format!("t{i}");
            ev.nt_record_contribution(nt_entry("s1", &task, NtContributionVerdict::Hurt, -1.0));
        }
        assert!(ev.nt_should_retire("s1"));
        // 有益技能不退役
        for i in 0..100 {
            let task = format!("g{i}");
            ev.nt_record_contribution(nt_entry("s2", &task, NtContributionVerdict::Helped, 0.8));
        }
        assert!(!ev.nt_should_retire("s2"));
    }

    #[test]
    fn test_nt_enforce_governance_retires_harmful() {
        let mut ev = make_evolver();
        for i in 0..100 {
            let task = format!("t{i}");
            ev.nt_record_contribution(nt_entry("s1", &task, NtContributionVerdict::Hurt, -0.5));
        }
        let changes = ev.nt_enforce_governance();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].0, "s1");
        let s1 = ev.candidates.iter().find(|c| c.id == "s1");
        match s1 {
            Some(c) => assert_eq!(c.maturity, SkillMaturity::Retired),
            None => assert!(false, "s1 must exist"),
        }
    }

    #[test]
    fn test_nt_ledger_stats_and_engagement_warn() {
        let mut ev = make_evolver();
        // 空账本: engagement=1.0, 不告警
        let empty = ev.nt_ledger_stats();
        assert_eq!(empty.total_entries, 0);
        assert!((empty.engagement - 1.0).abs() < 1e-10);
        assert!(!ev.nt_engagement_warned());
        // 路由器大量选择不注入 → engagement 跌破 0.5 告警
        ev.nt_record_contribution(nt_entry("s1", "t1", NtContributionVerdict::Helped, 0.5));
        for i in 0..9 {
            let task = format!("skip{i}");
            ev.nt_record_contribution(nt_entry(
                "s1",
                &task,
                NtContributionVerdict::Inapplicable,
                0.0,
            ));
        }
        let stats = ev.nt_ledger_stats();
        assert_eq!(stats.total_entries, 10);
        assert_eq!(stats.helped, 1);
        assert_eq!(stats.inapplicable, 9);
        assert!((stats.engagement - 0.1).abs() < 1e-10);
        assert!(ev.nt_engagement_warned());
    }

    #[test]
    fn test_nt_ledger_serde_backward_compat() {
        // 旧快照无 nt_ledger 键仍可解析
        let old_json = r#"{"candidates":[],"bandit":{"skill_scores":{},"total_evaluations":0,"budget_remaining":10},"exploration_rate":1.0}"#;
        let parsed: SkillEvolver = match serde_json::from_str(old_json) {
            Ok(v) => v,
            Err(e) => {
                assert!(false, "old snapshot must parse: {e}");
                return;
            }
        };
        assert!(parsed.nt_ledger.is_empty());
        // 往返保留账本
        let mut ev = parsed;
        ev.nt_record_contribution(nt_entry("s9", "t9", NtContributionVerdict::Hurt, -0.4));
        let value = match serde_json::to_value(&ev) {
            Ok(v) => v,
            Err(e) => {
                assert!(false, "serialize must succeed: {e}");
                return;
            }
        };
        let back: SkillEvolver = match serde_json::from_value(value) {
            Ok(v) => v,
            Err(e) => {
                assert!(false, "round-trip must parse: {e}");
                return;
            }
        };
        assert_eq!(back.nt_ledger.len(), 1);
        assert_eq!(back.nt_ledger[0].skill_id, "s9");
    }
}
