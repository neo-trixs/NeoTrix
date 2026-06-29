use std::collections::HashMap;

/// 输出格式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
    Toml,
    Markdown,
    Debug,
}

impl OutputFormat {
    pub fn from_ext(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "json" => Self::Json,
            "yaml" | "yml" => Self::Yaml,
            "toml" => Self::Toml,
            "md" | "markdown" => Self::Markdown,
            "txt" | "text" => Self::Text,
            _ => Self::Text,
        }
    }

    pub fn ext(&self) -> &str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Markdown => "md",
            Self::Text => "txt",
            Self::Debug => "txt",
        }
    }

    pub fn mime(&self) -> &str {
        match self {
            Self::Json => "application/json",
            Self::Yaml => "application/x-yaml",
            Self::Toml => "application/toml",
            Self::Markdown => "text/markdown",
            Self::Text | Self::Debug => "text/plain",
        }
    }
}

/// 输出条目 — 可路由的渲染结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutputEntry {
    pub format: OutputFormat,
    pub content: String,
    pub title: Option<String>,
    pub tags: Vec<String>,
}

impl OutputEntry {
    pub fn text(content: &str) -> Self {
        Self {
            format: OutputFormat::Text,
            content: content.to_string(),
            title: None,
            tags: Vec::new(),
        }
    }

    pub fn json(content: &str) -> Self {
        Self {
            format: OutputFormat::Json,
            content: content.to_string(),
            title: None,
            tags: Vec::new(),
        }
    }

    pub fn markdown(content: &str) -> Self {
        Self {
            format: OutputFormat::Markdown,
            content: content.to_string(),
            title: None,
            tags: Vec::new(),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
}

/// 输出路由目标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum OutputTarget {
    /// stdout
    Console,
    /// 文件系统
    File,
    /// HTTP 响应
    Http,
    /// 事件总线
    EventBus,
    /// 意识循环内部
    Consciousness,
}

/// 输出格式化器 trait
pub trait OutputFormatter: Send + Sync {
    fn format(&self, entry: &OutputEntry) -> String;
    fn format_to(&self, entry: &OutputEntry, target: OutputTarget) -> String {
        let base = self.format(entry);
        match target {
            OutputTarget::Console | OutputTarget::Consciousness => base,
            OutputTarget::File | OutputTarget::Http | OutputTarget::EventBus => base,
        }
    }
}

/// Text 格式化器
pub struct TextFormatter;

impl OutputFormatter for TextFormatter {
    fn format(&self, entry: &OutputEntry) -> String {
        entry.content.clone()
    }
}

/// JSON 格式化器
pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format(&self, entry: &OutputEntry) -> String {
        serde_json::to_string_pretty(entry).unwrap_or_else(|_| entry.content.clone())
    }
}

/// Markdown 格式化器
pub struct MarkdownFormatter;

impl OutputFormatter for MarkdownFormatter {
    fn format(&self, entry: &OutputEntry) -> String {
        let mut out = String::new();
        if let Some(ref title) = entry.title {
            out.push_str(&format!("# {}\n\n", title));
        }
        out.push_str(&entry.content);
        out.push('\n');
        out
    }
}

/// 输出路由器 — 按格式和目标分发输出
pub struct OutputRouter {
    formatters: HashMap<OutputFormat, Box<dyn OutputFormatter>>,
    routes: HashMap<(String, OutputTarget), OutputFormat>,
    fallback: OutputFormat,
}

impl OutputRouter {
    pub fn new() -> Self {
        let mut formatters: HashMap<OutputFormat, Box<dyn OutputFormatter>> = HashMap::new();
        formatters.insert(OutputFormat::Text, Box::new(TextFormatter));
        formatters.insert(OutputFormat::Json, Box::new(JsonFormatter));
        formatters.insert(OutputFormat::Markdown, Box::new(MarkdownFormatter));

        Self {
            formatters,
            routes: HashMap::new(),
            fallback: OutputFormat::Text,
        }
    }

    pub fn register_formatter(
        &mut self,
        format: OutputFormat,
        formatter: Box<dyn OutputFormatter>,
    ) {
        self.formatters.insert(format, formatter);
    }

    /// 注册路由：指定 channel + target 使用什么格式
    pub fn add_route(&mut self, channel: &str, target: OutputTarget, format: OutputFormat) {
        self.routes.insert((channel.to_string(), target), format);
    }

    /// 渲染并路由一个输出条目
    pub fn render(&self, entry: &OutputEntry, channel: &str, target: OutputTarget) -> String {
        let fmt = self
            .routes
            .get(&(channel.to_string(), target))
            .copied()
            .unwrap_or(entry.format);

        let formatter = self
            .formatters
            .get(&fmt)
            .or_else(|| self.formatters.get(&self.fallback));
        match formatter {
            Some(f) => f.format_to(entry, target),
            None => {
                log::warn!(
                    "output: no formatter for {:?} or fallback {:?}",
                    fmt,
                    self.fallback
                );
                entry.content.clone()
            }
        }
    }

    /// 直接格式化
    pub fn format(&self, entry: &OutputEntry, format: OutputFormat) -> String {
        let formatter = self
            .formatters
            .get(&format)
            .or_else(|| self.formatters.get(&self.fallback));
        match formatter {
            Some(f) => f.format(entry),
            None => {
                log::warn!(
                    "output: no formatter for {:?} or fallback {:?}",
                    format,
                    self.fallback
                );
                entry.content.clone()
            }
        }
    }

    /// 默认路由器
    pub fn default() -> Self {
        let mut router = Self::new();
        router.add_route("chat", OutputTarget::Console, OutputFormat::Text);
        router.add_route("report", OutputTarget::File, OutputFormat::Markdown);
        router.add_route("api", OutputTarget::Http, OutputFormat::Json);
        router
    }
}

impl Default for OutputRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_ext() {
        assert_eq!(OutputFormat::from_ext("json"), OutputFormat::Json);
        assert_eq!(OutputFormat::from_ext("md"), OutputFormat::Markdown);
        assert_eq!(OutputFormat::from_ext("txt"), OutputFormat::Text);
        assert_eq!(OutputFormat::from_ext("unknown"), OutputFormat::Text);
    }

    #[test]
    fn test_text_formatter() {
        let entry = OutputEntry::text("hello world");
        let fmt = TextFormatter;
        assert_eq!(fmt.format(&entry), "hello world");
    }

    #[test]
    fn test_json_formatter_valid() {
        let entry = OutputEntry::json("{\"key\":\"value\"}");
        let fmt = JsonFormatter;
        let result = fmt.format(&entry);
        assert!(result.contains("key"));
    }

    #[test]
    fn test_markdown_formatter_with_title() {
        let entry = OutputEntry::markdown("content").with_title("Title");
        let fmt = MarkdownFormatter;
        let result = fmt.format(&entry);
        assert!(result.contains("# Title"));
        assert!(result.contains("content"));
    }

    #[test]
    fn test_router_render_console() {
        let router = OutputRouter::default();
        let entry = OutputEntry::text("hi");
        let result = router.render(&entry, "chat", OutputTarget::Console);
        assert_eq!(result, "hi");
    }

    #[test]
    fn test_router_custom_route() {
        let mut router = OutputRouter::default();
        router.add_route("chat", OutputTarget::Console, OutputFormat::Markdown);
        let entry = OutputEntry::text("hello").with_title("Doc");
        let result = router.render(&entry, "chat", OutputTarget::Console);
        assert!(result.contains("# Doc"));
        assert!(result.contains("hello"));
    }
}
