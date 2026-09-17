#![deny(clippy::unwrap_used)]

use crate::core::nt_core_hex::FullReasoningState;
use crate::core::nt_core_policy::E8Policy;
use serde::{Deserialize, Serialize};

/// E8 Plan Mode — 将推理轨迹编码为结构化计划，每个步骤是对应 E8 卦象状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8Plan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<PlanStep>,
    pub e8_sequence: Vec<u8>,
    pub metrics: PlanMetrics,
    pub created_at: u64,
    pub execution_count: u64,
    /// J-Space 账本 Verified 段 — 编号、append-only、每条必须携带 verifier + coverage。
    #[serde(default)]
    pub verified: Vec<VerifiedEntry>,
    /// J-Space 账本 Open 段 — 编号永不复用；关闭须对已记录 checkpoint。
    #[serde(default)]
    pub open: Vec<OpenEntry>,
    /// J-Space 账本 Core 段 — 每条必须是 "name — 使它在意的那个事实" (jspace.py
    /// note --core: 无 defining fact 的 core 是 mention, 不是 load)。
    #[serde(default)]
    pub core: Vec<String>,
}

/// J-Space 账本 Verified 条目 (j-space: capacity.md / workspace-ledger.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedEntry {
    /// 编号 `✓NN`，append-only，为回滚提供地址
    pub id: usize,
    /// 现在成立的结论
    pub claim: String,
    /// 验证者 — 什么建立了它 (测试/差分/文档)
    pub verifier: String,
    /// 验证覆盖范围 — 覆盖了什么、没覆盖什么
    pub coverage: String,
    /// 关闭后新解锁的下一步 (non-empty)
    pub next: String,
}

/// J-Space 账本 Open 条目 (j-space: capacity.md / workspace-ledger.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenEntry {
    /// 编号 `?NN`，永不复用
    pub id: usize,
    /// 仍未解决的问题
    pub question: String,
    /// 什么可以解决它 (最便宜的证伪测试)
    pub settled_by: String,
    /// 关闭时引用的 checkpoint 编号；None = 仍开放
    pub closed_by: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub index: usize,
    pub e8_mode: u8,
    pub action: String,
    pub expected_outcome: String,
    pub prm_score: f64,
    pub status: StepStatus,
    pub actual_outcome: Option<String>,
    pub completion_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMetrics {
    pub total_steps: usize,
    pub completed_steps: usize,
    pub avg_prm_score: f64,
    pub estimated_cost: f64,
    pub est_completion_ms: u64,
    pub e8_mode_stability: f64,
    pub goal_alignment: f64,
}

/// 计划生成器 — 利用 E8 状态机 + PRM 策略生成最优计划
pub struct PlanGenerator {
    pub policy: Option<E8Policy>,
    pub planner_mode: u8,
    pub max_steps: usize,
    pub prm_threshold: f64,
}

impl PlanGenerator {
    pub fn new() -> Self {
        Self {
            policy: None,
            planner_mode: 7,
            max_steps: 12,
            prm_threshold: 0.3,
        }
    }

    pub fn with_policy(mut self, policy: E8Policy) -> Self {
        self.policy = Some(policy);
        self
    }

    pub fn generate_plan(&self, goal: &str, context: &[FullReasoningState]) -> E8Plan {
        let steps = self.generate_steps(goal, context);
        let scores: Vec<f64> = steps.iter().map(|s| s.prm_score).collect();
        let avg_prm = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        E8Plan {
            id,
            goal: goal.to_string(),
            e8_sequence: steps.iter().map(|s| s.e8_mode).collect(),
            metrics: PlanMetrics {
                total_steps: steps.len(),
                completed_steps: 0,
                avg_prm_score: avg_prm,
                estimated_cost: steps.len() as f64 * 0.005,
                est_completion_ms: steps.len() as u64 * 2000,
                e8_mode_stability: self.compute_mode_stability(&steps),
                goal_alignment: self.compute_goal_alignment(&steps, goal),
            },
            execution_count: 0,
            steps,
            created_at,
            verified: Vec::new(),
            open: Vec::new(),
            core: Vec::new(),
        }
    }

