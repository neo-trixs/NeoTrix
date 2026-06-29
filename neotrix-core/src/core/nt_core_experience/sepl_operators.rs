/// SEPL 算子形式化代数 — Self-Evolution Protocol Layer 5 算子
/// ρ Reflect → σ Select → ι Improve → ε Evaluate → κ Commit
use std::collections::HashSet;

// ─── 共享状态 ────────────────────────────────────────────────

/// SEPL 管线共享上下文，每个 cycle 由调用方填充
#[derive(Debug, Clone)]
pub struct SeplContext {
    pub cycle: u64,
    pub meta_accuracy: f64,
    pub ece: f64,
    pub composite_loss: f64,
    pub pending_tasks: usize,
    pub arch_gaps: Vec<String>,
}

impl Default for SeplContext {
    fn default() -> Self {
        Self {
            cycle: 0,
            meta_accuracy: 0.0,
            ece: 0.0,
            composite_loss: 0.0,
            pending_tasks: 0,
            arch_gaps: vec![],
        }
    }
}

// ─── ρ Reflect 输出 ─────────────────────────────────────────

/// 反射假设 — 从校准、架构、元认知维度生成的进化候选
#[derive(Debug, Clone)]
pub struct SeplHypothesis {
    pub id: String,
    pub description: String,
    pub source: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

// ─── σ Select → ι Improve 输出 ──────────────────────────────

/// 进化提议 — 可执行的改进操作
#[derive(Debug, Clone)]
pub struct SeplProposal {
    pub id: String,
    pub hypothesis_id: String,
    pub title: String,
    pub description: String,
    pub impact: f64,
    pub risk: f64,
    pub priority: u8,
    pub target_module: String,
    pub lineage: Vec<String>,
    pub status: ProposalStatus,
}

/// 提议生命周期状态
#[derive(Debug, Clone, PartialEq)]
pub enum ProposalStatus {
    Draft,
    Selected,
    Improving,
    Evaluated,
    Committed,
    Rejected,
    RolledBack,
}

// ─── ε Evaluate 输出 ────────────────────────────────────────

/// 评估分数 — 多维加权评价
#[derive(Debug, Clone)]
pub struct SeplScore {
    pub proposal_id: String,
    pub meta_acc_score: f64,
    pub ece_score: f64,
    pub proposal_quality: f64,
    pub composite: f64,
    pub passed: bool,
}

// ─── κ Commit 输出 ──────────────────────────────────────────

/// Commit 报告 — 闭合一次进化迭代
#[derive(Debug, Clone)]
pub struct SeplReport {
    pub committed_ids: Vec<String>,
    pub rejected_ids: Vec<String>,
    pub rollback_ids: Vec<String>,
    pub timestamp: u64,
}

// ═══════════════════════════════════════════════════════════════
// TRAITS
// ═══════════════════════════════════════════════════════════════

/// ρ Reflect — 从上下文（校准、架构、元认知）生成进化假设
pub trait SeplReflect {
    fn reflect(&mut self, ctx: &SeplContext) -> Vec<SeplHypothesis>;
    fn name(&self) -> &str {
        "Reflect"
    }
}

/// σ Select — 从假设集中选择最高优先级的候选
pub trait SeplSelect {
    fn select(&mut self, hypotheses: Vec<SeplHypothesis>, ctx: &SeplContext)
        -> Vec<SeplHypothesis>;
    fn name(&self) -> &str {
        "Select"
    }
}

/// ι Improve — 将假设转换为可执行的进化提议
pub trait SeplImprove {
    fn improve(&mut self, hypotheses: Vec<SeplHypothesis>, ctx: &SeplContext) -> Vec<SeplProposal>;
    fn name(&self) -> &str {
        "Improve"
    }
}

/// ε Evaluate — 对提议进行多维评分
pub trait SeplEvaluate {
    fn evaluate(&mut self, proposals: Vec<SeplProposal>, ctx: &SeplContext) -> Vec<SeplScore>;
    fn name(&self) -> &str {
        "Evaluate"
    }
}

/// κ Commit — 根据评分决定提交/拒绝/回滚
pub trait SeplCommit {
    fn commit(&mut self, scores: Vec<SeplScore>, ctx: &SeplContext) -> SeplReport;
    fn name(&self) -> &str {
        "Commit"
    }
}

// ═══════════════════════════════════════════════════════════════
// DEFAULT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════

// ─── HypothesisEngine (ρ Reflect) ────────────────────────────

pub struct HypothesisEngine {
    pub hypotheses: Vec<SeplHypothesis>,
    pub seq: u64,
}

impl HypothesisEngine {
    pub fn new() -> Self {
        Self {
            hypotheses: vec![],
            seq: 0,
        }
    }

