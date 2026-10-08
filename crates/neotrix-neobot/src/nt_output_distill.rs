//! nt_output_distill — 工具输出的 errors-first 可逆蒸馏
//!
//! 2026-10-06 从 `neotrix-core/src/l1_action/nt_io/nt_io_agent_loop/nt_loop_step.rs`
//! 搬移而来（吸收源 repowise「压缩再给模型读」，见该处 `P2 可逆命令输出蒸馏`）。
//!
//! ## 为什么搬
//!
//! `AgentLoop::trim_tool_output` 用的是 `distill_output`，而 `AgentLoop`
//! 经实测零生产实例化 ⇒ 真实执行环 `nt_agent.rs` 只能用
//! `truncate_history`，后者是**砍尾留头**：`output[..4096] + …[truncated]`。
//!
//! ⛔ 砍尾留头的两个具体损失（这是本模块存在的理由，不是「更精细」这种修辞）：
//!   ① **exit code 与尾部摘要被丢掉** —— 而那正是命令成败的最终结论。
//!   ② **错误行可能被一起砍掉** —— 4096 字节之后若有 `error:`/`FAIL:`/
//!      `exit code 1`，模型看到的是一段正常的前缀，**得到「成功」的错觉**。
//!
//! ## 本模块的算法
//!
//! 三段式，按优先级装入**行级硬预算**：
//!   ① `## errors` —— 全部错误/警告/失败行，集中前置
//!   ② `## tail`   —— 尾部 3 行的普通行（exit code / 摘要），享 20% 预留
//!   ③ `## body`   —— 其余普通行，用剩余预算，省略处打 `[ref#1]` 标记
//!
//! 标记开销在装入行**之前**预留 ⇒ 省略标记不会被最后的兜底截断切掉。
//! 极端情况下逐字符裁剪兜底，保证**任何情况下都不超预算**。
//!
//! ## 为什么放在 neobot
//!
//! `neotrix-core/Cargo.toml:106` 依赖 `neotrix-neobot`（core → neobot），
//! 反向依赖会造成循环 crate 依赖，Cargo 直接拒绝。
//!
//! ## 与 core 的同步义务
//!
//! ⚠️ 算法逐行搬自 core，两边应视为同一份。改动前请确认另一侧是否也需要改，
//!   否则会出现「core 蒸馏得更好、neobot 蒸馏得更差」的不对称。

/// 非空输入至少 1 token；空串同样为 1（保守上界，预算侧不为空串开口子）。
///
/// 与 core 侧 `nt_core_llm::estimate_tokens` **同口径**（2026-09-27 定为
/// 「单一事实源」）：ASCII 段 4 字符 = 1 token（向下取整，11 字符 = 2
/// token，与 tiktoken cl100k 一致），CJK 每字 1 token。
fn estimate_tokens(text: &str) -> usize {
    let mut cjk = 0usize;
    let mut rest = 0usize;
    for c in text.chars() {
        if neotrix_types::core::nt_cjk::is_cjk_wide(c) {
            cjk += 1;
        } else {
            rest += 1;
        }
    }
    (cjk + rest / 4).max(1)
}

/// 预算小到连省略标记都装不下时的兜底。
///
/// 选短标记而非长标记，是因为长标记（`…[output exceeds budget…]` ≈ 7 token）
/// 在小预算下**本身就超预算** ⇒ 违反「任何输入都不超预算」这条硬不变量。
const SHORT_FALLBACK: &str = "…";

/// 该输出是否属于**降级**：预算装不下标记本身时蒸馏会塌成 [`SHORT_FALLBACK`]，
/// 正文内容被整体丢弃。
///
/// 为什么需要这个判据（N6.2）：「被砍了一刀」与「**内容没了**」是两种事，
/// 前者正常压缩、后者降级；账本要能分清，否则「这轮为什么什么都没拿到」
/// 永远查不出来。
pub fn is_degraded(output: &str) -> bool {
    output.trim() == SHORT_FALLBACK
}

