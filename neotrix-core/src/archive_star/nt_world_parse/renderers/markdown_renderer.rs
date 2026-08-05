use super::super::doc_parser::ParsedDocument;

/// Markdown 渲染器
pub struct MarkdownRenderer;

impl MarkdownRenderer {
    pub fn render(doc: &ParsedDocument) -> String {
        let title = doc.title.as_deref().unwrap_or("Untitled");
        let mut output = format!("# {}\n\n", title);
        for page in &doc.pages {
            output.push_str(&format!("## Page {}\n\n", page.page_num));
            output.push_str(&page.markdown);
            output.push_str("\n\n---\n\n");
        }
        output
    }
}
