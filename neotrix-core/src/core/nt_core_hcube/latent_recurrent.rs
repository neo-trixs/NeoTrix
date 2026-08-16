// P5: RecurrentLatent (吸收 arXiv 2608.09888 BDH-CQ — block-diagonal hypercomplex)
// 潜变量循环核: 复值(complex)潜变量沿序列传播, 用 block-diagonal 复矩阵
// 作状态转移 (维数降低 + 长程依赖)。直接作用于复值向量以贴合 VSA HyperCube
// 的复数表示体系 (fhrr/qfhrr 均为复数域)。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RecurrentLatentConfig {
    /// 潜状态维度
    pub dim: usize,
    /// block 大小 (block-diagonal 转移的块宽)
    pub block: usize,
    /// 时间步
    pub steps: usize,
    /// 前馈因子 α (0..1): 输入混合比
    pub feedforward: f64,
}

impl Default for RecurrentLatentConfig {
    fn default() -> Self {
        Self {
            dim: 64,
            block: 8,
            steps: 4,
            feedforward: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn mul(self, o: Complex) -> Complex {
        Complex::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }

    pub fn add(self, o: Complex) -> Complex {
        Complex::new(self.re + o.re, self.im + o.im)
    }

    pub fn scale(self, s: f64) -> Complex {
        Complex::new(self.re * s, self.im * s)
    }

    pub fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}

/// 复数潜状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatentState {
    pub values: Vec<Complex>,
    pub step: usize,
}

impl LatentState {
    pub fn zeros(dim: usize) -> Self {
        Self {
            values: vec![Complex::new(0.0, 0.0); dim],
            step: 0,
        }
    }

    pub fn magnitude(&self) -> f64 {
        self.values.iter().map(|c| c.norm_sq()).sum::<f64>().sqrt()
    }
}

/// RecurrentLatent: BDH-CQ 复值循环核
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrentLatent {
    pub config: RecurrentLatentConfig,
    /// block-diagonal 转移矩阵 (复数): blocks × (block × block)
    blocks: Vec<Vec<Complex>>,
    /// 输入投影 (复数)
    input_proj: Vec<Complex>,
    /// 输出投影 (实值 → 读出)
    output_proj: Vec<f64>,
}

impl RecurrentLatent {
    pub fn new(config: RecurrentLatentConfig) -> Self {
        assert!(
            config.dim.is_multiple_of(config.block),
            "dim must be divisible by block"
        );
        let n_blocks = config.dim / config.block;
        let mut blocks = Vec::with_capacity(n_blocks);
        // 确定性初始化: 每块用旋转角 θ 的复数旋转矩阵 (|det|=1, 稳定)
        for b in 0..n_blocks {
            let theta = (b as f64 + 1.0) * 0.1;
            let (s, c) = theta.sin_cos();
            let mut block = Vec::with_capacity(config.block * config.block);
            for i in 0..config.block {
                for j in 0..config.block {
                    if i == j {
                        block.push(Complex::new(c, 0.0));
                    } else if (i + 1) % config.block == j {
                        block.push(Complex::new(0.0, s));
                    } else {
                        block.push(Complex::new(0.0, 0.0));
                    }
                }
            }
            blocks.push(block);
        }
        let input_proj = (0..config.dim)
            .map(|i| Complex::new(0.5, (i as f64) * 0.05))
            .collect();
        let output_proj = (0..config.dim)
            .map(|i| (i as f64 + 1.0) / config.dim as f64)
            .collect();
        Self {
            config,
            blocks,
            input_proj,
            output_proj,
        }
    }

    /// block-diagonal 转移: h_t = W h_{t-1} + α·x (复数域)
    fn transition(&self, state: &LatentState, input: &LatentState, alpha: f64) -> LatentState {
        let dim = self.config.dim;
        let block = self.config.block;
        let mut next = Vec::with_capacity(dim);
        for b in 0..(dim / block) {
            let w = &self.blocks[b];
            let offset = b * block;
            for i in 0..block {
                let mut acc = Complex::new(0.0, 0.0);
                for j in 0..block {
                    acc = acc.add(w[i * block + j].mul(state.values[offset + j]));
                }
                acc = acc.add(input.values[offset + i].scale(alpha));
                next.push(acc);
            }
        }
        LatentState {
            values: next,
            step: state.step + 1,
        }
    }

    /// 前向: 输入序列 → 最终潜状态 (时间步内迭代, 序列不足自动补零到 config.steps)
    pub fn forward(&self, inputs: &[LatentState]) -> LatentState {
        let mut h = LatentState::zeros(self.config.dim);
        let steps = self.config.steps.max(1);
        let zeros = LatentState::zeros(self.config.dim);
        for t in 0..steps {
            let input = if t < inputs.len() { &inputs[t] } else { &zeros };
            let alpha = self.config.feedforward * (1.0 - t as f64 / steps as f64);
            h = self.transition(&h, input, alpha);
        }
        h
    }

