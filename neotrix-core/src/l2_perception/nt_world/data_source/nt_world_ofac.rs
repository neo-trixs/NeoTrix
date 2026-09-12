#![forbid(unsafe_code)]

//! OFAC Sanctions SDN List 接线 — Wave6 H4 第7个情报工具 (P1, 10工具中第7个)
//!
//! - **数据源**: US Treasury OFAC 特别指定国民 (SDN) 名单 (`www.treasury.gov`, 免费无key, XML)
//! - **端点**: `https://www.treasury.gov/ofac/downloads/sdn.xml`
//! - **解析**: serde_xml_rs 未纳入依赖，采用零依赖精简 XML 字段扫描解析 (R-P42)
//! - **强化节点**: `nt_world_search::Ordered Backend Router` (R-P42) — 新增 `OfacBackend`
//! - **入库**: `nt_memory_kb::KnowledgeBase` (Organization, domain=ofac)
//! - **Egress**: `nt_shield_sandbox::INTEL_OFAC_HOST` allow (deny-wins)
//! - **测试**: fixture 驱动，无真实网络依赖，CI 稳定

use serde::{Deserialize, Serialize};

/// OFAC SDN Entry (serde 模型, 供未来 serde_xml_rs 使用)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct _OfacSdnEntry {
    #[serde(default)]
    pub uid: String,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub sdn_type: String,
    #[serde(default)]
    pub program: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub remarks: Option<String>,
}

/// 内部标准化结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _OfacEntry {
    pub id: String,
    pub name: String,
    pub sdn_type: String,
    pub programs: Vec<String>,
    pub url: String,
}

/// 零依赖精简 XML 字段扫描: 提取 <uid>/<firstName>/<lastName>/<sdnType>/<program>
fn extract_tag<'a>(xml: &'a str, tag: &str, from: usize) -> Option<&'a str> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml[from..].find(&open).map(|i| from + i + open.len())?;
    let end = xml[start..].find(&close).map(|i| start + i)?;
    Some(&xml[start..end])
}

fn parse_sdn_entries(xml: &str) -> Vec<_OfacEntry> {
    let mut out = Vec::new();
    let mut cursor = 0;
    while let Some(entry_start) = xml[cursor..].find("<sdnEntry>") {
        let abs_start = cursor + entry_start;
        let entry_end = match xml[abs_start..].find("</sdnEntry>") {
            Some(i) => abs_start + i,
            None => break,
        };
        let block = &xml[abs_start..entry_end];
        let uid = extract_tag(block, "uid", 0).unwrap_or("").to_string();
        let first = extract_tag(block, "firstName", 0).unwrap_or("").to_string();
        let last = extract_tag(block, "lastName", 0).unwrap_or("").to_string();
        let sdn_type = extract_tag(block, "sdnType", 0).unwrap_or("").to_string();
        let mut programs = Vec::new();
        let mut p = 0;
        while let Some(ps) = block[p..].find("<program>") {
            let pa = p + ps;
            if let Some(pe) = block[pa..].find("</program>") {
                programs.push(block[pa + 9..pa + pe].to_string());
                p = pa + pe;
            } else { break; }
        }
        let name = if first.is_empty() { last.clone() } else { format!("{} {}", first, last) };
        if !uid.is_empty() && !name.is_empty() {
            out.push(_OfacEntry {
                id: uid.clone(),
                name,
                sdn_type,
                programs,
                url: format!("https://sanctionssearch.ofac.treas.gov/SanctionsDSL/id/{}", uid),
            });
        }
        cursor = entry_end + 11;
    }
    out
}

// ── Fetch 层 ────────────────────────────────────────────────────

pub struct OfacFetcher {
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for OfacFetcher {
    fn default() -> Self { Self::new() }
}

impl OfacFetcher {
    pub fn new() -> Self { Self { client: std::sync::OnceLock::new() } }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (OFAC intel-watch; https://github.com/neotrix)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    pub fn build_url(&self) -> String {
        "https://www.treasury.gov/ofac/downloads/sdn.xml".to_string()
    }

