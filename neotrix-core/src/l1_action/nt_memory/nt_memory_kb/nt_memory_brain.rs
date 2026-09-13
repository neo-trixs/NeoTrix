//! # Brain-Inspired Memory Mechanisms
//!
//! 实现类人脑的记忆机制:
//! - Synaptic Plasticity (Hebbian 边强化)
//! - Forgetting Curve (Ebbinghaus 衰减)
//! - Associative Recall (PPR 扩散激活)
//! - Contradiction Detection (时序有效性)
//! - Memory Consolidation (STM→LTM 提升)

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use crate::core::nt_core_math::cosine_similarity_f64;

/// Synaptic Plasticity — Hebbian 边强化
///
/// "Neurons that fire together wire together"
/// 当两个节点被同时激活（被同一查询检索到）时，它们之间的边权重增强。
///
/// 实现 Kairos 验证门控：边增强仅在推理通过多维质量评估后发生，
/// 防止幻觉强化。
#[derive(Debug, Clone)]
pub struct SynapticPlasticity {
    /// 学习率 (η): 控制边权重更新速度
    pub learning_rate: f64,
    /// 衰减因子 (α): 防止权重无限增长
    pub decay_factor: f64,
    /// 质量阈值: 低于此质量的激活不触发边强化
    pub quality_threshold: f64,
    /// 最大权重: 边权重上限
    pub max_weight: f64,
}

impl Default for SynapticPlasticity {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            decay_factor: 0.99,
            quality_threshold: 0.5,
            max_weight: 10.0,
        }
    }
}

impl SynapticPlasticity {
    /// 强化共激活节点之间的边
    ///
    /// Δw_ij = η * quality * (1 - w_ij / w_max)
    ///
    /// - `activated_nodes`: 被同一查询激活的节点ID列表
    /// - `quality`: 检索质量分数 (0.0-1.0)
    /// - `conn`: SQLite连接
    pub(crate) fn _strengthen_coactivated(
        &self,
        activated_nodes: &[String],
        quality: f64,
        conn: &Connection,
    ) -> Result<usize, String> {
        if quality < self.quality_threshold {
            return Ok(0);
        }

        let mut strengthened = 0;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        for i in 0..activated_nodes.len() {
            for j in (i + 1)..activated_nodes.len() {
                let src = &activated_nodes[i];
                let tgt = &activated_nodes[j];

                // 获取当前边权重
                let current_weight: f64 = conn
                    .query_row(
                        "SELECT weight FROM edges WHERE source_id = ?1 AND target_id = ?2",
                        [src, tgt],
                        |r| r.get(0),
                    )
                    .unwrap_or(0.0);

                // Hebbian 更新: Δw = η * quality * (1 - w/w_max)
                let delta = self.learning_rate * quality * (1.0 - current_weight / self.max_weight);
                let new_weight = (current_weight + delta).min(self.max_weight);

                if new_weight > current_weight {
                    // 先检查边是否存在，再更新或插入
                    let edge_exists: bool = conn
                        .query_row(
                            "SELECT COUNT(*) > 0 FROM edges WHERE source_id = ?1 AND target_id = ?2",
                            rusqlite::params![src, tgt],
                            |r| r.get(0),
                        )
                        .unwrap_or(false);

                    if edge_exists {
                        // 更新现有边权重
                        conn.execute(
                            "UPDATE edges SET weight = ?1, updated_at = ?2 WHERE source_id = ?3 AND target_id = ?4",
                            rusqlite::params![new_weight, now, src, tgt],
                        )
                        .map_err(|e| format!("Edge update: {}", e))?;
                    } else {
                        // 插入新边
                        let edge_id = uuid::Uuid::new_v4().to_string();
                        conn.execute(
                            "INSERT INTO edges (id, source_id, target_id, relation_type, weight, created_at, updated_at)
                             VALUES (?1, ?2, ?3, 'related', ?4, ?5, ?5)",
                            rusqlite::params![edge_id, src, tgt, new_weight, now],
                        )
                        .map_err(|e| format!("Edge insert: {}", e))?;
                    }
                    strengthened += 1;
                }
            }
        }

        Ok(strengthened)
    }

