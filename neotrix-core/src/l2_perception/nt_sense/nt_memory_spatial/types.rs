use std::collections::HashMap;

/// Local types to break L3→L2 upward dependency on nt_world_map::types
#[derive(Debug, Clone)]
pub struct GeoPoint {
    pub lat: f64,
    pub lng: f64,
}

impl GeoPoint {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }

    pub fn distance_haversine(&self, other: &GeoPoint) -> f64 {
        const EARTH_RADIUS: f64 = 6_371_000.0;
        let d_lat = (other.lat - self.lat).to_radians();
        let d_lng = (other.lng - self.lng).to_radians();
        let a = (d_lat / 2.0).sin().powi(2)
            + self.lat.to_radians().cos()
            * other.lat.to_radians().cos()
            * (d_lng / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();
        EARTH_RADIUS * c
    }
}

#[derive(Debug, Clone)]
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

    pub fn world() -> Self {
        Self { min_lat: -90.0, min_lng: -180.0, max_lat: 90.0, max_lng: 180.0 }
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min_lat: self.min_lat.min(other.min_lat),
            min_lng: self.min_lng.min(other.min_lng),
            max_lat: self.max_lat.max(other.max_lat),
            max_lng: self.max_lng.max(other.max_lng),
        }
    }

    pub fn contains(&self, point: &GeoPoint) -> bool {
        point.lat >= self.min_lat && point.lat <= self.max_lat
            && point.lng >= self.min_lng && point.lng <= self.max_lng
    }

    pub fn center(&self) -> GeoPoint {
        GeoPoint::new(
            (self.min_lat + self.max_lat) / 2.0,
            (self.min_lng + self.max_lng) / 2.0,
        )
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min_lat <= other.max_lat
            && self.max_lat >= other.min_lat
            && self.min_lng <= other.max_lng
            && self.max_lng >= other.min_lng
    }
}

#[derive(Debug, Clone)]
pub enum GeoJsonGeometry {
    Point { lat: f64, lng: f64 },
    LineString { points: Vec<GeoPoint> },
    Polygon { rings: Vec<Vec<GeoPoint>> },
}

