//! DPP 多样性重排 — 吸收自 xai-org/x-algorithm (VM Ranker DPP) 与 orca 扇出合并
//!
//! 确定性点过程 (Determinantal Point Process) 的 greedy MAP 推断:
//! 在扇出多个候选结果后, 重排选择使「质量 + 多样性」联合最大化 —
//! 牺牲少量单项分换取候选集整体覆盖 (对应 x-algorithm DPP 重排)。

/// 一个候选结果: 质量分 + 特征向量 (用于多样性度量)。
#[derive(Debug, Clone)]
pub struct Candidate {
    pub id: String,
    /// 质量分 (越高越好, 由上游评分器给出)
    pub quality: f64,
    /// 归一化特征向量 (每个分量 ∈ [0,1], 用于多样性相似度)
    pub features: Vec<f64>,
}

impl Candidate {
    pub fn new(id: &str, quality: f64, features: Vec<f64>) -> Self {
        Self {
            id: id.to_string(),
            quality,
            features,
        }
    }
}

/// DPP 多样性重排器。
#[derive(Debug, Clone)]
pub struct DppSelector {
    /// 质量分权重 (相对多样性的平衡系数; 0=纯多样性, 大=纯质量)
    pub quality_weight: f64,
    /// 特征空间维数 (用于相似度核)
    dim: usize,
}

impl Default for DppSelector {
    fn default() -> Self {
        Self {
            quality_weight: 1.0,
            dim: 0,
        }
    }
}

impl DppSelector {
    /// 新建重排器。`feature_dim` 为特征空间维数 (所有候选特征长度应一致)。
    pub fn new(feature_dim: usize) -> Self {
        Self {
            quality_weight: 1.0,
            dim: feature_dim,
        }
    }

    /// 设置质量/多样性平衡权重。
    pub fn with_quality_weight(mut self, w: f64) -> Self {
        self.quality_weight = w.max(0.0);
        self
    }

    /// Greedy MAP inference over DPP:
    ///
    /// 1. 以质量分初始化每个候选的逐点增益 (quality term);
    /// 2. 每轮选取「边际增益最大」的候选加入选中集;
    /// 3. 已选集合的相似度矩阵作为惩罚项 — 已选越多、与新候选越相似, 增益越低。
    ///
    /// 返回选中候选 (保持原始输入顺序的输出, 便于下游按序消费)。
    pub fn select(&self, candidates: &[Candidate], k: usize) -> Vec<Candidate> {
        if candidates.is_empty() {
            return Vec::new();
        }
        let k = k.min(candidates.len());
        if k == 0 {
            return Vec::new();
        }

        let dim = if self.dim > 0 { self.dim } else { 1 };
        let n = candidates.len();

        // 相似度矩阵 S[i][j] = features 点积 (假设已归一化)
        let mut sim = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                sim[i][j] = dot(&candidates[i].features, &candidates[j].features, dim);
            }
        }

        // quality term: q[i] = exp(w * quality_i)
        let q: Vec<f64> = candidates
            .iter()
            .map(|c| (self.quality_weight * c.quality).exp())
            .collect();

        // greedy: 每次选最大化 log-gain 的项
        let mut selected: Vec<usize> = Vec::with_capacity(k);
        let mut available: Vec<usize> = (0..n).collect();

        while selected.len() < k && !available.is_empty() {
            let mut best_idx: Option<usize> = None;
            let mut best_gain = f64::NEG_INFINITY;

            for &i in &available {
                let mut kernel = q[i] * q[i];
                // DPP kernel: L = q * S * q^T (rank-1 quality + similarity penalty)
                for &s in &selected {
                    kernel *= (1.0 - sim[i][s]).max(0.0);
                }
                if kernel > best_gain {
                    best_gain = kernel;
                    best_idx = Some(i);
                }
            }

            if let Some(i) = best_idx {
                selected.push(i);
                available.retain(|&x| x != i);
            } else {
                break;
            }
        }

        // 按原始输入顺序输出选中候选 (保证下游消费顺序稳定)
        let mut chosen: Vec<Candidate> = selected.iter().map(|&i| candidates[i].clone()).collect();
        chosen.sort_by_key(|c| {
            candidates
                .iter()
                .position(|x| x.id == c.id)
                .unwrap_or(usize::MAX)
        });
        chosen
    }

    /// 扇出-比较-合并 (orca parallel worktree 模式):
    /// 从多个 agent 的独立结果中, 用 DPP 挑选一组高质且互不重复的候选,
    /// 用于后续合并 (如保留最优 + 多样备选, 而非仅保留一个 winner)。
    pub fn merge_winners(&self, candidates: &[Candidate], keep: usize) -> Vec<Candidate> {
        self.select(candidates, keep)
    }

    /// 输出命中率: 已选集合的多样覆盖度 (0..1, 越高越多样)。
    pub fn diversity_score(&self, candidates: &[Candidate], selected_ids: &[&str]) -> f64 {
        if candidates.is_empty() || selected_ids.is_empty() {
            return 0.0;
        }
        let dim = if self.dim > 0 { self.dim } else { 1 };
        let sel: Vec<&Candidate> = selected_ids
            .iter()
            .filter_map(|id| candidates.iter().find(|c| c.id == *id))
            .collect();
        if sel.len() < 2 {
            return 1.0;
        }
        let mut total = 0.0;
        let mut pairs = 0usize;
        for i in 0..sel.len() {
            for j in (i + 1)..sel.len() {
                total += 1.0 - dot(&sel[i].features, &sel[j].features, dim);
                pairs += 1;
            }
        }
        if pairs == 0 {
            return 1.0;
        }
        total / pairs as f64
    }
}

