use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};

// ============================================================
// ProjectSnapshot (from L8 nt_mind_evolution_loop)
// ============================================================

#[derive(Debug, Clone)]
pub struct ProjectSnapshot {
    pub total_files: usize,
    pub total_lines: usize,
    pub large_files: Vec<String>,
    pub modules_without_tests: Vec<String>,
    pub file_unsafe_hotspots: Vec<String>,
    pub unsafe_count: usize,
    pub unwrap_count: usize,
    pub todo_count: usize,
    pub compile_errors: usize,
    pub compile_warnings: usize,
    pub test_count: usize,
    pub test_failures: usize,
}

// ============================================================
// IssueType (from L8 nt_mind_evolution_loop)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueType {
    LargeFile,
    MissingTests,
    ExcessUnsafe,
    ExcessUnwrap,
    CircularDep,
    TodoLeftovers,
    CompileWarning,
    TestFailure,
    StagnantEvolve,
    HighFreeEnergy,
    LowPhi,
}

// ============================================================
// ActionPlan (from L8 nt_mind_self_diagnose)
// ============================================================

#[derive(Debug, Clone)]
pub enum ActionPlan {
    AddTestStub { file: String },
    RunCargoFix,
    RemoveTodo { file: String },
    HumanDecision { issue_type: IssueType, file: Option<String>, reason: String },
    SplitLargeFile { file: String },
    ReviewUnsafe { file: String },
    ReplaceUnwrap { file: String },
    NoAction { reason: String },
}

// ============================================================
// PrioritizedIssue & CodeUnderlyingIssue (from L8 nt_mind_self_diagnose)
// ============================================================

#[derive(Debug, Clone)]
pub struct PrioritizedIssue {
    pub action: ActionPlan,
    pub composite_score: f64,
    pub underlying_issue: CodeUnderlyingIssue,
}

#[derive(Debug, Clone)]
pub struct CodeUnderlyingIssue {
    pub file: Option<String>,
    pub issue_type: String,
}

// ============================================================
// EvolutionLoopProvider trait (shared L1/L8 protocol)
// ============================================================

pub trait EvolutionLoopProvider {
    fn self_diagnose(&mut self) -> (Vec<String>, Vec<PrioritizedIssue>);
    fn on_fix_applied(&mut self);
}

// ============================================================
// KbProvider trait (replaces direct L3 KnowledgeBase dependency)
// ============================================================

pub trait KbProvider: Send + Sync {
    fn kv_set(&self, ns: &str, key: &str, value: &str) -> Result<(), String>;
    fn kv_get(&self, ns: &str, key: &str) -> Result<Option<String>, String>;
    fn kv_delete(&self, ns: &str, key: &str) -> Result<(), String>;
    fn secret_set(&self, key: &str, value: &str) -> Result<(), String>;
    fn secret_get(&self, key: &str) -> Result<Option<String>, String>;
}

// ============================================================
// Tile types (from L3 nt_memory_spatial)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileFormat {
    Mvt,
    Png,
    Jpeg,
    Webp,
}

#[derive(Debug, Clone)]
pub struct TileCacheEntry {
    pub data: Vec<u8>,
    pub format: TileFormat,
    pub cached_at: u64,
    pub ttl_ms: u64,
    pub size_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct TileCacheStats {
    pub entries: usize,
    pub total_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

pub struct TileCache {
    entries: HashMap<String, TileCacheEntry>,
    order: VecDeque<String>,
    max_entries: usize,
    max_bytes: usize,
    total_bytes: usize,
    hits: u64,
    misses: u64,
}

impl TileCache {
    pub fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            max_entries,
            max_bytes,
            total_bytes: 0,
            hits: 0,
            misses: 0,
        }
    }

    pub fn default_tile_cache() -> Self {
        Self::new(10_000, 500_000_000)
    }

    pub fn get(&mut self, z: u8, x: u64, y: u64) -> Option<&TileCacheEntry> {
        let key = format!("tile/{}/{}/{}", z, x, y);
        if let Some(entry) = self.entries.get(&key) {
            self.hits += 1;
            return Some(entry);
        }
        self.misses += 1;
        None
    }