#[derive(Debug, Clone)]
pub struct GeoJsonFeature {
    pub geometry: GeoJsonGeometry,
    pub properties: HashMap<String, String>,
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

#[derive(Debug, Clone)]
pub enum SpatialGeometry {
    Point(GeoPoint),
    Line(Vec<GeoPoint>),
    Polygon(Vec<Vec<GeoPoint>>),
    MultiGeometry(Vec<SpatialGeometry>),
}

impl SpatialEntry {
    pub fn bbox(&self) -> BoundingBox {
        match &self.geometry {
            SpatialGeometry::Point(p) => BoundingBox::new(p.lat, p.lng, p.lat, p.lng),
            SpatialGeometry::Line(pts) => {
                if pts.is_empty() {
                    return BoundingBox::world();
                }
                let mut bb = BoundingBox::new(pts[0].lat, pts[0].lng, pts[0].lat, pts[0].lng);
                for p in pts { bb = bb.union(&BoundingBox::new(p.lat, p.lng, p.lat, p.lng)); }
                bb
            }
            SpatialGeometry::Polygon(rings) => {
                let Some(pts) = rings.first() else {
                    return BoundingBox::world();
                };
                if pts.is_empty() {
                    return BoundingBox::world();
                }
                let mut bb = BoundingBox::new(pts[0].lat, pts[0].lng, pts[0].lat, pts[0].lng);
                for p in pts { bb = bb.union(&BoundingBox::new(p.lat, p.lng, p.lat, p.lng)); }
                bb
            }
            SpatialGeometry::MultiGeometry(geoms) => {
                let mut bb = BoundingBox::world();
                for g in geoms {
                    let entry = SpatialEntry { geometry: g.clone(), ..self.clone() };
                    bb = bb.union(&entry.bbox());
                }
                bb
            }
        }
    }

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

#[derive(Debug, Clone)]
pub struct TileCacheEntry {
    pub data: Vec<u8>,
    pub format: TileFormat,
    pub cached_at: u64,
    pub ttl_ms: u64,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileFormat {
    Mvt,
    Png,
    Jpeg,
    Webp,
}

#[derive(Debug, Clone)]
pub struct TileCacheStats {
    pub entries: usize,
    pub total_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

#[derive(Debug, Clone)]
pub struct SpatialQuery {
    pub bbox: Option<BoundingBox>,
    pub center: Option<(GeoPoint, f64)>,
    pub tags: Vec<String>,
    pub source: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

impl Default for SpatialQuery {
    fn default() -> Self {
        Self { bbox: None, center: None, tags: vec![], source: None, limit: 100, offset: 0 }
    }
}

pub fn tile_key(z: u8, x: u64, y: u64) -> String {
    format!("tile/{}/{}/{}", z, x, y)
}

pub fn geohash_encode(point: &GeoPoint, precision: usize) -> String {
    let lat_range = (-90.0, 90.0);
    let lng_range = (-180.0, 180.0);
    let base32 = "0123456789bcdefghjkmnpqrstuvwxyz";
    let mut hash = String::new();
    let mut bits = 0u8;
    let mut bit_count = 0;
    let mut even = true;
    let (mut min_lat, mut max_lat) = lat_range;
    let (mut min_lng, mut max_lng) = lng_range;
    while hash.len() < precision {
        if even {
            let mid = (min_lng + max_lng) / 2.0;
            if point.lng >= mid {
                bits = (bits << 1) | 1;
                min_lng = mid;
            } else {
                bits <<= 1;
                max_lng = mid;
            }
        } else {
            let mid = (min_lat + max_lat) / 2.0;
            if point.lat >= mid {
                bits = (bits << 1) | 1;
                min_lat = mid;
            } else {
                bits <<= 1;
                max_lat = mid;
            }
        }
        bit_count += 1;
        even = !even;
        if bit_count == 5 {
            if let Some(c) = base32.chars().nth(bits as usize) {
                hash.push(c);
            }
            bits = 0;
            bit_count = 0;
        }
    }
    hash
}

pub fn geohash_decode(hash: &str) -> Option<BoundingBox> {
    let base32 = "0123456789bcdefghjkmnpqrstuvwxyz";
    let (mut min_lat, mut max_lat) = (-90.0, 90.0);
    let (mut min_lng, mut max_lng) = (-180.0, 180.0);
    let mut even = true;
    for c in hash.chars() {
        let idx = base32.find(c)?;
        for i in (0..5).rev() {
            let bit = (idx >> i) & 1;
            if even {
                let mid = (min_lng + max_lng) / 2.0;
                if bit == 1 { min_lng = mid; } else { max_lng = mid; }
            } else {
                let mid = (min_lat + max_lat) / 2.0;
                if bit == 1 { min_lat = mid; } else { max_lat = mid; }
            }
            even = !even;
        }
    }
    Some(BoundingBox::new(min_lat, min_lng, max_lat, max_lng))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geohash_roundtrip() {
        let p = GeoPoint::new(40.7128, -74.0060);
        let hash = geohash_encode(&p, 8);
        assert_eq!(hash.len(), 8);
        let bb = geohash_decode(&hash).unwrap();
        assert!(bb.contains(&p));
    }

    #[test]
    fn test_geohash_precision() {
        let p = GeoPoint::new(0.0, 0.0);
        let h1 = geohash_encode(&p, 1);
        let h6 = geohash_encode(&p, 6);
        assert!(h6.len() > h1.len());
    }

    #[test]
    fn test_spatial_entry_bbox_point() {
        let entry = SpatialEntry {
            id: "test".into(), name: "Test".into(),
            geometry: SpatialGeometry::Point(GeoPoint::new(40.0, -74.0)),
            properties: HashMap::new(), source: "test".into(),
            confidence: 1.0, created_at: 0, updated_at: 0, tags: vec![],
        };
        let bb = entry.bbox();
        assert!((bb.min_lat - 40.0).abs() < 1e-10);
        assert!((bb.min_lng + 74.0).abs() < 1e-10);
    }

    #[test]
    fn test_tile_key_format() {
        let key = tile_key(10, 512, 384);
        assert_eq!(key, "tile/10/512/384");
    }

    #[test]
    fn test_spatial_entry_to_geojson() {
        let entry = SpatialEntry {
            id: "1".into(), name: "Test".into(),
            geometry: SpatialGeometry::Point(GeoPoint::new(48.8566, 2.3522)),
            properties: HashMap::new(), source: "test".into(),
            confidence: 1.0, created_at: 0, updated_at: 0, tags: vec![],
        };
        let gj = entry.to_geojson();
        match gj.geometry {
            GeoJsonGeometry::Point { lat, lng } => {
                assert!((lat - 48.8566).abs() < 0.001);
                assert!((lng - 2.3522).abs() < 0.001);
            }
            _ => panic!("expected point"),
        }
    }

    #[test]
    fn test_spatial_entry_bbox_empty_geometry_no_panic() {
        // Regression: Line(pts) and Polygon(rings) indexed [0] on empty
        // input, panicking for arbitrary user/import geometry. Both now
        // fall back to BoundingBox::world().
        let mk = |geometry: SpatialGeometry| SpatialEntry {
            id: "e".into(), name: "E".into(),
            geometry, properties: HashMap::new(), source: "test".into(),
            confidence: 1.0, created_at: 0, updated_at: 0, tags: vec![],
        };
        let line = mk(SpatialGeometry::Line(vec![]));
        assert_eq!(line.bbox().min_lat, -90.0);
        assert_eq!(line.bbox().max_lat, 90.0);

        let poly_empty_rings = mk(SpatialGeometry::Polygon(vec![]));
        assert_eq!(poly_empty_rings.bbox().min_lng, -180.0);

        let poly_empty_ring = mk(SpatialGeometry::Polygon(vec![vec![]]));
        assert_eq!(poly_empty_ring.bbox().min_lng, -180.0);
    }
}
