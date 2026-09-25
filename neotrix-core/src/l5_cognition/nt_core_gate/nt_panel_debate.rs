//! nt_panel_debate — 三族法官/评审组聚合/辩论/ensemble.
//! 从 `nt_core_gate/mod.rs` 纯搬移, 行为零变更.

use serde::{Deserialize, Serialize};

use crate::l5_cognition::nt_core_prm::ScoredCriterion;

use super::nt_guardrail::{FaithfulnessReport, GuardrailReport};
use super::nt_judge::{AsyncPanelJudge, JudgeFamily, JudgeInput, JudgeOpinion, PanelJudge};
use super::nt_types::{DebiasConfig, GuardAction, Verdict};

/// 分析族法官 — 判据: 轨迹 soundness (step 成功率/外部奖励), 无轨迹则用候选文本结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticPanelJudge {
    pub id: String,
}

impl Default for AnalyticPanelJudge {
    fn default() -> Self {
        Self {
            id: "analytic-v1".to_string(),
        }
    }
}

impl PanelJudge for AnalyticPanelJudge {
    fn judge_id(&self) -> &str {
        &self.id
    }

    fn family(&self) -> JudgeFamily {
        JudgeFamily::Analytic
    }

    fn score(&self, input: &JudgeInput) -> JudgeOpinion {
        let mut opinion = JudgeOpinion::new(&self.id, self.family());

        if let Some(traj) = &input.trajectory {
            let total = traj.steps.len().max(1);
            let successes = traj.steps.iter().filter(|s| s.success).count();
            let base = successes as f64 / total as f64;
            let reward_boost = traj.outcome_reward.unwrap_or(0.0).max(0.0) * 0.15;
            let score = (base * 0.85 + reward_boost).max(0.0).min(1.0);
            opinion.raw_score = score;
            opinion.criteria.push(ScoredCriterion {
                name: "soundness".to_string(),
                score,
                rationale: Some(format!(
                    "轨迹 {} 步, {} 成功, 外部奖励 {}",
                    total,
                    successes,
                    traj.outcome_reward.unwrap_or(0.0)
                )),
            });
        } else {
            let non_empty = !input.candidate.trim().is_empty();
            let has_claims = !input.claims.is_empty();
            let score = match (non_empty, has_claims) {
                (true, true) => 0.85,
                (true, false) => 0.70,
                (false, _) => 0.25,
            };
            opinion.raw_score = score;
            opinion.criteria.push(ScoredCriterion {
                name: "completeness".to_string(),
                score,
                rationale: Some(format!("候选非空={}, 含声明={}", non_empty, has_claims)),
            });
        }

        // grounding 失败计数压低 soundness
        if input.grounding_failures > 0 {
            let penalty = (input.grounding_failures as f64 * 0.05).max(0.0).min(0.3);
            opinion.raw_score = (opinion.raw_score - penalty).max(0.0).min(1.0);
            opinion
                .attribution_tags
                .push(format!("grounding_failures={}", input.grounding_failures));
        }
        opinion.confidence = 0.8;
        opinion
    }
}

/// 启发式族法官 — 判据: 忠实度 (grounding_ratio) + 证据覆盖 + 冗长惩罚。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePanelJudge {
    pub id: String,
}

impl Default for EvidencePanelJudge {
    fn default() -> Self {
        Self {
            id: "evidence-v1".to_string(),
        }
    }
}

impl PanelJudge for EvidencePanelJudge {
    fn judge_id(&self) -> &str {
        &self.id
    }

    fn family(&self) -> JudgeFamily {
        JudgeFamily::Heuristic
    }