    fn next_id(&mut self) -> String {
        self.seq += 1;
        format!("hyp_{}", self.seq)
    }
}

impl SeplReflect for HypothesisEngine {
    fn reflect(&mut self, ctx: &SeplContext) -> Vec<SeplHypothesis> {
        let mut results: Vec<SeplHypothesis> = vec![];

        if ctx.ece > 0.15 {
            results.push(SeplHypothesis {
                id: self.next_id(),
                description: format!("ECE drift detected: {:.4}", ctx.ece),
                source: "calibration_drift".to_string(),
                confidence: (1.0 - ctx.ece).clamp(0.0, 1.0),
                evidence: vec![format!("ECE={:.4} above threshold 0.15", ctx.ece)],
            });
        }

        if ctx.meta_accuracy < 0.7 {
            results.push(SeplHypothesis {
                id: self.next_id(),
                description: format!("Meta-accuracy degradation: {:.4}", ctx.meta_accuracy),
                source: "meta_degradation".to_string(),
                confidence: ctx.meta_accuracy,
                evidence: vec![format!("meta_acc={:.4} below 0.7", ctx.meta_accuracy)],
            });
        }

        for gap in &ctx.arch_gaps {
            results.push(SeplHypothesis {
                id: self.next_id(),
                description: format!("Architecture gap: {}", gap),
                source: "arch_gap".to_string(),
                confidence: 0.6,
                evidence: vec![format!("arch_gap: {}", gap)],
            });
        }

        if ctx.pending_tasks > 5 {
            results.push(SeplHypothesis {
                id: self.next_id(),
                description: format!("Task backlog growing: {} pending", ctx.pending_tasks),
                source: "task_feedback".to_string(),
                confidence: 0.5,
                evidence: vec![format!("pending_tasks={} exceeds 5", ctx.pending_tasks)],
            });
        }

        self.hypotheses = results.clone();
        results
    }
}

// ─── SelectionEngine (σ Select) ─────────────────────────────

pub struct SelectionEngine {
    pub max_hypotheses: usize,
}

impl SelectionEngine {
    pub fn new(max_hypotheses: usize) -> Self {
        Self { max_hypotheses }
    }
}

impl Default for SelectionEngine {
    fn default() -> Self {
        Self::new(5)
    }
}

impl SeplSelect for SelectionEngine {
    fn select(
        &mut self,
        mut hypotheses: Vec<SeplHypothesis>,
        _ctx: &SeplContext,
    ) -> Vec<SeplHypothesis> {
        // 按 confidence 降序排序
        hypotheses.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 多样性去重：如果两个假设有相似的 description，保留高 confidence 的
        let mut kept: Vec<SeplHypothesis> = vec![];
        let mut seen_descriptions: HashSet<String> = HashSet::new();

        for h in hypotheses {
            let normalized = h.description.to_lowercase();
            let is_duplicate = seen_descriptions.iter().any(|existing| {
                existing.starts_with(&normalized) || normalized.starts_with(existing)
            });
            if !is_duplicate {
                seen_descriptions.insert(normalized);
                kept.push(h);
            }
        }

        kept.truncate(self.max_hypotheses);
        kept
    }
}

// ─── ImprovementEngine (ι Improve) ─────────────────────────

pub struct ImprovementEngine {
    pub proposals: Vec<SeplProposal>,
    pub seq: u64,
}

impl ImprovementEngine {
    pub fn new() -> Self {
        Self {
            proposals: vec![],
            seq: 0,
        }
    }

