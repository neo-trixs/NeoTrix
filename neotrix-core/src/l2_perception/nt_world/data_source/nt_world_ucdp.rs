#![forbid(unsafe_code)]

//! UCDP (Uppsala Conflict Data Program) 冲突数据 API 接线 — Wave6 H4 第5个情报工具 (P1, 10工具中第5个)
//!
//! - **数据源**: UCDP 冲突事件数据 (`ucdp.uu.se`, 公开 REST/GeoJSON，部分端点需 api_key)
//! - **端点**:
//!   `https://ucdp.uu.se/api/ged_events?country_id=...` (GED 事件)
//!   `https://ucdp.uu.se/api/ucdp_events?conflict_id=...` (UCDP 冲突事件)
//!   `https://ucdp.uu.se/api/state_based?year=...` (国家间/国内冲突)
//! - **强化节点**: `nt_world_search::Ordered Backend Router` (R-P42) — 新增 `UcdpBackend`
//!   作为有序路由第?后端，亦可独立作为 `UcdpFetcher` 直喂 intel-watch (R-P79)
//! - **入库**: `nt_memory_kb::KnowledgeBase` → `insert_or_get_node` (Event, domain=ucdp)
//! - **Egress**: `nt_shield_sandbox` UCDP_HOST allow 登记 (deny-wins)
//! - **测试**: fixture 驱动，无真实网络依赖，CI 稳定
//!
//! 参考: UCDP API 文档 https://ucdp.uu.se/apidocs/

use serde::{Deserialize, Serialize};

// ── UCDP 数据模型 ───────────────────────────────────────────────

/// UCDP 事件集合顶层结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct _UcdpEventCollection {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub events: Vec<_UcdpRawEvent>,
}

/// 单个 UCDP 冲突事件原始结构 (贴合 UCDP GED/API 字段)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct _UcdpRawEvent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub year: String,
    #[serde(default)]
    pub start_date: String,
    #[serde(default)]
    pub end_date: String,
    #[serde(default)]
    pub best: String,
    #[serde(default)]
    pub best_lo: String,
    #[serde(default)]
    pub side_a: String,
    #[serde(default)]
    pub side_b: String,
    #[serde(default)]
    pub deaths_a: String,
    #[serde(default)]
    pub deaths_b: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub type_of_violence: String,
    #[serde(default)]
    pub intensity: String,
}

/// 内部标准化事件结构 (用于入库)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _UcdpEvent {
    pub id: String,
    pub country: String,
    pub region: String,
    pub year: String,
    pub start_date: String,
    pub end_date: String,
    pub best: String,
    pub best_lo: String,
    pub side_a: String,
    pub side_b: String,
    pub deaths_a: String,
    pub deaths_b: String,
    pub r#type: String,
    pub type_of_violence: String,
    pub intensity: String,
}

// ── 解析层 ──────────────────────────────────────────────────────

fn extract_event_from_raw(raw: &_UcdpRawEvent) -> _UcdpEvent {
    _UcdpEvent {
        id: raw.id.clone(),
        country: raw.country.clone(),
        region: raw.region.clone(),
        year: raw.year.clone(),
        start_date: raw.start_date.clone(),
        end_date: raw.end_date.clone(),
        best: raw.best.clone(),
        best_lo: raw.best_lo.clone(),
        side_a: raw.side_a.clone(),
        side_b: raw.side_b.clone(),
        deaths_a: raw.deaths_a.clone(),
        deaths_b: raw.deaths_b.clone(),
        r#type: raw.r#type.clone(),
        type_of_violence: raw.type_of_violence.clone(),
        intensity: raw.intensity.clone(),
    }
}

// ── Fetch 层 ────────────────────────────────────────────────────

/// UCDP 数据集类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum _UcdpDataset {
    GedEvents,
    UcdpEvents,
    StateBased,
}

impl _UcdpDataset {
    fn path(&self) -> &'static str {
        match self {
            _UcdpDataset::GedEvents => "ged_events",
            _UcdpDataset::UcdpEvents => "ucdp_events",
            _UcdpDataset::StateBased => "state_based",
        }
    }
}

/// UCDP 冲突数据 Fetcher — UCDP REST API，可选 api_key，直喂 intel-watch。
pub struct UcdpFetcher {
    base_url: String,
    dataset: _UcdpDataset,
    api_key: Option<String>,
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for UcdpFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl UcdpFetcher {
    pub fn new() -> Self {
        Self::_with_dataset(_UcdpDataset::GedEvents)
    }

    pub fn _with_dataset(dataset: _UcdpDataset) -> Self {
        Self::_with_base_dataset_key("https://ucdp.uu.se", dataset, None)
    }

    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self::_with_base_dataset_key("https://ucdp.uu.se", _UcdpDataset::GedEvents, Some(api_key.into()))
    }