    fn score(&self, input: &JudgeInput) -> JudgeOpinion {
        let mut opinion = JudgeOpinion::new(&self.id, self.family());
        let faith = FaithfulnessReport::audit(&input.claims, &input.evidence_ids);

        let mut tags = Vec::new();
        let mut criteria = Vec::new();

        // 忠实度
        criteria.push(ScoredCriterion {
            name: "faithfulness".to_string(),
            score: faith.grounding_ratio,
            rationale: Some(format!(
                "grounded {}/{}",
                faith.grounded, faith.claims_total
            )),
        });
        if !faith.fabricated.is_empty() {
            tags.push(format!("fabrications={}", faith.fabricated.len()));
        }

        // 证据覆盖: 每个声明至少 1 条引用
        let refs_total: usize = input.claims.iter().map(|c| c.evidence_refs.len()).sum();
        let coverage = if input.claims.is_empty() {
            0.0
        } else {
            refs_total as f64 / (input.claims.len() as f64).max(1.0)
        };
        let coverage_score = (coverage / 2.0).max(0.0).min(1.0);
        criteria.push(ScoredCriterion {
            name: "evidence_coverage".to_string(),
            score: coverage_score,
            rationale: Some(format!(
                "平均引用 {}",
                if input.claims.is_empty() {
                    0
                } else {
                    refs_total / input.claims.len()
                }
            )),
        });

        // 冗长惩罚 (verbosity bias mitigation — 在 judge 侧显式记分)
        let len = input.candidate.chars().count();
        let mut score = faith.grounding_ratio * 0.7 + coverage_score * 0.3;
        if len > 0 && input.candidate.trim().is_empty() {
            score = 0.2;
            tags.push("empty_candidate".to_string());
        }
        opinion.raw_score = score;
        opinion.criteria = criteria;
        opinion.attribution_tags = tags;
        opinion.confidence = 0.75;
        opinion
    }
}

/// 结构化族法官 — 判据: schema 完整 + 声明结构 (引用非空)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralPanelJudge {
    pub id: String,
}

impl Default for StructuralPanelJudge {
    fn default() -> Self {
        Self {
            id: "structural-v1".to_string(),
        }
    }
}

impl PanelJudge for StructuralPanelJudge {
    fn judge_id(&self) -> &str {
        &self.id
    }

    fn family(&self) -> JudgeFamily {
        JudgeFamily::Symbolic
    }

    fn score(&self, input: &JudgeInput) -> JudgeOpinion {
        let mut opinion = JudgeOpinion::new(&self.id, self.family());

        let schema_ok = input.schema_failures.is_empty();
        let claims_structured = input.claims.iter().all(|c| !c.evidence_refs.is_empty());
        let score = match (schema_ok, claims_structured) {
            (true, true) => 0.9,
            (true, false) => 0.6,
            (false, true) => 0.35,
            (false, false) => 0.15,
        };
        opinion.raw_score = score;
        opinion.criteria.push(ScoredCriterion {
            name: "schema".to_string(),
            score,
            rationale: Some(format!(
                "schema_failures={}, claims_structured={}",
                input.schema_failures.len(),
                claims_structured
            )),
        });
        for s in &input.schema_failures {
            if !s.present {
                opinion
                    .attribution_tags
                    .push(format!("schema_missing:{}", s.field));
            }
        }
        opinion.confidence = 0.85;
        opinion
    }
}

/// 公正评审组 — 多家族法官聚合。
#[derive(Debug)]
pub struct JudgePanel {
    pub judges: Vec<Box<dyn PanelJudge>>,
    pub debias: DebiasConfig,
}

impl Default for JudgePanel {
    fn default() -> Self {
        Self::default_panel()
    }
}

impl JudgePanel {
    /// 三家族默认评审组 (Analytic + Heuristic + Symbolic)。
    pub fn default_panel() -> Self {
        Self {
            judges: vec![
                Box::new(AnalyticPanelJudge::default()),
                Box::new(EvidencePanelJudge::default()),
                Box::new(StructuralPanelJudge::default()),
            ],
            debias: DebiasConfig::default(),
        }
    }

    /// 运行评审组 → 去偏 → 聚合。
    ///
    /// 机械检查 (schema/grounding/fabrication) 优先于 LLM 分数: 护栏拒绝 → 直接 Block,
    /// 幻觉隔离 → Review 转人工; 之后才是多法官聚合 (median / agreement)。
    pub fn run(&self, input: &JudgeInput) -> PanelVerdict {
        let guard = GuardrailReport::evaluate(input, &self.debias);
        if guard.action == GuardAction::Reject {
            return PanelVerdict {
                opinions: Vec::new(),
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Block,
                routed_to_human: true,
                reasoning: format!("机械检查拒绝, 压过 LLM 分数: {}", guard.reason),
            };
        }
        if guard.action == GuardAction::Quarantine {
            return PanelVerdict {
                opinions: Vec::new(),
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Review,
                routed_to_human: true,
                reasoning: format!("幻觉隔离转人工: {}", guard.reason),
            };
        }

        let excluded: Vec<&str> = if self.debias.require_family_separation
            && input.producer_family != JudgeFamily::None
        {
            self.judges
                .iter()
                .filter(|j| j.family() == input.producer_family)
                .map(|j| j.judge_id())
                .collect()
        } else {
            Vec::new()
        };

        let mut opinions: Vec<JudgeOpinion> = Vec::new();
        for judge in &self.judges {
            if excluded.contains(&judge.judge_id()) {
                continue; // 家族分离: 评委 ≠ 生成方家族 (self-preference 硬排除)
            }
            let mut op = judge.score(input);
            self.debias_opinion(&mut op, input);
            opinions.push(op);
        }

        self.finalize(opinions, &excluded)
    }

