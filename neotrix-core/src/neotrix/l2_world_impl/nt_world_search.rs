use serde::{Deserialize, Serialize};

/// A reusable web search tool wrapping the unified ordered-backend router.
/// Provides structured-text results suitable for LLM context injection.
/// R-P79 接线: 作为统一搜索表面被 agent / CLI / tool 消费 (非死代码)。
pub struct WebSearchTool {
    search: UnifiedSearch,
}

impl WebSearchTool {
    pub fn new() -> Self {
        Self { search: UnifiedSearch::default() }
    }

    /// 当前生效的后端 (doctor 体检可报告走哪条路)。
    pub fn active_backend(&self) -> String {
        self.search.active_backend()
    }

    /// Execute a web search (unified: DDG → Wikipedia fallback) as formatted text.
    pub fn search(&self, query: &str, count: usize) -> Result<String, String> {
        let results = self.search.search(query, count)?;
        if results.is_empty() {
            return Ok("No web search results found.".to_string());
        }
        let mut msg = format!("Web search results for \"{}\":\n\n", query);
        for (i, r) in results.iter().enumerate() {
            msg.push_str(&format!("{}. {}\n   URL: {}\n   {}\n\n", i + 1, r.title, r.url, r.snippet));
        }
        Ok(msg.trim().to_string())
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DuckDuckGoResponse {
    #[serde(default)]
    AbstractText: String,
    #[serde(default)]
    AbstractSource: String,
    #[serde(default)]
    AbstractURL: String,
    #[serde(default)]
    Results: Vec<DuckDuckGoItem>,
    #[serde(default)]
    RelatedTopics: Vec<DuckDuckGoTopic>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DuckDuckGoItem {
    #[serde(default)]
    Text: String,
    #[serde(default)]
    FirstURL: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(non_snake_case, dead_code)]
enum DuckDuckGoTopic {
    Leaf {
        #[serde(default)]
        Text: String,
        #[serde(default)]
        FirstURL: String,
    },
    Category {
        #[serde(default)]
        Name: String,
        #[serde(default)]
        Topics: Vec<DuckDuckGoItem>,
    },
}

pub struct WebSearchEngine {
    api_base_url: String,
    /// 惰性初始化: 避免在 tokio async 上下文创建 blocking client 触发
    /// "Cannot drop a runtime in a context where blocking is not allowed" panic。
    /// 首次 search() 调用时才创建（此时通常在 spawn_blocking/同步上下文）。
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for WebSearchEngine {
    fn default() -> Self {
        let base = std::env::var("NEOTRIX_SEARCH_API")
            .unwrap_or_else(|_| "https://api.duckduckgo.com".to_string());
        Self::new(&base)
    }
}

impl WebSearchEngine {
    pub fn new(api_base_url: &str) -> Self {
        Self {
            api_base_url: api_base_url.trim_end_matches('/').to_string(),
            client: std::sync::OnceLock::new(),
        }
    }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    pub fn search(&self, query: &str, count: usize) -> Result<Vec<SearchResult>, String> {
        let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
        let url = format!(
            "{}/?q={}&format=json&no_html=1&skip_disambig=1",
            self.api_base_url, encoded
        );

        let resp = self
            .client()
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("API returned status: {}", resp.status()));
        }

        let ddg: DuckDuckGoResponse = resp
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let mut results = Vec::new();

        if !ddg.AbstractText.is_empty() {
            let url = if ddg.AbstractURL.is_empty() {
                format!("https://en.wikipedia.org/wiki/{}", query.replace(' ', "_"))
            } else {
                ddg.AbstractURL.clone()
            };
            results.push(SearchResult {
                title: if ddg.AbstractSource.is_empty() {
                    query.to_string()
                } else {
                    format!("{} — {}", query, ddg.AbstractSource)
                },
                url,
                snippet: ddg.AbstractText,
            });
        }

        for item in &ddg.Results {
            if results.len() >= count {
                break;
            }
            let title = item.Text.split(" — ").next().unwrap_or(&item.Text).to_string();
            results.push(SearchResult {
                title,
                url: item.FirstURL.clone(),
                snippet: item.Text.clone(),
            });
        }

        for topic in &ddg.RelatedTopics {
            if results.len() >= count {
                break;
            }
            match topic {
                DuckDuckGoTopic::Leaf { Text, FirstURL } => {
                    let title = Text.split(" — ").next().unwrap_or(Text).to_string();
                    results.push(SearchResult {
                        title,
                        url: FirstURL.clone(),
                        snippet: Text.clone(),
                    });
                }
                DuckDuckGoTopic::Category { Topics, .. } => {
                    for item in Topics {
                        if results.len() >= count {
                            break;
                        }
                        let title = item.Text.split(" — ").next().unwrap_or(&item.Text).to_string();
                        results.push(SearchResult {
                            title,
                            url: item.FirstURL.clone(),
                            snippet: item.Text.clone(),
                        });
                    }
                }
            }
        }

        Ok(results)
    }
}

// ============================================================
// 搜索统一化 (R-P82 有序后端路由 — 借鉴 OSINT BackendRouter)
// ============================================================
//
// 缺陷: 原 `WebSearchEngine` 单一后端 (仅 DuckDuckGo), 无 fallback — 一旦
// DDG 限流/封禁, 全网搜索即失效。R-P82 有序后端路由: 首选 + 备选的真实探测,
// 第一个完整可用的当选, 接入方式换代只调整列表顺序不重写能力层。

/// 搜索后端抽象 — 真实可探测, doctor 体检可报告当前路径。
pub trait SearchBackend: Send + Sync {
    fn name(&self) -> &str;
    fn search(&self, query: &str, count: usize) -> Result<Vec<SearchResult>, String>;
}

/// DuckDuckGo 后端 (首选) — 复用既有 WebSearchEngine 解析。
pub struct DuckDuckGoBackend {
    engine: WebSearchEngine,
}

impl Default for DuckDuckGoBackend {
    fn default() -> Self {
        Self { engine: WebSearchEngine::default() }
    }
}

impl SearchBackend for DuckDuckGoBackend {
    fn name(&self) -> &str {
        "duckduckgo"
    }

    fn search(&self, query: &str, count: usize) -> Result<Vec<SearchResult>, String> {
        self.engine.search(query, count)
    }
}

/// Wikipedia 后端 (备选) — 免费无 key, 用 search API 兜底。
pub struct WikipediaBackend {
    /// 惰性初始化: 避免在 tokio async 上下文创建 blocking client 触发 panic
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for WikipediaBackend {
    fn default() -> Self {
        Self { client: std::sync::OnceLock::new() }
    }
}

impl WikipediaBackend {
    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }
}

#[derive(Debug, Deserialize)]
struct WikipediaSearchResponse {
    query: WikipediaQuery,
}

#[derive(Debug, Deserialize)]
struct WikipediaQuery {
    #[serde(default)]
    search: Vec<WikipediaSearchItem>,
}

#[derive(Debug, Deserialize)]
struct WikipediaSearchItem {
    #[serde(default)]
    title: String,
    #[serde(default)]
    snippet: String,
}

impl SearchBackend for WikipediaBackend {
    fn name(&self) -> &str {
        "wikipedia"
    }

    fn search(&self, query: &str, count: usize) -> Result<Vec<SearchResult>, String> {
        let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
        let api = format!(
            "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&srlimit={}&format=json",
            encoded, count
        );
        let resp = self
            .client()
            .get(&api)
            .send()
            .map_err(|e| format!("Wikipedia request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Wikipedia returned status: {}", resp.status()));
        }
        let parsed: WikipediaSearchResponse = resp
            .json()
            .map_err(|e| format!("Wikipedia parse failed: {}", e))?;
        let results = parsed.query.search.into_iter().map(|item| {
            let snippet = item.snippet.replace("<span class=\"searchmatch\">", "")
                .replace("</span>", "")
                .replace("&quot;", "\"")
                .replace("&amp;", "&");
            SearchResult {
                title: item.title.clone(),
                url: format!("https://en.wikipedia.org/wiki/{}", item.title.replace(' ', "_")),
                snippet,
            }
        }).collect();
        Ok(results)
    }
}

/// 有序搜索路由 — 首选 + 备选, 失败自动切换 (R-P82)。
pub struct WebSearchRouter {
    backends: Vec<Box<dyn SearchBackend>>,
    /// 最近一次成功走的后端名 (doctor 体检展示)
    current: String,
    /// 各后端最后一次错误 (诊断用)
    last_errors: Vec<(String, String)>,
}

impl WebSearchRouter {
    pub fn new(backends: Vec<Box<dyn SearchBackend>>) -> Self {
        Self {
            backends,
            current: "none".to_string(),
            last_errors: Vec::new(),
        }
    }

