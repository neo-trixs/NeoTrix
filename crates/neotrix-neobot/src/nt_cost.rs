//! `nt_cost` — 诚实计价（调用账本 + Claude 官方价目）。
//!
//! 纪律：
//! 1. 未知模型永不猜价格（cost 0 + measured=false），费用只按明确来源算；
//! 2. 优先级：运维显式配置（`NEOBOT_PRICE_IN_PER_M` / `NEOBOT_PRICE_OUT_PER_M`）
//!    > 内置官方价目（Claude 系，2026-09，多源交叉） > 引擎自带 cost > 0 记零；
//! 3. 本地引擎（echo/CLI/自建 Ollama 口）默认 0 费 + measured=false——
//!    算力是用户自己的订阅，不进云账。
//!
//! 价目来源（2026-09-25 调研）：platform.claude.com 模型文档
//! （Sonnet 5 $2/$10、Fable 5.1 $10/$50、Mythos 5.1 $10/$50）+ 独立计算器交叉
//! （Opus 5/4.8/4.7/4.6 $5/$25、Sonnet 4.6/4.5 $3/$15、Haiku 4.5 $1/$5）。
//! 注意 Sonnet 5 于 2026-09-01 起执行标准价 $3/$15，此处取标准价。
/// 计价策略（环境变量显式配置）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CostPolicy {
    /// 美元/百万 input token（None = 未配置）。
    pub price_in_per_m: Option<f64>,
    /// 美元/百万 output token（None = 未配置）。
    pub price_out_per_m: Option<f64>,
}

impl CostPolicy {
    /// 本地默认：无配置（全 0 费、未计量）。
    pub fn local_default() -> Self {
        Self {
            price_in_per_m: None,
            price_out_per_m: None,
        }
    }

    /// `NEOBOT_PRICE_IN_PER_M` / `NEOBOT_PRICE_OUT_PER_M`（>0 才认）。
    pub fn from_env() -> Self {
        Self {
            price_in_per_m: parse_price("NEOBOT_PRICE_IN_PER_M"),
            price_out_per_m: parse_price("NEOBOT_PRICE_OUT_PER_M"),
        }
    }

    pub fn is_configured(self) -> bool {
        self.price_in_per_m.is_some() || self.price_out_per_m.is_some()
    }
}

fn parse_price(env: &str) -> Option<f64> {
    std::env::var(env)
        .ok()
        .and_then(|raw| raw.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v > 0.0)
}

/// Claude 官方价目（美元/百万 token；长名优先，子串命中）。
/// 2026-09-01 起 Sonnet 5 执行标准价，此处取标准价。
const CLAUDE_PRICES: &[(&str, f64, f64)] = &[
    ("claude-mythos-5", 10.0, 50.0),
    ("claude-fable-5", 10.0, 50.0),
    ("claude-opus-5", 5.0, 25.0),
    ("claude-opus-4-8", 5.0, 25.0),
    ("claude-opus-4-7", 5.0, 25.0),
    ("claude-opus-4-6", 5.0, 25.0),
    ("claude-opus-4-5", 5.0, 25.0),
    ("claude-sonnet-5", 3.0, 15.0),
    ("claude-sonnet-4-6", 3.0, 15.0),
    ("claude-sonnet-4-5", 3.0, 15.0),
    ("claude-haiku-4-5", 1.0, 5.0),
];

/// 查官方价目（大小写不敏感子串；无命中 → None）。
/// 备注：Anthropic 原生接口非 OpenAI 兼容，本引擎直连需兼容网关；
/// 价目只用于记账，不代表直连可用。
pub fn price_for(model: &str) -> Option<(f64, f64)> {
    let lower = model.to_ascii_lowercase();
    CLAUDE_PRICES
        .iter()
        .find(|(id, _, _)| lower.contains(id))
        .map(|(_, rate_in, rate_out)| (*rate_in, *rate_out))
}

/// 计价：返回 `(cost_usd, measured)`。
/// 运维配置 > 官方价目 > 引擎自带 > 归零；只配一边时另一边按 0 计但 measured=true
/// （运维明确说过“一边免费”，不是猜的）。
pub fn cost_for(
    policy: CostPolicy,
    model: &str,
    in_tokens: i64,
    out_tokens: i64,
) -> (f64, bool) {
    if policy.is_configured() {
        let rate_in = policy.price_in_per_m.unwrap_or(0.0);
        let rate_out = policy.price_out_per_m.unwrap_or(0.0);
        let cost = in_tokens.max(0) as f64 / 1_000_000.0 * rate_in
            + out_tokens.max(0) as f64 / 1_000_000.0 * rate_out;
        return (cost.max(0.0), true);
    }
    if let Some((rate_in, rate_out)) = price_for(model) {
        let cost = in_tokens.max(0) as f64 / 1_000_000.0 * rate_in
            + out_tokens.max(0) as f64 / 1_000_000.0 * rate_out;
        return (cost.max(0.0), true);
    }
    (0.0, false)
}

#[cfg(test)]
mod tests {
    use super::{CostPolicy, cost_for, price_for};

    #[test]
    fn unconfigured_unknown_model_never_guesses() {
        assert_eq!(cost_for(CostPolicy::local_default(), "qwen2.5", 1_000_000, 500_000), (0.0, false));
        assert_eq!(cost_for(CostPolicy::local_default(), "", 10, 5), (0.0, false));
    }

    #[test]
    fn configured_prices_measure() {
        let policy = CostPolicy {
            price_in_per_m: Some(1.0),
            price_out_per_m: Some(4.0),
        };
        assert_eq!(cost_for(policy, "anything", 1_000_000, 500_000), (3.0, true));
        // 负 token 按 0 计
        assert_eq!(cost_for(policy, "anything", -5, -5), (0.0, true));
    }

    #[test]
    fn official_claude_table_matches() {
        // 大小写不敏感 + 子串（含日期后缀的完整 id 也命中）
        assert_eq!(price_for("claude-sonnet-4-6"), Some((3.0, 15.0)));
        assert_eq!(price_for("Claude-Sonnet-4-6-20260217"), Some((3.0, 15.0)));
        assert_eq!(price_for("claude-opus-5"), Some((5.0, 25.0)));
        assert_eq!(price_for("claude-haiku-4-5"), Some((1.0, 5.0)));
        assert_eq!(price_for("claude-fable-5"), Some((10.0, 50.0)));
        assert_eq!(price_for("my-butler"), None);
        // 表驱动计价：1M in + 0.5M out 的 sonnet = 3 + 7.5
        assert_eq!(
            cost_for(CostPolicy::local_default(), "claude-sonnet-5", 1_000_000, 500_000),
            (10.5, true)
        );
        // 运维配置优先于官方表
        let policy = CostPolicy { price_in_per_m: Some(100.0), price_out_per_m: None };
        assert_eq!(cost_for(policy, "claude-sonnet-5", 1_000_000, 0), (100.0, true));
    }
}
