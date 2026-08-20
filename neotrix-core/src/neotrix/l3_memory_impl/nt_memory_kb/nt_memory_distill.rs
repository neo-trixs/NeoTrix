#![forbid(unsafe_code)]

//! DistilVDR pointwise 蒸馏学生 (吸收 arXiv:2608.10636, R-P79 生产接线)。
//!
//! 仓库无训练栈 (无 pytorch/candle/ort), 故落地为**手写 SGD 的两塔对角线性学生**:
//! - query 塔与 doc 塔各自可学习对角投影 `wq` / `wd` (逐维度缩放), score = <wq⊙q, wd⊙d> + b
//! - teacher = 现有 KB 向量余弦 (点级回归目标, 无需负采样 — DistilVDR 核心论断)
//! - 学生从 (query_vec, doc_vec, teacher_cosine) 样本点回归, MSE loss
//!
//! 接线: `hybrid_search` Tier 3 嵌入重排从裸余弦升级为蒸馏学生分数;
//! 无学生时退化回余弦 (降级安全)。学生参数落盘 `~/.neotrix/distill_student.json`。
//!
//! # 蒸馏 vs 合成公理
//! 合成公理#3 "蒸馏/衰减/混合检索三位一体": 衰减已接 (ai-memory M8, ac7f6947),
//! 混合检索已存在 (RRF 融合), 本模块补蒸馏一环 → 三位一体闭环。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// DistilVDR 双塔对角学生。两塔各为逐维度缩放 (对角线性), 参数可训练。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PointwiseDistillStudent {
    /// query 塔对角投影 (len == dim)
    pub wq: Vec<f32>,
    /// doc 塔对角投影 (len == dim)
    pub wd: Vec<f32>,
    /// 偏置
    pub bias: f32,
    /// 训练时的向量维度
    pub dim: usize,
    /// 训练样本数
    pub trained_on: usize,
}

impl PointwiseDistillStudent {
    /// 恒等初始化: wq=wd=ones, bias=0 → score 初始即 <q,d> (与裸余弦等价)。
    pub fn identity(dim: usize) -> Self {
        Self {
            wq: vec![1.0; dim],
            wd: vec![1.0; dim],
            bias: 0.0,
            dim,
            trained_on: 0,
        }
    }

    /// 蒸馏学生打分: score = <wq⊙q, wd⊙d> + bias。
    /// 输入向量长度不足 dim 的部分按 0 处理; 超过 dim 的维度忽略。
    pub fn score(&self, q: &[f32], d: &[f32]) -> f64 {
        let n = self.dim.min(q.len()).min(d.len());
        let mut s = self.bias as f64;
        for i in 0..n {
            s += (self.wq[i] * q[i]) as f64 * (self.wd[i] * d[i]) as f64;
        }
        s
    }

    /// 点级蒸馏训练 (手写 SGD + momentum, 无 autodiff 依赖)。
    ///
    /// 样本: `(query_vec, doc_vec, teacher_score)`。对每样本 MSE = (pred - teacher)²,
    /// 梯度:
    ///   ∂/∂wq[i] = 2·(pred-teacher)·wd[i]·q[i]·d[i]
    ///   ∂/∂wd[i] = 2·(pred-teacher)·wq[i]·q[i]·d[i]
    ///   ∂/∂bias  = 2·(pred-teacher)
    /// teacher 分数点回归 → 无负采样 (DistilVDR 核心: 负采样不必要)。
    pub fn train(
        samples: &[(Vec<f32>, Vec<f32>, f64)],
        dim: usize,
        epochs: usize,
        lr: f64,
        momentum: f64,
    ) -> Self {
        let mut student = Self::identity(dim);
        if samples.is_empty() {
            return student;
        }
        let mut vel_wq = vec![0.0; dim];
        let mut vel_wd = vec![0.0; dim];
        let mut vel_b = 0.0;
        for _ in 0..epochs {
            for (q, d, t) in samples {
                let pred = student.score(q, d);
                let err = 2.0 * (pred - t);
                let n = dim.min(q.len()).min(d.len());
                for i in 0..n {
                    let gq = err * student.wd[i] as f64 * q[i] as f64 * d[i] as f64;
                    let gd = err * student.wq[i] as f64 * q[i] as f64 * d[i] as f64;
                    vel_wq[i] = momentum * vel_wq[i] + lr * gq;
                    vel_wd[i] = momentum * vel_wd[i] + lr * gd;
                    student.wq[i] -= vel_wq[i] as f32;
                    student.wd[i] -= vel_wd[i] as f32;
                }
                vel_b = momentum * vel_b + lr * err;
                student.bias -= vel_b as f32;
            }
        }
        student.trained_on = samples.len();
        student
    }
}

fn student_path() -> PathBuf {
    std::env::var("NEOTRIX_DISTILL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".neotrix").join("distill_student.json")
        })
}

