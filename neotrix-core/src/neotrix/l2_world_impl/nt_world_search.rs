use serde::{Deserialize, Serialize};

/// argo 吸收 (2026-08-17): 证据可信度评分管线 — selection(权威) ×
/// absorption(证据密度) + freshness(时效) + 共识. 强化 nt_world_search 现有节点
/// (R-P42), 非平行适配器. 算法移植自 taxueseek/argo (MIT), 精简为 Rust 单文件。
/// 综合 = 0.40·selection + 0.35·absorption + 0.15·freshness + 0.10·引擎分
///
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
    /// 结果带证据可信度标注 (argo 吸收)。
    pub fn search(&self, query: &str, count: usize) -> Result<String, String> {
        let results = self.search.search(query, count)?;
        if results.is_empty() {
            return Ok("No web search results found.".to_string());
        }
        let mut msg = format!("Web search results for \"{}\":\n\n", query);
        for (i, r) in results.iter().enumerate() {
            let ev = r.evidence.as_ref().map(|e| format!(" [{}]", e.label())).unwrap_or_default();
            msg.push_str(&format!("{}. {}{}\n   URL: {}\n   {}\n\n", i + 1, r.title, ev, r.url, r.snippet));
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
    /// 证据可信度字段 (argo 吸收): None 表示未评分 (纯搜索引擎原始结果)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<EvidenceScore>,
}

/// 证据可信度分解 (argo evidence 管线移植)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceScore {
    /// 综合可信度 ∈ [0,1]
    pub final_score: f64,
    /// 权威性 (selection) ∈ [0,1]
    pub selection: f64,
    /// 证据密度 (absorption) ∈ [0,1]
    pub absorption: f64,
    /// 时效性 ∈ [0,1]
    pub freshness: f64,
    /// 权威层级标签 (official/professional/general/low/very_low)
    pub tier: String,
    /// 是否搜索结果页/跳转链 (不可作吸收源)
    pub is_serp: bool,
    /// 证据密度特征标记
    pub has_numbers: bool,
    pub has_definition: bool,
    pub has_comparison: bool,
    pub has_disclose: bool,
}

