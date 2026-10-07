//! nt_sim_eval — EVO-10 RL＋仿真评估器（verl＋MarS 思想）。
//!
//! 可控场景＋反事实冲击＋风格化事实＋可验证 reward，全部纯函数：
//! `compute_facts` 从价格序列提波动/趋势；`GrpoReward::score` 给
//! tool-use 风格的可验证分；`evaluate` 把场景×序列合成一份报告。
//! 训练集群与市场网关在外层，本文件无 IO / 全局状态。

use serde::{Deserialize, Serialize};

/// 场景步数上限。
pub const MAX_STEPS: usize = 1000;

/// 反事实冲击。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimShock {
    None,
    Spike,
    Drop,
}

/// 仿真场景。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimScenario {
    pub steps: usize,
    pub shock: SimShock,
}

impl SimScenario {
    pub fn valid(&self) -> bool {
        self.steps >= 1 && self.steps <= MAX_STEPS
    }
}

/// 风格化事实（波动＋趋势）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StylizedFacts {
    pub volatility: f64,
    pub trend: f64,
}

/// 从价格序列提事实（空/单点返回 None）。
pub fn compute_facts(prices: &[f64]) -> Option<StylizedFacts> {
    if prices.len() < 2 {
        return None;
    }
    let n = prices.len() as f64;
    let mean = prices.iter().sum::<f64>() / n;
    let var = prices.iter().map(|p| (p - mean) * (p - mean)).sum::<f64>() / n;
    let first = prices[0];
    let last = prices[prices.len() - 1];
    let trend = if first == 0.0 { 0.0 } else { (last - first) / first.abs() };
    Some(StylizedFacts { volatility: var.sqrt(), trend })
}

/// GRPO 风格可验证 reward。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GrpoReward {
    pub verifiable: f64,
    /// ⭐ 审计裁定 2026-10-07（`check-fake-signal` R4 命中）：
    /// **⛔ 这不是伪信号** —— `format_ok` 在 `score()`（本文件 L71）里
    /// **真的影响总分**（格式错则半罚）。
    ///
    /// 它之所以「只有字面量赋值」，是因为本 struct 是**评估结果的数据载体** ——
    /// ⛔ 值本就该由**调用方**（真正的格式检查器）填入，
    /// ⛔ 而不是在 struct 内部自己算。
    ///
    /// ⚠️ 但实测：`GrpoReward { .. }` 的构造点**全在本文件的测试里**
    ///（L114/115/116/123）⇒ **生产路径零消费**。
    /// ⇒ 本 struct 当前是**未接线的评估骨架**，
    ///    ⛔ 而非「格式检查在假装工作」。
    ///
    /// ⭐ 正解（未实施，需 owner 决策）：接上真实的格式检查器并构造本 reward；
    ///    或若该评估路线已废弃 ⇒ 删除整个 struct。
    /// ⛔ 我**不擅自删除**：它承载 GRPO 评估的**规格意图**，删字段会销毁规格。
    pub format_ok: bool,
}

impl GrpoReward {
    /// 总分：可验证分箝位 [0,1]，格式错半罚。
    pub fn score(&self) -> f64 {
        let v = if self.verifiable.is_nan() {
            0.0
        } else {
            self.verifiable.clamp(0.0, 1.0)
        };
        if self.format_ok {
            v
        } else {
            v * 0.5
        }
    }
}

/// 评估报告。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimReport {
    pub facts: StylizedFacts,
    pub reward: f64,
    pub shock: SimShock,
}

/// 场景×序列合成报告（非法场景/空序列返回 None）。
pub fn evaluate(scenario: &SimScenario, prices: &[f64], reward: &GrpoReward) -> Option<SimReport> {
    if !scenario.valid() {
        return None;
    }
    let facts = compute_facts(prices)?;
    Some(SimReport { facts, reward: reward.score(), shock: scenario.shock })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facts_edges() {
        assert!(compute_facts(&[]).is_none());
        assert!(compute_facts(&[1.0]).is_none());
        let f = compute_facts(&[1.0, 2.0, 3.0]).unwrap_or(StylizedFacts {
            volatility: 0.0,
            trend: 0.0,
        });
        assert!(f.volatility > 0.0);
        assert!(f.trend > 0.0);
    }

    #[test]
    fn reward_clamp_and_format_penalty() {
        assert_eq!(GrpoReward { verifiable: 2.0, format_ok: true }.score(), 1.0);
        assert_eq!(GrpoReward { verifiable: f64::NAN, format_ok: true }.score(), 0.0);
        assert_eq!(GrpoReward { verifiable: 1.0, format_ok: false }.score(), 0.5);
    }

    #[test]
    fn evaluate_gates() {
        let ok = SimScenario { steps: 10, shock: SimShock::None };
        let bad = SimScenario { steps: 0, shock: SimShock::Spike };
        let r = GrpoReward { verifiable: 0.7, format_ok: true };
        assert!(evaluate(&ok, &[1.0, 1.5], &r).is_some());
        assert!(evaluate(&bad, &[1.0, 1.5], &r).is_none());
        assert!(evaluate(&ok, &[], &r).is_none());
    }

    #[test]
    fn scenario_bounds() {
        assert!(!SimScenario { steps: MAX_STEPS + 1, shock: SimShock::Drop }.valid());
        assert!(SimScenario { steps: MAX_STEPS, shock: SimShock::Drop }.valid());
    }
}
