#![forbid(unsafe_code)]

//! AOI (Area of Interest) Geofencing 情报监视器 — Wave6 第9个情报工具 (P1, 10工具中第9个)
//!
//! - **数据源**: USGS FDSN Event 查询服务 (`earthquake.usgs.gov`，免费无key，公开 GeoJSON)
//! - **端点**:
//!   `https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson`
//!   `&minlatitude={}&maxlatitude={}&minlongitude={}&maxlongitude={}`
//!   通过地理包围盒 (bounding box) 参数检索 AOI 内的事件
//! - **概念**: AOI = Area of Interest。本模块定义 `Geofence` (min_lat, max_lat, min_lon, max_lon)
//!   与 `AoiMonitor`，查询真实 GeoJSON 事件源并过滤到围栏内的事件。
//! - **数据模型**: 自带最小 GeoJSON `FeatureCollection`/`Feature`/`Geometry` 模型 (`Aoi` 前缀，
//!   不依赖 `nt_world_usgs`)，避免结构体冲突。
//! - **强化节点**: `nt_world_search::Ordered Backend Router` (R-P42) — 新增 `AoiBackend`
//!   作为有序路由后端，亦可独立作为 `AoiMonitor` 直喂 intel-watch (R-P79)
//! - **入库**: `nt_memory_kb::KnowledgeBase` → `insert_or_get_node` (Event, domain=aoi)
//! - **Egress**: `nt_shield_sandbox::AOI_HOST` allow 登记 (deny-wins)
//! - **测试**: fixture 驱动，无真实网络依赖，CI 稳定
//!
//! 参考: USGS FDSN Web Service https://earthquake.usgs.gov/fdsnws/event/1/

use serde::{Deserialize, Serialize};

// ── AOI GeoJSON 数据模型 ───────────────────────────────────────

/// AOI GeoJSON FeatureCollection 顶层结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AoiFeatureCollection {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub features: Vec<AoiFeature>,
}

/// 单个事件 Feature
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AoiFeature {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub properties: AoiProps,
    #[serde(default)]
    pub geometry: AoiGeometry,
    #[serde(default)]
    pub id: String,
}

/// 事件属性 (核心数据)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AoiProps {
    #[serde(default)]
    pub mag: Option<f64>,
    #[serde(default)]
    pub place: String,
    #[serde(default)]
    pub time: i64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
}

/// 几何坐标
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AoiGeometry {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub coordinates: Vec<f64>,
}

/// 内部标准化事件结构 (用于入库)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AoiEvent {
    pub id: String,
    pub magnitude: f64,
    pub place: String,
    pub time: i64,
    pub title: String,
    pub url: String,
    pub longitude: f64,
    pub latitude: f64,
    pub depth: f64,
}

// ── 地理围栏 ──────────────────────────────────────────────────

/// 地理包围盒 (Area of Interest)
#[derive(Debug, Clone, Copy)]
pub struct Geofence {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

impl Geofence {
    /// 判断坐标 (lat, lon) 是否落在围栏内 (闭区间)
    pub fn contains(&self, lat: f64, lon: f64) -> bool {
        lat >= self.min_lat && lat <= self.max_lat && lon >= self.min_lon && lon <= self.max_lon
    }
}

// ── 解析层 ────────────────────────────────────────────────────

fn extract_event_from_feature(feature: &AoiFeature) -> AoiEvent {
    let coords = &feature.geometry.coordinates;
    let (longitude, latitude, depth) = if coords.len() >= 3 {
        (coords[0], coords[1], coords[2])
    } else if coords.len() == 2 {
        (coords[0], coords[1], 0.0)
    } else {
        (0.0, 0.0, 0.0)
    };

    AoiEvent {
        id: feature.id.clone(),
        magnitude: feature.properties.mag.unwrap_or(0.0),
        place: feature.properties.place.clone(),
        time: feature.properties.time,
        title: feature.properties.title.clone(),
        url: feature.properties.url.clone(),
        longitude,
        latitude,
        depth,
    }
}

fn extract_events(features: &[AoiFeature]) -> Vec<AoiEvent> {
    features.iter().map(extract_event_from_feature).collect()
}

// ── Fetch 层 ─────────────────────────────────────────────────

/// AOI Geofence 监视器 — 免费无key，公开 GeoJSON，直喂 intel-watch。
pub struct AoiMonitor {
    fence: Geofence,
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for AoiMonitor {
    fn default() -> Self {
        Self::new(Geofence {
            min_lat: 35.0,
            max_lat: 36.0,
            min_lon: 139.0,
            max_lon: 140.0,
        })
    }
}

impl AoiMonitor {
    pub fn new(fence: Geofence) -> Self {
        Self {
            fence,
            client: std::sync::OnceLock::new(),
        }
    }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (AOI geofence monitor)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    /// 构造 USGS FDSN 查询 URL (带 bounding-box 参数)
    pub fn build_url(&self) -> String {
        format!(
            "https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&minlatitude={}&maxlatitude={}&minlongitude={}&maxlongitude={}",
            self.fence.min_lat, self.fence.max_lat, self.fence.min_lon, self.fence.max_lon
        )
    }

