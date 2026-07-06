use super::super::doc_parser::ParsedDocument;

/// 分块渲染器 — 用于 KB 存储 (FTS5 分块索引)
pub struct ChunkRenderer;

impl ChunkRenderer {
    pub fn chunk(doc: &ParsedDocument, max_chars: usize) -> Vec<(String, usize)> {
        doc.pages.iter().map(|p| {
            let chunk = if p.markdown.len() > max_chars {
                p.markdown.chars().take(max_chars).collect::<String>()
            } else {
                p.markdown.clone()
            };
            (chunk, p.page_num)
        }).collect()
    }
}
