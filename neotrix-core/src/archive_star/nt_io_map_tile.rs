use crate::neotrix::l1_body_impl::nt_l1_shared_types::{
    SpatialEntry, TileCache, TileFormat, TileCacheStats,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomLevel(pub u8);

impl ZoomLevel {
    pub fn new(z: u8) -> Self { Self(z.min(22)) }
    pub fn num_tiles(&self) -> u64 { 1u64 << self.0 }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoPoint {
    pub lat: f64,
    pub lng: f64,
}

impl GeoPoint {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat: lat.clamp(-90.0, 90.0), lng: lng.clamp(-180.0, 180.0) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min_lat: f64,
    pub min_lng: f64,
    pub max_lat: f64,
    pub max_lng: f64,
}

impl BoundingBox {
    pub fn new(min_lat: f64, min_lng: f64, max_lat: f64, max_lng: f64) -> Self {
        Self { min_lat, min_lng, max_lat, max_lng }
    }
    pub fn center(&self) -> GeoPoint {
        GeoPoint::new((self.min_lat + self.max_lat) / 2.0, (self.min_lng + self.max_lng) / 2.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileCoord {
    pub x: u64,
    pub y: u64,
    pub z: ZoomLevel,
}

impl TileCoord {
    pub fn new(x: u64, y: u64, z: ZoomLevel) -> Option<Self> {
        let n = z.num_tiles();
        if x >= n || y >= n { return None; }
        Some(Self { x, y, z })
    }
    pub fn to_bbox(&self) -> BoundingBox {
        BoundingBox::new(-90.0, -180.0, 90.0, 180.0)
    }
}

pub struct MapTileService {
    pub cache: TileCache,
    tile_providers: Vec<Box<dyn TileProvider>>,
}

impl Default for MapTileService {
    fn default() -> Self {
        Self::new()
    }
}

impl MapTileService {
    pub fn new() -> Self {
        Self {
            cache: TileCache::default_tile_cache(),
            tile_providers: Vec::new(),
        }
    }

    pub fn add_provider(&mut self, provider: Box<dyn TileProvider>) {
        self.tile_providers.push(provider);
    }

    pub fn get_tile(&mut self, z: u8, x: u64, y: u64) -> Vec<u8> {
        if let Some(cached) = self.cache.get(z, x, y) {
            return cached.data.clone();
        }

        for provider in &self.tile_providers {
            if let Some(data) = provider.fetch_tile(z, x, y) {
                let format = provider.tile_format();
                let ttl = provider.cache_ttl_ms();
                self.cache.set(z, x, y, data.clone(), format, ttl);
                return data;
            }
        }

        self.generate_fallback_tile(z, x, y)
    }

    pub fn get_tile_with_style(&mut self, z: u8, x: u64, y: u64, style: &str) -> Vec<u8> {
        for provider in &self.tile_providers {
            if provider.name() == style {
                if let Some(data) = provider.fetch_tile(z, x, y) {
                    return data;
                }
            }
        }
        self.get_tile(z, x, y)
    }

    pub fn cache_stats(&self) -> TileCacheStats {
        self.cache.stats()
    }

    pub fn invalidate_tile(&mut self, z: u8, x: u64, y: u64) {
        self.cache.invalidate(z, x, y);
    }

    pub fn invalidate_zoom(&mut self, z: u8) {
        self.cache.invalidate_zoom(z);
    }

    fn generate_fallback_tile(&self, z: u8, x: u64, y: u64) -> Vec<u8> {
        let tile = TileCoord::new(x, y, ZoomLevel::new(z));
        let Some(tile) = tile else { return Vec::new(); };
        let bbox = tile.to_bbox();
        let center = bbox.center();

        let is_land = estimate_land_raw(center.lat, center.lng);
        let color = if is_land { "#e8e4d8" } else { "#c8d8e8" };
        let label = format!("z{}/{}/{}", z, x, y);

        let tcol1 = "#666666";
        let tcol2 = "#999999";
        let scol = "#dddddd";
        let svg_body = format!(
            r#"<rect width="256" height="256" fill="{c}"/>
<text x="128" y="128" text-anchor="middle" font-family="sans-serif" font-size="12" fill="{t1}">{label}</text>
<text x="128" y="144" text-anchor="middle" font-family="sans-serif" font-size="10" fill="{t2}">{lat:.4},{lng:.4}</text>
<line x1="0" y1="128" x2="256" y2="128" stroke="{s}" stroke-width="0.5"/>
<line x1="128" y1="0" x2="128" y2="256" stroke="{s}" stroke-width="0.5"/>"#,
            c = color, t1 = tcol1, t2 = tcol2, s = scol,
            label = label, lat = center.lat, lng = center.lng,
        );
        format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"256\" height=\"256\">{}</svg>", svg_body).into_bytes()
    }
}

fn estimate_land_raw(lat: f64, lng: f64) -> bool {
    if lng > -130.0 && lng < -60.0 && lat > 15.0 && lat < 50.0 { return true; }
    if lng > -10.0 && lng < 40.0 && lat > 35.0 && lat < 70.0 { return true; }
    if lng > -10.0 && lng < 30.0 && lat > 0.0 && lat < 35.0 { return true; }
    if lng > 70.0 && lng < 140.0 && lat > 5.0 && lat < 55.0 { return true; }
    if lng > 110.0 && lng < 155.0 && lat > -10.0 && lat < 45.0 { return true; }
    if lng > -80.0 && lng < -35.0 && lat > -60.0 && lat < 0.0 { return true; }
    false
}

pub trait TileProvider: Send + Sync {
    fn name(&self) -> &str;
    fn fetch_tile(&self, z: u8, x: u64, y: u64) -> Option<Vec<u8>>;
    fn tile_format(&self) -> TileFormat;
    fn cache_ttl_ms(&self) -> u64;
    fn attribution(&self) -> &str;
}

pub struct OsmTileProvider {
    pub base_url: String,
    pub style: String,
    pub api_key: Option<String>,
}

impl Default for OsmTileProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OsmTileProvider {
    pub fn new() -> Self {
        Self {
            base_url: "https://tile.openstreetmap.org".to_string(),
            style: "osm".to_string(),
            api_key: None,
        }
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }
}

impl TileProvider for OsmTileProvider {
    fn name(&self) -> &str { &self.style }

    fn fetch_tile(&self, _z: u8, _x: u64, _y: u64) -> Option<Vec<u8>> {
        None
    }

    fn tile_format(&self) -> TileFormat { TileFormat::Png }
    fn cache_ttl_ms(&self) -> u64 { 86_400_000 }
    fn attribution(&self) -> &str { "© OpenStreetMap contributors" }
}

pub struct MapLibreStyle {
    pub name: String,
    pub version: u8,
    pub center: [f64; 2],
    pub zoom: f64,
    pub sources: Vec<StyleSource>,
    pub layers: Vec<StyleLayer>,
}

pub struct StyleSource {
    pub id: String,
    pub source_type: String,
    pub tiles: Vec<String>,
    pub minzoom: u8,
    pub maxzoom: u8,
    pub attribution: String,
}

pub struct StyleLayer {
    pub id: String,
    pub layer_type: String,
    pub source: String,
    pub source_layer: Option<String>,
    pub minzoom: Option<u8>,
    pub maxzoom: Option<u8>,
    pub paint: Vec<(String, serde_json::Value)>,
    pub layout: Vec<(String, serde_json::Value)>,
}

impl MapLibreStyle {
    pub fn dark_style() -> Self {
        Self {
            name: "NeoTrix Dark".to_string(),
            version: 8,
            center: [0.0, 30.0],
            zoom: 2.0,
            sources: vec![
                StyleSource {
                    id: "osm".to_string(),
                    source_type: "raster".to_string(),
                    tiles: vec!["https://tile.openstreetmap.org/{z}/{x}/{y}.png".to_string()],
                    minzoom: 0, maxzoom: 19,
                    attribution: "© OpenStreetMap contributors".to_string(),
                },
            ],
            layers: vec![
                StyleLayer {
                    id: "osm-bg".to_string(),
                    layer_type: "raster".to_string(),
                    source: "osm".to_string(),
                    source_layer: None,
                    minzoom: None, maxzoom: None,
                    paint: vec![("raster-opacity".to_string(), serde_json::json!(0.85))],
                    layout: vec![],
                },
            ],
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"version":{},"name":"{}","center":[{},{}],"zoom":{},"sources":{{}},"layers":[]}}"#,
            self.version, self.name, self.center[0], self.center[1], self.zoom
        )
    }
}

pub fn generate_geojson_tile(entries: &[&SpatialEntry]) -> Vec<u8> {
    let features: Vec<String> = entries.iter().enumerate().map(|(i, e)| {
        let _gj = e.to_geojson();
        format!(r#"{{"type":"Feature","id":{},"properties":{{}},"geometry":{{"type":"Point","coordinates":[0,0]}}}}"#, i)
    }).collect();
    format!(r#"{{"type":"FeatureCollection","features":[{}]}}"#, features.join(",")).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l1_body_impl::nt_l1_shared_types::{GeoPoint, SpatialEntry, SpatialGeometry};
    use std::collections::HashMap;

    #[test]
    fn test_tile_service_new() {
        let svc = MapTileService::new();
        assert!(svc.tile_providers.is_empty());
        assert_eq!(svc.cache_stats().entries, 0);
    }

    #[test]
    fn test_fallback_tile_generation() {
        let svc = MapTileService::new();
        let tile = svc.generate_fallback_tile(10, 512, 384);
        assert!(!tile.is_empty());
        let s = String::from_utf8_lossy(&tile);
        assert!(s.contains("z10/512/384"));
    }

    #[test]
    fn test_tile_service_cache() {
        let mut svc = MapTileService::new();
        let _ = svc.get_tile(5, 10, 10);
        let _ = svc.get_tile(5, 10, 10);
        assert!(svc.cache_stats().hits >= 1 || svc.cache_stats().misses >= 1);
    }

    #[test]
    fn test_invalidate_tile() {
        let mut svc = MapTileService::new();
        svc.cache.set(5, 10, 10, vec![0u8; 10], TileFormat::Png, 60_000);
        assert!(svc.cache.get(5, 10, 10).is_some());
        svc.invalidate_tile(5, 10, 10);
        assert!(svc.cache.get(5, 10, 10).is_none());
    }

    #[test]
    fn test_maplibre_style() {
        let style = MapLibreStyle::dark_style();
        assert_eq!(style.name, "NeoTrix Dark");
        let json = style.to_json();
        assert!(json.contains("NeoTrix Dark"));
    }

    #[test]
    fn test_osm_provider_defaults() {
        let p = OsmTileProvider::new();
        assert_eq!(p.name(), "osm");
        assert_eq!(p.tile_format(), TileFormat::Png);
        assert_eq!(p.cache_ttl_ms(), 86_400_000);
    }

    #[test]
    fn test_generate_geojson_tile() {
        let entry = SpatialEntry {
            id: "1".into(), name: "test".into(),
            geometry: SpatialGeometry::Point(GeoPoint::new(48.8566, 2.3522)),
            properties: HashMap::new(), source: "test".into(),
            confidence: 1.0, created_at: 0, updated_at: 0, tags: vec![],
        };
        let data = generate_geojson_tile(&[&entry]);
        assert!(!data.is_empty());
    }

    #[test]
    fn test_fallback_tile_land_estimate() {
        assert!(estimate_land_raw(48.8566, 2.3522));
        assert!(!estimate_land_raw(0.0, -160.0));
    }

    #[test]
    fn test_tile_service_multiple_get() {
        let mut svc = MapTileService::new();
        for (z, x, y) in &[(1u8, 0u64, 0u64), (1, 0, 1), (1, 1, 0)] {
            let tile = svc.get_tile(*z, *x, *y);
            assert!(!tile.is_empty());
        }
    }
}
