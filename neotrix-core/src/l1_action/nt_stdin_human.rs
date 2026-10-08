//! # nt_stdin_human — 终端里的人
//!
//! `NtHumanChannel` 的 stdin 实现：对话窗口打到终端，人直接回车回复，
//! 回复进内需循环回灌重熔。行协议（打在窗口下方当提示）：
//!
//! ```text
//! <id>: <文字>   对某需求单给答案（批准，0.95 权重参与重熔）
//! <id>! <文字>   对某需求单给答案但驳回（0.6 权重当少数派）
//! ok <id>        批准该需求单（无新文字，直接关闭）
//! no <id>        驳回该需求单（无新文字，不关闭，等下一轮）
//! <纯文字>       总体意见（批准，参与重熔）
//! .              结束输入；直接 Ctrl-D / 空输入 = 沉默 → Stalled 挂起
//! # 开头         注释，忽略
//! ```
//!
//! # Safety
//! - 只读 stdin，无 unsafe (R-P1)；生产代码无 `unwrap/expect/panic`。

use crate::l1_action::nt_action_facade::{NtDemand, NtHumanChannel, NtHumanReply};
use std::io::BufRead;

/// 终端人类通道。
pub struct NtStdinHuman {
    max_lines: usize,
}

/// 归一化需求单 id：剥掉**外层成对方括号**，空则 `None`。
///
/// ## 2026-10-08 实测缺陷（本函数存在的理由）
///
/// 上窗显示那一行是 `format!("  [{}] {}：{}", d.id, d.kind.label(), d.text)`
/// （`l5_cognition/nt_crystal_core/nt_crystal_dialogue.rs`）⇒ **屏幕上带方括号**。
/// 而解析层原样把用户输入的 `[id]` 存进 `demand_id`，与需求单真实的**裸 id**
/// 比对不上 ⇒ **批准永不生效** ⇒ 同一道复核门在下一轮原样重上。
///
/// 实测（`ntcode --line`，无 TTY 管道喂回复）：
/// - `ok [review-fc47da46]` ⇒ `终态：Stalled（2 轮）`、exit **2**（照抄屏幕，死锁在复核门）
/// - `ok review-fc47da46`   ⇒ `终态：Converged（2 轮）`、exit **0**
///
/// ⇒ 用户**照抄自己看到的东西**是不成立的，这是可复制性缺陷而非用法错误。
/// ⛔ 报错拒绝不是好选择：那会把「屏幕上的合法字符串」判成非法输入，
/// 而显示层才是加括号的那一方（真实 id 永不含方括号）。
/// ⇒ 故做成**容忍**：剥掉成对外层括号，裸 id 走原路径，两种输入都成立。
fn normalize_demand_id(raw: &str) -> Option<String> {
    let t = raw.trim();
    let t = t
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .unwrap_or(t)
        .trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

impl NtStdinHuman {
    pub fn new() -> Self {
        Self { max_lines: 50 }
    }

    pub fn with_max_lines(mut self, n: usize) -> Self {
        self.max_lines = n.max(1);
        self
    }

    /// 纯函数：行协议 → replies（单独可测）。
    pub fn parse_lines(lines: &[String]) -> Vec<NtHumanReply> {
        let mut out = Vec::new();
        for line in lines {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') {
                continue;
            }
            if t == "." {
                break;
            }
            if let Some(rest) = t.strip_prefix("ok ") {
                out.push(NtHumanReply {
                    demand_id: normalize_demand_id(rest),
                    text: String::new(),
                    approved: true,
                });
                continue;
            }
            if let Some(rest) = t.strip_prefix("no ") {
                out.push(NtHumanReply {
                    demand_id: normalize_demand_id(rest),
                    text: String::new(),
                    approved: false,
                });
                continue;
            }
            // "<id>! <文字>"：驳回但给答案
            if let Some(bang) = t.find('!') {
                let (id, rest) = t.split_at(bang);
                let id = id.trim();
                let text = rest[1..].trim();
                if !id.is_empty() && !id.contains(char::is_whitespace) && !text.is_empty() {
                    out.push(NtHumanReply {
                        demand_id: normalize_demand_id(id),
                        text: text.to_string(),
                        approved: false,
                    });
                    continue;
                }
            }
            // "<id>: <文字>"：批准并给答案
            if let Some(colon) = t.find(':') {
                let (id, rest) = t.split_at(colon);
                let id = id.trim();
                let text = rest[1..].trim();
                if !id.is_empty() && !id.contains(char::is_whitespace) && !text.is_empty() {
                    out.push(NtHumanReply {
                        demand_id: normalize_demand_id(id),
                        text: text.to_string(),
                        approved: true,
                    });
                    continue;
                }
                if !id.is_empty() && !id.contains(char::is_whitespace) && text.is_empty() {
                    // "<id>:" 空文 = 批准关闭
                    out.push(NtHumanReply {
                        demand_id: normalize_demand_id(id),
                        text: String::new(),
                        approved: true,
                    });
                    continue;
                }
            }
            // 兜底：纯文字总体意见
            out.push(NtHumanReply {
                demand_id: None,
                text: t.to_string(),
                approved: true,
            });
        }
        out
    }

    fn help() -> &'static str {
        "回复格式：<id>: 文字=批准给答案；<id>! 文字=驳回给答案；ok <id>=批准关闭；no <id>=驳回；纯文字=总体意见；.=结束；空输入=沉默挂起"
    }
}

