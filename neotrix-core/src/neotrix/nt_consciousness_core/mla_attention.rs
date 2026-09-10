#![forbid(unsafe_code)]

//! 多头潜在注意力 (Multi-head Latent Attention, MLA)
//!
//! 将多个注意力头投影到潜在空间，通过压缩减少 KV 缓存大小，
//! 同时保持注意力质量，支持长上下文推理
//!
//! 参考: DeepSeek-V2 MLA — 低秩压缩 KV 缓存

use serde::{Deserialize, Serialize};

/// 注意力头维度
pub const DEFAULT_HEAD_DIM: usize = 128;

/// 潜在空间维度
pub const DEFAULT_LATENT_DIM: usize = 64;

/// MLA 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlaConfig {
    /// 注意力头数
    pub num_heads: usize,
    /// 每头维度
    pub head_dim: usize,
    /// 潜在空间维度 (压缩后)
    pub latent_dim: usize,
    /// 压缩比
    pub compression_ratio: f64,
    /// 是否启用旋转位置编码
    pub use_rope: bool,
    /// 上下文长度
    pub max_context_len: usize,
}

impl Default for MlaConfig {
    fn default() -> Self {
        Self {
            num_heads: 32,
            head_dim: DEFAULT_HEAD_DIM,
            latent_dim: DEFAULT_LATENT_DIM,
            compression_ratio: 0.5,
            use_rope: true,
            max_context_len: 128_000,
        }
    }
}

/// KV 压缩器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvCompressor {
    /// 下投影矩阵维度: (head_dim, latent_dim)
    pub down_proj_dim: (usize, usize),
    /// 上投影矩阵维度: (latent_dim, head_dim)
    pub up_proj_dim: (usize, usize),
    /// 压缩权重
    pub weights_down: Vec<Vec<f64>>,
    /// 解压权重
    pub weights_up: Vec<Vec<f64>>,
}

impl KvCompressor {
    /// 创建新的 KV 压缩器
    pub fn new(head_dim: usize, latent_dim: usize) -> Self {
        Self {
            down_proj_dim: (head_dim, latent_dim),
            up_proj_dim: (latent_dim, head_dim),
            weights_down: Self::init_weights(head_dim, latent_dim),
            weights_up: Self::init_weights(latent_dim, head_dim),
        }
    }

    /// 初始化权重 (Xavier 初始化)
    fn init_weights(rows: usize, cols: usize) -> Vec<Vec<f64>> {
        let scale = (2.0 / (rows + cols) as f64).sqrt();
        let mut weights = Vec::with_capacity(rows);
        for i in 0..rows {
            let mut row = Vec::with_capacity(cols);
            for j in 0..cols {
                // 简单的确定性伪随机
                let val = ((i * cols + j) as f64 * 0.618033988749895) % 1.0;
                row.push((val - 0.5) * 2.0 * scale);
            }
            weights.push(row);
        }
        weights
    }

    /// 压缩 KV: (head_dim,) → (latent_dim,)
    pub fn compress(&self, kv: &[f64]) -> Vec<f64> {
        let mut result = vec![0.0; self.down_proj_dim.1];
        for j in 0..self.down_proj_dim.1 {
            let mut sum = 0.0;
            for i in 0..self.down_proj_dim.0.min(kv.len()) {
                sum += kv[i] * self.weights_down[i][j];
            }
            result[j] = sum;
        }
        result
    }

    /// 解压 KV: (latent_dim,) → (head_dim,)
    pub fn decompress(&self, latent: &[f64]) -> Vec<f64> {
        let mut result = vec![0.0; self.up_proj_dim.1];
        for j in 0..self.up_proj_dim.1 {
            let mut sum = 0.0;
            for i in 0..self.up_proj_dim.0.min(latent.len()) {
                sum += latent[i] * self.weights_up[i][j];
            }
            result[j] = sum;
        }
        result
    }
}

/// 单头注意力状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadState {
    /// 头 ID
    pub head_id: usize,
    /// 压缩后的 KV 缓存
    pub compressed_kv: Vec<Vec<f64>>,
    /// 原始 KV 缓存大小
    pub original_size: usize,
    /// 压缩后大小
    pub compressed_size: usize,
    /// 序列位置
    pub positions: Vec<usize>,
}

/// MLA 注意力状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlaAttentionState {
    /// 各头状态
    pub heads: Vec<HeadState>,
    /// 当前序列长度
    pub seq_len: usize,
    /// 总压缩大小
    pub total_compressed_size: usize,
    /// 总原始大小
    pub total_original_size: usize,
}

/// 多头潜在注意力
pub struct MlaAttention {
    /// 配置
    config: MlaConfig,
    /// KV 压缩器
    compressor: KvCompressor,
    /// 注意力状态
    state: MlaAttentionState,
    /// 统计
    stats: MlaStats,
}

/// MLA 统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MlaStats {
    /// 总压缩次数
    pub total_compressions: u64,
    /// 总解压次数
    pub total_decompressions: u64,
    /// 平均压缩比
    pub avg_compression_ratio: f64,
    /// 缓存节省字节
    pub cache_bytes_saved: u64,
    /// 注意力计算次数
    pub attention_computations: u64,
}

impl MlaAttention {
    /// 创建新的 MLA 注意力
    pub fn new(config: MlaConfig) -> Self {
        let compressor = KvCompressor::new(config.head_dim, config.latent_dim);
        let heads = (0..config.num_heads)
            .map(|i| HeadState {
                head_id: i,
                compressed_kv: Vec::new(),
                original_size: 0,
                compressed_size: 0,
                positions: Vec::new(),
            })
            .collect();

        Self {
            compressor,
            state: MlaAttentionState {
                heads,
                seq_len: 0,
                total_compressed_size: 0,
                total_original_size: 0,
            },
            config,
            stats: MlaStats::default(),
        }
    }