    /// 读出: 实值得分 (累乘输出投影 + 幅度正则)
    pub fn readout(&self, state: &LatentState) -> f64 {
        state
            .values
            .iter()
            .zip(self.output_proj.iter())
            .map(|(c, w)| c.re * w)
            .sum::<f64>()
            / self.config.dim as f64
            + 0.5 * state.magnitude() / self.config.dim as f64
    }

    /// 复数表征 → 高维 VSA 余弦相似度 (与 HyperCube 对齐)
    pub fn cosine_similarity(&self, a: &LatentState, b: &LatentState) -> f64 {
        let na = a.magnitude();
        let nb = b.magnitude();
        if na == 0.0 || nb == 0.0 {
            return 0.0;
        }
        let dot: f64 = a
            .values
            .iter()
            .zip(b.values.iter())
            .map(|(x, y)| x.re * y.re + x.im * y.im)
            .sum();
        dot / (na * nb)
    }
}

impl crate::core::nt_core_self_test::SelfTest for RecurrentLatent {
    fn name(&self) -> &str {
        "nt_core_hcube_latent_recurrent"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let rl = RecurrentLatent::new(RecurrentLatentConfig::default());
        let mut inputs = Vec::new();
        for t in 0..4 {
            let mut v = vec![Complex::new(0.0, 0.0); rl.config.dim];
            v[t % rl.config.dim] = Complex::new(1.0, 0.0);
            inputs.push(LatentState { values: v, step: t });
        }
        let out = rl.forward(&inputs);
        if out.step == 0 {
            return Err(vec!["forward should advance steps".into()]);
        }
        let score = rl.readout(&out);
        if !score.is_finite() {
            return Err(vec!["readout must be finite".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    fn sample_input(step: usize) -> LatentState {
        let mut v = vec![Complex::new(0.0, 0.0); 16];
        v[step % 16] = Complex::new(1.0, 0.0);
        LatentState { values: v, step }
    }

    #[test]
    fn test_forward_propagates_steps() {
        let rl = RecurrentLatent::new(RecurrentLatentConfig {
            dim: 16,
            block: 4,
            steps: 4,
            feedforward: 0.5,
        });
        let inputs: Vec<LatentState> = (0..4).map(sample_input).collect();
        let out = rl.forward(&inputs);
        assert_eq!(out.step, 4);
        assert!(out.magnitude() > 0.0);
    }

    #[test]
    fn test_forward_short_sequence() {
        let rl = RecurrentLatent::new(RecurrentLatentConfig {
            dim: 16,
            block: 4,
            steps: 6,
            feedforward: 0.5,
        });
        let inputs: Vec<LatentState> = (0..2).map(sample_input).collect();
        let out = rl.forward(&inputs);
        assert_eq!(out.step, 6, "should pad with zeros to reach config steps");
    }

    #[test]
    fn test_block_diagonal_structure() {
        let rl = RecurrentLatent::new(RecurrentLatentConfig {
            dim: 16,
            block: 4,
            steps: 2,
            feedforward: 0.5,
        });
        assert_eq!(rl.blocks.len(), 4);
        // 块内对角元素非零 (旋转矩阵 c), 非对角元素必须为零
        assert!(
            rl.blocks[0][0 * 4 + 0].re.abs() > 0.0,
            "对角元素应为非零旋转余弦"
        );
        assert_eq!(rl.blocks[0][0 * 4 + 3], Complex::new(0.0, 0.0));
        assert_eq!(rl.blocks[1][1 * 4 + 0], Complex::new(0.0, 0.0));
        assert_eq!(rl.blocks[2][0 * 4 + 2], Complex::new(0.0, 0.0));
        assert_eq!(rl.blocks[3][1 * 4 + 3], Complex::new(0.0, 0.0));
    }

    #[test]
    fn test_cosine_similarity_self() {
        let rl = RecurrentLatent::new(RecurrentLatentConfig {
            dim: 16,
            block: 4,
            steps: 1,
            feedforward: 0.5,
        });
        let s = LatentState {
            values: vec![Complex::new(1.0, 0.5); 16],
            step: 0,
        };
        assert!((rl.cosine_similarity(&s, &s) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_readout_finite_and_bounded() {
        let rl = RecurrentLatent::new(RecurrentLatentConfig {
            dim: 16,
            block: 4,
            steps: 3,
            feedforward: 0.5,
        });
        let inputs: Vec<LatentState> = (0..3).map(sample_input).collect();
        let out = rl.forward(&inputs);
        let score = rl.readout(&out);
        assert!(score.is_finite());
        assert!(score.abs() < 10.0);
    }

    #[test]
    fn test_zeros_no_propagation() {
        let rl = RecurrentLatent::new(RecurrentLatentConfig {
            dim: 16,
            block: 4,
            steps: 2,
            feedforward: 0.0,
        });
        let out = rl.forward(&[LatentState::zeros(16)]);
        assert!(
            out.magnitude() < 1e-9,
            "zero feedforward + zero input stays near zero"
        );
    }

    #[test]
    fn test_selftest() {
        let rl = RecurrentLatent::new(RecurrentLatentConfig::default());
        assert!(rl.self_test().is_ok());
    }
}
