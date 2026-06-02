//! Active Inference Engine — Friston Free Energy Principle
//!
//! Normative drive for the consciousness kernel:
//!   variational free energy = prediction error - epistemic value
//!
//! Core equation:
//!   F = β·E_JEPA - H(E8)/T + γ·|∇E8|
//!
//! Where:
//!   - β: nt_world_sense precision (how much we trust observations)
//!   - E_JEPA: JEPA prediction energy (prediction error in latent space)
//!   - H(E8): E8 state entropy (uncertainty about hidden states)
//!   - T: temperature (exploration vs. exploitation balance)
//!   - γ·|∇E8|: novelty/transient detection (energy gradient penalty)
//!
//! Policy selection via expected free energy minimization:
//!   G(π) = -E_q[I(o;s|π)] + E_q[E_JEPA(o|s,π)]
//!        = -epistemic_value + expected_prediction_error
//!
//! Low free energy = well-calibrated world model + optimal exploration.

use serde::{Deserialize, Serialize};

// ============================================================
// 常量
// ============================================================

/// 默认 nt_world_sense precision (β)
pub const DEFAULT_SENSORY_PRECISION: f64 = 1.0;

/// 默认温度 (T) — 高 = 更多探索
pub const DEFAULT_TEMPERATURE: f64 = 0.5;

/// 默认能量梯度权重 (γ)
pub const DEFAULT_GRADIENT_WEIGHT: f64 = 0.1;

/// 自由能移动平均窗口
pub const FE_WINDOW_SIZE: usize = 20;

/// 自由能下降趋势阈值 — 低于此值认为模型收敛
pub const FE_CONVERGENCE_THRESHOLD: f64 = 0.01;

// ============================================================
// 自由能报告
// ============================================================

/// 自由能分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeEnergyReport {
    /// 当前变分自由能 F
    pub variational_fe: f64,
    /// JEPA 预测能量贡献 (β·E_JEPA)
    pub prediction_energy: f64,
    /// E8 熵贡献 (-H(E8)/T) — 负值 = 不确定性降低
    pub entropy_contribution: f64,
    /// 能量梯度贡献 (γ·|∇E8|) — 瞬态检测
    pub gradient_penalty: f64,
    /// 认识价值 (epistemic value) — 信息增益的正分量
    pub epistemic_value: f64,
    /// 自由能趋势 (负值 = 下降 = 学习)
    pub fe_trend: f64,
    /// 模型是否收敛
    pub is_converged: bool,
}

// ============================================================
// Active Inference Engine
// ============================================================

/// Active Inference Engine — Friston 自由能原理实现
///
/// 提供规范性驱动力: 最小化自由能 = 最大化世界模型的预测精度 + 最优探索
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveInferenceEngine {
    /// Sensory precision β — 观测置信度 (高 = 更相信观测)
    pub nt_world_sense_precision: f64,
    /// 温度 T — 探索/利用平衡
    pub temperature: f64,
    /// 能量梯度权重 γ — 瞬态灵敏度
    pub gradient_weight: f64,
    /// 自由能历史 (用于趋势分析)
    pub fe_history: Vec<f64>,
    /// 当前自由能
    pub current_fe: f64,
    /// 学习步数
    pub step: usize,
}

impl Default for ActiveInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveInferenceEngine {
    pub fn new() -> Self {
        Self {
            nt_world_sense_precision: DEFAULT_SENSORY_PRECISION,
            temperature: DEFAULT_TEMPERATURE,
            gradient_weight: DEFAULT_GRADIENT_WEIGHT,
            fe_history: Vec::with_capacity(FE_WINDOW_SIZE),
            current_fe: 0.0,
            step: 0,
        }
    }

    /// 计算变分自由能:
    ///   F = β·E_JEPA - H(E8)/T + γ·|∇E8|
    ///
    /// # Arguments
    /// * `jepa_energy` - JEPA 预测能量 (从 predict() 得到)
    /// * `e8_entropy` - E8 状态熵 (从 e8.entropy() 得到)
    /// * `e8_energy_gradient` - E8 能量梯度 (当前能量 - 上次能量)
    pub fn compute_free_energy(
        &mut self,
        jepa_energy: f64,
        e8_entropy: f64,
        e8_energy_gradient: f64,
    ) -> FreeEnergyReport {
        let prediction_energy = self.nt_world_sense_precision * jepa_energy;
        let entropy_contribution = -e8_entropy / self.temperature.max(1e-8);
        let gradient_penalty = self.gradient_weight * e8_energy_gradient.abs();

        // 认识价值 = E8 熵的降低 (正 = 不确定性减少)
        let epistemic_value = e8_entropy * (1.0 - (-self.nt_world_sense_precision).exp());

        let fe = prediction_energy + entropy_contribution + gradient_penalty;
        self.current_fe = fe;
        self.step += 1;

        // 更新历史
        self.fe_history.push(fe);
        if self.fe_history.len() > FE_WINDOW_SIZE {
            self.fe_history.remove(0);
        }

        // 趋势分析: 最近 5 步的斜率
        let fe_trend = if self.fe_history.len() >= 5 {
            let n = self.fe_history.len();
            let recent = &self.fe_history[n - 5..];
            let half = recent.len() / 2;
            let first_half_avg: f64 = recent[..half].iter().sum::<f64>() / half as f64;
            let second_half_avg: f64 = recent[half..].iter().sum::<f64>() / (recent.len() - half) as f64;
            second_half_avg - first_half_avg
        } else {
            0.0
        };

        let is_converged = fe_trend.abs() < FE_CONVERGENCE_THRESHOLD && fe < 1.0;

        FreeEnergyReport {
            variational_fe: fe,
            prediction_energy,
            entropy_contribution,
            gradient_penalty,
            epistemic_value,
            fe_trend,
            is_converged,
        }
    }