    fn next_id(&mut self) -> String {
        self.seq += 1;
        format!("prop_{}", self.seq)
    }
}

impl SeplImprove for ImprovementEngine {
    fn improve(
        &mut self,
        hypotheses: Vec<SeplHypothesis>,
        _ctx: &SeplContext,
    ) -> Vec<SeplProposal> {
        let mut results: Vec<SeplProposal> = vec![];

        for h in hypotheses {
            let impact = (h.confidence * 0.8).clamp(0.0, 1.0);
            let risk = (1.0 - h.confidence).clamp(0.0, 1.0);
            let priority = (impact * 10.0).round() as u8;

            let target_module = match h.source.as_str() {
                "calibration_drift" => "calibration_engine".to_string(),
                "arch_gap" => "self_arch_audit".to_string(),
                "meta_degradation" => "meta_cognitive_loop".to_string(),
                "task_feedback" => "evolution_task_system".to_string(),
                _ => "general".to_string(),
            };

            let proposal_id = self.next_id();
            results.push(SeplProposal {
                id: proposal_id.clone(),
                hypothesis_id: h.id,
                title: format!(
                    "SEPL-{}: {}",
                    proposal_id,
                    &h.description[..h.description.len().min(60)]
                ),
                description: h.description,
                impact,
                risk,
                priority,
                target_module,
                lineage: vec![],
                status: ProposalStatus::Draft,
            });
        }

        self.proposals = results.clone();
        results
    }
}

// ─── EvaluationEngine (ε Evaluate) ─────────────────────────

pub struct EvaluationEngine;

impl EvaluationEngine {
    pub fn new() -> Self {
        Self
    }
}

impl SeplEvaluate for EvaluationEngine {
    fn evaluate(&mut self, proposals: Vec<SeplProposal>, ctx: &SeplContext) -> Vec<SeplScore> {
        let mut results: Vec<SeplScore> = vec![];

        for proposal in proposals {
            let meta_acc_score = ctx.meta_accuracy.clamp(0.0, 1.0);
            let ece_score = (1.0 - ctx.ece).clamp(0.0, 1.0);
            let proposal_quality = if proposal.risk > 0.0 {
                (proposal.impact / proposal.risk).clamp(0.0, 1.0)
            } else {
                proposal.impact.clamp(0.0, 1.0)
            };
            let composite = meta_acc_score * 0.4 + ece_score * 0.3 + proposal_quality * 0.3;

            results.push(SeplScore {
                proposal_id: proposal.id,
                meta_acc_score,
                ece_score,
                proposal_quality,
                composite,
                passed: composite >= 0.4,
            });
        }

        results
    }
}

// ─── CommitGate (κ Commit) ─────────────────────────────────

pub struct CommitGate {
    pub committed: Vec<String>,
    pub rejected: Vec<String>,
}

impl CommitGate {
    pub fn new() -> Self {
        Self {
            committed: vec![],
            rejected: vec![],
        }
    }
}

impl SeplCommit for CommitGate {
    fn commit(&mut self, scores: Vec<SeplScore>, _ctx: &SeplContext) -> SeplReport {
        let mut report = SeplReport {
            committed_ids: vec![],
            rejected_ids: vec![],
            rollback_ids: vec![],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        for score in scores {
            if score.passed {
                self.committed.push(score.proposal_id.clone());
                report.committed_ids.push(score.proposal_id);
            } else {
                self.rejected.push(score.proposal_id.clone());
                report.rejected_ids.push(score.proposal_id);
            }
        }

        report
    }
}

// ═══════════════════════════════════════════════════════════════
// SEPL PIPELINE
// ═══════════════════════════════════════════════════════════════

/// SEPL 管线 — 组合 5 算子为完整进化迭代
pub struct SeplPipeline {
    pub reflector: HypothesisEngine,
    pub selector: SelectionEngine,
    pub improver: ImprovementEngine,
    pub evaluator: EvaluationEngine,
    pub committer: CommitGate,
}

impl SeplPipeline {
    pub fn new() -> Self {
        Self {
            reflector: HypothesisEngine::new(),
            selector: SelectionEngine::default(),
            improver: ImprovementEngine::new(),
            evaluator: EvaluationEngine::new(),
            committer: CommitGate::new(),
        }
    }

