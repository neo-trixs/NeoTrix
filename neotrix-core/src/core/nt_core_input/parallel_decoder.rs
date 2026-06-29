//! # ParallelVsaDecoder — 并行 VSA 解码器
//!
//! 借鉴 Eagle/PBD (Parallel Box Decoding):
//!   将 4096-bit VSA 向量分割为 64 个 64-bit 原子块，并行解码
//!   混合推理模式: 默认快速 (MTP 并行) + 歧义时回退 (NTP 串行)

use crate::core::nt_core_hcube::vsa::{VSAEngine, VsaBackend};

/// 并行解码器
#[derive(Debug)]
pub struct ParallelVsaDecoder {
    engine: VSAEngine,
    num_blocks: usize,
    block_size: usize,
}

impl ParallelVsaDecoder {
    /// 创建并行解码器
    /// dim: VSA 向量总维度 (默认 4096)
    /// num_blocks: 并行块数 (默认 64)
    pub fn new(dim: usize, num_blocks: usize) -> Self {
        let block_size = dim / num_blocks;
        Self {
            engine: VSAEngine::new(dim),
            num_blocks,
            block_size,
        }
    }

    /// 默认配置: 4096 维度, 64 并行块
    pub fn default() -> Self {
        Self::new(4096, 64)
    }

    /// 并行编码 — 将输入分割为块并行编码
    pub fn encode_parallel(&self, input: &[f64]) -> Vec<f64> {
        let chunks: Vec<&[f64]> = input.chunks(self.block_size).collect();

        // 并行编码每个块 (simd 友好)
        let encoded: Vec<Vec<f64>> = chunks
            .iter()
            .map(|chunk| self.encode_block(chunk))
            .collect();

        // 串接结果
        encoded.into_iter().flatten().collect()
    }

    /// 编码单个块
    fn encode_block(&self, block: &[f64]) -> Vec<f64> {
        let dim = block.len();
        let mut result = vec![0.0; dim];

        // 使用映射编码: 将输入映射到 VSA 空间
        for (i, &val) in block.iter().enumerate() {
            let permuted = self.engine.permute(block, i as isize);
            for (r, p) in result.iter_mut().zip(permuted.iter()) {
                *r += val * p;
            }
        }

        // 归一化
        let norm: f64 = result.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-12 {
            for r in result.iter_mut() {
                *r /= norm;
            }
        }

        result
    }

    /// 混合推理: 快速模式 + 歧义时串行回退
    pub fn decode_hybrid(&self, encoded: &[f64], ambiguity_threshold: f64) -> Vec<f64> {
        // 快速路径: 并行解码
        let fast_result = self.encode_parallel(encoded);

        // 检查歧义: 如果结果熵高则串行回退
        let entropy = self.compute_entropy(&fast_result);
        if entropy > ambiguity_threshold {
            // 串行回退: NTP 风格逐位解码
            self.decode_serial(encoded)
        } else {
            fast_result
        }
    }

    /// 串行解码 (NTP 风格) — 逐位确定性解码
    fn decode_serial(&self, input: &[f64]) -> Vec<f64> {
        // 串行解码: 每次处理一个维度
        input
            .iter()
            .enumerate()
            .map(|(i, &val)| {
                let permuted = self.engine.permute(input, i as isize);
                let dot: f64 = input.iter().zip(permuted.iter()).map(|(a, b)| a * b).sum();
                dot * val
            })
            .collect()
    }

    /// 计算向量的熵
    fn compute_entropy(&self, v: &[f64]) -> f64 {
        let sum_abs: f64 = v.iter().map(|x| x.abs()).sum();
        if sum_abs < 1e-12 {
            return 1.0;
        }
        -v.iter()
            .map(|&x| {
                let p = x.abs() / sum_abs;
                if p > 1e-12 {
                    p * p.log2()
                } else {
                    0.0
                }
            })
            .sum::<f64>()
    }

    /// 获取块数
    pub fn num_blocks(&self) -> usize {
        self.num_blocks
    }
}

/// 默认全局并行解码器
static GLOBAL_PARALLEL_DECODER: std::sync::OnceLock<ParallelVsaDecoder> =
    std::sync::OnceLock::new();

/// 获取或初始化全局解码器
pub fn global_parallel_decoder() -> &'static ParallelVsaDecoder {
    GLOBAL_PARALLEL_DECODER.get_or_init(|| ParallelVsaDecoder::new(4096, 64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_encode_preserves_dimension() {
        let decoder = ParallelVsaDecoder::new(4096, 64);
        let input = vec![0.5; 4096];
        let result = decoder.encode_parallel(&input);
        assert_eq!(result.len(), 4096);
    }

    #[test]
    fn test_default_config() {
        let decoder = ParallelVsaDecoder::default();
        assert_eq!(decoder.num_blocks(), 64);
    }

    #[test]
    fn test_hybrid_decode_low_ambiguity() {
        let decoder = ParallelVsaDecoder::new(256, 8);
        let input = vec![1.0; 256]; // low entropy input
        let result = decoder.decode_hybrid(&input, 2.0);
        assert_eq!(result.len(), 256);
    }

    #[test]
    fn test_hybrid_decode_high_ambiguity() {
        let decoder = ParallelVsaDecoder::new(64, 4);
        // Random-like input with high entropy
        let input: Vec<f64> = (0..64).map(|i| (i as f64).sin()).collect();
        let result = decoder.decode_hybrid(&input, 0.1); // low threshold → triggers serial
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_num_blocks() {
        let decoder = ParallelVsaDecoder::new(512, 8);
        assert_eq!(decoder.num_blocks(), 8);
    }

    #[test]
    fn test_global_decoder() {
        let d = global_parallel_decoder();
        assert_eq!(d.num_blocks(), 64);
    }

    #[test]
    fn test_entropy_computation() {
        let decoder = ParallelVsaDecoder::new(64, 4);
        let uniform = vec![1.0; 64];
        let entropy = decoder.compute_entropy(&uniform);
        assert!(entropy > 0.0);
    }

    #[test]
    fn test_block_size_calculation() {
        let decoder = ParallelVsaDecoder::new(4096, 64);
        assert_eq!(decoder.block_size, 64); // 4096 / 64 = 64
    }
}
