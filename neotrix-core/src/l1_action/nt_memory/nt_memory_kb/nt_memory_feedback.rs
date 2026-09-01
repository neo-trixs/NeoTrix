//! SEAL 反馈回流闭环 — 检索权重在线学习 (B4 瓶颈修复)
//!
//! 替换硬编码 `fuse_weights [0.25,0.15,0.40,0.20]`:
//! 1. 记录 (query_family, strategy) 的采纳/弃用信号
//! 2. 聚合采纳率 → 在线调整融合权重 (指数滑动平均)
//! 3. 权重钳制 [0.05, 0.7] 防震荡
//!
//! 设计来源: KB-AGENTIC-RAG-EVOLUTION.md §3.4

use std::sync::RwLock;

/// 反馈信号 — 单次检索-生成的采纳记录
#[derive(Debug, Clone, PartialEq)]
pub struct FeedbackSignal {
    /// 意图家族 (如 "relation-multi-hop" / "factual" / "semantic")
    pub query_family: String,
    /// 使用的检索策略 (通道名)
    pub strategy: String,
    /// 生成阶段采纳的节点 id
    pub adopted_ids: Vec<String>,
    /// 分级为 Irrelevant 弃用的节点 id
    pub rejected_ids: Vec<String>,
    /// 检索延迟 (ms)
    pub latency_ms: u64,
}

/// 聚合后的通道统计
#[derive(Debug, Clone, PartialEq)]
pub struct StrategyStats {
    pub strategy: String,
    pub adopted_count: u64,
    pub rejected_count: u64,
    pub total_count: u64,
    pub adoption_rate: f64,
}

/// 反馈存储 — 内存聚合 + SQLite 持久化
pub struct FeedbackStore {
    /// 当前融合权重 [fts, bm25, embed, graph]
    weights: RwLock<[f64; 4]>,
    /// 内存聚合表 (strategy → counters), 定期落盘
    aggregates: RwLock<std::collections::HashMap<String, (u64, u64, u64)>>,
    /// 学习率 (指数滑动平均)
    alpha: f64,
}

impl Default for FeedbackStore {
    fn default() -> Self {
        Self::new(0.05)
    }
}

impl FeedbackStore {
    pub fn new(alpha: f64) -> Self {
        Self {
            weights: RwLock::new([0.25, 0.15, 0.40, 0.20]),
            aggregates: RwLock::new(std::collections::HashMap::new()),
            alpha,
        }
    }

    /// 记录一次反馈信号 (strategy → 采纳/弃用计数)
    pub fn record(&self, signal: &FeedbackSignal) {
        let mut agg = match self.aggregates.write() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        let entry = agg
            .entry(signal.strategy.clone())
            .or_insert((0, 0, 0));
        entry.0 += signal.adopted_ids.len() as u64;
        entry.1 += signal.rejected_ids.len() as u64;
        entry.2 += 1;
    }

    /// 当前融合权重 (只读)
    pub fn weights(&self) -> [f64; 4] {
        self.weights.read().map(|w| *w).unwrap_or([0.25, 0.15, 0.40, 0.20])
    }

    /// 按策略名取权重 (Fast/Vector/Graph/AgentLoop/Decompose → 映射到 4 信号)
    pub fn weight_for_strategy(&self, strategy: &str) -> f64 {
        let w = self.weights();
        match strategy {
            "fast" | "bm25" => w[1],
            "vector" | "semantic" => w[2],
            "graph" => w[3],
            _ => w[0],
        }
    }

    /// 在线调整权重 — 基于各策略采纳率
    ///
    /// 规则: 采纳率 > 0.5 → 该策略对应权重上调 (受 α 控制); < 0.5 → 下调。
    /// 钳制到 [0.05, 0.7] 防单通道垄断或归零。
    pub fn update_weights(&self) -> [f64; 4] {
        let agg = match self.aggregates.read() {
            Ok(g) => g.clone(),
            Err(e) => e.into_inner().clone(),
        };
        if agg.is_empty() {
            return self.weights();
        }

        let mut weights = self.weights();
        for (strategy, (adopted, rejected, _total)) in &agg {
            let total = adopted + rejected;
            if total == 0 {
                continue;
            }
            let rate = *adopted as f64 / total as f64;
            // 信号映射到权重槽
            let idx: Option<usize> = match strategy.as_str() {
                "fast" | "bm25" => Some(1),
                "vector" | "semantic" => Some(2),
                "graph" => Some(3),
                "agent_loop" | "decompose" => Some(0),
                _ => None,
            };
            if let Some(i) = idx {
                let delta = self.alpha * (rate - 0.5) * 2.0; // [-α, +α]
                weights[i] = (weights[i] + delta).clamp(0.05, 0.7);
            }
        }

        if let Ok(mut w) = self.weights.write() {
            *w = weights;
        }
        weights
    }

