//! Unslop 写作去赘技能 (NT-IO)
//!
//! 吸收源: cursor/plugins/tree/main/pstack/skills/unslop
//! 成熟度: C1 (unit-tested stub, 无外部 LLM 集成)
//!
//! 核心能力: 检测并清理 AI 生成文本中的"slop"模式
//! (套话/空洞修饰/重复结构), 本 stub 负责基于规则的模式剔除与密度评估。

use crate::core::nt_core_self_test::SelfTest;

/// 已知 slop 短语 (套话/空洞修饰)。
const SLOP_PHRASES: &[&str] = &[
    "it is important to note that",
    "in today's fast-paced world",
    "leverage",
    "delve",
    "at the end of the day",
];

/// 去赘写作器 trait — 清理 slop 并评估密度。
pub trait UnslopWriter: Send + Sync {
    /// 清理文本中的已知 slop 短语 (大小写不敏感, 去除后折叠空格)。
    fn clean(&self, text: &str) -> String;
    /// 计算 slop 密度: 命中短语数 / 词数。
    fn slop_density(&self, text: &str) -> f64;
}

/// 默认实现: 线性替换 + 词频密度估算。
#[derive(Default)]
pub struct UnslopEngine;

impl UnslopWriter for UnslopEngine {
    fn clean(&self, text: &str) -> String {
        let mut out = text.to_string();
        for p in SLOP_PHRASES {
            let lower = out.to_lowercase();
            if let Some(idx) = lower.find(p) {
                let start = idx;
                let end = idx + p.len();
                out.replace_range(start..end, "");
            }
        }
        let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
        collapsed
    }

    fn slop_density(&self, text: &str) -> f64 {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }
        let lower = text.to_lowercase();
        let hits = SLOP_PHRASES.iter().filter(|p| lower.contains(*p)).count();
        hits as f64 / words.len() as f64
    }
}

/// T1 SelfTest: 验证去赘器存在且能移除 slop。
#[derive(Default)]
pub struct UnslopSelfTest;

impl SelfTest for UnslopSelfTest {
    fn name(&self) -> &str {
        "nt_io_unslop"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let e = UnslopEngine;
        let cleaned = e.clean("It is important to note that cats purr.");
        if cleaned.to_lowercase().contains("it is important to note that") {
            return Err(vec!["nt_io_unslop: slop phrase not removed".into()]);
        }
        if e.slop_density("Leverage delve at the end of the day.") <= 0.0 {
            return Err(vec!["nt_io_unslop: slop density zero on sloppy text".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_removes_slop() {
        let e = UnslopEngine;
        let c = e.clean("In today's fast-paced world we leverage tools.");
        assert!(!c.to_lowercase().contains("leverage"));
        assert!(!c.to_lowercase().contains("in today's fast-paced world"));
    }

    #[test]
    fn test_slop_density_positive() {
        let e = UnslopEngine;
        let d = e.slop_density("It is important to note that delve is used.");
        assert!(d > 0.0);
    }

    #[test]
    fn test_clean_preserves_clean_text() {
        let e = UnslopEngine;
        let t = "Cats purr when content.";
        assert_eq!(e.clean(t), t);
        assert_eq!(e.slop_density(t), 0.0);
    }
}
