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
                let id = rest.trim();
                out.push(NtHumanReply {
                    demand_id: if id.is_empty() { None } else { Some(id.to_string()) },
                    text: String::new(),
                    approved: true,
                });
                continue;
            }
            if let Some(rest) = t.strip_prefix("no ") {
                let id = rest.trim();
                out.push(NtHumanReply {
                    demand_id: if id.is_empty() { None } else { Some(id.to_string()) },
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
                        demand_id: Some(id.to_string()),
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
                        demand_id: Some(id.to_string()),
                        text: text.to_string(),
                        approved: true,
                    });
                    continue;
                }
                if !id.is_empty() && !id.contains(char::is_whitespace) && text.is_empty() {
                    // "<id>:" 空文 = 批准关闭
                    out.push(NtHumanReply {
                        demand_id: Some(id.to_string()),
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
}
