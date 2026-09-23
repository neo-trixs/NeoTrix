//! Deterministic file-risk scorer 0–10 (R13 吸收).
//!
//! 思想来源（非代码，AGPL 防火墙）：repowise 的三透镜评分（defect risk /
//! maintainability / performance）＋尺寸显式化（size 作为独立报告项，不藏进权重）。
//! 本实现为纯净室自研：纯函数、零 LLM、零 IO、无外部依赖（std only）。
//!
//! 设计约束（对照 repowise 方法论声明）：
//! - 每个透镜独立 0–10，可解释（权重写死在代码注释，不可调参隐藏）。
//! - 尺寸单独报告，不参与加权（防"大文件恒高分"混淆）。
//! - 输出是排序依据，不是判决；门禁阈值由调用方定。

use serde::{Deserialize, Serialize};

/// 风险输入：全部可度量、无需模型（调用方负责采集）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskInputs {
    /// 代码行数（仅报告与归一化参考，不直接加权）。
    pub loc: u32,
    /// 圈复杂度（McCabe 近似；1 = 线性）。
    pub cyclomatic: u32,
    /// 90 天变更次数。
    pub churn_90d: u32,
    /// 6 个月缺陷修复次数。
    pub prior_defects_6m: u32,
    /// 是否无测试覆盖的高频变更点。
    pub untested_hotspot: bool,
    /// 是否存在循环内 IO。
    pub io_in_loop: bool,
    /// 公开 API 面数量（pub fn/struct/enum 计数）。
    pub public_surface: u32,
}

/// 三透镜评分（各 0–10，越高越险）＋综合分。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskScore {
    /// 缺陷风险：历史缺陷 0.5/次（封顶 5）＋变更熵 0.2/次（封顶 3）＋无测试热点 ＋2。
    pub defect: f64,
    /// 可维护性风险：复杂度超 10 部分 0.3/点（封顶 5）＋公开面 0.1/个（封顶 3）＋体积档（>2000 行 ＋2）。
    pub maintainability: f64,
    /// 性能风险：循环内 IO ＋6＋复杂度超 20 部分 0.2/点（封顶 4）。
    pub performance: f64,
    /// 综合：三透镜均值（等权；调用方可自行重加权）。
    pub overall: f64,
    /// 回显输入行数（尺寸显式化，不参与加权）。
    pub loc: u32,
}

fn clamp10(x: f64) -> f64 {
    if x < 0.0 {
        0.0
    } else if x > 10.0 {
        10.0
    } else {
        x
    }
}

/// 确定性评分：同输入恒同输出，可单测锁定。
pub fn file_risk_score(input: &RiskInputs) -> RiskScore {
    let defect = clamp10(
        (input.prior_defects_6m as f64) * 0.5
            + (input.churn_90d as f64) * 0.2
            + if input.untested_hotspot { 2.0 } else { 0.0 },
    );
    let maintainability = clamp10(
        ((input.cyclomatic as f64) - 10.0).max(0.0) * 0.3
            + (input.public_surface as f64) * 0.1
            + if input.loc > 2000 { 2.0 } else { 0.0 },
    );
    let performance = clamp10(
        if input.io_in_loop { 6.0 } else { 0.0 }
            + ((input.cyclomatic as f64) - 20.0).max(0.0) * 0.2,
    );
    let overall = (defect + maintainability + performance) / 3.0;
    RiskScore {
        defect,
        maintainability,
        performance,
        overall,
        loc: input.loc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_file_scores_low() {
        let s = file_risk_score(&RiskInputs {
            loc: 120,
            cyclomatic: 3,
            ..Default::default()
        });
        assert!(s.overall < 1.0, "clean file must score low: {s:?}");
        assert_eq!(s.loc, 120);
    }

    #[test]
    fn test_hotspot_scores_high_and_monotone() {
        let base = RiskInputs {
            loc: 800,
            cyclomatic: 25,
            churn_90d: 12,
            prior_defects_6m: 4,
            untested_hotspot: true,
            io_in_loop: true,
            public_surface: 30,
        };
        let hot = file_risk_score(&base);
        assert!(hot.defect >= 5.0 && hot.performance >= 6.0);
        // 单调性：缺陷数只增不减 → defect 不降
        let worse = RiskInputs {
            prior_defects_6m: 9,
            ..base.clone()
        };
        let worse_score = file_risk_score(&worse);
        assert!(worse_score.defect >= hot.defect);
        assert!(worse_score.overall >= hot.overall);
    }

    #[test]
    fn test_size_never_leaks_into_weights() {
        // 同形状不同体积：三透镜不变（体积只回显），体积档除外（>2000 显式 +2 可解释）
        let small = file_risk_score(&RiskInputs {
            loc: 100,
            cyclomatic: 5,
            ..Default::default()
        });
        let big = file_risk_score(&RiskInputs {
            loc: 1900,
            cyclomatic: 5,
            ..Default::default()
        });
        assert_eq!(small.defect, big.defect);
        assert_eq!(small.performance, big.performance);
        assert_eq!(small.maintainability, big.maintainability);
    }
}