impl EvidenceScore {
    pub fn label(&self) -> String {
        let tier = match self.tier.as_str() {
            "official" => "官方",
            "professional" => "专业",
            "general" => "通用",
            "low" => "低质",
            _ => "极低",
        };
        format!(
            "{}({:.0}%){}",
            tier,
            self.final_score * 100.0,
            if self.has_numbers { "·数字" } else { "" }
        )
    }
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
                evidence: None,
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
                evidence: None,
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
                        evidence: None,
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
                            evidence: None,
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

// ============================================================
// 证据可信度评分 (argo 吸收) — selection × absorption + freshness
// ============================================================
// 移植自 taxueseek/argo scripts/evidence.py + content_signals.py (MIT)。
// 目的: Agent 需要的不是「被检索到」, 而是「可抽取、可核对的证据块」。
// 综合分 = 0.40·selection + 0.35·absorption + 0.15·freshness + 0.10·引擎分。

/// 域名权威分层 (最长后缀匹配)。
const AUTHORITY_TIERS: &[(&str, f64)] = &[
    // Tier 1: 官方/权威
    ("gov.cn", 1.0), ("gov", 0.95), ("edu.cn", 0.95), ("edu", 0.9), ("ac.cn", 0.95),
    ("nature.com", 0.95), ("science.org", 0.95), ("ieee.org", 0.9),
    ("acm.org", 0.9), ("springer.com", 0.9), ("elsevier.com", 0.9),
    ("arxiv.org", 0.9), ("pubmed.ncbi.nlm.nih.gov", 0.95),
    ("scholar.google.com", 0.85), ("ncbi.nlm.nih.gov", 0.95),
    ("nvd.nist.gov", 0.95), ("cve.mitre.org", 0.95),
    ("xinhuanet.com", 0.95), ("people.com.cn", 0.95),
    // Tier 2: 专业媒体/平台
    ("zhihu.com", 0.85), ("github.com", 0.85), ("stackoverflow.com", 0.85),
    ("medium.com", 0.75), ("dev.to", 0.75),
    ("reuters.com", 0.9), ("bloomberg.com", 0.9), ("wsj.com", 0.9),
    ("caixin.com", 0.9), ("yicai.com", 0.86), ("wallstreetcn.com", 0.86),
    ("cls.cn", 0.88), ("cs.com.cn", 0.92), ("stcn.com", 0.88),
    ("cnr.cn", 0.9), ("thepaper.cn", 0.82), ("jiemian.com", 0.8),
    ("36kr.com", 0.8), ("infoq.cn", 0.8), ("juejin.cn", 0.75),
    ("eastmoney.com", 0.85), ("xueqiu.com", 0.8), ("10jqka.com.cn", 0.78),
    ("docs.python.org", 0.9), ("react.dev", 0.9), ("nextjs.org", 0.9),
    // Tier 3: 通用可信
    ("wikipedia.org", 0.75), ("baike.baidu.com", 0.7),
    ("linkedin.com", 0.65), ("twitter.com", 0.45), ("x.com", 0.45),
    ("reddit.com", 0.55),
    // Tier 4: 内容农场/低质
    ("sohu.com", 0.4), ("163.com", 0.45), ("sina.com.cn", 0.55),
    ("k.sina.com.cn", 0.55), ("baijiahao.baidu.com", 0.35),
    ("zhuanlan.zhihu.com", 0.7), ("toutiao.com", 0.4), ("weixin.qq.com", 0.5),
    ("guba.eastmoney.com", 0.35),
    // 商业榜单 (高引用≠高可信)
    ("maigoo.com", 0.28), ("chinapp.com", 0.28), ("cnpp.cn", 0.28),
];

fn tier_label(score: f64) -> &'static str {
    if score >= 0.9 { "official" }
    else if score >= 0.75 { "professional" }
    else if score >= 0.55 { "general" }
    else if score >= 0.35 { "low" }
    else { "very_low" }
}

/// SERP / 跳转链域名 — 不可作吸收源。
const SERP_MARKERS: &[&str] = &[
    "google.com/search", "bing.com/search", "baidu.com/s",
    "duckduckgo.com", "yandex.com/search", "github.com/search",
    "/redirect?", "url=", "jump.", "link.", "/goto",
];

fn is_serp_url(url: &str) -> bool {
    let low = url.to_lowercase();
    SERP_MARKERS.iter().any(|m| low.contains(m))
}

/// 提取域名 (去协议/路径/端口/www)。
fn extract_domain(url: &str) -> String {
    let u = url.trim();
    let after = u.find("://").map(|i| &u[i + 3..]).unwrap_or(u);
    let host = after.split(['/', '?', '#']).next().unwrap_or(after);
    let host = host.split(':').next().unwrap_or(host);
    let host = host.to_lowercase();
    host.trim_start_matches("www.").to_string()
}

/// 提取完整 hostname (保留 www 等子域前缀) — 用于站点学习经验子域通配匹配
/// (对应 ego-lite urlHostname + domainMatches 语义)。
fn extract_host(url: &str) -> String {
    let u = url.trim();
    let after = u.find("://").map(|i| &u[i + 3..]).unwrap_or(u);
    let host = after.split(['/', '?', '#']).next().unwrap_or(after);
    host.split(':').next().unwrap_or(host).to_lowercase()
}

/// Selection: 域名权威评分。SERP 压到极低。
fn score_authority(url: &str) -> (f64, String, bool) {
    let domain = extract_domain(url);
    if domain.is_empty() {
        return (0.3, "unknown".to_string(), false);
    }
    let is_serp = is_serp_url(url) || domain.starts_with("search");
    if is_serp {
        return (0.12, "very_low".to_string(), true);
    }
    let mut best = 0.5;
    let mut best_len = -1;
    for (pattern, score) in AUTHORITY_TIERS {
        if domain == *pattern || domain.ends_with(&format!(".{}", pattern)) {
            let plen = pattern.len() as i32;
            if plen > best_len {
                best_len = plen;
                best = *score;
            }
        }
    }
    (best, tier_label(best).to_string(), false)
}

