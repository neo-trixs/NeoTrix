//! nt_credit_stages — 从 `pipeline.rs` 拆分 (credit_stages — 信用/奖励族: 信用分配/步级审计+分歧/预言门/奖励计算/收敛检查/自测/意识奖励 (行为零变更).)
//! 原文逐行搬运, 仅补可见性/导入, 行为零变更。

use super::nt_types::*;
use super::super::SelfIteratingBrain;
use crate::neotrix::nt_core_error::NeoTrixError;
use crate::make_stage;
use crate::l5_cognition::nt_mind::nt_mind::seal_core::core::PerformanceEvaluator;
use crate::l5_cognition::nt_mind::nt_mind::seal_core::core::ExecutionFeedback;
use crate::l5_cognition::nt_mind::foundation::seal_pipeline::L1OracleGate;
use crate::l5_cognition::nt_mind::foundation::seal_pipeline::L1SemanticEntropyGate;
use crate::l5_cognition::nt_mind::foundation::seal_pipeline::L1ActionSandbox;
use crate::l5_cognition::nt_mind::nt_mind::consciousness::consciousness_bridge::ConsciousnessBridge;
use crate::l5_cognition::nt_mind::nt_mind::consciousness::bbrain_monitor::BMonitor;

pub struct _CreditAssignmentStage;
impl Default for _CreditAssignmentStage {
    fn default() -> Self {
        Self
    }
}
impl _CreditAssignmentStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _CreditAssignmentStage {
    fn name(&self) -> &str {
        "credit_assignment"
    }
    fn frequency(&self) -> usize {
        20
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        // Build credit graph from PRM step rewards + E8 transitions
        let mut graph = crate::l5_cognition::nt_core_credit::CreditGraph::new();
        let policy = crate::l5_cognition::nt_core_credit::E8CreditPolicy::default();
        let mut visit_counts: std::collections::HashMap<u8, u64> = std::collections::HashMap::new();

        for (step, reward) in &brain._prm_step_rewards {
            let e8_state = (step % 64) as u8;
            let visit = visit_counts.entry(e8_state).or_insert(0);
            *visit += 1;
            let attribution = policy
                .compute_attribution(brain._prm_step_rewards.len().saturating_sub(*step), *visit);
            graph.add_event(crate::l5_cognition::nt_core_credit::CreditEvent {
                id: format!("prm_step_{}", step),
                parent_id: if *step > 0 {
                    Some(format!("prm_step_{}", step - 1))
                } else {
                    None
                },
                role: if *reward > 0.5 {
                    crate::l5_cognition::nt_core_credit::CreditRole::Outcome
                } else {
                    crate::l5_cognition::nt_core_credit::CreditRole::Actor
                },
                label: format!("step_{}_reward_{:.2}", step, reward),
                e8_state,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
                weight: *reward,
                metadata: std::collections::HashMap::new(),
            });
            if *step > 0 {
                graph.add_edge(crate::l5_cognition::nt_core_credit::CreditEdge {
                    from: format!("prm_step_{}", step - 1),
                    to: format!("prm_step_{}", step),
                    attribution,
                    discount: policy.step_discount,
                });
            }
        }

        let credits = graph.backpropagate(0.95);
        if !credits.is_empty() {
            let total: f64 = credits.values().sum();
            log::debug!(
                "[credit_assignment] {} events, total credit={:.3}",
                credits.len(),
                total
            );
            // Persist to KB
            if let Some(ref kb) = brain._nt_memory_kb {
                if let Ok(json) = graph.to_json() {
                    let _ = kb.kv_set("credit_graph", "latest", &json);
                    let _ = kb.kv_set("credit_graph", &format!("iter_{}", brain.iteration), &json);
                }
            }
            // GWT broadcast
            if let Some(ref mut engine) = brain.reasoning_engine {
                if let Some(ref mut gwt) = engine.gwt {
                    gwt.broadcast(&format!(
                        "credit_assignment: {} events, total={:.3}, persisted",
                        credits.len(),
                        total
                    ));
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}


/// W3.6 (batch3 2026-08-26, 源: arxiv 2608.19760 *Credit Without Ground Truth*)
/// 步级信用审计 — 无真值下交叉比对两条独立信用信号:
///   A. 折扣回传累积信用 (与 _CreditAssignmentStage 同口径, discount=0.95)
///   B. PRM 即时奖励 (raw reward)
/// 分歧步 = 可解释审计发现: 低即时高回传 = unsung_hero (铺垫步);
/// 高即时负回传 = lucky_start (透支未来); 附结构性发现 (e8_state 合成值)。
/// 输出持久化 KB `credit_audit/{latest,iter_N}` — 消费者: 自改进循环判定面。
pub struct StepCreditAuditStage;
impl Default for StepCreditAuditStage {
    fn default() -> Self {
        Self
    }
}
impl StepCreditAuditStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for StepCreditAuditStage {
    fn name(&self) -> &str {
        "step_credit_audit"
    }
    fn frequency(&self) -> usize {
        20
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain._prm_step_rewards.is_empty() {
            return Ok(StageDecision::Continue);
        }
        let rewards: Vec<(usize, f64)> = brain._prm_step_rewards.clone();
        let report = compute_credit_divergence(&rewards, 0.95);

        // 结构性发现: _CreditAssignmentStage 的 e8_state=(step%64) 为合成值
        // (非真实 E8 转移), 信用图的 E8 维度不可解释 — 审计首靶点留痕。
        let mut findings = vec![
            "e8_state=(step%64) 为合成值非真实转移 — E8 维度信用归因不可解释".to_string(),
        ];
        for d in &report.divergent_steps {
            findings.push(format!(
                "step_{} {} divergence={:.3} (backprop_z={:.2}, raw_z={:.2})",
                d.step_idx, d.label, d.divergence, d.backprop_z, d.raw_z
            ));
        }

        let summary = format!(
            "step_credit_audit: {} steps audited, {} divergent (>{:.2}), max_divergence={:.3}",
            report.steps_audited,
            report.divergent_steps.len(),
            DIVERGENCE_THRESHOLD,
            report.max_divergence
        );
        log::info!("[{}] {}", self.name(), summary);
        if let Some(ref kb) = brain._nt_memory_kb {
            let json = serde_json::json!({
                "steps_audited": report.steps_audited,
                "max_divergence": report.max_divergence,
                "divergent_steps": report.divergent_steps.iter().map(|d| serde_json::json!({
                    "step_idx": d.step_idx,
                    "label": d.label,
                    "divergence": d.divergence,
                    "backprop_z": d.backprop_z,
                    "raw_z": d.raw_z,
                })).collect::<Vec<_>>(),
                "structural_findings": findings,
                "audited_at": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            });
            let _ = kb.kv_set("credit_audit", "latest", json.to_string().as_str());
            let _ = kb.kv_set(
                "credit_audit",
                &format!("iter_{}", brain.iteration),
                json.to_string().as_str(),
            );
        }
        Ok(StageDecision::Continue)
    }
}

/// 分歧判定阈值 (z 分数差)
pub const DIVERGENCE_THRESHOLD: f64 = 1.0;

/// 单步分歧发现
#[derive(Debug, Clone, PartialEq)]
pub struct _StepDivergence {
    pub step_idx: usize,
    /// unsung_hero = 低即时/高回传; lucky_start = 高即时/低或负回传
    pub label: &'static str,
    pub divergence: f64,
    pub backprop_z: f64,
    pub raw_z: f64,
}

/// 步级信用分歧审计报告
#[derive(Debug, Clone, PartialEq)]
pub struct _CreditAuditReport {
    pub steps_audited: usize,
    pub max_divergence: f64,
    pub divergent_steps: Vec<_StepDivergence>,
}

/// 纯计算: 折扣回传信用 vs 即时奖励的 z 分数分歧 (无真值审计核心)。
pub fn compute_credit_divergence(
    rewards: &[(usize, f64)],
    discount: f64,
) -> _CreditAuditReport {
    let n = rewards.len();
    // 常值奖励 ⇒ 无信用分歧可言: raw z 分母退化归零, 几何回传坡度纯属折现伪影
    if rewards.windows(2).all(|w| (w[0].1 - w[1].1).abs() < 1e-12) {
        return _CreditAuditReport {
            steps_audited: n,
            max_divergence: 0.0,
            divergent_steps: Vec::new(),
        };
    }
    let mut backprop = vec![0.0f64; n];
    // 回传: credit[i] = r[i] + discount * credit[i+1]
    for i in (0..n).rev() {
        backprop[i] = rewards[i].1 + discount * backprop.get(i + 1).copied().unwrap_or(0.0);
    }
    let z = |v: &[f64]| -> Vec<f64> {
        let mean = v.iter().sum::<f64>() / n as f64;
        let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
        let sd = var.sqrt().max(1e-9);
        v.iter().map(|x| (x - mean) / sd).collect()
    };
    let bz = z(&backprop);
    let rz: Vec<f64> = rewards.iter().map(|(_, r)| *r).collect::<Vec<_>>();
    let rz = z(&rz);
    let mut out = _CreditAuditReport {
        steps_audited: n,
        max_divergence: 0.0,
        divergent_steps: Vec::new(),
    };
    for i in 0..n {
        let div = bz[i] - rz[i];
        out.max_divergence = out.max_divergence.max(div.abs());
        if div.abs() > DIVERGENCE_THRESHOLD {
            let label = if div > 0.0 { "unsung_hero" } else { "lucky_start" };
            out.divergent_steps.push(_StepDivergence {
                step_idx: rewards[i].0,
                label,
                divergence: div,
                backprop_z: bz[i],
                raw_z: rz[i],
            });
        }
    }
    out.divergent_steps.sort_by(|a, b| {
        b.divergence
            .abs()
            .partial_cmp(&a.divergence.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}


/// OracleGate stage: evaluates pipeline health to decide if external human
/// oracle intervention is needed. Frequency 10 — low overhead gate that
/// only triggers under critical conditions (high entropy, low reward).
pub struct _OracleGateStage;
impl Default for _OracleGateStage {
    fn default() -> Self {
        Self
    }
}
impl _OracleGateStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _OracleGateStage {
    fn name(&self) -> &str {
        "oracle_gate"
    }
    fn frequency(&self) -> usize {
        10
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let gate = brain._oracle_gate.get_or_insert_with(crate::l1_action::nt_act::nt_act_autonomy::OracleGate::new);

        let entropy = brain.entropy_crisis_level;
        let reward = brain._reward;

        let decision = if entropy > 0.7 && reward < 0.2 {
            gate.evaluate_failure(brain.iteration as u32, "entropy_crisis")
        } else {
            gate.evaluate_failure(0, "normal")
        };

        if decision.needs_oracle {
            log::warn!(
                "[oracle_gate] HUMAN INTERVENTION: {:?} — {}",
                decision.reason,
                decision.suggested_action
            );
            if let Some(ref kb) = brain._nt_memory_kb {
                if let Some(ref req) = decision.request {
                    let _ = kb.kv_set(
                        "oracle_request",
                        &format!("iter_{}", brain.iteration),
                        &serde_json::json!({"reason": format!("{:?}", req.reason)}).to_string(),
                    );
                }
            }
            if let Some(ref mut engine) = brain.reasoning_engine {
                if let Some(ref mut gwt) = engine.gwt {
                    gwt.broadcast("oracle_gate: needs intervention");
                }
            }
            return Ok(StageDecision::Skip(decision.suggested_action));
        }
        Ok(StageDecision::Continue)
    }
}


make_stage!(RewardCalculationStage);
impl BrainStage for RewardCalculationStage {
    fn name(&self) -> &str {
        "reward_calc"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let external = brain._external_reward();
        let (reward, source) = if let Some(ext) = external {
            // Q2 P3 情感引导: 若共享观测槽有情感候选 (数字人旁路事件发布), 经
            // combine_reward_with_affective 融合进外部奖励 (经 RewardSource::External)。
            // 负外部奖励**原样保留** (触发 snapshot restore rewind 语义, 不夹取到 0)。
            let blended = if ext >= 0.0 {
                let affective = crate::l2_perception::nt_core_knowledge::take_affective_observation();
                match affective {
                Some(a) => PerformanceEvaluator::combine_reward_with_affective(
                    brain._snapshot_score(),
                    ExecutionFeedback::new(true, 1.0, ext),
                        Some(a),
                        1.0,
                    ),
                    None => ext,
                }
            } else {
                ext
            };
            (blended, crate::l2_perception::nt_core_knowledge::types::RewardSource::External)
        } else {
            let task_type = brain._current_task_type();
            let score_before = brain._snapshot_score();
            let score_after = brain.brain.evaluate_capability(task_type);
            let regularization = brain.compute_regularization(&brain._snapshot_capability());
            let raw = (score_after - score_before) + regularization;
            let health = brain.evo_stats().health_score;
            let calibrated = raw * (0.5 + health * 0.5);
            (calibrated, crate::l2_perception::nt_core_knowledge::types::RewardSource::Internal)
        };
        brain._set_reward(reward);
        brain._set_reward_source(source);
        Ok(StageDecision::Continue)
    }
}


// ── _ConvergenceCheckStage ────────────────────────────────────
// Architecture self-audit: every 50 iterations, scan for ghost modules + orphan files.

pub struct _ConvergenceCheckStage;

impl Default for _ConvergenceCheckStage {
    fn default() -> Self {
        Self::new()
    }
}

impl _ConvergenceCheckStage {
    pub fn new() -> Self {
        Self
    }
}

impl BrainStage for _ConvergenceCheckStage {
    fn name(&self) -> &str {
        "convergence_check"
    }

    fn frequency(&self) -> usize {
        50
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let _ = brain; // unused: the audit runs on the source tree, not brain state
        use crate::l5_cognition::l1_facade::self_audit::converge_check;
        let report = converge_check(".");
        if !report.findings.is_empty() {
            log::warn!(
                "[seal] converge_check iter: {} ghosts, {} orphans, {} stale",
                report.ghost_count,
                report.stale_count,
                report.orphan_count
            );
        }
        Ok(StageDecision::Continue)
    }
}


// ── SelfTestStage ───────────────────────────────────────────
// Meta-audit: every 100 iterations, verify that detection modules themselves are intact.

pub struct SelfTestStage;

impl Default for SelfTestStage {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfTestStage {
    pub fn new() -> Self {
        Self
    }
}

impl BrainStage for SelfTestStage {
    fn name(&self) -> &str {
        "self_test"
    }

    fn frequency(&self) -> usize {
        100
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let _ = brain;
        use crate::l5_cognition::nt_core_consciousness::consciousness_runtime::ConsciousnessRuntime;
        use crate::l5_cognition::nt_core_consciousness::inner_critic::InnerCritic;
        use crate::l5_cognition::nt_core_consciousness_tree::ConsciousnessTree;
        use crate::l5_cognition::nt_core_gwt::monitor::EntropyMonitor;
        use crate::l5_cognition::l1_facade::knowledge_gap_detector::KnowledgeGapDetector;
        use crate::l5_cognition::l1_facade::metacognition_loop::MetaCognitiveLoop;
        use crate::l5_cognition::l1_facade::monitor::MetaMonitor;
        use crate::l5_cognition::l1_facade::nt_core_arch_lint::ArchLint;
        use crate::l5_cognition::l1_facade::nt_core_meta_auditor::MetaAuditor;
        use crate::l5_cognition::l1_facade::scanner::CodeScanner;
        use crate::l5_cognition::l1_facade::self_model::SelfModel;
        use crate::l0_substrate::nt_core_schema_watchdog::SchemaWatchdog;
        use crate::l5_cognition::l1_facade::metacognitive_evaluator::CognitiveEvaluator;
        use crate::l5_cognition::l1_facade::self_audit::ConvergeCheckFn;
        use crate::l5_cognition::l1_facade::SelfReviewGate;
        use crate::l0_substrate::nt_core_self_test::SelfTestRegistry;
        let mut registry = SelfTestRegistry::new();
        registry.register(Box::new(SchemaWatchdog::new()));
        registry.register(Box::new(ConvergeCheckFn));
        registry.register(Box::new(KnowledgeGapDetector::new()));
        registry.register(Box::new(CodeScanner::new(".")));
        registry.register(Box::new(EntropyMonitor::new(10, 0.5, 3)));
        registry.register(Box::new(BMonitor::default()));
        registry.register(Box::new(InnerCritic::new()));
        registry.register(Box::new(ConsciousnessRuntime::new()));
        registry.register(Box::new(crate::l5_cognition::nt_core_consciousness::ConsciousnessAwakening));
        registry.register(Box::new(SelfReviewGate::new(false)));
        registry.register(Box::new(ConsciousnessTree::new()));
        registry.register(Box::new(MetaAuditor::new()));
        registry.register(Box::new(ArchLint::new()));
        let sm = SelfModel::new();
        registry.register(Box::new(MetaMonitor::new(sm.clone())));
        registry.register(Box::new(MetaCognitiveLoop::new(sm)));
        // CognitiveLoadMonitor doesn't implement SelfTest, skip registration
        registry.register(Box::new(CognitiveEvaluator::new()));
        registry.register(Box::new({
            let mut cm = crate::l5_cognition::l1_facade::ConsciousnessMonitor::new();
            cm.observe();
            cm
        }));
        registry.register(Box::new(
            crate::l5_cognition::nt_mind::evolution::self_diagnose::SelfDiagnose,
        ));
        registry.register(Box::new(
            crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_svaf_gate::SvafGate::default(),
        ));
        registry.register(Box::new(
            crate::l5_cognition::nt_core::capability::nt_core_antidistil::DistillationDetector::new(),
        ));
        registry.register(Box::new(L1OracleGate::new()));
        registry.register(Box::new(L1SemanticEntropyGate::new()));
        registry.register(Box::new(L1ActionSandbox::new()));
        registry.register(Box::new(
            crate::l5_cognition::nt_core_consciousness_tree::review::ConsciousnessReview::new(),
        ));
        // nt_core_fep_iit module not found - removed
        // registry.register(Box::new(
        //     crate::l4_emotion::nt_feel::nt_core_fep_iit::bridge::FEPIITBridge::new(),
        // ));
        registry.register(Box::new(crate::l5_cognition::l1_facade::ConsciousnessGoldStandard::new()));
        registry.register(Box::new(ConsciousnessBridge::new()));
        registry.register(crate::l3_embodiment::nt_shield::shield_core::browser_security::create_browser_security_self_test());
        registry.register(crate::l3_embodiment::nt_shield::shield_core::check_registry::create_check_registry_self_test());
        registry.register(Box::new(
            crate::l0_substrate::nt_core_telemetry::TelemetryStore::new(100),
        ));
        registry.register(Box::new(crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_commit_tracker::NarrativeConsistencyChecker::new()));
        registry.register(Box::new(
            crate::l5_cognition::nt_core_scoring_substrate::ScoringSubstrate::new().with_threshold(0.5),
        ));
        registry.register(Box::new(
            crate::l5_cognition::nt_core::nt_state_substrate::StateSubstrate::new(),
        ));
        registry.register(Box::new(
            crate::l1_action::nt_core_simulate_engine::SimulateEngine::new(),
        ));
        registry.register(Box::new(
            crate::l5_cognition::nt_core_second_brain::SecondBrain::new(),
        ));
        registry.register(Box::new(
            crate::l5_cognition::nt_mind::foundation::cleanup_engine::CleanupEngineSelfTest,
        ));
        registry.register(Box::new(
            crate::neotrix::nt_file_ability::FileAbilitySelfTest,
        ));
        // registry.register(Box::new(
        //     crate::l5_cognition::nt_core::nt_core_parallel::CapabilityClusterSelfTest,
        // ));
        registry.register(Box::new(
            crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_write_guard::WriteGuardAudit,
        ));
        for t in crate::l5_cognition::nt_core_arch_fitness::arch_fitness_tests() {
            registry.register(t);
        }
        let results = registry.run_all();
        let passed = results.iter().filter(|r| r.passed).count();
        let total = results.len();
        if passed < total {
            log::error!(
                "[seal] self_test: {}/{} passed — DETECTION SYSTEM DEGRADED",
                passed,
                total
            );
            for r in &results {
                if !r.passed {
                    log::error!("[seal] self_test: {}", r.summary());
                }
            }
            // T2.5: 自测失败必须改变行为, 不能只写日志。对本次演化施加负向奖励惩罚,
            // 使检测系统降级真实传导到演化信号 (而非静默 Continue)。
            let degradation = -((total - passed) as f64 / total.max(1) as f64);
            let base = brain._reward();
            brain._set_reward(base + degradation);
        }
        Ok(StageDecision::Continue)
    }
}


// ── ConsciousnessRewardStage ──────────────────────────────────
// Bridges the consciousness quality score into the SEAL reward signal.
// Every N iterations, reads _last_consciousness_quality from SelfIteratingBrain
// and applies a quality-based bonus/penalty to _reward.

make_stage!(ConsciousnessRewardStage);
impl BrainStage for ConsciousnessRewardStage {
    fn name(&self) -> &str {
        "consciousness_reward"
    }
    fn frequency(&self) -> usize {
        5
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let q = brain._last_consciousness_quality;
        let count = brain._consciousness_critique_count;
        if count == 0 {
            return Ok(StageDecision::Continue);
        }
        let reward_adj = if q >= 0.8 {
            0.05
        } else if q >= 0.6 {
            0.02
        } else if q >= 0.4 {
            -0.02
        } else if q >= 0.2 {
            -0.08
        } else {
            -0.15
        };
        let current = brain._reward;
        brain._reward = (current + reward_adj).clamp(-1.0, 1.0);
        log::info!(
            "[seal] consciousness_reward: quality={:.3} count={} adj={:+.3} reward={:.3}",
            q,
            count,
            reward_adj,
            brain._reward,
        );
        Ok(StageDecision::Continue)
    }
}
