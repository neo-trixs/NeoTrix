#![forbid(unsafe_code)]

//! URLhaus + CISA KEV 情报接线 — Wave6 H4 第6个情报工具 (P1, 10工具中第6个)
//!
//! - **URLhaus**: abuse.ch 恶意URL feed (`urlhaus-api.abuse.ch`, 免费无key, POST)
//! - **CISA KEV**: 已知被利用漏洞 (`www.cisa.gov`, 免费无key, JSON feed)
//! - **强化节点**: `nt_world_search::Ordered Backend Router` (R-P42) — 新增 `UrlhausBackend`
//! - **入库**: `nt_memory_kb::KnowledgeBase` (External, domain=urlhaus / cisa-kev)
//! - **Egress**: `nt_shield_sandbox::INTEL_URLHAUS_HOST` / `INTEL_CISA_KEV_HOST` allow (deny-wins)
//! - **测试**: fixture 驱动，无真实网络依赖，CI 稳定

use serde::{Deserialize, Serialize};

// ── URLhaus 数据模型 ───────────────────────────────────────────

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UrlhausResponse {
    #[serde(default)]
    pub url_count: String,
    #[serde(default)]
    pub result: String,
    #[serde(default)]
    pub urls: Vec<UrlhausUrl>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UrlhausUrl {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub url_status: String,
    #[serde(default)]
    pub threat: String,
    #[serde(default)]
    pub md5_hash: String,
    #[serde(default)]
    pub sha256_hash: String,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub date_added: String,
    #[serde(default)]
    pub reporter: String,
}

/// 内部标准化结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlhausEvent {
    pub url: String,
    pub url_status: String,
    pub threat: String,
    pub md5_hash: String,
    pub sha256_hash: String,
    pub host: String,
    pub date_added: String,
    pub reporter: String,
}