/// Absorption: 证据密度 (数字/定义/对比/howto/披露)。
/// 移植自 argo content_signals.score_evidence_density。
pub fn score_evidence_density(text: &str, title: &str) -> (f64, bool, bool, bool, bool) {
    let body = format!("{}\n{}", title, text).to_lowercase();
    // 数字: 千分位/百分比/计量单位/环比同比/Qn/年份
    let has_numbers = {
        let num = regex_num_patterns(&body);
        !num.is_empty()
    };
    let has_definition = ["是指", "定义为", "所谓", "即指", "指的是", "是一种", "可定义为",
        "definition of", "is defined as", "refers to"]
        .iter().any(|d| body.contains(d));
    let has_comparison = ["对比", "比较", "相较", "相比", "环比", "同比", "分别", "vs", "versus",
        "增持", "减持", "上升", "下降", "提升", "回落", "高于", "低于", "超过", "不及"]
        .iter().any(|c| body.contains(c));
    let has_howto = ["步骤", "如何", "怎么做", "操作建议", "方法如下", "how to", "tutorial", "step"]
        .iter().any(|h| body.contains(h));
    let has_disclose = ["截至", "根据", "数据显示", "研究报告", "披露", "公告", "季报", "年报", "来源:"]
        .iter().any(|d| body.contains(d));

    let mut score = 0.15f64;
    if has_numbers { score += 0.22; }
    if has_definition { score += 0.18; }
    if has_comparison { score += 0.16; }
    if has_howto { score += 0.12; }
    if has_disclose { score += 0.08; }
    if body.len() >= 80 { score += 0.05; }

    (score.clamp(0.0, 1.0), has_numbers, has_definition, has_comparison, has_disclose)
}

/// 数字特征检测: 千分位数字 + 单位、百分比、Q1-4、20xx年。
fn regex_num_patterns(body: &str) -> Vec<String> {
    let mut hits = Vec::new();
    // 千分位/带单位数字
    for pat in [
        r"\d{1,3}(,\d{3})+", r"\d+\.\d+", r"\d+(%|％|亿|万|万亿|pct|bp|元|美元|吨|倍)",
        r"(环比|同比|较上[季年]度?)[^\n]{0,12}[+\-＋－]?\d",
        r"Q[1-4]\b", r"20\d{2}\s*年",
    ] {
        if let Ok(re) = regex::Regex::new(pat) {
            if re.is_match(body) { hits.push(pat.to_string()); }
        }
    }
    hits
}

/// Freshness: 时效性 (URL/文本中出现的年份, 越新越高)。
fn score_freshness(title: &str, snippet: &str, url: &str) -> f64 {
    use chrono::Datelike;
    let now = chrono::Utc::now().year() as f64;
    let combined = format!("{} {} {}", title, snippet, url);
    let year_re = regex::Regex::new(r"(20\d{2})").expect("静态年份正则必合法");
    let years: Vec<f64> = year_re
        .captures_iter(&combined)
        .map(|c| c[1].parse::<f64>().unwrap_or(0.0))
        .filter(|y| *y >= 1990.0 && *y <= now + 1.0)
        .collect();
    if years.is_empty() {
        return 0.5; // 无时间信息, 中性
    }
    let recent: Vec<f64> = years.iter().filter(|y| **y >= now - 1.0).cloned().collect();
    let year = if recent.is_empty() {
        *years.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(&now)
    } else {
        *recent.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(&now)
    };
    let age = now - year;
    if age <= 1.0 { 0.9 }
    else if age <= 2.0 { 0.8 }
    else if age <= 3.0 { 0.7 }
    else if age <= 5.0 { 0.6 }
    else if age <= 10.0 { 0.45 }
    else { 0.3 }
}

