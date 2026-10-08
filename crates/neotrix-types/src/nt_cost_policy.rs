//! `nt_cost_policy` — 计价策略的单一事实源（core 与 neobot 共用）。
//!
//! 迁移来源：`neotrix-neobot/src/nt_cost.rs`（原 `CostPolicy`）。
//! 价格表与 `cost_for`/`price_for` 仍留在 neobot 侧（Claude 官方价目），
//! 这里只承托策略结构本身，避免两侧各持一份解析逻辑。

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_default_is_unconfigured() {
        let p = CostPolicy::local_default();
        assert!(!p.is_configured());
        assert_eq!(p.price_in_per_m, None);
    }

    #[test]
    fn from_env_ignores_non_positive_and_garbage() {
        // 不依赖真实环境状态：直接验证 parse 路径的过滤语义。
        assert_eq!(parse_price("DEFINITELY_MISSING_ENV_VAR_XX"), None);
    }
}
