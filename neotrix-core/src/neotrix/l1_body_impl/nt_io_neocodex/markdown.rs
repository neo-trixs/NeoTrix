// ── Streaming Markdown (from Claude Code: tolerant parser) ──

#[derive(Debug, Clone)]
pub struct StreamingMarkdown {
    pub buffer: String,
    pub chunks: Vec<MarkdownChunk>,
    pub code_fence_open: Option<String>,
    pub list_stack: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum MarkdownChunk {
    Text(String),
    CodeBlock {
        language: Option<String>,
        content: String,
        complete: bool,
    },
    Heading {
        level: u8,
        text: String,
    },
    ListItem {
        depth: u8,
        text: String,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        complete: bool,
    },
    Image {
        alt: String,
        url: String,
    },
    Link {
        text: String,
        url: String,
    },
    BlockQuote(String),
    HorizontalRule,
}

impl Default for StreamingMarkdown {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingMarkdown {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            chunks: Vec::new(),
            code_fence_open: None,
            list_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, text: &str) {
        self.buffer.push_str(text);
        self.reparse();
    }

    fn reparse(&mut self) {
        let content = self.buffer.clone();
        self.chunks.clear();

        let mut i = 0;
        while i < content.len() {
            if let Some(rest) = content[i..].strip_prefix("```") {
                let line_end = rest.find('\n').unwrap_or(rest.len());
                let lang = rest[..line_end].trim().to_string();
                let lang = if lang.is_empty() { None } else { Some(lang) };
                let code_start = i + 3 + line_end + if line_end < rest.len() { 1 } else { 0 };
                let remaining = &content[code_start..];
                if let Some(close_pos) = remaining.find("```") {
                    let code = remaining[..close_pos].to_string();
                    self.chunks.push(MarkdownChunk::CodeBlock {
                        language: lang,
                        content: code,
                        complete: true,
                    });
                    i = code_start + close_pos + 3;
                } else {
                    self.chunks.push(MarkdownChunk::CodeBlock {
                        language: lang,
                        content: remaining.to_string(),
                        complete: false,
                    });
                    break;
                }
            } else if content[i..].starts_with("## ") {
                let end = content[i..]
                    .find('\n')
                    .map(|p| i + p)
                    .unwrap_or(content.len());
                self.chunks.push(MarkdownChunk::Heading {
                    level: 2,
                    text: content[i + 3..end].trim().to_string(),
                });
                i = end + 1;
            } else if content[i..].starts_with("# ") {
                let end = content[i..]
                    .find('\n')
                    .map(|p| i + p)
                    .unwrap_or(content.len());
                self.chunks.push(MarkdownChunk::Heading {
                    level: 1,
                    text: content[i + 2..end].trim().to_string(),
                });
                i = end + 1;
            } else if content[i..].starts_with("- ") || content[i..].starts_with("* ") {
                let end = content[i..]
                    .find('\n')
                    .map(|p| i + p)
                    .unwrap_or(content.len());
                self.chunks.push(MarkdownChunk::ListItem {
                    depth: 0,
                    text: content[i + 2..end].trim().to_string(),
                });
                i = end + 1;
            } else if content[i..].starts_with("> ") {
                let end = content[i..]
                    .find('\n')
                    .map(|p| i + p)
                    .unwrap_or(content.len());
                self.chunks.push(MarkdownChunk::BlockQuote(
                    content[i + 2..end].trim().to_string(),
                ));
                i = end + 1;
            } else if content[i..].starts_with("---") {
                self.chunks.push(MarkdownChunk::HorizontalRule);
                i += 3;
            } else {
                let end = content[i..]
                    .find('\n')
                    .map(|p| i + p + 1)
                    .unwrap_or(content.len());
                let text = content[i..end].trim().to_string();
                if !text.is_empty() {
                    self.chunks.push(MarkdownChunk::Text(text));
                }
                i = end;
            }
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.chunks.clear();
        self.code_fence_open = None;
        self.list_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_markdown_plain_text() {
        let mut sm = StreamingMarkdown::new();
        sm.push("Hello world");
        assert!(!sm.chunks.is_empty());
    }

    #[test]
    fn test_streaming_markdown_code_block() {
        let mut sm = StreamingMarkdown::new();
        sm.push("```rust\nfn main() {}\n```");
        assert!(sm
            .chunks
            .iter()
            .any(|c| matches!(c, MarkdownChunk::CodeBlock { .. })));
    }

    #[test]
    fn test_streaming_markdown_incomplete_code_block() {
        let mut sm = StreamingMarkdown::new();
        sm.push("```rust\nfn main() {\nprintln!(\"hello\");");
        assert!(sm.chunks.iter().any(|c| matches!(
            c,
            MarkdownChunk::CodeBlock {
                complete: false,
                ..
            }
        )));
    }

    #[test]
    fn test_streaming_markdown_headings() {
        let mut sm = StreamingMarkdown::new();
        sm.push("# Title\n## Subtitle\nText body");
        assert!(sm
            .chunks
            .iter()
            .any(|c| matches!(c, MarkdownChunk::Heading { level: 1, .. })));
        assert!(sm
            .chunks
            .iter()
            .any(|c| matches!(c, MarkdownChunk::Heading { level: 2, .. })));
    }

    #[test]
    fn test_streaming_markdown_clear() {
        let mut sm = StreamingMarkdown::new();
        sm.push("Some content");
        assert!(!sm.chunks.is_empty());
        sm.clear();
        assert!(sm.chunks.is_empty());
        assert!(sm.buffer.is_empty());
    }
}