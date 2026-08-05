//! ingest — 文档 → nt_memory_kb 管线
//!
//! 解析文档后, 创建 Concept 节点 (章节) 和 Relation 边 (层级),
//! 全文进入 FTS5 索引

pub struct DocIngester;

impl DocIngester {
    pub fn new() -> Self {
        Self
    }

    /// 将解析文档存入 KB
    pub fn ingest(&self, _doc: &super::doc_parser::ParsedDocument) -> Result<String, String> {
        Err("DocIngester not yet implemented — requires nt_memory_kb integration".into())
    }
}

impl Default for DocIngester {
    fn default() -> Self {
        Self::new()
    }
}
