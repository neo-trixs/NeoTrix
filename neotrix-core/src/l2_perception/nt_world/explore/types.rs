use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoPoint {
    pub lat: f64,
    pub lng: f64,
}

impl GeoPoint {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat: lat.clamp(-90.0, 90.0), lng: lng.clamp(-180.0, 180.0) }
    }

    pub fn zero() -> Self {
        Self { lat: 0.0, lng: 0.0 }
    }

    pub fn to_mercator(&self) -> (f64, f64) {
        let x = self.lng.to_radians() * EARTH_RADIUS;
        let y = (PI / 4.0 + self.lat.to_radians() / 2.0).tan().ln() * EARTH_RADIUS;
        (x, y)
    }

    pub fn distance_haversine(&self, other: &Self) -> f64 {
        let d_lat = (other.lat - self.lat).to_radians();
        let d_lng = (other.lng - self.lng).to_radians();
        let a = (d_lat / 2.0).sin().powi(2)
            + self.lat.to_radians().cos() * other.lat.to_radians().cos() * (d_lng / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();
        EARTH_RADIUS * c
    }

    pub fn bearing(&self, other: &Self) -> f64 {
        let d_lng = (other.lng - self.lng).to_radians();
        let y = d_lng.sin() * other.lat.to_radians().cos();
        let x = self.lat.to_radians().cos() * other.lat.to_radians().sin()
            - self.lat.to_radians().sin() * other.lat.to_radians().cos() * d_lng.cos();
        y.atan2(x).to_degrees()
    }

    pub fn destination(&self, bearing_deg: f64, distance_m: f64) -> Self {
        let bearing = bearing_deg.to_radians();
        let lat1 = self.lat.to_radians();
        let lng1 = self.lng.to_radians();
        let angular = distance_m / EARTH_RADIUS;
        let lat2 = (lat1.sin() * angular.cos() + lat1.cos() * angular.sin() * bearing.cos()).asin();
        let lng2 = lng1 + bearing.sin().atan2(angular.cos() - lat1.sin() * lat2.sin());
        Self::new(lat2.to_degrees(), lng2.to_degrees())
    }
}

const EARTH_RADIUS: f64 = 6_371_000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min_lat: f64,
    pub min_lng: f64,
    pub max_lat: f64,
    pub max_lng: f64,
}

impl BoundingBox {
    pub fn world() -> Self {
        Self { min_lat: -90.0, min_lng: -180.0, max_lat: 90.0, max_lng: 180.0 }
    }

    pub fn new(min_lat: f64, min_lng: f64, max_lat: f64, max_lng: f64) -> Self {
        Self { min_lat: min_lat.clamp(-90.0, 90.0), min_lng: min_lng.clamp(-180.0, 180.0), max_lat: max_lat.clamp(-90.0, 90.0), max_lng: max_lng.clamp(-180.0, 180.0) }
    }

    pub fn contains(&self, point: &GeoPoint) -> bool {
        point.lat >= self.min_lat && point.lat <= self.max_lat
            && point.lng >= self.min_lng && point.lng <= self.max_lng
    }

    pub fn center(&self) -> GeoPoint {
        GeoPoint::new((self.min_lat + self.max_lat) / 2.0, (self.min_lng + self.max_lng) / 2.0)
    }

    pub fn width_deg(&self) -> f64 {
        self.max_lng - self.min_lng
    }

    pub fn height_deg(&self) -> f64 {
        self.max_lat - self.min_lat
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min_lat: self.min_lat.min(other.min_lat),
            min_lng: self.min_lng.min(other.min_lng),
            max_lat: self.max_lat.max(other.max_lat),
            max_lng: self.max_lng.max(other.max_lng),
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min_lat < other.max_lat && self.max_lat > other.min_lat
            && self.min_lng < other.max_lng && self.max_lng > other.min_lng
    }
}

