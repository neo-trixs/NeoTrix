#![forbid(unsafe_code)]

//! adsb.lol Military Aviation Tracker 接线 — Wave6 H4 第10个情报工具 (P1, 10工具中第10个)
//!
//! - **数据源**: adsb.lol 开放 ADS-B 航班追踪 (`api.adsb.lol`, 免费无key, JSON)
//! - **端点**: `https://api.adsb.lol/v2/point/{lat}/{lon}` (或 `/callsign/{callsign}`)
//! - **强化节点**: `nt_world_search::Ordered Backend Router` (R-P42) — 新增 `AdsbBackend`
//! - **入库**: `nt_memory_kb::KnowledgeBase` (External, domain=adsb)
//! - **Egress**: `nt_shield_sandbox::INTEL_ADSB_HOST` allow (deny-wins)
//! - **测试**: fixture 驱动，无真实网络依赖，CI 稳定

use serde::{Deserialize, Serialize};

// ── adsb.lol 数据模型 ───────────────────────────────────────────

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AdsbResponse {
    #[serde(default)]
    pub ac: Vec<AdsbAircraftRaw>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AdsbAircraftRaw {
    #[serde(default)]
    pub icao: String,
    #[serde(default)]
    pub callsign: String,
    #[serde(default)]
    pub lon: f64,
    #[serde(default)]
    pub lat: f64,
    #[serde(default)]
    pub alt_baro: i64,
    #[serde(default)]
    pub gs: i64,
    #[serde(default)]
    pub track: i64,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub reg: String,
    #[serde(default)]
    pub species: String,
    #[serde(default)]
    pub mil: bool,
    #[serde(default)]
    pub flight: String,
}

/// 内部标准化结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdsbAircraft {
    pub icao: String,
    pub callsign: String,
    pub lat: f64,
    pub lon: f64,
    pub alt_baro: i64,
    pub gs: i64,
    pub track: i64,
    pub r#type: String,
    pub reg: String,
    pub mil: bool,
    pub flight: String,
}

fn extract_aircraft(raw: &AdsbAircraftRaw) -> AdsbAircraft {
    AdsbAircraft {
        icao: raw.icao.clone(),
        callsign: raw.callsign.clone(),
        lat: raw.lat,
        lon: raw.lon,
        alt_baro: raw.alt_baro,
        gs: raw.gs,
        track: raw.track,
        r#type: raw.r#type.clone(),
        reg: raw.reg.clone(),
        mil: raw.mil,
        flight: raw.flight.clone(),
    }
}

// ── Fetch 层 ────────────────────────────────────────────────────

pub struct AdsbFetcher {
    client: std::sync::OnceLock<reqwest::blocking::Client>,
    lat: f64,
    lon: f64,
}

impl Default for AdsbFetcher {
    fn default() -> Self { Self::new(35.6, 139.7) }
}