fn dot(a: &[f64], b: &[f64], dim: usize) -> f64 {
    let n = a.len().min(b.len()).min(dim);
    let mut acc = 0.0;
    for i in 0..n {
        acc += a[i] * b[i];
    }
    acc.min(1.0).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(id: &str, quality: f64, feat: &[f64]) -> Candidate {
        Candidate::new(id, quality, feat.to_vec())
    }

    #[test]
    fn test_empty_input() {
        let sel = DppSelector::new(2);
        assert!(sel.select(&[], 3).is_empty());
        assert_eq!(sel.merge_winners(&[], 2).len(), 0);
    }

    #[test]
    fn test_select_quality_dominant() {
        let sel = DppSelector::new(2).with_quality_weight(10.0);
        let cands = vec![
            c("a", 1.0, &[0.0, 0.0]),
            c("b", 5.0, &[0.0, 0.0]),
            c("c", 3.0, &[0.0, 0.0]),
        ];
        let chosen = sel.select(&cands, 2);
        assert_eq!(chosen.len(), 2);
        assert_eq!(chosen[0].id, "b", "highest quality first");
    }

    #[test]
    fn test_diversity_prefers_different_features() {
        let sel = DppSelector::new(2).with_quality_weight(0.0);
        // 质量全等, 多样性应优先选出特征差异大的组合
        let cands = vec![
            c("a", 1.0, &[1.0, 0.0]),
            c("b", 1.0, &[1.0, 0.0]),
            c("c", 1.0, &[0.0, 1.0]),
        ];
        let chosen = sel.select(&cands, 2);
        assert!(chosen.iter().any(|x| x.id == "c"));
    }

    #[test]
    fn test_k_cap() {
        let sel = DppSelector::new(2);
        let cands = vec![
            c("a", 1.0, &[1.0, 0.0]),
            c("b", 2.0, &[0.0, 1.0]),
            c("c", 3.0, &[1.0, 1.0]),
        ];
        let chosen = sel.select(&cands, 10);
        assert_eq!(chosen.len(), 3);
    }

    #[test]
    fn test_merge_winners_returns_ordered() {
        let sel = DppSelector::new(2);
        let cands = vec![
            c("z", 1.0, &[1.0, 0.0]),
            c("a", 2.0, &[0.0, 1.0]),
            c("m", 3.0, &[0.5, 0.5]),
        ];
        let winners = sel.merge_winners(&cands, 2);
        assert!(winners.len() <= 2);
        // 输出顺序应与原始输入顺序一致
        let pos: Vec<usize> = winners
            .iter()
            .map(|w| cands.iter().position(|c| c.id == w.id).unwrap())
            .collect();
        assert!(pos.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn test_diversity_score() {
        let sel = DppSelector::new(2);
        let cands = vec![
            c("a", 1.0, &[1.0, 0.0]),
            c("b", 1.0, &[1.0, 0.0]),
        ];
        let score = sel.diversity_score(&cands, &["a", "b"]);
        assert!(score < 0.5, "same features → low diversity, got {}", score);
    }
}