    /// 计算预期自由能 G(π) 为给定策略:
    ///   G(π) = -epistemic_value + expected_prediction_error
    ///
    /// 用于策略选择: 选择 G 最小的策略
    pub fn expected_free_energy(&self, epistemic_value: f64, expected_energy: f64) -> f64 {
        -epistemic_value + expected_energy
    }

    /// 选择最优策略: 最小化预期自由能
    pub fn select_policy(&self, policy_costs: &[(String, f64)]) -> Option<String> {
        policy_costs.iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.clone())
    }

    /// 自由能移动平均
    pub fn fe_moving_avg(&self) -> f64 {
        if self.fe_history.is_empty() { return 0.0; }
        self.fe_history.iter().sum::<f64>() / self.fe_history.len() as f64
    }

    /// 重置引擎
    pub fn reset(&mut self) {
        self.fe_history.clear();
        self.current_fe = 0.0;
        self.step = 0;
    }

    /// 配置 nt_world_sense precision (置信度)
    pub fn with_precision(mut self, precision: f64) -> Self {
        self.nt_world_sense_precision = precision;
        self
    }

    /// 配置温度 (探索/利用)
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }
}

// ============================================================
// 测试
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_energy_basic_components() {
        let mut engine = ActiveInferenceEngine::new();
        let report = engine.compute_free_energy(0.5, 2.0, 0.1);
        assert!(report.variational_fe.is_finite());
        assert!(report.prediction_energy > 0.0);
        assert!(report.entropy_contribution < 0.0); // -H/T
        assert!(report.gradient_penalty >= 0.0);
    }

    #[test]
    fn test_free_energy_decreases_with_learning() {
        let mut engine = ActiveInferenceEngine::new();
        // Same entropy, lower energy → FE decreases
        let r1 = engine.compute_free_energy(1.0, 1.0, 0.1);
        let r2 = engine.compute_free_energy(0.1, 1.0, 0.01);
        assert!(r2.variational_fe < r1.variational_fe);
    }

    #[test]
    fn test_epistemic_value_positive() {
        let engine = ActiveInferenceEngine::new();
        let fe = engine.expected_free_energy(0.5, 0.3);
        assert!(fe.is_finite());
        assert!(fe < 0.3); // -0.5 + 0.3 = -0.2
    }

    #[test]
    fn test_policy_selection() {
        let engine = ActiveInferenceEngine::new();
        let policies = vec![
            ("hero_ui".to_string(), 0.8),
            ("base_ui".to_string(), 0.3),
            ("nt_shield".to_string(), 1.2),
        ];
        let selected = engine.select_policy(&policies);
        assert_eq!(selected, Some("base_ui".to_string()));
    }

    #[test]
    fn test_fe_trend_negative_when_improving() {
        let mut engine = ActiveInferenceEngine::new();
        // Same entropy, decreasing energy → FE decreases
        for i in 0..10 {
            let e = (10 - i) as f64 * 0.2;
            engine.compute_free_energy(e, 0.5, 0.01);
        }
        let last_report = engine.compute_free_energy(0.01, 0.5, 0.001);
        assert!(last_report.fe_trend < 0.0 || last_report.is_converged,
            "FE should trend down: trend={:.6}", last_report.fe_trend);
    }

    #[test]
    fn test_moving_average_stable() {
        let mut engine = ActiveInferenceEngine::new();
        // Use values that guarantee positive FE: low entropy, moderate energy
        for _ in 0..5 {
            engine.compute_free_energy(0.5, 0.1, 0.01);
        }
        let avg = engine.fe_moving_avg();
        assert!(avg.is_finite());
    }

    #[test]
    fn test_convergence_detection() {
        let mut engine = ActiveInferenceEngine::new();
        // Very low free energy over many steps
        for _ in 0..10 {
            engine.compute_free_energy(0.01, 0.05, 0.001);
        }
        let report = engine.compute_free_energy(0.01, 0.05, 0.001);
        assert!(report.is_converged);
    }

    #[test]
    fn test_precision_increases_prediction_weight() {
        let mut engine = ActiveInferenceEngine::with_precision(
            ActiveInferenceEngine::new(), 5.0
        );
        let report = engine.compute_free_energy(1.0, 1.0, 0.1);
        assert!((report.prediction_energy - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_temperature_scales_entropy() {
        let mut engine = ActiveInferenceEngine::with_temperature(
            ActiveInferenceEngine::new(), 10.0
        );
        // High T → entropy contribution is small in magnitude
        // -H/T = -5.0/10.0 = -0.5
        let report = engine.compute_free_energy(0.5, 5.0, 0.1);
        assert!((report.entropy_contribution - (-0.5)).abs() < 1e-10,
            "Expected -0.5, got {}", report.entropy_contribution);
    }

    #[test]
    fn test_empty_history_fe_moving_avg() {
        let engine = ActiveInferenceEngine::new();
        assert!((engine.fe_moving_avg() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_select_policy_empty() {
        let engine = ActiveInferenceEngine::new();
        assert!(engine.select_policy(&[]).is_none());
    }

    #[test]
    fn test_reset_clears_state() {
        let mut engine = ActiveInferenceEngine::new();
        engine.compute_free_energy(1.0, 1.0, 0.1);
        assert!(engine.step > 0);
        engine.reset();
        assert_eq!(engine.step, 0);
        assert!(engine.fe_history.is_empty());
    }
}