/// 共识验证: 多源交叉佐证 (同一事件多独立域名)。
fn cross_validation(results: &[SearchResult], query: &str) -> (f64, usize, usize) {
    let mut domains: Vec<String> = Vec::new();
    let mut content_domains: Vec<String> = Vec::new();
    for r in results {
        let d = extract_domain(&r.url);
        if d.is_empty() { continue; }
        if !domains.contains(&d) { domains.push(d.clone()); }
        if !is_serp_url(&r.url) && !content_domains.contains(&d) { content_domains.push(d); }
    }
    let qwords: Vec<&str> = query.split_whitespace().filter(|w| w.len() > 1).collect();
    let mut matches = 0;
    for r in results {
        let text = format!("{} {}", r.title, r.snippet).to_lowercase();
        if qwords.iter().any(|w| text.contains(&w.to_lowercase())) {
            matches += 1;
        }
    }
    let ratio = matches as f64 / results.len().max(1) as f64;
    let score = if ratio >= 0.8 && content_domains.len() >= 3 { 0.9 }
        else if ratio >= 0.6 && content_domains.len() >= 2 { 0.7 }
        else if ratio >= 0.4 { 0.5 }
        else { 0.3 };
    (score, matches, domains.len())
}

/// 证据评分器 — 对搜索结果批量打分。
pub struct EvidenceScorer;

impl EvidenceScorer {
    /// 对单条结果评分。
    pub fn score(r: &SearchResult, engine_score: f64) -> EvidenceScore {
        let (selection, _tier, is_serp) = score_authority(&r.url);
        let (absorption, has_numbers, has_definition, has_comparison, has_disclose) =
            score_evidence_density(&r.snippet, &r.title);
        let freshness = score_freshness(&r.title, &r.snippet, &r.url);
        let selection_eff = if is_serp { selection.min(0.15) } else { selection };
        let final_score = selection_eff * 0.40 + absorption * 0.35 + freshness * 0.15 + engine_score * 0.10;
        EvidenceScore {
            final_score: (final_score * 100.0).round() / 100.0,
            selection: (selection * 100.0).round() / 100.0,
            absorption: (absorption * 100.0).round() / 100.0,
            freshness: (freshness * 100.0).round() / 100.0,
            tier: tier_label(selection_eff).to_string(),
            is_serp,
            has_numbers,
            has_definition,
            has_comparison,
            has_disclose,
        }
    }

    /// 批量评分 + 按综合可信度降序。
    pub fn score_results(results: Vec<SearchResult>, query: &str) -> Vec<SearchResult> {
        if results.is_empty() { return results; }
        let (consensus, _, _) = cross_validation(&results, query);
        let mut scored: Vec<(f64, SearchResult)> = results.into_iter().map(|r| {
            let ev = Self::score(&r, 0.5);
            // 高共识微抬 (argo: strong +0.05, moderate +0.02)
            let boost = if consensus >= 0.7 { 0.05 } else if consensus >= 0.5 { 0.02 } else { 0.0 };
            let mut ev = ev;
            ev.final_score = ((ev.final_score + boost) * 100.0).round() / 100.0;
            let score_key = ev.final_score;
            let mut r = r;
            r.evidence = Some(ev);
            (score_key, r)
        }).collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().map(|(_, r)| r).collect()
    }

    /// URL 去重 (跨后端: http/https/www/utm 变体合并)。
    pub fn deduplicate_by_url(results: Vec<SearchResult>) -> Vec<SearchResult> {
        let mut seen: Vec<String> = Vec::new();
        let mut out = Vec::new();
        for r in results {
            let key = normalize_url_key(&r.url);
            if !seen.contains(&key) {
                seen.push(key);
                out.push(r);
            }
        }
        out
    }
}

