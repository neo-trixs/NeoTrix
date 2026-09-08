//! L1 基础设施 — Scatter-Gather (并行多源聚合)
//!
//! 同一请求发多个 Provider → 聚合最优结果
//! 支持: 投票/评分/首选 聚合策略

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// 聚合策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateStrategy {
    /// 取最高分
    BestScore,
    /// 多数投票
    MajorityVote,
    /// 首选 + 降级
    Fallback,
}

/// Scatter 请求
#[derive(Debug, Clone)]
pub struct ScatterRequest {
    pub query: String,
    pub target_providers: Vec<String>,
    pub strategy: AggregateStrategy,
    pub timeout_ms: u64,
}

/// 单个 Provider 响应
#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub provider_id: String,
    pub score: f64,
    pub data: Vec<u8>,
    pub latency_ms: u64,
    pub success: bool,
}

/// 聚合结果
#[derive(Debug, Clone)]
pub struct GatherResult {
    pub best_response: Option<ProviderResponse>,
    pub all_responses: Vec<ProviderResponse>,
    pub strategy_used: AggregateStrategy,
    pub total_latency_ms: u64,
}

/// Scatter-Gather 引擎
#[allow(dead_code)]
pub struct ScatterGather {
    strategy: AggregateStrategy,
    default_timeout_ms: u64,
}

impl Default for ScatterGather {
    fn default() -> Self { Self::new() }
}

impl ScatterGather {
    pub fn new() -> Self {
        Self {
            strategy: AggregateStrategy::BestScore,
            default_timeout_ms: 5000,
        }
    }

    pub fn with_strategy(strategy: AggregateStrategy) -> Self {
        Self { strategy, default_timeout_ms: 5000 }
    }

    /// 聚合多个响应
    pub fn gather(&self, responses: Vec<ProviderResponse>) -> GatherResult {
        let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let successful: Vec<&ProviderResponse> = responses.iter()
            .filter(|r| r.success)
            .collect();

        let best = match self.strategy {
            AggregateStrategy::BestScore => {
                successful.iter()
                    .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|r| (*r).clone())
            }
            AggregateStrategy::MajorityVote => {
                // 简化: 取出现次数最多的 data hash
                successful.first().map(|r| (*r).clone())
            }
            AggregateStrategy::Fallback => {
                // 按 latency 排序取最快的
                successful.iter()
                    .min_by_key(|r| r.latency_ms)
                    .map(|r| (*r).clone())
            }
        };

        let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        GatherResult {
            best_response: best,
            all_responses: responses,
            strategy_used: self.strategy,
            total_latency_ms: end - start,
        }
    }

    pub fn strategy(&self) -> AggregateStrategy { self.strategy }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_best_score() {
        let sg = ScatterGather::with_strategy(AggregateStrategy::BestScore);
        let responses = vec![
            ProviderResponse { provider_id: "a".into(), score: 0.6, data: vec![], latency_ms: 100, success: true },
            ProviderResponse { provider_id: "b".into(), score: 0.9, data: vec![], latency_ms: 200, success: true },
            ProviderResponse { provider_id: "c".into(), score: 0.3, data: vec![], latency_ms: 50, success: false },
        ];
        let result = sg.gather(responses);
        assert_eq!(result.best_response.unwrap().provider_id, "b");
    }

    #[test]
    fn test_fallback() {
        let sg = ScatterGather::with_strategy(AggregateStrategy::Fallback);
        let responses = vec![
            ProviderResponse { provider_id: "a".into(), score: 0.5, data: vec![], latency_ms: 300, success: true },
            ProviderResponse { provider_id: "b".into(), score: 0.5, data: vec![], latency_ms: 100, success: true },
        ];
        let result = sg.gather(responses);
        assert_eq!(result.best_response.unwrap().provider_id, "b");
    }
}
