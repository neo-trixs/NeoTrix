//! Phase 6.3 — LatentReasoningTransformer (轻量循环潜在推理 Transformer).
//!
//! Thinking Pixel (arXiv:2604.25299) §3.3 + Scaling TTC with Latent Reasoning
//! (NeurIPS 2025) §3.2: 在连续潜在空间内做循环深度推理。一个潜在状态向量
//! $h_t \in \mathbb{R}^d$ 经残差前馈块 $h_{t+1} = (1-\tau) h_t + \tau f_\theta(h_t)$
//! 迭代更新; 每推进一步累积按深度折扣的奖励 $R = \sum_d \gamma^d \cdot m_d$,
//! $m_d$ 为相邻步隐藏状态变化范数 (推理推进量)。权重由固定 seed 生成 (He 风格),
//! 保证确定性 —— 与 `SeededProjection` 的 LCG 模式一致。
//!
//! 循环映射是压缩的 ($\tau < 1$, 权重范数 < 1), 因此更深推理向不动点几何收敛,
//! 更深轨迹累积更高奖励 —— 循环深度带来可验证的增益 (测试 test_depth_scaling)。

use serde::{Deserialize, Serialize};

/// 循环深度上限 (recursive depth cap)。
pub const MAX_LATENT_DEPTH: usize = 8;

/// 隐藏维度, 对齐 `UnifiedLatentSpace` 的 E₈ 潜在嵌入 (64-d native latent)。
pub const LATENT_HIDDEN_DIM: usize = 64;

/// 每步前馈块层数 (轻量: 2 层)。
pub const NUM_LATENT_LAYERS: usize = 2;

/// 递归深度奖励折扣因子 γ。
pub const RECURSIVE_REWARD_DISCOUNT: f64 = 0.9;

/// 默认温度 τ: 残差混合权重, 越小映射越压缩。
pub const DEFAULT_LATENT_TEMPERATURE: f64 = 0.1;

/// 权重初始化尺度 (He 风格, 收缩到 <1 保证压缩性)。
const WEIGHT_STD_SCALE: f64 = 0.25;

/// 单个潜在推理状态: 隐藏向量 + 循环深度 + 累积递归奖励。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentState {
    /// 隐藏状态向量 (维度恒等于 hidden_dim)。
    pub vector: Vec<f64>,
    /// 当前循环深度 (从 0 起, 每步 +1)。
    pub depth: usize,
    /// 累积奖励 (recursive depth reward, 按深度折扣)。
    pub accumulated_reward: f64,
}

/// Phase 6.3 — 轻量循环潜在推理 Transformer。
///
/// 纯数学, 无 unsafe, 无外部依赖: 权重由构造时的固定 seed LCG 生成, 同一
/// 参数构造的实例在任意机器上产生相同轨迹 (可复现性)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentReasoningTransformer {
    /// 每步前馈块层数。
    pub layers: usize,
    /// 隐藏维度。
    pub hidden_dim: usize,
    /// 循环深度上限。
    pub max_depth: usize,
    /// 残差温度 τ。
    pub temperature: f64,
    /// 推理轨迹 (每步一个 LatentState, 含初始态)。
    pub states: Vec<LatentState>,
    /// 总推进步数。
    pub total_steps: u64,
    /// 每层权重矩阵 (layers × hidden_dim × hidden_dim, 行主序)。
    #[serde(skip)]
    weights: Vec<Vec<f64>>,
    /// 每层偏置 (layers × hidden_dim)。
    #[serde(skip)]
    biases: Vec<Vec<f64>>,
}

impl Default for LatentReasoningTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl LatentReasoningTransformer {
    /// 用默认参数构造: 2 层, 64 维, 深度上限 8, 温度 0.1。
    pub fn new() -> Self {
        Self::new_with(NUM_LATENT_LAYERS, LATENT_HIDDEN_DIM, MAX_LATENT_DEPTH)
    }

    /// 用显式参数构造 (layers / hidden_dim / max_depth)。
    pub fn new_with(layers: usize, hidden_dim: usize, max_depth: usize) -> Self {
        Self::with_temperature(layers, hidden_dim, max_depth, DEFAULT_LATENT_TEMPERATURE)
    }

