//! 模型成本分桶与会话成本聚合 — dscode 吸收 B-5
//!
//! 吸收来源 (notes/absorption-dscode-1.md 条目 8 + 13 + Q5, 2026-08-13):
//! - model.ts:59-64    per-model cost 结构 `{input, output, cacheRead, cacheWrite}` USD/M tokens
//!                     (deepseek: input 0.14 / output 0.28 / cacheRead 0.0028 / cacheWrite 0)
//! - status.ts:35-57   会话条目遍历累加 input/output/cacheRead/cacheWrite, cost=Σ usage.cost.total
//! - status.ts:51-54   缓存命中率 = cacheRead/(input+cacheRead+cacheWrite)*100 (最新 assistant 条目)
//! - status.ts:76-93   展示: cache 命中率 · X read · Y write · 未缓存 input · output · cost $N.NNN
//!
//! Q5 成本/缓存计费建模 (absorption-dscode-1.md:419-424): 显式分桶计价,
//! 缓存读与输入分开计费是成本优化的核心信号。
//!
//! 接线点 (BLOCKED — 并行会话已改 nt_io_provider/mod.rs / types.rs / gateway.rs):
//! 生产接线应将本类型并入 `types.rs::Usage` (补 cache_read/cache_write 桶) 并在
//! `gateway.rs` usage 统计点聚合; 当前以独立模块交付。临时接线在 rate_limiter.rs。
#![allow(dead_code)] // 临时接线: 模块已编译但未进生产 usage 统计路径 (合并 types.rs/gateway.rs 后可移除)

use serde::{Deserialize, Serialize};

/// 模型成本结构 — model.ts:59-64, 单位 USD / 1M tokens。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ModelCost {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
}

/// dscode deepseek 成本表 (model.ts:59-64): 缓存读 0.0028 vs 输入 0.14, 显式分桶。
pub const DEEPSEEK_COST: ModelCost = ModelCost {
    input: 0.14,
    output: 0.28,
    cache_read: 0.0028,
    cache_write: 0.0,
};

impl ModelCost {
    pub const fn new(input: f64, output: f64, cache_read: f64, cache_write: f64) -> Self {
        Self {
            input,
            output,
            cache_read,
            cache_write,
        }
    }

    /// 按分桶估算单次用量成本 (USD)。
    pub fn estimate(&self, usage: &UsageBuckets) -> f64 {
        usage.input as f64 * self.input / 1_000_000.0
            + usage.output as f64 * self.output / 1_000_000.0
            + usage.cache_read as f64 * self.cache_read / 1_000_000.0
            + usage.cache_write as f64 * self.cache_write / 1_000_000.0
    }
}

/// 用量分桶 (tokens) — status.ts:35-57 累加 input/output/cacheRead/cacheWrite。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageBuckets {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

impl UsageBuckets {
    pub const fn new(input: u64, output: u64, cache_read: u64, cache_write: u64) -> Self {
        Self {
            input,
            output,
            cache_read,
            cache_write,
        }
    }

    /// prompt 侧总 token = input + cacheRead + cacheWrite (status.ts:52-54 分母)。
    pub fn prompt_tokens(&self) -> u64 {
        self.input + self.cache_read + self.cache_write
    }

    pub fn total_tokens(&self) -> u64 {
        self.prompt_tokens() + self.output
    }

    /// 缓存命中率 (%) = cacheRead/(input+cacheRead+cacheWrite)*100 (status.ts:51-54)。
    ///
    /// prompt 侧为 0 时返回 0.0, 避免除零。
    pub fn cache_hit_rate_percent(&self) -> f64 {
        let prompt = self.prompt_tokens();
        if prompt == 0 {
            0.0
        } else {
            self.cache_read as f64 / prompt as f64 * 100.0
        }
    }

    /// 饱和加法合并 (会话跨条目累加不溢出)。
    pub fn merge(&mut self, other: &UsageBuckets) {
        self.input = self.input.saturating_add(other.input);
        self.output = self.output.saturating_add(other.output);
        self.cache_read = self.cache_read.saturating_add(other.cache_read);
        self.cache_write = self.cache_write.saturating_add(other.cache_write);
    }
}

/// 会话条目 — 单条用量 + provider 上报成本 (status.ts:36-41, cost=Σ usage.cost.total)。
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct SessionEntry {
    pub usage: UsageBuckets,
    /// 单条 provider 上报成本 (USD); 0 表示未上报, 聚合时回退到 ModelCost 估算。
    pub cost_usd: f64,
}

/// 会话成本聚合结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionCostSummary {
    pub usage: UsageBuckets,
    pub cost_usd: f64,
    pub cache_hit_rate_percent: f64,
}

/// 会话用量聚合 (status.ts:35-40): Σ input/output/cacheRead/cacheWrite。
pub fn aggregate_session_usage(entries: &[SessionEntry]) -> UsageBuckets {
    let mut acc = UsageBuckets::default();
    for entry in entries {
        acc.merge(&entry.usage);
    }
    acc
}

