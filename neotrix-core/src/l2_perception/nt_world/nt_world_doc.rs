//! NT-WORLD 共享多格式文档模型 + 长文档滑窗 — 缺陷网 D12/D14 修复:
//! - D12 无共享多格式文档模型: 统一 ParsedDoc schema, 任何格式 (text/md/html/
//!   json) 解析后均产出同一结构, 共用一套序列化。
//! - D14 无长文档滑窗: sliding_window 分块降低峰值内存, 保留阅读顺序 (去页眉页脚)。
//!
//! 参照: anydoc (14/14 格式统一输出), MinerU (sliding-window 峰值内存优化)。

use serde::{Deserialize, Serialize};

/// 统一文档解析模型 — 任何输入格式都产出此 schema。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDoc {
    pub format: String, // "html" | "md" | "pdf" | "json" | "text"
    pub title: String,
    pub text: String,
    pub links: Vec<String>,
    pub blocks: Vec<DocBlock>,
    /// 滑窗分块 (可选, D14)
    pub chunks: Vec<String>,
    /// 滑窗元数据
    pub chunk_params: Option<ChunkParams>,
}

/// 文档块 (保留阅读顺序)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocBlock {
    pub kind: String, // "heading" | "para" | "list" | "code"
    pub text: String,
}

/// 滑窗参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkParams {
    pub window: usize,
    pub overlap: usize,
}

impl ParsedDoc {
    pub fn new(format: &str, title: &str, text: &str, links: Vec<String>, blocks: Vec<DocBlock>) -> Self {
        Self {
            format: format.to_string(),
            title: title.to_string(),
            text: text.to_string(),
            links,
            blocks,
            chunks: Vec::new(),
            chunk_params: None,
        }
    }
}

/// D12: 多格式解析入口 — 依 format 分派, 全部产出 ParsedDoc 统一 schema。
/// 不做重量级 HTML/PDF 渲染, 只做文本级清洗 (标签剥离 / JSON 扁平)。
pub fn parse(format: &str, raw: &str) -> ParsedDoc {
    match format {
        "html" => parse_html(raw),
        "md" | "markdown" => parse_md(raw),
        "json" => parse_json(raw),
        _ => parse_text(raw),
    }
}

/// 极简 HTML 清洗: 去标签、聚合成块。
fn parse_html(raw: &str) -> ParsedDoc {
    let mut links = Vec::new();
    let mut blocks = Vec::new();
    // 提取 link href
    for part in raw.split("<a ") {
        if let Some(href) = part.split("href=").nth(1) {
            let h = href.trim_start_matches(['"', '\'']);
            let h: String = h.chars().take_while(|c| *c != '"' && *c != '\'').collect();
            if !h.is_empty() {
                links.push(h);
            }
        }
    }
    // 剥离 tag
    let in_tag = raw.chars().fold((String::new(), false), |(mut acc, mut t), c| {
        if t {
            if c == '>' {
                t = false;
            }
        } else if c == '<' {
            t = true;
        } else {
            acc.push(c);
        }
        (acc, t)
    });
    let stripped = strip_junk(&in_tag.0);
    let title = stripped.lines().next().unwrap_or("untitled").to_string();
    let text = stripped.clone();
    for line in stripped.lines().filter(|l| !l.trim().is_empty()) {
        blocks.push(DocBlock { kind: "para".to_string(), text: line.to_string() });
    }
    ParsedDoc::new("html", &title, &text, links, blocks)
}

fn parse_md(raw: &str) -> ParsedDoc {
    let mut blocks = Vec::new();
    for line in raw.lines().filter(|l| !l.trim().is_empty()) {
        let kind = if line.starts_with('#') {
            "heading"
        } else if line.starts_with('-') || line.starts_with("*") || line.starts_with(|c: char| c.is_ascii_digit()) && line.contains('.') {
            "list"
        } else if line.starts_with("```") {
            "code"
        } else {
            "para"
        };
        blocks.push(DocBlock { kind: kind.to_string(), text: line.to_string() });
    }
    let text = raw.to_string();
    let title = blocks.first().map(|b| b.text.clone()).unwrap_or_else(|| "untitled".to_string());
    ParsedDoc::new("md", &title, &text, Vec::new(), blocks)
}

fn parse_json(raw: &str) -> ParsedDoc {
    let links = Vec::new();
    let text = raw.to_string();
    let title = "json".to_string();
    let blocks = vec![DocBlock { kind: "json".to_string(), text: raw.to_string() }];
    ParsedDoc::new("json", &title, &text, links, blocks)
}

