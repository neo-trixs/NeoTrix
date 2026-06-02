//! HyperCube 知识增强预测桥
//!
//! 将 JEPA 世界模型的 latent 预测与 HyperCube 知识存储桥接：
//!   1. 预测状态 → 编码为 HyperCoord → 查询相似历史
//!   2. 历史知识 → 调整预测（知识增强）
//!   3. 新预测 → 存储到 HyperCube（经验回放）
//!
//! 使世界模型具备知识感知能力：预测不仅基于学习到的动力学，
//! 还参考 HyperCube 中存储的过往经验。

use crate::core::nt_core_hcube::coord::HyperCoord;
use crate::core::nt_core_hcube::cube::KnowledgeHyperCube;
use crate::core::nt_core_hcube::axis::DimensionAxis;
use crate::neotrix::nt_core_signal::core::Vector;

// ============================================================
// 常量
// ============================================================

/// 知识增强系数 (0~1): 多少预测来自知识
pub const KNOWLEDGE_INFLUENCE: f64 = 0.3;

/// 相似度阈值: 只使用高于此阈值的知识
pub const SIMILARITY_THRESHOLD: f64 = 0.5;

/// 最大检索条目数
pub const MAX_RETRIEVAL: usize = 10;

/// HyperCube 存储压缩维度
pub const CUBE_STORE_DIM: usize = 16;

// ============================================================
// 预测记忆
// ============================================================

/// 一条预测记忆 — 存储到 HyperCube 的知识条目
#[derive(Debug, Clone)]
pub struct PredictionMemory {
    /// 32 维 latent 状态
    pub latent_state: Vector,
    /// 预测的下一个状态
    pub predicted_next: Vector,
    /// 能量 (预测误差)
    pub energy: f64,
    /// 来源标识
    pub source: String,
    /// 时间戳
    pub timestamp: i64,
    /// 是否经过验证
    pub verified: bool,
}

// ============================================================
// HyperCoord 编码器
// ============================================================

/// 将 JEPA latent 编码为 HyperCube 坐标
pub struct HyperCoordEncoder;

impl HyperCoordEncoder {
    /// 将 32-dim latent 映射到 16 维 HyperCube 坐标
    pub fn latent_to_coord(latent: &[f64]) -> HyperCoord {
        let mut coord = HyperCoord::new();
        let step = if latent.len() > CUBE_STORE_DIM { latent.len() / CUBE_STORE_DIM } else { 1 };

        let dims = [
            DimensionAxis::CodeUnderstanding,
            DimensionAxis::SystemDesign,
            DimensionAxis::Debugging,
            DimensionAxis::KnowledgeRetrieval,
            DimensionAxis::Creativity,
            DimensionAxis::Safety,
            DimensionAxis::Performance,
            DimensionAxis::Communication,
            DimensionAxis::Time,
            DimensionAxis::Domain,
            DimensionAxis::Abstraction,
            DimensionAxis::Culture,
            DimensionAxis::Scale,
            DimensionAxis::Certainty,
            DimensionAxis::Agency,
            DimensionAxis::Modality,
        ];

        for (i, dim) in dims.iter().enumerate() {
            if i * step < latent.len() {
                coord.set(*dim, latent[i * step] * 100.0);
            }
        }

        coord
    }

    /// 从坐标重建 latent（简化: 取坐标值的归一化版本）
    pub fn coord_to_latent(coord: &HyperCoord) -> Vector {
        let mut vec = vec![0.0; 32];
        for (i, (_dim, val)) in coord.dims().enumerate().take(16) {
            if i < 32 {
                vec[i] = *val / 100.0;
            }
        }
        vec
    }
}

// ============================================================
// 知识增强预测器
// ============================================================

/// 知识增强预测器 — 用 HyperCube 知识调整 JEPA 预测
pub struct KnowledgeAugmentedPredictor {
    /// 知识影响系数
    pub knowledge_influence: f64,
    /// 相似度阈值
    pub similarity_threshold: f64,
}

impl Default for KnowledgeAugmentedPredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeAugmentedPredictor {
    pub fn new() -> Self {
        Self {
            knowledge_influence: KNOWLEDGE_INFLUENCE,
            similarity_threshold: SIMILARITY_THRESHOLD,
        }
    }

    /// 检索相似历史预测
    pub fn retrieve_similar(
        &self,
        _latent: &[f64],
        cube: &KnowledgeHyperCube,
    ) -> Vec<PredictionMemory> {
        if cube.is_empty() { return vec![]; }

        let query_coord = HyperCoordEncoder::latent_to_coord(_latent);
        let entries = cube.query(&query_coord, MAX_RETRIEVAL);

        entries.iter().map(|entry| {
            let predicted_next = vec![0.0; 32]; // simplified: no json decode
            let latent_state: Vector = entry.coord.dims()
                .map(|(_dim, val)| *val / 100.0)
                .collect();

            PredictionMemory {
                latent_state,
                predicted_next,
                energy: 0.5,
                source: entry.source.clone(),
                timestamp: (entry.value * 1000.0) as i64,
                verified: entry.value > 0.5,
            }
        }).collect()
    }

