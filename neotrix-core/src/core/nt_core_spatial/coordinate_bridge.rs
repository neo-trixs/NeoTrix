#![allow(dead_code)]
use crate::core::nt_core_hcube::spatial_scene::SpatialSceneEngine;
use crate::core::nt_core_hcube::vsa_spatial_encoder::{VSASpatialEncoder, Vec3D};
use crate::neotrix::nt_memory_kb::spatial_graph::GeoCoord;

/// Whether a coordinate is geographic (lat/lng) or local Cartesian.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordType {
    Geographic,
    Local,
    Unknown,
}

/// Triple representation of a coordinate in all supported formats.
#[derive(Debug, Clone)]
pub struct CoordinateTriple {
    pub label: String,
    pub coord_type: CoordType,
    pub vec3d: Vec3D,
    pub geo: Option<GeoCoord>,
    pub ssp: Vec<u8>,
    pub vsa: Vec<u8>,
}

const SSP_DIM: usize = 1024;
const VSA_DIM: usize = 4096;
const DEGREE_TO_METER: f64 = 111_320.0;

/// Conversion between coordinate systems used by SpatialReasoner.
pub trait CoordinateConversion {
    fn to_vec3d(&self, origin: Option<&GeoCoord>) -> Vec3D;
    fn to_geo_coord(&self, origin: Option<&GeoCoord>) -> Option<GeoCoord>;
    fn encode_ssp(&self, engine: &SpatialSceneEngine, origin: Option<&GeoCoord>) -> Vec<u8>;
    fn encode_vsa(&self, encoder: &VSASpatialEncoder, origin: Option<&GeoCoord>) -> Vec<u8>;
    fn coordinate_label(&self) -> String;
}

impl CoordinateConversion for Vec3D {
    fn to_vec3d(&self, _origin: Option<&GeoCoord>) -> Vec3D {
        *self
    }

    fn to_geo_coord(&self, origin: Option<&GeoCoord>) -> Option<GeoCoord> {
        let origin = origin?;
        let lat = origin.lat + (self.z / DEGREE_TO_METER);
        let lng = origin.lng + (self.x / (DEGREE_TO_METER * origin.lat.to_radians().cos()));
        let alt = origin.alt.map(|a| a + self.y);
        Some(GeoCoord { lat, lng, alt })
    }

    fn encode_ssp(&self, _engine: &SpatialSceneEngine, _origin: Option<&GeoCoord>) -> Vec<u8> {
        SpatialSceneEngine::encode_position(self.x as f32, self.y as f32, self.z as f32)
    }

    fn encode_vsa(&self, encoder: &VSASpatialEncoder, _origin: Option<&GeoCoord>) -> Vec<u8> {
        encoder.encode(self)
    }

    fn coordinate_label(&self) -> String {
        format!("({:.2}, {:.2}, {:.2})", self.x, self.y, self.z)
    }
}

impl CoordinateConversion for GeoCoord {
    fn to_vec3d(&self, origin: Option<&GeoCoord>) -> Vec3D {
        match origin {
            Some(o) => {
                let x = (self.lng - o.lng) * DEGREE_TO_METER * o.lat.to_radians().cos();
                let z = (self.lat - o.lat) * DEGREE_TO_METER;
                let y = self.alt.unwrap_or(0.0) - o.alt.unwrap_or(0.0);
                Vec3D { x, y, z }
            }
            None => Vec3D::origin(),
        }
    }

    fn to_geo_coord(&self, _origin: Option<&GeoCoord>) -> Option<GeoCoord> {
        Some(*self)
    }

    fn encode_ssp(&self, engine: &SpatialSceneEngine, origin: Option<&GeoCoord>) -> Vec<u8> {
        let v = self.to_vec3d(origin);
        v.encode_ssp(engine, origin)
    }

    fn encode_vsa(&self, encoder: &VSASpatialEncoder, origin: Option<&GeoCoord>) -> Vec<u8> {
        let v = self.to_vec3d(origin);
        v.encode_vsa(encoder, origin)
    }

    fn coordinate_label(&self) -> String {
        format!(
            "({:.4}°, {:.4}°, {:.0}m)",
            self.lat,
            self.lng,
            self.alt.unwrap_or(0.0)
        )
    }
}

/// Holds an optional origin GeoCoord for relative↔absolute coordinate conversion.
/// Auto-detects whether coordinates are in geo or local space.
#[derive(Debug, Clone)]
pub struct CoordinateBridge {
    pub origin: Option<GeoCoord>,
    scene_engine: SpatialSceneEngine,
    vsa_encoder: VSASpatialEncoder,
}

