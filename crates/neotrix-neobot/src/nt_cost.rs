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

// ── 用量账本（按天按模型；口径取自 OpenGhost `usage.js`，MIT） ──
//
// 计价回答「多少钱」，账本回答「花在哪」：哪天、哪个 provider/model、
// 发了多少、命中多少缓存、回了多少、几次请求。两者是同一枚硬币的两面，
// 故同住 `nt_cost` 而不另开模块。
//
// 与上游同口径处：五列（input/cached/written/output/requests）、
// cached 钳制 ≤ input、全零不记、按本地日历天切分、`since` 为首条 epoch 毫秒。
// 偏离处（有意）：
//   ① provider 不限死 4 家 —— `[a-z0-9_-]{1,32}` 即认（本仓 provider 会长）；
//   ② 落盘是显式路径的 JSON 文件（原子 rename），不是 localStorage ——
//      库不拼路径（`data_dir` 是 app 壳的事），调用方传 path；
//   ③ 坏文件 Err 不回默认 —— 静默重置等于銷账，而调用方以为「还没花过」。

/// 单日单模型行。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DayUsage {
    pub input: u64,
    pub cached: u64,
    pub written: u64,
    pub output: u64,
    pub requests: u64,
}

/// 用量账本。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UsageLedger {
    pub version: u32,
    /// 首条记录的 epoch 毫秒（0 = 空账本）。
    pub since: i64,
    /// 天（本地 `YYYY-MM-DD`）→ `provider|model` → 行。
    pub days: std::collections::BTreeMap<String, std::collections::BTreeMap<String, DayUsage>>,
    /// `provider|model` → 展示名。
    pub names: std::collections::BTreeMap<String, String>,
}

/// 汇总口径。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UsageTotals {
    pub input: u64,
    pub cached: u64,
    pub written: u64,
    pub output: u64,
    pub requests: u64,
}

impl UsageTotals {
    /// 计费 token（input + output；cached/written 是 input 的子集，不另计）。
    pub fn tokens(self) -> u64 {
        self.input.saturating_add(self.output)
    }
}

