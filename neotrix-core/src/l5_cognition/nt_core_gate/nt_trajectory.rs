//! nt_trajectory — 洞察检测/黄金轨迹/校准集.
//! 从 `nt_core_gate/mod.rs` 纯搬移, 行为零变更.

use serde::{Deserialize, Serialize};

use crate::l5_cognition::nt_core_prm::AgentTrajectory;

use super::nt_guardrail::{Claim, SchemaCheck};
use super::nt_judge::{JudgeFamily, JudgeInput};
use super::nt_panel_debate::JudgePanel;

/// 洞察事件 — 单次 rollout 内实现质量相对历史最佳发生突破 (Replica #2:
/// within-rollout 洞察检测, 不依赖外部标签, 用 running-max 探测真实能力增长)。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InsightEvent {
    /// 触发该事件时的实现质量分数 (样本质量的 running-max 突破点)
    pub score: f64,
    /// 突破前的历史最佳 — 事件 = 相对历史的最佳改进幅度
    pub previous_best: f64,
    /// 相对改进强度 (0..1) — (score - prev) / (1 - prev); 饱和时 (score==prev==1) 为 0
    pub strength: f64,
    /// 该事件所在 rollout 的轮次/编号
    pub step: u64,
}

/// 洞察检测器 — 在单个 rollout 内跟踪实现质量的 running-max, 每次质量超过历史最佳
/// 且改进强度超过阈值时 emit InsightEvent。可序列化跨 session 持久化。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightDetector {
    running_best: f64,
    threshold: f64,
    last_step: u64,
}

impl Default for InsightDetector {
    fn default() -> Self {
        Self {
            running_best: 0.0,
            threshold: 0.1,
            last_step: 0,
        }
    }
}

impl InsightDetector {
    pub fn new(threshold: f64) -> Self {
        Self {
            running_best: 0.0,
            threshold: threshold.max(0.0).min(1.0),
            last_step: 0,
        }
    }

    /// 记录一次实现质量; 若突破 running-best 且强度达阈值则返回洞察事件。
    /// 首个样本仅建立基线 (running_best==0.0 → 无历史可比, 不构成突破)。
    pub fn record(&mut self, score: f64, step: u64) -> Option<InsightEvent> {
        let score = score.clamp(0.0, 1.0);
        if self.running_best == 0.0 {
            self.running_best = score;
            self.last_step = step;
            return None;
        }
        if score > self.running_best
            && (score - self.running_best) / (1.0 - self.running_best) >= self.threshold
        {
            let prev = self.running_best;
            self.running_best = score;
            self.last_step = step;
            return Some(InsightEvent {
                score,
                previous_best: prev,
                strength: if prev == 1.0 {
                    0.0
                } else {
                    (score - prev) / (1.0 - prev)
                },
                step,
            });
        }
        None
    }

    /// 当前 running-best — 序列化恢复后可直接继续追踪。
    pub fn best(&self) -> f64 {
        self.running_best
    }

    /// 重置 — 新 rollout 开始时清除历史最佳。
    pub fn reset(&mut self) {
        self.running_best = 0.0;
        self.last_step = 0;
    }
}

/// 黄金轨迹标签 — 来自真实日志 (clean run / broken run)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrajectoryLabel {
    Clean,
    Broken,
}

/// 一条真实日志黄金样本 — clean 应放行, broken 应拦截。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldTrajectory {
    pub id: String,
    pub label: TrajectoryLabel,
    pub trajectory: AgentTrajectory,
    pub claims: Vec<Claim>,
    pub evidence_ids: Vec<String>,
    pub grounding_failures: u64,
    pub schema_failures: Vec<SchemaCheck>,
}

/// 校准集 — 用真实 clean/broken 日志验证门控 (Anthropic: 最强测试来自真实 transcripts)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationSet {
    pub gold: Vec<GoldTrajectory>,
}

impl CalibrationSet {
    pub fn new(gold: Vec<GoldTrajectory>) -> Self {
        Self { gold }
    }

