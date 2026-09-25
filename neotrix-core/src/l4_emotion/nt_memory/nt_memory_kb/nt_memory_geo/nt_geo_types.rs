//! nt_geo_types — 地理共享记录类型 (Geo/Weather/Trajectory)，行为零变更纯搬移。

/// 单条地理索引记录。
#[derive(Debug, Clone)]
pub struct GeoRecord {
    pub node_id: String,
    pub lat: f64,
    pub lng: f64,
    pub country: String,
    pub region: String,
    pub city: String,
    pub tags: String,
    pub source: String,
    pub confidence: f64,
}

/// 气象快照记录。
#[derive(Debug, Clone)]
pub struct WeatherRecord {
    pub node_id: String,
    pub lat: f64,
    pub lng: f64,
    pub temp_c: Option<f64>,
    pub pressure_msl: Option<f64>,
    pub wind_kmh: Option<f64>,
    pub precip_mm: Option<f64>,
    pub elevation_m: Option<f64>,
    pub fetched_at: i64,
}

/// 查询轨迹结果行。
#[derive(Debug, Clone)]
pub struct TrajectoryRow {
    pub id: String,
    pub name: String,
    pub kind: Option<String>,
    pub points_json: String,
    pub bbox_west: f64,
    pub bbox_south: f64,
    pub bbox_east: f64,
    pub bbox_north: f64,
    pub distance_km: f64,
    pub created_at: i64,
}
