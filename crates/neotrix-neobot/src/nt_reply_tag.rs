//! `nt_reply_tag` — 每轮回复标签（消息流 agent 用；对标 Claude/Codex/Grok 气泡标签）。
//!
//! 给每轮回复附加机器可读标签，前端 chips 直消：
//! `model`（模型名）· `mode`（direct/passthrough/fallback 三态）·
//! `tools`（本轮调用的工具名列表，去重保序）·
//! `usage`（input/output tokens ＋ cost 微分 `cost_micros`）。
//!
//! 纪律：未知模型永不猜价格（计价走 `nt_cost::cost_for` 同律）；
//! 非展示工具（`reply` / `set_turn_status`）不进 `tools`；
//! `side-effect:<kind>` 折叠为 `<kind>`；全程无 `unwrap`/`expect`/`panic`。

use serde::{Deserialize, Serialize};

use crate::nt_cost::{CostPolicy, cost_for, price_for};

/// 回复路由三态（前端 chips 直显英文原串）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplyMode {
    /// 显式端点直连（`env`/provider 指定、本地 CLI/HTTP 配置引擎）。
    Direct,
    /// 经晶体核心透传（配对且在线，池内解析）。
    Passthrough,
    /// 本地回显兜底（`echo`，零耗不断档）。
    Fallback,
}

impl Default for ReplyMode {
    fn default() -> Self {
        Self::Direct
    }
}

impl ReplyMode {
    /// 展示串（serde 小写同形）。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Passthrough => "passthrough",
            Self::Fallback => "fallback",
        }
    }

    /// 解析展示串，未知回 `None`（调用方按 direct 降级，不猜）。
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "direct" => Some(Self::Direct),
            "passthrough" => Some(Self::Passthrough),
            "fallback" => Some(Self::Fallback),
            _ => None,
        }
    }
}

/// 单轮用量（tokens ＋ 美元/微美元双写；`measured=false` 即未计量不猜价）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TurnUsage {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
    /// 微美元（`cost_usd * 1e6` 四舍五入；展示用，前端 `tokens/cost` 直消）。
    pub cost_micros: i64,
    #[serde(default)]
    pub measured: bool,
}

impl TurnUsage {
    /// 组装（负数钳 0；非有限 cost 归 0；微分同步算）。
    pub fn new(input_tokens: i64, output_tokens: i64, cost_usd: f64, measured: bool) -> Self {
        let cost = if cost_usd.is_finite() && cost_usd > 0.0 {
            cost_usd
        } else {
            0.0
        };
        Self {
            input_tokens: input_tokens.max(0),
            output_tokens: output_tokens.max(0),
            cost_usd: cost,
            cost_micros: cost_micros(cost),
            measured,
        }
    }

    /// 空用量（零耗未计量；echo/纯本地轮用）。
    pub fn empty() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            cost_usd: 0.0,
            cost_micros: 0,
            measured: false,
        }
    }
}

/// 微美元换算（非有限/负值归 0；i64 上溢钳制）。
pub fn cost_micros(cost_usd: f64) -> i64 {
    if !cost_usd.is_finite() || cost_usd <= 0.0 {
        return 0;
    }
    let micros = cost_usd * 1_000_000.0;
    if micros >= i64::MAX as f64 {
        i64::MAX
    } else {
        micros.round() as i64
    }
}

/// 每轮回复标签（serde 前后端契约；全字段默认可缺省）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TurnLabels {
    /// 模型名（空即 `echo` 兜底由调用方填）。
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub mode: ReplyMode,
    /// 本轮工具名（去重保序，上限 20；空即无工具纯回复）。
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub usage: TurnUsage,
}

impl TurnLabels {
    /// 组装（模型空白回 `echo`；工具走过滤去重；用量原样）。
    pub fn new(
        model: &str,
        mode: ReplyMode,
        tools: Vec<String>,
        usage: TurnUsage,
    ) -> Self {
        let name = model.trim();
        Self {
            model: if name.is_empty() {
                "echo".to_owned()
            } else {
                name.to_owned()
            },
            mode,
            tools: normalize_tools(tools),
            usage,
        }
    }

    /// 空标签（模型占位；调用方无数据时用，前端藏 chips 不炸）。
    pub fn empty(model: &str, mode: ReplyMode) -> Self {
        Self::new(model, mode, Vec::new(), TurnUsage::empty())
    }
}

/// 工具名归一（前端 chips 口径唯一源）：
///
/// - 去空白/空串；`reply` 与 `set_turn_status` 为协议噪音不进标签；
/// - `side-effect:<kind>` 折叠为 `<kind>`（CLI 回传能力名）；
/// - 去重保序，上限 20（防模型刷屏打爆气泡）。
pub fn normalize_tools(raw: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in raw {
        let mut name = item.trim().to_owned();
        if name.is_empty() {
            continue;
        }
        if name == "reply" || name == "set_turn_status" {
            continue;
        }
        if let Some(kind) = name.strip_prefix("side-effect:") {
            name = kind.trim().to_owned();
            if name.is_empty() {
                continue;
            }
        }
        if out.iter().any(|seen| seen == &name) {
            continue;
        }
        out.push(name);
        if out.len() >= 20 {
            break;
        }
    }
    out
}