#[derive(Debug, Clone)]
pub struct GeoJsonFeature {
    pub geometry: GeoJsonGeometry,
    pub properties: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub enum GeoJsonGeometry {
    Point { lat: f64, lng: f64 },
    MultiPoint { points: Vec<GeoPoint> },
    LineString { points: Vec<GeoPoint> },
    MultiLineString { lines: Vec<Vec<GeoPoint>> },
    Polygon { rings: Vec<Vec<GeoPoint>> },
    MultiPolygon { polygons: Vec<Vec<Vec<GeoPoint>>> },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateSystem {
    Wgs84,
    WebMercator,
    Utm { zone: u8, northern: bool },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomLevel(pub u8);

impl ZoomLevel {
    pub fn new(z: u8) -> Self {
        Self(z.min(22))
    }

    pub fn num_tiles(&self) -> u64 {
        1u64 << self.0
    }

    pub fn resolution(&self) -> f64 {
        360.0 / self.num_tiles() as f64 / 256.0
    }

    pub fn min_zoom() -> Self { Self(0) }
    pub fn max_zoom() -> Self { Self(22) }
}

impl From<u8> for ZoomLevel {
    fn from(z: u8) -> Self { Self::new(z) }
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

    pub fn from_latlng(point: &GeoPoint, z: ZoomLevel) -> Self {
        let n = z.num_tiles() as f64;
        let lat_rad = point.lat.to_radians();
        let x = ((point.lng + 180.0) / 360.0 * n) as u64;
        let y = ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / PI) / 2.0 * n) as u64;
        Self { x, y, z }
    }

    pub fn to_bbox(&self) -> BoundingBox {
        let n = self.z.num_tiles() as f64;
        let min_lng = self.x as f64 / n * 360.0 - 180.0;
        let max_lng = (self.x as f64 + 1.0) / n * 360.0 - 180.0;
        let min_lat = (PI * (1.0 - 2.0 * (self.y as f64 + 1.0) / n)).exp().atan().atan() * 2.0 / PI * 180.0;
        let max_lat = (PI * (1.0 - 2.0 * self.y as f64 / n)).exp().atan().atan() * 2.0 / PI * 180.0;
        BoundingBox::new(min_lat, min_lng, max_lat, max_lng)
    }

    pub fn parent(&self) -> Option<Self> {
        if self.z.0 == 0 { return None; }
        Some(Self { x: self.x / 2, y: self.y / 2, z: ZoomLevel(self.z.0 - 1) })
    }

    pub fn children(&self) -> Vec<Self> {
        if self.z.0 >= 22 { return vec![]; }
        let z = ZoomLevel(self.z.0 + 1);
        vec![
            Self { x: self.x * 2, y: self.y * 2, z },
            Self { x: self.x * 2 + 1, y: self.y * 2, z },
            Self { x: self.x * 2, y: self.y * 2 + 1, z },
            Self { x: self.x * 2 + 1, y: self.y * 2 + 1, z },
        ]
    }

    pub fn quadkey(&self) -> String {
        let mut key = String::new();
        for i in (0..self.z.0).rev() {
            let digit = ((self.x >> i) & 1) | (((self.y >> i) & 1) << 1);
            key.push((b'0' + digit as u8) as char);
        }
        key
    }

    pub fn from_quadkey(qk: &str) -> Option<Self> {
        let z = qk.len() as u8;
        // 超过 31 层时 `(z-1-i)` 移位 ≥64 或下溢
        if z > 31 {
            return None;
        }
        let (mut x, mut y) = (0u64, 0u64);
        for (i, c) in qk.chars().enumerate() {
            let digit = c.to_digit(4)?;
            x |= ((digit & 1) as u64) << (z as usize - 1 - i);
            y |= (((digit >> 1) & 1) as u64) << (z as usize - 1 - i);
        }
        Some(Self { x, y, z: ZoomLevel(z) })
    }
}

pub fn geo_json_stringify(features: &[GeoJsonFeature]) -> String {
    let mut parts = Vec::new();
    for f in features {
        let props = f.properties.iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
            .collect::<Vec<_>>().join(",");
        parts.push(format!("\"type\":\"Feature\",\"geometry\":{},\"properties\":{{{}}}",
            geometry_to_string(&f.geometry), props));
    }
    format!("{{\"type\":\"FeatureCollection\",\"features\":[{}]}}", parts.join(","))
}

fn geometry_to_string(g: &GeoJsonGeometry) -> String {
    match g {
        GeoJsonGeometry::Point { lat, lng } => {
            format!("{{\"type\":\"Point\",\"coordinates\":[{},{}]}}", lng, lat)
        }
        GeoJsonGeometry::MultiPoint { points } => {
            let coords: Vec<String> = points.iter().map(|p| format!("[{},{}]", p.lng, p.lat)).collect();
            format!("{{\"type\":\"MultiPoint\",\"coordinates\":[{}]}}", coords.join(","))
        }
        GeoJsonGeometry::LineString { points } => {
            let coords: Vec<String> = points.iter().map(|p| format!("[{},{}]", p.lng, p.lat)).collect();
            format!("{{\"type\":\"LineString\",\"coordinates\":[{}]}}", coords.join(","))
        }
        GeoJsonGeometry::Polygon { rings } => {
            let r: Vec<String> = rings.iter().map(|ring| {
                let coords: Vec<String> = ring.iter().map(|p| format!("[{},{}]", p.lng, p.lat)).collect();
                format!("[{}]", coords.join(","))
            }).collect();
            format!("{{\"type\":\"Polygon\",\"coordinates\":[{}]}}", r.join(","))
        }
        _ => "{}".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_point_new_clamps() {
        let p = GeoPoint::new(100.0, 200.0);
        assert!((p.lat - 90.0).abs() < 1e-10);
        assert!((p.lng - 180.0).abs() < 1e-10);
    }

    #[test]
    fn test_haversine_distance() {
        let tokyo = GeoPoint::new(35.6762, 139.6503);
        let osaka = GeoPoint::new(34.6937, 135.5023);
        let dist = tokyo.distance_haversine(&osaka);
        assert!(dist > 390_000.0 && dist < 420_000.0);
    }

    #[test]
    fn test_bearing() {
        let nyc = GeoPoint::new(40.7128, -74.0060);
        let london = GeoPoint::new(51.5074, -0.1278);
        let b = nyc.bearing(&london);
        assert!(b > 30.0 && b < 80.0);
    }

    #[test]
    fn test_bounding_box_contains() {
        let bb = BoundingBox::new(30.0, -10.0, 50.0, 20.0);
        assert!(bb.contains(&GeoPoint::new(40.0, 5.0)));
        assert!(!bb.contains(&GeoPoint::new(60.0, 5.0)));
    }

    #[test]
    fn test_bounding_box_union() {
        let a = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
        let b = BoundingBox::new(5.0, 5.0, 20.0, 20.0);
        let u = a.union(&b);
        assert!((u.min_lat - 0.0).abs() < 1e-10);
        assert!((u.max_lat - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_tile_coord_from_latlng() {
        let p = GeoPoint::new(0.0, 0.0);
        let t = TileCoord::from_latlng(&p, ZoomLevel(1));
        assert_eq!(t.x, 1);
        assert_eq!(t.y, 1);
    }

    #[test]
    fn test_tile_coord_quadkey_roundtrip() {
        let p = GeoPoint::new(40.7128, -74.0060);
        let t = TileCoord::from_latlng(&p, ZoomLevel(12));
        let qk = t.quadkey();
        let t2 = TileCoord::from_quadkey(&qk).unwrap();
        assert_eq!(t.x, t2.x);
        assert_eq!(t.y, t2.y);
        assert_eq!(t.z.0, t2.z.0);
    }

    #[test]
    fn test_tile_children_count() {
        let t = TileCoord::new(0, 0, ZoomLevel(0)).unwrap();
        let children = t.children();
        assert_eq!(children.len(), 4);
    }

    #[test]
    fn test_tile_parent() {
        let t = TileCoord::new(3, 3, ZoomLevel(3)).unwrap();
        let p = t.parent().unwrap();
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 1);
        assert_eq!(p.z.0, 2);
    }

    #[test]
    fn test_mercator_projection_roundtrip() {
        let p = GeoPoint::new(48.8566, 2.3522);
        let (mx, my) = p.to_mercator();
        assert!(mx > 0.0);
        assert!(my > 0.0);
    }

    #[test]
    fn test_geo_json_stringify_point() {
        let f = GeoJsonFeature {
            geometry: GeoJsonGeometry::Point { lat: 48.8566, lng: 2.3522 },
            properties: vec![("name".to_string(), "Paris".to_string())],
        };
        let json = geo_json_stringify(&[f]);
        assert!(json.contains("Paris"));
        assert!(json.contains("Point"));
    }

    #[test]
    fn test_zoom_level_resolution() {
        let z0 = ZoomLevel(0);
        assert!((z0.resolution() - 1.40625).abs() < 1e-5);
        let z10 = ZoomLevel(10);
        assert!(z10.resolution() < z0.resolution());
    }
}
