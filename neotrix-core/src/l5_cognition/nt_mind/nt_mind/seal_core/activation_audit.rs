//! _ActivationAuditStage — 论文驱动进化 (arXiv:2605.30621 Harness Evolution) 的量化审计 stage。
//!
//! 论文核心发现：
//! - **激活失败 (SLR)**：弱模型 harness 激活成功率仅 25.1%（Opus 95.7%）。
//! - **跟随失败 (HFR)**：弱模型 harness 跟随率仅 0.142（Opus 0.757）。
//! - **阶段依从**：进化的 SEAL 阶段（explore→distill→self_test→absorb）是否按序执行。
//!
//! 本 stage 不接外部模型，纯量化累计每次 harness 进化的 SLR / HFR / phase-adherence，
//! 输出可序列化指标与单测验证的确定性计算。低风险评估，不改动既有 SEAL 主流程。

use serde::{Deserialize, Serialize};

/// 单次 harness 进化的可量化观测样本。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub struct _ActivationAuditSample {
    /// 激活尝试次数（harness 被触发 / 加载的次数）。
    pub activation_attempts: u64,
    /// 激活成功次数。
    pub activation_successes: u64,
    /// 发出的指令数（GWT 向 harness 下发的 directive）。
    pub directives_issued: u64,
    /// 被 harness 实际跟随的指令数。
    pub directives_followed: u64,
    /// 期望执行的 SEAL 阶段步数。
    pub phase_steps_expected: u64,
    /// 实际按序执行的阶段步数。
    pub phase_steps_adhered: u64,
}

impl _ActivationAuditSample {
    /// 单样本的三项比率（分母为 0 时返回 0.0，避免除零）。
    pub fn rates(&self) -> _AuditRates {
        _AuditRates {
            slr: rate(self.activation_successes, self.activation_attempts),
            hfr: rate(self.directives_followed, self.directives_issued),
            phase_adherence: rate(self.phase_steps_adhered, self.phase_steps_expected),
        }
    }
}

/// 安全比率：num/den，den==0 时返回 0.0。
fn rate(num: u64, den: u64) -> f64 {
    if den == 0 {
        0.0
    } else {
        num as f64 / den as f64
    }
}

/// 激活/跟随/阶段依从三项比率。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct _AuditRates {
    /// Harness 激活成功率 (Success-rate of Loading/activation)。
    pub slr: f64,
    /// Harness 跟随率 (Harness Follow Rate)。
    pub hfr: f64,
    /// SEAL 阶段依从率。
    pub phase_adherence: f64,
}

/// 基于累计 SLR 的进化健康分级（论文阈值：弱模型 25.1% / 强模型 95.7%）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum _AuditHealth {
    /// SLR < 0.5 —— 激活严重失败（接近论文弱模型档）。
    Critical,
    /// 0.5 <= SLR < 0.8 —— 部分退化。
    Degraded,
    /// SLR >= 0.8 —— 健康（接近论文强模型档）。
    Healthy,
}

/// _ActivationAuditStage — 累计追踪每次 harness 进化的激活/跟随/阶段依从。
#[derive(Debug, Clone, Default)]
pub struct _ActivationAuditStage {
    samples: Vec<_ActivationAuditSample>,
    total: _ActivationAuditSample,
}

impl _ActivationAuditStage {
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录一次 harness 进化样本，并累加到总计。
    pub fn record(&mut self, sample: _ActivationAuditSample) {
        self.total.activation_attempts += sample.activation_attempts;
        self.total.activation_successes += sample.activation_successes;
        self.total.directives_issued += sample.directives_issued;
        self.total.directives_followed += sample.directives_followed;
        self.total.phase_steps_expected += sample.phase_steps_expected;
        self.total.phase_steps_adhered += sample.phase_steps_adhered;
        self.samples.push(sample);
    }

    /// 已记录的进化轮数。
    pub(crate) fn _n_cycles(&self) -> usize {
        self.samples.len()
    }

    /// 最近一次样本的比率（无样本时 None）。
    pub(crate) fn _latest_rates(&self) -> Option<_AuditRates> {
        self.samples.last().map(|s| s.rates())
    }

    /// 跨所有样本的累计比率。
    pub(crate) fn _cumulative_rates(&self) -> _AuditRates {
        self.total.rates()
    }

    /// 生成可量化报告（含健康分级）。
    pub fn report(&self) -> _ActivationAuditReport {
        let r = self._cumulative_rates();
        let health = if self.samples.is_empty() {
            // 无进化轮 → 无故障数据，判 Healthy 而非 Critical。
            _AuditHealth::Healthy
        } else if r.slr < 0.5 {
            _AuditHealth::Critical
        } else if r.slr < 0.8 {
            _AuditHealth::Degraded
        } else {
            _AuditHealth::Healthy
        };
        _ActivationAuditReport {
            _n_cycles: self.samples.len() as u64,
            slr: r.slr,
            hfr: r.hfr,
            phase_adherence: r.phase_adherence,
            health,
        }
    }
}