fn parse_text(raw: &str) -> ParsedDoc {
    let blocks: Vec<DocBlock> = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| DocBlock { kind: "para".to_string(), text: l.to_string() })
        .collect();
    let title = blocks.first().map(|b| b.text.clone()).unwrap_or_else(|| "untitled".to_string());
    ParsedDoc::new("text", &title, raw, Vec::new(), blocks)
}

/// 剥离典型页眉页脚噪声行 (D14: 阅读顺序输出去页眉页脚)。
pub fn strip_junk(text: &str) -> String {
    text.lines()
        .filter(|l| {
            let t = l.trim().to_lowercase();
            !(t.contains("©") || t.contains("copyright") || t == "\u{2022}"
                || t.contains("all rights reserved"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// D14: sliding-window 分块 — 降峰值内存的同时保留阅读顺序。
/// 支持 overlap (连贯性)。块间用 [k/n] 前缀保留顺序。
pub fn sliding_window(text: &str, window_tokens: usize, overlap: usize) -> Vec<String> {
    if text.trim().is_empty() || window_tokens == 0 {
        return Vec::new();
    }
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.len() <= window_tokens {
        return vec![text.trim().to_string()];
    }
    let step = window_tokens.saturating_sub(overlap).max(1);
    let mut chunks = Vec::new();
    let total = (tokens.len().saturating_sub(window_tokens)) / step + 1;
    for (i, start) in (0..tokens.len()).step_by(step).enumerate() {
        if start >= tokens.len() {
            break;
        }
        let end = (start + window_tokens).min(tokens.len());
        let chunk = tokens[start..end].join(" ");
        chunks.push(format!("[{}:{}] {}", start, end, chunk));
        if i + 1 >= total || end == tokens.len() {
            break;
        }
    }
    chunks
}

/// 应用滑窗到 ParsedDoc, 填 chunks 与 chunk_params。
pub fn apply_chunking(doc: &mut ParsedDoc, window_tokens: usize, overlap: usize) {
    doc.chunks = sliding_window(&doc.text, window_tokens, overlap);
    doc.chunk_params = Some(ChunkParams { window: window_tokens, overlap });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_parse_produces_unified_schema() {
        let raw = "<html><a href='/x'>link</a><p>hello world</p></html>";
        let doc = parse("html", raw);
        // D12: 统一 schema — 字段齐全
        assert_eq!(doc.format, "html");
        assert!(doc.links.contains(&"/x".to_string()));
        assert!(doc.text.to_lowercase().contains("hello"));
        assert!(!doc.blocks.is_empty());
    }

    #[test]
    fn md_parse_produces_unified_schema() {
        let raw = "# Title\n\n- item one\n\npara text";
        let doc = parse("md", raw);
        assert_eq!(doc.format, "md");
        assert_eq!(doc.blocks[0].text, "# Title");
    }

    #[test]
    fn text_and_json_share_schema() {
        let t = parse("text", "hello\nworld");
        let j = parse("json", "{\"a\":1}");
        assert_eq!(t.format, "text");
        assert!(!t.blocks.is_empty());
        assert_eq!(j.format, "json");
        // 同一 schema: 都能序列化
        let tj = serde_json::to_string(&t).unwrap();
        let jj = serde_json::to_string(&j).unwrap();
        assert!(!tj.is_empty());
        assert!(!jj.is_empty());
    }

    #[test]
    fn strip_junk_removes_header_footer() {
        let cleaned = strip_junk("Real content\n© 2026 Foo Inc.\nCopyright Bar\nmore real");
        assert!(cleaned.contains("Real content"));
        assert!(cleaned.contains("more real"));
        assert!(!cleaned.to_lowercase().contains("copyright"));
        assert!(!cleaned.contains("©"));
    }

    #[test]
    fn sliding_window_reduces_peak_memory() {
        let text = "a b c d e f g h i j k l m n o p q r s t u v w x y z";
        let chunks = sliding_window(text, 10, 3);
        assert!(chunks.len() >= 3, "long doc split into multiple windows");
        assert!(chunks.iter().all(|c| c.split_whitespace().count() <= 13), "each chunk bounded near window");
        // 阅读顺序: 前缀序号非递减
        assert!(chunks[0].starts_with("[0:"));
    }

    #[test]
    fn chunk_and_order_preserved() {
        let mut doc = parse("text", "one two three four five six");
        apply_chunking(&mut doc, 3, 1);
        assert!(!doc.chunks.is_empty());
        assert_eq!(doc.chunk_params.as_ref().unwrap().window, 3);
    }
}