impl Default for NtStdinHuman {
    fn default() -> Self {
        Self::new()
    }
}

impl NtHumanChannel for NtStdinHuman {
    fn prompt(&self, window: &str, _demands: &[NtDemand]) -> Vec<NtHumanReply> {
        println!("{window}");
        println!("{}", Self::help());
        println!("你的回复（. 结束，Ctrl-D 直接挂起）：");
        let stdin = std::io::stdin();
        let mut lines = Vec::new();
        for line in stdin.lock().lines().take(self.max_lines) {
            match line {
                Ok(l) => {
                    if l.trim() == "." {
                        break;
                    }
                    lines.push(l);
                }
                Err(_) => break,
            }
        }
        Self::parse_lines(&lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_parse_answer_and_approve_forms() {
        let rs = NtStdinHuman::parse_lines(&lines(&[
            "retry-a1b2c3d4: 已经手动执行完毕结果符合预期",
            "ok review-e5f6a7b8",
            "no confirm-12345678",
        ]));
        assert_eq!(rs.len(), 3);
        assert_eq!(rs[0].demand_id.as_deref(), Some("retry-a1b2c3d4"));
        assert!(rs[0].approved);
        assert!(rs[0].text.contains("手动执行"));
        assert_eq!(rs[1].demand_id.as_deref(), Some("review-e5f6a7b8"));
        assert!(rs[1].text.is_empty());
        assert!(!rs[2].approved);
    }

    #[test]
    fn test_parse_reject_with_text() {
        let rs = NtStdinHuman::parse_lines(&lines(&["adjudicate-1! 结论反了应该是方案乙才对"]));
        assert_eq!(rs.len(), 1);
        assert!(!rs[0].approved);
        assert!(rs[0].text.contains("方案乙"));
    }

    #[test]
    fn test_parse_plain_and_dot_and_comment() {
        let rs = NtStdinHuman::parse_lines(&lines(&[
            "# 忽略我",
            "总体方向没问题继续推进工作",
            ".",
            "这行不会被读到因为点号截断",
        ]));
        assert_eq!(rs.len(), 1);
        assert_eq!(rs[0].demand_id, None);
        assert!(rs[0].approved);
    }

    #[test]
    fn test_parse_empty_is_silence() {
        let rs = NtStdinHuman::parse_lines(&lines(&["", "   "]));
        assert!(rs.is_empty());
    }

    /// 2026-10-08 补：**照抄屏幕上那串（含方括号）必须也能批准成功**。
    ///
    /// 为什么可能失败：显示层 `format!("  [{}] …", d.id)` 加了方括号，
    /// 解析层若原样存 `[id]` ⇒ 与裸 id 比对不上 ⇒ 批准静默失效 ⇒
    /// `ntcode --line` 永远 `Stalled`。本测试钉住「两种输入归一到同一个 id」。
    #[test]
    fn bracketed_id_from_display_normalizes_to_bare_id() {
        let bare = NtStdinHuman::parse_lines(&lines(&["ok review-e5f6a7b8"]));
        let bracketed = NtStdinHuman::parse_lines(&lines(&["ok [review-e5f6a7b8]"]));
        assert_eq!(
            bracketed[0].demand_id, bare[0].demand_id,
            "照抄屏幕（含方括号）必须归一到与裸 id 相同的 demand_id"
        );
        assert_eq!(bracketed[0].demand_id.as_deref(), Some("review-e5f6a7b8"));

        // `<id>: 文字` 与 `<id>! 文字` 两条 arm 同样要容忍方括号。
        let colon = NtStdinHuman::parse_lines(&lines(&["[review-e5f6a7b8]: 同意"]));
        assert_eq!(colon[0].demand_id.as_deref(), Some("review-e5f6a7b8"));
        let bang = NtStdinHuman::parse_lines(&lines(&["[review-e5f6a7b8]! 不同意"]));
        assert_eq!(bang[0].demand_id.as_deref(), Some("review-e5f6a7b8"));

        // `no` 那条 arm 也一样。
        let no = NtStdinHuman::parse_lines(&lines(&["no [review-e5f6a7b8]"]));
        assert_eq!(no[0].demand_id.as_deref(), Some("review-e5f6a7b8"));

        // 负例：只有**空**才归 None；`[]` 剥完是空 ⇒ None（不是 Some("[]")）。
        assert_eq!(
            NtStdinHuman::parse_lines(&lines(&["ok []"]))[0].demand_id,
            None,
            "空括号剥完应为空 id ⇒ None"
        );
        // 负例：不成对的单边括号**不剥**（宁可原样，也不要猜用户的意图）。
        assert_eq!(
            NtStdinHuman::parse_lines(&lines(&["ok [review-e5f6a7b8"]))[0]
                .demand_id
                .as_deref(),
            Some("[review-e5f6a7b8")
        );
    }
}
