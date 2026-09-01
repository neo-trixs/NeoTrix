// ── Context Pipeline (from Claude Code: 5-layer compaction) ──

use std::collections::VecDeque;

use crate::neotrix::nt_io_provider::context_budget::estimate_tokens;

/// Precise token counter. **单一事实源 (P0-7)**: 委托 `context_budget::estimate_tokens`,
/// 即 tiktoken cl100k_base 精确计数优先, tiktoken 不可用 (如离线首跑) 时回退
/// CJK 感知逐字符估算。BPE 由 estimate_tokens 内部进程级 `OnceLock` 构建一次。
pub fn count_tokens(text: &str) -> usize {
    estimate_tokens(text)
}

/// 按 token 预算截断字符串 (Layer-3 microcompact 口径)。
/// 用统一 estimator 二分查找最大字符前缀: 预算按 `estimate_tokens` 计, 与
/// `count_tokens`/`estimate_tokens` 同口径, 不再用 `chars().take(200)` 字符口径。
/// 字符边界安全 (按 char 迭代, 不会切坏 UTF-8 / CJK)。
fn truncate_to_token_budget(text: &str, budget: usize) -> String {
    if estimate_tokens(text) <= budget {
        return text.to_string();
    }
    // 二分找最大前缀使 estimate_tokens(prefix) <= budget。
    let mut lo = 0usize;
    let mut hi = text.chars().count();
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        let prefix: String = text.chars().take(mid).collect();
        if estimate_tokens(&prefix) <= budget {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    text.chars().take(lo).collect()
}

#[derive(Debug, Clone)]
pub struct ContextTurn {
    pub role: String,
    pub content: String,
    pub token_count: usize,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct ContextPipeline {
    pub turns: VecDeque<ContextTurn>,
    pub max_tokens: usize,
    pub budget_high: f64,
    pub budget_low: f64,
    /// When true, `push` counts tokens with tiktoken instead of trusting the
    /// caller's chars/4 estimate (which over-counts CJK text badly).
    pub use_tiktoken: bool,
}

impl ContextPipeline {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            turns: VecDeque::new(),
            max_tokens,
            budget_high: 0.8,
            budget_low: 0.5,
            use_tiktoken: true,
        }
    }

    pub fn push(&mut self, role: &str, content: String, token_count: usize) {
        let token_count = if self.use_tiktoken {
            count_tokens(&content)
        } else {
            token_count
        };
        self.turns.push_back(ContextTurn {
            role: role.to_string(),
            content,
            token_count,
            priority: match role {
                "system" => 5,
                "tool" => 1,
                _ => 3,
            },
        });
        self.compact_if_needed();
    }

    pub fn total_tokens(&self) -> usize {
        self.turns.iter().map(|t| t.token_count).sum()
    }

    /// 5-layer compaction pipeline (Claude Code-inspired)
    pub(crate) fn compact_if_needed(&mut self) {
        let total = self.total_tokens();
        if total < (self.max_tokens as f64 * self.budget_high) as usize {
            return;
        }

        // Layer 1: Budget reduce — trim oversized tool outputs.
        // Token-consistent: estimator is chars/4 everywhere, so a turn budgeted
        // at `max_turn_tokens` keeps `max_turn_tokens * 4` chars (bytes/4 ≈ tokens).
        let max_turn_tokens = self.max_tokens / 4;
        for turn in &mut self.turns {
            if turn.token_count > max_turn_tokens && turn.priority < 4 {
                let budget_chars = max_turn_tokens * 4;
                let kept = turn.content.chars().take(budget_chars).collect::<String>();
                turn.content = format!(
                    "{}... [trimmed {} bytes]",
                    kept,
                    turn.content.len().saturating_sub(kept.len())
                );
                turn.token_count = if self.use_tiktoken {
                    count_tokens(&turn.content)
                } else {
                    kept.len() / 4
                };
            }
        }

        if self.total_tokens() < (self.max_tokens as f64 * self.budget_low) as usize {
            return;
        }

        // Layer 2: Snip — reduce temporal depth (keep newest)
        while self.turns.len() > 50 {
            self.turns.pop_front();
        }

        if self.total_tokens() < (self.max_tokens as f64 * self.budget_low) as usize {
            return;
        }

        // Layer 3: Microcompact — squeeze low-priority turns. Char-safe: the old
        // String::truncate(200) panicked when byte 200 landed mid-UTF-8-char
        // (any non-ASCII tool output now hits this path via tool priority 1).
        let mut i = 0;
        while i < self.turns.len()
            && self.total_tokens() > (self.max_tokens as f64 * self.budget_low) as usize
        {
            if self.turns[i].priority < 2 {
                let kept = truncate_to_token_budget(&self.turns[i].content, 50);
                self.turns[i].content = format!("{}...", kept);
                self.turns[i].token_count = if self.use_tiktoken {
                    count_tokens(&self.turns[i].content)
                } else {
                    estimate_tokens(&self.turns[i].content)
                };
            }
            i += 1;
        }

        if self.total_tokens() < (self.max_tokens as f64 * self.budget_low) as usize {
            return;
        }

        // Layer 4: Context collapse — distill evicted turns into a single
        // capped summary. Real condensation (preserves role + first line per
        // turn) instead of a no-op placeholder, and terminates deterministically
        // (the previous pop/push-front oscillation could loop forever whenever
        // the pipeline reached this layer with >10 turns).
        let mut distilled = String::new();
        while self.turns.len() > 10 && distilled.len() < 4_000 {
            let front = self.turns.pop_front().expect("guarded by len > 10");
            if !distilled.is_empty() {
                distilled.push('\n');
            }
            let first = front.content.lines().next().unwrap_or_default();
            distilled.push_str(&format!(
                "[{}] {}",
                front.role,
                first.chars().take(120).collect::<String>()
            ));
        }
        if !distilled.is_empty() {
            let distilled_tokens = if self.use_tiktoken {
                count_tokens(&distilled)
            } else {
                distilled.len() / 4
            };
            self.turns.push_front(ContextTurn {
                role: "summary".into(),
                content: distilled,
                token_count: distilled_tokens,
                priority: 1,
            });
        }

        // Layer 5: Auto-compact — hard cap
        while self.total_tokens() > self.max_tokens {
            if self.turns.len() <= 2 {
                break;
            }
            self.turns.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_pipeline_simple() {
        let mut ctx = ContextPipeline::new(1000);
        ctx.use_tiktoken = false; // unit test of pipeline mechanics, not counting
        ctx.push("user", "test message".into(), 10);
        assert_eq!(ctx.turns.len(), 1);
        assert!(ctx.total_tokens() <= ctx.max_tokens);
    }

    #[test]
    fn test_context_pipeline_compaction() {
        let mut ctx = ContextPipeline::new(500);
        ctx.use_tiktoken = false; // drive compaction with exact caller estimates
        for i in 0..20 {
            ctx.push("user", format!("message {}", i), 100);
        }
        assert!(ctx.total_tokens() <= ctx.max_tokens);
    }

    #[test]
    fn test_context_pipeline_collapse_terminates() {
        // Regression: the old Layer 4 oscillated pop/push-front forever once
        // the pipeline reached it with >10 turns. Sixty small turns force
        // Layer 2 -> Layer 4 while staying well under the hard cap.
        let mut ctx = ContextPipeline::new(5000);
        ctx.use_tiktoken = false; // rely on caller estimates so compaction triggers
        for i in 0..60 {
            ctx.push("user", format!("message {} {}", i, "x".repeat(40)), 100);
        }
        assert!(ctx.total_tokens() <= ctx.max_tokens);
        assert!(ctx.turns.iter().any(|t| t.role == "summary"));
    }

    #[test]
    fn test_context_pipeline_tool_priority() {
        let mut ctx = ContextPipeline::new(10_000);
        ctx.push("system", "sys".into(), 5);
        ctx.push("tool", "big tool result".into(), 10);
        ctx.push("user", "hi".into(), 10);
        ctx.push("assistant", "ok".into(), 10);
        assert_eq!(ctx.turns[0].priority, 5);
        assert_eq!(ctx.turns[1].priority, 1);
        assert_eq!(ctx.turns[2].priority, 3);
        assert_eq!(ctx.turns[3].priority, 3);
    }

    #[test]
    fn test_context_pipeline_layer3_non_ascii_no_panic() {
        // Regression: Layer 3 used String::truncate(200) which panics when byte
        // 200 falls mid-UTF-8-char. Tool turns (priority 1) now enter this path.
        let mut ctx = ContextPipeline::new(5000);
        let big = "中文数据负载".repeat(50);
        for i in 0..14 {
            ctx.push("tool", format!("{} {}", i, big), 300);
        }
        assert!(ctx.total_tokens() <= ctx.max_tokens);
    }

    #[test]
    fn test_count_tokens_cjk_is_precise() {
        // Determinism: same text must count identically every time.
        let cjk = "这是一个用于验证中文分词精确性的测试句子，包含标点符号和数字123，以及英文 mixed content here.";
        let a = count_tokens(cjk);
        let b = count_tokens(cjk);
        assert_eq!(a, b, "token counting must be deterministic");

        // Known reference: cl100k encodes "hello world" as 2 tokens.
        assert_eq!(count_tokens("hello world"), 2);

        // CJK is ~1-3 tokens per char under cl100k; the old bytes/4 estimate
        // divides UTF-8 byte length (3 bytes/char for CJK) by 4, so it
        // systematically UNDER-counts CJK-heavy text. Precise must be >= crude
        // here — a full-character CJK string never collapses below bytes/4.
        let precise = a;
        let crude = cjk.len() / 4;
        assert!(
            precise >= crude.saturating_sub(1),
            "precise {} should not fall below crude {} for CJK text (old estimator under-counts)",
            precise,
            crude
        );
        assert!(precise > 0);

        // English stays ~4 chars/token: precise should track crude within a
        // small band rather than blowing up.
        let english = "the quick brown fox jumps over the lazy dog and runs far away from the town";
        let en_precise = count_tokens(english);
        let en_crude = english.len() / 4;
        assert!(
            en_precise >= en_crude.saturating_sub(2),
            "english precise {} vs crude {}",
            en_precise,
            en_crude
        );
        assert!(
            en_precise <= en_crude + 4,
            "english precise {} vs crude {}",
            en_precise,
            en_crude
        );
    }

    #[test]
    fn test_context_pipeline_push_uses_tiktoken() {
        let mut pipe = ContextPipeline::new(10_000);
        assert!(pipe.use_tiktoken, "tiktoken should be enabled by default");

        // A payload whose true token count differs from the caller estimate.
        let cjk = "上下文管线测试：中文内容不应该被错误估计，每一个字符大约一个token。".to_string();
        pipe.push("user", cjk.clone(), 9999); // caller's estimate is deliberately wrong
        let turn = pipe.turns.front().expect("one turn");
        assert!(
            turn.token_count < 9999,
            "tiktoken should override the caller estimate: {}",
            turn.token_count
        );
        assert!(turn.token_count > 0);
        // The turn's count must equal the precise counter's output.
        assert_eq!(turn.token_count, count_tokens(&cjk));
    }

    #[test]
    fn test_context_pipeline_fallback_without_tiktoken() {
        let mut pipe = ContextPipeline::new(10_000);
        pipe.use_tiktoken = false;
        let text = "plain ascii payload for fallback path".to_string();
        pipe.push("user", text.clone(), 42);
        let turn = pipe.turns.front().expect("one turn");
        assert_eq!(
            turn.token_count, 42,
            "caller estimate should be honored when tiktoken disabled"
        );
    }
}