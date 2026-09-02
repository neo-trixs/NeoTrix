//! ErrorClassifier — 错误分类器
//!
//! 分类错误类型 (Transient/Permanent/ContentPolicy)，并根据类型决定重试策略。
//! 集成指数退避 + 抖动 + 故障转移逻辑。

use std::time::{Duration, Instant};

/// 错误类型分类
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// 瞬态错误: 可重试 (429, 5xx, timeout)
    Transient,
    /// 永久错误: 不可重试 (400, 401, 403)
    Permanent,
    /// 内容策略错误: 安全拦截 (content_policy)
    ContentPolicy,
    /// 未知错误: 保守策略
    Unknown,
}

/// 单次错误信息
#[derive(Debug, Clone)]
pub struct ErrorRecord {
    pub category: ErrorCategory,
    pub message: String,
    pub timestamp: Instant,
    pub attempt: u32,
}

/// 错误分类器
pub struct ErrorClassifier {
    /// 分类历史
    history: Vec<ErrorRecord>,
    /// 最大历史记录数
    max_history: usize,
    /// 瞬态错误阈值 (超过此值触发故障转移)
    transient_threshold: u32,
}

impl ErrorClassifier {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            max_history: 100,
            transient_threshold: 3,
        }
    }

    /// 分类错误
    pub fn classify(&mut self, error: &str, status_code: Option<u16>) -> ErrorCategory {
        let category = if let Some(code) = status_code {
            match code {
                429 => ErrorCategory::Transient,
                500..=599 => ErrorCategory::Transient,
                400 | 401 | 403 => ErrorCategory::Permanent,
                _ => ErrorCategory::Unknown,
            }
        } else {
            if error.contains("timeout") || error.contains("timed out") {
                ErrorCategory::Transient
            } else if error.contains("rate limit") || error.contains("429") {
                ErrorCategory::Transient
            } else if error.contains("invalid") || error.contains("unauthorized") {
                ErrorCategory::Permanent
            } else {
                ErrorCategory::Unknown
            }
        };

        self.history.push(ErrorRecord {
            category: category.clone(),
            message: error.to_string(),
            timestamp: Instant::now(),
            attempt: self.history.len() as u32 + 1,
        });

        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        category
    }

    /// 检查是否应重试
    pub fn should_retry(&self, category: &ErrorCategory, current_attempt: u32, max_retries: u32) -> bool {
        if current_attempt >= max_retries {
            return false;
        }

        match category {
            ErrorCategory::Transient => true,
            ErrorCategory::Permanent => false,
            ErrorCategory::ContentPolicy => false,
            ErrorCategory::Unknown => current_attempt < 2, // 未知错误最多重试1次
        }
    }

    /// 计算重试延迟 (指数退避 + 抖动)
    pub fn retry_delay(&self, attempt: u32, base_delay: Duration) -> Duration {
        let exponential = base_delay * 2u32.pow(attempt);
        let jitter = Duration::from_millis(rand::Rng::gen_range(&mut rand::thread_rng(), 0..100));
        exponential + jitter
    }

    /// 检查是否应触发故障转移
    pub fn should_fallback(&self, provider: &str) -> bool {
        let recent_errors: Vec<_> = self.history.iter()
            .filter(|e| e.timestamp.elapsed() < Duration::from_secs(60))
            .collect();

        let transient_count = recent_errors.iter()
            .filter(|e| e.category == ErrorCategory::Transient)
            .count();

        transient_count as u32 >= self.transient_threshold
    }

    /// 获取错误统计
    pub fn stats(&self) -> ErrorStats {
        let mut transient = 0;
        let mut permanent = 0;
        let mut content_policy = 0;
        let mut unknown = 0;

        for record in &self.history {
            match record.category {
                ErrorCategory::Transient => transient += 1,
                ErrorCategory::Permanent => permanent += 1,
                ErrorCategory::ContentPolicy => content_policy += 1,
                ErrorCategory::Unknown => unknown += 1,
            }
        }

        ErrorStats {
            total: self.history.len() as u32,
            transient,
            permanent,
            content_policy,
            unknown,
        }
    }
}

impl Default for ErrorClassifier {
    fn default() -> Self {
        Self::new()
    }
}

/// 错误统计
#[derive(Debug, Clone)]
pub struct ErrorStats {
    pub total: u32,
    pub transient: u32,
    pub permanent: u32,
    pub content_policy: u32,
    pub unknown: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_transient() {
        let mut classifier = ErrorClassifier::new();
        let category = classifier.classify("rate limit exceeded", Some(429));
        assert_eq!(category, ErrorCategory::Transient);
    }

    #[test]
    fn test_classify_permanent() {
        let mut classifier = ErrorClassifier::new();
        let category = classifier.classify("invalid api key", Some(401));
        assert_eq!(category, ErrorCategory::Permanent);
    }

    #[test]
    fn test_should_retry() {
        let classifier = ErrorClassifier::new();
        assert!(classifier.should_retry(&ErrorCategory::Transient, 1, 3));
        assert!(!classifier.should_retry(&ErrorCategory::Permanent, 1, 3));
    }
}
