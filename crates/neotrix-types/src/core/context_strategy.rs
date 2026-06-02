use super::file_parser::{FileFormat, FileParseResult};

/// 内容尺寸分级的上下文策略
///
/// 根据 2026 RAG 基准测试：
/// - Recursive 512-token splitting (10-20% overlap) = 69% accuracy (vs semantic 54%)
/// - Metadata enrichment boosts QA accuracy 50%→75%
/// - Token:char ratio ≈ 1:4（英文）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextStrategy {
    /// <50KB: 完整内容
    FullStuff,
    /// 50KB~500KB: 头部 + 索引
    HeadIndex,
    /// >=500KB: 混合（头部摘要 + 分段索引 + 尾部）
    Hybrid,
}

#[derive(Debug, Clone)]
pub struct ContextStrategyResult {
    pub content: String,
    pub strategy: ContextStrategy,
    pub original_size: usize,
    pub truncated: bool,
    pub token_estimate: usize,
    pub format: FileFormat,
    pub sections: Vec<String>,
}

impl ContextStrategy {
    pub const SMALL_THRESHOLD: usize = 50_000;
    pub const MEDIUM_THRESHOLD: usize = 500_000;
    pub const HEAD_SIZE: usize = 50_000;
    pub const TAIL_SIZE: usize = 20_000;
    pub const SECTION_LINE_LIMIT: usize = 200;

    pub fn select_strategy(size_bytes: usize) -> ContextStrategy {
        if size_bytes < Self::SMALL_THRESHOLD {
            ContextStrategy::FullStuff
        } else if size_bytes < Self::MEDIUM_THRESHOLD {
            ContextStrategy::HeadIndex
        } else {
            ContextStrategy::Hybrid
        }
    }

    pub fn estimate_tokens(text: &str) -> usize {
        // 英文估算: 1 token ≈ 4 chars
        // 中文估算: 1 token ≈ 2 chars
        let char_count = text.chars().count();
        let ascii_count = text.chars().filter(|c| c.is_ascii()).count();
        let non_ascii = char_count.saturating_sub(ascii_count);
        ascii_count / 4 + non_ascii / 2 + 1
    }

