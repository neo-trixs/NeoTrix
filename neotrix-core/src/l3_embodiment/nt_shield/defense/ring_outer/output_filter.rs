//! Output Filter - 输出过滤器
//!
//! 提取 output_sentinel 精髓
//! 设计原则：输出净化 + 敏感信息过滤

/// 过滤结果
#[derive(Debug, Clone)]
pub struct FilterResult {
    pub original: String,
    pub filtered: String,
    pub threats_removed: Vec<String>,
    pub modified: bool,
}

/// 输出过滤器
pub struct OutputFilter {
    /// 敏感信息模式
    sensitive_patterns: Vec<(String, f64)>, // (pattern, severity)
    /// 过滤策略
    filter_mode: FilterMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    /// 严格模式：移除所有匹配
    Strict,
    /// 标准模式：仅移除高严重度
    Standard,
    /// 宽松模式：仅记录不移除
    Lenient,
}

impl OutputFilter {
    pub fn new(mode: FilterMode) -> Self {
        Self {
            sensitive_patterns: vec![
                // API 密钥
                (r"(?i)api[_-]?key\s*[=:]\s*\S+".to_string(), 0.9),
                // 密码
                (r"(?i)password\s*[=:]\s*\S+".to_string(), 0.9),
                // Token
                (r"(?i)token\s*[=:]\s*\S+".to_string(), 0.8),
                // 私钥
                (r"(?i)private[_-]?key\s*[=:]\s*\S+".to_string(), 0.95),
                // 内部路径
                (r"/Users/[^/\s]+".to_string(), 0.6),
                (r"/home/[^/\s]+".to_string(), 0.6),
                // IP 地址
                (r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}".to_string(), 0.5),
            ],
            filter_mode: mode,
        }
    }

    /// 过滤输出
    pub fn filter(&self, output: &str) -> FilterResult {
        let mut filtered = output.to_string();
        let mut threats_removed = Vec::new();

        for (pattern, severity) in &self.sensitive_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                let matches: Vec<_> = re.find_iter(&filtered).map(|m| m.as_str().to_string()).collect();
                for mat in matches {
                    match self.filter_mode {
                        FilterMode::Strict => {
                            filtered = filtered.replace(&mat, "[REDACTED]");
                            threats_removed.push(format!("Removed: {} (severity: {:.1})", mat, severity));
                        }
                        FilterMode::Standard if *severity > 0.7 => {
                            filtered = filtered.replace(&mat, "[REDACTED]");
                            threats_removed.push(format!("Removed: {} (severity: {:.1})", mat, severity));
                        }
                        FilterMode::Lenient => {
                            threats_removed.push(format!("Flagged: {} (severity: {:.1})", mat, severity));
                        }
                        _ => {}
                    }
                }
            }
        }

        let is_modified = output != filtered.as_str();

        FilterResult {
            original: output.to_string(),
            filtered,
            modified: is_modified,
            threats_removed,
        }
    }
}

impl Default for OutputFilter {
    fn default() -> Self {
        Self::new(FilterMode::Standard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_api_key() {
        let filter = OutputFilter::new(FilterMode::Strict);
        let result = filter.filter("api_key=sk_1234567890");
        assert!(result.modified);
        assert!(result.filtered.contains("[REDACTED]"));
    }

    #[test]
    fn test_filter_password() {
        let filter = OutputFilter::new(FilterMode::Strict);
        let result = filter.filter("password=secret123");
        assert!(result.modified);
    }

    #[test]
    fn test_filter_clean() {
        let filter = OutputFilter::new(FilterMode::Strict);
        let result = filter.filter("Hello world!");
        assert!(!result.modified);
    }
}
