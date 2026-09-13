#![forbid(unsafe_code)]

//! OpenCorporates Corporate Registry 接线 — 外部吸收批次 (awesome-osint-arsenal)
//!
//! - 数据源: OpenCorporates 全球公司注册库 (api.opencorporates.com, 基础检索免费无key, JSON)
//! - 端点: https://api.opencorporates.com/v0.9/companies/search?q=<query>
//! - 强化节点: nt_world_search::Ordered Backend Router (R-P42) — 新增 OpencorporatesBackend
//!   与现有 OFAC/UCDP 企业/冲突情报互补 (R-P42 强化现有 nt_world 情报节点, 非平行适配器)
//! - 入库: nt_memory_kb::KnowledgeBase (External, domain=opencorporates)
//! - Egress: nt_shield_sandbox::INTEL_OPENCORPORATES_HOST allow (deny-wins)
//! - 测试: fixture 驱动，无真实网络依赖，CI 稳定

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

// ── OpenCorporates 数据模型 ───────────────────────────────────

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct _OcSearchResponse {
    #[serde(default)]
    pub results: _OcResults,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct _OcResults {
    #[serde(default)]
    pub companies: Vec<_OcCompany>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct _OcCompany {
    #[serde(default)]
    pub company: _OcCompanyDetail,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct _OcCompanyDetail {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub jurisdiction_code: String,
    #[serde(default)]
    pub company_number: String,
    #[serde(default)]
    pub incorporation_date: String,
    #[serde(default)]
    pub registered_address_in_full: String,
}

fn company_to_search(c: &_OcCompanyDetail) -> crate::l2_perception::nt_world::nt_world_search::SearchResult {
    let url = format!(
        "https://opencorporates.com/companies/{}/{}",
        c.jurisdiction_code, c.company_number
    );
    let snippet = format!(
        "jurisdiction={} | incorporated={} | {}",
        c.jurisdiction_code, c.incorporation_date, c.registered_address_in_full
    );
    crate::l2_perception::nt_world::nt_world_search::SearchResult {
        title: c.name.clone(),
        url,
        snippet,
        evidence: None,
    }
}

// ── Fetch 层 ────────────────────────────────────────────────────

pub struct OpencorporatesFetcher {
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for OpencorporatesFetcher {
    fn default() -> Self { Self::new() }
}

impl OpencorporatesFetcher {
    pub fn new() -> Self {
        Self { client: std::sync::OnceLock::new() }
    }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (opencorporates intel; https://github.com/neotrix)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    pub fn build_url(&self, query: &str) -> String {
        format!("https://api.opencorporates.com/v0.9/companies/search?q={}&per_page=10", percent_encode(query))
    }

    pub fn fetch(&self, query: &str) -> Result<Vec<_OcCompanyDetail>, String> {
        let resp = self.client().get(self.build_url(query)).send()
            .map_err(|e| format!("opencorporates request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("opencorporates returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("opencorporates read body: {}", e))?;
        Self::parse_json(&text)
    }

    pub fn parse_json(json: &str) -> Result<Vec<_OcCompanyDetail>, String> {
        let resp: _OcSearchResponse = serde_json::from_str(json)
            .map_err(|e| format!("opencorporates parse failed: {}", e))?;
        Ok(resp.results.companies.into_iter().map(|c| c.company).collect())
    }

    pub fn ingest_from_json(
        &self,
        kb: &dyn super::super::l1_facade::KnowledgeStore,
        json: &str,
    ) -> Result<_OcIngestReport, String> {
        let companies = Self::parse_json(json)?;
        let mut report = _OcIngestReport { companies_fetched: companies.len(), ..Default::default() };
        for c in &companies {
            let key = if !c.company_number.is_empty() {
                format!("{}/{}", c.jurisdiction_code, c.company_number)
            } else {
                c.name.clone()
            };
            if key.trim().is_empty() { report.errors.push("skip empty key".into()); continue; }
            let summary = format!("{} | jurisdiction={} | incorporated={}", c.name, c.jurisdiction_code, c.incorporation_date);
            let url = format!("https://opencorporates.com/companies/{}", key);
            let existing = kb.find_node_by_url(&url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb.insert_or_get_node(&key, crate::core::nt_core_kb_types::NodeType::External, Some(&summary), Some(&url), Some("opencorporates"))
                .map_err(|e| format!("KB ingest failed for {}: {}", key, e))?;
            if is_new { report.nodes_created += 1; } else { report.nodes_reused += 1; }
        }
        Ok(report)
    }

    pub fn to_search_results(companies: &[_OcCompanyDetail]) -> Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult> {
        companies.iter().map(company_to_search).collect()
    }
}

// ── Egress ───────────────────────────────────────────────────

pub const OPENCORPORATES_HOST: &str = "api.opencorporates.com";
pub fn opencorporates_egress_rule() -> super::super::l1_facade::EgressRule {
    super::super::l1_facade::EgressRule::allow(OPENCORPORATES_HOST, "443")
}

pub fn opencorporates_egress_policy() -> super::super::l1_facade::EgressPolicy {
    super::super::l1_facade::EgressPolicy::new(vec![opencorporates_egress_rule()], false)
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct _OcIngestReport {
    pub companies_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

// ── SearchBackend ────────────────────────────────────────────

pub struct OpencorporatesBackend {
    fetcher: OpencorporatesFetcher,
}

impl Default for OpencorporatesBackend {
    fn default() -> Self { Self { fetcher: OpencorporatesFetcher::default() } }
}

impl OpencorporatesBackend {
    pub fn new() -> Self { Self::default() }
}

impl crate::l2_perception::nt_world::nt_world_search::SearchBackend for OpencorporatesBackend {
    fn name(&self) -> &str { "opencorporates" }
    fn search(&self, query: &str, count: usize) -> Result<Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult>, String> {
        let companies = self.fetcher.fetch(query)?;
        let limited: Vec<_OcCompanyDetail> = companies.into_iter().take(count).collect();
        Ok(OpencorporatesFetcher::to_search_results(&limited))
    }
}

// ── Fixture ────────────────────────────────────────────────

pub const OC_FIXTURE_JSON: &str = r#"{"results":{"companies":[{"company":{"name":"NeoCorp Ltd","jurisdiction_code":"gb","company_number":"12345678","incorporation_date":"2020-01-01","registered_address_in_full":"1 Main St, London"}},{"company":{"name":"Trix Industries","jurisdiction_code":"us_de","company_number":"DE-999","incorporation_date":"2019-05-05","registered_address_in_full":"2 Market St, Dover"}}]}}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture() {
        let c = OpencorporatesFetcher::parse_json(OC_FIXTURE_JSON).expect("parse");
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].name, "NeoCorp Ltd");
        assert_eq!(c[1].jurisdiction_code, "us_de");
    }

    #[test]
    fn test_build_url() {
        assert_eq!(OpencorporatesFetcher::new().build_url("NeoCorp"), "https://api.opencorporates.com/v0.9/companies/search?q=NeoCorp&per_page=10");
    }

    #[test]
    fn test_egress_policy() {
        let policy = opencorporates_egress_policy();
        assert!(policy.check("api.opencorporates.com", 443));
        assert!(!policy.check("evil.com", 443));
    }

    #[test]
    fn test_to_search_results() {
        let c = OpencorporatesFetcher::parse_json(OC_FIXTURE_JSON).expect("parse");
        let sr = OpencorporatesFetcher::to_search_results(&c);
        assert_eq!(sr.len(), 2);
        assert!(sr[0].url.contains("gb/12345678"));
    }

    #[test]
    fn test_ingest_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
        let report = OpencorporatesFetcher::new().ingest_from_json(&kb, OC_FIXTURE_JSON).expect("ingest");
        assert_eq!(report.nodes_created, 2);
    }
}
