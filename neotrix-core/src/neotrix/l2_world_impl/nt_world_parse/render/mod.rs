//! Render module — MCP/CLI 渲染入口
//!
//! 提供统一的渲染接口, 供 MCP 工具和 CLI 命令调用

use super::renderers::markdown_renderer::MarkdownRenderer;
use super::renderers::json_renderer::JsonRenderer;
use super::renderers::chunk_renderer::ChunkRenderer;
use super::doc_parser::ParsedDocument;

pub enum OutputFormat {
    Markdown,
    Json,
    Chunks(usize),
}

/// 渲染文档为指定格式
pub fn render(doc: &ParsedDocument, format: OutputFormat) -> String {
    match format {
        OutputFormat::Markdown => MarkdownRenderer::render(doc),
        OutputFormat::Json => serde_json::to_string_pretty(&JsonRenderer::render(doc)).unwrap_or_default(),
        OutputFormat::Chunks(max_chars) => {
            let chunks = ChunkRenderer::chunk(doc, max_chars);
            chunks.iter().map(|(text, page)| format!("<!-- Page {} -->\n{}", page, text)).collect::<Vec<_>>().join("\n\n")
        }
    }
}
