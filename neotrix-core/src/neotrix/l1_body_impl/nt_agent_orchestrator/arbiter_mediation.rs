//! P4 融合节点: `arbiter_mediation` (吸收自 Anthropic multiagent 研究)。
//!
//! 向 orchestrator 注入多 agent 协调的调解/仲裁能力:
//! - 检测四类串扰: Collusion (串谋), Escalation (冲突升级),
//!   Inconsistency (跨 agent 事实不一致), Stalemate (死锁/无进展)。
//! - 输出 arbiter 式裁决: Overrule / Merge / Rerun / Escalate / Accept。
//!
//! 纯确定性启发式 — 无网络、无 tokio、零 unsafe。跨轮检测依赖
//! `ArbiterMediator::round_history` (内存态, 由 `run_mediation` 推进)。

/// 单个 agent 的一轮产出判断。
#[derive(Debug, Clone, PartialEq)]
pub struct AgentJudgement {
    pub agent_id: String,
    pub topic: String,
    pub claim: String,
    pub round: usize,
    /// 0.0..=1.0, 声明置信度 (串谋判定参考信号)。
    pub confidence: f32,
}

impl AgentJudgement {
    pub fn new(
        agent_id: &str,
        topic: &str,
        claim: &str,
        round: usize,
        confidence: f32,
    ) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            topic: topic.to_string(),
            claim: claim.to_string(),
            round,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// 调解问题类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediationIssue {
    /// 多个 agent 输出一致性异常偏高 — 串谋。
    Collusion,
    /// 冲突在 N 轮内未消解 — 升级。
    Escalation,
    /// 同一话题下跨 agent 事实矛盾。
    Inconsistency,
    /// 跨轮无进展 — 死锁。
    Stalemate,
}

/// 一次检测发现。
#[derive(Debug, Clone, PartialEq)]
pub struct MediationFinding {
    pub issue: MediationIssue,
    pub round: usize,
    pub agents: Vec<String>,
    /// 0.0..=1.0 严重度 (越高越严重)。
    pub severity: f32,
    pub details: String,
}

/// arbiter 式裁决。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArbitrationVerdict {
    /// 串谋 → 否决嫌疑产出。
    Overrule,
    /// 不一致但有公共面 → 合并。
    Merge,
    /// 死锁 → 重跑该话题。
    Rerun,
    /// 升级 → 上报上级编排器。
    Escalate,
    /// 低严重度 / 无问题 → 接受。
    Accept,
}

/// 一轮快照: 记录该轮出现的 (topic, claim), 供跨轮对比。
/// round 字段已删除 (never read — Dark Forest); 历史分析按 claims 全量比对。
#[derive(Debug, Clone)]
struct RoundSnapshot {
    claims: Vec<(String, String)>,
}

/// 调解器 — 持有跨轮历史, 驱动多 agent 协调。
#[derive(Debug, Default)]
pub struct ArbiterMediator {
    round_history: Vec<RoundSnapshot>,
}

/// 串谋判定阈值: 两 claim 归一化相似度 ≥ 此值视为串谋。
pub const COLLUSION_SIM_THRESHOLD: f32 = 0.85;
/// 事实不一致判定阈值: 同一话题下相似度 ≤ 此值视为矛盾。
pub const INCONSISTENCY_SIM_THRESHOLD: f32 = 0.30;
/// 死锁判定: 同一 (topic, claim) 在历史中连续出现 ≥ 此轮数。
pub const STALEMATE_ROUNDS: usize = 2;
/// 升级判定: 同一话题连续不一致 ≥ 此轮数。
pub const ESCALATION_LIMIT: usize = 3;
/// 低于此严重度的 finding 仲裁时直接 Accept。
pub const ACCEPT_SEVERITY: f32 = 0.5;
/// 参与相似度对比的 claim 最小有效 token 数 (低于此视为噪音跳过)。
pub const MIN_CLAIM_TOKENS: usize = 2;

impl ArbiterMediator {
    pub fn new() -> Self {
        Self::default()
    }