impl CoordinateBridge {
    pub fn new(origin: Option<GeoCoord>) -> Self {
        Self {
            origin,
            scene_engine: SpatialSceneEngine::new(SSP_DIM),
            vsa_encoder: VSASpatialEncoder::new(0.01, 32),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(None)
    }

    /// Auto-detect if a string contains geo coordinates (lat/lng pattern).
    pub fn detect_coord_type(text: &str) -> CoordType {
        let lower = text.to_lowercase();
        let geo_indicators = [
            "lat",
            "lng",
            "lat:",
            "lon:",
            "°",
            "north",
            "south",
            "east",
            "west",
            "coordinates",
            "geo",
            "latitude",
            "longitude",
            "deg",
        ];
        let local_indicators = [
            "position:",
            "location:",
            "coord:",
            "3d",
            "cartesian",
            "local",
            "x:",
            "y:",
            "z:",
            "vec3",
        ];

        let has_geo = geo_indicators.iter().any(|p| lower.contains(p));
        let has_local = local_indicators.iter().any(|p| lower.contains(p));

        if has_geo && !has_local {
            return CoordType::Geographic;
        }
        if has_local && !has_geo {
            return CoordType::Local;
        }

        CoordType::Unknown
    }

    /// Bridge a coordinate through all 3 representations.
    pub fn bridge<C: CoordinateConversion>(&self, coord: &C) -> CoordinateTriple {
        let label = coord.coordinate_label();
        let vec3d = coord.to_vec3d(self.origin.as_ref());
        let geo = coord.to_geo_coord(self.origin.as_ref());
        let ssp = coord.encode_ssp(&self.scene_engine, self.origin.as_ref());
        let vsa = coord.encode_vsa(&self.vsa_encoder, self.origin.as_ref());
        let coord_type = match (&vec3d, &geo) {
            _ if geo.is_some() && vec3d.x.abs() > 1.0 => CoordType::Geographic,
            _ if vec3d.x.abs() < 1.0e-6 && vec3d.y.abs() < 1.0e-6 && vec3d.z.abs() < 1.0e-6 => {
                CoordType::Local
            }
            _ => CoordType::Local,
        };
        CoordinateTriple {
            label,
            coord_type,
            vec3d,
            geo,
            ssp,
            vsa,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_origin() -> GeoCoord {
        GeoCoord::new(39.9, 116.4)
    }

    #[test]
    fn test_vec3d_ssp_roundtrip() {
        let v = Vec3D::new(10.0, 20.0, 30.0);
        let engine = SpatialSceneEngine::new(SSP_DIM);
        let ssp = v.encode_ssp(&engine, None);
        assert_eq!(ssp.len(), SSP_DIM);
    }

    #[test]
    fn test_vec3d_vsa_roundtrip() {
        let v = Vec3D::new(10.0, 20.0, 30.0);
        let encoder = VSASpatialEncoder::new(0.01, 32);
        let vsa = v.encode_vsa(&encoder, None);
        assert_eq!(vsa.len(), VSA_DIM);
    }

    #[test]
    fn test_geo_to_vec3d_conversion() {
        let origin = test_origin();
        let geo = GeoCoord::with_alt(40.9, 117.4, 100.0);
        let v = geo.to_vec3d(Some(&origin));

        let expected_z = 1.0 * DEGREE_TO_METER;
        let expected_x = 1.0 * DEGREE_TO_METER * origin.lat.to_radians().cos();
        assert!(
            (v.x - expected_x).abs() < 1.0,
            "x={} expected={}",
            v.x,
            expected_x
        );
        assert!(
            (v.z - expected_z).abs() < 1.0,
            "z={} expected={}",
            v.z,
            expected_z
        );
        assert!((v.y - 100.0).abs() < 0.01, "y={} expected=100.0", v.y);
    }

    #[test]
    fn test_vec3d_to_geo_with_origin() {
        let origin = test_origin();
        let lat_offset = 1.0_f64.to_radians();
        let expected_x = 1.0 * DEGREE_TO_METER * origin.lat.to_radians().cos();
        let expected_z = 1.0 * DEGREE_TO_METER;
        let v = Vec3D::new(expected_x, 100.0, expected_z);
        let geo = v.to_geo_coord(Some(&origin)).unwrap();
        assert!(
            (geo.lat - 40.9).abs() < 0.001,
            "lat={} expected=40.9",
            geo.lat
        );
        assert!(
            (geo.lng - 117.4).abs() < 0.001,
            "lng={} expected=117.4",
            geo.lng
        );
        assert_eq!(geo.alt, Some(100.0));
    }

    #[test]
    fn test_vec3d_to_geo_no_origin() {
        let v = Vec3D::new(100.0, 200.0, 300.0);
        let geo = v.to_geo_coord(None);
        assert!(geo.is_none());
    }

    #[test]
    fn test_geo_to_vec3d_no_origin() {
        let geo = GeoCoord::new(40.0, 120.0);
        let v = geo.to_vec3d(None);
        assert!((v.x - 0.0).abs() < 1e-10);
        assert!((v.y - 0.0).abs() < 1e-10);
        assert!((v.z - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_coordinate_label_vec3d() {
        let v = Vec3D::new(1.234, 5.678, 9.012);
        let label = v.coordinate_label();
        assert_eq!(label, "(1.23, 5.68, 9.01)");
    }

    #[test]
    fn test_coordinate_label_geo() {
        let geo = GeoCoord::new(39.9042, 116.4074);
        let label = geo.coordinate_label();
        assert_eq!(label, "(39.9042°, 116.4074°, 0m)");
    }

    #[test]
    fn test_coordinate_label_geo_with_alt() {
        let geo = GeoCoord::with_alt(27.9881, 86.9250, 8848.86);
        let label = geo.coordinate_label();
        assert_eq!(label, "(27.9881°, 86.9250°, 8849m)");
    }

    #[test]
    fn test_bridge_vec3d() {
        let origin = test_origin();
        let bridge = CoordinateBridge::new(Some(origin));
        let v = Vec3D::new(5000.0, 200.0, 3000.0);
        let triple = bridge.bridge(&v);

        assert_eq!(triple.label, "(5000.00, 200.00, 3000.00)");
        assert_eq!(triple.vec3d, v);
        assert!(triple.geo.is_some());
        assert_eq!(triple.ssp.len(), SSP_DIM);
        assert_eq!(triple.vsa.len(), VSA_DIM);

        let geo = triple.geo.unwrap();
        let expected_lat = origin.lat + (3000.0 / DEGREE_TO_METER);
        let expected_lng =
            origin.lng + (5000.0 / (DEGREE_TO_METER * origin.lat.to_radians().cos()));
        assert!((geo.lat - expected_lat).abs() < 0.01);
        assert!((geo.lng - expected_lng).abs() < 0.01);
    }

    #[test]
    fn test_detect_coord_type() {
        assert_eq!(
            CoordinateBridge::detect_coord_type("latitude: 39.9, longitude: 116.4"),
            CoordType::Geographic
        );
        assert_eq!(
            CoordinateBridge::detect_coord_type("position: 10 20 30"),
            CoordType::Local
        );
        assert_eq!(
            CoordinateBridge::detect_coord_type("hello world"),
            CoordType::Unknown
        );
    }

    #[test]
    fn test_geo_roundtrip_with_origin() {
        let origin = GeoCoord::new(35.0, 135.0);
        let original = GeoCoord::with_alt(35.5, 135.5, 50.0);

        let v = original.to_vec3d(Some(&origin));
        let recovered = v.to_geo_coord(Some(&origin)).unwrap();

        assert!((recovered.lat - original.lat).abs() < 0.01);
        assert!((recovered.lng - original.lng).abs() < 0.01);
        assert!((recovered.alt.unwrap_or(0.0) - original.alt.unwrap_or(0.0)).abs() < 0.1);
    }

    #[test]
    fn test_vec3d_identity() {
        let v = Vec3D::new(7.0, 8.0, 9.0);
        let origin = GeoCoord::new(0.0, 0.0);
        let same = v.to_vec3d(Some(&origin));
        assert_eq!(v, same);
    }

    #[test]
    fn test_geo_to_geo_identity() {
        let g = GeoCoord::new(45.0, 90.0);
        let origin = GeoCoord::new(0.0, 0.0);
        let same = g.to_geo_coord(Some(&origin)).unwrap();
        assert_eq!(g, same);
    }

    #[test]
    fn test_bridge_geo() {
        let origin = test_origin();
        let bridge = CoordinateBridge::new(Some(origin));
        let geo = GeoCoord::with_alt(45.0, 100.0, 500.0);
        let triple = bridge.bridge(&geo);

        assert_eq!(triple.label, "(45.0000°, 100.0000°, 500m)");
        assert!(triple.geo.is_some());
        assert_eq!(triple.geo.unwrap(), geo);
        assert_eq!(triple.ssp.len(), SSP_DIM);
        assert_eq!(triple.vsa.len(), VSA_DIM);
    }

    #[test]
    fn test_bridge_geo_no_origin() {
        let bridge = CoordinateBridge::with_defaults();
        let geo = GeoCoord::new(45.0, 90.0);
        let triple = bridge.bridge(&geo);

        assert!(triple.geo.is_some());
        assert_eq!(triple.vec3d, Vec3D::origin());
    }
}
