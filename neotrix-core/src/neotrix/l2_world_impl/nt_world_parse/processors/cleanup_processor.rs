/// 文档清理后处理器
pub struct CleanupProcessor;

impl CleanupProcessor {
    pub fn clean(markdown: &str) -> String {
        markdown.trim().to_string()
    }
}