    /// 显式参数 + 温度构造。
    pub fn with_temperature(
        layers: usize,
        hidden_dim: usize,
        max_depth: usize,
        temperature: f64,
    ) -> Self {
        let layers = layers.max(1);
        let hidden_dim = hidden_dim.max(1);
        let max_depth = max_depth.max(1);
        let mut rng = Lcg::new(0x4E54_6C61 ^ hidden_dim as u64 ^ layers as u64);
        let std = WEIGHT_STD_SCALE / (hidden_dim as f64).sqrt();
        let mut weights = Vec::with_capacity(layers);
        let mut biases = Vec::with_capacity(layers);
        for _ in 0..layers {
            let mut w = Vec::with_capacity(hidden_dim * hidden_dim);
            for _ in 0..hidden_dim * hidden_dim {
                w.push((rng.next01() * 2.0 - 1.0) * std);
            }
            weights.push(w);
            let mut b = Vec::with_capacity(hidden_dim);
            for _ in 0..hidden_dim {
                b.push((rng.next01() * 2.0 - 1.0) * 0.1);
            }
            biases.push(b);
        }
        Self {
            layers,
            hidden_dim,
            max_depth,
            temperature: temperature.clamp(0.0, 1.0),
            states: Vec::new(),
            total_steps: 0,
            weights,
            biases,
        }
    }

    /// 当前循环深度 (无轨迹时为 0)。
    pub fn current_depth(&self) -> usize {
        self.states.last().map(|s| s.depth).unwrap_or(0)
    }

    /// 将任意长度输入对齐到 hidden_dim (短则零填充, 长则截断)。
    fn resize_to_hidden(&self, input: &[f64]) -> Vec<f64> {
        let mut v = vec![0.0f64; self.hidden_dim];
        for (i, &x) in input.iter().enumerate().take(self.hidden_dim) {
            v[i] = x;
        }
        v
    }

    /// 前馈块: 逐层 tanh(线性), 输出非线性变换结果。
    fn apply_block(&self, x: &[f64]) -> Vec<f64> {
        let mut h = x.to_vec();
        for l in 0..self.layers {
            let w = &self.weights[l];
            let b = &self.biases[l];
            let mut next = vec![0.0f64; self.hidden_dim];
            for i in 0..self.hidden_dim {
                let row = &w[i * self.hidden_dim..(i + 1) * self.hidden_dim];
                let mut acc = b[i];
                for (j, wgt) in row.iter().enumerate() {
                    acc += wgt * h[j];
                }
                next[i] = acc.tanh();
            }
            h = next;
        }
        h
    }

    /// 欧氏距离。
    fn euclid(&self, a: &[f64], b: &[f64]) -> f64 {
        let mut s = 0.0f64;
        for (x, y) in a.iter().zip(b.iter()) {
            let d = x - y;
            s += d * d;
        }
        s.sqrt()
    }

    /// 递归深度奖励: 第 `depth` 步 (0 起) 推进 `magnitude` 的折扣贡献 γ^depth · m。
    pub fn recursive_depth_reward(&self, magnitude: f64, depth: usize) -> f64 {
        RECURSIVE_REWARD_DISCOUNT.powf(depth as f64) * magnitude
    }

    /// 前推一步: 残差连接 + 前馈线性变换, 输出更新后的隐藏状态, 深度 +1。
    ///
    /// 累积奖励 = 上一步累积 + γ^depth · ‖Δh‖ (相邻步变化范数 = 推理推进量)。
    pub fn step(&mut self, input: &[f64]) -> LatentState {
        let x = self.resize_to_hidden(input);
        let (prev_vec, prev_depth, prev_reward) = match self.states.last() {
            Some(s) => (s.vector.clone(), s.depth, s.accumulated_reward),
            None => (vec![0.0f64; self.hidden_dim], 0usize, 0.0f64),
        };
        let g = self.apply_block(&x);
        let mut new_vec = Vec::with_capacity(self.hidden_dim);
        for i in 0..self.hidden_dim {
            new_vec.push((1.0 - self.temperature) * x[i] + self.temperature * g[i]);
        }
        let magnitude = self.euclid(&new_vec, &prev_vec);
        let depth = prev_depth + 1;
        let accumulated_reward = prev_reward + self.recursive_depth_reward(magnitude, prev_depth);
        let state = LatentState {
            vector: new_vec,
            depth,
            accumulated_reward,
        };
        self.states.push(state.clone());
        self.total_steps += 1;
        state
    }

