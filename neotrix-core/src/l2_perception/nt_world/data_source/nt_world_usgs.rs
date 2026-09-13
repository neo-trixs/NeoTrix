#![forbid(unsafe_code)]

//! USGS Earthquakes API 接线 — Wave6 H4 第3个情报工具 (P1, 10工具中第3个)
//!
//! - **数据源**: USGS 地震实时数据 (`earthquake.usgs.gov`, 免费无key，公开 GeoJSON)
//! - **端点**:
//!   `https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_day.geojson` (过去24小时)
//!   `https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_week.geojson` (过去7天)
//!   `https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_month.geojson` (过去30天)
//!   `https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/significant_week.geojson` (显著地震)
//! - **强化节点**: `nt_world_search::Ordered Backend Router` (R-P42) — 新增 `UsgsBackend`
//!   作为有序路由第5后端，亦可独立作为 `UsgsFetcher` 直喂 intel-watch (R-P79)
//! - **入库**: `nt_memory_kb::KnowledgeBase` → `insert_or_get_node` (Event, domain=usgs)
//! - **Egress**: `nt_shield_sandbox::INTEL_USGS_HOST` allow 登记 (deny-wins)
//! - **测试**: fixture 驱动，无真实网络依赖，CI 稳定
//!
//! 参考: USGS GeoJSON Feed 文档 https://earthquake.usgs.gov/earthquakes/feed/v1.0/geojson.php

use serde::{Deserialize, Serialize};

// ── USGS GeoJSON 数据模型 ───────────────────────────────────────────

/// USGS GeoJSON FeatureCollection 顶层结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct _UsgsFeatureCollection {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub metadata: _UsgsMetadata,
    #[serde(default)]
    pub features: Vec<_UsgsFeature>,
}

/// Metadata 包含查询信息
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct _UsgsMetadata {
    #[serde(default)]
    pub generated: i64,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub status: i32,
    #[serde(default)]
    pub api: String,
    #[serde(default)]
    pub count: i32,
}

/// 单个地震事件 Feature
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct _UsgsFeature {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub properties: _UsgsProperties,
    #[serde(default)]
    pub geometry: _UsgsGeometry,
    #[serde(default)]
    pub id: String,
}

/// 地震属性 (核心数据)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct _UsgsProperties {
    #[serde(default)]
    pub mag: Option<f64>,
    #[serde(default)]
    pub place: String,
    #[serde(default)]
    pub time: i64,
    #[serde(default)]
    pub updated: i64,
    #[serde(default)]
    pub tz: Option<i32>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub felt: Option<i32>,
    #[serde(default)]
    pub cdi: Option<f64>,
    #[serde(default)]
    pub mmi: Option<f64>,
    #[serde(default)]
    pub alert: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub tsunami: i32,
    #[serde(default)]
    pub sig: i32,
    #[serde(default)]
    pub net: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub ids: String,
    #[serde(default)]
    pub sources: String,
    #[serde(default)]
    pub types: String,
    #[serde(default)]
    pub nst: Option<i32>,
    #[serde(default)]
    pub dmin: Option<f64>,
    #[serde(default)]
    pub rms: Option<f64>,
    #[serde(default)]
    pub gap: Option<f64>,
    #[serde(default)]
    pub mag_type: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub title: String,
}

/// 几何坐标
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct _UsgsGeometry {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub coordinates: Vec<f64>,
}

/// 内部标准化事件结构 (用于入库)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _UsgsEvent {
    pub id: String,
    pub magnitude: f64,
    pub place: String,
    pub time: i64,
    pub updated: i64,
    pub url: String,
    pub detail_url: String,
    pub felt: Option<i32>,
    pub cdi: Option<f64>,
    pub mmi: Option<f64>,
    pub alert: Option<String>,
    pub status: String,
    pub tsunami: i32,
    pub sig: i32,
    pub net: String,
    pub code: String,
    pub mag_type: String,
    pub longitude: f64,
    pub latitude: f64,
    pub depth: f64,
    pub title: String,
}

// ── 解析层 ──────────────────────────────────────────────────────

fn extract_event_from_feature(feature: &_UsgsFeature) -> _UsgsEvent {
    let coords = &feature.geometry.coordinates;
    let (longitude, latitude, depth) = if coords.len() >= 3 {
        (coords[0], coords[1], coords[2])
    } else if coords.len() == 2 {
        (coords[0], coords[1], 0.0)
    } else {
        (0.0, 0.0, 0.0)
    };

    _UsgsEvent {
        id: feature.id.clone(),
        magnitude: feature.properties.mag.unwrap_or(0.0),
        place: feature.properties.place.clone(),
        time: feature.properties.time,
        updated: feature.properties.updated,
        url: feature.properties.url.clone(),
        detail_url: feature.properties.detail.clone(),
        felt: feature.properties.felt,
        cdi: feature.properties.cdi,
        mmi: feature.properties.mmi,
        alert: feature.properties.alert.clone(),
        status: feature.properties.status.clone(),
        tsunami: feature.properties.tsunami,
        sig: feature.properties.sig,
        net: feature.properties.net.clone(),
        code: feature.properties.code.clone(),
        mag_type: feature.properties.mag_type.clone(),
        longitude,
        latitude,
        depth,
        title: feature.properties.title.clone(),
    }
}