    /// 默认有序后端: DuckDuckGo 首选 → Wikipedia 备选。
    pub fn default_ordered() -> Self {
        Self::new(vec![
            Box::new(DuckDuckGoBackend::default()),
            Box::new(WikipediaBackend::default()),
        ])
    }

    /// 按顺序探测, 第一个成功返回的后端当选; 全失败则收集错误。
    pub fn search(&mut self, query: &str, count: usize) -> Result<Vec<SearchResult>, String> {
        self.last_errors.clear();
        for backend in &self.backends {
            match backend.search(query, count) {
                Ok(results) if !results.is_empty() => {
                    self.current = backend.name().to_string();
                    return Ok(results);
                }
                Ok(_) => {
                    self.last_errors.push((backend.name().to_string(), "empty results".into()));
                }
                Err(e) => {
                    self.last_errors.push((backend.name().to_string(), e));
                }
            }
        }
        let detail = self.last_errors.iter()
            .map(|(n, e)| format!("{}: {}", n, e))
            .collect::<Vec<_>>()
            .join("; ");
        Err(format!("all search backends failed: {}", detail))
    }

    /// 当前走哪条路 (doctor 体检)。
    pub fn current_backend(&self) -> &str {
        &self.current
    }

    /// 全后端健康体检。
    pub fn backends(&self) -> &[Box<dyn SearchBackend>] {
        &self.backends
    }
}

/// 统一搜索表面 — 给 agent/工具一条路由, 封装 router 的可变状态。
pub struct UnifiedSearch {
    router: std::sync::Mutex<WebSearchRouter>,
}

impl Default for UnifiedSearch {
    fn default() -> Self {
        Self {
            router: std::sync::Mutex::new(WebSearchRouter::default_ordered()),
        }
    }
}

impl UnifiedSearch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn search(&self, query: &str, count: usize) -> Result<Vec<SearchResult>, String> {
        let mut guard = self.router.lock().map_err(|_| "router lock poisoned".to_string())?;
        guard.search(query, count)
    }