// ── CISA KEV 数据模型 ───────────────────────────────────────────

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CisaKevResponse {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub catalog_version: String,
    #[serde(default)]
    pub vulnerabilities: Vec<CisaKevVuln>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CisaKevVuln {
    #[serde(default, alias = "cveID")]
    pub cve_id: String,
    #[serde(default)]
    pub vendor_project: String,
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub vulnerability_name: String,
    #[serde(default)]
    pub date_added: String,
    #[serde(default)]
    pub short_description: String,
    #[serde(default)]
    pub required_action: String,
    #[serde(default)]
    pub due_date: String,
    #[serde(default)]
    pub known_ransomware_campaign_use: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CisaKevEvent {
    pub cve_id: String,
    pub vendor_project: String,
    pub product: String,
    pub vulnerability_name: String,
    pub date_added: String,
    pub short_description: String,
    pub required_action: String,
    pub due_date: String,
    pub known_ransomware_campaign_use: String,
}

// ── Fetch 层 ────────────────────────────────────────────────────

pub struct UrlhausFetcher {
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for UrlhausFetcher {
    fn default() -> Self { Self::new() }
}

impl UrlhausFetcher {
    pub fn new() -> Self { Self { client: std::sync::OnceLock::new() } }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (URLhaus intel-watch; https://github.com/neotrix)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    pub fn build_url(&self) -> String {
        "https://urlhaus-api.abuse.ch/v1/url/recent/".to_string()
    }

    pub fn fetch(&self) -> Result<Vec<UrlhausEvent>, String> {
        let resp = self.client().post(self.build_url()).form(&[("limit", "50")]).send()
            .map_err(|e| format!("URLhaus request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("URLhaus returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("URLhaus read body: {}", e))?;
        Self::parse_json(&text)
    }

    pub fn parse_json(json: &str) -> Result<Vec<UrlhausEvent>, String> {
        let resp: UrlhausResponse = serde_json::from_str(json).map_err(|e| format!("URLhaus parse failed: {}", e))?;
        Ok(resp.urls.into_iter().map(|u| UrlhausEvent {
            url: u.url, url_status: u.url_status, threat: u.threat,
            md5_hash: u.md5_hash, sha256_hash: u.sha256_hash, host: u.host,
            date_added: u.date_added, reporter: u.reporter,
        }).collect())
    }

    pub fn ingest_from_json(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        json: &str,
    ) -> Result<UrlhausIngestReport, String> {
        let events = Self::parse_json(json)?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest_events(
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        events: &[UrlhausEvent],
    ) -> Result<UrlhausIngestReport, String> {
        let mut report = UrlhausIngestReport { events_fetched: events.len(), ..Default::default() };
        for evt in events {
            if evt.url.trim().is_empty() { report.errors.push("skip empty url".into()); continue; }
            let summary = format!("{} | {} | {}", evt.threat, evt.url_status, evt.host);
            let existing = kb.find_node_by_url(&evt.url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb.insert_or_get_node(&evt.url, crate::core::nt_core_kb_types::NodeType::External, Some(&summary), Some(&evt.url), Some("urlhaus"))
                .map_err(|e| format!("KB ingest failed for {}: {}", evt.url, e))?;
            if is_new { report.nodes_created += 1; } else { report.nodes_reused += 1; }
        }
        Ok(report)
    }

    pub fn to_search_results(events: &[UrlhausEvent]) -> Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult> {
        events.iter().map(|e| crate::l2_perception::nt_world::nt_world_search::SearchResult {
            title: e.url.clone(),
            url: e.url.clone(),
            snippet: format!("{} | {} | host={}", e.threat, e.url_status, e.host),
            evidence: None,
        }).collect()
    }
}

pub struct CisaKevFetcher {
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for CisaKevFetcher {
    fn default() -> Self { Self::new() }
}

impl CisaKevFetcher {
    pub fn new() -> Self { Self { client: std::sync::OnceLock::new() } }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (CISA-KEV intel-watch; https://github.com/neotrix)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    pub fn build_url(&self) -> String {
        "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json".to_string()
    }

    pub fn fetch(&self) -> Result<Vec<CisaKevEvent>, String> {
        let resp = self.client().get(self.build_url()).send().map_err(|e| format!("CISA KEV request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("CISA KEV returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("CISA KEV read body: {}", e))?;
        Self::parse_json(&text)
    }

    pub fn parse_json(json: &str) -> Result<Vec<CisaKevEvent>, String> {
        let resp: CisaKevResponse = serde_json::from_str(json).map_err(|e| format!("CISA KEV parse failed: {}", e))?;
        Ok(resp.vulnerabilities.into_iter().map(|v| CisaKevEvent {
            cve_id: v.cve_id, vendor_project: v.vendor_project, product: v.product,
            vulnerability_name: v.vulnerability_name, date_added: v.date_added,
            short_description: v.short_description, required_action: v.required_action,
            due_date: v.due_date, known_ransomware_campaign_use: v.known_ransomware_campaign_use,
        }).collect())
    }

    pub fn ingest_events(
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        events: &[CisaKevEvent],
    ) -> Result<CisaKevIngestReport, String> {
        let mut report = CisaKevIngestReport { events_fetched: events.len(), ..Default::default() };
        for evt in events {
            if evt.cve_id.trim().is_empty() { report.errors.push("skip empty cve".into()); continue; }
            let url = format!("https://www.cisa.gov/known-exploited-vulnerabilities-catalog?search={}", evt.cve_id);
            let summary = format!("{} {} | {}", evt.vendor_project, evt.product, evt.vulnerability_name);
            let existing = kb.find_node_by_url(&url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb.insert_or_get_node(&evt.cve_id, crate::core::nt_core_kb_types::NodeType::External, Some(&summary), Some(&url), Some("cisa-kev"))
                .map_err(|e| format!("KB ingest failed for {}: {}", evt.cve_id, e))?;
            if is_new { report.nodes_created += 1; } else { report.nodes_reused += 1; }
        }
        Ok(report)
    }

    pub fn to_search_results(events: &[CisaKevEvent]) -> Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult> {
        events.iter().map(|e| crate::l2_perception::nt_world::nt_world_search::SearchResult {
            title: e.cve_id.clone(),
            url: format!("https://www.cisa.gov/known-exploited-vulnerabilities-catalog?search={}", e.cve_id),
            snippet: format!("{} {} | ransomware={}", e.vendor_project, e.product, e.known_ransomware_campaign_use),
            evidence: None,
        }).collect()
    }
}

// ── Egress ───────────────────────────────────────────────────

pub const URLHAUS_HOST: &str = "urlhaus-api.abuse.ch";
pub const CISA_KEV_HOST: &str = "www.cisa.gov";
pub fn urlhaus_egress_rule() -> crate::l1_action::nt_io::nt_shield_sandbox::EgressRule {
    crate::l1_action::nt_io::nt_shield_sandbox::EgressRule::allow(URLHAUS_HOST, "443")
}
pub fn urlhaus_egress_policy() -> crate::l1_action::nt_io::nt_shield_sandbox::EgressPolicy {
    crate::l1_action::nt_io::nt_shield_sandbox::EgressPolicy::new(vec![urlhaus_egress_rule()], false)
}
pub fn cisa_kev_egress_rule() -> crate::l1_action::nt_io::nt_shield_sandbox::EgressRule {
    crate::l1_action::nt_io::nt_shield_sandbox::EgressRule::allow(CISA_KEV_HOST, "443")
}
pub fn cisa_kev_egress_policy() -> crate::l1_action::nt_io::nt_shield_sandbox::EgressPolicy {
    crate::l1_action::nt_io::nt_shield_sandbox::EgressPolicy::new(vec![cisa_kev_egress_rule()], false)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UrlhausIngestReport {
    pub events_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CisaKevIngestReport {
    pub events_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

// ── SearchBackend ────────────────────────────────────────────

pub struct UrlhausBackend {
    fetcher: UrlhausFetcher,
}

impl Default for UrlhausBackend {
    fn default() -> Self { Self { fetcher: UrlhausFetcher::new() } }
}

impl UrlhausBackend {
    pub fn new() -> Self { Self::default() }
}

impl crate::l2_perception::nt_world::nt_world_search::SearchBackend for UrlhausBackend {
    fn name(&self) -> &str { "urlhaus" }
    fn search(&self, _query: &str, count: usize) -> Result<Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult>, String> {
        let events = self.fetcher.fetch()?;
        let limited: Vec<UrlhausEvent> = events.into_iter().take(count).collect();
        Ok(UrlhausFetcher::to_search_results(&limited))
    }
}

// ── Fixture ────────────────────────────────────────────────

pub const URLHAUS_FIXTURE_JSON: &str = r#"{"url_count":"2","result":"ok","urls":[{"url":"http://evil.example/malware.exe","url_status":"online","threat":"malware","md5_hash":"abc123","sha256_hash":"def456","host":"evil.example","date_added":"2024-01-01","reporter":"user1"}]}"#;

pub const CISA_KEV_FIXTURE_JSON: &str = r#"{"title":"Known Exploited Vulnerabilities","catalog_version":"2024.01","vulnerabilities":[{"cveID":"CVE-2024-0001","vendorProject":"Vendor","product":"Product","vulnerabilityName":"RCE","dateAdded":"2024-01-01","shortDescription":"Desc","requiredAction":"Patch","dueDate":"2024-02-01","knownRansomwareCampaignUse":"Known","notes":""}]}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_urlhaus_fixture() {
        let events = UrlhausFetcher::parse_json(URLHAUS_FIXTURE_JSON).expect("parse");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].threat, "malware");
    }

    #[test]
    fn test_parse_cisa_fixture() {
        let events = CisaKevFetcher::parse_json(CISA_KEV_FIXTURE_JSON).expect("parse");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].cve_id, "CVE-2024-0001");
    }

    #[test]
    fn test_build_url() {
        assert_eq!(UrlhausFetcher::new().build_url(), "https://urlhaus-api.abuse.ch/v1/url/recent/");
    }

    #[test]
    fn test_egress_policies() {
        assert!(urlhaus_egress_policy().check("urlhaus-api.abuse.ch", 443));
        assert!(cisa_kev_egress_policy().check("www.cisa.gov", 443));
        assert!(!urlhaus_egress_policy().check("evil.com", 443));
    }

    #[test]
    fn test_ingest_both() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
        let r1 = UrlhausFetcher::new().ingest_from_json(&kb, URLHAUS_FIXTURE_JSON).expect("ingest urlhaus");
        assert_eq!(r1.nodes_created, 1);
        let r2 = CisaKevFetcher::ingest_events(&kb, &CisaKevFetcher::parse_json(CISA_KEV_FIXTURE_JSON).unwrap()).expect("ingest cisa");
        assert_eq!(r2.nodes_created, 1);
    }

    #[test]
    fn test_backend_name() {
        use crate::l2_perception::nt_world::nt_world_search::SearchBackend;
        assert_eq!(SearchBackend::name(&UrlhausBackend::new()), "urlhaus");
    }
}