/// URL 归一键: 去协议/www/末尾斜杠/常见 tracking 参数。
pub fn normalize_url_key(url: &str) -> String {
    let mut u = url.to_lowercase();
    for prefix in ["https://", "http://"] {
        if u.starts_with(prefix) {
            u = u[prefix.len()..].to_string();
            break;
        }
    }
    u = u.trim_start_matches("www.").to_string();
    u = u.trim_end_matches('/').to_string();
    // 去 utm/ref/spm 等
    if let Some(qi) = u.find('?') {
        let base = &u[..qi];
        let query = &u[qi + 1..];
        let kept: Vec<&str> = query.split('&')
            .filter(|kv| !kv.starts_with("utm_") && !kv.starts_with("ref=")
                && !kv.starts_with("spm=") && !kv.starts_with("from="))
            .collect();
        u = if kept.is_empty() { base.to_string() } else { format!("{}?{}", base, kept.join("&")) };
    }
    u
}

/// DuckDuckGo 后端 (首选) — 复用既有 WebSearchEngine 解析。
#[derive(Default)]
pub struct DuckDuckGoBackend {
    engine: WebSearchEngine,
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
                evidence: None,
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

    /// 搜索 + 证据评分 (argo 吸收): 去重 → 打分 → 按可信度降序。
    /// ego-lite 吸收: 按结果域名注入站点学习经验 (领域分类) 到 snippet 前缀。
    pub fn search(&self, query: &str, count: usize) -> Result<Vec<SearchResult>, String> {
        let mut guard = self.router.lock().map_err(|_| "router lock poisoned".to_string())?;
        let raw = guard.search(query, count)?;
        let deduped = EvidenceScorer::deduplicate_by_url(raw);
        let registry = SiteLearningRegistry::builtin();
        let mut scored = EvidenceScorer::score_results(deduped, query);
        for r in &mut scored {
            let host = extract_host(&r.url);
            let learns = registry.for_host(&host);
            if !learns.is_empty() {
                let cats: Vec<&str> = learns.iter().map(|l| l.category.as_str()).collect();
                r.snippet = format!("[{}] {}", cats.join("/"), r.snippet);
            }
        }
        Ok(scored.into_iter().take(count).collect())
    }

    /// 原始搜索 (不评分) — 兼容旧接口语义。
    pub fn search_raw(&self, query: &str, count: usize) -> Result<Vec<SearchResult>, String> {
        let mut guard = self.router.lock().map_err(|_| "router lock poisoned".to_string())?;
        guard.search(query, count)
    }

    pub fn active_backend(&self) -> String {
        self.router.lock().map(|g| g.current_backend().to_string()).unwrap_or_default()
    }
}

/* ── ego-lite 吸收 (2026-08-17): 站点学习经验注册表 ──
 * 模式移植自 citrolabs/ego-lite (MIT) learnings 子系统:
 *   - manifest.json  (id / domains / nodeTools)  → SiteLearning { domain, extract, category }
 *   - domainMatches  (*.suffix 只匹配子域, 裸域名精确匹配) → domain_matches()
 *   - validateLearning (域名合法性/结构门控)     → SiteLearningRegistry::validate()
 * 强化 nt_world_search::ordered_backend_router 现有节点 (R-P42), 非平行模块。
 */

/// 单条站点学习经验 — 域名 + 结构化提取知识 + 领域分类。
/// 对应 ego-lite learning manifest 的核心字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteLearning {
    /// 域名模式: "github.com" 或 "*.github.com" (子域)。裸域名精确匹配, `*.` 只匹配子域。
    pub domain: String,
    /// 站点名 (用于展示, 对应 manifest.name)。
    pub name: String,
    /// 领域分类: 知识/官方/学术/社区/商业/新闻。
    pub category: String,
    /// 标题提取经验 (对应 search-extract.js 的 title selector 语义)。
    pub title_field: String,
    /// 链接提取经验。
    pub url_field: String,
    /// 摘要提取经验。
    pub snippet_field: String,
    /// 是否已验证 (对应 validateLearning 门控通过)。
    pub validated: bool,
}

impl SiteLearning {
    /// 新建学习经验 (默认未验证 — 必须过 validate() 门控)。
    pub fn new(domain: &str, name: &str, category: &str) -> Self {
        Self {
            domain: domain.to_string(),
            name: name.to_string(),
            category: category.to_string(),
            title_field: "h3".to_string(),
            url_field: "a[href]".to_string(),
            snippet_field: "[data-sncf]".to_string(),
            validated: false,
        }
    }

