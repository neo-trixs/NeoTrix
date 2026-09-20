//! T2.7: 输出压缩 + 回忆模块 (RTK T1)
//!
//! 4 策略压缩管线: filter → group → truncate → dedup
//! 支持完整输出回溯检索

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Compression Config
// ============================================================================

/// 压缩配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// 最大字符数
    pub max_chars: usize,
    /// 过滤模式 (正则)
    pub filter_patterns: Vec<String>,
    /// 去重阈值 (0.0-1.0, Jaccard 相似度)
    pub dedup_threshold: f64,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            max_chars: 4096,
            filter_patterns: vec![],
            dedup_threshold: 0.8,
        }
    }
}

// ============================================================================
// OutputFilter Trait
// ============================================================================

/// 输出压缩 trait — 4 策略组合
pub trait OutputFilter {
    /// 过滤: 移除无关行
    fn filter(&self, output: &str) -> String;

    /// 分组: 相似行合并
    fn group(&self, output: &str) -> String;

    /// 截断: 限制最大长度
    fn truncate(&self, output: &str, max_chars: usize) -> String;

    /// 去重: 移除重复行
    fn dedup(&self, output: &str) -> String;

    /// 组合: 按顺序执行所有策略
    fn compress(&self, output: &str, config: &CompressionConfig) -> String {
        let filtered = self.filter(output);
        let grouped = self.group(&filtered);
        let truncated = self.truncate(&grouped, config.max_chars);
        self.dedup(&truncated)
    }
}

// ============================================================================
// DefaultOutputFilter
// ============================================================================

/// 默认输出过滤器实现
pub struct DefaultOutputFilter;

impl OutputFilter for DefaultOutputFilter {
    fn filter(&self, output: &str) -> String {
        output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter(|line| {
                let lower = line.to_lowercase();
                !lower.starts_with("warning:")
                    && !lower.starts_with("note:")
                    && !lower.starts_with("debug:")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn group(&self, output: &str) -> String {
        let mut groups: Vec<(String, usize)> = Vec::new();
        for line in output.lines() {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(entry) = groups.iter_mut().find(|(key, _)| similarity(key, &trimmed) > 0.7)
            {
                entry.1 += 1;
            } else {
                groups.push((trimmed, 1));
            }
        }
        groups
            .into_iter()
            .map(|(line, count)| {
                if count > 1 {
                    format!("{} (x{})", line, count)
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn truncate(&self, output: &str, max_chars: usize) -> String {
        if output.len() <= max_chars {
            output.to_string()
        } else {
            let truncated: String = output.chars().take(max_chars - 20).collect();
            format!("{}\n[... truncated]", truncated)
        }
    }

    fn dedup(&self, output: &str) -> String {
        let mut seen = std::collections::HashSet::new();
        output
            .lines()
            .filter(|line| seen.insert(line.trim().to_string()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Jaccard similarity for short strings (character bigrams)
fn similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let bigrams_a = char_bigrams(a);
    let bigrams_b = char_bigrams(b);
    let intersection = bigrams_a.intersection(&bigrams_b).count();
    let union = bigrams_a.union(&bigrams_b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

fn char_bigrams(s: &str) -> std::collections::HashSet<(char, char)> {
    let chars: Vec<char> = s.chars().collect();
    chars.windows(2).map(|w| (w[0], w[1])).collect()
}

// ============================================================================
// CompressedOutput
// ============================================================================

/// 压缩后的输出结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedOutput {
    /// 压缩后文本
    pub text: String,
    /// 原始大小 (bytes)
    pub original_size: usize,
    /// 压缩后大小 (bytes)
    pub compressed_size: usize,
    /// 使用的策略列表
    pub strategies_applied: Vec<String>,
}

impl CompressedOutput {
    /// 压缩比 (0.0-1.0, 越小压缩越多)
    pub fn ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 1.0;
        }
        self.compressed_size as f64 / self.original_size as f64
    }
}

// ============================================================================
// OutputRecall
// ============================================================================

/// 回忆机制: 完整输出检索 (内存存储, 生产环境可替换为 SQLite)
pub struct OutputRecall {
    /// command → full_output 映射
    store: HashMap<String, String>,
}

impl OutputRecall {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    /// 保存完整输出 (压缩前)
    pub fn save(&mut self, command: &str, full_output: &str) {
        self.store.insert(command.to_string(), full_output.to_string());
    }

    /// 回忆完整输出
    pub fn recall(&self, command: &str) -> Option<&str> {
        self.store.get(command).map(|s| s.as_str())
    }

    /// 已存储条目数
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

impl Default for OutputRecall {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_basic() {
        let filter = DefaultOutputFilter;
        let config = CompressionConfig::default();
        let input = "line1\nline2\nline1\n\n\nline3\nWARNING: skip this";
        let result = filter.compress(input, &config);
        assert!(result.contains("line1"));
        assert!(!result.contains("WARNING"));
        assert!(!result.contains("skip this"));
    }

    #[test]
    fn test_truncate() {
        let filter = DefaultOutputFilter;
        let long = "a".repeat(1000);
        let result = filter.truncate(&long, 100);
        assert!(result.len() <= 120);
        assert!(result.contains("truncated"));
    }

    #[test]
    fn test_dedup() {
        let filter = DefaultOutputFilter;
        let input = "foo\nbar\nfoo\nbaz\nbar";
        let result = filter.dedup(input);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_output_recall() {
        let mut recall = OutputRecall::new();
        recall.save("ls -la", "total 48");
        assert_eq!(recall.recall("ls -la"), Some("total 48"));
        assert_eq!(recall.recall("unknown"), None);
    }

    #[test]
    fn test_compressed_output_ratio() {
        let co = CompressedOutput {
            text: "short".to_string(),
            original_size: 1000,
            compressed_size: 100,
            strategies_applied: vec!["filter".into()],
        };
        assert!((co.ratio() - 0.1).abs() < f64::EPSILON);
    }
}