    /// 重置聚合统计 (用于周期结算后)
    pub fn reset_aggregates(&self) {
        if let Ok(mut agg) = self.aggregates.write() {
            agg.clear();
        }
    }

    /// 导出各策略统计 (审计/监控)
    pub fn strategy_stats(&self) -> Vec<StrategyStats> {
        let agg = match self.aggregates.read() {
            Ok(g) => g.clone(),
            Err(e) => e.into_inner().clone(),
        };
        let mut out: Vec<StrategyStats> = agg
            .into_iter()
            .map(|(strategy, (adopted, rejected, total))| {
                let total_events = adopted + rejected;
                let rate = if total_events == 0 {
                    0.0
                } else {
                    adopted as f64 / total_events as f64
                };
                StrategyStats {
                    strategy,
                    adopted_count: adopted,
                    rejected_count: rejected,
                    total_count: total,
                    adoption_rate: rate,
                }
            })
            .collect();
        out.sort_by(|a, b| b.adoption_rate.partial_cmp(&a.adoption_rate).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    /// 持久化权重到 SQLite kv_store (经 unify 命名空间)
    pub fn persist(&self, conn: &rusqlite::Connection) -> Result<(), String> {
        let w = self.weights();
        let json = serde_json::to_string(&w).map_err(|e| format!("serde: {}", e))?;
        super::nt_memory_unify::kv_set(conn, "feedback", "weights", &json)
    }

    /// 从 SQLite 恢复权重
    pub fn load(&self, conn: &rusqlite::Connection) -> Result<(), String> {
        let json = super::nt_memory_unify::kv_get(conn, "feedback", "weights")?;
        if let Some(data) = json {
            if let Ok(w) = serde_json::from_str::<[f64; 4]>(&data) {
                if let Ok(mut weights) = self.weights.write() {
                    *weights = w;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signal(family: &str, strategy: &str, adopted: usize, rejected: usize) -> FeedbackSignal {
        FeedbackSignal {
            query_family: family.to_string(),
            strategy: strategy.to_string(),
            adopted_ids: (0..adopted).map(|i| format!("adopt-{}", i)).collect(),
            rejected_ids: (0..rejected).map(|i| format!("reject-{}", i)).collect(),
            latency_ms: 42,
        }
    }

    #[test]
    fn test_default_weights_match_baseline() {
        let store = FeedbackStore::default();
        assert_eq!(store.weights(), [0.25, 0.15, 0.40, 0.20]);
    }

    #[test]
    fn test_record_counts() {
        let store = FeedbackStore::new(0.05);
        store.record(&signal("factual", "fast", 5, 1));
        store.record(&signal("factual", "fast", 4, 2));
        let stats = store.strategy_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].strategy, "fast");
        assert_eq!(stats[0].adopted_count, 9);
        assert_eq!(stats[0].rejected_count, 3);
        assert!((stats[0].adoption_rate - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_high_adoption_raises_weight() {
        let store = FeedbackStore::new(0.10);
        let before = store.weights();
        store.record(&signal("factual", "vector", 10, 0)); // 100% 采纳
        let after = store.update_weights();
        assert!(after[2] > before[2], "高采纳率应抬升 vector 权重: {} -> {}", before[2], after[2]);
        // 钳制上限
        for _ in 0..50 {
            store.record(&signal("factual", "vector", 10, 0));
            store.update_weights();
        }
        let clamped = store.weights();
        assert!(clamped[2] <= 0.7 + 1e-9, "权重应钳制上限 0.7, 实际 {}", clamped[2]);
    }

    #[test]
    fn test_low_adoption_lowers_weight() {
        let store = FeedbackStore::new(0.10);
        store.record(&signal("factual", "graph", 0, 10)); // 0% 采纳
        let after = store.update_weights();
        assert!(after[3] < 0.20, "低采纳率应压低 graph 权重: {}", after[3]);
        assert!(after[3] >= 0.05 - 1e-9, "权重应钳制下限 0.05: {}", after[3]);
    }

    #[test]
    fn test_empty_store_no_change() {
        let store = FeedbackStore::new(0.10);
        let before = store.weights();
        let after = store.update_weights();
        assert_eq!(before, after, "无反馈时权重不应变化");
    }

    #[test]
    fn test_strategy_mapping() {
        let store = FeedbackStore::default();
        assert_eq!(store.weight_for_strategy("vector"), 0.40);
        assert_eq!(store.weight_for_strategy("graph"), 0.20);
        assert_eq!(store.weight_for_strategy("fast"), 0.15);
        assert_eq!(store.weight_for_strategy("unknown"), 0.25);
    }

    #[test]
    fn test_persist_roundtrip() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        super::super::nt_memory_schema::initialize(&conn).unwrap();
        let store = FeedbackStore::new(0.05);
        store.persist(&conn).unwrap();
        store.load(&conn).unwrap();
        assert_eq!(store.weights(), [0.25, 0.15, 0.40, 0.20]);
    }
}