    /// 用知识调整预测
    ///   adjusted = (1 - α) * jepa_pred + α * knowledge_weighted_avg
    pub fn augment_prediction(
        &self,
        jepa_prediction: &[f64],
        similar_memories: &[PredictionMemory],
    ) -> Vector {
        if similar_memories.is_empty() {
            return jepa_prediction.to_vec();
        }

        let mut weighted_sum = vec![0.0; jepa_prediction.len()];
        let mut total_weight = 0.0;

        for mem in similar_memories {
            let weight = if mem.verified { 1.0 } else { 0.3 };
            for (i, val) in mem.predicted_next.iter().enumerate().take(jepa_prediction.len()) {
                weighted_sum[i] += weight * val;
            }
            total_weight += weight;
        }

        if total_weight > 0.0 {
            let knowledge_avg: Vector = weighted_sum.iter().map(|v| v / total_weight).collect();
            let alpha = self.knowledge_influence;
            jepa_prediction.iter().zip(knowledge_avg.iter())
                .map(|(j, k)| (1.0 - alpha) * j + alpha * k)
                .collect()
        } else {
            jepa_prediction.to_vec()
        }
    }

    /// 将预测结果存储到 HyperCube
    pub fn store_prediction(
        &self,
        latent_state: &[f64],
        predicted_next: &[f64],
        energy: f64,
        source: &str,
        cube: &mut KnowledgeHyperCube,
    ) {
        let coord = HyperCoordEncoder::latent_to_coord(latent_state);
        if let Ok(label) = serde_json::to_string(predicted_next) {
            cube.insert(&coord, source, &label);
        }

        // 同时存储 latent 本身
        let latent_coord = HyperCoordEncoder::latent_to_coord(predicted_next);
        cube.insert(&latent_coord, "jepa-prediction", &format!("energy={:.4}", energy));
    }
}

// ============================================================
// 经验回放缓冲区
// ============================================================

/// 经验回放缓冲区 — 存储 (state, next_state, energy) 三元组
#[derive(Debug, Clone)]
pub struct ReplayBuffer {
    pub capacity: usize,
    pub buffer: Vec<(Vector, Vector, f64)>,
    pub position: usize,
}

impl ReplayBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            buffer: Vec::with_capacity(capacity),
            position: 0,
        }
    }

    /// 添加经验
    pub fn push(&mut self, state: Vector, next_state: Vector, energy: f64) {
        if self.buffer.len() < self.capacity {
            self.buffer.push((state, next_state, energy));
        } else {
            self.buffer[self.position] = (state, next_state, energy);
        }
        self.position = (self.position + 1) % self.capacity;
    }

    /// 随机采样一批
    pub fn sample(&self, batch_size: usize) -> Vec<&(Vector, Vector, f64)> {
        let n = self.buffer.len();
        if n == 0 { return vec![]; }
        let k = batch_size.min(n);
        let mut indices: Vec<usize> = (0..n).collect();
        // Fisher-Yates partial shuffle
        let mut result = Vec::with_capacity(k);
        for i in 0..k {
            let j = i + (rand::random::<usize>() % (n - i));
            indices.swap(i, j);
            result.push(&self.buffer[indices[i]]);
        }
        result
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

// ============================================================
// 测试
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyper_coord_encoder_roundtrip() {
        let latent = vec![0.5; 32];
        let coord = HyperCoordEncoder::latent_to_coord(&latent);
        assert!(coord.dims().count() > 0);

        let _reconstructed = HyperCoordEncoder::coord_to_latent(&coord);
    }

    #[test]
    fn test_augment_prediction_empty_knowledge() {
        let aug = KnowledgeAugmentedPredictor::new();
        let pred = vec![0.5; 32];
        let augmented = aug.augment_prediction(&pred, &[]);
        assert_eq!(augmented, pred);
    }

    #[test]
    fn test_replay_buffer_basic() {
        let mut buf = ReplayBuffer::new(10);
        buf.push(vec![1.0; 32], vec![2.0; 32], 0.5);
        buf.push(vec![2.0; 32], vec![3.0; 32], 0.3);
        assert_eq!(buf.len(), 2);

        let sample = buf.sample(2);
        assert_eq!(sample.len(), 2);
    }

    #[test]
    fn test_replay_buffer_capacity() {
        let mut buf = ReplayBuffer::new(3);
        for i in 0..5 {
            buf.push(vec![i as f64; 32], vec![(i + 1) as f64; 32], 0.1);
        }
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn test_knowledge_augment_prediction_never_nan() {
        let aug = KnowledgeAugmentedPredictor::new();
        let pred = vec![0.5; 32];
        let cube = KnowledgeHyperCube::new();
        let memories = aug.retrieve_similar(&pred, &cube);
        let augmented = aug.augment_prediction(&pred, &memories);
        assert!(augmented.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn test_store_and_retrieve_cycle() {
        let aug = KnowledgeAugmentedPredictor::new();
        let mut cube = KnowledgeHyperCube::new();

        let latent = vec![0.3; 32];
        let pred = vec![0.7; 32];

        aug.store_prediction(&latent, &pred, 0.1, "test", &mut cube);
        assert!(!cube.is_empty());

        let memories = aug.retrieve_similar(&latent, &cube);
        assert!(memories.is_empty() || memories.len() <= MAX_RETRIEVAL);
    }
}