    fn generate_steps(&self, goal: &str, context: &[FullReasoningState]) -> Vec<PlanStep> {
        let mut steps = Vec::new();
        let task_len = goal.len().min(200);

        // Plan phases mapped to E8 modes
        let phase_modes: [u8; 6] = [1, 9, 17, 33, 41, 57];
        let phase_actions = [
            "analyze_goal",
            "gather_context",
            "generate_strategy",
            "execute",
            "verify",
            "reflect",
        ];

        for (i, (&mode, action)) in phase_modes.iter().zip(phase_actions.iter()).enumerate() {
            if i >= self.max_steps {
                break;
            }
            let score = self.score_mode_for_goal(mode, goal, context);
            if score >= self.prm_threshold {
                steps.push(PlanStep {
                    index: i,
                    e8_mode: mode,
                    action: action.to_string(),
                    expected_outcome: format!("Execute phase {} with E8 mode {}", action, mode),
                    prm_score: score,
                    status: StepStatus::Pending,
                    actual_outcome: None,
                    completion_time_ms: None,
                });
            }
        }

        if steps.is_empty() {
            steps.push(PlanStep {
                index: 0,
                e8_mode: self.planner_mode,
                action: "default_execute".to_string(),
                expected_outcome: format!("Default execution for: {}", &goal[..task_len.min(60)]),
                prm_score: 0.5,
                status: StepStatus::Pending,
                actual_outcome: None,
                completion_time_ms: None,
            });
        }

        steps
    }

    fn score_mode_for_goal(&self, mode: u8, _goal: &str, context: &[FullReasoningState]) -> f64 {
        if self.policy.is_some() {
            0.5 + (mode as f64) / 128.0
        } else if !context.is_empty() {
            let recent = context.last().unwrap_or(&context[0]);
            let similarity = 1.0 - (recent.mode.0 as f64 - mode as f64).abs() / 64.0;
            0.3 + similarity * 0.5
        } else {
            0.4
        }
    }

    fn compute_mode_stability(&self, steps: &[PlanStep]) -> f64 {
        if steps.len() < 2 {
            return 1.0;
        }
        let transitions = steps
            .windows(2)
            .filter(|w| w[0].e8_mode != w[1].e8_mode)
            .count();
        1.0 - transitions as f64 / steps.len() as f64
    }

    fn compute_goal_alignment(&self, steps: &[PlanStep], _goal: &str) -> f64 {
        if steps.is_empty() {
            return 0.0;
        }
        steps.iter().map(|s| s.prm_score).sum::<f64>() / steps.len() as f64
    }

    pub fn execute_step(&self, step: &mut PlanStep, outcome: &str, duration_ms: u64) {
        step.status = StepStatus::Completed;
        step.actual_outcome = Some(outcome.to_string());
        step.completion_time_ms = Some(duration_ms);
    }

    pub fn fail_step(&self, step: &mut PlanStep, error: &str) {
        step.status = StepStatus::Failed(error.to_string());
    }
}

impl Default for PlanGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl E8Plan {
    pub fn next_pending(&self) -> Option<&PlanStep> {
        self.steps
            .iter()
            .find(|s| matches!(s.status, StepStatus::Pending))
    }

    pub fn completion_pct(&self) -> f64 {
        if self.steps.is_empty() {
            return 1.0;
        }
        self.steps
            .iter()
            .filter(|s| matches!(s.status, StepStatus::Completed))
            .count() as f64
            / self.steps.len() as f64
    }

    pub fn is_complete(&self) -> bool {
        self.steps
            .iter()
            .all(|s| matches!(s.status, StepStatus::Completed | StepStatus::Skipped))
    }

    pub fn duration_ms(&self) -> u64 {
        self.steps.iter().filter_map(|s| s.completion_time_ms).sum()
    }

    /// 记录一个 Verified 条目 (J-Space checkpoint)。id 单调递增，append-only。
    /// `next` 为空即拒绝 — 账本不允许无下一步的 checkpoint (jspace.py:
    /// "a checkpoint with no record is not a checkpoint")。
    /// `coverage` 为空即拒绝 — "verified without stated coverage is a mood,
    /// not a result" (jspace.py ship / note --by)。coverage 必须描述覆盖了什么、
    /// 没覆盖什么，且不可与 claim 相同 (防填充式覆盖)。
    /// `claim`/`verifier` 为空即拒绝；claim 以 "## " 开头即拒绝 (防 ledger
    /// section heading 注入)。
    pub fn record_verified(&mut self, claim: &str, verifier: &str, coverage: &str, next: &str) {
        if next.trim().is_empty()
            || coverage.trim().is_empty()
            || coverage.trim().eq_ignore_ascii_case(claim.trim())
            || verifier.trim().is_empty()
            || claim.trim().is_empty()
            || claim.trim_start().starts_with("## ")
        {
            return;
        }
        let id = self.verified.len() + 1;
        self.verified.push(VerifiedEntry {
            id,
            claim: claim.to_string(),
            verifier: verifier.to_string(),
            coverage: coverage.to_string(),
            next: next.to_string(),
        });
    }