    /// 全局衰减: 所有边权重乘以衰减因子
    ///
    /// w_ij(t+1) = α * w_ij(t)
    ///
    /// 防止权重无限增长，模拟遗忘曲线。
    pub(crate) fn _global_decay(&self, conn: &Connection) -> Result<usize, String> {
        let changed = conn
            .execute(
                "UPDATE edges SET weight = weight * ?1 WHERE weight > 0.01",
                rusqlite::params![self.decay_factor],
            )
            .map_err(|e| format!("Global decay: {}", e))?;
        Ok(changed)
    }
}

/// Forgetting Curve — Ebbinghaus 衰减
///
/// R(t) = e^(-t/S)
///
/// - R(t): t 时刻的记忆保持率
/// - S: 记忆稳定性 (access_count 越高，S 越大)
///
/// 结合 FSRS (Free Spaced Repetition Scheduler) 调度复习时间。
#[derive(Debug, Clone)]
pub struct ForgettingCurve {
    /// 最小稳定性: 新节点的默认稳定性
    pub min_stability: f64,
    /// 稳定性增长率: access_count 增加时 S 的增长速度
    pub stability_growth: f64,
    /// 遗忘阈值: 低于此保持率的节点被标记为应遗忘
    pub forget_threshold: f64,
    /// 复习间隔因子: FSRS 复习间隔 = S * interval_factor
    pub interval_factor: f64,
}

impl Default for ForgettingCurve {
    fn default() -> Self {
        Self {
            min_stability: 1.0,
            stability_growth: 0.5,
            forget_threshold: 0.1,
            interval_factor: 2.5,
        }
    }
}

impl ForgettingCurve {
    /// 计算记忆保持率
    ///
    /// R(t) = e^(-t/S)
    pub fn retention_rate(time_since_access: f64, stability: f64) -> f64 {
        if stability <= 0.0 {
            return 0.0;
        }
        (-time_since_access / stability).exp()
    }

    /// 计算记忆稳定性
    ///
    /// S = min_stability + stability_growth * ln(1 + access_count)
    pub fn stability(&self, access_count: i64) -> f64 {
        self.min_stability + self.stability_growth * (1.0 + access_count as f64).ln()
    }

    /// 计算下次复习时间
    ///
    /// interval = S * interval_factor
    pub(crate) fn _next_review_interval(&self, access_count: i64) -> f64 {
        self.stability(access_count) * self.interval_factor
    }

    /// 检查节点是否应遗忘
    ///
    /// 基于当前时间和最后访问时间计算保持率。
    pub fn should_forget(
        &self,
        last_access: i64,
        access_count: i64,
        now: i64,
    ) -> bool {
        let time_since = (now - last_access) as f64;
        let s = self.stability(access_count);
        let r = Self::retention_rate(time_since, s);
        r < self.forget_threshold
    }

    /// 批量更新节点的遗忘状态
    pub fn update_freshness(&self, conn: &Connection) -> Result<usize, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut stmt = conn
            .prepare(
                "SELECT id, updated_at, access_count FROM nodes WHERE access_count > 0",
            )
            .map_err(|e| format!("Prepare: {}", e))?;

        let nodes: Vec<(String, i64, i64)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|e| format!("Query: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        let mut forgotten = 0;
        for (id, last_access, access_count) in nodes {
            if self.should_forget(last_access, access_count, now) {
                conn.execute(
                    "UPDATE nodes SET metadata = json_set(COALESCE(metadata, '{}'), '$.should_forget', true) WHERE id = ?1",
                    rusqlite::params![id],
                )
                .map_err(|e| format!("Update: {}", e))?;
                forgotten += 1;
            }
        }

        Ok(forgotten)
    }
}

