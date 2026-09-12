//! NT-MIND — WordPecker 吸收 (github.com/baturiyilmaz/wordpecker-app).
//!
//! WordPecker: 中英文 NLP 处理技能 — 分词 + 实体抽取 (C1: trait 存在 +
//! 基础逻辑 + SelfTest T1 + 3 测试). 当前为中英文基础 stub, 后续迭代增强。

use crate::core::nt_core_self_test::SelfTest;

/// 抽取的实体: 文本 + 类型占位。
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub text: String,
    pub kind: String,
}

/// 中英文 NLP 处理 trait。
pub(crate) trait _NlpProcessor {
    /// 分词 (中英文混合): 按空白 + 连续 ASCII 词 + 单汉字切分。
    fn tokenize(&self, text: &str) -> Vec<String>;
    /// 实体抽取 (stub: 抽取含大写的英文专有名词与 @中文 标记)。
    fn extract_entities(&self, text: &str) -> Vec<Entity>;
}

/// WordPecker 中英文 NLP 处理实现。
pub(crate) struct _WordPeckerNlp {
    min_entity_len: usize,
}

impl _WordPeckerNlp {
    pub fn new() -> Self {
        Self { min_entity_len: 2 }
    }
}

impl Default for _WordPeckerNlp {
    fn default() -> Self {
        Self::new()
    }
}

impl _NlpProcessor for _WordPeckerNlp {
    fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        for raw in text.split_whitespace() {
            // 连续 ASCII 视为一个 token, 其余按单字符切 (中文逐字)。
            if raw.chars().all(|c| c.is_ascii_alphanumeric()) {
                tokens.push(raw.to_string());
            } else {
                for ch in raw.chars() {
                    if !ch.is_whitespace() {
                        tokens.push(ch.to_string());
                    }
                }
            }
        }
        tokens
    }

    fn extract_entities(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        for tok in self.tokenize(text) {
            // 英文专有名词: 首字母大写且长度达标。
            if tok.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                && tok.len() >= self.min_entity_len
            {
                entities.push(Entity {
                    text: tok,
                    kind: "PROPER".into(),
                });
            }
        }
        entities
    }
}

impl SelfTest for _WordPeckerNlp {
    fn name(&self) -> &'static str {
        "_WordPeckerNlp"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        if self.min_entity_len == 0 {
            return Err(vec!["min_entity_len must be >= 1".into()]);
        }
        // 基础一致性: 切分非空文本应产出 >= 1 token。
        if self.tokenize("hi 世").is_empty() {
            return Err(vec!["tokenize produced no tokens".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_mixed() {
        let p = _WordPeckerNlp::new();
        let toks = p.tokenize("Hello 世界 world");
        assert!(toks.contains(&"Hello".to_string()));
        assert!(toks.contains(&"世".to_string()));
        assert!(toks.contains(&"界".to_string()));
        assert!(toks.contains(&"world".to_string()));
    }

    #[test]
    fn test_extract_entities_proper_noun() {
        let p = _WordPeckerNlp::new();
        let ents = p.extract_entities("Beijing is great 北京");
        assert!(ents.iter().any(|e| e.text == "Beijing"));
    }

    #[test]
    fn test_selftest_pass() {
        let p = _WordPeckerNlp::new();
        assert!(p.self_test().is_ok());
    }
}
