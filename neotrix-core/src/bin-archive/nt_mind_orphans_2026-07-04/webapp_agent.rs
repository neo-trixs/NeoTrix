use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 浏览器采集的信息所有权标记 — 所有外部数据归意识所有
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedKnowledge {
    pub source_url: String,
    pub source_type: String,       // "webapp" | "webpage" | "search"
    pub title: String,
    pub content: String,
    pub summary: String,
    pub collected_at: u64,
    pub consumed: bool,            // 已被 SEAL pipeline 消费
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WebAppType {
    WhatsApp,
    Gmail,
    TwitterX,
    GitHub,
    Slack,
    Telegram,
    LinkedIn,
    Notion,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAppAction {
    pub id: String,
    pub label: String,
    pub script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAppAgent {
    pub id: String,
    pub name: String,
    pub url_pattern: String,
    pub app_type: WebAppType,
    pub actions: Vec<WebAppAction>,
    pub is_active: bool,
    pub last_seen: u64,
}

impl WebAppAgent {
    pub fn detect(url: &str, title: &str) -> Option<WebAppType> {
        let lower_url = url.to_lowercase();
        let lower_title = title.to_lowercase();

        if lower_url.contains("web.whatsapp.com") || lower_title.contains("whatsapp") {
            return Some(WebAppType::WhatsApp);
        }
        if lower_url.contains("mail.google.com") || lower_title.contains("gmail") {
            return Some(WebAppType::Gmail);
        }
        if lower_url.contains("twitter.com") || lower_url.contains("x.com") || lower_title.contains("twitter") || lower_title == "x" {
            return Some(WebAppType::TwitterX);
        }
        if lower_url.contains("github.com") {
            return Some(WebAppType::GitHub);
        }
        if lower_url.contains("slack.com") {
            return Some(WebAppType::Slack);
        }
        if lower_url.contains("web.telegram.org") || lower_title.contains("telegram") {
            return Some(WebAppType::Telegram);
        }
        if lower_url.contains("linkedin.com") {
            return Some(WebAppType::LinkedIn);
        }
        if lower_url.contains("notion.so") || lower_url.contains("notion.site") {
            return Some(WebAppType::Notion);
        }

        None
    }

    pub fn builtin_actions(app_type: &WebAppType) -> Vec<WebAppAction> {
        match app_type {
            WebAppType::WhatsApp => vec![
                WebAppAction {
                    id: "whatsapp_send".into(),
                    label: "Send Message".into(),
                    script: r#"
(async function() {
    const input = document.querySelector('div[contenteditable="true"][data-tab="10"]');
    if (!input) return "no input field found";
    input.focus();
    document.execCommand('insertText', false, ARGS.message || 'Hello from NeoTrix');
    input.dispatchEvent(new Event('input', {bubbles:true}));
    await new Promise(r => setTimeout(r, 500));
    const sendBtn = document.querySelector('button[data-tab="11"]') || document.querySelector('span[data-icon="send"]');
    if (sendBtn) { sendBtn.click(); return "sent"; }
    return "send button not found";
})();
"#.into(),
                },
                WebAppAction {
                    id: "whatsapp_read".into(),
                    label: "Read Messages".into(),
                    script: r#"
(function() {
    const chats = document.querySelectorAll('div[data-testid="conversation-panel-messages"] div.message-in, div[data-testid="conversation-panel-messages"] div.message-out');
    const msgs = Array.from(chats).slice(-10).map(m => m.textContent).filter(Boolean);
    return msgs.length ? msgs.join('\n---\n') : "no messages found";
})();
"#.into(),
                },
                WebAppAction {
                    id: "whatsapp_qr".into(),
                    label: "Check QR Code".into(),
                    script: r#"
(function() {
    const canvas = document.querySelector('canvas');
    if (canvas) return "QR code visible - scan with phone";
    const loggedIn = document.querySelector('header');
    if (loggedIn) return "already logged in";
    return "unknown state";
})();
"#.into(),
                },
            ],
            WebAppType::Gmail => vec![
                WebAppAction {
                    id: "gmail_inbox".into(),
                    label: "Read Inbox".into(),
                    script: r#"
(function() {
    const threads = document.querySelectorAll('tr.zA');
    return Array.from(threads).slice(0,10).map(t => t.querySelector('b')?.textContent || t.textContent?.slice(0,100)).filter(Boolean).join('\n');
})();
"#.into(),
                },
            ],
            WebAppType::TwitterX => vec![
                WebAppAction {
                    id: "x_timeline".into(),
                    label: "Read Timeline".into(),
                    script: r#"
(async function() {
    const MAX = 20;
    const tweets = [];
    let lastHeight = 0;
    while (tweets.length < MAX) {
        const articles = document.querySelectorAll('article[data-testid="tweet"]');
        for (const art of articles) {
            if (tweets.length >= MAX) break;
            const id = art.querySelector('a[href*="/status/"]')?.href?.split('/status/').pop()?.split('?')[0] || '';
            if (tweets.some(t => t.id === id)) continue;
            const textEl = art.querySelector('div[data-testid="tweetText"]');
            const authorEl = art.querySelector('div[data-testid="User-Name"] a');
            tweets.push({
                id,
                author: authorEl?.textContent?.split('@')[0]?.trim() || '',
                handle: authorEl?.href?.split('/').pop() || '',
                text: textEl?.textContent || '',
            });
        }
        window.scrollTo(0, document.body.scrollHeight);
        await new Promise(r => setTimeout(r, 1500));
        const h = document.body.scrollHeight;
        if (h === lastHeight) break;
        lastHeight = h;
    }
    return JSON.stringify(tweets);
})();
"#.into(),
                },
                WebAppAction {
                    id: "x_thread".into(),
                    label: "Read Thread".into(),
                    script: r#"
(function() {
    const articles = document.querySelectorAll('article[data-testid="tweet"]');
    return Array.from(articles).map(a => {
        const textEl = a.querySelector('div[data-testid="tweetText"]');
        const authorEl = a.querySelector('div[data-testid="User-Name"] a');
        return (authorEl?.textContent || '') + ': ' + (textEl?.textContent || '');
    }).join('\n---\n');
})();
"#.into(),
                },
                WebAppAction {
                    id: "x_search".into(),
                    label: "Search X".into(),
                    script: r#"
(async function() {
    const q = ARGS.query || 'AI';
    const input = document.querySelector('input[data-testid="SearchBox_Search_Input"]');
    if (input) {
        input.value = q;
        input.dispatchEvent(new Event('input', {bubbles:true}));
        await new Promise(r => setTimeout(r, 1000));
        const searchBtn = document.querySelector('button[data-testid="search"]') || input.closest('form')?.querySelector('button');
        if (searchBtn) searchBtn.click();
        await new Promise(r => setTimeout(r, 3000));
    }
    const articles = document.querySelectorAll('article[data-testid="tweet"]');
    return Array.from(articles).slice(0,10).map(a => {
        const textEl = a.querySelector('div[data-testid="tweetText"]');
        return textEl?.textContent || '';
    }).filter(Boolean).join('\n---\n');
})();
"#.into(),
                },
            ],
            WebAppType::GitHub => vec![
                WebAppAction {
                    id: "github_notifications".into(),
                    label: "Notifications".into(),
                    script: r#"
(function() {
    const items = document.querySelectorAll('.notifications-list-item');
    return Array.from(items).slice(0,10).map(i => i.textContent?.trim()).filter(Boolean).join('\n');
})();
"#.into(),
                },
            ],
            _ => vec![
                WebAppAction {
                    id: "page_info".into(),
                    label: "Page Info".into(),
                    script: r#"
(function() {
    return JSON.stringify({title: document.title, url: location.href, links: document.querySelectorAll('a').length});
})();
"#.into(),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAppRegistry {
    agents: Vec<WebAppAgent>,
    by_url: HashMap<String, usize>,
    collected: Vec<CollectedKnowledge>,
}

/// 智能内容提取 — 从页面文本中找出有意义的结构化信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContent {
    pub title: String,
    pub primary_text: String,
    pub headings: Vec<String>,
    pub links: Vec<LinkInfo>,
    pub lists: Vec<Vec<String>>,
    pub tables: Vec<Vec<Vec<String>>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    pub text: String,
    pub href: String,
}

impl ExtractedContent {
    pub fn from_html(html: &str, url: &str) -> Self {
        let title = Self::extract_title(html).unwrap_or_else(|| url.to_string());
        let primary_text = Self::extract_primary_text(html);
        let headings = Self::extract_headings(html);
        let links = Self::extract_links(html);
        let lists = Self::extract_lists(html);
        let tables = Self::extract_tables(html);
        let metadata = Self::extract_metadata(html);
        Self { title, primary_text, headings, links, lists, tables, metadata }
    }

    /// 智能提取页面主要内容 (去除导航/广告/脚注噪声)
    pub fn extract_primary_text(html: &str) -> String {
        let stripped = Self::strip_tags(html);
        let lines: Vec<&str> = stripped.lines()
            .map(|l| l.trim())
            .filter(|l| l.len() > 20)
            .collect();
        lines.join("\n")
    }

    fn extract_title(html: &str) -> Option<String> {
        html.split("<title>").nth(1)
            .and_then(|s| s.split("</title>").next())
            .map(|s| s.trim().to_string())
    }

    fn extract_headings(html: &str) -> Vec<String> {
        let mut out = Vec::new();
        for tag in &["h1", "h2", "h3"] {
            let open = format!("<{}", tag);
            let close = format!("</{}>", tag);
            for part in html.split(&open).skip(1) {
                if let Some(content) = part.split('>').nth(1).and_then(|s| s.split(&close).next()) {
                    let cleaned = Self::strip_tags(content).trim().to_string();
                    if !cleaned.is_empty() {
                        out.push(cleaned);
                    }
                }
            }
        }
        out
    }

    fn extract_links(html: &str) -> Vec<LinkInfo> {
        let mut out = Vec::new();
        for part in html.split("<a ").skip(1) {
            let href = part.split("href=\"").nth(1)
                .and_then(|s| s.split('"').next())
                .unwrap_or("");
            let text = part.split('>').nth(1)
                .and_then(|s| s.split("</a>").next())
                .map(|s| Self::strip_tags(s).trim().to_string())
                .unwrap_or_default();
            if !href.is_empty() && !text.is_empty() {
                out.push(LinkInfo { text, href: href.to_string() });
            }
        }
        out.into_iter().take(50).collect()
    }

    fn extract_lists(html: &str) -> Vec<Vec<String>> {
        let mut lists = Vec::new();
        for list_html in html.split("<ul>").skip(1) {
            let list_end = list_html.split("</ul>").next().unwrap_or("");
            let items: Vec<String> = list_end.split("<li")
                .skip(1)
                .filter_map(|item| {
                    item.split('>').nth(1)
                        .and_then(|s| s.split("</li>").next())
                        .map(|s| Self::strip_tags(s).trim().to_string())
                })
                .filter(|s| !s.is_empty())
                .collect();
            if !items.is_empty() {
                lists.push(items);
            }
        }
        lists.into_iter().take(5).collect()
    }

    fn extract_tables(html: &str) -> Vec<Vec<Vec<String>>> {
        let mut tables = Vec::new();
        for table_html in html.split("<table").skip(1) {
            let table_end = table_html.split("</table>").next().unwrap_or("");
            let mut rows = Vec::new();
            for row_html in table_end.split("<tr").skip(1) {
                let row_end = row_html.split("</tr>").next().unwrap_or("");
                let cells: Vec<String> = row_end.split("<td")
                    .skip(1)
                    .filter_map(|cell| {
                        cell.split('>').nth(1)
                            .and_then(|s| s.split("</td>").next())
                            .map(|s| Self::strip_tags(s).trim().to_string())
                    })
                    .collect();
                if !cells.is_empty() {
                    rows.push(cells);
                }
            }
            if !rows.is_empty() {
                tables.push(rows);
            }
        }
        tables.into_iter().take(3).collect()
    }

    fn extract_metadata(html: &str) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        for part in html.split("<meta ").skip(1) {
            let name = part.split("name=\"").nth(1)
                .and_then(|s| s.split('"').next())
                .unwrap_or("");
            let content = part.split("content=\"").nth(1)
                .and_then(|s| s.split('"').next())
                .unwrap_or("");
            if !name.is_empty() && !content.is_empty() {
                meta.insert(name.to_string(), content.to_string());
            }
        }
        meta
    }

    /// 简单 HTML 标签剥离 (保留文本)
    fn strip_tags(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut in_tag = false;
        for c in input.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => out.push(c),
                _ => {}
            }
        }
        out
    }

    /// 摘要: 前 N 字符
    pub fn summary(&self, max_chars: usize) -> String {
        let mut parts: Vec<String> = Vec::new();
        if !self.headings.is_empty() {
            parts.push(format!("📑 {}", self.headings[..self.headings.len().min(5)].join(" > ")));
        }
        if !self.primary_text.is_empty() {
            let text = if self.primary_text.len() > max_chars {
                // 使用 chars() 而非字节索引以防多字节 UTF-8
                let truncated: String = self.primary_text.chars().take(max_chars).collect();
                format!("{}...", truncated)
            } else {
                self.primary_text.clone()
            };
            parts.push(text);
        }
        if !self.links.is_empty() {
            parts.push(format!("🔗 {} links", self.links.len()));
        }
        if !self.tables.is_empty() {
            parts.push(format!("📊 {} tables", self.tables.len()));
        }
        if !self.lists.is_empty() {
            parts.push(format!("📋 {} lists", self.lists.len()));
        }
        parts.join("\n\n")
    }
}

impl WebAppRegistry {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            by_url: HashMap::new(),
            collected: Vec::new(),
        }
    }

    pub fn detect_or_create(&mut self, url: &str, title: &str) -> Option<&WebAppAgent> {
        if let Some(&idx) = self.by_url.get(url) {
            self.agents[idx].last_seen = timestamp_now();
            self.agents[idx].is_active = true;
            return Some(&self.agents[idx]);
        }

        let app_type = WebAppAgent::detect(url, title)?;
        let name = format!("{:?}", app_type);
        let id = format!("webapp-{}", self.agents.len());

        let agent = WebAppAgent {
            id,
            name,
            url_pattern: url.to_string(),
            app_type: app_type.clone(),
            actions: WebAppAgent::builtin_actions(&app_type),
            is_active: true,
            last_seen: timestamp_now(),
        };

        let idx = self.agents.len();
        self.by_url.insert(url.to_string(), idx);
        self.agents.push(agent);
        Some(&self.agents[idx])
    }

    pub fn all_agents(&self) -> &[WebAppAgent] {
        &self.agents
    }

    pub fn get_by_id(&self, id: &str) -> Option<&WebAppAgent> {
        self.agents.iter().find(|a| a.id == id)
    }

    pub fn get_by_url(&self, url: &str) -> Option<&WebAppAgent> {
        self.by_url.get(url).map(|&idx| &self.agents[idx])
    }

    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.agents.iter().position(|a| a.id == id) {
            self.by_url.retain(|_, v| *v != pos);
            self.agents.remove(pos);
            true
        } else {
            false
        }
    }

    /// 采集知识入队 — 浏览器数据归属意识所有, 供 SEAL pipeline 消费
    pub fn enqueue_collected_knowledge(&mut self, knowledge: CollectedKnowledge) {
        self.collected.push(knowledge);
    }

    /// 取出所有未消费的知识 (所有权转移, 消费后标记)
    pub fn drain_unconsumed(&mut self) -> Vec<CollectedKnowledge> {
        let mut result = Vec::new();
        self.collected.retain(|k| {
            if k.consumed {
                true
            } else {
                result.push(CollectedKnowledge {
                    source_url: k.source_url.clone(),
                    source_type: k.source_type.clone(),
                    title: k.title.clone(),
                    content: k.content.clone(),
                    summary: k.summary.clone(),
                    collected_at: k.collected_at,
                    consumed: false,
                });
                false
            }
        });
        result
    }

    /// 列出未消费知识 (非破坏性, 仅查看)
    pub fn list_unconsumed(&self) -> Vec<CollectedKnowledge> {
        self.collected.iter()
            .filter(|k| !k.consumed)
            .map(|k| CollectedKnowledge {
                source_url: k.source_url.clone(),
                source_type: k.source_type.clone(),
                title: k.title.clone(),
                content: k.content.clone(),
                summary: k.summary.clone(),
                collected_at: k.collected_at,
                consumed: false,
            })
            .collect()
    }

    /// 从 HTML 文本自动采集并入队
    pub fn ingest_from_browser(&mut self, url: &str, title: &str, html: &str) -> CollectedKnowledge {
        let extracted = ExtractedContent::from_html(html, url);
        let summary = extracted.summary(500);
        let knowledge = CollectedKnowledge {
            source_url: url.to_string(),
            source_type: if WebAppAgent::detect(url, title).is_some() { "webapp".into() } else { "webpage".into() },
            title: title.to_string(),
            content: extracted.primary_text.clone(),
            summary,
            collected_at: timestamp_now(),
            consumed: false,
        };
        let k = knowledge.clone();
        self.collected.push(knowledge);
        k
    }

    /// 采集队列大小
    pub fn collected_count(&self) -> usize {
        self.collected.len()
    }
}

fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_whatsapp() {
        assert_eq!(
            WebAppAgent::detect("https://web.whatsapp.com", "WhatsApp"),
            Some(WebAppType::WhatsApp)
        );
    }

    #[test]
    fn test_detect_gmail() {
        assert_eq!(
            WebAppAgent::detect("https://mail.google.com", "Inbox"),
            Some(WebAppType::Gmail)
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(
            WebAppAgent::detect("https://example.com", "Example"),
            None
        );
    }

    #[test]
    fn test_register_and_find() {
        let mut reg = WebAppRegistry::new();
        let agent = reg.detect_or_create("https://web.whatsapp.com", "WhatsApp");
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().app_type, WebAppType::WhatsApp);
        assert_eq!(reg.all_agents().len(), 1);
    }

    #[test]
    fn test_detect_twitter() {
        let t = WebAppAgent::detect("https://x.com/home", "Home / X");
        assert_eq!(t, Some(WebAppType::TwitterX));
    }

    #[test]
    fn test_whatsapp_actions() {
        let actions = WebAppAgent::builtin_actions(&WebAppType::WhatsApp);
        assert!(actions.iter().any(|a| a.id == "whatsapp_send"));
        assert!(actions.iter().any(|a| a.id == "whatsapp_read"));
    }

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Test Page</title></head><body><h1>Hello</h1><p>Some content here that is long enough to be meaningful text content.</p></body></html>";
        let extracted = ExtractedContent::from_html(html, "https://example.com");
        assert_eq!(extracted.title, "Test Page");
        assert!(extracted.primary_text.contains("meaningful text content"));
    }

    #[test]
    fn test_extract_headings() {
        let html = "<h1>Title</h1><h2>Section 1</h2><h3>Sub 1</h3>";
        let extracted = ExtractedContent::from_html(html, "");
        assert!(extracted.headings.contains(&"Title".to_string()));
        assert!(extracted.headings.contains(&"Section 1".to_string()));
    }

    #[test]
    fn test_extract_links() {
        let html = r#"<a href="https://example.com">Example Link</a>"#;
        let extracted = ExtractedContent::from_html(html, "");
        assert_eq!(extracted.links.len(), 1);
        assert_eq!(extracted.links[0].href, "https://example.com");
    }

    #[test]
    fn test_extract_tables() {
        let html = "<table><tr><td>A</td><td>B</td></tr><tr><td>C</td><td>D</td></tr></table>";
        let extracted = ExtractedContent::from_html(html, "");
        assert_eq!(extracted.tables.len(), 1);
        assert_eq!(extracted.tables[0].len(), 2);
        assert_eq!(extracted.tables[0][0][0], "A");
    }

    #[test]
    fn test_extract_metadata() {
        let html = r#"<meta name="description" content="Test desc"><meta name="keywords" content="test,rust">"#;
        let extracted = ExtractedContent::from_html(html, "");
        assert_eq!(extracted.metadata.get("description").map(|s| s.as_str()), Some("Test desc"));
    }

    #[test]
    fn test_strip_tags() {
        let result = ExtractedContent::strip_tags("<p>Hello <b>World</b></p>");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_remove_agent() {
        let mut reg = WebAppRegistry::new();
        reg.detect_or_create("https://github.com", "GitHub");
        let ag_id = reg.detect_or_create("https://web.whatsapp.com", "WhatsApp").unwrap().id.clone();
        assert!(reg.remove(&ag_id));
        assert_eq!(reg.all_agents().len(), 1);
    }

    #[test]
    fn test_ingest_from_browser() {
        let mut reg = WebAppRegistry::new();
        let k = reg.ingest_from_browser("https://example.com", "Test", "<html><body><p>Hello world this is some content that is long enough to pass the length filter for extraction. More content here to make it even longer and more meaningful for the test case.</p></body></html>");
        assert_eq!(k.source_url, "https://example.com");
        assert_eq!(k.source_type, "webpage");
        assert!(k.content.contains("Hello world"));
    }

    #[test]
    fn test_drain_unconsumed() {
        let mut reg = WebAppRegistry::new();
        reg.ingest_from_browser("https://a.com", "A", "<p>this content is long enough to pass the length filter for extraction and be considered meaningful text content for the test.</p>");
        reg.ingest_from_browser("https://b.com", "B", "<p>this content is long enough to pass the length filter for extraction and be considered meaningful text content for the test.</p>");
        assert_eq!(reg.collected_count(), 2);
        let drained = reg.drain_unconsumed();
        assert_eq!(drained.len(), 2);
        assert_eq!(reg.collected_count(), 0);
    }

    #[test]
    fn test_collected_knowledge_ownership() {
        let mut reg = WebAppRegistry::new();
        reg.ingest_from_browser("https://whatsapp.com", "WhatsApp", "<p>WhatsApp messages content here that is long enough to pass the length filter and be considered meaningful text for the test case we are running right now.</p>");
        let drained = reg.drain_unconsumed();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].source_type, "webapp");
        assert!(!drained[0].consumed);
    }
}
