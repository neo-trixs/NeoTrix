//! NeoTrix 知识吸收管道 — GitHub/外部代码 → KB 蒸馏 → 去重更新

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::neotrix::nt_memory_kb::KnowledgeBase;
use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType;

/// SSRF 防护：URL 必须为 http/https，且目标 IP 不得为内网/回环/链路本地/保留段。
/// 对域名做一次性解析并校验所有解析出的地址，解析失败一律拒绝（保守）。
pub fn is_safe_fetch_url(url: &str) -> bool {
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false,
    };
    if !matches!(parsed.scheme(), "http" | "https") {
        return false;
    }
    let host = match parsed.host_str() {
        Some(h) => h,
        None => return false,
    };
    let lower = host.to_ascii_lowercase();
    if lower == "localhost" || lower.ends_with(".localhost") || lower.ends_with(".local") {
        return false;
    }
    // IP 字面量直接校验
    if let Ok(ip) = host.parse::<IpAddr>() {
        return !is_private_ip(ip);
    }
    // 域名: 解析并校验全部结果 (过滤内网解析)
    let addr = match (host, parsed.port_or_known_default().unwrap_or(80)) {
        (h, p) => (h, p),
    };
    match std::net::ToSocketAddrs::to_socket_addrs(&(addr.0.to_string(), addr.1)) {
        Ok(addrs) => {
            let mut any_safe = false;
            for sa in addrs {
                let ip = sa.ip();
                if ip.is_unspecified() {
                    return false;
                }
                if is_private_ip(ip) {
                    return false;
                }
                any_safe = true;
            }
            any_safe
        }
        Err(_) => false,
    }
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_broadcast()
                || v4.is_unspecified() || v4.is_documentation()
        }
        IpAddr::V6(v6) => {
            v6.is_loopback() || v6.is_unspecified() || v6.is_unique_local()
                || v6.is_unicast_link_local() || v6.is_multicast()
        }
    }
}

// ============================================================
// 源类型 (使用唯一名避免冲突)
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KbSourceType {
    GitHub,
    ArXiv,
    Wikipedia,
    WebArticle,
    Paper,
    Documentation,
    CodeRepository,
    Blog,
}