    /// 从 experience-tree KB 的 kv_store `experience` namespace 构建真实黄金集
    /// (FPAM P1: 用真实轨迹校准, 而非 fixture)。
    ///
    /// 输入: `(key, value)` 序列, 每条 value 是 neotrix-experience 写入的
    /// `JSON {type, domain, content, evidence, verify_by}`。
    ///
    /// 标签规则 (确定性, 非猜测): `type` ∈ 失败族 → Broken; 其余 → Clean。
    pub fn from_kb_experience(entries: &[(String, String)]) -> Self {
        let failure_types = [
            "defect",
            "error",
            "blocker",
            "block",
            "regression",
            "fail",
            "wip",
            "warning",
            "bug",
        ];
        let mut gold: Vec<GoldTrajectory> = Vec::new();
        let mut id = 0u64;
        for (key, value) in entries {
            if !key.starts_with("branch_") {
                continue;
            }
            let Ok(v) = serde_json::from_str::<serde_json::Value>(value) else {
                continue;
            };
            let Some(etype) = v.get("type").and_then(|t| t.as_str()) else {
                continue;
            };
            let content = v.get("content").and_then(|t| t.as_str()).unwrap_or("");
            let evidence = v.get("evidence").and_then(|t| t.as_str()).unwrap_or("");
            let label = if failure_types.iter().any(|f| etype.eq_ignore_ascii_case(f)) {
                TrajectoryLabel::Broken
            } else {
                TrajectoryLabel::Clean
            };
            let mut traj = AgentTrajectory::new(id, content.to_string());
            traj.push(crate::l5_cognition::nt_core_prm::TrajectoryStep {
                step_idx: 0,
                specialist: crate::l0_substrate::nt_core_traits::SpecialistType::RiskAssessor,
                e8_mode: crate::l5_cognition::nt_core_hex::ReasoningHexagram::new(0b001010),
                action: "absorb".to_string(),
                input: evidence.to_string(),
                output: content.to_string(),
                duration_ms: None,
                success: label == TrajectoryLabel::Clean,
                external_reward: Some(if label == TrajectoryLabel::Clean {
                    1.0
                } else {
                    0.0
                }),
            });
            traj.completed = true;
            let evidence_ids: Vec<String> = evidence
                .split([',', ' ', ';'])
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect();
            let evidence_refs: Vec<&str> = evidence_ids.iter().map(|s| s.as_str()).collect();
            gold.push(GoldTrajectory {
                id: key.clone(),
                label,
                trajectory: traj,
                claims: vec![Claim::new(content, &evidence_refs)],
                evidence_ids,
                grounding_failures: if label == TrajectoryLabel::Broken {
                    2
                } else {
                    0
                },
                schema_failures: Vec::new(),
            });
            id += 1;
        }
        Self { gold }
    }

    /// pass^k 校准: 对每条黄金样本跑评审组 (runs 次, 需 k 次通过),
    /// 报告 clean 召回 / broken 拦截 / 综合。
    pub fn pass_k(&self, panel: &JudgePanel, runs: usize, k: usize) -> CalibrationReport {
        let mut clean_total = 0usize;
        let mut clean_pass = 0usize;
        let mut broken_total = 0usize;
        let mut broken_block = 0usize;

        for g in &self.gold {
            let input = JudgeInput {
                candidate: g.trajectory.task.clone(),
                claims: g.claims.clone(),
                evidence_ids: g.evidence_ids.clone(),
                trajectory: Some(g.trajectory.clone()),
                grounding_failures: g.grounding_failures,
                schema_failures: g.schema_failures.clone(),
                producer_family: JudgeFamily::None,
                rubric: None,
                samples: 1,
                attestation: None,
            };
            let ens = panel.run_ensemble(&input, runs, k);
            match g.label {
                TrajectoryLabel::Clean => {
                    clean_total += 1;
                    if ens.passed {
                        clean_pass += 1;
                    }
                }
                TrajectoryLabel::Broken => {
                    broken_total += 1;
                    if !ens.passed {
                        broken_block += 1;
                    }
                }
            }
        }

        let clean_recall = if clean_total == 0 {
            0.0
        } else {
            clean_pass as f64 / clean_total as f64
        };
        let broken_precision = if broken_total == 0 {
            1.0
        } else {
            broken_block as f64 / broken_total as f64
        };
        let balanced = if (clean_recall + broken_precision) == 0.0 {
            0.0
        } else {
            2.0 * clean_recall * broken_precision / (clean_recall + broken_precision)
        };

        CalibrationReport {
            clean_total,
            clean_pass,
            broken_total,
            broken_block,
            clean_recall,
            broken_precision,
            balanced,
        }
    }
}

/// 校准报告。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationReport {
    pub clean_total: usize,
    pub clean_pass: usize,
    pub broken_total: usize,
    pub broken_block: usize,
    /// clean 样本通过率
    pub clean_recall: f64,
    /// broken 样本拦截率
    pub broken_precision: f64,
    /// F1 调和均值
    pub balanced: f64,
}
