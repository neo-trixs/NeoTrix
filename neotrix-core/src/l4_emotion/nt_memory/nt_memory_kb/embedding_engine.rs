#![forbid(unsafe_code)]

//! 嵌入引擎 — 提取 kb-embed-pq.py / kb-embed-server.py 本质模式
//!
//! 核心抽象:
//! - `PqCodebook`: Product Quantization 码本训练 + 编码 (kmeans per subspace)
//! - `EmbeddingPipeline`: 文本 → 向量 → 可选 PQ 量化, 统一入口
//! - `LocalEmbeddingServer`: OpenAI 兼容本地嵌入服务 (MiniLM)
//!
//! 设计: 纯算法层, 不持有 DB 连接; 持久化由调用方通过 `PqCodebook::persist` / `PqCodebook::load` 完成。

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Product Quantization — kb-embed-pq.py 本质
// ═══════════════════════════════════════════════════════════════════

/// PQ 码本: M 个子空间 × K 个质心, 用于 ANN 加速检索。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqCodebook {
    /// 子空间数 (必须整除向量维度)
    pub m: usize,
    /// 每子空间质心数
    pub k: usize,
    /// 子空间维度 = dimension / m
    pub sub_dim: usize,
    /// 码本数据: M 个 [K × sub_dim] 质心, 扁平化存储
    pub codewords: Vec<f32>,
    /// 原始向量维度
    pub dimension: usize,
    /// 训练所用模型名
    pub model: String,
    /// 训练时向量数
    pub num_vectors: usize,
    /// 训练时间戳 (unix seconds)
    pub trained_at: i64,
}

impl PqCodebook {
    /// 最小训练向量数: K × 4 (确保每个质心至少有少量样本)
    pub const MIN_TRAIN_VECTORS_MULTIPLIER: usize = 4;

    /// 创建新码本并训练
    ///
    /// `vectors` 为 n × dimension 的扁平化向量 (行优先)。
    pub fn train(
        vectors: &[f32],
        n: usize,
        dimension: usize,
        m: usize,
        k: usize,
        model: &str,
    ) -> Result<Self, String> {
        if m == 0 || dimension % m != 0 {
            return Err(format!("m={} must be a positive divisor of dimension={}", m, dimension));
        }
        let min_vectors = k * Self::MIN_TRAIN_VECTORS_MULTIPLIER;
        if n < min_vectors {
            return Err(format!(
                "Not enough vectors to train PQ: {} < {}. Need more embeddings first.",
                n, min_vectors
            ));
        }

        let sub_dim = dimension / m;
        let mut codewords_all = Vec::with_capacity(m * k * sub_dim);

        // 逐子空间 kmeans 训练
        for s in 0..m {
            // 提取子空间数据: vectors[:, s*sub_dim .. (s+1)*sub_dim]
            let mut sub = vec![0.0f32; n * sub_dim];
            for i in 0..n {
                let src_start = i * dimension + s * sub_dim;
                let dst_start = i * sub_dim;
                sub[dst_start..dst_start + sub_dim]
                    .copy_from_slice(&vectors[src_start..src_start + sub_dim]);
            }

            let centroids = kmeans(&sub, n, sub_dim, k, 10, 42)?;
            codewords_all.extend_from_slice(&centroids);
        }

        Ok(Self {
            m,
            k,
            sub_dim,
            codewords: codewords_all,
            dimension,
            model: model.to_string(),
            num_vectors: n,
            trained_at: now_secs(),
        })
    }

    /// 对单个向量编码为 PQ codes (M 个 u8)
    pub fn encode(&self, vector: &[f32]) -> Vec<u8> {
        assert_eq!(vector.len(), self.dimension, "vector dimension mismatch");
        let mut codes = Vec::with_capacity(self.m);
        for s in 0..self.m {
            let sub_start = s * self.sub_dim;
            let cw_start = s * self.k * self.sub_dim;
            let mut best_dist = f32::MAX;
            let mut best_code = 0u8;
            for j in 0..self.k {
                let mut dist = 0.0f32;
                for d in 0..self.sub_dim {
                    let diff = vector[sub_start + d] - self.codewords[cw_start + j * self.sub_dim + d];
                    dist += diff * diff;
                }
                if dist < best_dist {
                    best_dist = dist;
                    best_code = j as u8;
                }
            }
            codes.push(best_code);
        }
        codes
    }

