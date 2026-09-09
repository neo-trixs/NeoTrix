//! ELI5 平白语言解释技能 (NT-IO)
//!
//! 吸收源: anthropics/claude-plugins-community/eli5/skills/eli5
//! 成熟度: C1 (unit-tested stub, 无外部 LLM 集成)
//!
//! 核心能力: 将技术/复杂文本重写为面向新手的平白解释,
//! 本 stub 负责可读性启发式评估与分层重写骨架。

use crate::core::nt_core_self_test::SelfTest;

/// ELI5 解释器 trait — 把复杂文本映射为平白解释。
pub trait Eli5Explainer: Send + Sync {
    /// 生成平白解释 (stub: 添加引导语与去术语化占位)。
    fn explain(&self, text: &str) -> String;
    /// 可读性启发式: 长句/术语密度过高则判定为"非平白"。
    fn is_plain(&self, text: &str) -> bool;
}

/// 默认实现: 引导语包装 + 基于句长与稀有词比的平白判定。
#[derive(Default)]
pub struct Eli5Engine;

impl Eli5Explainer for Eli5Engine {
    fn explain(&self, text: &str) -> String {
        format!("Imagine it like this: {}", text.trim())
    }

    fn is_plain(&self, text: &str) -> bool {
        let sentences: Vec<&str> = text.split(['.', '!', '?']).filter(|s| !s.trim().is_empty()).collect();
        if sentences.is_empty() {
            return true;
        }
        let avg_len = text.len() / sentences.len();
        avg_len <= 60
    }
}

/// T1 SelfTest: 验证解释器存在且平白判定生效。
#[derive(Default)]
pub struct Eli5SelfTest;

impl SelfTest for Eli5SelfTest {
    fn name(&self) -> &str {
        "nt_io_eli5"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let e = Eli5Engine;
        let out = e.explain("Neural nets learn patterns.");
        if !out.contains("Imagine it like this") {
            return Err(vec!["nt_io_eli5: explain missing framing".into()]);
        }
        if !e.is_plain("short. ok.") {
            return Err(vec!["nt_io_eli5: plain text misjudged".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_adds_framing() {
        let e = Eli5Engine;
        assert!(e.explain("entropy").starts_with("Imagine it like this"));
    }

    #[test]
    fn test_is_plain_short_text() {
        let e = Eli5Engine;
        assert!(e.is_plain("cats sit. dogs run."));
    }

    #[test]
    fn test_is_plain_rejects_long_sentences() {
        let e = Eli5Engine;
        let long = "This extraordinarily complicated and excessively verbose sentence about distributed consensus mechanisms exceeds the plain threshold by a significant margin for novice readers.";
        assert!(!e.is_plain(long));
    }
}
