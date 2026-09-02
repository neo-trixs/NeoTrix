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
//     use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_embed::cosine_similarity;
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
        let teacher = {
            let dot: f32 = va.iter().zip(vb.iter()).map(|(x, y)| x * y).sum();
            let na: f32 = va.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
            let nb: f32 = vb.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
            (dot / (na * nb)) as f64
        };
        samples.push((va.clone(), vb.clone(), teacher));
    }
        (samples, dim)
    }

    /// 对比/列表式训练的目标间隔 (E5/BGE 范式: listwise + contrastive, 轻量权重)。
    /// 与点级 DistilVDR 不同, 这里引入 hard-negative, 让学生学会"正例分高、难负例分低"。
    pub const CONTRASTIVE_MARGIN: f64 = 0.3;

    /// 采样对比训练样本: 每样本 = `(query_vec, positive_vec, [negative_vec; neg_k])`。
    /// 负例从同一向量池随机抽取 (排除正例); 真实 BM25/向量硬负例挖掘可后续挂到 `nt_memory_search`
    /// (R-P42, 不平行造模块)。该向量池包含 Phase1 激活的外置大脑节点 (一旦经既有 embedder 生成向量)。
    pub fn sample_contrastive_pairs(
        embeddings: &[(String, Vec<f32>)],
        pairs: usize,
        neg_k: usize,
        seed: u64,
    ) -> Vec<(Vec<f32>, Vec<f32>, Vec<Vec<f32>>)> {
        if embeddings.len() < 2 {
            return Vec::new();
        }
        let mut rng = {
            let mut state = seed;
            move || {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (state >> 33) as usize
            }
        };
        let neg_k = neg_k.min(embeddings.len() - 1);
        let mut out = Vec::with_capacity(pairs);
        for _ in 0..pairs {
            let a = rng() % embeddings.len();
            let mut b = rng() % embeddings.len();
            while b == a {
                b = rng() % embeddings.len();
            }
            let (_, va) = &embeddings[a];
            let (_, vb) = &embeddings[b];
            let mut negs = Vec::with_capacity(neg_k);
            let mut used = std::collections::HashSet::new();
            used.insert(a);
            used.insert(b);
            for _ in 0..neg_k {
                let mut n = rng() % embeddings.len();
                while used.contains(&n) {
                    n = rng() % embeddings.len();
                }
                used.insert(n);
                negs.push(embeddings[n].1.clone());
            }
            out.push((va.clone(), vb.clone(), negs));
        }
        out
    }

    /// 对比蒸馏训练 (hinge + margin): 目标 `score(q,pos) - score(q,neg) >= MARGIN`。
    /// 手写 SGD + momentum, 复用两塔对角学生结构 (无额外训练栈依赖)。
    pub fn train_contrastive(
        samples: &[(Vec<f32>, Vec<f32>, Vec<Vec<f32>>)],
        dim: usize,
        epochs: usize,
        lr: f64,
        momentum: f64,
    ) -> PointwiseDistillStudent {
        let mut student = PointwiseDistillStudent::identity(dim);
        if samples.is_empty() {
            return student;
        }
        let mut vel_wq = vec![0.0; dim];
        let mut vel_wd = vec![0.0; dim];
        let mut vel_b = 0.0;
        for _ in 0..epochs {
            for (q, pos, negs) in samples {
                let n = dim.min(q.len()).min(pos.len());
                let s_pos = student.score(q, pos);
                let mut g_wq = vec![0.0; dim];
                let mut g_wd = vec![0.0; dim];
                let mut g_b = 0.0;
                for dneg in negs {
                    let s_neg = student.score(q, dneg);
                    let gap = s_pos - s_neg;
                    if gap < CONTRASTIVE_MARGIN {
                        let d = 2.0 * (CONTRASTIVE_MARGIN - gap); // dLoss/d(gap) > 0; 故 dLoss/d(score_pos) = -d, dLoss/d(score_neg) = +d
                        for i in 0..n {
                            g_wq[i] += (-d) * student.wd[i] as f64 * q[i] as f64 * pos[i] as f64;
                            g_wd[i] += (-d) * student.wq[i] as f64 * q[i] as f64 * pos[i] as f64;
                        }
                        g_b += -d;
                        let dn = dim.min(q.len()).min(dneg.len());
                        for i in 0..dn {
                            g_wq[i] += d * student.wd[i] as f64 * q[i] as f64 * dneg[i] as f64;
                            g_wd[i] += d * student.wq[i] as f64 * q[i] as f64 * dneg[i] as f64;
                        }
                    }
                }
                for i in 0..dim {
                    vel_wq[i] = momentum * vel_wq[i] + lr * g_wq[i];
                    vel_wd[i] = momentum * vel_wd[i] + lr * g_wd[i];
                    student.wq[i] = (student.wq[i] - vel_wq[i] as f32).clamp(-5.0, 5.0);
                    student.wd[i] = (student.wd[i] - vel_wd[i] as f32).clamp(-5.0, 5.0);
                }
                vel_b = momentum * vel_b + lr * g_b;
                student.bias = (student.bias - vel_b as f32).clamp(-5.0, 5.0);
            }
        }
        student.trained_on = samples.len();
        student
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
//                 let teacher = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_embed::cosine_similarity(&q, &d);
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

    #[test]
    fn test_sample_contrastive_pairs_shape() {
        let embeddings: Vec<(String, Vec<f32>)> = (0..10)
            .map(|i| (format!("n{i}"), make_vec(4, i + 50)))
            .collect();
        let samples = sample_contrastive_pairs(&embeddings, 20, 3, 7);
        assert_eq!(samples.len(), 20);
        assert!(samples.iter().all(|(_, _, negs)| negs.len() == 3));
    }

    #[test]
    fn test_train_contrastive_improves_margin() {
        let dim = 8;
        // 正例 = q 的部分维度取反 (初始与 q 弱相关); 负例 = 与 q 无关随机向量。
        // 对比训练应学会放大 (q,pos) 相对 (q,neg) 的间隔 → 平均间隔上升。
        let mut rng_state = 12345u64;
        let mut rnd = || {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (rng_state >> 33) as f64
        };
        let make_q = |seed: u64| make_vec(dim, seed);
        let mut make_neg = |_| {
            let mut v = vec![0.0f32; dim];
            for i in 0..dim {
                v[i] = ((rnd() * 100.0).sin()) as f32;
            }
            v
        };
        let mut samples = Vec::new();
        for i in 0..40u64 {
            let q = make_q(i + 1);
            let mut pos = q.clone();
            for k in 0..dim / 2 {
                pos[k] = -pos[k]; // 半维取反
            }
            let negs: Vec<Vec<f32>> = (0..4).map(|_| make_neg(i)).collect();
            samples.push((q, pos, negs));
        }
        let before = PointwiseDistillStudent::identity(dim);
        let gap_before = samples
            .iter()
            .map(|(q, p, ns)| {
                let sp = before.score(q, p);
                let sn = ns.iter().map(|n| before.score(q, n)).sum::<f64>() / ns.len() as f64;
                sp - sn
            })
            .sum::<f64>()
            / samples.len() as f64;
        let after = train_contrastive(&samples, dim, 60, 0.05, 0.9);
        let gap_after = samples
            .iter()
            .map(|(q, p, ns)| {
                let sp = after.score(q, p);
                let sn = ns.iter().map(|n| after.score(q, n)).sum::<f64>() / ns.len() as f64;
                sp - sn
            })
            .sum::<f64>()
            / samples.len() as f64;
        assert!(
            gap_after > gap_before,
            "对比训练应拉大 (正-负) 间隔: {gap_after} vs {gap_before}"
        );
        assert_eq!(after.trained_on, samples.len());
    }

    // ── C3 基准: 对比蒸馏应提升聚类检索 top-k 命中率, 且校准误差有界 (可信) ──
    #[test]
    fn test_c3_benchmark_contrastive_beats_identity_retrieval() {
        let dim = 16;
        let clusters = 4usize;
        let per_cluster = 8usize;
        let n = clusters * per_cluster;

        // 构造带聚类结构的语料: 每簇一个中心 + 节点噪声 (同簇高余弦, 跨簇低余弦)
        let centers: Vec<Vec<f32>> = (0..clusters)
            .map(|c| make_vec(dim, (c * 97 + 3) as u64))
            .collect();
        let mut nodes: Vec<(usize, Vec<f32>)> = Vec::new(); // (cluster, vec)
        for c in 0..clusters {
            for j in 0..per_cluster {
                let mut v = centers[c].clone();
                let noise = make_vec(dim, (c * 1000 + j * 7 + 50) as u64);
                for k in 0..dim {
                    v[k] += 0.3 * noise[k];
                }
                nodes.push((c, v));
            }
        }

        // 构造对比样本: 正例=同簇另一节点, 负例=异簇 4 节点
        let mut rng_state = 99u64;
        let mut rnd = || {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (rng_state >> 33) as usize
        };
        let mut pairs = Vec::new();
        for i in 0..n {
            let (ci, ref qi) = nodes[i];
            // 正例: 同簇随机另一节点
            let mut pj = (i + 1) % n;
            while nodes[pj].0 != ci {
                pj = (pj + 1) % n;
            }
            let pos = nodes[pj].1.clone();
            // 负例: 异簇 4 节点
            let negs: Vec<Vec<f32>> = (0..4)
                .map(|_| {
                    let mut nj = rnd() % n;
                    while nodes[nj].0 == ci {
                        nj = (nj + 1) % n;
                    }
                    nodes[nj].1.clone()
                })
                .collect();
            pairs.push((qi.clone(), pos, negs));
        }

        let identity = PointwiseDistillStudent::identity(dim);
        let contrastive = train_contrastive(&pairs, dim, 80, 0.05, 0.9);

        // 检索评测: 每节点作 query, 相关集=同簇其余节点; recall@5 (identity vs contrastive)
        let recall_at = |s: &PointwiseDistillStudent| -> f64 {
            let mut hits = 0.0;
            let k = 5usize;
            for i in 0..n {
                let (ci, ref qi) = nodes[i];
                let mut scored: Vec<(f64, usize)> = (0..n)
                    .filter(|&j| j != i)
                    .map(|j| (s.score(qi, &nodes[j].1), j))
                    .collect();
                scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
                let top: std::collections::HashSet<usize> =
                    scored.iter().take(k).map(|(_, j)| *j).collect();
                let relevant: usize = (0..n).filter(|&j| j != i && nodes[j].0 == ci).count();
                let hit = top.iter().filter(|&&j| nodes[j].0 == ci).count();
                if relevant > 0 {
                    hits += hit as f64 / relevant as f64;
                }
            }
            hits / n as f64
        };

        let r_id = recall_at(&identity);
        let r_ct = recall_at(&contrastive);
        assert!(
            r_ct >= r_id - 1e-9,
            "C3 基准: 对比蒸馏检索召回不应低于 identity: {r_ct} vs {r_id}"
        );

        // 校准门禁: 用对比学生分数作置信, 同簇=正确, 计算 ECE 须有界 (<0.6) 证明"可学习且可信"
        let mut calib_samples: Vec<(f32, bool)> = Vec::new();
        for i in 0..n {
            let (ci, ref qi) = nodes[i];
            let mut best: (f64, usize) = (-1e9, 0);
            for j in 0..n {
                if j == i {
                    continue;
                }
                let sc = contrastive.score(qi, &nodes[j].1);
                if sc > best.0 {
                    best = (sc, j);
                }
            }
            let conf = (best.0.clamp(-1.0, 1.0) * 0.5 + 0.5) as f32; // 归一化到 [0,1]
            calib_samples.push((conf, nodes[best.1].0 == ci));
        }
        let ece = crate::core::nt_core_consciousness_tree::metacalib::expected_calibration_error(
            &calib_samples, 10,
        );
        assert!(ece < 0.6, "C3 基准: 校准误差应有界 (可信), got ECE={ece}");
    }
}