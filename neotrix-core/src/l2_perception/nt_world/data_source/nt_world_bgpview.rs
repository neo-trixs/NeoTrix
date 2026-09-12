#![forbid(unsafe_code)]

//! BGPview.io Network Intelligence 接线 — 外部吸收批次 (awesome-osint-arsenal)
//!
//! - **数据源**: BGPview.io 开放 BGP/ASN/路由情报 (`api.bgpview.io`, 免费无key, JSON)
//! - **端点**: `https://api.bgpview.io/search?query=` (通用检索 ASN/IP/Org)
//! - **强化节点**: `nt_world_search::Ordered Backend Router` (R-P42) — 新增 `BgpviewBackend`
//! - **入库**: `nt_memory_kb::KnowledgeBase` (External, domain=bgpview)
//! - **Egress**: `nt_shield_sandbox::INTEL_BGPVIEW_HOST` allow (deny-wins)
//! - **测试**: fixture 驱动，无真实网络依赖，CI 稳定

use serde::{Deserialize, Serialize};

/// 最小 percent-encode (空格 + 保留字符)，避免引入额外 crate。
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

// ── BGPview 数据模型 ───────────────────────────────────────────

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct _BgpviewSearchResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub data: _BgpviewSearchData,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct _BgpviewSearchData {
    #[serde(default)]
    pub results: Vec<_BgpviewResult>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct _BgpviewResult {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub asn: u32,
    #[serde(default)]
    pub ip: String,
    #[serde(default)]
    pub country_code: String,
}

fn result_to_search(r: &_BgpviewResult) -> crate::l2_perception::nt_world::nt_world_search::SearchResult {
    let title = if !r.name.is_empty() {
        format!("[{}] {}", r.r#type, r.name)
    } else {
        format!("[{}] {}", r.r#type, r.ip)
    };
    let snippet = match r.r#type.as_str() {
        "autonomous_system" => format!("ASN {} — {}", r.asn, r.description),
        "ip" => format!("IP {} — {}", r.ip, r.description),
        _ => format!("{} {}", r.country_code, r.description),
    };
    crate::l2_perception::nt_world::nt_world_search::SearchResult {
        title,
        url: format!("https://bgpview.io/search?query={}", r.name),
        snippet,
        evidence: None,
    }
}

// ── Fetch 层 ────────────────────────────────────────────────────

pub struct BgpviewFetcher {
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for BgpviewFetcher {
    fn default() -> Self { Self::new() }
}

impl BgpviewFetcher {
    pub fn new() -> Self {
        Self { client: std::sync::OnceLock::new() }
    }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (bgpview intel; https://github.com/neotrix)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    pub fn build_url(&self, query: &str) -> String {
        format!("https://api.bgpview.io/search?query={}", percent_encode(query))
    }

    pub fn fetch(&self, query: &str) -> Result<Vec<_BgpviewResult>, String> {
        let resp = self.client().get(self.build_url(query)).send()
            .map_err(|e| format!("bgpview request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("bgpview returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("bgpview read body: {}", e))?;
        Self::parse_json(&text)
    }

    pub fn parse_json(json: &str) -> Result<Vec<_BgpviewResult>, String> {
        let resp: _BgpviewSearchResponse = serde_json::from_str(json)
            .map_err(|e| format!("bgpview parse failed: {}", e))?;
        Ok(resp.data.results)
    }

    pub fn ingest_from_json(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        json: &str,
    ) -> Result<_BgpviewIngestReport, String> {
        let results = Self::parse_json(json)?;
        let mut report = _BgpviewIngestReport { results_fetched: results.len(), ..Default::default() };
        for r in &results {
            let key = if r.asn != 0 { format!("AS{}", r.asn) } else { r.ip.clone() };
            if key.trim().is_empty() { report.errors.push("skip empty key".into()); continue; }
            let summary = format!("[{}] {} | {}", r.r#type, r.name, r.description);
            let url = format!("https://bgpview.io/search?query={}", key);
            let existing = kb.find_node_by_url(&url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb.insert_or_get_node(&key, crate::core::nt_core_kb_types::NodeType::External, Some(&summary), Some(&url), Some("bgpview"))
                .map_err(|e| format!("KB ingest failed for {}: {}", key, e))?;
            if is_new { report.nodes_created += 1; } else { report.nodes_reused += 1; }
        }
        Ok(report)
    }

    pub fn to_search_results(results: &[_BgpviewResult]) -> Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult> {
        results.iter().map(result_to_search).collect()
    }
}

// ── Egress ───────────────────────────────────────────────────

pub const BGPVIEW_HOST: &str = "api.bgpview.io";
pub fn bgpview_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(BGPVIEW_HOST, "443")
}
pub fn bgpview_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![bgpview_egress_rule()], false)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct _BgpviewIngestReport {
    pub results_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

// ── SearchBackend ────────────────────────────────────────────

pub struct BgpviewBackend {
    fetcher: BgpviewFetcher,
}

impl Default for BgpviewBackend {
    fn default() -> Self { Self { fetcher: BgpviewFetcher::default() } }
}

impl BgpviewBackend {
    pub fn new() -> Self { Self::default() }
}

impl crate::l2_perception::nt_world::nt_world_search::SearchBackend for BgpviewBackend {
    fn name(&self) -> &str { "bgpview" }
    fn search(&self, query: &str, count: usize) -> Result<Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult>, String> {
        let results = self.fetcher.fetch(query)?;
        let limited: Vec<_BgpviewResult> = results.into_iter().take(count).collect();
        Ok(BgpviewFetcher::to_search_results(&limited))
    }
}

// ── Fixture ────────────────────────────────────────────────

pub const BGPVIEW_FIXTURE_JSON: &str = r#"{"status":"ok","data":{"results":[{"type":"autonomous_system","name":"CLOUDFLARENET","description":"Cloudflare, Inc.","asn":13335,"ip":"","country_code":"US"},{"type":"ip","name":"","description":"Cloudflare Los Angeles","asn":0,"ip":"1.1.1.1","country_code":"US"}]}}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture() {
        let r = BgpviewFetcher::parse_json(BGPVIEW_FIXTURE_JSON).expect("parse");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].asn, 13335);
        assert_eq!(r[1].ip, "1.1.1.1");
    }

    #[test]
    fn test_build_url() {
        assert_eq!(BgpviewFetcher::new().build_url("1.1.1.1"), "https://api.bgpview.io/search?query=1.1.1.1");
    }

    #[test]
    fn test_egress_policy() {
        let policy = bgpview_egress_policy();
        assert!(policy.check("api.bgpview.io", 443));
        assert!(!policy.check("evil.com", 443));
    }

    #[test]
    fn test_to_search_results() {
        let r = BgpviewFetcher::parse_json(BGPVIEW_FIXTURE_JSON).expect("parse");
        let sr = BgpviewFetcher::to_search_results(&r);
        assert_eq!(sr.len(), 2);
        assert!(sr[0].title.contains("AS13335") || sr[0].title.contains("CLOUDFLARENET"));
    }

    #[test]
    fn test_ingest_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
        let report = BgpviewFetcher::new().ingest_from_json(&kb, BGPVIEW_FIXTURE_JSON).expect("ingest");
        assert_eq!(report.nodes_created, 2);
    }
}
