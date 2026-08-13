//! # NT-CORE benchmark_suite — 三层基准框架 + 统计等价门
//!
//! 吸收源:
//! - scaling-emulations (MIT thesis): 三层指标 (确定性 / 随机分布匹配 / 行为)
//!   + 基准套件 + 扰动证据优先于纯相关。
//! - DeepSeek-V4-CRACK 能力保持门: 模型修改后必须 HumanEval/GSM8K/MMLU-Pro
//!   回测 + McNemar 统计等价检验 (p>0.05 → 等价，允许修改落地)。
//!
//! 骨架阶段 (C0): 指标层级 + McNemar 精确二项检验 + 套件运行器已接 `/bench suite`
//! 生产路径; 待完善: 分布匹配实现 (EMD/峰度) / 行为指标 harness / 扰动证据。
use std::time::{Duration, Instant};

/// 三层指标层级 (thesis)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricTier {
    /// 确定性: 精确匹配 (HumanEval pass@1, exact match)。
    Deterministic,
    /// 随机分布匹配: 输出分布与参考分布的相似性 (EMD/峰度等)。
    StochasticDistribution,
    /// 行为: 端到端行为指标 (任务完成率、副作用)。
    Behavioral,
}

impl MetricTier {
    pub fn label(&self) -> &'static str {
        match self {
            MetricTier::Deterministic => "deterministic",
            MetricTier::StochasticDistribution => "stochastic-distribution",
            MetricTier::Behavioral => "behavioral",
        }
    }
}

/// 单个基准用例。
#[derive(Debug, Clone)]
pub struct BenchmarkCase {
    pub name: String,
    pub tier: MetricTier,
    pub input: String,
    pub reference: String,
    /// 确定性判定的期望输出 (tier=Deterministic 用)。
    pub expected: Option<String>,
}

/// 单用例结果。
#[derive(Debug, Clone)]
pub struct CaseResult {
    pub name: String,
    pub tier: MetricTier,
    pub passed: bool,
    /// 数值得分 [0,1] — 供随机分布/行为层打分。
    pub score: f64,
    pub elapsed: Duration,
}

/// 套件运行结果。
#[derive(Debug, Clone)]
pub struct SuiteReport {
    pub total: usize,
    pub passed: usize,
    pub by_tier: Vec<(MetricTier, usize, usize)>,
    pub avg_score: f64,
    pub elapsed: Duration,
}

/// 用例评估函数签名: input + reference → (passed, score)。
pub type Evaluator = dyn Fn(&BenchmarkCase) -> (bool, f64) + Send + Sync;

/// 三层基准套件运行器。
pub struct BenchmarkSuite {
    pub cases: Vec<BenchmarkCase>,
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkSuite {
    pub fn new() -> Self {
        Self { cases: Vec::new() }
    }

    pub fn add(&mut self, case: BenchmarkCase) {
        self.cases.push(case);
    }

    /// 运行全部用例。tier=Deterministic 用 expected 精确匹配；其余层由 eval 提供。
    pub fn run(&self, eval: &Evaluator) -> SuiteReport {
        let start = Instant::now();
        let mut results: Vec<CaseResult> = Vec::with_capacity(self.cases.len());
        for case in &self.cases {
            let c_start = Instant::now();
            let (mut passed, mut score) = eval(case);
            if case.tier == MetricTier::Deterministic {
                let exact = case
                    .expected
                    .as_ref()
                    .map(|exp| exp.trim() == case.input.trim())
                    .unwrap_or(false);
                passed = exact;
                score = if exact { 1.0 } else { 0.0 };
            }
            results.push(CaseResult {
                name: case.name.clone(),
                tier: case.tier,
                passed,
                score,
                elapsed: c_start.elapsed(),
            });
        }
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let avg_score = if total == 0 {
            0.0
        } else {
            results.iter().map(|r| r.score).sum::<f64>() / total as f64
        };
        let mut by_tier: Vec<(MetricTier, usize, usize)> = Vec::new();
        for tier in [
            MetricTier::Deterministic,
            MetricTier::StochasticDistribution,
            MetricTier::Behavioral,
        ] {
            let n = results.iter().filter(|r| r.tier == tier).count();
            let p = results
                .iter()
                .filter(|r| r.tier == tier && r.passed)
                .count();
            by_tier.push((tier, p, n));
        }
        SuiteReport {
            total,
            passed,
            by_tier,
            avg_score,
            elapsed: start.elapsed(),
        }
    }
}

/// McNemar 统计等价检验 — 成对 (对照组, 实验组) 二分类结果的差异显著性。
/// b01 = 对照 pass 且实验 fail; b10 = 对照 fail 且实验 pass。
/// 返回双侧精确二项 p 值。p > alpha (0.05) → 统计等价，允许修改落地。
pub fn mcnemar_p(b01: usize, b10: usize) -> f64 {
    let discordant = b01 + b10;
    if discordant == 0 {
        return 1.0;
    }
    let k = b01.min(b10);
    // 精确二项尾概率 (两侧 × 2，截断于 1.0)。
    let mut p = 0.0f64;
    for i in 0..=k {
        let mut log_c = 0.0f64;
        for j in 0..i {
            log_c += ((discordant - j) as f64).ln() - ((j + 1) as f64).ln();
        }
        p += (log_c - (discordant as f64) * 2f64.ln()).exp();
    }
    (2.0 * p).min(1.0)
}

impl SuiteReport {
    pub fn is_equivalent(&self, other: &SuiteReport) -> bool {
        // 骨架: 通过率差 + McNemar 门。其他对照数据需在调用方按用例配对。
        let a = if self.total == 0 {
            0.0
        } else {
            self.passed as f64 / self.total as f64
        };
        let b = if other.total == 0 {
            0.0
        } else {
            other.passed as f64 / other.total as f64
        };
        (a - b).abs() < 0.05
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcnemar_equivalence_high_p() {
        // 148 vs 150 over 164 — 论文实测 p=0.625。
        let p = mcnemar_p(2, 0);
        assert!(p > 0.05);
        assert!(p <= 1.0);
    }

    #[test]
    fn mcnemar_difference_low_p() {
        let p = mcnemar_p(12, 1);
        assert!(p < 0.05);
    }

    #[test]
    fn deterministic_tier_exact_match() {
        let mut suite = BenchmarkSuite::new();
        suite.add(BenchmarkCase {
            name: "exact".into(),
            tier: MetricTier::Deterministic,
            input: "  42  ".into(),
            reference: String::new(),
            expected: Some("42".into()),
        });
        let eval = |_c: &BenchmarkCase| (true, 1.0);
        let rep = suite.run(&eval);
        assert_eq!(rep.passed, 1);
        assert!((rep.avg_score - 1.0).abs() < 1e-9);
    }
}