/// trace kinds → 工具名（`agent_run` 透传路径用；服务端只给 trace 时本地派生）。
pub fn tools_from_trace(kinds: &[String]) -> Vec<String> {
    normalize_tools(kinds.to_vec())
}

/// 用量组装（含诚实计价）：tokens 差值走账本，`measured` 与 `cost` 走
/// `nt_cost` 同律（运维配置 > 官方价目 > 引擎自带 > 归零未计量）。
pub fn usage_for_turn(
    model: &str,
    in_tokens: i64,
    out_tokens: i64,
    ledger_cost_usd: f64,
) -> TurnUsage {
    let policy = CostPolicy::from_env();
    if policy.is_configured() || price_for(model).is_some() {
        let (cost, measured) = cost_for(policy, model, in_tokens, out_tokens);
        return TurnUsage::new(in_tokens, out_tokens, cost, measured);
    }
    if ledger_cost_usd.is_finite() && ledger_cost_usd > 0.0 {
        return TurnUsage::new(in_tokens, out_tokens, ledger_cost_usd, true);
    }
    TurnUsage::new(in_tokens, out_tokens, 0.0, false)
}

/// 标签组装（含计价；跑轮命令唯一入口：账本差值 + 步骤工具 + 路由三态）。
pub fn labels_for_turn(
    model: &str,
    mode: ReplyMode,
    step_tools: Vec<String>,
    in_tokens: i64,
    out_tokens: i64,
    ledger_cost_usd: f64,
) -> TurnLabels {
    TurnLabels::new(
        model,
        mode,
        step_tools,
        usage_for_turn(model, in_tokens, out_tokens, ledger_cost_usd),
    )
}

#[cfg(test)]
mod tests {
    use super::{ReplyMode, TurnLabels, TurnUsage, cost_micros, normalize_tools, usage_for_turn};

    #[test]
    fn mode_roundtrip() {
        for mode in [ReplyMode::Direct, ReplyMode::Passthrough, ReplyMode::Fallback] {
            assert_eq!(ReplyMode::parse(mode.as_str()), Some(mode));
        }
        assert_eq!(ReplyMode::parse("nope"), None);
        assert_eq!(ReplyMode::default(), ReplyMode::Direct);
    }

    #[test]
    fn tools_filter_dedupes_and_folds_side_effects() {
        let got = normalize_tools(vec![
            "reply".to_owned(),
            "set_turn_status".to_owned(),
            "bash".to_owned(),
            "bash".to_owned(),
            "side-effect:web_search".to_owned(),
            "  ".to_owned(),
            "read_file".to_owned(),
        ]);
        assert_eq!(got, vec!["bash", "web_search", "read_file"]);
    }

    #[test]
    fn micros_rounds_and_clamps() {
        assert_eq!(cost_micros(0.00032), 320);
        assert_eq!(cost_micros(0.0), 0);
        assert_eq!(cost_micros(-1.0), 0);
        assert_eq!(cost_micros(f64::NAN), 0);
        assert_eq!(cost_micros(f64::INFINITY), 0);
    }

    #[test]
    fn usage_never_guesses_without_price() {
        let usage = usage_for_turn("qwen-local-unknown-xyz", 100, 50, 0.0);
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert!(!usage.measured);
        assert_eq!(usage.cost_usd, 0.0);
        assert_eq!(usage.cost_micros, 0);
    }

    #[test]
    fn labels_serde_frontend_shape() {
        let labels = TurnLabels::new(
            "neotrix-crystal",
            ReplyMode::Passthrough,
            vec!["bash".to_owned()],
            TurnUsage::new(120, 45, 0.00032, true),
        );
        let value = serde_json::to_value(&labels).expect("serialize");
        assert_eq!(
            value.get("model").and_then(|v| v.as_str()),
            Some("neotrix-crystal")
        );
        assert_eq!(
            value.get("mode").and_then(|v| v.as_str()),
            Some("passthrough")
        );
        assert_eq!(value.get("tools").and_then(|v| v.as_array()).map(|a| a.len()), Some(1));
        let usage = value.get("usage").expect("usage");
        assert_eq!(
            usage.get("input_tokens").and_then(|v| v.as_i64()),
            Some(120)
        );
        assert_eq!(
            usage.get("cost_micros").and_then(|v| v.as_i64()),
            Some(320)
        );
        // 空模型回 echo，前端永不拿空串。
        let fallback = TurnLabels::empty("", ReplyMode::Fallback);
        assert_eq!(fallback.model, "echo");
        assert!(fallback.tools.is_empty());
    }
}