/// 可量化审计产出（供上报 / 持久化 / 对照论文阈值）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct _ActivationAuditReport {
    pub _n_cycles: u64,
    pub slr: f64,
    pub hfr: f64,
    pub phase_adherence: f64,
    pub health: _AuditHealth,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slr_basic_m_over_n() {
        // 给定 100 次尝试、25 次成功激活 → SLR = 0.25（论文弱模型档）。
        let s = _ActivationAuditSample {
            activation_attempts: 100,
            activation_successes: 25,
            ..Default::default()
        };
        let r = s.rates();
        assert!((r.slr - 0.25).abs() < 1e-9);
    }

    #[test]
    fn test_slr_strong_model() {
        let s = _ActivationAuditSample {
            activation_attempts: 1000,
            activation_successes: 957,
            ..Default::default()
        };
        assert!((s.rates().slr - 0.957).abs() < 1e-9);
    }

    #[test]
    fn test_hfr_basic() {
        // 论文弱模型 HFR = 0.142。
        let s = _ActivationAuditSample {
            directives_issued: 1000,
            directives_followed: 142,
            ..Default::default()
        };
        assert!((s.rates().hfr - 0.142).abs() < 1e-9);
    }

    #[test]
    fn test_phase_adherence() {
        let s = _ActivationAuditSample {
            phase_steps_expected: 8,
            phase_steps_adhered: 6,
            ..Default::default()
        };
        assert!((s.rates().phase_adherence - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_empty_sample_rates_are_zero() {
        let r = _ActivationAuditSample::default().rates();
        assert_eq!(r.slr, 0.0);
        assert_eq!(r.hfr, 0.0);
        assert_eq!(r.phase_adherence, 0.0);
    }

    #[test]
    fn test_cumulative_rates_across_cycles() {
        let mut stage = _ActivationAuditStage::new();
        stage.record(_ActivationAuditSample {
            activation_attempts: 10,
            activation_successes: 5,
            directives_issued: 10,
            directives_followed: 5,
            phase_steps_expected: 4,
            phase_steps_adhered: 4,
        });
        stage.record(_ActivationAuditSample {
            activation_attempts: 30,
            activation_successes: 25, // 累计 30/40 = 0.75
            directives_issued: 20,
            directives_followed: 18, // 累计 23/30 ≈ 0.7667
            phase_steps_expected: 4,
            phase_steps_adhered: 3,
        });
        let r = stage._cumulative_rates();
        assert!((r.slr - 0.75).abs() < 1e-9);
        assert!((r.hfr - (23.0 / 30.0)).abs() < 1e-9);
        assert!((r.phase_adherence - (7.0 / 8.0)).abs() < 1e-9);
        assert_eq!(stage._n_cycles(), 2);
    }

    #[test]
    fn test_latest_rates_is_last_sample() {
        let mut stage = _ActivationAuditStage::new();
        stage.record(_ActivationAuditSample {
            activation_attempts: 10,
            activation_successes: 1,
            ..Default::default()
        });
        stage.record(_ActivationAuditSample {
            activation_attempts: 10,
            activation_successes: 9,
            ..Default::default()
        });
        let latest = stage._latest_rates().unwrap();
        assert!((latest.slr - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_report_health_thresholds() {
        let mut crit = _ActivationAuditStage::new();
        crit.record(_ActivationAuditSample {
            activation_attempts: 100,
            activation_successes: 25,
            ..Default::default()
        });
        assert_eq!(crit.report().health, _AuditHealth::Critical);

        let mut deg = _ActivationAuditStage::new();
        deg.record(_ActivationAuditSample {
            activation_attempts: 100,
            activation_successes: 60,
            ..Default::default()
        });
        assert_eq!(deg.report().health, _AuditHealth::Degraded);

        let mut ok = _ActivationAuditStage::new();
        ok.record(_ActivationAuditSample {
            activation_attempts: 100,
            activation_successes: 95,
            ..Default::default()
        });
        assert_eq!(ok.report().health, _AuditHealth::Healthy);
    }

    #[test]
    fn test_report_zero_cycle_healthy_default() {
        let stage = _ActivationAuditStage::new();
        let rep = stage.report();
        assert_eq!(rep._n_cycles, 0);
        assert_eq!(rep.health, _AuditHealth::Healthy);
    }
}