    /// 归一化 token Jaccard 相似度 (0.0..=1.0)。公开以便阈值边界测试。
    pub fn claim_similarity(a: &str, b: &str) -> f32 {
        let ta = tokens(a);
        let tb = tokens(b);
        if ta.is_empty() && tb.is_empty() {
            return 1.0;
        }
        if ta.is_empty() || tb.is_empty() {
            return 0.0;
        }
        let mut inter = 0usize;
        for t in &ta {
            if tb.contains(t) {
                inter += 1;
            }
        }
        let union = ta.len() + tb.len() - inter;
        if union == 0 {
            return 0.0;
        }
        inter as f32 / union as f32
    }

    /// 基于当前轮 judgements + 跨轮历史做启发式检测 (纯读, 不改状态)。
    pub fn detect(&self, judgements: &[AgentJudgement]) -> Vec<MediationFinding> {
        let mut findings = Vec::new();

        // 成对检测: 同 topic 下的 Collusion / Inconsistency。
        for i in 0..judgements.len() {
            for j in (i + 1)..judgements.len() {
                let a = &judgements[i];
                let b = &judgements[j];
                if a.topic != b.topic || a.round != b.round {
                    continue;
                }
                let sim = Self::claim_similarity(&a.claim, &b.claim);
                let claim_rich = tokens(&a.claim).len() >= MIN_CLAIM_TOKENS
                    && tokens(&b.claim).len() >= MIN_CLAIM_TOKENS;

                if sim >= COLLUSION_SIM_THRESHOLD {
                    let severity = sim * (a.confidence + b.confidence) / 2.0;
                    findings.push(MediationFinding {
                        issue: MediationIssue::Collusion,
                        round: a.round,
                        agents: vec![a.agent_id.clone(), b.agent_id.clone()],
                        severity,
                        details: format!(
                            "agents produced near-identical claims on '{}' (sim={:.3})",
                            a.topic, sim
                        ),
                    });
                } else if claim_rich && sim <= INCONSISTENCY_SIM_THRESHOLD {
                    let severity = 1.0 - sim;
                    findings.push(MediationFinding {
                        issue: MediationIssue::Inconsistency,
                        round: a.round,
                        agents: vec![a.agent_id.clone(), b.agent_id.clone()],
                        severity,
                        details: format!(
                            "contradictory claims on '{}' (sim={:.3})",
                            a.topic, sim
                        ),
                    });
                }
            }
        }

        // 跨轮检测: Stalemate (同一 claim 连续重复无进展)。
        for j in judgements {
            if tokens(&j.claim).len() < MIN_CLAIM_TOKENS {
                continue;
            }
            let prior_hits = self
                .round_history
                .iter()
                .filter(|snap| {
                    snap.claims
                        .iter()
                        .any(|(t, c)| t == &j.topic && Self::claim_similarity(c, &j.claim) >= COLLUSION_SIM_THRESHOLD)
                })
                .count();
            if prior_hits >= STALEMATE_ROUNDS {
                findings.push(MediationFinding {
                    issue: MediationIssue::Stalemate,
                    round: j.round,
                    agents: vec![j.agent_id.clone()],
                    severity: prior_hits as f32 / (prior_hits as f32 + 1.0),
                    details: format!(
                        "no progress on '{}': same claim repeated across {} prior rounds",
                        j.topic, prior_hits
                    ),
                });
            }
        }

        // 跨轮检测: Escalation (同一话题连续不一致 ≥ ESCALATION_LIMIT 轮)。
        let inconsistent_topics: Vec<String> = findings
            .iter()
            .filter(|f| f.issue == MediationIssue::Inconsistency)
            .map(|f| {
                f.details
                    .split('\'')
                    .nth(1)
                    .unwrap_or("")
                    .to_string()
            })
            .collect();

        for topic in &inconsistent_topics {
            let prior_rounds = self
                .round_history
                .iter()
                .filter(|snap| snap.claims.iter().any(|(t, _)| t == topic))
                .count();
            if prior_rounds + 1 >= ESCALATION_LIMIT {
                findings.push(MediationFinding {
                    issue: MediationIssue::Escalation,
                    round: judgements.first().map(|j| j.round).unwrap_or(0),
                    agents: Vec::new(),
                    severity: 1.0,
                    details: format!(
                        "unresolved conflict on '{}' persists across >= {} rounds",
                        topic, ESCALATION_LIMIT
                    ),
                });
            }
        }

        findings
    }