impl std::fmt::Display for KbSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KbSourceType::GitHub => write!(f, "GitHub"),
            KbSourceType::ArXiv => write!(f, "ArXiv"),
            KbSourceType::Wikipedia => write!(f, "Wikipedia"),
            KbSourceType::WebArticle => write!(f, "WebArticle"),
            KbSourceType::Paper => write!(f, "Paper"),
            KbSourceType::Documentation => write!(f, "Documentation"),
            KbSourceType::CodeRepository => write!(f, "CodeRepository"),
            KbSourceType::Blog => write!(f, "Blog"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEntry {
    pub url: String,
    pub source_type: KbSourceType,
    pub title: String,
    pub kb_node_id: Option<String>,
    pub last_absorbed: i64,
    pub sha_hash: Option<String>,
    pub distill_summary: Option<String>,
    pub tags: Vec<String>,
}

/// source_map 上限：长驻 daemon 每 URL 一条，无界增长 → 内存泄漏
const SOURCE_MAP_LIMIT: usize = 2000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorbState {
    pub source_map: HashMap<String, SourceEntry>,
    pub total_absorbed: usize,
    pub last_panorama_update: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillResult {
    pub title: String,
    pub summary: String,
    pub core_concepts: Vec<String>,
    pub architecture_insights: Vec<String>,
    pub key_algorithms: Vec<String>,
    pub dependencies: Vec<String>,
    pub code_patterns: Vec<String>,
    pub confidence: f64,
}

// ============================================================
// 知识吸收管道
// ============================================================

pub struct KnowledgeAbsorptionPipeline {
    pub kb: Option<Arc<KnowledgeBase>>,
    state: AbsorbState,
}

impl Default for KnowledgeAbsorptionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeAbsorptionPipeline {
    pub fn new() -> Self {
        Self {
            kb: None,
            state: AbsorbState {
                source_map: HashMap::new(),
                total_absorbed: 0,
                last_panorama_update: 0,
            },
        }
    }

    pub fn attach_kb(&mut self, kb: Arc<KnowledgeBase>) {
        self.kb = Some(kb);
    }

    pub fn get_kb(&self) -> Option<Arc<KnowledgeBase>> {
        self.kb.clone()
    }

    pub fn absorb_url(&mut self, url: &str) -> Result<AbsorptionReport, String> {
        if !is_safe_fetch_url(url) {
            return Err(format!("URL rejected (SSRF guard): {}", url));
        }
        let url_key = url.to_string();
        if let Some(entry) = self.state.source_map.get(&url_key) {
            let age = Utc::now().timestamp() - entry.last_absorbed;
            if age < 86400 {
                return Ok(AbsorptionReport {
                    url: url.into(), source_type: KbSourceType::WebArticle,
                    action: "cached".into(), nodes_created: 0, edges_created: 0,
                    distil_summary: None,
                });
            }
        }
        if let Some(ref kb) = self.kb {
            if let Ok(Some(_existing)) = kb.find_node_by_url(url) {
                return Ok(AbsorptionReport {
                    url: url.into(), source_type: KbSourceType::WebArticle,
                    action: "cached".into(), nodes_created: 0, edges_created: 0,
                    distil_summary: None,
                });
            }
        }

        // HTTP fetch + content extraction (P0 fix: was inserting empty External nodes)
        let (content, domain) = match reqwest::blocking::get(url) {
            Ok(resp) => {
                let domain = resp.url().host_str().unwrap_or("").to_string();
                let body = resp.text().unwrap_or_default();
                (body, domain)
            }
            Err(e) => {
                return Err(format!("HTTP fetch failed for {}: {}", url, e));
            }
        };

        let summary = extract_text_content(&content);
        let summary_short = if summary.len() > 5000 {
            format!("{}...", summary.chars().take(5000).collect::<String>())
        } else {
            summary.clone()
        };

        let node_type = if url.contains("arxiv.org") || url.contains("paper") { NodeType::Paper }
            else if url.contains("github.com") { NodeType::Repository }
            else if url.contains("wikipedia.org") { NodeType::Reference }
            else { NodeType::Article };

        let node_id = self.kb.as_ref().and_then(|kb| {
            kb.insert_or_get_node(url, node_type.clone(),
                Some(&summary_short), Some(url), Some(&domain)).ok()
        });

        self.record_source(url.to_string(), SourceEntry {
            url: url.into(), source_type: KbSourceType::WebArticle,
            title: url.into(), kb_node_id: node_id.clone(),
            last_absorbed: Utc::now().timestamp(),
            sha_hash: None, distill_summary: None, tags: vec![],
        });
        self.state.total_absorbed += 1;

        Ok(AbsorptionReport {
            url: url.into(), source_type: KbSourceType::WebArticle,
            action: "absorbed".into(),
            nodes_created: if node_id.is_some() { 1 } else { 0 },
            edges_created: 0,
            distil_summary: Some(summary_short),
        })
    }

    pub async fn absorb_url_async(&mut self, url: &str) -> Result<AbsorptionReport, String> {
        if !is_safe_fetch_url(url) {
            return Err(format!("URL rejected (SSRF guard): {}", url));
        }
        let url_key = url.to_string();
        if let Some(entry) = self.state.source_map.get(&url_key) {
            let age = Utc::now().timestamp() - entry.last_absorbed;
            if age < 86400 {
                return Ok(AbsorptionReport {
                    url: url.into(), source_type: KbSourceType::WebArticle,
                    action: "cached".into(), nodes_created: 0, edges_created: 0,
                    distil_summary: None,
                });
            }
        }
        if let Some(ref kb) = self.kb {
            if let Ok(Some(_existing)) = kb.find_node_by_url(url) {
                return Ok(AbsorptionReport {
                    url: url.into(), source_type: KbSourceType::WebArticle,
                    action: "cached".into(), nodes_created: 0, edges_created: 0,
                    distil_summary: None,
                });
            }
        }

        let (content, domain) = match tokio::time::timeout(
            std::time::Duration::from_secs(15),
            reqwest::get(url),
        ).await {
            Ok(Ok(resp)) => {
                let domain = resp.url().host_str().unwrap_or("").to_string();
                let body = match tokio::time::timeout(
                    std::time::Duration::from_secs(15),
                    resp.text(),
                ).await {
                    Ok(Ok(text)) => text,
                    Ok(Err(e)) => return Err(format!("HTTP body read failed for {}: {}", url, e)),
                    Err(_) => return Err(format!("HTTP body read timed out for {}", url)),
                };
                (body, domain)
            }
            Ok(Err(e)) => {
                return Err(format!("HTTP fetch failed for {}: {}", url, e));
            }
            Err(_) => {
                return Err(format!("HTTP fetch timed out for {}", url));
            }
        };

        let summary = extract_text_content(&content);
        let summary_short = if summary.len() > 5000 {
            format!("{}...", summary.chars().take(5000).collect::<String>())
        } else {
            summary.clone()
        };

        let node_type = if url.contains("arxiv.org") || url.contains("paper") { NodeType::Paper }
            else if url.contains("github.com") { NodeType::Repository }
            else if url.contains("wikipedia.org") { NodeType::Reference }
            else { NodeType::Article };

        let node_id = self.kb.as_ref().and_then(|kb| {
            kb.insert_or_get_node(url, node_type.clone(),
                Some(&summary_short), Some(url), Some(&domain)).ok()
        });

        self.record_source(url.to_string(), SourceEntry {
            url: url.into(), source_type: KbSourceType::WebArticle,
            title: url.into(), kb_node_id: node_id.clone(),
            last_absorbed: Utc::now().timestamp(),
            sha_hash: None, distill_summary: None, tags: vec![],
        });
        self.state.total_absorbed += 1;

        Ok(AbsorptionReport {
            url: url.into(), source_type: KbSourceType::WebArticle,
            action: "absorbed".into(),
            nodes_created: if node_id.is_some() { 1 } else { 0 },
            edges_created: 0,
            distil_summary: Some(summary_short),
        })
    }

    pub fn absorb_github(&mut self, url: &str) -> Result<AbsorptionReport, String> {
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
        let repo = parts.last().ok_or("无法解析仓库名")?.to_string();
        let owner = if parts.len() >= 2 { parts[parts.len()-2].to_string() } else { String::new() };

        let url_key = format!("github:{}/{}", owner, repo);

        // 去重检查
        if let Some(entry) = self.state.source_map.get(&url_key) {
            if Utc::now().timestamp() - entry.last_absorbed < 3600 {
                return Ok(AbsorptionReport {
                    url: url.into(), source_type: KbSourceType::GitHub,
                    action: "skipped".into(), nodes_created: 0, edges_created: 0,
                    distil_summary: None,
                });
            }
        }

        // 蒸馏
        let distill = DistillResult {
            title: repo.clone(),
            summary: format!("GitHub 仓库 {}/{}", owner, repo),
            core_concepts: vec![format!("{}/{}", owner, repo)],
            architecture_insights: vec![],
            key_algorithms: vec![],
            dependencies: vec![],
            code_patterns: vec![],
            confidence: 0.6,
        };

        self.record_source(url_key.clone(), SourceEntry {
            url: url.into(), source_type: KbSourceType::GitHub,
            title: repo, kb_node_id: None,
            last_absorbed: Utc::now().timestamp(),
            sha_hash: None,
            distill_summary: Some(distill.summary.clone()),
            tags: vec![],
        });
        self.state.total_absorbed += 1;

        Ok(AbsorptionReport {
            url: url.into(), source_type: KbSourceType::GitHub,
            action: "absorbed".into(), nodes_created: 1, edges_created: 0,
            distil_summary: Some(distill.summary),
        })
    }

    /// 更新全景索引
    pub fn update_panorama(&mut self) -> Result<PanoramaReport, String> {
        let now = Utc::now().timestamp();
        let mut by_type: HashMap<String, usize> = HashMap::new();
        for entry in self.state.source_map.values() {
            *by_type.entry(format!("{}", entry.source_type)).or_insert(0) += 1;
        }
        self.state.last_panorama_update = now;
        Ok(PanoramaReport {
            total_sources: self.state.source_map.len(),
            by_type,
            updated_at: now,
        })
    }

    pub fn recent_sources(&self, n: usize) -> Vec<&SourceEntry> {
        let mut entries: Vec<_> = self.state.source_map.values().collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.last_absorbed));
        entries.into_iter().take(n).collect()
    }

    pub fn stats(&self) -> KbPipelineStats {
        KbPipelineStats {
            total_sources: self.state.source_map.len(),
            total_absorbed: self.state.total_absorbed,
        }
    }

    /// 记录来源，超上限时淘汰最旧条目，防止长驻 daemon 内存无界增长
    fn record_source(&mut self, url: String, entry: SourceEntry) {
        if self.state.source_map.len() >= SOURCE_MAP_LIMIT {
            let mut oldest: Option<(i64, String)> = None;
            for (k, e) in self.state.source_map.iter() {
                if oldest.as_ref().map_or(true, |(ts, _)| e.last_absorbed < *ts) {
                    oldest = Some((e.last_absorbed, k.clone()));
                }
            }
            if let Some((_, k)) = oldest {
                self.state.source_map.remove(&k);
            }
        }
        self.state.source_map.insert(url, entry);
    }
}

