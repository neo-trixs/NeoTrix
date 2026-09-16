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

/// 中英文 NLP 处理实现。
pub struct _WordPeckerNlp {
    min_entity_len: usize,
}

impl _WordPeckerNlp {
    pub fn new() -> Self {
        Self { min_entity_len: 2 }
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        for raw in text.split_whitespace() {
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

    pub fn extract_entities(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        for tok in self.tokenize(text) {
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

impl Default for _WordPeckerNlp {
    fn default() -> Self {
        Self::new()
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
        // ALWAYS-PASS: self_test() is a C0 stub that always returns Ok — it
        // validates the type exists, not real NLP behavior. This test documents
        // the SelfTest contract but does NOT validate real NLP quality.
        // TODO(R-P79): Replace with integration test that validates real NLP behavior:
        //   - tokenize() handles edge cases (empty string, unicode, emoji)
        //   - extract_entities() finds real proper nouns in mixed-language text
        //   - self_test() returns Err when NLP pipeline is not wired to real models
        let p = _WordPeckerNlp::new();
        assert!(p.self_test().is_ok());
    }
}