    /// arbiter 式裁决: 问题类别 → 裁决, 低严重度降级为 Accept。
    pub fn arbitrate(&self, finding: &MediationFinding) -> ArbitrationVerdict {
        if finding.severity < ACCEPT_SEVERITY {
            return ArbitrationVerdict::Accept;
        }
        match finding.issue {
            MediationIssue::Collusion => ArbitrationVerdict::Overrule,
            MediationIssue::Inconsistency => ArbitrationVerdict::Merge,
            MediationIssue::Stalemate => ArbitrationVerdict::Rerun,
            MediationIssue::Escalation => ArbitrationVerdict::Escalate,
        }
    }

    /// 记录一轮产出到历史 (供跨轮 stalemate/escalation 检测)。
    /// `round` 参数保留用于调用方语义 (未来时间窗检测预留), 当前未消费。
    fn record_round(&mut self, _round: usize, judgements: &[AgentJudgement]) {
        let claims: Vec<(String, String)> = judgements
            .iter()
            .map(|j| (j.topic.clone(), j.claim.clone()))
            .collect();
        self.round_history.push(RoundSnapshot { claims });
    }

    /// 生产入口: 对一轮 judgement 做完整调解, 产出报告。
    /// 先检测 → 逐条仲裁 → 再记录本轮历史。
    pub fn run_mediation(
        &mut self,
        round: usize,
        judgements: &[AgentJudgement],
    ) -> MediationReport {
        let findings = self.detect(judgements);
        let verdicts: Vec<ArbitrationVerdict> =
            findings.iter().map(|f| self.arbitrate(f)).collect();
        self.record_round(round, judgements);
        MediationReport {
            round,
            findings,
            verdicts,
        }
    }

    pub fn history_len(&self) -> usize {
        self.round_history.len()
    }
}

/// 调解报告。
#[derive(Debug, Clone, PartialEq)]
pub struct MediationReport {
    pub round: usize,
    pub findings: Vec<MediationFinding>,
    pub verdicts: Vec<ArbitrationVerdict>,
}

impl MediationReport {
    /// 无任何 finding → 协调干净。
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }

    /// 该话题的裁决 (首个命中)。
    pub fn verdict_for(&self, topic: &str) -> Option<ArbitrationVerdict> {
        self.findings
            .iter()
            .position(|f| f.details.contains(topic))
            .map(|i| self.verdicts[i])
    }

    pub fn summarize(&self) -> String {
        if self.is_clean() {
            format!("round {}: clean", self.round)
        } else {
            let mut s = format!("round {}: {} finding(s)\n", self.round, self.findings.len());
            for (f, v) in self.findings.iter().zip(&self.verdicts) {
                s.push_str(&format!(
                    "  [{:?}] {:?} — {}\n",
                    f.issue, v, f.details
                ));
            }
            s
        }
    }
}

