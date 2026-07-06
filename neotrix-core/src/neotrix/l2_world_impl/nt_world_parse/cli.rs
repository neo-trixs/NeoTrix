//! CLI 命令 — 文档解析 CLI
//!
//! neotrix parse <path> [--format markdown|json] [--output <path>]

#[derive(Default)]
pub struct DocParseCli;

impl DocParseCli {
    pub fn new() -> Self {
        Self
    }

    pub fn name(&self) -> &str {
        "parse"
    }

    pub fn description(&self) -> &str {
        "Parse documents to Markdown/JSON"
    }
}