/// Associative Recall — PPR 扩散激活
///
/// 基于 HippoRAG 的 Personalized PageRank 算法。
/// 从查询实体种子出发，通过图结构扩散激活，找到关联节点。
#[derive(Debug, Clone)]
pub struct AssociativeRecall {
    /// 阻尼因子 (d): PPR 阻尼因子，控制随机游走概率
    pub damping: f64,
    /// 迭代次数: PPR 迭代次数
    pub iterations: usize,
    /// 收敛阈值: PPR 收敛判断阈值
    pub convergence_threshold: f64,
    /// 最大扩散深度: 限制扩散范围
    pub max_depth: usize,
}

impl Default for AssociativeRecall {
    fn default() -> Self {
        Self {
            damping: 0.85,
            iterations: 20,
            convergence_threshold: 1e-6,
            max_depth: 3,
        }
    }
}

impl AssociativeRecall {
    /// PPR 扩散激活
    ///
    /// 从种子节点出发，通过 Personalized PageRank 扩散激活。
    ///
    /// - `seed_nodes`: 种子节点ID列表
    /// - `seed_scores`: 种子节点初始分数
    /// - `conn`: SQLite连接
    /// - `top_k`: 返回最相关的K个节点
    pub fn spread_activation(
        &self,
        seed_nodes: &[String],
        seed_scores: &[f64],
        conn: &Connection,
        top_k: usize,
    ) -> Result<Vec<(String, f64)>, String> {
        if seed_nodes.is_empty() {
            return Ok(Vec::new());
        }

        // 构建邻接表
        let mut adj: HashMap<String, Vec<(String, f64)>> = HashMap::new();
        let mut stmt = conn
            .prepare("SELECT source_id, target_id, weight FROM edges")
            .map_err(|e| format!("Prepare: {}", e))?;

        let edges: Vec<(String, String, f64)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|e| format!("Query: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        for (src, tgt, weight) in edges {
            adj.entry(src.clone()).or_default().push((tgt.clone(), weight));
            adj.entry(tgt).or_default().push((src, weight));
        }

        // 初始化 PPR 分数
        let mut scores: HashMap<String, f64> = HashMap::new();
        let total_seed_score: f64 = seed_scores.iter().sum();
        for (node, score) in seed_nodes.iter().zip(seed_scores.iter()) {
            scores.insert(node.clone(), score / total_seed_score);
        }

        // PPR 迭代
        for _ in 0..self.iterations {
            let mut new_scores: HashMap<String, f64> = HashMap::new();

            for (node, &_score) in &scores {
                let neighbors = adj.get(node).map(|v| v.as_slice()).unwrap_or(&[]);
                let neighbor_sum: f64 = neighbors
                    .iter()
                    .map(|(n, w)| {
                        let n_degree = adj.get(n).map(|v| v.len()).unwrap_or(1) as f64;
                        scores.get(n).unwrap_or(&0.0) * w / n_degree
                    })
                    .sum();

                let personalization = if seed_nodes.contains(node) {
                    seed_scores[seed_nodes.iter().position(|s| s == node).unwrap()]
                        / total_seed_score
                } else {
                    0.0
                };

                let new_score =
                    (1.0 - self.damping) * personalization + self.damping * neighbor_sum;
                new_scores.insert(node.clone(), new_score);
            }

            // 检查收敛
            let diff: f64 = new_scores
                .iter()
                .map(|(k, v)| (v - scores.get(k).unwrap_or(&0.0)).abs())
                .sum();

            scores = new_scores;

            if diff < self.convergence_threshold {
                break;
            }
        }

        // 排序并返回 top_k
        let mut results: Vec<(String, f64)> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);

        Ok(results)
    }
}

/// Contradiction Detection — 时序有效性
///
/// 基于 Graphiti 的 bi-temporal 模型和 Kontrast 分类法。
/// 检测知识库中的矛盾信息。
#[derive(Debug, Clone)]
pub struct ContradictionDetector {
    /// 语义相似度阈值: 高于此阈值的节点对可能矛盾
    pub semantic_threshold: f64,
    /// 矛盾置信度阈值: 高于此阈值的矛盾被标记
    pub contradiction_threshold: f64,
}

impl Default for ContradictionDetector {
    fn default() -> Self {
        Self {
            semantic_threshold: 0.8,
            contradiction_threshold: 0.6,
        }
    }
}

