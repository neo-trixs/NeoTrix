//! NullNormalizer — 空值标准化器
//!
//! 标准化 LLM 输出中的 null/空值，确保下游处理一致性。
//! 支持多种空值模式检测和替换策略。

use std::collections::HashMap;

/// 空值模式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NullPattern {
    /// JSON null
    JsonNull,
    /// 空字符串
    EmptyString,
    /// "null" 字符串
    NullString,
    /// "undefined"
    UndefinedString,
    /// "N/A"
    NaString,
    /// None
    NoneValue,
    /// 自定义模式
    Custom(String),
}

/// 标准化策略
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizeStrategy {
    /// 替换为默认值
    ReplaceDefault,
    /// 移除该字段
    RemoveField,
    /// 跳过 (保留原值)
    Skip,
    /// 使用模板
    UseTemplate,
}

/// 标准化规则
#[derive(Debug, Clone)]
pub struct NormalizeRule {
    pub pattern: NullPattern,
    pub strategy: NormalizeStrategy,
    pub default_value: Option<String>,
    pub template: Option<String>,
}

/// 空值标准化器
pub struct NullNormalizer {
    /// 规则集
    rules: Vec<NormalizeRule>,
    /// 统计信息
    stats: NormalizeStats,
}

impl NullNormalizer {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            stats: NormalizeStats::default(),
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: NormalizeRule) {
        self.rules.push(rule);
    }

    /// 标准化字符串值
    pub fn normalize_string(&mut self, value: &str) -> String {
        self.stats.total_checks += 1;

        for rule in &self.rules {
            if self.matches_pattern(value, &rule.pattern) {
                self.stats.nulls_found += 1;
                return self.apply_strategy(value, &rule.strategy, &rule.default_value, &rule.template);
            }
        }

        value.to_string()
    }

    /// 标准化 JSON 值
    pub fn normalize_json(&mut self, json: &str) -> String {
        // 简化的 JSON 空值处理
        let mut result = json.to_string();

        // 处理 JSON null
        result = result.replace(": null", ": \"\"");
        result = result.replace(":null", ":\"\"");

        result
    }

    /// 匹配模式
    fn matches_pattern(&self, value: &str, pattern: &NullPattern) -> bool {
        match pattern {
            NullPattern::JsonNull => value == "null",
            NullPattern::EmptyString => value.is_empty(),
            NullPattern::NullString => value.to_lowercase() == "null",
            NullPattern::UndefinedString => value.to_lowercase() == "undefined",
            NullPattern::NaString => value.to_uppercase() == "N/A",
            NullPattern::NoneValue => value.to_lowercase() == "none",
            NullPattern::Custom(pattern) => value.contains(pattern),
        }
    }

    /// 应用策略
    fn apply_strategy(&self, value: &str, strategy: &NormalizeStrategy, default_value: &Option<String>, template: &Option<String>) -> String {
        match strategy {
            NormalizeStrategy::ReplaceDefault => {
                default_value.clone().unwrap_or_else(|| value.to_string())
            }
            NormalizeStrategy::RemoveField => {
                String::new() // 标记为删除
            }
            NormalizeStrategy::Skip => {
                value.to_string()
            }
            NormalizeStrategy::UseTemplate => {
                template.clone().unwrap_or_else(|| value.to_string())
            }
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> NormalizeStats {
        self.stats.clone()
    }
}

impl Default for NullNormalizer {
    fn default() -> Self {
        let mut normalizer = Self::new();
        normalizer.add_rule(NormalizeRule {
            pattern: NullPattern::JsonNull,
            strategy: NormalizeStrategy::ReplaceDefault,
            default_value: Some(String::new()),
            template: None,
        });
        normalizer.add_rule(NormalizeRule {
            pattern: NullPattern::EmptyString,
            strategy: NormalizeStrategy::ReplaceDefault,
            default_value: Some(String::new()),
            template: None,
        });
        normalizer.add_rule(NormalizeRule {
            pattern: NullPattern::NullString,
            strategy: NormalizeStrategy::ReplaceDefault,
            default_value: Some(String::new()),
            template: None,
        });
        normalizer
    }
}

/// 标准化统计
#[derive(Debug, Clone, Default)]
pub struct NormalizeStats {
    pub total_checks: u32,
    pub nulls_found: u32,
}

impl NormalizeStats {
    pub fn null_rate(&self) -> f64 {
        if self.total_checks == 0 {
            return 0.0;
        }
        self.nulls_found as f64 / self.total_checks as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_null() {
        let mut normalizer = NullNormalizer::default();
        assert_eq!(normalizer.normalize_string("null"), "");
    }

    #[test]
    fn test_normalize_empty() {
        let mut normalizer = NullNormalizer::default();
        assert_eq!(normalizer.normalize_string(""), "");
    }

    #[test]
    fn test_normalize_valid() {
        let mut normalizer = NullNormalizer::default();
        assert_eq!(normalizer.normalize_string("hello"), "hello");
    }
}