    /// 异步评审路径 — 同步启发式法官 + 真实 LLM 法官共同聚合。
    ///
    /// 机械护栏 (schema/grounding/fabrication) 仍确定性前置, LLM 只是聚合中的打分器
    /// (FPAM: 无真实 LLM 法官 → 补齐; R-P79: 同 session 接线生产路径)。
    pub async fn run_async(
        &self,
        input: &JudgeInput,
        async_judges: &[&dyn AsyncPanelJudge],
    ) -> PanelVerdict {
        let guard = GuardrailReport::evaluate(input, &self.debias);
        if guard.action == GuardAction::Reject {
            return PanelVerdict {
                opinions: Vec::new(),
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Block,
                routed_to_human: true,
                reasoning: format!("机械检查拒绝, 压过 LLM 分数: {}", guard.reason),
            };
        }
        if guard.action == GuardAction::Quarantine {
            return PanelVerdict {
                opinions: Vec::new(),
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Review,
                routed_to_human: true,
                reasoning: format!("幻觉隔离转人工: {}", guard.reason),
            };
        }

        let excluded: Vec<&str> = if self.debias.require_family_separation
            && input.producer_family != JudgeFamily::None
        {
            self.judges
                .iter()
                .filter(|j| j.family() == input.producer_family)
                .map(|j| j.judge_id())
                .chain(
                    async_judges
                        .iter()
                        .filter(|j| j.family() == input.producer_family)
                        .map(|j| j.judge_id()),
                )
                .collect()
        } else {
            Vec::new()
        };

        let mut opinions: Vec<JudgeOpinion> = Vec::new();
        for judge in &self.judges {
            if excluded.contains(&judge.judge_id()) {
                continue;
            }
            let mut op = judge.score(input);
            self.debias_opinion(&mut op, input);
            opinions.push(op);
        }
        for judge in async_judges {
            if excluded.contains(&judge.judge_id()) {
                continue;
            }
            let mut op = judge.score(input).await;
            self.debias_opinion(&mut op, input);
            opinions.push(op);
        }

        self.finalize(opinions, &excluded)
    }

    /// 异步评审 + 自证观测比对 (Replica #9: 声称与观测矛盾 → 分砍半)。
    /// 在 run_async 基础上叠加矛盾惩罚: 惩罚系数 → 分数衰减;
    /// 若衰减后跌破 pass 阈值则 Pass 降级为 Review 转人工。
    pub async fn run_async_with_observations(
        &self,
        input: &JudgeInput,
        async_judges: &[&dyn AsyncPanelJudge],
        observed_budget: Option<f64>,
        observed_seeds: Option<u32>,
    ) -> PanelVerdict {
        let mut verdict = self.run_async(input, async_judges).await;
        if let Some(attestation) = &input.attestation {
            let penalty = attestation.contradiction_score(observed_budget, observed_seeds);
            if penalty > 0.0 {
                verdict.median_score = (verdict.median_score * (1.0 - penalty)).max(0.0);
                if verdict.median_score < self.debias.pass_threshold
                    && verdict.verdict == Verdict::Pass
                {
                    verdict.verdict = Verdict::Review;
                    verdict.routed_to_human = true;
                }
                verdict.reasoning = format!(
                    "{}; attestation_penalty={:.2} (矛盾 vs 观测)",
                    verdict.reasoning, penalty
                );
            }
        }
        verdict
    }

    /// 对单条意见应用去偏 (verbosity + self-preference), 同步/异步路径共用。
    fn debias_opinion(&self, op: &mut JudgeOpinion, input: &JudgeInput) {
        let family_same =
            input.producer_family != JudgeFamily::None && op.family == input.producer_family;
        let penalty = self.debias.verbosity_penalty_for(&input.candidate);
        let mut debiased = op.raw_score - penalty;
        if family_same {
            debiased -= self.debias.self_preference_penalty;
            op.attribution_tags
                .push("self_preference_penalized".to_string());
        }
        op.debiased_score = debiased.max(0.0).min(1.0);
    }