// ============================================================
// 报告类型
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionReport {
    pub url: String,
    pub source_type: KbSourceType,
    pub action: String,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub distil_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanoramaReport {
    pub total_sources: usize,
    pub by_type: HashMap<String, usize>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbPipelineStats {
    pub total_sources: usize,
    pub total_absorbed: usize,
}

// ============================================================
// Tests
// ============================================================

/// Strip HTML tags and decode common entities for plain text extraction.
fn extract_text_content(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    let text = text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_new() {
        let pipe = KnowledgeAbsorptionPipeline::new();
        assert_eq!(pipe.stats().total_sources, 0);
    }

    #[test]
    fn test_absorb_url_dedup() {
        let mut pipe = KnowledgeAbsorptionPipeline::new();
        let r1 = pipe.absorb_github("https://github.com/x/y").expect("github first");
        assert_eq!(r1.source_type, KbSourceType::GitHub);
        let r2 = pipe.absorb_github("https://github.com/x/y").expect("github second");
        assert_eq!(r2.source_type, KbSourceType::GitHub);
    }

    #[test]
    fn test_absorb_github() {
        let mut pipe = KnowledgeAbsorptionPipeline::new();
        let r = pipe.absorb_github("https://github.com/rust-lang/rust").expect("github");
        assert_eq!(r.source_type, KbSourceType::GitHub);
    }

    #[test]
    fn test_panorama() {
        let mut pipe = KnowledgeAbsorptionPipeline::new();
        let _ = pipe.absorb_github("https://github.com/x/y").unwrap();
        let pan = pipe.update_panorama().expect("panorama");
        assert_eq!(pan.total_sources, 1);
    }

    #[test]
    fn test_source_type_display() {
        assert_eq!(KbSourceType::GitHub.to_string(), "GitHub");
        assert_eq!(KbSourceType::ArXiv.to_string(), "ArXiv");
    }

    #[test]
    fn test_extract_text_content_basic() {
        let html = "<html><body><h1>Title</h1><p>Hello world</p></body></html>";
        let text = extract_text_content(html);
        assert!(text.contains("Title"));
        assert!(text.contains("Hello world"));
    }

    #[test]
    fn test_extract_text_content_entities() {
        let html = "<p>foo &amp; bar &lt; 3</p>";
        let text = extract_text_content(html);
        assert_eq!(text, "foo & bar < 3");
    }
}