fn valid_provider(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 32
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

fn today_key() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

impl UsageLedger {
    /// 记一轮问答的用量。全零不记（上游同）；cached 钳制 ≤ input。
    pub fn record(
        &mut self,
        provider: &str,
        model: &str,
        name: Option<&str>,
        input: u64,
        cached: u64,
        written: u64,
        output: u64,
    ) -> Result<(), String> {
        if !valid_provider(provider) {
            return Err(format!("bad usage provider '{provider}'"));
        }
        if model.trim().is_empty() {
            return Err("usage model is empty".to_owned());
        }
        if input == 0 && output == 0 {
            return Ok(());
        }
        if self.since == 0 {
            self.since = chrono::Utc::now().timestamp_millis();
            self.version = 1;
        }
        let id = format!("{provider}|{model}");
        let row = self.days.entry(today_key()).or_default().entry(id.clone()).or_default();
        row.input = row.input.saturating_add(input);
        row.cached = row.cached.saturating_add(cached.min(input));
        row.written = row.written.saturating_add(written);
        row.output = row.output.saturating_add(output);
        row.requests = row.requests.saturating_add(1);
        if let Some(name) = name {
            if !name.trim().is_empty() {
                self.names.insert(id, name.trim().to_owned());
            }
        }
        Ok(())
    }

    /// 最近 `days` 个本地日历天（含今天）的汇总；`days == 0` = 有史以来。
    pub fn totals(&self, days: u64) -> UsageTotals {
        let cutoff = if days == 0 {
            String::new()
        } else {
            let mut date = chrono::Local::now().date_naive();
            for _ in 1..days {
                date = date.pred_opt().unwrap_or(date);
            }
            date.format("%Y-%m-%d").to_string()
        };
        let mut out = UsageTotals::default();
        for (day, models) in &self.days {
            if !cutoff.is_empty() && day.as_str() < cutoff.as_str() {
                continue;
            }
            for row in models.values() {
                out.input = out.input.saturating_add(row.input);
                out.cached = out.cached.saturating_add(row.cached);
                out.written = out.written.saturating_add(row.written);
                out.output = out.output.saturating_add(row.output);
                out.requests = out.requests.saturating_add(row.requests);
            }
        }
        out
    }

    /// 读盘：缺文件 = 空账本；坏文件 = Err（静默重置等于銷账）。
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        match std::fs::read_to_string(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("读用量账本失败：{e}")),
            Ok(s) => serde_json::from_str(&s).map_err(|e| format!("用量账本不是合法 JSON：{e}")),
        }
    }

    /// 写盘（临时文件 + 原子 rename；半截文件不合法，必须换名）。
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| format!("建用量目录失败：{e}"))?;
            }
        }
        let body =
            serde_json::to_string_pretty(self).map_err(|e| format!("用量账本序列化失败：{e}"))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, body).map_err(|e| format!("写用量账本失败：{e}"))?;
        std::fs::rename(&tmp, path).map_err(|e| format!("用量账本换名失败：{e}"))
    }
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

    #[test]
    fn 账本校验与汇总() {
        use super::UsageLedger;
        let mut ledger = UsageLedger::default();
        // 非法 provider / 空模型被拒。
        assert!(ledger.record("DeepSeek", "m", None, 1, 0, 0, 1).is_err());
        assert!(ledger.record("deepseek", "  ", None, 1, 0, 0, 1).is_err());
        // 全零不记（since 保持 0，还是空账本）。
        ledger.record("deepseek", "deepseek-chat", None, 0, 0, 0, 0).expect("全零不报错");
        assert_eq!(ledger.since, 0);
        // 正常记：cached 钳制 ≤ input。
        ledger
            .record("deepseek", "deepseek-chat", Some("DeepSeek"), 100, 999, 10, 50)
            .expect("记");
        let all = ledger.totals(0);
        assert_eq!((all.input, all.cached, all.written, all.output, all.requests), (100, 100, 10, 50, 1));
        assert_eq!(all.tokens(), 150);
        // 今天窗口与有史以来一致（只有今天有数）。
        assert_eq!(ledger.totals(1), all);
        assert_eq!(ledger.totals(30), all);
    }

    #[test]
    fn 账本窗口按日历天切分() {
        use super::{DayUsage, UsageLedger};
        let mut ledger = UsageLedger::default();
        ledger.record("qwen", "qwen-plus", None, 10, 0, 0, 5).expect("记今天");
        // 手工塞一条昨天（直写 map，不走 record 的今天键）。
        let yesterday = (chrono::Local::now().date_naive().pred_opt().unwrap()).format("%Y-%m-%d").to_string();
        ledger.days.entry(yesterday).or_default().insert(
            "qwen|qwen-plus".to_owned(),
            DayUsage { input: 20, cached: 0, written: 0, output: 0, requests: 1 },
        );
        assert_eq!(ledger.totals(1).input, 10);
        assert_eq!(ledger.totals(2).input, 30);
        assert_eq!(ledger.totals(0).input, 30);
    }

    #[test]
    fn 账本落盘往返与坏文件() {
        use super::UsageLedger;
        let dir = std::env::temp_dir().join(format!("nt-cost-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("建目录");
        let path = dir.join("usage.json");
        // 缺文件 = 空账本，不报错。
        assert_eq!(UsageLedger::load(&path).expect("缺文件").since, 0);
        let mut ledger = UsageLedger::load(&path).expect("空账");
        ledger.record("ollama", "llama", None, 7, 0, 0, 3).expect("记");
        ledger.save(&path).expect("存");
        // 换名是原子的：tmp 不该留下。
        assert!(!path.with_extension("tmp").exists());
        let back = UsageLedger::load(&path).expect("读回");
        assert_eq!(back.totals(0).input, 7);
        // 坏文件 Err 不回默认（静默重置等于銷账）。
        std::fs::write(&path, "{broken").expect("写坏");
        assert!(UsageLedger::load(&path).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
