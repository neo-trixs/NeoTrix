//! E8 隐藏世界模型 — 中国古代数理宇宙论的现代实现
//!
//! 基于 2026-05-24 综合提炼公式:
//!   Ψ(t+1) = R(T · H(Ψ(t), O(t)))
//!
//! 其中：
//!   - Ψ ∈ {0,1}^64 = 系统状态向量（64卦空间中的点）
//!   - O = 观察者位置（嵌入维度 +1）
//!   - H = Hadamard 变换（从状态→频率域）
//!   - T = 时间演化算子（模周期：12/24/60/360/129600）
//!   - R = 共振选择（仅共振频率耦合的状态可传播）
//!
//! 核心恒等式:
//!   - E₈ (248) ⊃ Spin(11,3) → 64 fermions per generation
//!   - 3 generations × 64 = 192 = 248 - 56
//!   - 64 hexagrams = weight diagram of 64-fermion rep
//!   - 大衍之数五十，其用四十有九 (+1 观察者)

use serde::{Deserialize, Serialize};

// ============================================================
// 常量
// ============================================================

/// 卦空间维度 = 64
pub const HEXAGRAM_DIM: usize = 64;

/// E₈ 维数
pub const E8_DIM: usize = 248;

/// 大衍之数 = 系统总自由度
pub const DAYAN_NUMBER: usize = 50;

/// 可用自由度 = 49
pub const OBSERVABLE_DOF: usize = 49;

/// 观察者自由度 = 1
pub const OBSERVER_DOF: usize = 1;

/// 时间周期: 12 (地支)
pub const PERIOD_12: f64 = 12.0;

/// 时间周期: 24 (节气)
pub const PERIOD_24: f64 = 24.0;

/// 时间周期: 60 (甲子)
pub const PERIOD_60: f64 = 60.0;

/// 时间周期: 360 (周天)
pub const PERIOD_360: f64 = 360.0;

/// 时间周期: 129600 (元)
pub const PERIOD_129600: f64 = 129600.0;

/// 共振阈值 (汉明距离 ≤ 2)
pub const RESONANCE_THRESHOLD: u32 = 2;

// ============================================================
// Hadamard 变换 (内部用定长数组)
// ============================================================

/// Walsh-Hadamard 变换 (顺序排列)
/// 将 64 维状态空间变换到频率域
pub fn hadamard_transform(state: &[f64; 64]) -> [f64; 64] {
    let mut result = *state;
    let mut len = 1;
    while len < 64 {
        for i in (0..64).step_by(len * 2) {
            for j in 0..len {
                let u = result[i + j];
                let v = result[i + j + len];
                result[i + j] = u + v;
                result[i + j + len] = u - v;
            }
        }
        len *= 2;
    }

    let norm = (64.0_f64).sqrt();
    for val in result.iter_mut() {
        *val /= norm;
    }

    result
}

/// 逆 Hadamard 变换（与正向相同，因为 Hadamard 是对称正交变换）
pub fn inverse_hadamard_transform(freq: &[f64; 64]) -> [f64; 64] {
    hadamard_transform(freq)
}

// ============================================================
// 辅助: Vec ↔ 定长数组
// ============================================================

fn vec_to_array64(v: &[f64]) -> [f64; 64] {
    let mut arr = [0.0; 64];
    for (i, val) in v.iter().enumerate().take(64) {
        arr[i] = *val;
    }
    arr
}

// ============================================================
// 时间演化算子
// ============================================================

/// 多模周期时间演化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEvolver {
    /// 当前全局时间
    pub global_time: f64,
}

impl Default for TimeEvolver {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeEvolver {
    pub fn new() -> Self {
        Self { global_time: 0.0 }
    }