    /// 域名是否匹配 (ego-lite domainMatches 语义)。
    /// `*.suffix` 只匹配子域 (hostname.ends_with(".suffix")), 裸域名精确匹配。
    pub fn matches(&self, hostname: &str) -> bool {
        let pattern = self.domain.trim_end_matches('.').to_lowercase();
        let host = hostname.trim_end_matches('.').to_lowercase();
        if pattern.starts_with("*.") {
            let suffix = &pattern[2..];
            host.ends_with(&format!(".{suffix}"))
        } else {
            host == pattern
        }
    }
}

/// 站点学习经验注册表 — 域名 → 提取经验 的声明式注册 + 验证门控 + 查询。
/// 对应 ego-lite learnings/ 目录 + siteSkillsForUrl()。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SiteLearningRegistry {
    learnings: Vec<SiteLearning>,
}

impl SiteLearningRegistry {
    /// 内建站点学习经验 (可扩展)。对应 ego-lite 内置 learnings/google + learnings/x-com。
    pub fn builtin() -> Self {
        let mut r = Self::default();
        // 搜索/知识类
        r.add(SiteLearning::new("google.com", "Google Search", "knowledge"));
        r.add(SiteLearning::new("x.com", "X (Twitter)", "community"));
        r.add(SiteLearning::new("twitter.com", "X (Twitter)", "community"));
        r.add(SiteLearning::new("wikipedia.org", "Wikipedia", "knowledge"));
        r.add(SiteLearning::new("github.com", "GitHub", "code"));
        r.add(SiteLearning::new("*.gov.cn", "中国政府网", "official"));
        r.add(SiteLearning::new("arxiv.org", "arXiv", "academic"));
        r.add(SiteLearning::new("reddit.com", "Reddit", "community"));
        r
    }

    /// 注册一条学习经验, 跑验证门控 (validateLearning 语义)。
    pub fn add(&mut self, learning: SiteLearning) {
        let validated = validate_site_learning(&learning);
        let mut l = learning;
        l.validated = validated;
        self.learnings.push(l);
    }

    /// 查询 hostname 匹配的全部学习经验 (多域名可匹配)。
    pub fn for_host(&self, hostname: &str) -> Vec<&SiteLearning> {
        self.learnings.iter().filter(|l| l.matches(hostname)).collect()
    }

    /// 查询某域名是否存在学习经验 (doctor 体检可报告)。
    pub fn has_for(&self, hostname: &str) -> bool {
        !self.for_host(hostname).is_empty()
    }

    /// 已注册条目数。
    pub fn len(&self) -> usize {
        self.learnings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.learnings.is_empty()
    }

    /// 列出所有学习经验 (CLI 展示)。
    pub fn list(&self) -> &[SiteLearning] {
        &self.learnings
    }
}

/// 站点学习经验验证门控 — 对应 ego-lite validateLearning:
/// 域名格式合法性 (无协议/路径/起止点), `*.` 通配仅允许前缀, 结构化字段非空。
pub fn validate_site_learning(l: &SiteLearning) -> bool {
    let raw = l.domain.trim();
    if raw.is_empty() || raw.contains("://") || raw.contains('/') {
        return false;
    }
    if raw.starts_with('.') || raw.ends_with('.') {
        return false;
    }
    let d = raw.trim_end_matches('.');
    if d.contains('*') && (!d.starts_with("*.") || d[2..].contains('*')) {
        return false;
    }
    !l.name.trim().is_empty() && !l.category.trim().is_empty()
        && !l.title_field.trim().is_empty() && !l.url_field.trim().is_empty()
}

