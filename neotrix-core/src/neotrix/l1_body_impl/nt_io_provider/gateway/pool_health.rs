//! LLM 池健康探测器 (NT-REPAIR 自愈节点 / SelfTest T1)。
//!
//! 每次请求入口经 `GatewayV2::ensure_pool_sufficient` (T3 生产接线) 驱动; 本模块产出
//! 结构化 `PoolHealthReport`, 量化自有 LLM 池的存活度 (总/免费/付费/锁定/充足),
//! 使自愈行为可观测、可断言, 而非黑盒补充。对齐 OmniRoute Radar 健康度 + NeoTrix 自愈公理。

use serde::Serialize;

use super::GatewayV2;

/// LLM 池健康快照 — 由 `LlmPoolHealth::evaluate` 产出, 供日志/断言/自愈决策消费。
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct PoolHealthReport {
    /// 已注册 provider 总数
    pub total: usize,
    /// 免费 (keyless) provider 数
    pub free: usize,
    /// 付费 provider 数
    pub paid: usize,
    /// 本地 (主体) provider 数
    pub local: usize,
    /// 当前被 L3 模型级熔断锁定的数量
    pub model_locked: usize,
    /// 是否满足 `min_free` 充足阈值
    pub sufficient: bool,
    /// 充足阈值 (免费 provider 下限)
    pub min_free: usize,
}

/// LLM 池健康探测器 — T1 存在 (impl) + T3 生产接线 (由 `ensure_pool_sufficient` 调用其 `evaluate`)。
pub struct LlmPoolHealth;

impl LlmPoolHealth {
    /// 对 `gw` 当前池子做健康评估, 返回结构化报告。纯读, 无副作用。
    pub fn evaluate(gw: &GatewayV2, min_free: usize) -> PoolHealthReport {
        let providers = gw.providers();
        let total = providers.len();
        let free = gw.available_free_providers().len();
        let model_locked = gw.model_locked_count();
        let sufficient = gw.is_pool_sufficient(min_free);
        let local = providers
            .iter()
            .filter(|n| n.contains("ollama") || n.contains("vllm") || n.contains("llama"))
            .count();
        PoolHealthReport {
            total,
            free,
            paid: total.saturating_sub(free),
            local,
            model_locked,
            sufficient,
            min_free,
        }
    }

    /// 健康度文字摘要 (供日志)。
    pub fn summarize(gw: &GatewayV2, min_free: usize) -> String {
        let r = Self::evaluate(gw, min_free);
        format!(
            "LLM pool health: total={} free={} paid={} local={} model_locked={} sufficient={} (min_free={})",
            r.total, r.free, r.paid, r.local, r.model_locked, r.sufficient, r.min_free
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_health_evaluate_empty_pool() {
        let gw = GatewayV2::new();
        let r = LlmPoolHealth::evaluate(&gw, 3);
        assert_eq!(r.total, 0);
        assert_eq!(r.free, 0);
        assert!(!r.sufficient, "空池不应判定为充足");
        assert_eq!(r.min_free, 3);
    }

    #[test]
    fn test_pool_health_summary_format() {
        let gw = GatewayV2::new();
        let s = LlmPoolHealth::summarize(&gw, 3);
        assert!(s.contains("LLM pool health"), "摘要应含前缀");
        assert!(s.contains("total=0"), "空池 total=0");
    }
}