/// 持久化学生参数到 `~/.neotrix/distill_student.json`。
pub fn save_student(student: &PointwiseDistillStudent) -> Result<(), String> {
    let path = student_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let json = serde_json::to_string_pretty(student).map_err(|e| format!("serialize: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("write {}: {e}", path.display()))
}

/// 加载已训练学生。文件不存在 → None (检索侧退化回裸余弦)。
pub fn load_student() -> Option<PointwiseDistillStudent> {
    let path = student_path();
    let text = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&text).ok()
}

/// 从 KB 已有向量采样点级蒸馏训练样本: 随机配对 (q, d), teacher = 余弦相似度。
/// 纯点级 — 样本对来自库内真实向量, 标签即现成余弦, 无需人工负采样。
///
/// `pairs` 为抽取的 (node_a, node_b) 向量对索引; 向量不足时按可用数截断。
/// 返回 (样本, 实际使用维度)。
pub fn sample_training_pairs(
    embeddings: &[(String, Vec<f32>)],
    pairs: usize,
    seed: u64,
) -> (Vec<(Vec<f32>, Vec<f32>, f64)>, usize) {
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_embed::cosine_similarity;
    if embeddings.len() < 2 {
        return (Vec::new(), 0);
    }
    let dim = embeddings[0].1.len();
    let mut rng = {
        let mut state = seed;
        move || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (state >> 33) as usize
        }
    };
    let mut samples = Vec::with_capacity(pairs);
    for _ in 0..pairs {
        let a = rng() % embeddings.len();
        let b = rng() % embeddings.len();
        let (_, va) = &embeddings[a];
        let (_, vb) = &embeddings[b];
        let teacher = cosine_similarity(va, vb);
        samples.push((va.clone(), vb.clone(), teacher));
    }
    (samples, dim)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vec(dim: usize, seed: u64) -> Vec<f32> {
        (0..dim)
            .map(|i| {
                let x = ((seed.wrapping_mul(31).wrapping_add(i as u64 * 17) as f64).sin()) as f32;
                x
            })
            .collect()
    }

    #[test]
    fn test_identity_score_equals_raw_dot() {
        let dim = 8;
        let q = make_vec(dim, 1);
        let d = make_vec(dim, 2);
        let student = PointwiseDistillStudent::identity(dim);
        let pred = student.score(&q, &d);
        let expected: f64 = q.iter().zip(d.iter()).map(|(a, b)| (*a as f64) * (*b as f64)).sum();
        assert!((pred - expected).abs() < 1e-3, "identity student 应等于原始点积: {pred} vs {expected}");
    }

    #[test]
    fn test_train_reduces_mse() {
        let dim = 4;
        // 构造强相关样本: teacher = 真实内积比值 (人工), 学生应能学出 w 使 score 逼近 teacher
        let samples: Vec<(Vec<f32>, Vec<f32>, f64)> = (0..40)
            .map(|i| {
                let q = make_vec(dim, i as u64 + 10);
                let d = make_vec(dim, i as u64 + 100);
                let teacher = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_embed::cosine_similarity(&q, &d);
                (q, d, teacher)
            })
            .collect();
        let before = PointwiseDistillStudent::identity(dim);
        let before_mse = samples
            .iter()
            .map(|(q, d, t)| (before.score(q, d) - t).powi(2))
            .sum::<f64>()
            / samples.len() as f64;
        let after = PointwiseDistillStudent::train(&samples, dim, 50, 0.05, 0.9);
        let after_mse = samples
            .iter()
            .map(|(q, d, t)| (after.score(q, d) - t).powi(2))
            .sum::<f64>()
            / samples.len() as f64;
        assert!(after_mse < before_mse, "训练后 MSE 应下降: {after_mse} vs {before_mse}");
        assert_eq!(after.trained_on, samples.len());
    }

    #[test]
    fn test_sample_training_pairs_reuses_embeddings() {
        let embeddings: Vec<(String, Vec<f32>)> = (0..10)
            .map(|i| (format!("n{i}"), make_vec(4, i + 50)))
            .collect();
        let (samples, dim) = sample_training_pairs(&embeddings, 20, 42);
        assert_eq!(samples.len(), 20);
        assert_eq!(dim, 4);
        assert!(samples.iter().all(|(_, _, t)| (-1.0..=1.0).contains(t)));
    }

    #[test]
    fn test_save_load_roundtrip() {
        let student = PointwiseDistillStudent {
            wq: vec![0.5, 0.25],
            wd: vec![0.75, 0.125],
            bias: 0.1,
            dim: 2,
            trained_on: 7,
        };
        let path = std::env::temp_dir().join("nt_distill_test.json");
        std::env::set_var("NEOTRIX_DISTILL_PATH", &path);
        save_student(&student).expect("save");
        let loaded = load_student().expect("load");
        assert_eq!(loaded, student);
        let _ = std::fs::remove_file(&path);
        std::env::remove_var("NEOTRIX_DISTILL_PATH");
    }
}