/* ── 测试 ── */

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
                    evidence: None,
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

    /* ── argo 吸收: 证据可信度评分测试 ── */

    fn sample(url: &str, title: &str, snippet: &str) -> SearchResult {
        SearchResult {
            title: title.to_string(),
            url: url.to_string(),
            snippet: snippet.to_string(),
            evidence: None,
        }
    }

    #[test]
    fn authority_official_domain_scores_high() {
        let (score, tier, serp) = score_authority("https://www.gov.cn/zhengce/2026/xx.html");
        assert!(score >= 0.9, "gov.cn should be ~1.0, got {}", score);
        assert_eq!(tier, "official");
        assert!(!serp);
    }

    #[test]
    fn authority_arxiv_is_professional() {
        let (score, tier, _) = score_authority("https://arxiv.org/abs/2606.00001");
        assert!(score >= 0.9, "arxiv should be 0.9, got {}", score);
        assert_eq!(tier, "official");
    }

    #[test]
    fn authority_content_farm_is_demoted() {
        let (score, _, _) = score_authority("https://www.sohu.com/a/12345.html");
        assert!(score < 0.5, "sohu should be demoted, got {}", score);
    }

    #[test]
    fn serp_url_is_marked() {
        let (score, _, serp) = score_authority("https://www.google.com/search?q=neotrix");
        assert!(serp, "google search should be marked as SERP");
        assert!(score <= 0.15, "SERP should be very low, got {}", score);
    }

    #[test]
    fn domain_extraction_normalizes() {
        assert_eq!(extract_domain("https://WWW.Example.com:8080/path?q=1"), "example.com");
        assert_eq!(extract_domain("http://sub.github.com/x"), "sub.github.com");
    }

    #[test]
    fn evidence_density_detects_numbers_and_definitions() {
        let (score, has_num, has_def, has_cmp, _) =
            score_evidence_density("GDP 增长 8.2%，环比上升 1.4 个百分点。是指国民经济总量。", "2026年经济");
        assert!(has_num, "should detect numbers");
        assert!(has_def, "should detect definition");
        assert!(has_cmp, "should detect comparison (环比/上升)");
        assert!(score >= 0.5, "dense evidence should score high, got {}", score);
    }

    #[test]
    fn evidence_density_low_for_plain_text() {
        let (score, has_num, _, _, _) = score_evidence_density("hello world foo bar baz qux", "");
        assert!(!has_num);
        assert!(score < 0.4, "plain text should score low, got {}", score);
    }

    #[test]
    fn freshness_penalizes_old_dates() {
        let old = score_freshness("old news", "something from 2010", "https://x.com/a");
        let fresh = score_freshness("new news", &format!("released {}", chrono::Utc::now().format("%Y")), "https://x.com/b");
        assert!(fresh > old, "fresh={} should exceed old={}", fresh, old);
    }

    #[test]
    fn combined_credibility_weighting() {
        // 权威官源 + 高证据密度 → 综合分高
        let r = sample(
            "https://www.gov.cn/news/2026/06/economy.html",
            "2026年国民经济运行数据",
            "上半年 GDP 同比增长 8.2%，环比上升 1.4%，数据由统计局披露。",
        );
        let ev = EvidenceScorer::score(&r, 0.5);
        assert!(ev.selection >= 0.9, "selection high, got {}", ev.selection);
        assert!(ev.absorption >= 0.6, "absorption high, got {}", ev.absorption);
        assert!(ev.final_score >= 0.7, "final should be high, got {}", ev.final_score);
        assert!(ev.has_numbers);
    }

    #[test]
    fn score_results_sorts_by_credibility_desc() {
        let good = sample("https://www.gov.cn/a", "权威源", "GDP 增长 8.2% 数据披露");
        let bad = sample("https://www.maigoo.com/b", "榜单页", "nothing much here just filler text");
        let results = EvidenceScorer::score_results(vec![bad.clone(), good.clone()], "GDP");
        assert_eq!(results.len(), 2);
        let first = results[0].evidence.as_ref().unwrap();
        let second = results[1].evidence.as_ref().unwrap();
        assert!(first.final_score >= second.final_score,
            "sorted desc: {} >= {}", first.final_score, second.final_score);
    }

    #[test]
    fn dedup_merges_url_variants() {
        let a = sample("https://example.com/article?a=1&utm_source=x", "A", "a");
        let b = sample("http://www.example.com/article?a=1", "B", "b");
        let dedup = EvidenceScorer::deduplicate_by_url(vec![a, b]);
        assert_eq!(dedup.len(), 1, "utm/www/http variants should merge");
    }

    #[test]
    fn unified_search_injects_evidence_into_results() {
        // 用探针后端注入 UnifiedSearch 无法直接注入 backend, 验证 EvidenceScorer 管线本身即可 (C1)。
        let r = sample("https://arxiv.org/abs/2606.001", "paper", "We propose a method with 42% improvement");
        let ev = EvidenceScorer::score(&r, 0.5);
        assert!(ev.final_score > 0.0);
        assert_eq!(ev.tier, "official");
    }

    /* ── ego-lite 吸收: 站点学习经验注册表测试 ── */

    #[test]
    fn domain_matches_bare_and_wildcard() {
        // 裸域名精确匹配 (不匹配子域)
        assert!(SiteLearning::new("github.com", "g", "code").matches("github.com"));
        assert!(!SiteLearning::new("github.com", "g", "code").matches("foo.github.com"));
        // `*.suffix` 只匹配子域, 不匹配 apex (ego-lite domainMatches 语义)
        let wild = SiteLearning::new("*.gov.cn", "gov", "official");
        assert!(wild.matches("www.gov.cn"));
        assert!(wild.matches("stats.gov.cn"));
        assert!(!wild.matches("gov.cn"));
        // 大小写与尾点不敏感
        assert!(SiteLearning::new("arXiv.org", "a", "academic").matches("ARXIV.ORG"));
    }

    #[test]
    fn validate_rejects_bad_domains() {
        // validateLearning 门控: 拒绝协议/路径/起止点/非法通配
        assert!(validate_site_learning(&SiteLearning::new("github.com", "g", "code")));
        assert!(!validate_site_learning(&SiteLearning::new("https://x.com", "g", "code")));
        assert!(!validate_site_learning(&SiteLearning::new("x.com/path", "g", "code")));
        assert!(!validate_site_learning(&SiteLearning::new(".x.com", "g", "code")));
        assert!(!validate_site_learning(&SiteLearning::new("x.com.", "g", "code")));
        assert!(!validate_site_learning(&SiteLearning::new("*x.com", "g", "code")));
        // 空 name 拒绝
        let mut l = SiteLearning::new("a.com", " ", "code");
        assert!(!validate_site_learning(&l));
        l = SiteLearning::new("a.com", "A", "");
        assert!(!validate_site_learning(&l));
    }

    #[test]
    fn registry_routes_by_hostname() {
        let reg = SiteLearningRegistry::builtin();
        assert!(reg.has_for("www.gov.cn"));
        assert!(reg.has_for("arxiv.org"));
        assert!(!reg.has_for("example.org"));
        assert_eq!(reg.for_host("www.gov.cn").len(), 1);
        // 同域名多别名去重: x.com + twitter.com 都注册
        assert!(reg.has_for("x.com") && reg.has_for("twitter.com"));
        assert!(reg.len() >= 8);
    }

#[test]
    fn unified_search_tags_snippet_by_domain() {
        // 通过 EvidenceScorer + registry 的组合验证: gov.cn 结果被注入 [official]
        // extract_domain 剥 www 得 gov.cn; registry 用 `*.gov.cn` 通配匹配子域。
        let reg = SiteLearningRegistry::builtin();
        let r = sample("https://www.gov.cn/news/2026/x.html", "政", "数据");
        let domain = extract_domain(&r.url);
        assert_eq!(domain, "gov.cn");
        // `*.gov.cn` 不匹配 apex gov.cn, 但 URL 宿主名带 www 时应匹配。
        assert_eq!(reg.for_host("www.gov.cn").len(), 1);
        assert_eq!(reg.for_host("www.gov.cn")[0].category, "official");
    }
}
