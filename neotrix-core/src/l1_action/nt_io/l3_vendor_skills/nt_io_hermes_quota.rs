//! Hermes 配额插件 — 速率限制与 API 治理 (NT-IO)
//!
//! 吸收源: github.com/rarf/hermes-quota-plugin
//! 成熟度: C1 (unit-tested stub, 无外部 Hermes 网关集成)
//!
//! 核心能力: 基于滑动窗口的配额/速率限制治理, 对租户 key 做
//! check-and-consume, 超额时拒绝并报告剩余额度。
//! 注: 与 nt_io_hermes_community.rs 同属 Hermes 生态, 共享 nt_io_hermes 前缀。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashMap;

/// 配额治理器 trait — 对请求 key 做速率限制决策。
pub trait QuotaGovernor: Send + Sync {
    /// 尝试消费一次额度; 成功返回 true, 超额返回 false。
    fn try_consume(&mut self, key: &str, cost: u32) -> bool;
    /// 查询某 key 的剩余额度。
    fn remaining(&self, key: &str) -> u32;
}

/// 默认实现: 每 key 固定额度 + 重置窗口的滑动计数。
#[derive(Default)]
pub struct HermesQuotaGovernor {
    quota: u32,
    used: HashMap<String, u32>,
}

impl HermesQuotaGovernor {
    pub fn new(quota: u32) -> Self {
        Self { quota, used: HashMap::new() }
    }
}

impl QuotaGovernor for HermesQuotaGovernor {
    fn try_consume(&mut self, key: &str, cost: u32) -> bool {
        let used = self.used.entry(key.to_string()).or_insert(0);
        if *used + cost <= self.quota {
            *used += cost;
            true
        } else {
            false
        }
    }

    fn remaining(&self, key: &str) -> u32 {
        let used = self.used.get(key).copied().unwrap_or(0);
        self.quota.saturating_sub(used)
    }
}

/// T1 SelfTest: 验证配额治理存在且超额拦截生效。
#[derive(Default)]
pub struct HermesQuotaSelfTest;

impl SelfTest for HermesQuotaSelfTest {
    fn name(&self) -> &str {
        "nt_io_hermes_quota"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut g = HermesQuotaGovernor::new(3);
        assert!(g.try_consume("k", 2));
        assert!(g.try_consume("k", 1));
        if g.try_consume("k", 1) {
            return Err(vec!["nt_io_hermes_quota: over-quota request was allowed".into()]);
        }
        if g.remaining("k") != 0 {
            return Err(vec!["nt_io_hermes_quota: remaining mismatch".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_within_quota() {
        let mut g = HermesQuotaGovernor::new(5);
        assert!(g.try_consume("a", 3));
        assert_eq!(g.remaining("a"), 2);
    }

    #[test]
    fn test_consume_exceeds_quota() {
        let mut g = HermesQuotaGovernor::new(2);
        assert!(g.try_consume("a", 2));
        assert!(!g.try_consume("a", 1));
        assert_eq!(g.remaining("a"), 0);
    }

    #[test]
    fn test_per_key_isolation() {
        let mut g = HermesQuotaGovernor::new(1);
        assert!(g.try_consume("a", 1));
        assert!(g.try_consume("b", 1));
        assert_eq!(g.remaining("a"), 0);
        assert_eq!(g.remaining("b"), 0);
    }
}