    /// 循环推理: 从初始潜在态开始, 反复 step 直到深度达 min(max_steps, max_depth)。
    ///
    /// 每个调用视为一次独立 episode (先清空轨迹)。返回最终隐藏状态。
    pub fn reason(&mut self, initial: &[f64], max_steps: usize) -> LatentState {
        self.states.clear();
        self.total_steps = 0;
        let init_vec = self.resize_to_hidden(initial);
        self.states.push(LatentState {
            vector: init_vec,
            depth: 0,
            accumulated_reward: 0.0,
        });
        let limit = max_steps.min(self.max_depth);
        let mut last = self.states.last().expect("initial state pushed").clone();
        for _ in 0..limit {
            let input = last.vector.clone();
            last = self.step(&input);
        }
        last
    }

    /// 相邻步隐藏状态变化范数 (衡量推理推进量)。
    ///
    /// 若 `state` 属于当前轨迹, 返回它与前一步的欧氏距离; 首步返回自身范数。
    pub fn step_magnitude(&self, state: &LatentState) -> f64 {
        let idx = self
            .states
            .iter()
            .position(|s| s.vector == state.vector && s.depth == state.depth);
        match idx {
            Some(0) => self.euclid(&state.vector, &vec![0.0f64; self.hidden_dim]),
            Some(i) => self.euclid(&self.states[i - 1].vector, &state.vector),
            None => 0.0,
        }
    }

    /// 收敛检测: 相邻步隐藏状态变化 < tol。
    pub fn is_converged(&self, tol: f64) -> bool {
        if self.states.len() < 2 {
            return false;
        }
        let n = self.states.len();
        self.euclid(&self.states[n - 1].vector, &self.states[n - 2].vector) < tol
    }

    /// 清空轨迹与步数。
    pub fn reset(&mut self) {
        self.states.clear();
        self.total_steps = 0;
    }
}