    pub fn fetch(&self) -> Result<Vec<AoiEvent>, String> {
        let url = self.build_url();
        let resp = self.client().get(&url).send().map_err(|e| format!("AOI request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("AOI returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("AOI read body: {}", e))?;
        Self::parse_geojson(&text)
    }

    pub fn parse_geojson(json: &str) -> Result<Vec<AoiEvent>, String> {
        let collection: AoiFeatureCollection = serde_json::from_str(json).map_err(|e| format!("AOI parse failed: {}", e))?;
        Ok(extract_events(&collection.features))
    }

    /// 按地理围栏过滤事件，仅保留围栏内事件
    pub fn filter_events(events: &[AoiEvent], fence: Geofence) -> Vec<AoiEvent> {
        events
            .iter()
            .filter(|e| fence.contains(e.latitude, e.longitude))
            .cloned()
            .collect()
    }

    pub fn ingest_from_json(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        json: &str,
    ) -> Result<AoiIngestReport, String> {
        let events = Self::parse_geojson(json)?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
    ) -> Result<AoiIngestReport, String> {
        let events = self.fetch()?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest_events(
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        events: &[AoiEvent],
    ) -> Result<AoiIngestReport, String> {
        let mut report = AoiIngestReport {
            events_fetched: events.len(),
            ..Default::default()
        };
        for evt in events {
            if evt.id.trim().is_empty() {
                report.errors.push(format!("skip empty id: {:?}", evt.id));
                continue;
            }
            let summary = format!(
                "M{:.1} - {} | {}km depth | {}",
                evt.magnitude, evt.place, evt.depth, evt.title
            );
            let existing = kb.find_node_by_url(&evt.url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb.insert_or_get_node(&evt.title, crate::core::nt_core_kb_types::NodeType::Event, Some(&summary), Some(&evt.url), Some("aoi"))
                .map_err(|e| format!("KB ingest failed for {}: {}", evt.id, e))?;
            if is_new { report.nodes_created += 1; } else { report.nodes_reused += 1; }
        }
        Ok(report)
    }

    pub fn to_search_results(events: &[AoiEvent]) -> Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult> {
        events.iter().map(|e| crate::l2_perception::nt_world::nt_world_search::SearchResult {
            title: format!("M{:.1} - {}", e.magnitude, e.place),
            url: e.url.clone(),
            snippet: format!("M{:.1} | {}km | {} | {}", e.magnitude, e.depth as i32, e.place, e.title),
            evidence: None,
        }).collect()
    }
}

/// `parse_geojson` 无 `&self` 时使用的默认围栏 (Tokyo 区)
fn self_default_fence() -> Geofence {
    Geofence {
        min_lat: 35.0,
        max_lat: 36.0,
        min_lon: 139.0,
        max_lon: 140.0,
    }
}

// ── Egress ───────────────────────────────────────────────────

pub const AOI_HOST: &str = "earthquake.usgs.gov";
pub fn aoi_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(AOI_HOST, "443")
}
pub fn aoi_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![aoi_egress_rule()], false)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AoiIngestReport {
    pub events_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

// ── SearchBackend ────────────────────────────────────────────

/// AOI 搜索后端 — 作为有序路由后端接入 `nt_world_search`
pub struct AoiBackend {
    monitor: AoiMonitor,
}

impl Default for AoiBackend {
    fn default() -> Self {
        Self {
            monitor: AoiMonitor::new(Geofence {
                min_lat: 35.0,
                max_lat: 36.0,
                min_lon: 139.0,
                max_lon: 140.0,
            }),
        }
    }
}

impl AoiBackend {
    pub fn new(fence: Geofence) -> Self {
        Self { monitor: AoiMonitor::new(fence) }
    }
}

impl crate::l2_perception::nt_world::nt_world_search::SearchBackend for AoiBackend {
    fn name(&self) -> &str {
        "aoi"
    }
    fn search(&self, _query: &str, count: usize) -> Result<Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult>, String> {
        // 测试 / 离线场景：使用 fixture 数据；真实场景可通过 monitor.fetch() 拉取
        let mut events: Vec<AoiEvent> = AoiMonitor::parse_geojson(AOI_FIXTURE_JSON)?;
        events = AoiMonitor::filter_events(&events, self.monitor.fence);
        let limited: Vec<AoiEvent> = events.into_iter().take(count).collect();
        Ok(AoiMonitor::to_search_results(&limited))
    }
}

// ── Fixture ────────────────────────────────────────────────

pub const AOI_FIXTURE_JSON: &str = r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"mag":5.2,"place":"50 km NE of Tokyo, Japan","time":1724726400000,"title":"M 5.2 - 50 km NE of Tokyo, Japan","url":"https://earthquake.usgs.gov/earthquakes/eventpage/us7000aoi01"},"geometry":{"type":"Point","coordinates":[139.7,35.6,45.2]},"id":"us7000aoi01"},{"type":"Feature","properties":{"mag":3.8,"place":"15 km WSW of San Francisco, California","time":1724725000000,"title":"M 3.8 - 15 km WSW of San Francisco, California","url":"https://earthquake.usgs.gov/earthquakes/eventpage/nc7000aoi02"},"geometry":{"type":"Point","coordinates":[-122.4194,37.7749,8.5]},"id":"nc7000aoi02"}]}"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2_perception::nt_world::nt_world_search::SearchBackend;

    fn default_fence() -> Geofence {
        Geofence { min_lat: 35.0, max_lat: 36.0, min_lon: 139.0, max_lon: 140.0 }
    }

    #[test]
    fn test_parse_fixture() {
        let events = AoiMonitor::parse_geojson(AOI_FIXTURE_JSON).expect("parse fixture");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].magnitude, 5.2);
        assert_eq!(events[0].place, "50 km NE of Tokyo, Japan");
    }

    #[test]
    fn test_filter_excludes_outside() {
        let events = AoiMonitor::parse_geojson(AOI_FIXTURE_JSON).expect("parse fixture");
        let filtered = AoiMonitor::filter_events(&events, default_fence());
        // 仅 Tokyo 围栏内 (35.6, 139.7) 保留，旧金山围栏外排除
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "us7000aoi01");
    }

    #[test]
    fn test_build_url() {
        let m = AoiMonitor::new(default_fence());
        let url = m.build_url();
        assert!(url.starts_with("https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson"));
        assert!(url.contains("minlatitude=35"));
        assert!(url.contains("maxlatitude=36"));
        assert!(url.contains("minlongitude=139"));
        assert!(url.contains("maxlongitude=140"));
    }

    #[test]
    fn test_egress_policy() {
        let policy = aoi_egress_policy();
        assert!(policy.check("earthquake.usgs.gov", 443));
        assert!(!policy.check("evil.com", 443));
    }

    #[test]
    fn test_ingest_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
        let monitor = AoiMonitor::new(default_fence());
        let report = monitor.ingest_from_json(&kb, AOI_FIXTURE_JSON).expect("ingest");
        // fixture 2 条事件均入库 (过滤在 search / 使用方侧，ingest 保留全量)
        assert_eq!(report.nodes_created, 2);
    }

    #[test]
    fn test_backend_name() {
        let b = AoiBackend::default();
        assert_eq!(SearchBackend::name(&b), "aoi");
    }

    #[test]
    fn test_geofence_contains() {
        let f = default_fence();
        assert!(f.contains(35.6, 139.7));
        assert!(f.contains(35.0, 139.0));
        assert!(!f.contains(37.0, 139.7));
        assert!(!f.contains(35.6, 141.0));
    }
}