impl AdsbFetcher {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { client: std::sync::OnceLock::new(), lat, lon }
    }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (adsb.lol intel-watch; https://github.com/neotrix)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    pub fn build_url(&self) -> String {
        format!("https://api.adsb.lol/v2/point/{}/{}", self.lat, self.lon)
    }

    pub fn fetch(&self) -> Result<Vec<AdsbAircraft>, String> {
        let resp = self.client().get(self.build_url()).send().map_err(|e| format!("adsb.lol request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("adsb.lol returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("adsb.lol read body: {}", e))?;
        Self::parse_json(&text)
    }

    pub fn parse_json(json: &str) -> Result<Vec<AdsbAircraft>, String> {
        let resp: AdsbResponse = serde_json::from_str(json).map_err(|e| format!("adsb.lol parse failed: {}", e))?;
        Ok(resp.ac.iter().map(extract_aircraft).collect())
    }

    pub fn ingest_from_json(
        &self,
        kb: &crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase,
        json: &str,
    ) -> Result<AdsbIngestReport, String> {
        let events = Self::parse_json(json)?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest(
        &self,
        kb: &crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase,
    ) -> Result<AdsbIngestReport, String> {
        let events = self.fetch()?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest_events(
        kb: &crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase,
        events: &[AdsbAircraft],
    ) -> Result<AdsbIngestReport, String> {
        let mut report = AdsbIngestReport { events_fetched: events.len(), ..Default::default() };
        for evt in events {
            if evt.icao.trim().is_empty() { report.errors.push("skip empty icao".into()); continue; }
            let url = format!("https://globe.adsb.lol/?icao={}", evt.icao);
            let summary = format!("callsign={} type={} mil={} alt={} gs={}", evt.callsign.trim(), evt.r#type, evt.mil, evt.alt_baro, evt.gs);
            let existing = kb.find_node_by_url(&url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb.insert_or_get_node(&evt.callsign.trim(), crate::core::nt_core_kb_types::NodeType::External, Some(&summary), Some(&url), Some("adsb"))
                .map_err(|e| format!("KB ingest failed for {}: {}", evt.icao, e))?;
            if is_new { report.nodes_created += 1; } else { report.nodes_reused += 1; }
        }
        Ok(report)
    }

    pub fn to_search_results(events: &[AdsbAircraft]) -> Vec<crate::neotrix::l2_world_impl::nt_world_search::SearchResult> {
        events.iter().map(|e| crate::neotrix::l2_world_impl::nt_world_search::SearchResult {
            title: e.callsign.trim().to_string(),
            url: format!("https://globe.adsb.lol/?icao={}", e.icao),
            snippet: format!("icao={} type={} mil={} alt={} gs={}", e.icao, e.r#type, e.mil, e.alt_baro, e.gs),
            evidence: None,
        }).collect()
    }
}

// ── Egress ───────────────────────────────────────────────────

pub const ADSB_HOST: &str = "api.adsb.lol";
pub fn adsb_egress_rule() -> crate::neotrix::l1_body_impl::nt_shield_sandbox::EgressRule {
    crate::neotrix::l1_body_impl::nt_shield_sandbox::EgressRule::allow(ADSB_HOST, "443")
}
pub fn adsb_egress_policy() -> crate::neotrix::l1_body_impl::nt_shield_sandbox::EgressPolicy {
    crate::neotrix::l1_body_impl::nt_shield_sandbox::EgressPolicy::new(vec![adsb_egress_rule()], false)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdsbIngestReport {
    pub events_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

// ── SearchBackend ────────────────────────────────────────────

pub struct AdsbBackend {
    fetcher: AdsbFetcher,
}

impl Default for AdsbBackend {
    fn default() -> Self { Self { fetcher: AdsbFetcher::default() } }
}

impl AdsbBackend {
    pub fn new() -> Self { Self::default() }
    pub fn with_point(lat: f64, lon: f64) -> Self { Self { fetcher: AdsbFetcher::new(lat, lon) } }
}

impl crate::neotrix::l2_world_impl::nt_world_search::SearchBackend for AdsbBackend {
    fn name(&self) -> &str { "adsb" }
    fn search(&self, _query: &str, count: usize) -> Result<Vec<crate::neotrix::l2_world_impl::nt_world_search::SearchResult>, String> {
        let events = self.fetcher.fetch()?;
        let limited: Vec<AdsbAircraft> = events.into_iter().take(count).collect();
        Ok(AdsbFetcher::to_search_results(&limited))
    }
}

// ── Fixture ────────────────────────────────────────────────

pub const ADSB_FIXTURE_JSON: &str = r#"{"ac":[{"icao":"abc123","callsign":"ABCD ","lon":139.7,"lat":35.6,"alt_baro":35000,"gs":450,"track":90,"type":"A320","reg":"N123","species":"A1","mil":false,"flight":"FL123"},{"icao":"def456","callsign":"MILFL ","lon":140.1,"lat":36.0,"alt_baro":40000,"gs":500,"track":180,"type":"C17","reg":"99-0001","species":"A1","mil":true,"flight":"MIL1"}]}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture() {
        let ac = AdsbFetcher::parse_json(ADSB_FIXTURE_JSON).expect("parse");
        assert_eq!(ac.len(), 2);
        assert_eq!(ac[0].icao, "abc123");
        assert!(ac[1].mil);
    }

    #[test]
    fn test_build_url() {
        assert_eq!(AdsbFetcher::new(35.6, 139.7).build_url(), "https://api.adsb.lol/v2/point/35.6/139.7");
    }

    #[test]
    fn test_egress_policy() {
        let policy = adsb_egress_policy();
        assert!(policy.check("api.adsb.lol", 443));
        assert!(!policy.check("evil.com", 443));
    }

    #[test]
    fn test_ingest_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kb = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
        let report = AdsbFetcher::new(35.6, 139.7).ingest_from_json(&kb, ADSB_FIXTURE_JSON).expect("ingest");
        assert_eq!(report.nodes_created, 2);
    }

    #[test]
    fn test_backend_name() {
        use crate::neotrix::l2_world_impl::nt_world_search::SearchBackend;
        assert_eq!(SearchBackend::name(&AdsbBackend::new()), "adsb");
    }
}