    pub fn _with_base_dataset_key(base_url: &str, dataset: _UcdpDataset, api_key: Option<String>) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            dataset,
            api_key,
            client: std::sync::OnceLock::new(),
        }
    }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (UCDP intel-watch)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    pub fn build_url(&self) -> String {
        match &self.api_key {
            Some(key) => format!(
                "{}/api/{}?api_key={}",
                self.base_url,
                self.dataset.path(),
                urlencoding_safe(key)
            ),
            None => format!("{}/api/{}", self.base_url, self.dataset.path()),
        }
    }

    pub fn fetch(&self) -> Result<Vec<_UcdpEvent>, String> {
        let url = self.build_url();
        let mut req = self.client().get(&url);
        if let Some(key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        let resp = req.send().map_err(|e| format!("UCDP request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("UCDP returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("UCDP read body: {}", e))?;
        Self::parse_json(&text)
    }

    pub fn parse_json(json: &str) -> Result<Vec<_UcdpEvent>, String> {
        let collection: _UcdpEventCollection = serde_json::from_str(json).map_err(|e| format!("UCDP parse failed: {}", e))?;
        Ok(collection.events.iter().map(extract_event_from_raw).collect())
    }

    pub fn ingest_from_json(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        json: &str,
    ) -> Result<_UcdpIngestReport, String> {
        let events = Self::parse_json(json)?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
    ) -> Result<_UcdpIngestReport, String> {
        let events = self.fetch()?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest_events(
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        events: &[_UcdpEvent],
    ) -> Result<_UcdpIngestReport, String> {
        let mut report = _UcdpIngestReport {
            events_fetched: events.len(),
            ..Default::default()
        };
        for evt in events {
            if evt.id.trim().is_empty() {
                report.errors.push(format!("skip empty id: {:?}", evt.id));
                continue;
            }
            let summary = format!(
                "{} vs {} | deaths A:{} B:{} | type:{} intensity:{} | {}",
                evt.side_a, evt.side_b, evt.deaths_a, evt.deaths_b, evt.type_of_violence, evt.intensity, evt.country
            );
            let url = format!("https://ucdp.uu.se/#/event/{}/{}", evt.year, evt.id);
            let existing = kb.find_node_by_url(&url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb.insert_or_get_node(&evt.id, crate::core::nt_core_kb_types::NodeType::Event, Some(&summary), Some(&url), Some("ucdp"))
                .map_err(|e| format!("KB ingest failed for {}: {}", evt.id, e))?;
            if is_new { report.nodes_created += 1; } else { report.nodes_reused += 1; }
        }
        Ok(report)
    }

    pub fn to_search_results(events: &[_UcdpEvent]) -> Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult> {
        events.iter().map(|e| crate::l2_perception::nt_world::nt_world_search::SearchResult {
            title: format!("UCDP {} - {} ({})", e.id, e.country, e.year),
            url: format!("https://ucdp.uu.se/#/event/{}/{}", e.year, e.id),
            snippet: format!("{} vs {} | deaths A:{} B:{}", e.side_a, e.side_b, e.deaths_a, e.deaths_b),
            evidence: None,
        }).collect()
    }
}

// 避免引入额外依赖，做最小 url 编码 (仅处理常见敏感字符)
fn urlencoding_safe(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

// ── Egress ───────────────────────────────────────────────────

pub const UCDP_HOST: &str = "ucdp.uu.se";
pub fn ucdp_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(UCDP_HOST, "443")
}
pub fn ucdp_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![ucdp_egress_rule()], false)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct _UcdpIngestReport {
    pub events_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

// ── SearchBackend ────────────────────────────────────────────

pub struct UcdpBackend {
    fetcher: UcdpFetcher,
}

impl Default for UcdpBackend {
    fn default() -> Self { Self { fetcher: UcdpFetcher::new() } }
}

impl UcdpBackend {
    pub fn new() -> Self { Self::default() }
    pub fn _with_dataset(dataset: _UcdpDataset) -> Self { Self { fetcher: UcdpFetcher::_with_dataset(dataset) } }
}

impl crate::l2_perception::nt_world::nt_world_search::SearchBackend for UcdpBackend {
    fn name(&self) -> &str { "ucdp" }
    fn search(&self, _query: &str, count: usize) -> Result<Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult>, String> {
        let events = self.fetcher.fetch()?;
        let limited: Vec<_UcdpEvent> = events.into_iter().take(count).collect();
        Ok(UcdpFetcher::to_search_results(&limited))
    }
}

// ── Fixture ────────────────────────────────────────────────

pub const UCDP_FIXTURE_JSON: &str = r#"{"type":"_UcdpEventCollection","events":[{"id":"12345","country":"Ukraine","region":"Europe","year":"2024","start_date":"2024-01-01","end_date":"2024-03-15","best":"150","best_lo":"100","side_a":"Government of Ukraine","side_b":"Rebels","deaths_a":"100","deaths_b":"50","type":"5","type_of_violence":"1","intensity":"3"},{"id":"67890","country":"Syria","region":"Middle East","year":"2023","start_date":"2023-05-10","end_date":"2023-12-31","best":"300","best_lo":"250","side_a":"Government of Syria","side_b":"Opposition","deaths_a":"200","deaths_b":"100","type":"5","type_of_violence":"2","intensity":"4"}]}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture() {
        let events = UcdpFetcher::parse_json(UCDP_FIXTURE_JSON).expect("parse fixture");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].country, "Ukraine");
        assert_eq!(events[0].side_a, "Government of Ukraine");
        assert_eq!(events[1].deaths_b, "100");
    }

    #[test]
    fn test_build_url_default() {
        let f = UcdpFetcher::new();
        assert_eq!(f.build_url(), "https://ucdp.uu.se/api/ged_events");
    }

    #[test]
    fn test_build_url_with_api_key() {
        let f = UcdpFetcher::with_api_key("secret-key");
        assert_eq!(f.build_url(), "https://ucdp.uu.se/api/ged_events?api_key=secret-key");
    }

    #[test]
    fn test_egress_policy() {
        let policy = ucdp_egress_policy();
        assert!(policy.check("ucdp.uu.se", 443));
        assert!(!policy.check("evil.com", 443));
    }

    #[test]
    fn test_ingest_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
        let fetcher = UcdpFetcher::new();
        let report = fetcher.ingest_from_json(&kb, UCDP_FIXTURE_JSON).expect("ingest");
        assert_eq!(report.nodes_created, 2);
    }

    #[test]
    fn test_ucdp_backend_name() {
        use crate::l2_perception::nt_world::nt_world_search::SearchBackend;
        let b = UcdpBackend::new();
        assert_eq!(SearchBackend::name(&b), "ucdp");
    }
}