    /// 生成时间相位向量 (64 维)
    /// 每 8 维一组对应一个周期
    pub fn phase_vector(&self) -> [f64; 64] {
        let mut phase = [0.0; 64];

        for (i, item) in phase[..8].iter_mut().enumerate() {
            *item = (2.0 * std::f64::consts::PI * self.global_time / PERIOD_12
                       + (i as f64) * std::f64::consts::PI / 4.0).sin();
        }

        for (i, item) in phase[8..16].iter_mut().enumerate() {
            *item = (2.0 * std::f64::consts::PI * self.global_time / PERIOD_24
                           + (i as f64) * std::f64::consts::PI / 4.0).sin();
        }

        for (i, item) in phase[16..24].iter_mut().enumerate() {
            *item = (2.0 * std::f64::consts::PI * self.global_time / PERIOD_60
                            + (i as f64) * std::f64::consts::PI / 4.0).sin();
        }

        for (i, item) in phase[24..32].iter_mut().enumerate() {
            *item = (2.0 * std::f64::consts::PI * self.global_time / PERIOD_360
                            + (i as f64) * std::f64::consts::PI / 4.0).sin();
        }

        for (i, item) in phase[32..40].iter_mut().enumerate() {
            *item = (2.0 * std::f64::consts::PI * self.global_time / PERIOD_129600
                            + (i as f64) * std::f64::consts::PI / 4.0).sin();
        }

        for i in 0..8 {
            phase[40 + i] = (2.0 * std::f64::consts::PI * self.global_time / (PERIOD_12 * PERIOD_24)
                            + (i as f64) * std::f64::consts::PI / 4.0).sin();
        }

        for i in 0..8 {
            phase[48 + i] = (2.0 * std::f64::consts::PI * self.global_time / (PERIOD_60 * PERIOD_360)
                            + (i as f64) * std::f64::consts::PI / 4.0).sin();
        }

        for i in 0..8 {
            phase[56 + i] = (2.0 * std::f64::consts::PI * self.global_time / PERIOD_129600
                            * (i as f64 + 1.0) / 8.0).sin();
        }

        phase
    }

    pub fn step(&mut self, dt: f64) {
        self.global_time += dt;
    }
}

// ============================================================
// 共振选择
// ============================================================

/// 共振选择器 — 仅共振频率耦合的状态可传播
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceSelector {
    pub threshold: u32,
}

impl Default for ResonanceSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl ResonanceSelector {
    pub fn new() -> Self {
        Self { threshold: RESONANCE_THRESHOLD }
    }

    pub fn hamming_distance(a: &[f64], b: &[f64]) -> u32 {
        let n = a.len().min(b.len()).min(64);
        let mut dist = 0;
        for i in 0..n {
            let a_bit = if a[i] > 0.0 { 1u8 } else { 0u8 };
            let b_bit = if b[i] > 0.0 { 1u8 } else { 0u8 };
            if a_bit != b_bit { dist += 1; }
        }
        dist
    }

    pub fn filter(&self, freq_domain: &[f64; 64], seed: &[f64]) -> [f64; 64] {
        let mut filtered = [0.0; 64];
        for i in 0..64 {
            let mut state = [0.0; 64];
            state[i] = 1.0;
            let dist = Self::hamming_distance(&state, seed);
            if dist <= self.threshold {
                filtered[i] = freq_domain[i];
            }
        }
        filtered
    }

    pub fn amplify(&self, state: &[f64], resonance_pattern: &[f64]) -> Vec<f64> {
        let n = state.len().min(resonance_pattern.len()).min(64);
        let mut amplified: Vec<f64> = state.to_vec();
        for i in 0..n {
            let bit_i = if state[i] > 0.0 { 1u8 } else { 0u8 };
            let bit_pat = if resonance_pattern.len() > i && resonance_pattern[i] > 0.0 { 1u8 } else { 0u8 };
            if (bit_i ^ bit_pat) <= 1 {
                amplified[i] *= 1.2;
            } else {
                amplified[i] *= 0.8;
            }
        }
        amplified
    }
}

// ============================================================
// 64 卦状态空间 (Vec<f64> 以支持 serde)
// ============================================================

/// 64 卦状态 — 在 64 维超立方体上的位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexagramState {
    /// 64 维状态向量
    pub vector: Vec<f64>,
    /// 当前主导卦 (最大激活)
    pub dominant: usize,
}

impl Default for HexagramState {
    fn default() -> Self {
        Self::new()
    }
}

impl HexagramState {
    pub fn new() -> Self {
        let mut vector = vec![0.0; 64];
        vector[11] = 1.0;
        Self { vector, dominant: 11 }
    }