    pub fn extract_sections(text: &str) -> Vec<String> {
        let lines: Vec<&str> = text.lines().collect();
        let mut sections = Vec::new();
        let mut current = String::new();
        let mut line_count = 0;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.len() > 2 && line_count < Self::SECTION_LINE_LIMIT {
                if trimmed.ends_with(':')
                    || trimmed.starts_with("# ")
                    || trimmed.starts_with("## ")
                    || trimmed.starts_with("### ")
                    || trimmed.chars().all(|c| c.is_uppercase() || c.is_whitespace() || c.is_ascii_punctuation())
                {
                    if !current.is_empty() {
                        sections.push(current);
                        current = String::new();
                    }
                }
                if current.len() < 200 {
                    if !current.is_empty() {
                        current.push(' ');
                    }
                    current.push_str(trimmed);
                    line_count += 1;
                }
            }
        }
        if !current.is_empty() {
            sections.push(current);
        }
        sections
    }

    fn generate_index(sections: &[String]) -> String {
        if sections.is_empty() {
            return String::new();
        }
        let mut index = String::from("--- 内容索引 ---\n");
        for (i, sec) in sections.iter().enumerate() {
            let preview: String = sec.chars().take(80).collect();
            if !preview.is_empty() {
                index.push_str(&format!("  [{:3}] {}\n", i, preview));
            }
        }
        index
    }

    fn apply_full(text: &str, format: &FileFormat) -> ContextStrategyResult {
        let sections = Self::extract_sections(text);
        ContextStrategyResult {
            content: text.to_string(),
            strategy: ContextStrategy::FullStuff,
            original_size: text.len(),
            truncated: false,
            token_estimate: Self::estimate_tokens(text),
            format: format.clone(),
            sections,
        }
    }

    fn apply_head_index(text: &str, format: &FileFormat) -> ContextStrategyResult {
        let sections = Self::extract_sections(text);
        let head: String = text.chars().take(Self::HEAD_SIZE).collect();
        let mut content = String::new();

        content.push_str("--- 文件头部 (前50KB) ---\n");
        content.push_str(&head);
        if text.len() > Self::HEAD_SIZE {
            content.push_str("\n... (内容截断) ...\n\n");
        }
        content.push_str(&Self::generate_index(&sections));
        let tail_start = text.len().saturating_sub(Self::TAIL_SIZE);
        let tail_chars: String = text.chars().skip(tail_start).take(Self::TAIL_SIZE).collect();
        if !tail_chars.trim().is_empty() {
            content.push_str("\n--- 文件尾部 (后20KB) ---\n");
            content.push_str(&tail_chars);
        }

        let token_estimate = Self::estimate_tokens(&content);
        ContextStrategyResult {
            content,
            strategy: ContextStrategy::HeadIndex,
            original_size: text.len(),
            truncated: text.len() > Self::HEAD_SIZE + Self::TAIL_SIZE,
            token_estimate,
            format: format.clone(),
            sections,
        }
    }

    fn apply_hybrid(text: &str, format: &FileFormat) -> ContextStrategyResult {
        let sections = Self::extract_sections(text);
        let head: String = text.chars().take(Self::HEAD_SIZE).collect();
        let mut content = String::new();

        content.push_str("--- 文件头部 (前50KB) ---\n");
        content.push_str(&head);
        content.push_str("\n... (大型文件，内容已截断) ...\n\n");
        content.push_str(&Self::generate_index(&sections));

        let section_count = sections.len();
        let section_detail_lines: usize = sections.iter().map(|s| s.len()).sum();
        content.push_str(&format!(
            "\n--- 文件统计 ---\n 原始大小: {} bytes | 策略: Hybrid\n 段落数: {} | 索引长度: {} chars\n",
            text.len(),
            section_count,
            section_detail_lines
        ));

        let tail_start = text.len().saturating_sub(Self::TAIL_SIZE);
        let tail_chars: String = text.chars().skip(tail_start).take(Self::TAIL_SIZE).collect();
        if !tail_chars.trim().is_empty() {
            content.push_str("\n--- 文件尾部 (后20KB) ---\n");
            content.push_str(&tail_chars);
        }

        let token_estimate = Self::estimate_tokens(&content);
        ContextStrategyResult {
            content,
            strategy: ContextStrategy::Hybrid,
            original_size: text.len(),
            truncated: true,
            token_estimate,
            format: format.clone(),
            sections,
        }
    }

    /// 根据内容大小自动选择并应用策略
    pub fn apply(text: &str, format: &FileFormat) -> ContextStrategyResult {
        let strategy = Self::select_strategy(text.len());
        match strategy {
            ContextStrategy::FullStuff => Self::apply_full(text, format),
            ContextStrategy::HeadIndex => Self::apply_head_index(text, format),
            ContextStrategy::Hybrid => Self::apply_hybrid(text, format),
        }
    }

    /// 从 FileParseResult 直接应用策略
    pub fn from_parse_result(result: &FileParseResult) -> ContextStrategyResult {
        Self::apply(&result.text, &result.format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::file_parser::FileParser;

    #[test]
    fn test_select_strategy_small() {
        assert_eq!(ContextStrategy::select_strategy(1_000), ContextStrategy::FullStuff);
        assert_eq!(ContextStrategy::select_strategy(49_999), ContextStrategy::FullStuff);
    }

    #[test]
    fn test_select_strategy_medium() {
        assert_eq!(ContextStrategy::select_strategy(50_001), ContextStrategy::HeadIndex);
        assert_eq!(ContextStrategy::select_strategy(100_000), ContextStrategy::HeadIndex);
        assert_eq!(ContextStrategy::select_strategy(499_999), ContextStrategy::HeadIndex);
    }

    #[test]
    fn test_select_strategy_large() {
        assert_eq!(ContextStrategy::select_strategy(500_001), ContextStrategy::Hybrid);
    }

    #[test]
    fn test_estimate_tokens_ascii() {
        let tokens = ContextStrategy::estimate_tokens("hello world this is a test");
        assert!(tokens > 0);
        assert!(tokens < 20);
    }

    #[test]
    fn test_estimate_tokens_mixed() {
        let tokens = ContextStrategy::estimate_tokens("你好 world 测试 test");
        assert!(tokens > 0);
    }

    #[test]
    fn test_extract_sections_from_markdown() {
        let text = "# Title\n\nSome content\n\n## Section One\n\nDetail here\n\n## Section Two\n\nMore details\n\nNOTE: important\n";
        let sections = ContextStrategy::extract_sections(text);
        assert!(!sections.is_empty(), "should find sections");
        assert!(sections.iter().any(|s| s.contains("Title")), "Title section");
    }

    #[test]
    fn test_apply_full_stuff_small() {
        let result = ContextStrategy::apply("short text content", &FileFormat::PlainText);
        assert_eq!(result.strategy, ContextStrategy::FullStuff);
        assert!(!result.truncated);
        assert!(result.content.contains("short text content"));
    }

    #[test]
    fn test_apply_head_index_medium() {
        let text = "A\n".repeat(30_000);
        let result = ContextStrategy::apply(&text, &FileFormat::Markdown);
        assert_eq!(result.strategy, ContextStrategy::HeadIndex);
        assert!(result.content.contains("文件头部"));
    }

    #[test]
    fn test_apply_hybrid_large() {
        let text = "B\n".repeat(300_000);
        let result = ContextStrategy::apply(&text, &FileFormat::Code);
        assert_eq!(result.strategy, ContextStrategy::Hybrid);
        assert!(result.truncated);
        assert!(result.content.contains("文件统计"));
    }

    #[test]
    fn test_from_parse_result() {
        let parsed = FileParser::extract_text("test.txt", "text/plain", b"hello world from parse");
        let result = ContextStrategy::from_parse_result(&parsed);
        assert_eq!(result.format, FileFormat::PlainText);
        assert!(result.content.contains("hello world"));
    }

    #[test]
    fn test_strategy_no_data_loss_for_small() {
        let text = "small content for testing";
        let result = ContextStrategy::apply(text, &FileFormat::Code);
        assert_eq!(result.original_size, text.len());
        assert!(result.content.contains("small content"));
    }

    #[test]
    fn test_empty_text() {
        let result = ContextStrategy::apply("", &FileFormat::PlainText);
        assert_eq!(result.strategy, ContextStrategy::FullStuff);
        assert!(result.content.is_empty());
    }

    #[test]
    fn test_token_estimate_empty() {
        assert_eq!(ContextStrategy::estimate_tokens(""), 1);
    }

    #[test]
    fn test_sections_empty_text() {
        let sections = ContextStrategy::extract_sections("");
        assert!(sections.is_empty());
    }

    #[test]
    fn test_strategy_boundary_small_max() {
        let text = "x".repeat(49_999);
        let result = ContextStrategy::apply(&text, &FileFormat::PlainText);
        assert_eq!(result.strategy, ContextStrategy::FullStuff);
    }

    #[test]
    fn test_strategy_boundary_medium_max() {
        let text = "y".repeat(499_999);
        let result = ContextStrategy::apply(&text, &FileFormat::PlainText);
        assert_eq!(result.strategy, ContextStrategy::HeadIndex);
    }
}