    /// ρ → σ → ι → ε → κ 完整闭环
    pub fn run(&mut self, ctx: &SeplContext) -> SeplReport {
        let hypotheses = self.reflector.reflect(ctx);
        let selected = self.selector.select(hypotheses, ctx);
        let proposals = self.improver.improve(selected, ctx);
        let scores = self.evaluator.evaluate(proposals, ctx);
        self.committer.commit(scores, ctx)
    }

    pub fn summary(&self) -> String {
        format!(
            "SEPL: {}→{}→{}→{}→{}",
            self.reflector.name(),
            self.selector.name(),
            self.improver.name(),
            self.evaluator.name(),
            self.committer.name(),
        )
    }
}

// ═══════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── HypothesisEngine tests ──────────────────────────────

    #[test]
    fn test_reflect_on_high_ece() {
        let mut engine = HypothesisEngine::new();
        let ctx = SeplContext {
            ece: 0.25,
            ..SeplContext::default()
        };
        let results = engine.reflect(&ctx);
        assert!(
            !results.is_empty(),
            "should generate hypotheses from high ECE"
        );
        let has_ece = results.iter().any(|h| h.source == "calibration_drift");
        assert!(has_ece, "should have a calibration_drift hypothesis");
    }

    #[test]
    fn test_reflect_on_low_meta_accuracy() {
        let mut engine = HypothesisEngine::new();
        let ctx = SeplContext {
            meta_accuracy: 0.45,
            ..SeplContext::default()
        };
        let results = engine.reflect(&ctx);
        let has_meta = results.iter().any(|h| h.source == "meta_degradation");
        assert!(has_meta, "should have a meta_degradation hypothesis");
    }

    #[test]
    fn test_reflect_on_arch_gaps() {
        let mut engine = HypothesisEngine::new();
        let ctx = SeplContext {
            arch_gaps: vec!["h4_rag".to_string(), "lila_e8".to_string()],
            ..SeplContext::default()
        };
        let results = engine.reflect(&ctx);
        let gap_count = results.iter().filter(|h| h.source == "arch_gap").count();
        assert_eq!(gap_count, 2, "should generate one hypothesis per arch gap");
    }

    // ── SelectionEngine tests ──────────────────────────────

    #[test]
    fn test_select_limits_by_max_hypotheses() {
        let mut engine = SelectionEngine::new(2);
        let hypotheses = vec![
            SeplHypothesis {
                id: "h1".into(),
                description: "a".into(),
                source: "arch_gap".into(),
                confidence: 0.9,
                evidence: vec![],
            },
            SeplHypothesis {
                id: "h2".into(),
                description: "b".into(),
                source: "arch_gap".into(),
                confidence: 0.8,
                evidence: vec![],
            },
            SeplHypothesis {
                id: "h3".into(),
                description: "c".into(),
                source: "arch_gap".into(),
                confidence: 0.7,
                evidence: vec![],
            },
        ];
        let ctx = SeplContext::default();
        let selected = engine.select(hypotheses, &ctx);
        assert_eq!(selected.len(), 2, "should keep at most max_hypotheses=2");
        assert_eq!(selected[0].id, "h1", "should keep highest confidence first");
    }

    #[test]
    fn test_select_diversity_dedup() {
        let mut engine = SelectionEngine::new(5);
        let hypotheses = vec![
            SeplHypothesis {
                id: "h1".into(),
                description: "ECE drift detected".into(),
                source: "calibration_drift".into(),
                confidence: 0.8,
                evidence: vec![],
            },
            SeplHypothesis {
                id: "h2".into(),
                description: "ECE drift detected: high".into(),
                source: "calibration_drift".into(),
                confidence: 0.9,
                evidence: vec![],
            },
        ];
        let ctx = SeplContext::default();
        let selected = engine.select(hypotheses, &ctx);
        assert_eq!(selected.len(), 1, "should dedup similar descriptions");
        assert_eq!(selected[0].id, "h2", "should keep higher confidence");
    }