fn tokens(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

impl crate::core::nt_core_self_test::SelfTest for ArbiterMediator {
    fn name(&self) -> &str {
        "nt_agent_orchestrator_arbiter_mediation"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut mediator = ArbiterMediator::new();
        let js = vec![
            AgentJudgement::new("a1", "http", "status is 200", 1, 0.9),
            AgentJudgement::new("a2", "http", "status is 200", 1, 0.9),
        ];
        let report = mediator.run_mediation(1, &js);
        if report.is_clean() {
            return Err(vec!["expected a collusion finding".into()]);
        }
        if report.verdicts.iter().all(|v| *v != ArbitrationVerdict::Overrule) {
            return Err(vec!["expected Overrule verdict".into()]);
        }
        if mediator.history_len() != 1 {
            return Err(vec!["round history not recorded".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn judgement(agent: &str, topic: &str, claim: &str, round: usize) -> AgentJudgement {
        AgentJudgement::new(agent, topic, claim, round, 0.9)
    }

    #[test]
    fn collusion_detected_for_near_identical_claims() {
        let mediator = ArbiterMediator::new();
        let js = vec![
            judgement("a1", "http", "the response status is 200 ok", 1),
            judgement("a2", "http", "the response status is 200 ok", 1),
        ];
        let findings = mediator.detect(&js);
        assert!(
            findings.iter().any(|f| f.issue == MediationIssue::Collusion),
            "identical claims must trigger collusion"
        );
    }

    #[test]
    fn collusion_threshold_edge_one_token_difference() {
        // 6 tokens, 1 不同 → sim = 5/6 ≈ 0.833 < 0.85 → 不触发串谋。
        let mediator = ArbiterMediator::new();
        let js = vec![
            judgement("a1", "http", "the response status is 200 ok", 1),
            judgement("a2", "http", "the response status is 404 ok", 1),
        ];
        let findings = mediator.detect(&js);
        assert!(
            !findings.iter().any(|f| f.issue == MediationIssue::Collusion),
            "below threshold must not trigger collusion"
        );
    }

    #[test]
    fn inconsistency_detected_for_contradictory_claims() {
        // 同一话题, 断言词汇几乎无重叠 → 归一化相似度 ≤ 0.30 判为矛盾。
        let mediator = ArbiterMediator::new();
        let js = vec![
            judgement("a1", "auth", "credentials grant access", 1),
            judgement("a2", "auth", "credentials are denied", 1),
        ];
        let findings = mediator.detect(&js);
        assert!(
            findings.iter().any(|f| f.issue == MediationIssue::Inconsistency),
            "contradictory claims must trigger inconsistency"
        );
    }

    #[test]
    fn stalemate_detected_across_rounds() {
        let mut mediator = ArbiterMediator::new();
        let c = "the query planner still returns no index";
        // 两轮历史出现相同 claim。
        mediator.record_round(1, &[judgement("a1", "perf", c, 1)]);
        mediator.record_round(2, &[judgement("a1", "perf", c, 2)]);
        let js = vec![judgement("a1", "perf", c, 3)];
        let findings = mediator.detect(&js);
        assert!(
            findings.iter().any(|f| f.issue == MediationIssue::Stalemate),
            "repeated claim across rounds must trigger stalemate"
        );
        assert_eq!(
            mediator.arbitrate(findings.iter().find(|f| f.issue == MediationIssue::Stalemate).unwrap()),
            ArbitrationVerdict::Rerun
        );
    }

    #[test]
    fn escalation_detected_after_limit_rounds() {
        let mut mediator = ArbiterMediator::new();
        let contradict = |round: usize| {
            vec![
                judgement("a1", "db", "string key maps rows", round),
                judgement("a2", "db", "integer id numbers records", round),
            ]
        };
        // 前两轮产生不一致 (积累 2 轮), 第三轮达到 ESCALATION_LIMIT。
        mediator.run_mediation(1, &contradict(1));
        mediator.run_mediation(2, &contradict(2));
        let report = mediator.run_mediation(3, &contradict(3));
        assert!(
            report.findings.iter().any(|f| f.issue == MediationIssue::Escalation),
            "persistent conflict must escalate"
        );
        assert_eq!(
            mediator.arbitrate(report.findings.iter().find(|f| f.issue == MediationIssue::Escalation).unwrap()),
            ArbitrationVerdict::Escalate
        );
    }

    #[test]
    fn low_severity_findings_accept() {
        let mediator = ArbiterMediator::new();
        let finding = MediationFinding {
            issue: MediationIssue::Inconsistency,
            round: 1,
            agents: vec!["a".into(), "b".into()],
            severity: 0.2,
            details: "minor".into(),
        };
        assert_eq!(mediator.arbitrate(&finding), ArbitrationVerdict::Accept);
    }

    #[test]
    fn clean_round_reports_clean() {
        let mut mediator = ArbiterMediator::new();
        let js = vec![
            judgement("a1", "json", "serializer writes key value pairs", 1),
            judgement("a2", "yaml", "parser reads indented blocks", 1),
        ];
        let report = mediator.run_mediation(1, &js);
        assert!(report.is_clean());
        assert!(report.verdicts.is_empty());
    }

    #[test]
    fn similarity_bounds() {
        assert_eq!(ArbiterMediator::claim_similarity("a b c", "a b c"), 1.0);
        assert_eq!(ArbiterMediator::claim_similarity("a b c", "x y z"), 0.0);
        assert_eq!(ArbiterMediator::claim_similarity("", ""), 1.0);
    }
}
