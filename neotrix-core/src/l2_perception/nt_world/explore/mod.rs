pub mod types;
pub mod projection;
pub mod spatial;
pub mod geocode;
pub mod system_scanner;
pub mod cache_detector;
pub mod large_file_finder;

pub use types::{
    GeoPoint, BoundingBox, GeoJsonFeature, GeoJsonGeometry,
    CoordinateSystem, ZoomLevel, TileCoord,
};
pub use projection::{
    Projection, Project, WebMercator, Equirectangular, Orthographic,
    ProjectionSet, tile_scale_factor,
};
pub use spatial::{
    SpatialQuery, SpatialIndex, GridIndex, cluster_points,
};
pub use geocode::{
    Geocoder, BuiltinGeocoder, CompositeGeocoder,
    GeocodingResult, ReverseGeocodingResult, format_coordinate,
};
pub use system_scanner::{ScanPath, SystemScanner, ScanCategory, ScanRiskLevel, ScanResult, calculate_age_days};
pub use cache_detector::{CacheDetector, CacheType, CacheInfo};
pub use large_file_finder::LargeFileFinder;
