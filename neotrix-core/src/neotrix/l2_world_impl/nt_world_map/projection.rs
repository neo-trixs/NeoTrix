use crate::neotrix::l2_world_impl::nt_world_map::types::{GeoPoint, BoundingBox};
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Projection {
    #[default]
    WebMercator,
    Equirectangular,
    Robinson,
    Orthographic { center_lat: f64, center_lng: f64 },
}

pub trait Project: Send + Sync {
    fn project(&self, point: &GeoPoint) -> (f64, f64);
    fn unproject(&self, x: f64, y: f64) -> GeoPoint;
    fn bounds(&self) -> BoundingBox;
    fn name(&self) -> &'static str;
}

pub struct WebMercator;

impl Project for WebMercator {
    fn project(&self, point: &GeoPoint) -> (f64, f64) {
        let x = (point.lng + 180.0) / 360.0;
        let lat_rad = point.lat.to_radians();
        let y = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / PI) / 2.0;
        (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))
    }

    fn unproject(&self, x: f64, y: f64) -> GeoPoint {
        let lng = x * 360.0 - 180.0;
        let lat_rad = (PI * (1.0 - 2.0 * y.clamp(0.0, 1.0))).exp().atan() * 2.0 - PI / 2.0;
        GeoPoint::new(lat_rad.to_degrees(), lng)
    }

    fn bounds(&self) -> BoundingBox {
        BoundingBox::new(-85.051129, -180.0, 85.051129, 180.0)
    }

    fn name(&self) -> &'static str { "WebMercator" }
}

pub struct Equirectangular;

impl Project for Equirectangular {
    fn project(&self, point: &GeoPoint) -> (f64, f64) {
        ((point.lng + 180.0) / 360.0, (point.lat + 90.0) / 180.0)
    }

    fn unproject(&self, x: f64, y: f64) -> GeoPoint {
        GeoPoint::new(y * 180.0 - 90.0, x * 360.0 - 180.0)
    }

    fn bounds(&self) -> BoundingBox { BoundingBox::world() }
    fn name(&self) -> &'static str { "Equirectangular" }
}

pub struct Orthographic {
    center_lat: f64,
    center_lng: f64,
}

impl Orthographic {
    pub fn new(center_lat: f64, center_lng: f64) -> Self {
        Self { center_lat, center_lng }
    }
}

impl Project for Orthographic {
    fn project(&self, point: &GeoPoint) -> (f64, f64) {
        let d_lng = (point.lng - self.center_lng).to_radians();
        let lat_rad = point.lat.to_radians();
        let c_lat = self.center_lat.to_radians();
        let cos_c = c_lat.sin() * lat_rad.sin() + c_lat.cos() * lat_rad.cos() * d_lng.cos();
        if cos_c < 0.0 { return (-1.0, -1.0); }
        let x = lat_rad.cos() * d_lng.sin();
        let y = c_lat.cos() * lat_rad.sin() - c_lat.sin() * lat_rad.cos() * d_lng.cos();
        (x * 0.5 + 0.5, y * 0.5 + 0.5)
    }

    fn unproject(&self, x: f64, y: f64) -> GeoPoint {
        let nx = (x - 0.5) * 2.0;
        let ny = (y - 0.5) * 2.0;
        let r = (nx * nx + ny * ny).sqrt();
        if r > 1.0 { return GeoPoint::zero(); }
        let c = r.asin();
        let c_lat = self.center_lat.to_radians();
        let lat = (c.cos() * c_lat.sin() + ny * c.sin() * c_lat.cos() / r).asin();
        let lng = self.center_lng + (nx * c.sin() / (r * c_lat.cos())).asin().to_degrees();
        GeoPoint::new(lat.to_degrees(), lng)
    }

    fn bounds(&self) -> BoundingBox { BoundingBox::world() }
    fn name(&self) -> &'static str { "Orthographic" }
}

pub struct ProjectionSet {
    projections: Vec<Box<dyn Project>>,
}

impl ProjectionSet {
    pub fn default_set() -> Self {
        Self {
            projections: vec![
                Box::new(WebMercator),
                Box::new(Equirectangular),
                Box::new(Orthographic::new(0.0, 0.0)),
            ],
        }
    }

    pub fn by_name(&self, name: &str) -> Option<&dyn Project> {
        self.projections.iter().find(|p| p.name() == name).map(Box::as_ref)
    }

    pub fn all(&self) -> &[Box<dyn Project>] {
        &self.projections
    }
}

pub fn tile_scale_factor(lat: f64, zoom: u8) -> f64 {
    let lat_rad = lat.to_radians();
    let n = (1u64 << zoom) as f64;
    EARTH_CIRCUMFERENCE * lat_rad.cos() / n / 256.0
}

const EARTH_CIRCUMFERENCE: f64 = 40_075_016.686;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_mercator_roundtrip() {
        let p = GeoPoint::new(40.0, -74.0);
        let merc = WebMercator;
        let (x, y) = merc.project(&p);
        let p2 = merc.unproject(x, y);
        assert!((p.lng - p2.lng).abs() < 0.001);
        assert!(p2.lat > -90.0 && p2.lat < 90.0);
    }

    #[test]
    fn test_equirectangular_roundtrip() {
        let p = GeoPoint::new(40.0, -74.0);
        let eq = Equirectangular;
        let (x, y) = eq.project(&p);
        let p2 = eq.unproject(x, y);
        assert!((p.lat - p2.lat).abs() < 1e-10);
        assert!((p.lng - p2.lng).abs() < 1e-10);
    }

    #[test]
    fn test_mercator_bounds() {
        let merc = WebMercator;
        let b = merc.bounds();
        assert!(b.min_lat > -90.0);
        assert!(b.max_lat < 90.0);
    }

    #[test]
    fn test_projections_set() {
        let set = ProjectionSet::default_set();
        assert_eq!(set.all().len(), 3);
        assert!(set.by_name("WebMercator").is_some());
        assert!(set.by_name("Orthographic").is_some());
    }

    #[test]
    fn test_tile_scale_factor() {
        let scale = tile_scale_factor(0.0, 10);
        assert!(scale > 0.0);
        let scale_high = tile_scale_factor(0.0, 15);
        assert!(scale_high < scale);
    }
}