    #[test]
    fn test_select_empty_input() {
        let mut engine = SelectionEngine::new(5);
        let ctx = SeplContext::default();
        let selected = engine.select(vec![], &ctx);
        assert!(
            selected.is_empty(),
            "empty input should produce empty output"
        );
    }

    // ── ImprovementEngine tests ────────────────────────────

    #[test]
    fn test_improve_calibration_drift_target() {
        let mut engine = ImprovementEngine::new();
        let hypotheses = vec![SeplHypothesis {
            id: "hyp_1".into(),
            description: "ECE drift".into(),
            source: "calibration_drift".into(),
            confidence: 0.7,
            evidence: vec![],
        }];
        let ctx = SeplContext::default();
        let proposals = engine.improve(hypotheses, &ctx);
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].target_module, "calibration_engine");
    }

    #[test]
    fn test_improve_arch_gap_target() {
        let mut engine = ImprovementEngine::new();
        let hypotheses = vec![SeplHypothesis {
            id: "hyp_1".into(),
            description: "Missing module".into(),
            source: "arch_gap".into(),
            confidence: 0.6,
            evidence: vec![],
        }];
        let ctx = SeplContext::default();
        let proposals = engine.improve(hypotheses, &ctx);
        assert_eq!(proposals[0].target_module, "self_arch_audit");
    }

    #[test]
    fn test_improve_priority_scales_with_impact() {
        let mut engine = ImprovementEngine::new();
        let hypotheses = vec![
            SeplHypothesis {
                id: "hyp_1".into(),
                description: "High confidence".into(),
                source: "calibration_drift".into(),
                confidence: 0.95,
                evidence: vec![],
            },
            SeplHypothesis {
                id: "hyp_2".into(),
                description: "Low confidence".into(),
                source: "arch_gap".into(),
                confidence: 0.2,
                evidence: vec![],
            },
        ];
        let ctx = SeplContext::default();
        let proposals = engine.improve(hypotheses, &ctx);
        assert!(
            proposals[0].priority >= proposals[1].priority,
            "higher confidence should yield higher priority"
        );
    }

    #[test]
    fn test_improve_empty_hypotheses() {
        let mut engine = ImprovementEngine::new();
        let ctx = SeplContext::default();
        let proposals = engine.improve(vec![], &ctx);
        assert!(proposals.is_empty());
    }

    // ── EvaluationEngine tests ─────────────────────────────

    #[test]
    fn test_evaluate_passes_above_threshold() {
        let mut engine = EvaluationEngine::new();
        let proposals = vec![SeplProposal {
            id: "prop_1".into(),
            hypothesis_id: "hyp_1".into(),
            title: "test".into(),
            description: "fix ECE".into(),
            impact: 0.8,
            risk: 0.2,
            priority: 8,
            target_module: "calibration_engine".into(),
            lineage: vec![],
            status: ProposalStatus::Draft,
        }];
        let ctx = SeplContext {
            meta_accuracy: 0.8,
            ece: 0.1,
            ..SeplContext::default()
        };
        let scores = engine.evaluate(proposals, &ctx);
        assert_eq!(scores.len(), 1);
        assert!(scores[0].passed, "should pass with good metrics");
        assert!(scores[0].composite >= 0.4);
    }

    #[test]
    fn test_evaluate_rejects_below_threshold() {
        let mut engine = EvaluationEngine::new();
        let proposals = vec![SeplProposal {
            id: "prop_1".into(),
            hypothesis_id: "hyp_1".into(),
            title: "bad".into(),
            description: "bad proposal".into(),
            impact: 0.1,
            risk: 0.9,
            priority: 1,
            target_module: "general".into(),
            lineage: vec![],
            status: ProposalStatus::Draft,
        }];
        let ctx = SeplContext {
            meta_accuracy: 0.2,
            ece: 0.8,
            ..SeplContext::default()
        };
        let scores = engine.evaluate(proposals, &ctx);
        assert!(!scores[0].passed, "should fail with poor metrics");
    }

    #[test]
    fn test_evaluate_composite_weighting() {
        let mut engine = EvaluationEngine::new();
        let proposals = vec![SeplProposal {
            id: "prop_1".into(),
            hypothesis_id: "hyp_1".into(),
            title: "test".into(),
            description: "test".into(),
            impact: 0.5,
            risk: 0.5,
            priority: 5,
            target_module: "general".into(),
            lineage: vec![],
            status: ProposalStatus::Draft,
        }];
        let ctx = SeplContext {
            meta_accuracy: 0.6,
            ece: 0.2,
            ..SeplContext::default()
        };
        let scores = engine.evaluate(proposals, &ctx);
        // meta_acc=0.6*0.4=0.24, ece_score=(1-0.2)=0.8*0.3=0.24, quality=(0.5/0.5)=1.0*0.3=0.3
        // composite=0.24+0.24+0.3=0.78
        let expected = 0.6 * 0.4 + (1.0 - 0.2) * 0.3 + (0.5_f64 / 0.5).clamp(0.0, 1.0) * 0.3;
        assert!((scores[0].composite - expected).abs() < 1e-6);
    }

    // ── CommitGate tests ──────────────────────────────────

    #[test]
    fn test_commit_passed_proposals() {
        let mut gate = CommitGate::new();
        let scores = vec![
            SeplScore {
                proposal_id: "p1".into(),
                meta_acc_score: 0.8,
                ece_score: 0.8,
                proposal_quality: 0.8,
                composite: 0.8,
                passed: true,
            },
            SeplScore {
                proposal_id: "p2".into(),
                meta_acc_score: 0.2,
                ece_score: 0.2,
                proposal_quality: 0.2,
                composite: 0.2,
                passed: false,
            },
        ];
        let ctx = SeplContext::default();
        let report = gate.commit(scores, &ctx);
        assert_eq!(report.committed_ids, vec!["p1"]);
        assert_eq!(report.rejected_ids, vec!["p2"]);
    }

    #[test]
    fn test_commit_all_fail() {
        let mut gate = CommitGate::new();
        let scores = vec![SeplScore {
            proposal_id: "p1".into(),
            composite: 0.1,
            passed: false,
            ..SeplScore {
                proposal_id: "p1".into(),
                meta_acc_score: 0.0,
                ece_score: 0.0,
                proposal_quality: 0.0,
                composite: 0.1,
                passed: false,
            }
        }];
        let ctx = SeplContext::default();
        let report = gate.commit(scores, &ctx);
        assert!(report.committed_ids.is_empty());
        assert_eq!(report.rejected_ids.len(), 1);
    }

    #[test]
    fn test_commit_empty_scores() {
        let mut gate = CommitGate::new();
        let ctx = SeplContext::default();
        let report = gate.commit(vec![], &ctx);
        assert!(report.committed_ids.is_empty());
        assert!(report.rejected_ids.is_empty());
    }

    // ── SeplPipeline tests ────────────────────────────────

    #[test]
    fn test_pipeline_full_run() {
        let mut pipeline = SeplPipeline::new();
        let ctx = SeplContext {
            ece: 0.25,
            meta_accuracy: 0.5,
            arch_gaps: vec!["test_gap".into()],
            ..SeplContext::default()
        };
        let report = pipeline.run(&ctx);
        // ECE>0.15 → 1 hyp, meta_acc<0.7 → 1 hyp, 1 arch_gap → 1 hyp
        // total 3 hypotheses → after diversity = 3. Each → 1 proposal. Eval → potentially pass 3.
        assert!(!report.committed_ids.is_empty() || !report.rejected_ids.is_empty());
    }

    #[test]
    fn test_pipeline_summary() {
        let pipeline = SeplPipeline::new();
        let s = pipeline.summary();
        assert_eq!(s, "SEPL: Reflect→Select→Improve→Evaluate→Commit");
    }
}