/// 会话成本聚合 (任务 B-5 规格)。
///
/// - Σ input/output/cacheRead/cacheWrite (status.ts:35-40)
/// - cost = Σ usage.cost.total (status.ts:41); 全部未上报时回退 ModelCost::estimate (model.ts:59-64)
/// - 缓存命中率 = cacheRead/(input+cacheRead+cacheWrite)*100 取全会话聚合
pub fn aggregate_session_cost(entries: &[SessionEntry], model_cost: Option<&ModelCost>) -> SessionCostSummary {
    let usage = aggregate_session_usage(entries);
    let reported: f64 = entries.iter().map(|e| e.cost_usd).sum();
    let cost_usd = if reported > 0.0 {
        reported
    } else {
        model_cost.map(|m| m.estimate(&usage)).unwrap_or(0.0)
    };
    SessionCostSummary {
        usage,
        cost_usd,
        cache_hit_rate_percent: usage.cache_hit_rate_percent(),
    }
}

/// 最新条目缓存命中率 — status.ts:51-54 (取最后一个 prompt 侧非空的条目)。
///
/// dscode 展示用"最新 assistant 条目"命中率, 与全会话聚合区分。
pub fn latest_cache_hit_rate_percent(entries: &[SessionEntry]) -> f64 {
    for entry in entries.iter().rev() {
        if entry.usage.prompt_tokens() > 0 {
            return entry.usage.cache_hit_rate_percent();
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_buckets_deepseek_rate_estimate() {
        // model.ts:59-64 deepseek 定价: 1M input=0.14, 1M output=0.28, 1M cacheRead=0.0028
        let usage = UsageBuckets::new(1_000_000, 1_000_000, 1_000_000, 0);
        let cost = DEEPSEEK_COST.estimate(&usage);
        assert!((cost - 0.4228).abs() < 1e-9, "got {cost}");
    }

    #[test]
    fn cost_buckets_cache_hit_rate_formula() {
        // status.ts:51-54: 100/(100+400+0) = 20%
        let usage = UsageBuckets::new(400, 1000, 100, 0);
        assert_eq!(usage.cache_hit_rate_percent(), 20.0);
    }

    #[test]
    fn cost_buckets_cache_hit_rate_zero_prompt_is_zero() {
        let usage = UsageBuckets::default();
        assert_eq!(usage.cache_hit_rate_percent(), 0.0);
    }

    #[test]
    fn cost_buckets_aggregate_sums_buckets() {
        let entries = [
            SessionEntry {
                usage: UsageBuckets::new(100, 50, 20, 10),
                cost_usd: 0.0,
            },
            SessionEntry {
                usage: UsageBuckets::new(200, 150, 80, 5),
                cost_usd: 0.0,
            },
        ];
        let usage = aggregate_session_usage(&entries);
        assert_eq!(usage, UsageBuckets::new(300, 200, 100, 15));
    }

    #[test]
    fn cost_buckets_aggregate_cost_falls_back_to_model_cost() {
        let entries = [
            SessionEntry {
                usage: UsageBuckets::new(1_000_000, 0, 0, 0),
                cost_usd: 0.0,
            },
            SessionEntry {
                usage: UsageBuckets::new(0, 500_000, 0, 0),
                cost_usd: 0.0,
            },
        ];
        let summary = aggregate_session_cost(&entries, Some(&DEEPSEEK_COST));
        // 1M input(0.14) + 0.5M output(0.14) = 0.28
        assert!((summary.cost_usd - 0.28).abs() < 1e-9, "got {}", summary.cost_usd);
    }

    #[test]
    fn cost_buckets_aggregate_prefers_reported_cost() {
        let entries = [
            SessionEntry {
                usage: UsageBuckets::new(1_000_000, 0, 0, 0),
                cost_usd: 0.5,
            },
            SessionEntry {
                usage: UsageBuckets::new(0, 1_000_000, 0, 0),
                cost_usd: 0.25,
            },
        ];
        let summary = aggregate_session_cost(&entries, Some(&DEEPSEEK_COST));
        assert!((summary.cost_usd - 0.75).abs() < 1e-9, "got {}", summary.cost_usd);
        assert_eq!(summary.cache_hit_rate_percent, 0.0);
    }

    #[test]
    fn cost_buckets_latest_entry_hit_rate() {
        let entries = [
            SessionEntry {
                usage: UsageBuckets::new(100, 10, 0, 0),
                cost_usd: 0.0,
            },
            SessionEntry {
                usage: UsageBuckets::new(80, 10, 80, 0),
                cost_usd: 0.0,
            },
            SessionEntry {
                usage: UsageBuckets::new(0, 5, 0, 0),
                cost_usd: 0.0,
            },
        ];
        // 最新 prompt 非空条目: cacheRead 80/(80+80)=50%
        assert_eq!(latest_cache_hit_rate_percent(&entries), 50.0);
    }
}