    /// 添加 KV 到缓存 (压缩存储)
    pub fn append_kv(&mut self, key: Vec<f64>, value: Vec<f64>, position: usize) {
        // 压缩 K
        let compressed_k = self.compressor.compress(&key);
        // 压缩 V
        let compressed_v = self.compressor.compress(&value);

        let original_size = key.len() + value.len();
        let compressed_size = compressed_k.len() + compressed_v.len();

        for head in &mut self.state.heads {
            head.compressed_kv.push(compressed_k.clone());
            head.compressed_kv.push(compressed_v.clone());
            head.original_size += original_size;
            head.compressed_size += compressed_size;
            head.positions.push(position);
        }

        self.state.seq_len += 1;
        self.state.total_original_size += original_size * self.config.num_heads;
        self.state.total_compressed_size += compressed_size * self.config.num_heads;
        self.stats.total_compressions += 2 * self.config.num_heads as u64;
        self.stats.cache_bytes_saved += ((original_size - compressed_size) * self.config.num_heads) as u64;

        if self.state.total_original_size > 0 {
            self.stats.avg_compression_ratio = self.state.total_compressed_size as f64
                / self.state.total_original_size as f64;
        }
    }

    /// 计算注意力分数
    pub fn compute_attention(
        &mut self,
        query: &[f64],
        head_id: usize,
    ) -> Option<Vec<f64>> {
        if head_id >= self.state.heads.len() {
            return None;
        }

        let head = &self.state.heads[head_id];
        if head.compressed_kv.is_empty() {
            return None;
        }

        // 解压 KV 用于注意力计算
        let mut scores = Vec::new();
        let mut i = 0;
        while i + 1 < head.compressed_kv.len() {
            let compressed_k = &head.compressed_kv[i];
            let _compressed_v = &head.compressed_kv[i + 1];

            let k = self.compressor.decompress(compressed_k);

            // 计算 Q·K^T
            let mut score = 0.0;
            for j in 0..query.len().min(k.len()) {
                score += query[j] * k[j];
            }

            // 缩放
            score /= (self.config.head_dim as f64).sqrt();
            scores.push(score);

            i += 2;
        }

        // Softmax
        let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_scores: Vec<f64> = scores.iter().map(|s| (s - max_score).exp()).collect();
        let sum_exp: f64 = exp_scores.iter().sum();
        let attention_weights: Vec<f64> = exp_scores.iter().map(|s| s / sum_exp).collect();

        self.stats.attention_computations += 1;
        self.stats.total_decompressions += head.compressed_kv.len() as u64;

        Some(attention_weights)
    }

    /// 获取压缩比
    pub fn compression_ratio(&self) -> f64 {
        if self.state.total_original_size == 0 {
            return 1.0;
        }
        self.state.total_compressed_size as f64 / self.state.total_original_size as f64
    }

    /// 获取状态
    pub fn state(&self) -> &MlaAttentionState {
        &self.state
    }

    /// 获取统计
    pub fn stats(&self) -> &MlaStats {
        &self.stats
    }

    /// 获取配置
    pub fn config(&self) -> &MlaConfig {
        &self.config
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        for head in &mut self.state.heads {
            head.compressed_kv.clear();
            head.original_size = 0;
            head.compressed_size = 0;
            head.positions.clear();
        }
        self.state.seq_len = 0;
        self.state.total_compressed_size = 0;
        self.state.total_original_size = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mla_creation() {
        let mla = MlaAttention::new(MlaConfig::default());
        assert_eq!(mla.state().seq_len, 0);
        assert_eq!(mla.state().heads.len(), 32);
    }

    #[test]
    fn test_kv_compression() {
        let compressor = KvCompressor::new(128, 64);
        let kv: Vec<f64> = (0..128).map(|i| i as f64).collect();
        
        let compressed = compressor.compress(&kv);
        assert_eq!(compressed.len(), 64);
        
        let decompressed = compressor.decompress(&compressed);
        assert_eq!(decompressed.len(), 128);
    }

    #[test]
    fn test_append_kv() {
        let mut mla = MlaAttention::new(MlaConfig::default());
        let key: Vec<f64> = (0..128).map(|i| i as f64).collect();
        let value: Vec<f64> = (0..128).map(|i| i as f64 * 2.0).collect();
        
        mla.append_kv(key, value, 0);
        assert_eq!(mla.state().seq_len, 1);
        assert!(mla.compression_ratio() < 1.0);
    }

    #[test]
    fn test_attention_computation() {
        let mut mla = MlaAttention::new(MlaConfig::default());
        let key: Vec<f64> = (0..128).map(|i| i as f64).collect();
        let value: Vec<f64> = (0..128).map(|i| i as f64 * 2.0).collect();
        
        mla.append_kv(key, value, 0);
        
        let query: Vec<f64> = (0..128).map(|i| i as f64 * 0.5).collect();
        let result = mla.compute_attention(&query, 0);
        assert!(result.is_some());
        
        let weights = result.unwrap();
        assert_eq!(weights.len(), 1);
        // Softmax 输出应和为 1
        let sum: f64 = weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_compression_ratio() {
        let mut mla = MlaAttention::new(MlaConfig::default());
        
        // 添加多个 KV
        for i in 0..10 {
            let key: Vec<f64> = (0..128).map(|j| (j + i) as f64).collect();
            let value: Vec<f64> = (0..128).map(|j| (j + i) as f64 * 2.0).collect();
            mla.append_kv(key, value, i);
        }
        
        // 压缩比应小于 1 (节省空间)
        assert!(mla.compression_ratio() < 1.0);
        assert!(mla.stats().cache_bytes_saved > 0);
    }
}
