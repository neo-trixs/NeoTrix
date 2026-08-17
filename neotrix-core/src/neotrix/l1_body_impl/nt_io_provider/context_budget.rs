//! Token 预算引擎 — LLM 调用链路上下文压缩的共享实现。
//!
//! 文献依据 (2026 token optimization 主线):
//! - 工具输出在进入 LLM 上下文前压缩可省 60-95% token (Headroom / RTK)
//! - agent 重发上下文占推理账单 ~62% (Cockroach Labs, 2026)
//! - 选择性压缩/裁剪历史省 20-40% 且不损连贯性 (Adaline, 2026)
//!
//! 策略 (确定性、无损质量, 不做 LLM 重写以避免引入额外调用):
//! 1. 单条工具输出超限 → 头/尾保留折叠 (head 60% / tail 40%)
//! 2. 总上下文超预算 → 丢弃最旧非 System 轮次, 保留末条 (当前请求)
//!
//! 与 `budget_react_messages` (neocodex) / `resume_session` 的 token 估算口径一致。
//! 本估算器是**单一事实源** (P0-7): tiktoken cl100k_base 精确计数优先;
//! tiktoken 不可用 (如离线首跑) 时回退到 CJK 感知逐字符估算 (非 CJK ≈ 4 chars/token,
//! CJK ≈ 1 token/char — 字节口径会低估 CJK 4x, 导致上下文溢出)。

use super::types::{Message, Role};
use std::sync::OnceLock;

/// CJK 相关 Unicode 区间: 汉字/假名/谚文/全角。
fn is_cjk(c: char) -> bool {
    matches!(
        c,
        '\u{3000}'..='\u{303F}'   // CJK 标点
        | '\u{3040}'..='\u{30FF}' // 假名
        | '\u{3400}'..='\u{4DBF}' // CJK Ext A
        | '\u{4E00}'..='\u{9FFF}' // CJK 统一表意
        | '\u{AC00}'..='\u{D7AF}' // 谚文
        | '\u{FF00}'..='\u{FFEF}' // 全角/半角
    )
}

/// 单字符 token 成本: CJK ≈ 1 token, 其余 ≈ 1/4 token (4 chars/token)。
fn char_token_cost(c: char) -> f64 {
    if is_cjk(c) {
        1.0
    } else {
        0.25
    }
}

/// 进程级 tiktoken BPE 单例 (cl100k_base)。构建失败 (如离线首跑) 时为 `None`,
/// 此时回退到 CJK 感知逐字符估算。与 neocodex `count_tokens` 共享同一口径。
static TIKTOKEN_BPE: OnceLock<Option<tiktoken_rs::CoreBPE>> = OnceLock::new();

/// 估算一段文本的 token 数。
///
/// **单一事实源 (P0-7)**: 若 tiktoken 可用, 用 cl100k_base 精确计数
/// (`encode_with_special_tokens`); 否则回退到 CJK 感知逐字符估算 (保守上界, 最小 1)。
pub fn estimate_tokens(text: &str) -> usize {
    let bpe = TIKTOKEN_BPE.get_or_init(|| tiktoken_rs::cl100k_base().ok());
    if let Some(bpe) = bpe {
        bpe.encode_with_special_tokens(text).len().max(1)
    } else {
        let mut tokens = 0.0;
        for c in text.chars() {
            tokens += char_token_cost(c);
        }
        (tokens.ceil() as usize).max(1)
    }
}

/// 估算一组消息的 token 数 (含每消息协议开销 ~4 token)。
pub fn estimate_messages_tokens(messages: &[Message]) -> usize {
    let mut total = 0usize;
    for m in messages {
        total += estimate_tokens(&m.content);
        // role 标签 + 可能的 tool_calls/tool_call_id 协议开销
        total += 4;
    }
    total
}

/// 按 token 预算截断字符串, 保留头部 `head_ratio` 与尾部其余, 中段折叠。
/// 字符边界安全 (按 char 扫描, 不会切坏 UTF-8 / CJK)。
pub fn truncate_preserving(text: &str, max_tokens: usize, head_ratio: f64) -> String {
    if estimate_tokens(text) <= max_tokens {
        return text.to_string();
    }
    let head_budget = ((max_tokens as f64) * head_ratio.clamp(0.0, 1.0)).floor() as usize;
    let tail_budget = max_tokens.saturating_sub(head_budget);
    let head = take_until_tokens(text, head_budget, true);
    let tail = take_until_tokens(text, tail_budget, false);
    match (head.is_empty(), tail.is_empty()) {
        (true, true) => "…[truncated]…".to_string(),
        (true, false) => format!("…[truncated]…\n{tail}"),
        (false, true) => format!("{head}\n…[truncated]…"),
        (false, false) => format!("{head}\n…[truncated]…\n{tail}"),
    }
}