    pub fn set(&mut self, z: u8, x: u64, y: u64, data: Vec<u8>, format: TileFormat, ttl_ms: u64) {
        let key = format!("tile/{}/{}/{}", z, x, y);
        let size = data.len();
        let entry = TileCacheEntry {
            data,
            format,
            cached_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            ttl_ms,
            size_bytes: size,
        };
        if let Some(old) = self.entries.get(&key) {
            self.total_bytes = self.total_bytes.saturating_sub(old.size_bytes);
            self.order.retain(|k| k != &key);
        }
        while self.entries.len() >= self.max_entries {
            if let Some(evict_key) = self.order.pop_front() {
                if let Some(evicted) = self.entries.remove(&evict_key) {
                    self.total_bytes = self.total_bytes.saturating_sub(evicted.size_bytes);
                }
            } else {
                break;
            }
        }
        while self.total_bytes + size > self.max_bytes && !self.entries.is_empty() {
            if let Some(evict_key) = self.order.pop_front() {
                if let Some(evicted) = self.entries.remove(&evict_key) {
                    self.total_bytes = self.total_bytes.saturating_sub(evicted.size_bytes);
                }
            } else {
                break;
            }
        }
        self.total_bytes += size;
        self.order.push_back(key.clone());
        self.entries.insert(key, entry);
    }

    pub fn stats(&self) -> TileCacheStats {
        let total = self.hits + self.misses;
        TileCacheStats {
            entries: self.entries.len(),
            total_bytes: self.total_bytes,
            hits: self.hits,
            misses: self.misses,
            hit_rate: if total > 0 { self.hits as f64 / total as f64 } else { 0.0 },
        }
    }

    pub fn invalidate(&mut self, z: u8, x: u64, y: u64) {
        let key = format!("tile/{}/{}/{}", z, x, y);
        if let Some(entry) = self.entries.remove(&key) {
            self.total_bytes = self.total_bytes.saturating_sub(entry.size_bytes);
            self.order.retain(|k| k != &key);
        }
    }

    pub fn invalidate_zoom(&mut self, z: u8) {
        let prefix = format!("tile/{}/", z);
        let to_remove: Vec<String> = self.entries.keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        for key in to_remove {
            if let Some(entry) = self.entries.remove(&key) {
                self.total_bytes = self.total_bytes.saturating_sub(entry.size_bytes);
                self.order.retain(|k| k != &key);
            }
        }
    }
}

// ============================================================
// Spatial types (from L3 nt_memory_spatial)
// ============================================================

#[derive(Debug, Clone)]
pub struct GeoPoint {
    pub lat: f64,
    pub lng: f64,
}

impl GeoPoint {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }
}

#[derive(Debug, Clone)]
pub enum SpatialGeometry {
    Point(GeoPoint),
    Line(Vec<GeoPoint>),
    Polygon(Vec<Vec<GeoPoint>>),
    MultiGeometry(Vec<SpatialGeometry>),
}

#[derive(Debug, Clone)]
pub struct GeoJsonFeature {
    pub geometry: GeoJsonGeometry,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum GeoJsonGeometry {
    Point { lat: f64, lng: f64 },
    LineString { points: Vec<GeoPoint> },
    Polygon { rings: Vec<Vec<GeoPoint>> },
}

#[derive(Debug, Clone)]
pub struct SpatialEntry {
    pub id: String,
    pub name: String,
    pub geometry: SpatialGeometry,
    pub properties: HashMap<String, String>,
    pub source: String,
    pub confidence: f64,
    pub created_at: u64,
    pub updated_at: u64,
    pub tags: Vec<String>,
}

impl SpatialEntry {
    pub fn to_geojson(&self) -> GeoJsonFeature {
        let geometry = match &self.geometry {
            SpatialGeometry::Point(p) => GeoJsonGeometry::Point { lat: p.lat, lng: p.lng },
            SpatialGeometry::Line(pts) => GeoJsonGeometry::LineString { points: pts.clone() },
            SpatialGeometry::Polygon(rings) => GeoJsonGeometry::Polygon { rings: rings.clone() },
            SpatialGeometry::MultiGeometry(_) => GeoJsonGeometry::Point { lat: 0.0, lng: 0.0 },
        };
        GeoJsonFeature {
            geometry,
            properties: self.properties.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        }
    }
}
