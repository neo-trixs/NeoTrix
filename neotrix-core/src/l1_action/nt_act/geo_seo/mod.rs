//! GEO/SEO 通用能力模块
//!
//! 提供跨域复用的地理位置、搜索引擎优化等基础类型。
//! 外贸Agent、内容Agent、营销Agent等均可复用此模块。

#![forbid(unsafe_code)]

pub mod geo;
pub mod search_engine;
pub mod visibility;

pub use geo::{GeoLocation, GeoRegion, LocationType};
pub use search_engine::{SearchEngine, SearchEngineType, SearchResult};
pub use visibility::{VisibilityMetric, VisibilityReport, VisibilityScore};

