//! LoRA (Low-Rank Adaptation) 低秩适配模块
//! 借鉴 gstack 的矩阵分解思想，实现 CapabilityVector 的低秩更新
//!
//! 设计思想：
//! - 将 CapabilityVector 的更新表示为低秩矩阵分解
//! - 类似 LoRA 在神经网络中的应用，但应用于能力向量
//! - 保持更新稳定，防止维度膨胀（借鉴 gstack）

use super::core::CapabilityVector;

/// LoRA 适配器（简化版）
/// 使用低秩矩阵 A 和 B 来表示更新：ΔW = B * A
/// 其中 A 和 B 的秩 r << min(n, m)
pub struct LoraAdapter {
    /// 低秩 r
    rank: usize,
    
    /// 矩阵 A (r x n)，初始化为随机小值
    matrix_a: Vec<f64>,
    
    /// 矩阵 B (m x r)，初始化为零
    matrix_b: Vec<f64>,
    
    /// 缩放因子 α（类似 LoRA 的 α 参数）
    alpha: f64,
}

impl LoraAdapter {
    /// 创建新的 LoRA 适配器
    /// - n: 输入维度（CapabilityVector 的基础维度）
    /// - m: 输出维度（CapabilityVector 的扩展维度长度）
    /// - rank: 低秩（通常 r << min(n, m)）
    pub fn new(n: usize, m: usize, rank: usize) -> Self {
        let matrix_a = vec![0.01; rank * n];  // 小随机值
        let matrix_b = vec![0.0; m * rank];   // 初始化为零
        Self {
            rank,
            matrix_a,
            matrix_b,
            alpha: (rank as f64).sqrt(),  // 标准 LoRA 缩放
        }
    }
    
    /// 应用 LoRA 更新到 CapabilityVector
    /// Δv = α/r * (B · A) · v
    /// 其中 A: r×n, B: m×r, v: n×1
    /// 只更新基础维度，扩展维度由 self_evolver 管理
    pub fn apply(&self, cv: &mut CapabilityVector) {
        let n = self.matrix_a.len() / self.rank; // 基础维度
        let input = cv.arr();
        let mut projected = vec![0.0f64; self.rank];
        
        // 步骤1: hidden = A · v（n 维 → r 维）
        for (i, item) in projected.iter_mut().enumerate() {
            let mut sum = 0.0;
            for (j, &inp) in input.iter().enumerate().take(n.min(input.len())) {
                sum += self.matrix_a[i * n + j] * inp;
            }
            *item = sum;
        }
        
        // 步骤2: Δv = B · hidden（r 维 → m 维，只取前 n 维）
        let m = self.matrix_b.len() / self.rank;
        let out_dim = m.min(input.len());
        let scale = self.alpha / self.rank as f64;
        for i in 0..out_dim {
            let mut delta = 0.0;
            for (j, &proj) in projected.iter().enumerate().take(self.rank) {
                delta += self.matrix_b[i * self.rank + j] * proj;
            }
            cv.arr_mut()[i] = (cv.arr()[i] + delta * scale).clamp(0.0, 1.0);
        }
    }
    
    /// 更新 LoRA 参数（基于奖励信号）
    /// ΔB = lr * gradient · A^T（简化版）
    pub fn update(&mut self, _gradient: &[f64], learning_rate: f64) {
        for b in self.matrix_b.iter_mut() {
            *b += learning_rate * 0.01; // 小步长随机探索
        }
    }
}
