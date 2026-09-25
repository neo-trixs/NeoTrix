//! nt_memory_geo — 地理索引层门面：子模块重导出，行为零变更。
//!
//! 地球知识世界仿真的索引层：把 KB 知识节点按地理标签整合，供地图渲染与地理检索使用。
//! 拆分：nt_geo_types / nt_geo_base / nt_geo_elevation / nt_geo_weather / nt_geo_trajectory / nt_geo_volcano / nt_geo_ntpack_cold + tests。

pub mod nt_geo_base;
pub mod nt_geo_elevation;
pub mod nt_geo_ntpack_cold;
pub mod nt_geo_trajectory;
pub mod nt_geo_types;
pub mod nt_geo_volcano;
pub mod nt_geo_weather;

pub use nt_geo_base::{
    export_geojson, geo_coverage_report, geo_linked_nodes, geo_stats, geo_tag_cities,
    geo_tag_nodes, query_bbox, query_by_place, upsert_geo, CITY_LATLNG, COUNTRY_CAPITALS,
};
pub use nt_geo_elevation::{ensure_elevation_table, fetch_elevations, query_elevations};
pub use nt_geo_ntpack_cold::{
    append_geo_ntpack, archive_geo_cold, export_geo_ntpack, geo_cold_layers, geo_layer_inventory,
    import_geo_ntpack, import_geo_ntpack_to_kb, query_bbox_with_cold, query_by_place_with_cold,
};
pub use nt_geo_trajectory::{ensure_trajectory_table, insert_trajectory, query_trajectories};
pub use nt_geo_types::{GeoRecord, TrajectoryRow, WeatherRecord};
pub use nt_geo_volcano::ingest_geo_volcanoes;
pub use nt_geo_weather::{ensure_weather_table, fetch_weather_snapshot, query_weather};

#[cfg(test)]
mod tests;