// ── Fetch 层 ────────────────────────────────────────────────────

/// USGS Feed 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum _UsgsFeedType {
    AllDay,
    AllWeek,
    AllMonth,
    SignificantWeek,
    SignificantMonth,
    M45Week,
    M25Week,
    M10Week,
}

impl _UsgsFeedType {
    fn path(&self) -> &'static str {
        match self {
            _UsgsFeedType::AllDay => "all_day.geojson",
            _UsgsFeedType::AllWeek => "all_week.geojson",
            _UsgsFeedType::AllMonth => "all_month.geojson",
            _UsgsFeedType::SignificantWeek => "significant_week.geojson",
            _UsgsFeedType::SignificantMonth => "significant_month.geojson",
            _UsgsFeedType::M45Week => "4.5_week.geojson",
            _UsgsFeedType::M25Week => "2.5_week.geojson",
            _UsgsFeedType::M10Week => "1.0_week.geojson",
        }
    }
}

/// USGS Earthquakes Fetcher — 免费无key，公开 GeoJSON，直喂 intel-watch。
pub struct UsgsFetcher {
    base_url: String,
    feed_type: _UsgsFeedType,
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for UsgsFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl UsgsFetcher {
    pub fn new() -> Self {
        Self::_with_feed(_UsgsFeedType::AllDay)
    }

    pub fn _with_feed(feed_type: _UsgsFeedType) -> Self {
        Self::_with_base_and_feed("https://earthquake.usgs.gov", feed_type)
    }

    pub fn _with_base_and_feed(base_url: &str, feed_type: _UsgsFeedType) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            feed_type,
            client: std::sync::OnceLock::new(),
        }
    }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (USGS intel-watch; https://github.com/neotrix)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    pub fn build_url(&self) -> String {
        format!("{}/earthquakes/feed/v1.0/summary/{}", self.base_url, self.feed_type.path())
    }

    pub fn fetch(&self) -> Result<Vec<_UsgsEvent>, String> {
        let url = self.build_url();
        let resp = self.client().get(&url).send().map_err(|e| format!("USGS request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("USGS returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("USGS read body: {}", e))?;
        Self::parse_geojson(&text)
    }

    pub fn parse_geojson(json: &str) -> Result<Vec<_UsgsEvent>, String> {
        let collection: _UsgsFeatureCollection = serde_json::from_str(json).map_err(|e| format!("USGS parse failed: {}", e))?;
        Ok(collection.features.iter().map(extract_event_from_feature).collect())
    }

    pub fn ingest_from_json(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        json: &str,
    ) -> Result<_UsgsIngestReport, String> {
        let events = Self::parse_geojson(json)?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
    ) -> Result<_UsgsIngestReport, String> {
        let events = self.fetch()?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest_events(
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        events: &[_UsgsEvent],
    ) -> Result<_UsgsIngestReport, String> {
        let mut report = _UsgsIngestReport {
            events_fetched: events.len(),
            ..Default::default()
        };
        for evt in events {
            if evt.id.trim().is_empty() {
                report.errors.push(format!("skip empty id: {:?}", evt.id));
                continue;
            }
            let summary = format!(
                "M{:.1} - {} | {}km depth | {} | {}",
                evt.magnitude, evt.place, evt.depth, evt.net, evt.title
            );
            let existing = kb.find_node_by_url(&evt.url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb.insert_or_get_node(&evt.title, crate::core::nt_core_kb_types::NodeType::Event, Some(&summary), Some(&evt.url), Some("usgs"))
                .map_err(|e| format!("KB ingest failed for {}: {}", evt.id, e))?;
            if is_new { report.nodes_created += 1; } else { report.nodes_reused += 1; }
        }
        Ok(report)
    }

    pub fn to_search_results(events: &[_UsgsEvent]) -> Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult> {
        events.iter().map(|e| crate::l2_perception::nt_world::nt_world_search::SearchResult {
            title: format!("M{:.1} - {}", e.magnitude, e.place),
            url: e.url.clone(),
            snippet: format!("M{:.1} | {}km | {} | {}", e.magnitude, e.depth as i32, e.net, e.title),
            evidence: None,
        }).collect()
    }
}

// ── Egress ───────────────────────────────────────────────────

pub const USGS_HOST: &str = "earthquake.usgs.gov";
pub fn usgs_egress_rule() -> super::super::l1_facade::EgressRule {
    super::super::l1_facade::EgressRule::allow(USGS_HOST, "443")
}

pub fn usgs_egress_policy() -> super::super::l1_facade::EgressPolicy {
    super::super::l1_facade::EgressPolicy::new(vec![usgs_egress_rule()], false)
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct _UsgsIngestReport {
    pub events_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

// ── SearchBackend ────────────────────────────────────────────

pub struct UsgsBackend {
    fetcher: UsgsFetcher,
}

impl Default for UsgsBackend {
    fn default() -> Self { Self { fetcher: UsgsFetcher::new() } }
}

impl UsgsBackend {
    pub fn new() -> Self { Self::default() }
    pub fn _with_feed(feed_type: _UsgsFeedType) -> Self { Self { fetcher: UsgsFetcher::_with_feed(feed_type) } }
}

impl crate::l2_perception::nt_world::nt_world_search::SearchBackend for UsgsBackend {
    fn name(&self) -> &str { "usgs" }
    fn search(&self, _query: &str, count: usize) -> Result<Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult>, String> {
        let events = self.fetcher.fetch()?;
        let limited: Vec<_UsgsEvent> = events.into_iter().take(count).collect();
        Ok(UsgsFetcher::to_search_results(&limited))
    }
}

// ── Fixture ────────────────────────────────────────────────

pub const USGS_FIXTURE_JSON: &str = r#"{"type":"FeatureCollection","metadata":{"generated":1724726400000,"url":"https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_day.geojson","title":"USGS All Earthquakes, Past Day","status":200,"api":"1.10.3","count":2},"features":[{"type":"Feature","properties":{"mag":5.2,"place":"50 km NE of Tokyo, Japan","time":1724726400000,"updated":1724726500000,"tz":540,"url":"https://earthquake.usgs.gov/earthquakes/eventpage/us7000test1","detail":"https://earthquake.usgs.gov/earthquakes/feed/v1.0/detail/us7000test1.geojson","felt":1200,"cdi":4.5,"mmi":5.8,"alert":"yellow","status":"reviewed","tsunami":0,"sig":450,"net":"us","code":"us7000test1","ids":",us7000test1,","sources":",us,","types":",geoserve,origin,phase-data,nearby-cities,losspager,moment-tensor,focal-mechanism,","nst":150,"dmin":0.345,"rms":0.87,"gap":28,"magType":"mb","type":"earthquake","title":"M 5.2 - 50 km NE of Tokyo, Japan"},"geometry":{"type":"Point","coordinates":[140.1234,35.6789,45.2]},"id":"us7000test1"},{"type":"Feature","properties":{"mag":3.8,"place":"15 km WSW of San Francisco, California","time":1724725000000,"updated":1724725100000,"tz":-480,"url":"https://earthquake.usgs.gov/earthquakes/eventpage/nc7000test2","detail":"https://earthquake.usgs.gov/earthquakes/feed/v1.0/detail/nc7000test2.geojson","felt":250,"cdi":3.2,"mmi":4.1,"alert":"green","status":"automatic","tsunami":0,"sig":180,"net":"nc","code":"nc7000test2","ids":",nc7000test2,","sources":",nc,","types":",geoserve,origin,phase-data,nearby-cities,","nst":45,"dmin":0.123,"rms":0.45,"gap":42,"magType":"md","type":"earthquake","title":"M 3.8 - 15 km WSW of San Francisco, California"},"geometry":{"type":"Point","coordinates":[-122.4194,37.7749,8.5]},"id":"nc7000test2"}]}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture() {
        let events = UsgsFetcher::parse_geojson(USGS_FIXTURE_JSON).expect("parse fixture");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].magnitude, 5.2);
        assert_eq!(events[0].place, "50 km NE of Tokyo, Japan");
    }

    #[test]
    fn test_build_url_default() {
        let f = UsgsFetcher::new();
        assert_eq!(f.build_url(), "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_day.geojson");
    }

    #[test]
    fn test_egress_policy() {
        let policy = usgs_egress_policy();
        assert!(policy.check("earthquake.usgs.gov", 443));
        assert!(!policy.check("evil.com", 443));
    }

    #[test]
    fn test_ingest_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
        let fetcher = UsgsFetcher::new();
        let report = fetcher.ingest_from_json(&kb, USGS_FIXTURE_JSON).expect("ingest");
        assert_eq!(report.nodes_created, 2);
    }

    #[test]
    fn test_usgs_backend_name() {
        use crate::l2_perception::nt_world::nt_world_search::SearchBackend;
        let b = UsgsBackend::new();
        assert_eq!(SearchBackend::name(&b), "usgs");
    }
}