    /// 打开一个 Open 条目 (J-Space 命名未知量)。id 单调递增，永不复用。
    /// `settled_by` 为空即拒绝 — 无法关闭的问题不能打开 (jspace.py:
    /// "an open question with nothing that would settle it cannot be closed")。
    /// question 以 "## " 开头即拒绝 (防 ledger section heading 注入)。
    pub fn open_question(&mut self, question: &str, settled_by: &str) {
        if settled_by.trim().is_empty()
            || question.trim().is_empty()
            || question.trim_start().starts_with("## ")
        {
            return;
        }
        let id = self.open.len() + 1;
        self.open.push(OpenEntry {
            id,
            question: question.to_string(),
            settled_by: settled_by.to_string(),
            closed_by: None,
        });
    }

    /// 关闭一个 Open 条目 — 必须引用已记录的 checkpoint id，否则拒绝。
    /// 返回 false 表示关闭失败 (引用了不存在的 checkpoint)。
    pub fn close_question(&mut self, open_id: usize, verified_id: usize) -> bool {
        if verified_id == 0 || verified_id > self.verified.len() {
            return false;
        }
        let entry = self.open.iter_mut().find(|o| o.id == open_id);
        match entry {
            Some(e) if e.closed_by.is_none() => {
                e.closed_by = Some(verified_id);
                true
            }
            _ => false,
        }
    }

    /// J-Space `Next` 非空约束: 取最后一个 checkpoint 的 next，否则第一个 pending 步骤。
    pub fn next_action(&self) -> String {
        if let Some(last) = self.verified.last() {
            return last.next.clone();
        }
        self.next_pending()
            .map(|s| s.action.clone())
            .unwrap_or_default()
    }

    /// 最近一个可回滚地址 (最后一个 verified id)，J-Space rollback 用。
    pub fn last_checkpoint(&self) -> Option<&VerifiedEntry> {
        self.verified.last()
    }

    /// 记录一个 Core 账本条目 (jspace.py note --core)。每个条目必须是
    /// "name — 使它在意的那个事实" (或 "name - fact")：无 defining fact 的
    /// core 是 mention, 不是 load, 被拒绝。已存在同名条目则不重复追加。
    /// 返回是否真正写入。
    pub fn note_core(&mut self, entry: &str) -> bool {
        let has_fact = entry.contains('—') || entry.contains(" - ");
        if entry.trim().is_empty() || !has_fact {
            return false;
        }
        if self.core.iter().any(|c| c == entry) {
            return false;
        }
        self.core.push(entry.to_string());
        true
    }

    /// 批量账本编辑 (jspace.py mode_note 原子拒绝语义): 接受所有独立合法编辑,
    /// 拒绝格式错误项, 被拒绝的项不影响已接受的项 ("a declined independent
    /// edit must not cost an accepted one")。返回 (接受数, 拒绝的条目)。
    pub fn apply_notes(
        &mut self,
        goal: Option<&str>,
        next: Option<&str>,
        checks: &[(String, String, String, String)],
        opens: &[(String, String)],
        core: &[String],
    ) -> (usize, Vec<String>) {
        let mut accepted = 0;
        let mut refused = Vec::new();
        if let Some(g) = goal {
            if g.trim_start().starts_with("## ") || g.trim().is_empty() {
                refused.push("goal: must not begin with a ledger section heading (## )".to_string());
            } else {
                self.goal = g.to_string();
                accepted += 1;
            }
        }
        if let Some(n) = next {
            if n.trim_start().starts_with("## ") || n.trim().is_empty() {
                refused.push("next: must not begin with a ledger section heading (## )".to_string());
            } else {
                self.record_verified("interim", "batch", "n/a", n);
                accepted += 1;
            }
        }
        for (claim, verifier, coverage, next) in checks {
            let before = self.verified.len();
            self.record_verified(claim, verifier, coverage, next);
            if self.verified.len() > before {
                accepted += 1;
            } else {
                refused.push(format!("check: rejected ({claim})"));
            }
        }
        for (question, settled_by) in opens {
            let before = self.open.len();
            self.open_question(question, settled_by);
            if self.open.len() > before {
                accepted += 1;
            } else {
                refused.push(format!("open: rejected ({question})"));
            }
        }
        for entry in core {
            if self.note_core(entry) {
                accepted += 1;
            } else {
                refused.push(format!("core: rejected ({entry})"));
            }
        }
        (accepted, refused)
    }