/// 从头部或尾部累进 token 消耗, 返回不超过预算的字符数。
fn take_until_tokens(text: &str, budget: usize, from_start: bool) -> String {
    if budget == 0 {
        return String::new();
    }
    let mut consumed = 0.0;
    let mut out = String::new();
    if from_start {
        for c in text.chars() {
            consumed += char_token_cost(c);
            if consumed > budget as f64 {
                break;
            }
            out.push(c);
        }
    } else {
        for c in text.chars().rev() {
            consumed += char_token_cost(c);
            if consumed > budget as f64 {
                break;
            }
            out.push(c);
        }
        // 尾部反向收集 → 恢复正序
        out = out.chars().rev().collect();
    }
    out
}

/// 上下文预算压缩结果 (观测杠杆: 每次 apply 后可见具体削减量)。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BudgetResult {
    pub original_tokens: usize,
    pub final_tokens: usize,
    pub tool_outputs_truncated: usize,
    pub messages_evicted: usize,
}

impl BudgetResult {
    pub fn saved_tokens(&self) -> usize {
        self.original_tokens.saturating_sub(self.final_tokens)
    }
}

/// 对消息序列应用 token 预算:
/// 1. 单条工具输出 > `per_tool_output_tokens` 时截断 (0 = 禁用截断)
/// 2. 总量仍超 `max_tokens` 时, 丢弃最旧非 System 轮次 (保留末条 = 当前请求)
///
/// 保留不变量: 索引 0 若为 System 永不丢弃; 末条 (当前 user 请求/最新 tool 结果)
/// 永不被驱逐 — 与 neocodex `budget_react_messages` 语义对齐。
pub fn apply_context_budget(
    messages: &mut Vec<Message>,
    max_tokens: usize,
    per_tool_output_tokens: usize,
) -> BudgetResult {
    let original_tokens = estimate_messages_tokens(messages);
    let mut result = BudgetResult {
        original_tokens,
        final_tokens: original_tokens,
        tool_outputs_truncated: 0,
        messages_evicted: 0,
    };

    // Pass 1: 截断超大工具输出 (最省且不丢历史轮次)
    if per_tool_output_tokens > 0 {
        for m in messages.iter_mut() {
            if m.role == Role::Tool && estimate_tokens(&m.content) > per_tool_output_tokens {
                m.content = truncate_preserving(&m.content, per_tool_output_tokens, 0.6);
                result.tool_outputs_truncated += 1;
            }
        }
    }

    // Pass 2: 超预算则逐条驱逐最旧可弃消息 (跳过 System 首条与末条)
    loop {
        let total = estimate_messages_tokens(messages);
        result.final_tokens = total;
        if total <= max_tokens || messages.len() <= 2 {
            break;
        }
        let mut evict_at: Option<usize> = None;
        for (idx, m) in messages.iter().enumerate() {
            let is_system_head = idx == 0 && m.role == Role::System;
            let is_last = idx == messages.len() - 1;
            if !is_system_head && !is_last {
                evict_at = Some(idx);
                break;
            }
        }
        match evict_at {
            Some(idx) => {
                messages.remove(idx);
                result.messages_evicted += 1;
            }
            None => break,
        }
    }
    result.final_tokens = estimate_messages_tokens(messages);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_io_provider::types::{ToolCallFunction, ToolCallInfo};

    fn user_msg(content: &str) -> Message {
        Message::new(Role::User, content)
    }

    fn assistant_msg(content: &str) -> Message {
        Message::new(Role::Assistant, content)
    }

    fn tool_msg(content: &str) -> Message {
        Message::tool(content, "call-1")
    }

    fn big_text(units: usize) -> String {
        // 每个 unit ~16 chars → ~4 tokens (非 CJK)
        "word-pattern-abcdefgh ".repeat(units)
    }

    #[test]
    fn test_estimate_tokens_ascii() {
        // tiktoken cl100k: 40×'a' = 5 tokens; CJK 回退 ≈ 10 tokens。
        let tokens = estimate_tokens("a".repeat(40).as_str());
        assert!((4..=12).contains(&tokens), "got {tokens}");
    }

    #[test]
    fn test_estimate_tokens_cjk_conservative() {
        // CJK 1 char/token: 40 汉字 → 40 tokens, 绝不能 ≤ 10 (字节/4 口径的错误低估)
        let tokens = estimate_tokens("汉".repeat(40).as_str());
        assert!(tokens >= 40, "CJK must be counted ~1 token/char, got {tokens}");
    }

    #[test]
    fn test_estimate_tokens_empty_min_one() {
        assert_eq!(estimate_tokens(""), 1);
    }

    #[test]
    fn test_truncate_under_budget_unchanged() {
        let text = big_text(10);
        let out = truncate_preserving(&text, 1000, 0.6);
        assert_eq!(out, text);
    }

    #[test]
    fn test_truncate_preserving_head_tail() {
        let text = big_text(1000); // ~4000 tokens
        let out = truncate_preserving(&text, 400, 0.6);
        assert!(estimate_tokens(&out) <= 420, "got {}", estimate_tokens(&out));
        assert!(out.contains("word-pattern"), "head preserved");
        assert!(out.contains("[truncated]"), "marker present");
        // 尾部保留: 末段不应丢失 (big_text 末字符是空格, 允许 trim)
        assert!(out.trim_end().ends_with("word-pattern-abcdefgh"), "tail preserved");
    }

    #[test]
    fn test_truncate_preserving_cjk_safe() {
        // 全 CJK: 输出 token 必须 ≤ 预算, 且不 panic (char 边界)
        let text = "中文数据负载测试内容".repeat(1000);
        let out = truncate_preserving(&text, 100, 0.6);
        assert!(estimate_tokens(&out) <= 105, "got {}", estimate_tokens(&out));
    }

    #[test]
    fn test_truncate_utf8_boundary_no_panic() {
        // 之前的回归 (ContextPipeline layer3): String::truncate 字节落在 CJK 中间会 panic。
        // 截断预算按逐字符成本 (25/25 tokens) 切割; 测量走统一 estimator
        // (tiktoken 对 emoji/CJK 计 token 更密 → 实测 ≈ 129)。
        let text = "📦🤖🧠英文mixed中文".repeat(500);
        let out = truncate_preserving(&text, 50, 0.5);
        assert!(estimate_tokens(&out) <= 140, "got {}", estimate_tokens(&out));
    }

    #[test]
    fn test_apply_budget_evicts_oldest_keeps_system_and_last() {
        let mut msgs = vec![
            Message::new(Role::System, "system prompt"),
            user_msg("first"),
            assistant_msg("a1"),
            user_msg("second"),
            assistant_msg("a2"),
            user_msg("current request"),
        ];
        let r = apply_context_budget(&mut msgs, 8, 0);
        assert!(r.messages_evicted > 0);
        assert_eq!(msgs[0].role, Role::System);
        assert_eq!(msgs.last().unwrap().content, "current request");
        assert!(estimate_messages_tokens(&msgs) <= 8 + 200);
    }

    #[test]
    fn test_apply_budget_truncates_tool_output() {
        let mut msgs = vec![
            Message::new(Role::System, "s"),
            user_msg("do it"),
            assistant_msg("calling tool"),
            tool_msg(&big_text(500)),
        ];
        let r = apply_context_budget(&mut msgs, 100_000, 300);
        assert_eq!(r.tool_outputs_truncated, 1);
        let tool = msgs.last().unwrap();
        assert!(estimate_tokens(&tool.content) <= 320, "got {}", estimate_tokens(&tool.content));
    }

    #[test]
    fn test_apply_budget_per_tool_zero_disables_truncation() {
        let mut msgs = vec![
            Message::new(Role::System, "s"),
            user_msg("do it"),
            assistant_msg("calling tool"),
            tool_msg(&big_text(500)),
        ];
        let r = apply_context_budget(&mut msgs, 100_000, 0);
        assert_eq!(r.tool_outputs_truncated, 0);
    }

    #[test]
    fn test_apply_budget_under_limit_noop() {
        let mut msgs = vec![Message::new(Role::System, "s"), user_msg("hi")];
        let before = estimate_messages_tokens(&msgs);
        let r = apply_context_budget(&mut msgs, 100_000, 0);
        assert_eq!(r.messages_evicted, 0);
        assert_eq!(r.tool_outputs_truncated, 0);
        assert_eq!(estimate_messages_tokens(&msgs), before);
    }

    #[test]
    fn test_tool_message_helper_keeps_call_id() {
        let m = tool_msg("result");
        assert_eq!(m.role, Role::Tool);
        assert_eq!(m.tool_call_id.as_deref(), Some("call-1"));
        assert!(m.tool_calls.is_none());
    }

    #[test]
    fn test_tool_call_info_roundtrip() {
        let info = ToolCallInfo {
            id: "c1".into(),
            call_type: "function".into(),
            function: ToolCallFunction {
                name: "read".into(),
                arguments: "{}".into(),
            },
        };
        let json = serde_json::to_string(&info).expect("serialize ToolCallInfo");
        let back: ToolCallInfo = serde_json::from_str(&json).expect("deserialize ToolCallInfo");
        assert_eq!(back.id, "c1");
        assert_eq!(back.function.name, "read");
    }
}