/// 矛盾类型
#[derive(Debug, Clone, PartialEq)]
pub enum ContradictionType {
    /// 直接矛盾: 两个节点对同一主题有相反断言
    Direct,
    /// 时间矛盾: 同一实体在不同时间有不同状态
    Temporal,
    /// 粒度矛盾: 同一事实的不同详细程度
    Granularity,
    /// 关系矛盾: 关系方向或类型冲突
    Relational,
}

/// 矛盾检测结果
#[derive(Debug, Clone)]
pub struct ContradictionResult {
    pub node_a: String,
    pub node_b: String,
    pub contradiction_type: ContradictionType,
    pub confidence: f64,
    pub evidence: String,
}

impl ContradictionDetector {
    /// 检测两个节点是否矛盾
    pub fn detect_contradiction(
        &self,
        node_a: &str,
        node_b: &str,
        embedding_a: &[f32],
        embedding_b: &[f32],
        metadata_a: &serde_json::Value,
        metadata_b: &serde_json::Value,
    ) -> Option<ContradictionResult> {
        let sim = cosine_similarity_f64(
            &embedding_a.iter().map(|x| *x as f64).collect::<Vec<_>>(),
            &embedding_b.iter().map(|x| *x as f64).collect::<Vec<_>>(),
        );

        // 语义相似但不完全相同 → 可能矛盾
        if sim < self.semantic_threshold || sim > 0.99 {
            return None;
        }

        // 检查时序矛盾
        let temporal_a = metadata_a.get("temporal_validity");
        let temporal_b = metadata_b.get("temporal_validity");

        if let (Some(ta), Some(tb)) = (temporal_a, temporal_b) {
            let valid_from_a = ta.get("valid_from").and_then(|v| v.as_i64()).unwrap_or(0);
            let valid_until_a = ta.get("valid_until").and_then(|v| v.as_i64());
            let valid_from_b = tb.get("valid_from").and_then(|v| v.as_i64()).unwrap_or(0);
            let valid_until_b = tb.get("valid_until").and_then(|v| v.as_i64());

            // 时间窗口重叠 → 可能时间矛盾
            if valid_from_a < valid_until_b.unwrap_or(i64::MAX)
                && valid_from_b < valid_until_a.unwrap_or(i64::MAX)
            {
                return Some(ContradictionResult {
                    node_a: node_a.to_string(),
                    node_b: node_b.to_string(),
                    contradiction_type: ContradictionType::Temporal,
                    confidence: sim * 0.8,
                    evidence: format!(
                        "Overlapping temporal validity: [{}, {:?}] vs [{}, {:?}]",
                        valid_from_a, valid_until_a, valid_from_b, valid_until_b
                    ),
                });
            }
        }

        // 检查直接矛盾 (基于元数据中的断言)
        let assertion_a = metadata_a.get("assertion").and_then(|v| v.as_str());
        let assertion_b = metadata_b.get("assertion").and_then(|v| v.as_str());

        if let (Some(a), Some(b)) = (assertion_a, assertion_b) {
            if a.contains("not") != b.contains("not") && sim > self.contradiction_threshold {
                return Some(ContradictionResult {
                    node_a: node_a.to_string(),
                    node_b: node_b.to_string(),
                    contradiction_type: ContradictionType::Direct,
                    confidence: sim,
                    evidence: format!("Opposite assertions: '{}' vs '{}'", a, b),
                });
            }
        }

        None
    }