    pub fn active_backend(&self) -> String {
        self.router.lock().map(|g| g.current_backend().to_string()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 探针后端 — 可控成功/失败, 用于离线验证路由 fallback 语义。
    struct ProbeBackend {
        name: &'static str,
        succeed: bool,
        results: usize,
    }

    impl SearchBackend for ProbeBackend {
        fn name(&self) -> &str {
            self.name
        }
        fn search(&self, _query: &str, _count: usize) -> Result<Vec<SearchResult>, String> {
            if !self.succeed {
                return Err("probe backend down".to_string());
            }
            Ok((0..self.results)
                .map(|i| SearchResult {
                    title: format!("{} result {}", self.name, i),
                    url: "https://probe.test".into(),
                    snippet: "probe".into(),
                })
                .collect())
        }
    }

    #[test]
    fn router_uses_first_working_backend() {
        let mut router = WebSearchRouter::new(vec![
            Box::new(ProbeBackend { name: "primary", succeed: true, results: 3 }),
            Box::new(ProbeBackend { name: "backup", succeed: true, results: 2 }),
        ]);
        let results = router.search("hello", 5).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(router.current_backend(), "primary");
    }

    #[test]
    fn router_falls_back_when_primary_fails() {
        // R-P82: 首选失败 → 自动切备选, 不整体失败。
        let mut router = WebSearchRouter::new(vec![
            Box::new(ProbeBackend { name: "primary", succeed: false, results: 0 }),
            Box::new(ProbeBackend { name: "backup", succeed: true, results: 4 }),
        ]);
        let results = router.search("hello", 5).unwrap();
        assert_eq!(results.len(), 4);
        assert_eq!(router.current_backend(), "backup");
    }

    #[test]
    fn router_all_fail_reports_errors() {
        // 全失败 → 收集各后端错误, 诊断信息完整。
        let mut router = WebSearchRouter::new(vec![
            Box::new(ProbeBackend { name: "primary", succeed: false, results: 0 }),
            Box::new(ProbeBackend { name: "backup", succeed: false, results: 0 }),
        ]);
        let err = router.search("hello", 5).unwrap_err();
        assert!(err.contains("primary"));
        assert!(err.contains("backup"));
    }

    #[test]
    fn router_empty_primary_uses_nonempty_backup() {
        // 首选返回空 (被限流/无结果) → 切换到有结果的备选。
        let mut router = WebSearchRouter::new(vec![
            Box::new(ProbeBackend { name: "primary", succeed: true, results: 0 }),
            Box::new(ProbeBackend { name: "backup", succeed: true, results: 2 }),
        ]);
        let results = router.search("hello", 5).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(router.current_backend(), "backup");
    }

    #[test]
    fn default_router_has_ordered_backends() {
        // 默认路由 = DDG 首选 + Wikipedia 备选。
        let router = WebSearchRouter::default_ordered();
        let names: Vec<String> = router.backends().iter().map(|b| b.name().to_string()).collect();
        assert_eq!(names, vec!["duckduckgo", "wikipedia"]);
        // doctor 体检: 未搜索前 current 应为 none。
        assert_eq!(router.current_backend(), "none");
    }

    #[test]
    fn unified_search_exposes_active_backend() {
        let search = UnifiedSearch::new();
        // 未搜索前无路由 (避免依赖网络), 接口可调即可。
        let _ = search.active_backend();
        // WebSearchTool 作为统一表面可构造。
        let tool = WebSearchTool::new();
        assert_eq!(tool.active_backend(), "none");
    }
}
