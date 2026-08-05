use super::super::doc_parser::ParsedDocument;

/// JSON 树渲染器 — 映射到 nt_memory_kb 的 22 节点类型 / 19 关系
pub struct JsonRenderer;

impl JsonRenderer {
    pub fn render(doc: &ParsedDocument) -> serde_json::Value {
        let pages: Vec<serde_json::Value> = doc.pages.iter().map(|p| {
            serde_json::json!({
                "page_num": p.page_num,
                "text": p.markdown,
                "confidence": p.confidence,
                "backend": p.backend_used,
            })
        }).collect();

        serde_json::json!({
            "title": doc.title,
            "page_count": doc.pages.len(),
            "avg_confidence": doc.avg_confidence,
            "pages": pages,
        })
    }
}