    pub fn fetch(&self) -> Result<Vec<_OfacEntry>, String> {
        let resp = self.client().get(self.build_url()).send().map_err(|e| format!("OFAC request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("OFAC returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("OFAC read body: {}", e))?;
        Ok(parse_sdn_entries(&text))
    }

    pub fn parse_xml(xml: &str) -> Result<Vec<_OfacEntry>, String> {
        Ok(parse_sdn_entries(xml))
    }

    pub fn _ingest_from_xml(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        xml: &str,
    ) -> Result<_OfacIngestReport, String> {
        let events = parse_sdn_entries(xml);
        Self::ingest_events(kb, &events)
    }

    pub fn ingest(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
    ) -> Result<_OfacIngestReport, String> {
        let events = self.fetch()?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest_events(
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        events: &[_OfacEntry],
    ) -> Result<_OfacIngestReport, String> {
        let mut report = _OfacIngestReport { events_fetched: events.len(), ..Default::default() };
        for evt in events {
            if evt.id.trim().is_empty() { report.errors.push("skip empty uid".into()); continue; }
            let summary = format!("{} | type={} | programs={}", evt.name, evt.sdn_type, evt.programs.join(","));
            let existing = kb.find_node_by_url(&evt.url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb.insert_or_get_node(&evt.name, crate::core::nt_core_kb_types::NodeType::Organization, Some(&summary), Some(&evt.url), Some("ofac"))
                .map_err(|e| format!("KB ingest failed for {}: {}", evt.id, e))?;
            if is_new { report.nodes_created += 1; } else { report.nodes_reused += 1; }
        }
        Ok(report)
    }

    pub fn to_search_results(events: &[_OfacEntry]) -> Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult> {
        events.iter().map(|e| crate::l2_perception::nt_world::nt_world_search::SearchResult {
            title: e.name.clone(),
            url: e.url.clone(),
            snippet: format!("type={} | programs={}", e.sdn_type, e.programs.join(",")),
            evidence: None,
        }).collect()
    }
}

// ── Egress ───────────────────────────────────────────────────

pub const OFAC_HOST: &str = "www.treasury.gov";
pub fn ofac_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(OFAC_HOST, "443")
}
pub fn ofac_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![ofac_egress_rule()], false)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct _OfacIngestReport {
    pub events_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

// ── SearchBackend ────────────────────────────────────────────

pub struct OfacBackend {
    fetcher: OfacFetcher,
}

impl Default for OfacBackend {
    fn default() -> Self { Self { fetcher: OfacFetcher::new() } }
}

impl OfacBackend {
    pub fn new() -> Self { Self::default() }
}

impl crate::l2_perception::nt_world::nt_world_search::SearchBackend for OfacBackend {
    fn name(&self) -> &str { "ofac" }
    fn search(&self, _query: &str, count: usize) -> Result<Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult>, String> {
        let events = self.fetcher.fetch()?;
        let limited: Vec<_OfacEntry> = events.into_iter().take(count).collect();
        Ok(OfacFetcher::to_search_results(&limited))
    }
}

// ── Fixture ────────────────────────────────────────────────

pub const OFAC_FIXTURE_XML: &str = r#"<sdnList><sdnEntry><uid>1</uid><firstName>John</firstName><lastName>Doe</lastName><sdnType>Individual</sdnType><programList><program>SDGT</program></programList></sdnEntry><sdnEntry><uid>2</uid><lastName>Bad Corp</lastName><sdnType>Entity</sdnType><programList><program>IRAN</program><program>SYRIA</program></programList></sdnEntry></sdnList>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xml_fixture() {
        let entries = parse_sdn_entries(OFAC_FIXTURE_XML);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "John Doe");
        assert_eq!(entries[0].programs, vec!["SDGT".to_string()]);
        assert_eq!(entries[1].programs.len(), 2);
    }

    #[test]
    fn test_build_url() {
        assert_eq!(OfacFetcher::new().build_url(), "https://www.treasury.gov/ofac/downloads/sdn.xml");
    }

    #[test]
    fn test_egress_policy() {
        let policy = ofac_egress_policy();
        assert!(policy.check("www.treasury.gov", 443));
        assert!(!policy.check("evil.com", 443));
    }

    #[test]
    fn test_ingest_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
        let report = OfacFetcher::new()._ingest_from_xml(&kb, OFAC_FIXTURE_XML).expect("ingest");
        assert_eq!(report.nodes_created, 2);
    }

    #[test]
    fn test_backend_name() {
        use crate::l2_perception::nt_world::nt_world_search::SearchBackend;
        assert_eq!(SearchBackend::name(&OfacBackend::new()), "ofac");
    }
}
