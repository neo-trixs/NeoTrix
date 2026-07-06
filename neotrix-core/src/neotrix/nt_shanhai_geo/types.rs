use serde::{Deserialize, Serialize};

// ─── Coordinate Systems ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoCoord {
    pub lat: f64,
    pub lng: f64,
}

impl GeoCoord {
    pub const fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoBounds {
    pub min: GeoCoord,
    pub max: GeoCoord,
}

impl GeoBounds {
    pub fn contains(&self, p: &GeoCoord) -> bool {
        p.lat >= self.min.lat
            && p.lat <= self.max.lat
            && p.lng >= self.min.lng
            && p.lng <= self.max.lng
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceLi(pub f64);

/// Which scale theory to use for li → meter conversion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LiScale {
    /// 1 li = 12m (Liu Zongdi)
    LiuZongdi,
    /// 1 li = 300m (Shang dynasty)
    Shang,
    /// 1 li = 400m (Zhou dynasty)
    Zhou,
    /// 1 li = 576m (Qin/early Han)
    QinHan,
    /// 1 li = 500m (modern standard)
    Modern,
}

impl LiScale {
    pub fn meters_per_li(&self) -> f64 {
        match self {
            LiScale::LiuZongdi => 12.0,
            LiScale::Shang => 300.0,
            LiScale::Zhou => 400.0,
            LiScale::QinHan => 576.0,
            LiScale::Modern => 500.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LiScale::LiuZongdi => "刘宗迪尺度——1里=12米",
            LiScale::Shang => "商代尺度——1里=300米",
            LiScale::Zhou => "周代尺度——1里=400米",
            LiScale::QinHan => "秦汉尺度——1里=576米",
            LiScale::Modern => "现代尺度——1里=500米",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShanhaiCoord {
    /// The host mountain range name
    pub range_name: String,
    /// Distance from origin in li
    pub distance_from_origin: DistanceLi,
    /// Direction from origin
    pub direction: Direction,
    /// Which li scale is assumed
    pub li_scale: LiScale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Direction {
    East,
    South,
    West,
    North,
    Center,
}

impl Direction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Direction::East => "东",
            Direction::South => "南",
            Direction::West => "西",
            Direction::North => "北",
            Direction::Center => "中",
        }
    }
}

// ─── Mountain Data ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FiveTreasury {
    SouthMountains,
    WestMountains,
    NorthMountains,
    EastMountains,
    CentralMountains,
}

impl FiveTreasury {
    pub fn name(&self) -> &'static str {
        match self {
            FiveTreasury::SouthMountains => "南山经",
            FiveTreasury::WestMountains => "西山经",
            FiveTreasury::NorthMountains => "北山经",
            FiveTreasury::EastMountains => "东山经",
            FiveTreasury::CentralMountains => "中山经",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountainRange {
    pub id: String,
    pub name: String,
    pub treasury: FiveTreasury,
    pub range_index: u32,
    pub approximate_bounds: Option<GeoBounds>,
    pub li_scale_preferred: LiScale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountainPeak {
    pub id: String,
    pub range_id: String,
    pub name: String,
    /// Position within the range (1-indexed)
    pub position: u32,
    pub shanhai_coord: Option<ShanhaiCoord>,
    pub modern_location: Option<GeoCoord>,
    pub identification_confidence: f64,
    /// Which school(s) support this identification
    pub attributed_by: Vec<SchoolRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolRef {
    pub school: String,
    pub scholar: String,
    pub confidence: f64,
}

// ─── Global Mapping ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceMapping {
    pub shanhai_name: String,
    pub modern_name: String,
    pub modern_location: Option<GeoCoord>,
    pub modern_bounds: Option<GeoBounds>,
    pub confidence: f64,
    pub school_attribution: Vec<SchoolRef>,
    pub evidence_summary: String,
    pub relation_type: String,
}

// ─── School Parameters ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolParameters {
    pub name: String,
    pub founder: String,
    pub description: String,
    pub li_scale: LiScale,
    pub scope: GeographicScope,
    pub key_mountains: Vec<String>,
    pub confidence_base: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GeographicScope {
    /// Everything within modern China borders
    ChinaOnly,
    /// Global — Asia, Africa, Americas
    Global,
    /// Specific region only (Shandong / Yunnan)
    LocalRegion(String),
}

impl GeographicScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            GeographicScope::ChinaOnly => "华夏范围内",
            GeographicScope::Global => "全球范围内",
            GeographicScope::LocalRegion(_) => "局部区域",
        }
    }
}