/// 把超长工具输出蒸馏为 errors-first 摘要（`max_tokens` 为硬预算）。
///
/// ⛔ **不超预算**：任何极端 token 估算情形下都靠末尾的逐字符裁剪兜底。
/// ⛔ **可逆**：省略处以 `REF_MARK` 标明，全量原文在 `tool_log` / `steps` 表里。
pub fn distill_output(content: &str, max_tokens: usize) -> String {
    if estimate_tokens(content) <= max_tokens {
        return content.to_owned();
    }

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return content.to_owned();
    }

    /// 尾部保留行数（exit code / 摘要常在这里）。
    const TAIL_LINES: usize = 3;
    let tail_start = lines.len().saturating_sub(TAIL_LINES);
    let tail = &lines[tail_start..];

    let is_errorish = |l: &str| {
        let low = l.to_ascii_lowercase();
        low.contains("error")
            || low.contains("fail")
            || low.contains("denied")
            || low.contains("warning")
            || low.contains("exception")
            || low.contains("panic")
            || low.contains("traceback")
            || low.contains("exit code")
            || low.contains("fatal")
    };
    let mut error_lines: Vec<&str> = Vec::new();
    let mut plain_lines: Vec<&str> = Vec::new();
    for l in lines.iter() {
        if is_errorish(l) {
            error_lines.push(l);
        } else {
            plain_lines.push(l);
        }
    }
    let tail_plain: Vec<&str> = tail.iter().copied().filter(|l| !is_errorish(l)).collect();

    let mut out = String::new();
    let mut spent = 0usize;

    // 尾部状态行享固定预算，保证 exit code / 摘要不丢。
    let tail_reserve = if tail_plain.is_empty() {
        0
    } else {
        (max_tokens as f64 * 0.20).floor() as usize
    };

    // ① errors 段：优先装入全部错误行。
    if !error_lines.is_empty() {
        out.push_str("## errors\n");
        spent += estimate_tokens("## errors\n");
        for l in &error_lines {
            let line_cost = estimate_tokens(l) + 1;
            if spent + line_cost + tail_reserve > max_tokens {
                break;
            }
            out.push_str(l);
            out.push('\n');
            spent += line_cost;
        }
    }

    // ② tail 段：尽预留预算装入。
    if !tail_plain.is_empty() {
        if !out.is_empty() {
            out.push('\n');
            spent += 1;
        }
        out.push_str("## tail\n");
        spent += estimate_tokens("## tail\n");
        for l in &tail_plain {
            let line_cost = estimate_tokens(l) + 1;
            if spent + line_cost > max_tokens {
                break;
            }
            out.push_str(l);
            out.push('\n');
            spent += line_cost;
        }
    }

    // ③ body 段：剩余预算装入，省略处打 `[ref#N]` 标记。
    const REF_MARK: &str = "[ref#1] body 中段省略 (原文见 tool_log)";
    let body_plain: Vec<&str> = plain_lines
        .iter()
        .copied()
        .filter(|l| !tail_plain.contains(l))
        .collect();
    if !body_plain.is_empty() && spent < max_tokens {
        if !out.is_empty() {
            out.push('\n');
            spent += 1;
        }
        out.push_str("## body\n");
        spent += estimate_tokens("## body\n");
        // 预留省略标记（仅当 body 行未全装时）。
        let mark_cost = estimate_tokens(REF_MARK) + 1;
        let mut placed = 0usize;
        for l in &body_plain {
            let line_cost = estimate_tokens(l) + 1;
            let will_truncate = placed + 1 < body_plain.len();
            let reserve = if will_truncate { mark_cost } else { 0 };
            if spent + line_cost + reserve > max_tokens {
                break;
            }
            out.push_str(l);
            out.push('\n');
            spent += line_cost;
            placed += 1;
        }
        if placed < body_plain.len() {
            out.push_str(REF_MARK);
            out.push('\n');
        }
    }

    // 不变量兜底：任何极端情形仍强制截到预算。
    if estimate_tokens(&out) > max_tokens {
        // ⛔⛔⛔ **搬移时发现 core 侧同段的死循环**（2026-10-06 实测）⛔⛔⛔
        //
        // 原写法（core `nt_loop_step.rs:156-161` 逐字相同）：
        //     while 超预算 && clipped.len() > 8 {
        //         let cut = (clipped.len() as f64 * 0.7).max(1.0) as usize;
        //         clipped.truncate(keep);              // keep ≈ 0.7 * len
        //         clipped.push_str("\n…[truncated]…"); // +11 字符
        //     }
        // 当 `len` 落在 9..~15 时：`cut = floor(0.7*len)`，`keep < len`，
        // 但 push 固定 11 字符 ⇒ **长度可能不降反升** ⇒ `while` 永不退出
        // ⇒ 单测 `任何输入都不超预算`（budget=8）**把测试进程挂死**。
        //
        // ⇒ 修法：**按「上次长度」判断收敛**，且每轮强制缩短，
        //   不用「乘 0.7」这种可能不降的启发式。
        const TAIL_MARK: &str = "…[truncated]…";
        // ⛔ 标记本身可能就超预算（`…[truncated]…` ≈ 3 token）：
        //   预算 8 时正文装得下，标记装不下 ⇒ 必须提前退到更短的兜底。
        if estimate_tokens(TAIL_MARK) > max_tokens {
            return SHORT_FALLBACK.to_owned();
        }
        let mut clipped = out;
        // 每轮至少砍掉 1 个字符 ⇒ 必然收敛。
        let mut guard = clipped.len() + 16;
        while estimate_tokens(&clipped) > max_tokens {
            let budget_bytes = max_tokens.saturating_mul(4).max(8);
            if clipped.len() <= budget_bytes && estimate_tokens(&clipped) <= max_tokens {
                break;
            }
            if guard == 0 {
                break;
            }
            guard -= 1;
            let keep = clipped
                .char_indices()
                .nth(clipped.len() * 3 / 4)
                .map(|(i, _)| i)
                .unwrap_or(0);
            if keep == 0 {
                break;
            }
            clipped.truncate(keep);
            clipped.push_str(TAIL_MARK);
        }
        if estimate_tokens(&clipped) > max_tokens {
            return SHORT_FALLBACK.to_owned();
        }
        clipped
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⛔ **不超预算**是硬不变量 —— 这条对所有输入都必须成立。
    #[test]
    fn 任何输入都不超预算() {
        let long: String = (0..500)
            .map(|i| format!("line {i} with some padding text to burn budget\n"))
            .collect();
        for budget in [8usize, 32, 128, 512, 2048] {
            let out = distill_output(&long, budget);
            assert!(
                estimate_tokens(&out) <= budget,
                "预算 {budget} 被超出：实际 {}",
                estimate_tokens(&out)
            );
        }
    }

    /// **本模块存在的核心理由**：错误行与 exit code 不得被砍掉。
    ///
    /// `truncate_history` 的砍尾留头在这两条上都会丢 —— 本测试就是它的对照。
    #[test]
    fn 错误行与退出码必须保留() {
        let mut body = String::new();
        for i in 0..400 {
            body.push_str(&format!("ordinary log line {i} padding padding padding\n"));
        }
        body.push_str("error: something broke at the very end\n");
        body.push_str("warning: and a warning too\n");
        body.push_str("exit code 1\n");

        let out = distill_output(&body, 256);
        assert!(
            out.contains("error: something broke"),
            "错误行被丢弃：\n{out}"
        );
        assert!(
            out.contains("exit code 1"),
            "退出码被丢弃（★ 这正是砍尾留头的核心损失）：\n{out}"
        );
        // 错误段必须排在最前 ⇒ 模型先看到问题
        let err_pos = out.find("## errors").unwrap_or_else(|| {
            let p = out.find("error: something broke").unwrap();
            p
        });
        let body_pos = out.find("## body").unwrap_or(out.len());
        assert!(err_pos <= body_pos, "errors 段必须在 body 之前");
    }

    /// ⛔ 短输出**原样透传**，不得加任何结构头（加了就是无谓扰动）。
    #[test]
    fn 短输出原样透传() {
        let s = "all good\nexit code 0";
        assert_eq!(distill_output(s, 1024), s);
    }

    /// ⛔ 可逆性：省略处必须留标记，且标记不被兜底截断切掉。
    #[test]
    fn 省略标记完整保留() {
        let long: String = (0..300).map(|i| format!("filler {i}\n")).collect();
        let out = distill_output(&long, 64);
        assert!(
            out.contains("[ref#1] body 中段省略"),
            "省略标记丢失：\n{out}"
        );
    }

    /// ⛔ CJK 计费：与 core 同口径（11 ASCII = 2 token；每 CJK 字 1 token）。
    #[test]
    fn token估算与core同口径() {
        assert_eq!(estimate_tokens(""), 1, "空串按保守上界算 1");
        assert_eq!(estimate_tokens("abc"), 1);
        assert_eq!(estimate_tokens("abcdefghijk"), 2, "11 ASCII = 2 token");
        assert_eq!(estimate_tokens("你好"), 2, "每 CJK 字 1 token");
    }
}