    /// J-Space reentry anchor (jspace.py resume / seam long-gap): 长间隔后
    /// 完整重入锚 — 完整账本 + invariants + "state the current pass, then
    /// Next names the first action back"。
    pub fn reentry_anchor(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Goal: {}\n", self.goal));
        out.push_str("Core:\n");
        for (i, c) in self.core.iter().enumerate() {
            out.push_str(&format!("  [{}] {}\n", if i < 2 { "live" } else { "parked" }, c));
        }
        if self.core.is_empty() {
            out.push_str("  (empty)\n");
        }
        out.push_str("Verified:\n");
        for v in &self.verified {
            out.push_str(&format!(
                "  ✓{:02} {} — verified by: {} ({})\n",
                v.id, v.claim, v.verifier, v.coverage
            ));
        }
        if self.verified.is_empty() {
            out.push_str("  (none yet)\n");
        }
        out.push_str("Open:\n");
        for o in &self.open {
            let state = o
                .closed_by
                .map(|c| format!("closed by ✓{c}"))
                .unwrap_or_else(|| format!("settled by: {}", o.settled_by));
            out.push_str(&format!("  ?{:02} {} — {state}\n", o.id, o.question));
        }
        if self.open.is_empty() {
            out.push_str("  (none)\n");
        }
        out.push_str(&format!("Next: {}\n\n", self.next_action()));
        out.push_str("Not working if:\n");
        out.push_str("  the ledger stops being state (no next action).\n");
        out.push_str("State the current pass, then make Next name the first action back.\n");
        out
    }

    /// 账本一致性校验 (jspace.py LedgerReadError → CANNOT, 修复前拒写更多状态):
    /// 任何 open 引用了不存在的 closed_by、或 closed_by 为 None 但 settled_by 为空、
    /// 或 Verified 非空但 next_action 无落点, 视为损坏。
    pub fn is_well_formed(&self) -> bool {
        for o in &self.open {
            if let Some(c) = o.closed_by {
                if c == 0 || c > self.verified.len() {
                    return false;
                }
            } else if o.settled_by.trim().is_empty() {
                return false;
            }
        }
        for v in &self.verified {
            if v.id != 0 && v.id > self.verified.len() {
                return false;
            }
            if v.next.trim().is_empty() || v.coverage.trim().is_empty() {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_generation() {
        let gen = PlanGenerator::new();
        let plan = gen.generate_plan("Build a web search tool", &[]);
        assert!(!plan.steps.is_empty());
        assert!(plan.metrics.total_steps > 1);
        assert_eq!(plan.steps[0].index, 0);
    }

    #[test]
    fn test_plan_step_execution() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Test step execution", &[]);
        let step = plan.steps.first_mut().unwrap();
        gen.execute_step(step, "completed successfully", 1500);
        assert!(matches!(step.status, StepStatus::Completed));
        assert_eq!(step.completion_time_ms, Some(1500));
    }

    #[test]
    fn test_plan_completion() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Test completion", &[]);
        for step in plan.steps.iter_mut() {
            gen.execute_step(step, "done", 100);
        }
        assert!(plan.is_complete());
        assert!(plan.completion_pct() > 0.99);
    }

    #[test]
    fn test_plan_metrics() {
        let gen = PlanGenerator::new();
        let plan = gen.generate_plan("Test metrics", &[]);
        assert!(plan.metrics.avg_prm_score >= 0.0);
        assert!(plan.metrics.e8_mode_stability >= 0.0);
        assert!(plan.metrics.goal_alignment >= 0.0);
    }

    #[test]
    fn test_empty_context_plan() {
        let gen = PlanGenerator::new();
        let plan = gen.generate_plan("High threshold plan", &[]);
        assert!(!plan.steps.is_empty());
    }

    #[test]
    fn test_ledger_record_verified_requires_next() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Ledger", &[]);
        plan.record_verified("claim", "test", "coverage", "");
        assert!(plan.verified.is_empty(), "empty next must be rejected");
        plan.record_verified("claim", "test", "coverage", "next step");
        assert_eq!(plan.verified.len(), 1);
        assert_eq!(plan.verified[0].id, 1);
        assert_eq!(plan.next_action(), "next step");
    }