    /// 批量检测矛盾
    pub(crate) fn _scan_contradictions(
        &self,
        conn: &Connection,
        embeddings: &[(String, Vec<f32>)],
    ) -> Result<Vec<ContradictionResult>, String> {
        let mut contradictions = Vec::new();

        for i in 0..embeddings.len() {
            for j in (i + 1)..embeddings.len() {
                let (id_a, emb_a) = &embeddings[i];
                let (id_b, emb_b) = &embeddings[j];

                // 获取元数据
                let meta_a: serde_json::Value = conn
                    .query_row(
                        "SELECT COALESCE(metadata, '{}') FROM nodes WHERE id = ?1",
                        [id_a],
                        |r| r.get::<_, Option<String>>(0),
                    )
                    .ok()
                    .flatten()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or(serde_json::Value::Null);

                let meta_b: serde_json::Value = conn
                    .query_row(
                        "SELECT COALESCE(metadata, '{}') FROM nodes WHERE id = ?1",
                        [id_b],
                        |r| r.get::<_, Option<String>>(0),
                    )
                    .ok()
                    .flatten()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or(serde_json::Value::Null);

                if let Some(result) =
                    self.detect_contradiction(id_a, id_b, emb_a, emb_b, &meta_a, &meta_b)
                {
                    contradictions.push(result);
                }
            }
        }

        Ok(contradictions)
    }
}

/// Memory Consolidation — STM→LTM 提升
///
/// 基于 Anda Brain 的睡眠巩固机制。
/// 短期记忆 (STM) 经过巩固周期提升为长期记忆 (LTM)。
#[derive(Debug, Clone)]
pub struct MemoryConsolidation {
    /// STM 容量: 短期记忆最大条目数
    pub stm_capacity: usize,
    /// 巩固阈值: access_count 超过此值的 STM 条目可提升为 LTM
    pub consolidation_threshold: i64,
    /// 巩固周期: 每隔多少次迭代执行一次巩固
    pub consolidation_cycle: usize,
}

impl Default for MemoryConsolidation {
    fn default() -> Self {
        Self {
            stm_capacity: 100,
            consolidation_threshold: 3,
            consolidation_cycle: 10,
        }
    }
}

impl MemoryConsolidation {
    /// 将 STM 条目提升为 LTM
    pub fn consolidate(
        &self,
        conn: &Connection,
    ) -> Result<usize, String> {
        let mut stmt = conn
            .prepare(
                "UPDATE nodes SET importance = importance + 0.1, access_count = access_count + 1
                 WHERE access_count >= ?1 AND importance < 0.8",
            )
            .map_err(|e| format!("Prepare: {}", e))?;

        let consolidated = stmt
            .execute(rusqlite::params![self.consolidation_threshold])
            .map_err(|e| format!("Execute: {}", e))?;

        Ok(consolidated)
    }

    /// 清理过期的 STM 条目
    pub(crate) fn _prune_stm(
        &self,
        conn: &Connection,
    ) -> Result<usize, String> {
        let seven_days_ago = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
            - 7 * 86400; // 7 days in seconds

        let pruned = conn
            .execute(
                "DELETE FROM nodes WHERE importance < 0.2 AND access_count < 2
                 AND updated_at < ?1",
                rusqlite::params![seven_days_ago],
            )
            .map_err(|e| format!("Prune: {}", e))?;

        Ok(pruned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_rate() {
        let r0 = ForgettingCurve::retention_rate(0.0, 1.0);
        assert!((r0 - 1.0).abs() < 1e-6);

        let r1 = ForgettingCurve::retention_rate(1.0, 1.0);
        assert!((r1 - (-1.0f64).exp()).abs() < 1e-6);

        let r_inf = ForgettingCurve::retention_rate(1000.0, 1.0);
        assert!(r_inf < 0.01);
    }

    #[test]
    fn test_stability() {
        let fc = ForgettingCurve::default();
        let s0 = fc.stability(0);
        assert!((s0 - fc.min_stability).abs() < 1e-6);

        let s10 = fc.stability(10);
        assert!(s10 > s0);
    }

    #[test]
    fn test_should_forget() {
        let fc = ForgettingCurve::default();
        // 新节点，长时间未访问 → 应遗忘
        assert!(fc.should_forget(0, 1, 1000000));
        // 频繁访问的节点 → 不应遗忘
        assert!(!fc.should_forget(1000000, 100, 1000001));
    }

    #[test]
    fn test_contradiction_types() {
        assert_ne!(ContradictionType::Direct, ContradictionType::Temporal);
        assert_eq!(ContradictionType::Direct, ContradictionType::Direct);
    }
}