    /// 聚合去偏后的意见 → 裁决。同步与异步评审路径共用。
    fn finalize(&self, opinions: Vec<JudgeOpinion>, excluded: &[&str]) -> PanelVerdict {
        if opinions.is_empty() {
            return PanelVerdict {
                opinions,
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Block,
                routed_to_human: false,
                reasoning: "无可用法官 (全部被家族分离排除)".to_string(),
            };
        }

        let scores: Vec<f64> = opinions.iter().map(|o| o.debiased_score).collect();
        let median = median_f64(&scores);
        let agreement = agreement(&scores);

        let verdict = if median < self.debias.pass_threshold {
            Verdict::Block
        } else if agreement < self.debias.agreement_review_threshold {
            Verdict::Review // 高分歧 → 转人工, 不自动放行
        } else {
            Verdict::Pass
        };

        let routed_to_human = verdict == Verdict::Review || verdict == Verdict::Block;
        let reasoning = format!(
            "median={:.3}, agreement={:.3}, judges={}, excluded={:?}, verdict={:?}",
            median,
            agreement,
            opinions.len(),
            excluded,
            verdict
        );

        PanelVerdict {
            opinions,
            median_score: median,
            agreement,
            verdict,
            routed_to_human,
            reasoning,
        }
    }

    /// pass^k 门控: N 次运行, 至少 k 次 Pass 才算放行 (Anthropic: 门禁不用 Pass@1)。
    ///
    /// 机械检查 (schema/grounding/fabrication/grounding_failures) 确定性优先:
    /// 任一运行触发 Reject → 直接 Block; 触发 Quarantine → Review; 无视后续 LLM 分。
    pub fn run_ensemble(&self, input: &JudgeInput, runs: usize, k: usize) -> EnsembleVerdict {
        // 机械护栏前置 — 确定性检查压过 LLM 聚合
        let guard = GuardrailReport::evaluate(input, &self.debias);
        if guard.action == GuardAction::Reject {
            let v = PanelVerdict {
                opinions: Vec::new(),
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Block,
                routed_to_human: true,
                reasoning: format!("ensemble 前置护栏拒绝: {}", guard.reason),
            };
            return EnsembleVerdict {
                runs,
                k,
                passes: 0,
                passed: false,
                verdict: Verdict::Block,
                last: Box::new(v),
            };
        }
        if guard.action == GuardAction::Quarantine {
            let v = PanelVerdict {
                opinions: Vec::new(),
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Review,
                routed_to_human: true,
                reasoning: format!("ensemble 前置护栏隔离: {}", guard.reason),
            };
            return EnsembleVerdict {
                runs,
                k,
                passes: 0,
                passed: false,
                verdict: Verdict::Review,
                last: Box::new(v),
            };
        }

        let k = k.min(runs).max(1);
        let mut passes = 0usize;
        let mut last = None;
        for _ in 0..runs {
            let v = self.run(input);
            if v.verdict == Verdict::Pass {
                passes += 1;
            }
            last = Some(v);
        }
        let passed = passes >= k;
        let verdict = if passed {
            Verdict::Pass
        } else if passes == 0 {
            Verdict::Block
        } else {
            Verdict::Review
        };
        EnsembleVerdict {
            runs,
            k,
            passes,
            passed,
            verdict,
            last: Box::new(last.expect("runs>=1")),
        }
    }
}

fn median_f64(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = s.len();
    if n == 0 {
        0.0
    } else if n % 2 == 1 {
        s[n / 2]
    } else {
        (s[n / 2 - 1] + s[n / 2]) / 2.0
    }
}

/// 评分者间一致率: 1 - 归一化平均绝对差 (0..1)。1 = 完全一致。
fn agreement(scores: &[f64]) -> f64 {
    if scores.len() < 2 {
        return 1.0;
    }
    let mut sum = 0.0;
    let mut count = 0usize;
    for i in 0..scores.len() {
        for j in (i + 1)..scores.len() {
            sum += (scores[i] - scores[j]).abs();
            count += 1;
        }
    }
    let mean_abs = sum / count as f64;
    (1.0 - mean_abs).max(0.0).min(1.0)
}

