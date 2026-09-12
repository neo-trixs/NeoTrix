//! _NullNormalizer — 空值标准化器
//!
//! 标准化 LLM 输出中的 null/空值，确保下游处理一致性。
//! 支持多种空值模式检测和替换策略。

/// 空值模式
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum _NullPattern {
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
pub(crate) enum _NormalizeStrategy {
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
pub(crate) struct _NormalizeRule {
    pub pattern: _NullPattern,
    pub strategy: _NormalizeStrategy,
    pub default_value: Option<String>,
    pub template: Option<String>,
}

/// 空值标准化器
pub(crate) struct _NullNormalizer {
    /// 规则集
    rules: Vec<_NormalizeRule>,
    /// 统计信息
    stats: _NormalizeStats,
}

impl _NullNormalizer {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            stats: _NormalizeStats::default(),
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: _NormalizeRule) {
        self.rules.push(rule);
    }

    /// 标准化字符串值
    pub(crate) fn _normalize_string(&mut self, value: &str) -> String {
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
    pub(crate) fn _normalize_json(&mut self, json: &str) -> String {
        // 简化的 JSON 空值处理
        let mut result = json.to_string();

        // 处理 JSON null
        result = result.replace(": null", ": \"\"");
        result = result.replace(":null", ":\"\"");

        result
    }

    /// 匹配模式
    fn matches_pattern(&self, value: &str, pattern: &_NullPattern) -> bool {
        match pattern {
            _NullPattern::JsonNull => value == "null",
            _NullPattern::EmptyString => value.is_empty(),
            _NullPattern::NullString => value.to_lowercase() == "null",
            _NullPattern::UndefinedString => value.to_lowercase() == "undefined",
            _NullPattern::NaString => value.to_uppercase() == "N/A",
            _NullPattern::NoneValue => value.to_lowercase() == "none",
            _NullPattern::Custom(pattern) => value.contains(pattern),
        }
    }

    /// 应用策略
    fn apply_strategy(&self, value: &str, strategy: &_NormalizeStrategy, default_value: &Option<String>, template: &Option<String>) -> String {
        match strategy {
            _NormalizeStrategy::ReplaceDefault => {
                default_value.clone().unwrap_or_else(|| value.to_string())
            }
            _NormalizeStrategy::RemoveField => {
                String::new() // 标记为删除
            }
            _NormalizeStrategy::Skip => {
                value.to_string()
            }
            _NormalizeStrategy::UseTemplate => {
                template.clone().unwrap_or_else(|| value.to_string())
            }
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> _NormalizeStats {
        self.stats.clone()
    }
}

impl Default for _NullNormalizer {
    fn default() -> Self {
        let mut normalizer = Self::new();
        normalizer.add_rule(_NormalizeRule {
            pattern: _NullPattern::JsonNull,
            strategy: _NormalizeStrategy::ReplaceDefault,
            default_value: Some(String::new()),
            template: None,
        });
        normalizer.add_rule(_NormalizeRule {
            pattern: _NullPattern::EmptyString,
            strategy: _NormalizeStrategy::ReplaceDefault,
            default_value: Some(String::new()),
            template: None,
        });
        normalizer.add_rule(_NormalizeRule {
            pattern: _NullPattern::NullString,
            strategy: _NormalizeStrategy::ReplaceDefault,
            default_value: Some(String::new()),
            template: None,
        });
        normalizer
    }
}

/// 标准化统计
#[derive(Debug, Clone, Default)]
pub(crate) struct _NormalizeStats {
    pub total_checks: u32,
    pub nulls_found: u32,
}

impl _NormalizeStats {
    pub(crate) fn _null_rate(&self) -> f64 {
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
        let mut normalizer = _NullNormalizer::default();
        assert_eq!(normalizer._normalize_string("null"), "");
    }

    #[test]
    fn test_normalize_empty() {
        let mut normalizer = _NullNormalizer::default();
        assert_eq!(normalizer._normalize_string(""), "");
    }

    #[test]
    fn test_normalize_valid() {
        let mut normalizer = _NullNormalizer::default();
        assert_eq!(normalizer._normalize_string("hello"), "hello");
    }
}