    #[test]
    fn test_ledger_append_only_monotonic_ids() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Ledger", &[]);
        plan.record_verified("v1", "t", "c", "n1");
        plan.record_verified("v2", "t", "c", "n2");
        assert_eq!(plan.verified.len(), 2);
        assert_eq!(plan.verified[1].id, 2);
        assert_eq!(plan.next_action(), "n2");
        assert_eq!(plan.last_checkpoint().map(|v| v.id), Some(2));
    }

    #[test]
    fn test_ledger_open_close_binds_checkpoint() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Ledger", &[]);
        plan.open_question("CAP value", "brute force n<=6");
        plan.open_question("greedy vs opt", "differential test");
        assert_eq!(plan.open.len(), 2);

        assert!(!plan.close_question(1, 5), "nonexistent checkpoint rejected");
        plan.record_verified("CAP=m-2", "brute", "n<=6", "write sol.cpp");
        assert!(plan.close_question(1, 1));
        assert_eq!(plan.open[0].closed_by, Some(1));
        assert!(!plan.close_question(1, 2), "double close rejected");
        assert!(plan.close_question(2, 1), "open stays closeable");
    }

    #[test]
    fn test_next_falls_back_to_pending_step() {
        let gen = PlanGenerator::new();
        let plan = gen.generate_plan("No verified yet", &[]);
        let expected = plan.steps.first().map(|s| s.action.clone()).unwrap_or_default();
        assert_eq!(plan.next_action(), expected);
    }

    #[test]
    fn test_record_verified_requires_coverage() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Ledger", &[]);
        plan.record_verified("claim", "test", "", "next step");
        assert!(plan.verified.is_empty(), "empty coverage must be rejected");
        plan.record_verified("claim", "test", "coverage", "next step");
        assert_eq!(plan.verified.len(), 1);
    }

    #[test]
    fn test_record_verified_rejects_heading_injection() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Ledger", &[]);
        plan.record_verified("## Verified", "test", "coverage", "next");
        assert!(plan.verified.is_empty(), "## prefix must be rejected");
        plan.record_verified("claim", "test", "coverage", "next");
        assert_eq!(plan.verified.len(), 1);
    }

    #[test]
    fn test_open_question_requires_settled_by_and_rejects_heading() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Ledger", &[]);
        plan.open_question("q", "");
        assert!(plan.open.is_empty(), "empty settled_by must be rejected");
        plan.open_question("## Open", "brute force");
        assert!(plan.open.is_empty(), "## prefix must be rejected");
        plan.open_question("q", "brute force n<=6");
        assert_eq!(plan.open.len(), 1);
    }

    #[test]
    fn test_note_core_requires_defining_fact() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Ledger", &[]);
        assert!(!plan.note_core("name only"), "mention without fact rejected");
        assert!(!plan.note_core(""), "empty rejected");
        assert!(plan.note_core("CAP — the invariant that gates overflow"));
        assert!(plan.note_core("greedy - the fallback comparator"));
        assert!(!plan.note_core("CAP — the invariant that gates overflow"), "dup rejected");
        assert_eq!(plan.core.len(), 2);
    }

    #[test]
    fn test_apply_notes_atomic_decline() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Ledger", &[]);
        let (accepted, refused) = plan.apply_notes(
            Some("done means x"),
            Some("next action"),
            &[("c1".into(), "t".into(), "cov".into(), "n1".into())],
            &[("q1".into(), "test".into())],
            &["mention without fact".into()],
        );
        assert!(accepted >= 4, "valid edits must be accepted, got {accepted}");
        assert_eq!(refused.len(), 1, "mention rejected atomically: {refused:?}");
        assert_eq!(plan.goal, "done means x");
        assert_eq!(plan.verified.len(), 2);
        assert_eq!(plan.open.len(), 1);
        assert_eq!(plan.core.len(), 0, "rejected core must not land");
    }

    #[test]
    fn test_reentry_anchor_prints_full_ledger() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Anchor", &[]);
        plan.note_core("A — fact");
        plan.record_verified("v1", "t", "cov", "n1");
        plan.open_question("q", "test");
        let anchor = plan.reentry_anchor();
        assert!(anchor.contains("Goal: Anchor"));
        assert!(anchor.contains("[live] A — fact"));
        assert!(anchor.contains("✓01 v1"));
        assert!(anchor.contains("?01 q"));
        assert!(anchor.contains("State the current pass"));
    }

    #[test]
    fn test_is_well_formed_rejects_dangling_close() {
        let gen = PlanGenerator::new();
        let mut plan = gen.generate_plan("Ledger", &[]);
        assert!(plan.is_well_formed());
        plan.record_verified("v1", "t", "cov", "n1");
        plan.open_question("q", "test");
        assert!(plan.is_well_formed());
        plan.open[0].closed_by = Some(5);
        assert!(!plan.is_well_formed(), "closed_by referencing missing checkpoint");
        plan.open[0].closed_by = None;
        plan.open[0].settled_by = String::new();
        assert!(!plan.is_well_formed(), "open without settled_by is damaged");
    }
}