/// 评审组裁决。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelVerdict {
    pub opinions: Vec<JudgeOpinion>,
    pub median_score: f64,
    pub agreement: f64,
    pub verdict: Verdict,
    pub routed_to_human: bool,
    pub reasoning: String,
}

impl PanelVerdict {
    pub fn is_pass(&self) -> bool {
        self.verdict == Verdict::Pass
    }
}

/// 辩论角色 — TradingAgents 多空辩论 + oh-my-hermes Planner→Architect→Critic 参照。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DebateRole {
    Pro,     // 正方: 支持提案
    Con,     // 反方: 反对提案
    Neutral, // 中立: 仲裁
}

/// 单轮辩论发言。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateRound {
    pub role: DebateRole,
    pub score: f64,
    pub confidence: f64,
    pub argument: String,
}

/// 辩论报告 — 对抗轮次 + 收敛结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateReport {
    pub rounds: Vec<DebateRound>,
    /// 辩论后收敛分数 (反方意见修正正方)。
    pub converged_score: f64,
    /// 分歧度 (0=一致, 1=完全对抗)。
    pub divergence: f64,
    /// 是否收敛到明确结论。
    pub converged: bool,
}

/// D6: 对抗式辩论 — 用 panel 的多数意见与少数意见互搏, 收敛出修正分数。
/// 机制: 高分意见=Pro, 低分意见=Con, 中间=Neutral; 3 轮辩论按分歧度衰减
/// Pro/Con 偏差, 得到更稳健的最终分。无外部 LLM 依赖, 纯启发式对抗。
pub fn deliberate(opinions: &[JudgeOpinion]) -> DebateReport {
    if opinions.is_empty() {
        return DebateReport {
            rounds: Vec::new(),
            converged_score: 0.0,
            divergence: 0.0,
            converged: true,
        };
    }
    let scores: Vec<f64> = opinions.iter().map(|o| o.debiased_score).collect();
    let n = scores.len() as f64;
    let mean: f64 = scores.iter().sum::<f64>() / n;
    let (min, max) = (
        scores.iter().cloned().fold(f64::INFINITY, f64::min),
        scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
    );
    let spread = max - min;
    // 分歧度 = 极差 (0=一致, 1=最大对立), 不用 /2 以反映真实对抗强度
    let divergence = if spread < 1e-9 { 0.0 } else { spread.min(1.0) };

    // 角色分配: 低于均值 → Con, 高于 → Pro, 等于 → Neutral
    let mut rounds = Vec::new();
    for (i, o) in opinions.iter().enumerate() {
        let role = if o.debiased_score < mean - 1e-9 {
            DebateRole::Con
        } else if o.debiased_score > mean + 1e-9 {
            DebateRole::Pro
        } else {
            DebateRole::Neutral
        };
        rounds.push(DebateRound {
            role,
            score: o.debiased_score,
            confidence: o.confidence,
            argument: format!(
                "{} opinion #{} {:.2}",
                match role {
                    DebateRole::Pro => "支持",
                    DebateRole::Con => "反对",
                    DebateRole::Neutral => "中立",
                },
                i,
                o.debiased_score
            ),
        });
    }

    // 收敛: 反方拉动 (均值向 Con 方向收敛), 分歧大时收敛更强 (取反方意见更重)。
    let con_mean: Vec<f64> = rounds
        .iter()
        .filter(|r| r.role == DebateRole::Con)
        .map(|r| r.score)
        .collect();
    let pro_mean: Vec<f64> = rounds
        .iter()
        .filter(|r| r.role == DebateRole::Pro)
        .map(|r| r.score)
        .collect();
    let con_avg = if con_mean.is_empty() {
        mean
    } else {
        con_mean.iter().sum::<f64>() / con_mean.len() as f64
    };
    let pro_avg = if pro_mean.is_empty() {
        mean
    } else {
        pro_mean.iter().sum::<f64>() / pro_mean.len() as f64
    };
    // 三因素修正: 基准均值 + 反方权重 (分歧度) + 正方残余 (1-分歧度)
    let converged_score =
        mean * 0.5 + con_avg * divergence * 0.5 + pro_avg * (1.0 - divergence) * 0.5;

    DebateReport {
        rounds,
        converged_score,
        divergence,
        converged: divergence < 0.5,
    }
}

/// pass^k 汇总裁决。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsembleVerdict {
    pub runs: usize,
    pub k: usize,
    pub passes: usize,
    pub passed: bool,
    pub verdict: Verdict,
    pub last: Box<PanelVerdict>,
}