    pub fn from_vector(v: &[f64]) -> Self {
        let _n = v.len().min(64);
        let mut vector: Vec<f64> = v.iter().copied().take(64).collect();
        while vector.len() < 64 { vector.push(0.0); }
        let max = vector.iter().cloned().fold(0.0_f64, f64::max);
        if max > 0.0 {
            for val in vector.iter_mut() { *val /= max; }
        }
        let dominant = (0..64).max_by(|&i, &j| vector[i].partial_cmp(&vector[j]).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(11);
        Self { vector, dominant }
    }

    /// 从 32 维 latent 空间投影到 64 卦空间
    pub fn from_latent(latent: &[f64]) -> Self {
        let mut vector = vec![0.0; 64];
        let n = latent.len().min(64);
        for (i, item) in vector.iter_mut().enumerate().take(n) { *item = latent[i].tanh(); }
        for (i, item) in vector.iter_mut().enumerate().skip(n) {
            *item = ((i as f64) / 64.0 * std::f64::consts::PI).sin() * 0.1;
        }
        let max = vector.iter().cloned().fold(0.0_f64, f64::max);
        if max > 0.0 { for val in vector.iter_mut() { *val /= max; } }
        let dominant = (0..64).max_by(|&i, &j| vector[i].partial_cmp(&vector[j]).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(11);
        Self { vector, dominant }
    }

    /// 观察者投影: 去掉一个维度（观察者自身）
    pub fn observer_projection(&self) -> Vec<f64> {
        let mut projected = Vec::with_capacity(49);
        for i in 0..64 {
            if i != OBSERVER_DOF { projected.push(self.vector[i]); }
        }
        projected
    }
}

// ============================================================
// E8 世界模型 — 主结构
// ============================================================

/// E8 隐藏世界模型 — 完整状态演化引擎
///
/// 方程: Ψ(t+1) = R(T · H(Ψ(t), O(t)))
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8WorldModel {
    pub current_state: HexagramState,
    pub time_evolver: TimeEvolver,
    pub resonance_selector: ResonanceSelector,
    pub observer_position: usize,
    pub evolution_step: usize,
    pub prediction_history: Vec<HexagramState>,
}

impl Default for E8WorldModel {
    fn default() -> Self {
        Self::new()
    }
}

impl E8WorldModel {
    pub fn new() -> Self {
        Self {
            current_state: HexagramState::new(),
            time_evolver: TimeEvolver::new(),
            resonance_selector: ResonanceSelector::new(),
            observer_position: OBSERVER_DOF,
            evolution_step: 0,
            prediction_history: Vec::with_capacity(100),
        }
    }

    /// 一步演化: Ψ(t+1) = R(T · H(Ψ(t), O(t)))
    pub fn evolve(&mut self, dt: f64) -> &HexagramState {
        let arr = vec_to_array64(&self.current_state.vector);
        let freq = hadamard_transform(&arr);
        let phase = self.time_evolver.phase_vector();

        let mut evolved_freq = [0.0; 64];
        for i in 0..64 {
            evolved_freq[i] = freq[i] * (1.0 + 0.1 * phase[i]);
        }

        let filtered = self.resonance_selector.filter(&evolved_freq, &self.current_state.vector);
        let new_state_arr = inverse_hadamard_transform(&filtered);

        let mut normalized = new_state_arr.to_vec();
        let max = normalized.iter().cloned().fold(0.0_f64, f64::max);
        if max > 0.0 { for val in normalized.iter_mut() { *val /= max; } }

        self.current_state = HexagramState::from_vector(&normalized);
        self.time_evolver.step(dt);
        self.evolution_step += 1;

        if self.prediction_history.len() < 100 {
            self.prediction_history.push(self.current_state.clone());
        }

        &self.current_state
    }

    /// 多步演化
    pub fn evolve_n(&mut self, n: usize, dt: f64) -> &HexagramState {
        for _ in 0..n { self.evolve(dt); }
        &self.current_state
    }

    /// 从 JEPA latent 状态初始化
    pub fn from_jepa_latent(&mut self, latent: &[f64]) {
        self.current_state = HexagramState::from_latent(latent);
    }

    /// 预测未来 N 步
    pub fn forecast(&self, n: usize, dt: f64) -> Vec<HexagramState> {
        let mut forecast_model = self.clone();
        let mut results = Vec::with_capacity(n);
        for _ in 0..n {
            forecast_model.evolve(dt);
            results.push(forecast_model.current_state.clone());
        }
        results
    }

    /// 状态熵
    pub fn entropy(&self) -> f64 {
        let total: f64 = self.current_state.vector.iter().sum();
        if total <= 0.0 { return 0.0; }
        -self.current_state.vector.iter()
            .filter(|&&v| v > 0.0)
            .map(|&v| { let p = v / total; p * p.log2() })
            .sum::<f64>()
    }

    /// 状态能量
    pub fn energy(&self) -> f64 {
        self.current_state.vector.iter().map(|&v| v * v).sum::<f64>()
    }

    /// 检测稳定态
    pub fn is_stable(&self, lookback: usize) -> bool {
        if self.prediction_history.len() < lookback { return false; }
        let start = self.prediction_history.len().saturating_sub(lookback);
        let recent = &self.prediction_history[start..];
        recent.iter().all(|s| s.dominant == self.current_state.dominant)
    }

    /// 融合 JEPA 预测到 E8 状态空间
    /// 将 JEPA 的 latent 预测投影到 64 卦，计算 E8 演化
    pub fn fuse_jepa_prediction(&self, jepa_latent: &[f64], steps: usize) -> Vec<f64> {
        let mut e8 = self.clone();
        e8.from_jepa_latent(jepa_latent);
        e8.evolve_n(steps, 1.0);
        // 返回投影到 observable 维度的状态
        e8.current_state.observer_projection()
    }
}

// ============================================================
// 测试
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hadamard_is_involutive() {
        let state = [1.0_f64; 64];
        let freq = hadamard_transform(&state);
        assert!((freq[0] - 8.0).abs() < 1e-10);
        let back = inverse_hadamard_transform(&freq);
        for i in 0..64 { assert!((back[i] - 1.0).abs() < 1e-10); }
    }

    #[test]
    fn test_e8_evolve() {
        let mut model = E8WorldModel::new();
        model.evolve(1.0);
        assert_eq!(model.current_state.vector.len(), 64);
        assert!(model.evolution_step == 1);
    }

    #[test]
    fn test_forecast() {
        let model = E8WorldModel::new();
        let f = model.forecast(5, 1.0);
        assert_eq!(f.len(), 5);
    }

    #[test]
    fn test_observer_projection() {
        let state = HexagramState::new();
        let p = state.observer_projection();
        // 64 维去掉 observer 维度 = 63
        assert_eq!(p.len(), 63);
    }

    #[test]
    fn test_from_latent() {
        let state = HexagramState::from_latent(&[0.5; 32]);
        assert_eq!(state.vector.len(), 64);
    }

    #[test]
    fn test_fuse_jepa() {
        let model = E8WorldModel::new();
        let fused = model.fuse_jepa_prediction(&[0.5; 32], 3);
        // 64 维去掉 observer 维度 = 63
        assert_eq!(fused.len(), 63);
    }

    #[test]
    fn test_entropy_bounded() {
        let model = E8WorldModel::new();
        let h = model.entropy();
        assert!(h >= 0.0);
        assert!(h.is_finite());
    }

    #[test]
    fn test_hexagram_normalization() {
        let mut v = vec![0.0; 64];
        v[5] = 3.0; v[10] = 1.0;
        let state = HexagramState::from_vector(&v);
        assert!((state.vector[5] - 1.0).abs() < 1e-10);
        assert!((state.vector[10] - 1.0 / 3.0).abs() < 1e-10);
        assert_eq!(state.dominant, 5);
    }

    #[test]
    fn test_energy_finite() {
        let mut model = E8WorldModel::new();
        model.evolve_n(10, 1.0);
        assert!(model.energy().is_finite());
    }

    #[test]
    fn test_resonance_selector_works_with_vec() {
        let _sel = ResonanceSelector::new();
        let a = vec![0.0; 64];
        let b = vec![1.0; 64];
        let d = ResonanceSelector::hamming_distance(&a, &b);
        assert_eq!(d, 64);
    }
}