/// 确定性线性同余生成器 (与 unified_latent::SeededProjection 同模式)。
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(
            seed.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407),
        )
    }
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) & 0x7FFF_FFFF
    }
    fn next01(&mut self) -> f64 {
        self.next() as f64 / 0x7FFF_FFFF as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_e8::unified_latent::UnifiedLatentSpace;
    use crate::core::nt_core_hex::ReasoningHexagram;

    fn cosine(a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let mut dot = 0.0f64;
        let mut na = 0.0f64;
        let mut nb = 0.0f64;
        for (x, y) in a.iter().zip(b.iter()) {
            dot += x * y;
            na += x * x;
            nb += y * y;
        }
        let denom = na.sqrt() * nb.sqrt();
        if denom == 0.0 {
            0.0
        } else {
            (dot / denom).clamp(-1.0, 1.0)
        }
    }

    #[test]
    fn test_initial_state_zero_depth() {
        let t = LatentReasoningTransformer::new();
        assert!(t.states.is_empty());
        assert_eq!(t.current_depth(), 0);
        assert_eq!(t.total_steps, 0);
        assert_eq!(t.hidden_dim, LATENT_HIDDEN_DIM);
        assert_eq!(t.max_depth, MAX_LATENT_DEPTH);
    }

    #[test]
    fn test_step_advances_depth() {
        let mut t = LatentReasoningTransformer::new();
        let input = vec![0.5f64; LATENT_HIDDEN_DIM];
        let s1 = t.step(&input);
        assert_eq!(s1.depth, 1);
        assert_eq!(t.current_depth(), 1);
        assert_eq!(t.states.len(), 1);
        let s2 = t.step(&input);
        assert_eq!(s2.depth, 2);
        assert_eq!(t.states.len(), 2);
        assert_eq!(t.total_steps, 2);
    }

    #[test]
    fn test_reason_respects_max_depth() {
        let mut t = LatentReasoningTransformer::new();
        let final_state = t.reason(&vec![0.3f64; LATENT_HIDDEN_DIM], 100);
        assert_eq!(final_state.depth, MAX_LATENT_DEPTH);
        assert!(t.states.iter().all(|s| s.depth <= MAX_LATENT_DEPTH));
        assert_eq!(t.states.len(), MAX_LATENT_DEPTH + 1);
    }

    #[test]
    fn test_reason_accumulates_reward() {
        let mut t = LatentReasoningTransformer::new();
        let s = t.reason(&vec![0.7f64; LATENT_HIDDEN_DIM], 3);
        assert!(s.accumulated_reward > 0.0);
        assert_eq!(s.depth, 3);
    }

    #[test]
    fn test_depth_scaling() {
        let mut t_shallow = LatentReasoningTransformer::new();
        let mut t_deep = LatentReasoningTransformer::new();
        let shallow = t_shallow.reason(&vec![0.7f64; LATENT_HIDDEN_DIM], 2);
        let deep = t_deep.reason(&vec![0.7f64; LATENT_HIDDEN_DIM], 6);
        assert!(
            deep.accumulated_reward > shallow.accumulated_reward,
            "deeper reasoning must accumulate more reward: deep={:.6} shallow={:.6}",
            deep.accumulated_reward,
            shallow.accumulated_reward
        );
    }

    #[test]
    fn test_convergence_detection() {
        let mut t = LatentReasoningTransformer::with_temperature(
            2,
            LATENT_HIDDEN_DIM,
            400,
            DEFAULT_LATENT_TEMPERATURE,
        );
        t.reason(&vec![1.0f64; LATENT_HIDDEN_DIM], 400);
        assert!(
            t.is_converged(1e-4),
            "contraction should drive steps toward a fixed point"
        );
    }

    #[test]
    fn test_reset_clears_trajectory() {
        let mut t = LatentReasoningTransformer::new();
        t.reason(&vec![0.4f64; LATENT_HIDDEN_DIM], 5);
        assert!(!t.states.is_empty());
        assert!(t.total_steps > 0);
        t.reset();
        assert!(t.states.is_empty());
        assert_eq!(t.total_steps, 0);
        assert_eq!(t.current_depth(), 0);
    }

    #[test]
    fn test_latent_vector_dimension_stable() {
        let mut t = LatentReasoningTransformer::new();
        for step in 0..5usize {
            let input = vec![0.2 + step as f64 * 0.1; LATENT_HIDDEN_DIM];
            let s = t.step(&input);
            assert_eq!(s.vector.len(), LATENT_HIDDEN_DIM);
        }
        assert_eq!(t.current_depth(), 5);
    }

    #[test]
    fn test_latent_coherence() {
        let u = UnifiedLatentSpace::new();
        // E8 近邻态 (一位翻转) 与远邻态 (六位翻转) 在统一潜在空间中的嵌入。
        let e_near_a = u.project_e8_state(ReasoningHexagram::new(0));
        let e_near_b = u.project_e8_state(ReasoningHexagram::new(1));
        let e_far = u.project_e8_state(ReasoningHexagram::new(63));
        assert_eq!(e_near_a.len(), u.dim);

        let mut t_a = LatentReasoningTransformer::new_with(2, u.dim, 8);
        let mut t_b = LatentReasoningTransformer::new_with(2, u.dim, 8);
        let mut t_far = LatentReasoningTransformer::new_with(2, u.dim, 8);
        let fa = t_a.reason(&e_near_a, 4);
        let fb = t_b.reason(&e_near_b, 4);
        let ff = t_far.reason(&e_far, 4);

        let sim_near = cosine(&fa.vector, &fb.vector);
        let sim_far = cosine(&fa.vector, &ff.vector);
        assert!(
            sim_near > sim_far,
            "similar inputs must stay closer in latent space: near={sim_near:.4} far={sim_far:.4}"
        );
    }

    #[test]
    fn test_deterministic_reproduction() {
        let input = vec![0.6f64; LATENT_HIDDEN_DIM];
        let mut t1 = LatentReasoningTransformer::new();
        let mut t2 = LatentReasoningTransformer::new();
        let s1 = t1.reason(&input, 5);
        let s2 = t2.reason(&input, 5);
        assert_eq!(
            s1.vector, s2.vector,
            "same params must reproduce the same trajectory"
        );
        assert!((s1.accumulated_reward - s2.accumulated_reward).abs() < 1e-12);
    }
}
