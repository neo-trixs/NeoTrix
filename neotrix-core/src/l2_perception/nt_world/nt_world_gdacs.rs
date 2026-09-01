#![forbid(unsafe_code)]

//! GDACS Disasters API 接线 — Wave6 H4 第4个情报工具 (P1, 10工具中第4个)
//!
//! - **数据源**: GDACS 全球灾难警报协调系统 (`www.gdacs.org`, 免费注册/公开)
//! - **端点**: `https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH` (JSON)
//! - **灾害类型**: EQ(地震)/TC(热带气旋)/FL(洪水)/WF(野火)/DR(干旱)/VO(火山) 等
//! - **强化节点**: `nt_world_search::Ordered Backend Router` (R-P42) — 新增 `GdacsBackend`
//! - **入库**: `nt_memory_kb::KnowledgeBase` → `insert_or_get_node` (Event, domain=gdacs)
//! - **Egress**: `nt_shield_sandbox::INTEL_GDACS_HOST` allow 登记 (deny-wins)
//! - **测试**: fixture 驱动，无真实网络依赖，CI 稳定

use serde::{Deserialize, Serialize};

// ── GDACS JSON 数据模型 ───────────────────────────────────────────

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GdacsRoot {
    #[serde(default)]
    pub result: Vec<GdacsItem>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GdacsItem {
    #[serde(default)]
    pub eventid: String,
    #[serde(default)]
    pub alertlevel: String,
    #[serde(default)]
    pub eventtype: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub lat: String,
    #[serde(default)]
    pub lon: String,
    #[serde(default)]
    pub fromdate: String,
    #[serde(default)]
    pub todate: String,
    #[serde(default)]
    pub severitydata: Option<GdacsSeverity>,
    #[serde(default)]
    pub populationdata: Option<GdacsPopulation>,
    #[serde(default)]
    pub r#type: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GdacsSeverity {
    #[serde(default)]
    pub severity: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GdacsPopulation {
    #[serde(default)]
    pub value: String,
}

/// 内部标准化灾难事件结构 (用于入库)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdacsEvent {
    pub eventid: String,
    pub alert_level: String,
    pub event_type: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub from_date: String,
    pub to_date: String,
    pub severity: String,
    pub population: String,
    pub url: String,
}

fn parse_coord(s: &str) -> f64 {
    s.trim().parse::<f64>().unwrap_or(0.0)
}

fn extract_event(item: &GdacsItem) -> GdacsEvent {
    GdacsEvent {
        eventid: item.eventid.clone(),
        alert_level: item.alertlevel.clone(),
        event_type: item.eventtype.clone(),
        name: item.name.clone(),
        latitude: parse_coord(&item.lat),
        longitude: parse_coord(&item.lon),
        from_date: item.fromdate.clone(),
        to_date: item.todate.clone(),
        severity: item.severitydata.as_ref().map(|s| s.severity.clone()).unwrap_or_default(),
        population: item.populationdata.as_ref().map(|p| p.value.clone()).unwrap_or_default(),
        url: format!("https://www.gdacs.org/report.aspx?eventid={}", item.eventid),
    }
}

// ── Fetch 层 ────────────────────────────────────────────────────

/// GDACS Fetcher — 免费 JSON API，直喂 intel-watch。
pub struct GdacsFetcher {
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for GdacsFetcher {
    fn default() -> Self { Self::new() }
}

impl GdacsFetcher {
    pub fn new() -> Self {
        Self { client: std::sync::OnceLock::new() }
    }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (GDACS intel-watch; https://github.com/neotrix)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    pub fn build_url(&self) -> String {
        "https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH".to_string()
    }

    pub fn fetch(&self) -> Result<Vec<GdacsEvent>, String> {
        let resp = self.client().get(self.build_url()).send().map_err(|e| format!("GDACS request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("GDACS returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("GDACS read body: {}", e))?;
        Self::parse_json(&text)
    }

    pub fn parse_json(json: &str) -> Result<Vec<GdacsEvent>, String> {
        let root: GdacsRoot = serde_json::from_str(json).map_err(|e| format!("GDACS parse failed: {}", e))?;
        Ok(root.result.iter().map(extract_event).collect())
    }

    pub fn ingest_from_json(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        json: &str,
    ) -> Result<GdacsIngestReport, String> {
        let events = Self::parse_json(json)?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
    ) -> Result<GdacsIngestReport, String> {
        let events = self.fetch()?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest_events(
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        events: &[GdacsEvent],
    ) -> Result<GdacsIngestReport, String> {
        let mut report = GdacsIngestReport { events_fetched: events.len(), ..Default::default() };
        for evt in events {
            if evt.eventid.trim().is_empty() {
                report.errors.push(format!("skip empty eventid: {:?}", evt.eventid));
                continue;
            }
            let summary = format!(
                "[{}] {} @ {} | alert={} sev={} pop={}",
                evt.event_type, evt.name, evt.from_date, evt.alert_level, evt.severity, evt.population
            );
            let existing = kb.find_node_by_url(&evt.url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb.insert_or_get_node(&evt.name, crate::core::nt_core_kb_types::NodeType::Event, Some(&summary), Some(&evt.url), Some("gdacs"))
                .map_err(|e| format!("KB ingest failed for {}: {}", evt.eventid, e))?;
            if is_new { report.nodes_created += 1; } else { report.nodes_reused += 1; }
        }
        Ok(report)
    }

    pub fn to_search_results(events: &[GdacsEvent]) -> Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult> {
        events.iter().map(|e| crate::l2_perception::nt_world::nt_world_search::SearchResult {
            title: format!("[{}] {}", e.event_type, e.name),
            url: e.url.clone(),
            snippet: format!("alert={} sev={} pop={} {}", e.alert_level, e.severity, e.population, e.from_date),
            evidence: None,
        }).collect()
    }
}

// ── Egress ───────────────────────────────────────────────────

pub const GDACS_HOST: &str = "www.gdacs.org";
pub fn gdacs_egress_rule() -> crate::l1_action::nt_io::nt_shield_sandbox::EgressRule {
    crate::l1_action::nt_io::nt_shield_sandbox::EgressRule::allow(GDACS_HOST, "443")
}
pub fn gdacs_egress_policy() -> crate::l1_action::nt_io::nt_shield_sandbox::EgressPolicy {
    crate::l1_action::nt_io::nt_shield_sandbox::EgressPolicy::new(vec![gdacs_egress_rule()], false)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GdacsIngestReport {
    pub events_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

// ── SearchBackend ────────────────────────────────────────────

pub struct GdacsBackend {
    fetcher: GdacsFetcher,
}

impl Default for GdacsBackend {
    fn default() -> Self { Self { fetcher: GdacsFetcher::new() } }
}

impl GdacsBackend {
    pub fn new() -> Self { Self::default() }
}

impl crate::l2_perception::nt_world::nt_world_search::SearchBackend for GdacsBackend {
    fn name(&self) -> &str { "gdacs" }
    fn search(&self, _query: &str, count: usize) -> Result<Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult>, String> {
        let events = self.fetcher.fetch()?;
        let limited: Vec<GdacsEvent> = events.into_iter().take(count).collect();
        Ok(GdacsFetcher::to_search_results(&limited))
    }
}

// ── Fixture ────────────────────────────────────────────────

pub const GDACS_FIXTURE_JSON: &str = r#"{"result":[{"eventid":"2024-0001","alertlevel":"Green","eventtype":"EQ","name":"Tokyo","lat":"35.6","lon":"139.7","fromdate":"2024-01-01","todate":"2024-01-02","severitydata":{"severity":"1.2"},"populationdata":{"value":"1000"}},{"eventid":"2024-0002","alertlevel":"Orange","eventtype":"TC","name":"Typhoon X","lat":"20.0","lon":"130.0","fromdate":"2024-02-01","todate":"2024-02-05","severitydata":{"severity":"3.5"},"populationdata":{"value":"500000"}}]}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture() {
        let events = GdacsFetcher::parse_json(GDACS_FIXTURE_JSON).expect("parse");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, "EQ");
        assert_eq!(events[0].latitude, 35.6);
    }

    #[test]
    fn test_build_url() {
        assert_eq!(GdacsFetcher::new().build_url(), "https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH");
    }

    #[test]
    fn test_egress_policy() {
        let policy = gdacs_egress_policy();
        assert!(policy.check("www.gdacs.org", 443));
        assert!(!policy.check("evil.com", 443));
    }

    #[test]
    fn test_ingest_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
        let report = GdacsFetcher::new().ingest_from_json(&kb, GDACS_FIXTURE_JSON).expect("ingest");
        assert_eq!(report.nodes_created, 2);
    }

    #[test]
    fn test_backend_name() {
        use crate::l2_perception::nt_world::nt_world_search::SearchBackend;
        assert_eq!(SearchBackend::name(&GdacsBackend::new()), "gdacs");
    }
}