    /// 批量编码 (返回 n × M 的 codes)
    pub fn encode_batch(&self, vectors: &[f32], n: usize) -> Vec<Vec<u8>> {
        (0..n)
            .map(|i| {
                let start = i * self.dimension;
                self.encode(&vectors[start..start + self.dimension])
            })
            .collect()
    }

    /// PQ codes → 可序列化 blob (M bytes packed as u8 array)
    pub fn codes_to_blob(codes: &[u8]) -> Vec<u8> {
        codes.to_vec()
    }

    /// blob → PQ codes
    pub fn blob_to_codes(blob: &[u8]) -> Vec<u8> {
        blob.to_vec()
    }

    /// 序列化码本为 blob (用于存入 SQLite)
    pub fn codewords_blob(&self) -> Vec<u8> {
        // f32 → little-endian bytes
        let mut blob = Vec::with_capacity(self.codewords.len() * 4);
        for &v in &self.codewords {
            blob.extend_from_slice(&v.to_le_bytes());
        }
        blob
    }

    /// 从 blob 反序列化码本
    pub fn from_codewords_blob(
        blob: &[u8],
        m: usize,
        k: usize,
        sub_dim: usize,
        dimension: usize,
        model: &str,
        num_vectors: usize,
        trained_at: i64,
    ) -> Result<Self, String> {
        if blob.len() != m * k * sub_dim * 4 {
            return Err(format!(
                "Codewords blob size mismatch: expected {} bytes, got {}",
                m * k * sub_dim * 4,
                blob.len()
            ));
        }
        let mut codewords = Vec::with_capacity(m * k * sub_dim);
        for chunk in blob.chunks_exact(4) {
            codewords.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        Ok(Self {
            m,
            k,
            sub_dim,
            codewords,
            dimension,
            model: model.to_string(),
            num_vectors,
            trained_at,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════
// K-Means — kb-embed-pq.py 核心算法
// ═══════════════════════════════════════════════════════════════════

/// 简易 kmeans: data (n × dim), 返回 k 个质心 (k × dim)
fn kmeans(
    data: &[f32],
    n: usize,
    dim: usize,
    k: usize,
    iters: usize,
    seed: u64,
) -> Result<Vec<f32>, String> {
    if n == 0 || dim == 0 || k == 0 {
        return Err("kmeans: empty input".to_string());
    }

    // 初始化: 随机选 k 个样本 (deterministic via seed)
    let mut centroids = Vec::with_capacity(k * dim);
    let step = n / k;
    for j in 0..k {
        let idx = ((j * step + seed as usize) % n).min(n - 1);
        centroids.extend_from_slice(&data[idx * dim..(idx + 1) * dim]);
    }

    let mut labels = vec![0u32; n];

    for _ in 0..iters {
        // Assignment step
        for i in 0..n {
            let mut best_dist = f32::MAX;
            let mut best_j = 0usize;
            for j in 0..k {
                let mut dist = 0.0f32;
                for d in 0..dim {
                    let diff = data[i * dim + d] - centroids[j * dim + d];
                    dist += diff * diff;
                }
                if dist < best_dist {
                    best_dist = dist;
                    best_j = j;
                }
            }
            labels[i] = best_j as u32;
        }

        // Update step
        let mut counts = vec![0u32; k];
        let mut sums = vec![0.0f32; k * dim];
        for i in 0..n {
            let j = labels[i] as usize;
            counts[j] += 1;
            for d in 0..dim {
                sums[j * dim + d] += data[i * dim + d];
            }
        }
        for j in 0..k {
            if counts[j] > 0 {
                for d in 0..dim {
                    centroids[j * dim + d] = sums[j * dim + d] / counts[j] as f32;
                }
            }
        }
    }

    Ok(centroids)
}

// ═══════════════════════════════════════════════════════════════════
// 嵌入管线 — 统一入口
// ═══════════════════════════════════════════════════════════════════

/// 嵌入管线配置
#[derive(Debug, Clone)]
pub struct EmbeddingPipelineConfig {
    /// PQ 子空间数 (0 = 不使用 PQ)
    pub pq_m: usize,
    /// PQ 每子空间质心数
    pub pq_k: usize,
    /// 向量维度
    pub dimension: usize,
    /// 模型名
    pub model: String,
}

impl Default for EmbeddingPipelineConfig {
    fn default() -> Self {
        Self {
            pq_m: 24,
            pq_k: 256,
            dimension: 384,
            model: "all-MiniLM-L6-v2".to_string(),
        }
    }
}

/// 嵌入管线: 文本 → 向量 + 可选 PQ 量化
///
/// 不持有 DB 连接; 调用方负责持久化。
pub struct EmbeddingPipeline {
    config: EmbeddingPipelineConfig,
}

impl EmbeddingPipeline {
    pub fn new(config: EmbeddingPipelineConfig) -> Self {
        Self { config }
    }

    /// 对已有向量进行 PQ 量化 (输入 n × dimension 的扁平向量)
    pub fn quantize(
        &self,
        vectors: &[f32],
        n: usize,
    ) -> Result<PqQuantizationResult, String> {
        let codebook = PqCodebook::train(
            vectors,
            n,
            self.config.dimension,
            self.config.pq_m,
            self.config.pq_k,
            &self.config.model,
        )?;
        let codes = codebook.encode_batch(vectors, n);
        Ok(PqQuantizationResult { codebook, codes })
    }

    /// 计算两个 PQ 编码之间的 L2 距离估计 (用码本质心)
    pub fn pq_distance_estimated(codebook: &PqCodebook, codes_a: &[u8], codes_b: &[u8]) -> f32 {
        assert_eq!(codes_a.len(), codebook.m);
        assert_eq!(codes_b.len(), codebook.m);
        let mut dist = 0.0f32;
        for s in 0..codebook.m {
            let cw_start = s * codebook.k * codebook.sub_dim;
            let a = codes_a[s] as usize;
            let b = codes_b[s] as usize;
            for d in 0..codebook.sub_dim {
                let diff = codebook.codewords[cw_start + a * codebook.sub_dim + d]
                    - codebook.codewords[cw_start + b * codebook.sub_dim + d];
                dist += diff * diff;
            }
        }
        dist
    }
}

/// PQ 量化结果
pub struct PqQuantizationResult {
    pub codebook: PqCodebook,
    /// n 个 PQ codes (每个 M bytes)
    pub codes: Vec<Vec<u8>>,
}

// ═══════════════════════════════════════════════════════════════════
// 辅助
// ═══════════════════════════════════════════════════════════════════

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pq_roundtrip() {
        let n = 128;
        let dim = 24; // divisible by m=4
        let m = 4;
        let k = 8;
        // 生成伪随机数据
        let vectors: Vec<f32> = (0..n * dim).map(|i| (i as f32) / (n * dim) as f32).collect();
        let codebook = PqCodebook::train(&vectors, n, dim, m, k, "test").unwrap();
        assert_eq!(codebook.m, m);
        assert_eq!(codebook.k, k);
        assert_eq!(codebook.codewords.len(), m * k * (dim / m));

        // 编码 + blob 往返
        let codes = codebook.encode(&vectors[0..dim]);
        assert_eq!(codes.len(), m);
        for &c in &codes {
            assert!((c as usize) < k);
        }

        let blob = PqCodebook::codewords_blob(&codebook);
        let loaded =
            PqCodebook::from_codewords_blob(&blob, m, k, dim / m, dim, "test", n, 0).unwrap();
        assert_eq!(loaded.codewords, codebook.codewords);
    }

    #[test]
    fn test_kmeans_basic() {
        let data = vec![
            0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, // cluster A
            10.0, 10.0, 11.0, 11.0, 10.0, 10.0, 11.0, 11.0, // cluster B
        ];
        let centroids = kmeans(&data, 4, 2, 2, 20, 42).unwrap();
        assert_eq!(centroids.len(), 4); // 2 centroids × 2 dim
    }

    #[test]
    fn test_pq_distance_estimated() {
        let n = 64;
        let dim = 12;
        let m = 3;
        let k = 4;
        let vectors: Vec<f32> = (0..n * dim).map(|i| (i as f32 * 0.1).sin()).collect();
        let codebook = PqCodebook::train(&vectors, n, dim, m, k, "test").unwrap();
        let codes_a = codebook.encode(&vectors[0..dim]);
        let codes_b = codebook.encode(&vectors[dim..2 * dim]);
        let dist = EmbeddingPipeline::pq_distance_estimated(&codebook, &codes_a, &codes_b);
        assert!(dist >= 0.0);
    }
}
